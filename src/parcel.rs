use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "parcel";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = Some(3);

const DEFAULT_PALETTE: [&str; 3] = ["#e9f5db", "#e63946", "#2b2d42"];

const CELLS: Param = Param::new(SLUG, "cells", 8, 28, 1);
const COVER: Param = Param::new(SLUG, "cover", 20, 80, 100);
const CHUNK: Param = Param {
    step: 5,
    ..Param::new(SLUG, "chunk", 50, 200, 100)
};
const GRIDS: Param = Param::new(SLUG, "grids", 0, 8, 1);

pub(crate) const PARAMS: &[Param] = &[CELLS, COVER, CHUNK, GRIDS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineBlend {
    Multiply,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParcelParams {
    cells: u32,
    cover: f64,
    chunk: f64,
    grids: u32,
    #[serde(rename = "blends")]
    blend: LineBlend,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for ParcelParams {
    fn default() -> Self {
        Self {
            cells: 16,
            cover: 0.5,
            chunk: 1.0,
            grids: 4,
            blend: LineBlend::Multiply,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl ParcelParams {
    pub fn cells(&self) -> u32 {
        self.cells
    }

    pub fn set_cells(&mut self, cells: u32) -> Result<(), Error> {
        CELLS.check(f64::from(cells))?;
        self.cells = cells;
        Ok(())
    }

    pub fn cover(&self) -> f64 {
        self.cover
    }

    pub fn set_cover(&mut self, cover: f64) -> Result<(), Error> {
        self.cover = COVER.check(cover)?;
        Ok(())
    }

    pub fn chunk(&self) -> f64 {
        self.chunk
    }

    pub fn set_chunk(&mut self, chunk: f64) -> Result<(), Error> {
        self.chunk = CHUNK.check(chunk)?;
        Ok(())
    }

    pub fn grids(&self) -> u32 {
        self.grids
    }

    pub fn set_grids(&mut self, grids: u32) -> Result<(), Error> {
        GRIDS.check(f64::from(grids))?;
        self.grids = grids;
        Ok(())
    }

    pub fn blend(&self) -> LineBlend {
        self.blend
    }

    pub fn set_blend(&mut self, blend: LineBlend) {
        self.blend = blend;
    }

    pub fn dither(&self) -> &Dither {
        &self.dither
    }

    pub fn dither_mut(&mut self) -> &mut Dither {
        &mut self.dither
    }

    pub fn grain(&self) -> &Grain {
        &self.grain
    }

    pub fn grain_mut(&mut self) -> &mut Grain {
        &mut self.grain
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    cells: u32,
    cover: f64,
    chunk: f64,
    grids: u32,
    blends: LineBlend,
    #[serde(rename = "ditherTog")]
    dither_on: bool,
    #[serde(rename = "dthKinds")]
    dither_kind: DitherKind,
    #[serde(rename = "dthSize")]
    dither_size: u32,
    #[serde(rename = "dthLevels")]
    dither_levels: u32,
    #[serde(rename = "dthAmount")]
    dither_amount: f64,
    #[serde(rename = "grainTog")]
    grain_on: bool,
    #[serde(rename = "grnBlends")]
    grain_blend: Blend,
    #[serde(rename = "grnAmount")]
    grain_amount: f64,
    #[serde(rename = "grnSize")]
    grain_size: f64,
    #[serde(rename = "grnSpecks")]
    grain_specks: f64,
    #[serde(rename = "grnVignette")]
    grain_vignette: f64,
}

pub(crate) fn from_json(params: serde_json::Value) -> Result<ParcelParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = ParcelParams::default();
    params.set_cells(raw.cells)?;
    params.set_cover(raw.cover)?;
    params.set_chunk(raw.chunk)?;
    params.set_grids(raw.grids)?;
    params.set_blend(raw.blends);
    let dither = params.dither_mut();
    dither.set_on(raw.dither_on);
    dither.set_kind(raw.dither_kind);
    dither.set_size(raw.dither_size)?;
    dither.set_levels(raw.dither_levels)?;
    dither.set_amount(raw.dither_amount)?;
    let grain = params.grain_mut();
    grain.set_on(raw.grain_on);
    grain.set_blend(raw.grain_blend);
    grain.set_amount(raw.grain_amount)?;
    grain.set_size(raw.grain_size)?;
    grain.set_specks(raw.grain_specks)?;
    grain.set_vignette(raw.grain_vignette)?;
    Ok(params)
}

pub(crate) fn parameters() -> Vec<Parameter> {
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(std::iter::once(Parameter::choice(
            "blends",
            &["Multiply", "Normal"],
        )))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> ParcelParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    ParcelParams {
        cells: pick(&CELLS) as u32,
        cover: pick(&COVER),
        chunk: pick(&CHUNK),
        grids: pick(&GRIDS) as u32,
        ..ParcelParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &ParcelParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, p: &ParcelParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let mut rng = Xorshift::new(tool_seed);
    let (ox, oy) = (rng.next() * 43.0, rng.next() * 37.0);
    let sw = (rng.next() * 1e9) as u32;
    let ns = tool_seed ^ 0x51ed_270b;

    let gw = p.cells as i32;
    let gh = 4.max((f64::from(gw) * h / w).round() as i32);
    let (cw, ch) = (w / f64::from(gw), h / f64::from(gh));

    surface.fill(palette.ink(0));
    let vn = |x: f64, y: f64, k: u32| {
        let (xi, yi) = (x.floor() as i32, y.floor() as i32);
        let (fx, fy) = (x - f64::from(xi), y - f64::from(yi));
        let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
        let a = hash(xi, yi, k, ns);
        let b = hash(xi + 1, yi, k, ns);
        let c = hash(xi, yi + 1, k, ns);
        let d = hash(xi + 1, yi + 1, k, ns);
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    };
    let fbm = |x: f64, y: f64, k: u32| vn(x, y, k) * 0.62 + vn(x * 2.17, y * 2.17, k + 9) * 0.38;
    let sc = 0.2 / p.chunk;
    let th = 0.5 + (0.5 - p.cover) * 0.55;
    let inked = |u: i32, v: i32| fbm(f64::from(u) * sc + ox + 0.9, f64::from(v) * sc + oy, 5) > th;
    let px = |at: i32, size: f64| (f64::from(at) * size).round() as i32;
    for v in 0..gh {
        let mut run = None;
        for u in 0..=gw {
            let on = u < gw && inked(u, v);
            match (on, run) {
                (true, None) => run = Some(u),
                (false, Some(start)) => {
                    let (x0, x1) = (px(start, cw), px(u, cw));
                    let (y0, y1) = (px(v, ch), px(v + 1, ch));
                    surface.fill_rect(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32, palette.ink(1));
                    run = None;
                }
                _ => {}
            }
        }
    }

    let mut path = Path2D::default();
    for (x0, y0, x1, y1) in line_runs(p.grids, gw, gh, sw) {
        path.move_to(f64::from(px(x0, cw)) + 0.5, f64::from(px(y0, ch)) + 0.5);
        path.line_to(f64::from(px(x1, cw)) + 0.5, f64::from(px(y1, ch)) + 0.5);
    }
    if path.is_empty() {
        return;
    }
    let ink = palette.ink(2);
    match p.blend {
        LineBlend::Multiply => surface.with_multiply(|surface| {
            surface.stroke(&path, ink, 1.0, Cap::Butt, Join::Miter);
        }),
        LineBlend::Normal => surface.stroke(&path, ink, 1.0, Cap::Butt, Join::Miter),
    }
}

fn line_runs(grids: u32, gw: i32, gh: i32, sw: u32) -> Vec<(i32, i32, i32, i32)> {
    let mut across: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
    let mut down: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
    for i in 0..grids as i32 {
        let h = |k: i32| hash(i, k, 0, sw);
        let cols = 2 + (h(1) * 6.0) as i32;
        let rows = 2 + (h(2) * 5.0) as i32;
        let cx = (h(3) * f64::from(0.max(gw - cols))).round() as i32;
        let cy = (h(4) * f64::from(0.max(gh - rows))).round() as i32;
        for b in 0..rows {
            for a in 0..cols {
                if hash(a, b, (i * 17 + 7) as u32, sw) >= 0.78 {
                    continue;
                }
                let (a, b) = (cx + a, cy + b);
                across.entry(b).or_default().insert(a);
                across.entry(b + 1).or_default().insert(a);
                down.entry(a).or_default().insert(b);
                down.entry(a + 1).or_default().insert(b);
            }
        }
    }
    let mut runs = Vec::new();
    let mut merge = |lines: BTreeMap<i32, BTreeSet<i32>>, horizontal: bool| {
        for (line, edges) in lines {
            let mut edges = edges.into_iter();
            let Some(first) = edges.next() else { continue };
            let (mut start, mut end) = (first, first + 1);
            for at in edges.chain(std::iter::once(i32::MIN)) {
                if at == end {
                    end += 1;
                    continue;
                }
                runs.push(if horizontal {
                    (start, line, end, line)
                } else {
                    (line, start, line, end)
                });
                (start, end) = (at, at + 1);
            }
        }
    };
    merge(across, true);
    merge(down, false);
    runs
}
