use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, value_noise,
};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "cipher";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 7] = [
    "#2a0a14", "#dce0f0", "#f5d20a", "#7cc4f0", "#f58a1e", "#e8412c", "#2e6bc0",
];

const CELLS: Param = Param::new(SLUG, "cells", 20, 140, 1);
const BANDS: Param = Param::new(SLUG, "bands", 2, 7, 1);
const WAVE: Param = Param::new(SLUG, "wave", 0, 100, 100);
const TILT: Param = Param::new(SLUG, "tilt", -100, 100, 100);
const DETAIL: Param = Param::new(SLUG, "detail", 0, 100, 100);
const SIZE: Param = Param {
    taste: (48, 100),
    ..Param::new(SLUG, "size", 30, 100, 100)
};
const EDGES: Param = Param::new(SLUG, "edges", 0, 100, 100);
const FLECKS: Param = Param::new(SLUG, "flecks", 0, 30, 100);
const SMEARS: Param = Param::new(SLUG, "smears", 0, 60, 100);
const DOTS: Param = Param::new(SLUG, "dots", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    CELLS, BANDS, WAVE, TILT, DETAIL, SIZE, EDGES, FLECKS, SMEARS, DOTS,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherField {
    Figure,
    Relief,
    Slope,
}

const FIELDS: [CipherField; 3] = [CipherField::Figure, CipherField::Relief, CipherField::Slope];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CipherParams {
    cells: u32,
    bands: u32,
    #[serde(rename = "fields")]
    field: CipherField,
    wave: f64,
    tilt: f64,
    detail: f64,
    size: f64,
    edges: f64,
    flecks: f64,
    smears: f64,
    dots: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for CipherParams {
    fn default() -> Self {
        Self {
            cells: 80,
            bands: 5,
            field: CipherField::Figure,
            wave: 0.6,
            tilt: 0.2,
            detail: 0.5,
            size: 0.8,
            edges: 0.6,
            flecks: 0.06,
            smears: 0.15,
            dots: 0.6,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl CipherParams {
    pub fn cells(&self) -> u32 {
        self.cells
    }

    pub fn set_cells(&mut self, cells: u32) -> Result<(), Error> {
        CELLS.check(f64::from(cells))?;
        self.cells = cells;
        Ok(())
    }

    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn field(&self) -> CipherField {
        self.field
    }

    pub fn set_field(&mut self, field: CipherField) {
        self.field = field;
    }

    pub fn wave(&self) -> f64 {
        self.wave
    }

    pub fn set_wave(&mut self, wave: f64) -> Result<(), Error> {
        self.wave = WAVE.check(wave)?;
        Ok(())
    }

    pub fn tilt(&self) -> f64 {
        self.tilt
    }

    pub fn set_tilt(&mut self, tilt: f64) -> Result<(), Error> {
        self.tilt = TILT.check(tilt)?;
        Ok(())
    }

    pub fn detail(&self) -> f64 {
        self.detail
    }

    pub fn set_detail(&mut self, detail: f64) -> Result<(), Error> {
        self.detail = DETAIL.check(detail)?;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn edges(&self) -> f64 {
        self.edges
    }

    pub fn set_edges(&mut self, edges: f64) -> Result<(), Error> {
        self.edges = EDGES.check(edges)?;
        Ok(())
    }

    pub fn flecks(&self) -> f64 {
        self.flecks
    }

    pub fn set_flecks(&mut self, flecks: f64) -> Result<(), Error> {
        self.flecks = FLECKS.check(flecks)?;
        Ok(())
    }

    pub fn smears(&self) -> f64 {
        self.smears
    }

    pub fn set_smears(&mut self, smears: f64) -> Result<(), Error> {
        self.smears = SMEARS.check(smears)?;
        Ok(())
    }

    pub fn dots(&self) -> f64 {
        self.dots
    }

    pub fn set_dots(&mut self, dots: f64) -> Result<(), Error> {
        self.dots = DOTS.check(dots)?;
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
    cells: u32,
    bands: u32,
    #[serde(rename = "fields")]
    field: CipherField,
    wave: f64,
    tilt: f64,
    detail: f64,
    size: f64,
    edges: f64,
    flecks: f64,
    smears: f64,
    dots: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<CipherParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = CipherParams::default();
    params.set_cells(raw.cells)?;
    params.set_bands(raw.bands)?;
    params.set_field(raw.field);
    params.set_wave(raw.wave)?;
    params.set_tilt(raw.tilt)?;
    params.set_detail(raw.detail)?;
    params.set_size(raw.size)?;
    params.set_edges(raw.edges)?;
    params.set_flecks(raw.flecks)?;
    params.set_smears(raw.smears)?;
    params.set_dots(raw.dots)?;
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
    PARAMS[..2]
        .iter()
        .map(Param::parameter)
        .chain(std::iter::once(Parameter::choice(
            "fields",
            &["Figure", "Relief", "Slope"],
        )))
        .chain(PARAMS[2..].iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> CipherParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    CipherParams {
        cells: pick(&CELLS) as u32,
        bands: pick(&BANDS) as u32,
        field: FIELDS[(draw("fields") % FIELDS.len() as u64) as usize],
        wave: pick(&WAVE),
        tilt: pick(&TILT),
        detail: pick(&DETAIL),
        size: pick(&SIZE),
        edges: pick(&EDGES),
        flecks: pick(&FLECKS),
        smears: pick(&SMEARS),
        dots: pick(&DOTS),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &CipherParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Glyph {
    Square,
    Diamond,
    Ring,
    X,
}

const GLYPHS: [Glyph; 4] = [Glyph::Square, Glyph::Diamond, Glyph::Ring, Glyph::X];

struct Shape {
    spine: f64,
    fx2: f64,
    fy2: f64,
}

fn field(params: &CipherParams, shape: &Shape, seed: u32, u: f64, v: f64) -> f64 {
    let (wave, tilt) = (params.wave, params.tilt);
    let det = 1.0 + params.detail * 3.0;
    let noise = |x: f64, y: f64, k: u32| value_noise(x, y, seed.wrapping_mul(7).wrapping_add(k));
    match params.field {
        CipherField::Figure => {
            let wv = 0.16
                + 0.5
                    * wave
                    * (0.3
                        + noise(v * det * 1.6 + 3.0, 0.5, 1) * 1.0
                        + 0.35 * noise(v * det * 4.0 + 9.0, 1.5, 1));
            let centre = shape.spine + tilt * (v - 0.5);
            let s1 = (u - centre).abs() / wv;
            let wv2 = 0.12 + 0.36 * wave * (0.4 + noise(v * det * 1.9 + 13.0, 2.5, 1));
            let (dx, dy) = ((u - shape.fx2) / wv2, (v - shape.fy2) / (wv2 * 1.6));
            let s2 = (dx * dx + dy * dy).sqrt() * 0.9;
            1f64.min(s1.min(s2))
        }
        CipherField::Relief => {
            let w = v + tilt * u;
            let n = 0.65 * noise(u * det * 1.3 + 5.0, w * det * 1.3 + 2.0, 2)
                + 0.35 * noise(u * det * 3.1 + 8.0, w * det * 3.1 + 4.0, 2);
            1f64.min(0f64.max((n - 0.2) / 0.6 * (0.6 + wave * 0.6)))
        }
        CipherField::Slope => {
            let s = v
                + tilt * (u - 0.5)
                + wave * 0.35 * (noise(u * det * 1.5 + 7.0, 0.5, 3) - 0.5) * 2.0
                + wave * 0.08 * (noise(u * det * 6.0 + 3.0, 1.5, 3) - 0.5) * 2.0;
            1f64.min(0f64.max(s))
        }
    }
}

fn paint(surface: &mut Surface, params: &CipherParams, palette: &Palette, tool_seed: u32) {
    let seed = tool_seed;
    let inks = palette.inks();
    let ground = inks[0];
    let marks = &inks[1..];
    let dim = ground.map(|c| round_half_up(f64::from(c) + (255.0 - f64::from(c)) * 0.22) as u8);

    let bands = params.bands as usize;
    let mut rng = XorShift::new(seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let mut band_glyph = Vec::with_capacity(bands);
    let mut band_ink = Vec::with_capacity(bands);
    let (mut last_glyph, mut last_ink) = (usize::MAX, usize::MAX);
    for _ in 0..bands {
        let mut g = (rng.next() * 4.0) as usize;
        if g == last_glyph {
            g = (g + 1) % 4;
        }
        let mut k = (rng.next() * marks.len() as f64) as usize;
        if k == last_ink {
            k = (k + 1) % marks.len();
        }
        band_glyph.push(GLYPHS[g]);
        band_ink.push(marks[k]);
        (last_glyph, last_ink) = (g, k);
    }
    let spine = 0.5 + (rng.next() - 0.5) * 0.2;
    let fy2 = 0.55 + rng.next() * 0.3;
    let fx2 = 0.3 + rng.next() * 0.4;
    let shape = Shape { spine, fx2, fy2 };

    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let cols = params.cells as usize;
    let cs = width / cols as f64;
    let rows = (height / cs).ceil() as usize;
    let g = cs * params.size;
    let mut band = vec![0; cols * rows];
    for j in 0..rows {
        for i in 0..cols {
            let u = (i as f64 + 0.5) / cols as f64;
            let v = (j as f64 + 0.5) / rows as f64;
            let b = (field(params, &shape, seed, u, v) * bands as f64).floor() as usize;
            band[j * cols + i] = b.min(bands - 1);
        }
    }

    surface.fill(ground);
    let h = |x: usize, y: usize| hash(x as i32, y as i32, seed);
    let smears: Vec<Option<(usize, usize)>> = (0..rows)
        .map(|j| {
            (h(j, 1) < params.smears).then(|| {
                let at = (h(j, 2) * cols as f64).floor() as usize;
                let len = 3 + (h(j, 3) * 14f64.min(cols as f64 * 0.2)).floor() as usize;
                (at, at + len)
            })
        })
        .collect();

    for j in 0..rows {
        let y = (j as f64 + 0.5) * cs;
        for i in 0..cols {
            let x = (i as f64 + 0.5) * cs;
            let b = band[j * cols + i];
            let hc = h(i, j);
            let fleck = || marks[(h(i, j + 7) * marks.len() as f64) as usize];
            if b == bands - 1 {
                if params.dots > 0.0 && hc > params.flecks * 0.4 {
                    surface.fill_circle(x, y, 0.3f64.max(cs * 0.09 * params.dots), dim);
                } else if params.flecks > 0.0 && hc <= params.flecks * 0.4 {
                    surface.fill_box(x - g * 0.3, y - g * 0.3, g * 0.6, g * 0.6, fleck());
                }
                continue;
            }
            let (mut ink, mut glyph) = (band_ink[b], band_glyph[b]);
            if hc < params.flecks {
                (ink, glyph) = (fleck(), Glyph::Square);
            } else {
                let at = |i: usize, j: usize| band[j * cols + i];
                let right = if i + 1 < cols { at(i + 1, j) } else { b };
                let left = if i > 0 { at(i - 1, j) } else { b };
                let down = if j + 1 < rows { at(i, j + 1) } else { b };
                let up = if j > 0 { at(i, j - 1) } else { b };
                if [right, left, down, up].iter().any(|&n| n != b) && h(i, j + 3) < params.edges {
                    let first = band_ink
                        .iter()
                        .position(|&k| k == ink)
                        .expect("a band's ink is a band ink");
                    let step =
                        (hash(i as i32, (j + 11) as i32, seed) * (marks.len() - 1) as f64) as usize;
                    (ink, glyph) = (marks[(first + 1 + step) % marks.len()], Glyph::X);
                }
            }
            if let Some((start, end)) = smears[j]
                && (start..end).contains(&i)
            {
                surface.fill_box(x - cs * 0.5, y - g * 0.32, cs + 0.5, g * 0.64, ink);
                continue;
            }
            draw_glyph(surface, glyph, x, y, g, ink, ground);
        }
    }
}

fn draw_glyph(
    surface: &mut Surface,
    glyph: Glyph,
    x: f64,
    y: f64,
    g: f64,
    ink: [u8; 3],
    ground: [u8; 3],
) {
    match glyph {
        Glyph::Square => surface.fill_box(x - g * 0.5, y - g * 0.5, g, g, ink),
        Glyph::Diamond => {
            let d = g * 0.55;
            let mut path = Path2D::default();
            path.move_to(x, y - d);
            path.line_to(x + d * 0.78, y);
            path.line_to(x, y + d);
            path.line_to(x - d * 0.78, y);
            path.close();
            surface.fill_path(&path.into_path(), ink);
        }
        Glyph::Ring => {
            surface.fill_box(x - g * 0.5, y - g * 0.5, g, g, ink);
            let mut ring = Path2D::default();
            ring.push_circle(x, y, g * 0.3);
            surface.stroke(&ring, ground, 0.6f64.max(g * 0.1), Cap::Butt, Join::Miter);
            surface.fill_circle(x, y, g * 0.1, ground);
        }
        Glyph::X => {
            let d = g * 0.36;
            let mut path = Path2D::default();
            path.move_to(x - d, y - d);
            path.line_to(x + d, y + d);
            path.move_to(x + d, y - d);
            path.line_to(x - d, y + d);
            surface.stroke(&path, ink, 0.6f64.max(g * 0.16), Cap::Butt, Join::Miter);
        }
    }
}
