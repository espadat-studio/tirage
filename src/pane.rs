use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, hash, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "pane";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 8] = [
    "#22223b", "#4a4e69", "#9a8c98", "#c9ada7", "#f2e9e4", "#ff7b54", "#ffb26b", "#ffd56f",
];

const DIRS: [[f64; 4]; 8] = [
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0, 1.0],
    [0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 1.0, 1.0, 0.0],
];

const ROWS: Param = Param::new(SLUG, "rows", 1, 10, 1);
const CELLS: Param = Param::new(SLUG, "cells", 1, 16, 1);
const VARY: Param = Param::new(SLUG, "vary", 0, 100, 100);
const DIAG: Param = Param::new(SLUG, "diag", 0, 100, 100);
const SOFT: Param = Param::new(SLUG, "soft", 0, 100, 100);
const SPREAD: Param = Param::new(SLUG, "spread", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[ROWS, CELLS, VARY, DIAG, SOFT, SPREAD];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PaneParams {
    rows: u32,
    cells: u32,
    vary: f64,
    diag: f64,
    soft: f64,
    spread: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for PaneParams {
    fn default() -> Self {
        Self {
            rows: 3,
            cells: 6,
            vary: 0.55,
            diag: 0.45,
            soft: 0.85,
            spread: 0.55,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl PaneParams {
    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn set_rows(&mut self, rows: u32) -> Result<(), Error> {
        ROWS.check(f64::from(rows))?;
        self.rows = rows;
        Ok(())
    }

    pub fn cells(&self) -> u32 {
        self.cells
    }

    pub fn set_cells(&mut self, cells: u32) -> Result<(), Error> {
        CELLS.check(f64::from(cells))?;
        self.cells = cells;
        Ok(())
    }

    pub fn vary(&self) -> f64 {
        self.vary
    }

    pub fn set_vary(&mut self, vary: f64) -> Result<(), Error> {
        self.vary = VARY.check(vary)?;
        Ok(())
    }

    pub fn diag(&self) -> f64 {
        self.diag
    }

    pub fn set_diag(&mut self, diag: f64) -> Result<(), Error> {
        self.diag = DIAG.check(diag)?;
        Ok(())
    }

    pub fn soft(&self) -> f64 {
        self.soft
    }

    pub fn set_soft(&mut self, soft: f64) -> Result<(), Error> {
        self.soft = SOFT.check(soft)?;
        Ok(())
    }

    pub fn spread(&self) -> f64 {
        self.spread
    }

    pub fn set_spread(&mut self, spread: f64) -> Result<(), Error> {
        self.spread = SPREAD.check(spread)?;
        Ok(())
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
    rows: u32,
    cells: u32,
    vary: f64,
    diag: f64,
    soft: f64,
    spread: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<PaneParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = PaneParams::default();
    params.set_rows(raw.rows)?;
    params.set_cells(raw.cells)?;
    params.set_vary(raw.vary)?;
    params.set_diag(raw.diag)?;
    params.set_soft(raw.soft)?;
    params.set_spread(raw.spread)?;
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
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> PaneParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    PaneParams {
        rows: pick(&ROWS) as u32,
        cells: pick(&CELLS) as u32,
        vary: pick(&VARY),
        diag: pick(&DIAG),
        soft: pick(&SOFT),
        spread: pick(&SPREAD),
        ..PaneParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &PaneParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, p: &PaneParams, palette: &Palette, s: u32) {
    let (w, h) = (surface.width(), surface.height());
    let weights = |n: u32, salt: u32| {
        let raw: Vec<f64> = (0..n)
            .map(|i| {
                1.0 + (hash(i as i32, salt as i32, s.wrapping_add(salt)) * 2.0 - 1.0)
                    * p.vary
                    * 0.85
            })
            .collect();
        let total: f64 = raw.iter().sum();
        raw.into_iter().map(|v| v / total).collect::<Vec<_>>()
    };
    let edges = |shares: &[f64], extent: u32| {
        let mut at = 0.0;
        let mut edges = vec![0];
        for (k, share) in shares.iter().enumerate() {
            at += share;
            edges.push(if k == shares.len() - 1 {
                extent
            } else {
                round_half_up(at * f64::from(extent)) as u32
            });
        }
        edges
    };

    let rows = edges(&weights(p.rows.max(1), 3), h);
    for (j, band) in rows.windows(2).enumerate() {
        let j = j as u32;
        let count =
            round_half_up(f64::from(p.cells) * (0.55 + hash(j as i32, 5, s.wrapping_add(7)) * 0.9))
                .max(1.0) as u32;
        let columns = edges(&weights(count, 11 + j * 7), w);
        for (i, span) in columns.windows(2).enumerate() {
            cell(
                surface,
                p,
                palette,
                s,
                (span[0], band[0]),
                (span[1], band[1]),
                i as u32,
                j,
            );
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a cell needs its rect and indices"
)]
fn cell(
    surface: &mut Surface,
    p: &PaneParams,
    palette: &Palette,
    s: u32,
    (x0, y0): (u32, u32),
    (x1, y1): (u32, u32),
    i: u32,
    j: u32,
) {
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let h = |salt: u32| hash(i as i32, j as i32, s.wrapping_add(salt));
    let (x, y) = (f64::from(x0), f64::from(y0));
    let (cw, ch) = (f64::from(x1 - x0), f64::from(y1 - y0));
    let diagonal = h(13) < p.diag;
    let k = (h(17) * 4.0) as usize * 2 + usize::from(diagonal);
    let d = DIRS[k & 7];

    let n = palette.len();
    let a = (h(19) * n as f64) as usize;
    let reach = round_half_up(1.0 + p.spread * (n as f64 - 2.0)).max(1.0);
    let step = 1 + (h(23) * reach) as usize;
    let b = (a + step) % n;

    let sp = 0.5 * (1.0 - p.soft);
    let p0 = (sp * (0.4 + h(29) * 1.2)).min(0.49);
    let p1 = (1.0 - sp * (0.4 + h(31) * 1.2)).max(0.51);

    surface.fill_rect_linear(
        x0 as i32,
        y0 as i32,
        x1 - x0,
        y1 - y0,
        (x + d[0] * cw, y + d[1] * ch),
        (x + d[2] * cw, y + d[3] * ch),
        &ramp(palette.ink(a), palette.ink(b), p0, p1),
    );
}

fn ramp(from: [u8; 3], to: [u8; 3], p0: f64, p1: f64) -> Vec<(f64, [u8; 3])> {
    let (mut a, mut b) = (hsl(from), hsl(to));
    if a[1] < 0.04 {
        a[0] = b[0];
    }
    if b[1] < 0.04 {
        b[0] = a[0];
    }
    let mut dh = b[0] - a[0];
    if dh > 180.0 {
        dh -= 360.0;
    } else if dh < -180.0 {
        dh += 360.0;
    }
    let mut stops = vec![(0.0, from)];
    if p0 > 0.0 {
        stops.push((p0, from));
    }
    for q in 1..8 {
        let t = f64::from(q) / 8.0;
        stops.push((
            p0 + (p1 - p0) * t,
            rgb(
                a[0] + dh * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ),
        ));
    }
    if p1 < 1.0 {
        stops.push((p1, to));
    }
    stops.push((1.0, to));
    stops
}

fn hsl(ink: [u8; 3]) -> [f64; 3] {
    let [r, g, b] = ink.map(|c| f64::from(c) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    let l = (max + min) / 2.0;
    if d == 0.0 {
        return [0.0, 0.0, l];
    }
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    [h * 60.0, s, l]
}

fn rgb(h: f64, s: f64, l: f64) -> [u8; 3] {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let (s, l) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
    let a = s * l.min(1.0 - l);
    [0.0, 8.0, 4.0].map(|n: f64| {
        let k = (n + h / 30.0) % 12.0;
        let v = l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        round_half_up(v * 255.0) as u8
    })
}

#[cfg(test)]
mod tests {
    use super::{hsl, rgb};

    #[test]
    fn hsl_round_trips_an_ink() {
        for ink in [[255, 123, 84], [34, 34, 59], [255, 255, 255], [0, 200, 10]] {
            let [h, s, l] = hsl(ink);
            assert_eq!(rgb(h, s, l), ink, "{ink:?}");
        }
    }
}
