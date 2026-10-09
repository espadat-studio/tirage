use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, hash, round_half_up, value_noise};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "sear";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#03071e", "#370617", "#9d0208", "#e85d04", "#faa307", "#ffe066",
];

const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const WARP: Param = Param::new(SLUG, "warp", 0, 100, 100);
const DETAIL: Param = Param::new(SLUG, "detail", 1, 5, 1);
const SMEAR: Param = Param::new(SLUG, "smear", 0, 100, 100);
const DRAG: Param = Param::new(SLUG, "drag", 0, 100, 100);
const TEAR: Param = Param {
    taste: (0, 60),
    ..Param::new(SLUG, "tear", 0, 100, 100)
};
const STEPS: Param = Param::new(SLUG, "steps", 2, 24, 1);
const GRAIN: Param = Param {
    taste: (0, 70),
    ..Param::new(SLUG, "grain", 0, 100, 100)
};

pub(crate) const PARAMS: &[Param] = &[SCALE, WARP, DETAIL, SMEAR, DRAG, TEAR, STEPS, GRAIN];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearParams {
    scale: f64,
    warp: f64,
    detail: u32,
    smear: f64,
    drag: f64,
    tear: f64,
    steps: u32,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for SearParams {
    fn default() -> Self {
        Self {
            scale: 0.62,
            warp: 0.35,
            detail: 4,
            smear: 0.35,
            drag: 0.22,
            tear: 0.3,
            steps: 6,
            grain: 0.1,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl SearParams {
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

    pub fn detail(&self) -> u32 {
        self.detail
    }

    pub fn set_detail(&mut self, detail: u32) -> Result<(), Error> {
        DETAIL.check(f64::from(detail))?;
        self.detail = detail;
        Ok(())
    }

    pub fn smear(&self) -> f64 {
        self.smear
    }

    pub fn set_smear(&mut self, smear: f64) -> Result<(), Error> {
        self.smear = SMEAR.check(smear)?;
        Ok(())
    }

    pub fn drag(&self) -> f64 {
        self.drag
    }

    pub fn set_drag(&mut self, drag: f64) -> Result<(), Error> {
        self.drag = DRAG.check(drag)?;
        Ok(())
    }

    pub fn tear(&self) -> f64 {
        self.tear
    }

    pub fn set_tear(&mut self, tear: f64) -> Result<(), Error> {
        self.tear = TEAR.check(tear)?;
        Ok(())
    }

    pub fn steps(&self) -> u32 {
        self.steps
    }

    pub fn set_steps(&mut self, steps: u32) -> Result<(), Error> {
        STEPS.check(f64::from(steps))?;
        self.steps = steps;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
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
    scale: f64,
    warp: f64,
    detail: u32,
    smear: f64,
    drag: f64,
    tear: f64,
    steps: u32,
    grain: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SearParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SearParams::default();
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
    params.set_detail(raw.detail)?;
    params.set_smear(raw.smear)?;
    params.set_drag(raw.drag)?;
    params.set_tear(raw.tear)?;
    params.set_steps(raw.steps)?;
    params.set_grain(raw.grain)?;
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
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SearParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SearParams {
        scale: pick(&SCALE),
        warp: pick(&WARP),
        detail: pick(&DETAIL) as u32,
        smear: pick(&SMEAR),
        drag: pick(&DRAG),
        tear: pick(&TEAR),
        steps: pick(&STEPS) as u32,
        grain: pick(&GRAIN),
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &SearParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

const BINS: usize = 512;

fn fbm(x: f64, y: f64, seed: i32, octaves: u32) -> f64 {
    let (mut sum, mut a, mut f, mut total) = (0.0, 0.5, 1.0, 0.0);
    for i in 0..octaves as i32 {
        sum += a * value_noise(x * f, y * f, seed.wrapping_add(i * 131) as u32);
        total += a;
        f *= 2.05;
        a *= 0.58;
    }
    sum / total
}

struct Heat {
    grid: Vec<f32>,
    width: usize,
    height: usize,
}

fn heat(params: &SearParams, seed: i32, aspect: f64) -> Heat {
    let width = if aspect > 1.0 {
        round_half_up(640.0 / aspect).max(8.0)
    } else {
        640.0
    };
    let height = round_half_up(width * aspect).max(8.0);
    let (width, height) = (width as usize, height as usize);
    let octaves = params.detail.clamp(1, 5);
    let span = 0.7 + (1.0 - params.scale) * 7.5;
    let k = params.warp * 2.6;
    let (mut lo, mut hi) = (1.0_f64, 0.0_f64);
    let mut grid = Vec::with_capacity(width * height);
    for j in 0..height {
        let v = (j as f64 / height as f64) * span * aspect;
        for i in 0..width {
            let u = (i as f64 / width as f64) * span;
            let wx = fbm(u * 0.55 + 11.3, v * 0.55 + 4.1, seed.wrapping_add(7), 2) - 0.5;
            let wy = fbm(u * 0.55 + 2.7, v * 0.55 + 19.7, seed.wrapping_add(13), 2) - 0.5;
            let t = fbm(u + wx * k, v + wy * k, seed, octaves);
            grid.push(t as f32);
            lo = lo.min(t);
            hi = hi.max(t);
        }
    }
    let r = if hi - lo > 1e-4 { 1.0 / (hi - lo) } else { 1.0 };
    let bins: Vec<usize> = grid
        .iter()
        .map(|&t| ((f64::from(t) - lo) * r * (BINS - 1) as f64) as i32 as usize)
        .collect();
    let mut counts = [0_u32; BINS];
    for &b in &bins {
        counts[b] += 1;
    }
    let inverse = 1.0 / grid.len() as f64;
    let mut run = 0_u32;
    let map: Vec<f32> = counts
        .iter()
        .enumerate()
        .map(|(b, &count)| {
            let eq = (f64::from(run) + f64::from(count) * 0.5) * inverse;
            let raw = b as f64 / (BINS - 1) as f64;
            run += count;
            (raw + (eq - raw) * 0.88) as f32
        })
        .collect();
    Heat {
        grid: bins.iter().map(|&b| map[b]).collect(),
        width,
        height,
    }
}

fn ramp(palette: &Palette, steps: u32) -> [[u8; 3]; 256] {
    let inks = palette.inks();
    let n = inks.len();
    let steps = f64::from(steps.max(2));
    std::array::from_fn(|i| {
        let t = (i as f64 / 255.0 * steps).floor().min(steps - 1.0) / (steps - 1.0);
        let u = t.clamp(0.0, 1.0) * (n - 1) as f64;
        let k = (u.floor() as usize).min(n - 2);
        let fr = u - k as f64;
        let (a, b) = (inks[k], inks[(k + 1).min(n - 1)]);
        std::array::from_fn(|c| {
            let (a, b) = (f64::from(a[c]), f64::from(b[c]));
            round_half_up(a + (b - a) * fr) as u8
        })
    })
}

fn paint(surface: &mut Surface, params: &SearParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let seed = tool_seed as i32;
    let at = |k: i32| seed.wrapping_add(k) as u32;
    let lut = ramp(palette, params.steps);
    let Heat {
        grid,
        width: fw,
        height: fh,
    } = heat(params, seed, hf / wf);
    let fwf = fw as f64;

    let max_run = round_half_up((0.02 + params.smear * params.smear * 1.2) * wf).max(2.0) as u32;
    let thresh = 0.0015 + (1.0 - params.smear).powf(2.2) * 0.22;
    let drag_px = params.drag * wf * 0.3;
    let band_h = round_half_up(hf * 0.028).max(2.0);
    let kx = fwf / wf;
    let gain = params.grain * 0.34;
    let tear = params.tear;

    let mut rgba = Vec::with_capacity((w * h) as usize * 4);
    for y in 0..h {
        let yf = f64::from(y);
        let band = (yf / band_h).floor();
        let torn = hash(band as i32, 77, at(3)) < tear * 0.62;
        let shift = if torn {
            (hash(band as i32, 91, at(5)) - 0.5) * wf * 1.4 * tear
        } else {
            0.0
        };
        let row_run = if torn { w } else { max_run };
        let row_thr = if torn { thresh + 0.45 * tear } else { thresh };

        let gy = (yf / hf) * fh as f64;
        let fy = gy - gy.floor();
        let y0 = (gy.floor() as usize).min(fh - 1);
        let y1 = (y0 + 1).min(fh - 1);
        let (r0, r1) = (y0 * fw, y1 * fw);
        let pull = drag_px
            * (0.15 + 0.85 * value_noise(yf * 0.014, band * 0.37, at(17)))
            * (0.5 + 0.5 * (yf * 0.031).sin());

        let (mut held, mut run): (Option<f64>, u32) = (None, 0);
        for x in 0..w {
            let sx = (f64::from(x) - pull).max(0.0);
            let mut gx = (sx + shift) * kx;
            gx -= (gx / fwf).floor() * fwf;
            let x0 = gx as usize;
            let fx = gx - x0 as f64;
            let x0 = x0.min(fw - 1);
            let x1 = if x0 + 1 > fw - 1 { 0 } else { x0 + 1 };
            let cell = |k: usize| f64::from(grid[k]);
            let a = cell(r0 + x0) + (cell(r0 + x1) - cell(r0 + x0)) * fx;
            let b = cell(r1 + x0) + (cell(r1 + x1) - cell(r1 + x0)) * fx;
            let v = a + (b - a) * fy;
            let current = match held {
                Some(held) if (v - held).abs() <= row_thr && run < row_run => held,
                _ => {
                    run = 0;
                    v
                }
            };
            held = Some(current);
            run += 1;
            let mut q = current;
            if gain > 0.001 {
                q += (hash(x as i32, y as i32, at(29)) - 0.5) * gain;
            }
            let [r, g, b] = lut[((q * 255.0) as i32).clamp(0, 255) as usize];
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, w, h);
}
