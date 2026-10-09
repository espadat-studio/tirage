use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, store, value_noise};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "benday";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 8] = [
    "#0a0a0c", "#5b1e9e", "#2c4be8", "#31c6f0", "#8fe23a", "#f5f03a", "#ff8a1e", "#ff2a1e",
];

const TURB: Param = Param::new(SLUG, "turb", 0, 100, 100);
const STREAK: Param = Param::new(SLUG, "streak", 0, 100, 100);
const DIR: Param = Param::new(SLUG, "dir", -100, 100, 100);
const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const BLACK: Param = Param {
    taste: (0, 60),
    ..Param::new(SLUG, "black", 0, 100, 100)
};
const BANDS: Param = Param::new(SLUG, "bands", 2, 16, 1);
const RIMS: Param = Param::new(SLUG, "rims", 0, 100, 100);
const DOT: Param = Param::new(SLUG, "dot", 0, 100, 100);
const RING: Param = Param::new(SLUG, "ring", 0, 100, 100);
const ANGLE: Param = Param::new(SLUG, "angle", 0, 100, 100);
const BITE: Param = Param::new(SLUG, "bite", 0, 100, 100);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BendayParams {
    turb: f64,
    streak: f64,
    dir: f64,
    scale: f64,
    black: f64,
    bands: u32,
    rims: f64,
    dot: f64,
    ring: f64,
    angle: f64,
    bite: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for BendayParams {
    fn default() -> Self {
        Self {
            turb: 0.7,
            streak: 0.5,
            dir: 0.35,
            scale: 0.5,
            black: 0.35,
            bands: 8,
            rims: 0.12,
            dot: 0.45,
            ring: 0.6,
            angle: 0.25,
            bite: 0.5,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl BendayParams {
    pub fn turb(&self) -> f64 {
        self.turb
    }

    pub fn set_turb(&mut self, turb: f64) -> Result<(), Error> {
        self.turb = TURB.check(turb)?;
        Ok(())
    }

    pub fn streak(&self) -> f64 {
        self.streak
    }

    pub fn set_streak(&mut self, streak: f64) -> Result<(), Error> {
        self.streak = STREAK.check(streak)?;
        Ok(())
    }

    pub fn dir(&self) -> f64 {
        self.dir
    }

    pub fn set_dir(&mut self, dir: f64) -> Result<(), Error> {
        self.dir = DIR.check(dir)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn black(&self) -> f64 {
        self.black
    }

    pub fn set_black(&mut self, black: f64) -> Result<(), Error> {
        self.black = BLACK.check(black)?;
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

    pub fn rims(&self) -> f64 {
        self.rims
    }

    pub fn set_rims(&mut self, rims: f64) -> Result<(), Error> {
        self.rims = RIMS.check(rims)?;
        Ok(())
    }

    pub fn dot(&self) -> f64 {
        self.dot
    }

    pub fn set_dot(&mut self, dot: f64) -> Result<(), Error> {
        self.dot = DOT.check(dot)?;
        Ok(())
    }

    pub fn ring(&self) -> f64 {
        self.ring
    }

    pub fn set_ring(&mut self, ring: f64) -> Result<(), Error> {
        self.ring = RING.check(ring)?;
        Ok(())
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn set_angle(&mut self, angle: f64) -> Result<(), Error> {
        self.angle = ANGLE.check(angle)?;
        Ok(())
    }

    pub fn bite(&self) -> f64 {
        self.bite
    }

    pub fn set_bite(&mut self, bite: f64) -> Result<(), Error> {
        self.bite = BITE.check(bite)?;
        Ok(())
    }

    pub fn dither(&self) -> &Dither {
        &self.dither
    }

    pub fn dither_mut(&mut self) -> &mut Dither {
        &mut self.dither
    }

    pub fn grain_pass(&self) -> &Grain {
        &self.grain
    }

    pub fn grain_pass_mut(&mut self) -> &mut Grain {
        &mut self.grain
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    turb: f64,
    streak: f64,
    dir: f64,
    scale: f64,
    black: f64,
    bands: u32,
    rims: f64,
    dot: f64,
    ring: f64,
    angle: f64,
    bite: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<BendayParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = BendayParams::default();
    params.set_turb(raw.turb)?;
    params.set_streak(raw.streak)?;
    params.set_dir(raw.dir)?;
    params.set_scale(raw.scale)?;
    params.set_black(raw.black)?;
    params.set_bands(raw.bands)?;
    params.set_rims(raw.rims)?;
    params.set_dot(raw.dot)?;
    params.set_ring(raw.ring)?;
    params.set_angle(raw.angle)?;
    params.set_bite(raw.bite)?;
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

pub(crate) const PARAMS: &[Param] = &[
    TURB, STREAK, DIR, SCALE, BLACK, BANDS, RIMS, DOT, RING, ANGLE, BITE,
];

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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> BendayParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    BendayParams {
        turb: pick(&TURB),
        streak: pick(&STREAK),
        dir: pick(&DIR),
        scale: pick(&SCALE),
        black: pick(&BLACK),
        bands: pick(&BANDS) as u32,
        rims: pick(&RIMS),
        dot: pick(&DOT),
        ring: pick(&RING),
        angle: pick(&ANGLE),
        bite: pick(&BITE),
        ..BendayParams::default()
    }
}

const PAPER: [f64; 3] = [244.0, 241.0, 232.0];
const DOT_MIN: f64 = 0.13;
const DOT_MAX: f64 = 0.7;

pub(crate) fn render(
    surface: &mut Surface,
    params: &BendayParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &BendayParams, palette: &Palette, tool_seed: u32) {
    let inks: Vec<[f64; 3]> = palette
        .inks()
        .iter()
        .map(|ink| ink.map(f64::from))
        .collect();
    let last = inks.len() - 1;
    let ramp = |t: f64| {
        let p = t.clamp(0.0, 1.0) * last as f64;
        let k = (last - 1).min(p.floor() as usize);
        let w = p - k as f64;
        let (c0, c1) = (inks[k], inks[k + 1]);
        [0, 1, 2].map(|c| c0[c] + (c1[c] - c0[c]) * w)
    };
    let seed = |k: u32| tool_seed.wrapping_mul(7).wrapping_add(k);
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let u = (w * h).sqrt();
    let sc = u * (0.25 + params.scale * 0.7);
    let da = params.dir * PI / 2.0;
    let (cd, sd) = (da.cos(), da.sin());
    let stretch = 1.0 + params.streak * 3.5;
    let black = params.black * 0.6;
    let bands = f64::from(params.bands.max(2));
    let rim = params.rims * 0.5;
    let cell = (u * (0.006 + params.dot * 0.03)).max(2.0);
    let aa = params.angle * PI / 180.0 * 80.0;
    let (ca, sa) = (aa.cos(), aa.sin());
    let gamma = 1.0 / (0.45 + params.bite * 1.4);
    let core = params.ring * 0.8;

    surface.edit_rgba(|rgba, width, _| {
        for (y, row) in rgba.chunks_exact_mut(width as usize * 4).enumerate() {
            let y = y as f64;
            let v0 = (y - h * 0.5) / sc;
            for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let x = x as f64;
                let u0 = (x - w * 0.5) / sc;
                let ru = (u0 * cd + v0 * sd) / stretch;
                let rv = -u0 * sd + v0 * cd;
                let q1 = value_noise(ru * 1.5 + 3.1, rv * 1.5 + 7.7, seed(1));
                let q2 = value_noise(ru * 1.5 + 9.2, rv * 1.5 + 1.3, seed(2));
                let wu = ru + params.turb * (q1 - 0.5) * 3.0;
                let wv = rv + params.turb * (q2 - 0.5) * 3.0;
                let m = value_noise(wu * 2.0, wv * 2.0, seed(3)) * 0.65
                    + value_noise(wu * 4.7, wv * 4.7, seed(4)) * 0.35;
                let val = (m - 0.5) * 1.9 + 0.5;
                let (b, mut c) = if val < black {
                    (0.0, inks[0])
                } else {
                    let t = ((val - black) / (1.0 - black)).min(0.9999);
                    let kb = (t * bands).floor();
                    let fb = t * bands - kb;
                    let band = if rim > 0.0 && (fb < rim || fb > 1.0 - rim) {
                        ramp(((kb + 2.0) / (bands - 1.0)).min(1.0))
                    } else {
                        ramp(kb / (bands - 1.0))
                    };
                    ((kb + 0.5) / bands, band)
                };
                let gx = (x * ca + y * sa) / cell;
                let gy = (-x * sa + y * ca) / cell;
                let (dx, dy) = (gx - gx.floor() - 0.5, gy - gy.floor() - 0.5);
                let d = (dx * dx + dy * dy).sqrt();
                let r = DOT_MIN + (DOT_MAX - DOT_MIN) * (1.0 - b).powf(gamma);
                let edge = (r - d) * cell + 0.5;
                if edge > 0.0 {
                    let mut dot = inks[0];
                    let ce = (r * core - d) * cell + 0.5;
                    if ce > 0.0 {
                        let lit = if b > 0.0 { 0.7 } else { 0.22 };
                        for ch in 0..3 {
                            let tint = c[ch] + (PAPER[ch] - c[ch]) * lit;
                            dot[ch] += (tint - dot[ch]) * ce.min(1.0);
                        }
                    }
                    for ch in 0..3 {
                        c[ch] += (dot[ch] - c[ch]) * edge.min(1.0);
                    }
                }
                for (out, channel) in px.iter_mut().zip(c) {
                    *out = store(channel);
                }
                px[3] = 255;
            }
        }
    });
}
