use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "culture";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 3] = ["#f6d8b0", "#e4572e", "#17bebb"];

const COUNT: Param = Param {
    taste: (15, 70),
    ..Param::new(SLUG, "count", 1, 90, 1)
};
const SIZE: Param = Param {
    taste: (15, 60),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const FUSE: Param = Param {
    taste: (0, 80),
    ..Param::new(SLUG, "fuse", 0, 100, 100)
};
const COVER: Param = Param::new(SLUG, "cover", 0, 100, 100);
const SPREAD: Param = Param::new(SLUG, "spread", 5, 200, 100);
const SOFT: Param = Param::new(SLUG, "soft", 2, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);
const DOT: Param = Param::new(SLUG, "dot", 1, 10, 1);

pub(crate) const PARAMS: &[Param] = &[COUNT, SIZE, FUSE, COVER, SPREAD, SOFT, DOT];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CultureTexture {
    Fine,
    Stipple,
}

const TEXTURES: [CultureTexture; 2] = [CultureTexture::Fine, CultureTexture::Stipple];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CultureParams {
    count: u32,
    size: f64,
    fuse: f64,
    cover: f64,
    spread: f64,
    soft: f64,
    #[serde(rename = "textures")]
    texture: CultureTexture,
    grain: f64,
    dot: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for CultureParams {
    fn default() -> Self {
        Self {
            count: 36,
            size: 0.4,
            fuse: 0.62,
            cover: 0.72,
            spread: 1.7,
            soft: 0.3,
            texture: CultureTexture::Fine,
            grain: 0.62,
            dot: 2,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl CultureParams {
    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn fuse(&self) -> f64 {
        self.fuse
    }

    pub fn set_fuse(&mut self, fuse: f64) -> Result<(), Error> {
        self.fuse = FUSE.check(fuse)?;
        Ok(())
    }

    pub fn cover(&self) -> f64 {
        self.cover
    }

    pub fn set_cover(&mut self, cover: f64) -> Result<(), Error> {
        self.cover = COVER.check(cover)?;
        Ok(())
    }

    pub fn spread(&self) -> f64 {
        self.spread
    }

    pub fn set_spread(&mut self, spread: f64) -> Result<(), Error> {
        self.spread = SPREAD.check(spread)?;
        Ok(())
    }

    pub fn soft(&self) -> f64 {
        self.soft
    }

    pub fn set_soft(&mut self, soft: f64) -> Result<(), Error> {
        self.soft = SOFT.check(soft)?;
        Ok(())
    }

    pub fn texture(&self) -> CultureTexture {
        self.texture
    }

    pub fn set_texture(&mut self, texture: CultureTexture) {
        self.texture = texture;
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn dot(&self) -> u32 {
        self.dot
    }

    pub fn set_dot(&mut self, dot: u32) -> Result<(), Error> {
        DOT.check(f64::from(dot))?;
        self.dot = dot;
        Ok(())
    }

    pub fn dither(&self) -> &Dither {
        &self.dither
    }

    pub fn dither_mut(&mut self) -> &mut Dither {
        &mut self.dither
    }

    pub fn grain_pass(&self) -> &Grain {
        &self.chassis_grain
    }

    pub fn grain_pass_mut(&mut self) -> &mut Grain {
        &mut self.chassis_grain
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    count: u32,
    size: f64,
    fuse: f64,
    cover: f64,
    spread: f64,
    soft: f64,
    #[serde(rename = "textures")]
    texture: CultureTexture,
    grain: f64,
    dot: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<CultureParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = CultureParams::default();
    params.set_count(raw.count)?;
    params.set_size(raw.size)?;
    params.set_fuse(raw.fuse)?;
    params.set_cover(raw.cover)?;
    params.set_spread(raw.spread)?;
    params.set_soft(raw.soft)?;
    params.set_texture(raw.texture);
    params.set_grain(raw.grain)?;
    params.set_dot(raw.dot)?;
    let dither = params.dither_mut();
    dither.set_on(raw.dither_on);
    dither.set_kind(raw.dither_kind);
    dither.set_size(raw.dither_size)?;
    dither.set_levels(raw.dither_levels)?;
    dither.set_amount(raw.dither_amount)?;
    let grain = params.grain_pass_mut();
    grain.set_on(raw.grain_on);
    grain.set_blend(raw.grain_blend);
    grain.set_amount(raw.grain_amount)?;
    grain.set_size(raw.grain_size)?;
    grain.set_specks(raw.grain_specks)?;
    grain.set_vignette(raw.grain_vignette)?;
    Ok(params)
}

pub(crate) fn parameters() -> Vec<Parameter> {
    let (before, after) = PARAMS.split_at(6);
    before
        .iter()
        .map(Param::parameter)
        .chain(std::iter::once(Parameter::choice(
            "textures",
            &["Fine", "Stipple"],
        )))
        .chain([&GRAIN].into_iter().chain(after).map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> CultureParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    CultureParams {
        count: pick(&COUNT) as u32,
        size: pick(&SIZE),
        fuse: pick(&FUSE),
        cover: pick(&COVER),
        spread: pick(&SPREAD),
        soft: pick(&SOFT),
        texture: TEXTURES[(draw("textures") % TEXTURES.len() as u64) as usize],
        grain: 0.62,
        dot: pick(&DOT) as u32,
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &CultureParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

struct Plate {
    field: Vec<f32>,
    width: usize,
    height: usize,
}

fn plate(params: &CultureParams, tool_seed: u32, aspect: f64) -> Plate {
    let width = if aspect > 1.0 {
        round_half_up(560.0 / aspect).max(8.0)
    } else {
        560.0
    };
    let height = round_half_up(width * aspect).max(8.0);
    let mut field = vec![0.0_f32; (width * height) as usize];
    let mut rng = XorShift::new(tool_seed.wrapping_mul(2_654_435_761));
    let base =
        (0.05 + params.size * 0.20) * (0.75 + params.fuse * 0.8) * (width * height).sqrt() * 0.87;
    for _ in 0..params.count.max(1) {
        let cx = (-0.15 + rng.next() * 1.3) * width;
        let cy = (-0.15 + rng.next() * 1.3) * height;
        rng.next();
        let r = base * (0.55 + rng.next() * 0.9);
        let x0 = (cx - r).floor().max(0.0);
        let x1 = (cx + r).ceil().min(width - 1.0);
        let y0 = (cy - r).floor().max(0.0);
        let y1 = (cy + r).ceil().min(height - 1.0);
        if x1 < x0 || y1 < y0 {
            continue;
        }
        let r2 = r * r;
        for j in y0 as usize..=y1 as usize {
            let dy = j as f64 - cy;
            let dy2 = dy * dy;
            let row = j * width as usize;
            for i in x0 as usize..=x1 as usize {
                let dx = i as f64 - cx;
                let d2 = (dx * dx + dy2) / r2;
                if d2 < 1.0 {
                    let t = 1.0 - d2;
                    let cell = &mut field[row + i];
                    *cell = (f64::from(*cell) + t * t) as f32;
                }
            }
        }
    }
    Plate {
        field,
        width: width as usize,
        height: height as usize,
    }
}

fn rings(n: usize, cover: f64, spread: f64) -> Vec<f64> {
    let t0 = ((1.0 - cover) * 0.9).clamp(0.002, 0.998);
    let edge = (1.0 - t0.sqrt()).max(0.0).sqrt();
    let mut th = vec![0.0; n + 1];
    for (k, slot) in th.iter_mut().enumerate().take(n).skip(1) {
        let d = edge * ((n - k) as f64 / (n - 1) as f64).powf(spread);
        let e = 1.0 - d * d;
        *slot = e * e;
    }
    th[n] = (th[n - 1] + 1e-4).max(1.55);
    th
}

fn paint(surface: &mut Surface, params: &CultureParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let seed = tool_seed as i32;
    let inks: Vec<[f64; 3]> = palette.inks().iter().map(|c| c.map(f64::from)).collect();
    let n = inks.len();
    let Plate {
        field,
        width: fw,
        height: fh,
    } = plate(params, tool_seed, hf / wf);
    let th = rings(n, params.cover, params.spread);
    let soft = params.soft.max(0.001);
    let gq = params.grain * 1.05;
    let gl = params.grain * 46.0;
    let dot = params.dot.max(1);
    let stipple = params.texture == CultureTexture::Stipple;
    let (kx, ky) = (fw as f64 / wf, fh as f64 / hf);
    let band_seed = seed.wrapping_add(29) as u32;
    let colour_seed = seed.wrapping_add(71) as u32;
    let at = |k: usize| f64::from(field[k]);

    let mut rgba = Vec::with_capacity((w * h) as usize * 4);
    for y in 0..h {
        let gy = f64::from(y) * ky;
        let y0 = (gy as usize).min(fh - 1);
        let fy = gy - y0 as f64;
        let y1 = (y0 + 1).min(fh - 1);
        let (r0, r1) = (y0 * fw, y1 * fw);
        for x in 0..w {
            let gx = f64::from(x) * kx;
            let x0 = (gx as usize).min(fw - 1);
            let fx = gx - x0 as f64;
            let x1 = (x0 + 1).min(fw - 1);
            let a = at(r0 + x0) + (at(r0 + x1) - at(r0 + x0)) * fx;
            let b = at(r1 + x0) + (at(r1 + x1) - at(r1 + x0)) * fx;
            let v = a + (b - a) * fy;

            let mut u = if v < th[1] {
                if th[1] > 0.0 { v / th[1] } else { 0.0 }
            } else {
                let mut k = 1;
                while k < n - 1 && v >= th[k + 1] {
                    k += 1;
                }
                let (lo, hi) = (th[k], th[k + 1]);
                k as f64 + if hi > lo { (v - lo) / (hi - lo) } else { 0.0 }
            };
            let (nx, ny) = if stipple {
                ((x / dot) as i32, (y / dot) as i32)
            } else {
                (x as i32, y as i32)
            };
            if gq > 0.002 {
                u += (hash(nx, ny, band_seed) - 0.5) * gq;
            }
            let u = u.clamp(0.0, (n - 1) as f64);
            let i0 = (u as usize).min(n - 2);
            let fr = ((u - i0 as f64 - (1.0 - soft)) / soft).clamp(0.0, 1.0);
            let fr = fr * fr * (3.0 - 2.0 * fr);
            let (ia, ib) = (inks[i0], inks[i0 + 1]);
            let mut c: [f64; 3] = std::array::from_fn(|i| ia[i] + (ib[i] - ia[i]) * fr);
            if gl > 0.002 {
                let g = (hash(nx, ny, colour_seed) - 0.5) * gl;
                for v in &mut c {
                    *v += g;
                }
            }
            let [r, g, b] = c.map(store);
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, w, h);
}
