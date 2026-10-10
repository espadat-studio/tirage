use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up, store, to_int32};
use crate::param::Param;
use crate::surface::{Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "delta";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#ff6f00", "#0057ff", "#ffea00", "#00d68f", "#ff2e88", "#2a2a2a",
];
const SILT: [f64; 3] = [38.0, 36.0, 46.0];
const SHORE: [f64; 3] = [236.0, 231.0, 220.0];

const SCALE: Param = Param::new(SLUG, "scale", 10, 90, 10);
const WARP: Param = Param::new(SLUG, "warp", 0, 160, 100);
const CONTRAST: Param = Param::new(SLUG, "contrast", 0, 100, 100);
const BLEACH: Param = Param::new(SLUG, "bleach", 0, 100, 100);
const PATCHES: Param = Param::new(SLUG, "patches", 0, 18, 1);
const CELL: Param = Param::new(SLUG, "cell", 8, 90, 1);
const FILL: Param = Param::new(SLUG, "fill", 5, 100, 100);
const QUANT: Param = Param::new(SLUG, "quant", 0, 100, 100);
const BLOCKS: Param = Param::new(SLUG, "blocks", 0, 10, 1);
const BANDS: Param = Param::new(SLUG, "bands", 0, 6, 1);
const SLICE: Param = Param::new(SLUG, "slice", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    SCALE, WARP, CONTRAST, BLEACH, PATCHES, CELL, FILL, QUANT, BLOCKS, BANDS, SLICE,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeltaParams {
    scale: f64,
    warp: f64,
    contrast: f64,
    bleach: f64,
    patches: u32,
    cell: u32,
    fill: f64,
    quant: f64,
    blocks: u32,
    bands: u32,
    slice: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for DeltaParams {
    fn default() -> Self {
        Self {
            scale: 3.2,
            warp: 0.55,
            contrast: 0.6,
            bleach: 0.35,
            patches: 6,
            cell: 28,
            fill: 0.55,
            quant: 0.35,
            blocks: 3,
            bands: 1,
            slice: 0.25,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl DeltaParams {
    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn contrast(&self) -> f64 {
        self.contrast
    }

    pub fn set_contrast(&mut self, contrast: f64) -> Result<(), Error> {
        self.contrast = CONTRAST.check(contrast)?;
        Ok(())
    }

    pub fn bleach(&self) -> f64 {
        self.bleach
    }

    pub fn set_bleach(&mut self, bleach: f64) -> Result<(), Error> {
        self.bleach = BLEACH.check(bleach)?;
        Ok(())
    }

    pub fn patches(&self) -> u32 {
        self.patches
    }

    pub fn set_patches(&mut self, patches: u32) -> Result<(), Error> {
        PATCHES.check(f64::from(patches))?;
        self.patches = patches;
        Ok(())
    }

    pub fn cell(&self) -> u32 {
        self.cell
    }

    pub fn set_cell(&mut self, cell: u32) -> Result<(), Error> {
        CELL.check(f64::from(cell))?;
        self.cell = cell;
        Ok(())
    }

    pub fn fill(&self) -> f64 {
        self.fill
    }

    pub fn set_fill(&mut self, fill: f64) -> Result<(), Error> {
        self.fill = FILL.check(fill)?;
        Ok(())
    }

    pub fn quant(&self) -> f64 {
        self.quant
    }

    pub fn set_quant(&mut self, quant: f64) -> Result<(), Error> {
        self.quant = QUANT.check(quant)?;
        Ok(())
    }

    pub fn blocks(&self) -> u32 {
        self.blocks
    }

    pub fn set_blocks(&mut self, blocks: u32) -> Result<(), Error> {
        BLOCKS.check(f64::from(blocks))?;
        self.blocks = blocks;
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

    pub fn slice(&self) -> f64 {
        self.slice
    }

    pub fn set_slice(&mut self, slice: f64) -> Result<(), Error> {
        self.slice = SLICE.check(slice)?;
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
    scale: f64,
    warp: f64,
    contrast: f64,
    bleach: f64,
    patches: u32,
    cell: u32,
    fill: f64,
    quant: f64,
    blocks: u32,
    bands: u32,
    slice: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<DeltaParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = DeltaParams::default();
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
    params.set_contrast(raw.contrast)?;
    params.set_bleach(raw.bleach)?;
    params.set_patches(raw.patches)?;
    params.set_cell(raw.cell)?;
    params.set_fill(raw.fill)?;
    params.set_quant(raw.quant)?;
    params.set_blocks(raw.blocks)?;
    params.set_bands(raw.bands)?;
    params.set_slice(raw.slice)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> DeltaParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    DeltaParams {
        scale: pick(&SCALE),
        warp: pick(&WARP),
        contrast: pick(&CONTRAST),
        bleach: pick(&BLEACH),
        patches: pick(&PATCHES) as u32,
        cell: pick(&CELL) as u32,
        fill: pick(&FILL),
        quant: pick(&QUANT),
        blocks: pick(&BLOCKS) as u32,
        bands: pick(&BANDS) as u32,
        slice: pick(&SLICE),
        ..DeltaParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &DeltaParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn hash(x: f64, y: f64, k: f64) -> f64 {
    let a = to_int32(x * 374_761_393.0 + y * 668_265_263.0 + k * 1_442_695_040_888_963_456.0);
    let m = to_int32(f64::from(a ^ (a >> 13)) * 1_274_126_177.0);
    f64::from((m ^ (m >> 16)) as u32) / 4_294_967_296.0
}

fn noise(x: f64, y: f64, k: f64) -> f64 {
    let (xi, yi) = (x.floor(), y.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (smooth(x - xi), smooth(y - yi));
    let corner = |dx: f64, dy: f64| hash(xi + dx, yi + dy, k);
    (corner(0.0, 0.0) * (1.0 - u) + corner(1.0, 0.0) * u) * (1.0 - v)
        + (corner(0.0, 1.0) * (1.0 - u) + corner(1.0, 1.0) * u) * v
}

fn fbm(x: f64, y: f64, k: f64, octaves: u32) -> f64 {
    let (mut sum, mut amp, mut f, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for i in 0..octaves {
        sum += amp * noise(x * f, y * f, k + f64::from(i) * 57.0);
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    sum / norm
}

fn ramp(p: &DeltaParams, palette: &Palette) -> Vec<[f64; 3]> {
    let luma =
        |[r, g, b]: [u8; 3]| f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114;
    let mut inks: Vec<[u8; 3]> = (0..palette.len()).map(|i| palette.ink(i)).collect();
    inks.sort_by(|a, b| luma(*a).total_cmp(&luma(*b)));
    let mix = 0.1 + 0.55 * p.bleach;
    let mut stops = vec![SILT];
    stops.extend(inks.into_iter().map(|ink| {
        let grey = 70.0 + 150.0 * (luma(ink) / 255.0);
        ink.map(|c| round_half_up(f64::from(c) * (1.0 - mix) + grey * mix))
    }));
    stops.push(SHORE);
    stops
}

fn field(p: &DeltaParams, palette: &Palette, s: u32, w: u32, h: u32) -> (u32, u32, Vec<u8>) {
    let long = f64::from(w.max(h));
    let res = round_half_up(long / 3.0).clamp(120.0, 420.0);
    let fw = round_half_up(res * f64::from(w) / long).max(2.0) as u32;
    let fh = round_half_up(res * f64::from(h) / long).max(2.0) as u32;
    let stops = ramp(p, palette);
    let n = stops.len() - 1;
    let ct = 0.5 + p.contrast * 1.6;
    let d = f64::from(s) * 13.0;
    let mut rgba = Vec::with_capacity((fw * fh * 4) as usize);
    for y in 0..fh {
        for x in 0..fw {
            let u = f64::from(x) / f64::from(fh) * p.scale;
            let v = f64::from(y) / f64::from(fh) * p.scale;
            let wx = u + p.warp * (fbm(u * 1.7 + 11.0, v * 1.7, d + 91.0, 3) - 0.5) * 2.4;
            let wy = v + p.warp * (fbm(u * 1.7, v * 1.7 + 7.0, d + 37.0, 3) - 0.5) * 2.4;
            let mut t = fbm(wx, wy, d, 6);
            let ridge = 1.0 - (fbm(wx * 2.1 + 5.0, wy * 2.1 - 3.0, d + 211.0, 4) * 2.0 - 1.0).abs();
            t = t * 0.74 + ridge * ridge * 0.26;
            t = ((t - 0.5) * ct + 0.5).clamp(0.0, 1.0);
            let at = t * n as f64;
            let i = (n - 1).min(at.floor() as usize);
            let m = at - i as f64;
            let (a, b) = (stops[i], stops[i + 1]);
            rgba.extend((0..3).map(|c| store(a[c] + (b[c] - a[c]) * m)));
            rgba.push(255);
        }
    }
    (fw, fh, rgba)
}

fn paint(surface: &mut Surface, p: &DeltaParams, palette: &Palette, s: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let (fw, fh, field) = field(p, palette, s, w, h);
    surface.draw_smooth(&field, fw, fh, Smoothing::Bilinear);

    let mut rng = Xorshift::from_state(s);
    let unit = round_half_up(f64::from(p.cell) * wf / 1200.0).max(4.0);
    let cols = (wf / unit).ceil();
    let rows = (hf / unit).ceil();
    let inks = palette.len();
    let pick = |rng: &mut Xorshift| palette.ink((rng.next() * inks as f64) as usize);
    let rect = |surface: &mut Surface, x: f64, y: f64, rw: f64, rh: f64, ink: [u8; 3]| {
        surface.fill_rect(x as i32, y as i32, rw as u32, rh as u32, ink);
    };

    if p.quant > 0.01 {
        for _ in 0..round_half_up(2.0 + p.quant * 9.0) as u32 {
            let px = round_half_up(unit * (0.5 + rng.next() * 1.6)).max(2.0);
            let rw = round_half_up((3.0 + rng.next() * 11.0) * px);
            let rh = round_half_up((3.0 + rng.next() * 9.0) * px);
            let rx = round_half_up((rng.next() * wf - rw / 2.0) / px) * px;
            let ry = round_half_up((rng.next() * hf - rh / 2.0) / px) * px;
            let mut y = ry;
            while y < ry + rh {
                let mut x = rx;
                while x < rx + rw {
                    if x + px >= 0.0 && y + px >= 0.0 && x <= wf && y <= hf {
                        let sx = ((x + px / 2.0) / wf * f64::from(fw))
                            .floor()
                            .clamp(0.0, f64::from(fw - 1));
                        let sy = ((y + px / 2.0) / hf * f64::from(fh))
                            .floor()
                            .clamp(0.0, f64::from(fh - 1));
                        let o = ((sy as u32 * fw + sx as u32) * 4) as usize;
                        rect(
                            surface,
                            x,
                            y,
                            px,
                            px,
                            [field[o], field[o + 1], field[o + 2]],
                        );
                    }
                    x += px;
                }
                y += px;
            }
        }
    }

    for _ in 0..p.patches {
        let fine = rng.next() < 0.38;
        let u = round_half_up(unit * if fine { 0.42 } else { 1.0 }).max(3.0);
        let pw = round_half_up((3.0 + rng.next() * if fine { 16.0 } else { 9.0 }) * u);
        let ph = round_half_up((3.0 + rng.next() * if fine { 14.0 } else { 8.0 }) * u);
        let px = round_half_up((rng.next() * wf - pw / 2.0) / u) * u;
        let py = round_half_up((rng.next() * hf - ph / 2.0) / u) * u;
        let mut local: Vec<[u8; 3]> = (0..inks).map(|i| palette.ink(i)).collect();
        shuffle(&mut local, &mut rng);
        local.truncate(2 + (rng.next() * 3.0) as usize);
        let density = p.fill * if fine { 0.85 } else { 1.0 };
        let mut y = py;
        while y < py + ph {
            let mut x = px;
            while x < px + pw {
                if rng.next() <= density {
                    let ink = local[(rng.next() * local.len() as f64) as usize];
                    rect(surface, x, y, u, u, ink);
                }
                x += u;
            }
            y += u;
        }
    }

    for _ in 0..p.blocks {
        let bw = round_half_up((4.0 + rng.next() * 12.0) * unit);
        let bh = round_half_up((2.0 + rng.next() * 8.0) * unit);
        let bx = round_half_up(rng.next() * cols) * unit;
        let by = round_half_up(rng.next() * rows) * unit;
        let ink = pick(&mut rng);
        rect(
            surface,
            bx - round_half_up(bw / 2.0),
            by - round_half_up(bh / 2.0),
            bw,
            bh,
            ink,
        );
    }

    for _ in 0..p.bands {
        let bh = round_half_up((1.0 + rng.next() * 4.0) * unit);
        let by = round_half_up(rng.next() * rows) * unit;
        let ink = pick(&mut rng);
        let alpha = 0.85 + rng.next() * 0.15;
        surface.with_alpha(alpha as f32, |surface| rect(surface, 0.0, by, wf, bh, ink));
    }

    if p.slice > 0.01 {
        let mut rs = Xorshift::from_state(s.wrapping_add(555));
        let strips = round_half_up(2.0 + p.slice * 14.0) as u32;
        surface.edit_rgba(|rgba, w, h| {
            for _ in 0..strips {
                let sh = round_half_up((0.4 + rs.next() * 2.2) * unit) as i64;
                let sy = (round_half_up(rs.next() * rows) * unit) as i64;
                let dx = round_half_up((rs.next() - 0.5) * p.slice * 10.0 * unit) as i64;
                if dx == 0 {
                    continue;
                }
                if sy == i64::from(h) {
                    break;
                }
                shift(rgba, w, h, sy, sh, dx);
            }
        });
    }
}

fn shift(rgba: &mut [u8], w: u32, h: u32, sy: i64, sh: i64, dx: i64) {
    let (w, h) = (i64::from(w), i64::from(h));
    if sy >= h {
        return;
    }
    let tall = sh.min(h - sy);
    let row = (w * 4) as usize;
    let strip = rgba[sy as usize * row..(sy + tall) as usize * row].to_vec();
    for offset in [dx, dx - dx.signum() * w] {
        for y in 0..tall {
            for x in 0..w {
                let to = x + offset;
                if (0..w).contains(&to) {
                    let from = (y * w + x) as usize * 4;
                    let at = ((sy + y) * w + to) as usize * 4;
                    rgba[at..at + 4].copy_from_slice(&strip[from..from + 4]);
                }
            }
        }
    }
}

fn shuffle<T>(items: &mut [T], rng: &mut Xorshift) {
    let len = items.len();
    let mut run = 1;
    if len > 7 {
        run = 2;
        let descending = rng.next() < 0.5;
        while run < len && (rng.next() < 0.5) == descending {
            run += 1;
        }
        if descending {
            items[..run].reverse();
        }
    }
    for start in run..len {
        let (mut left, mut right) = (0, start);
        while left < right {
            let mid = left + (right - left) / 2;
            if rng.next() < 0.5 {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        items[left..=start].rotate_right(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Xorshift, shuffle};

    #[test]
    fn shuffle_matches_v8_sort_with_a_random_comparator() {
        let cases: [(u32, &[usize], f64); 5] = [
            (7, &[2, 3, 1, 4, 0, 5], 0.6565695323515683),
            (9611518, &[4, 0, 1, 2, 5, 3], 0.7709579619113356),
            (3, &[1, 0], 0.046982408268377185),
            (12345, &[0, 3, 2, 4, 1], 0.6742795133031905),
            (99, &[3, 1, 4, 7, 2, 5, 6, 0], 0.9303633281961083),
        ];
        for (seed, sorted, next) in cases {
            let mut items: Vec<usize> = (0..sorted.len()).collect();
            let mut rng = Xorshift::from_state(seed);
            shuffle(&mut items, &mut rng);
            assert_eq!(items, sorted, "seed {seed}");
            assert_eq!(rng.next(), next, "seed {seed} drew a different count");
        }
    }
}
