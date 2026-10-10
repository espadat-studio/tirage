use serde::{Deserialize, Serialize};

use crate::aura::{Noise, Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up, store};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "mist";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const BUDGET: f64 = 700.0;
const INKS: usize = 4;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#dcff3a", "#1e1b3a", "#3b2a6e", "#5a2e7a"];

const STREAKS: Param = Param {
    step: 5,
    ..Param::new(SLUG, "streaks", 40, 200, 100)
};
const COVER: Param = Param {
    taste: (30, 60),
    ..Param::new(SLUG, "cover", 20, 85, 100)
};
const SOFT: Param = Param::new(SLUG, "soft", 10, 100, 100);
const BLOT: Param = Param::new(SLUG, "blot", 20, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[STREAKS, COVER, SOFT, BLOT];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MistParams {
    streaks: f64,
    cover: f64,
    soft: f64,
    blot: f64,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for MistParams {
    fn default() -> Self {
        Self {
            streaks: 1.0,
            cover: 0.55,
            soft: 0.5,
            blot: 0.5,
            grain: 0.4,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl MistParams {
    pub fn streaks(&self) -> f64 {
        self.streaks
    }

    pub fn set_streaks(&mut self, streaks: f64) -> Result<(), Error> {
        self.streaks = STREAKS.check(streaks)?;
        Ok(())
    }

    pub fn cover(&self) -> f64 {
        self.cover
    }

    pub fn set_cover(&mut self, cover: f64) -> Result<(), Error> {
        self.cover = COVER.check(cover)?;
        Ok(())
    }

    pub fn soft(&self) -> f64 {
        self.soft
    }

    pub fn set_soft(&mut self, soft: f64) -> Result<(), Error> {
        self.soft = SOFT.check(soft)?;
        Ok(())
    }

    pub fn blot(&self) -> f64 {
        self.blot
    }

    pub fn set_blot(&mut self, blot: f64) -> Result<(), Error> {
        self.blot = BLOT.check(blot)?;
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
    streaks: f64,
    cover: f64,
    soft: f64,
    blot: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<MistParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = MistParams::default();
    params.set_streaks(raw.streaks)?;
    params.set_cover(raw.cover)?;
    params.set_soft(raw.soft)?;
    params.set_blot(raw.blot)?;
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
        .chain([&GRAIN])
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> MistParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    MistParams {
        streaks: pick(&STREAKS),
        cover: pick(&COVER),
        soft: pick(&SOFT),
        blot: pick(&BLOT),
        grain: 0.4,
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let width = e1 - e0;
    let t = ((x - e0) / if width == 0.0 { 1e-6 } else { width }).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &MistParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &MistParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let bw = round_half_up(width).clamp(90.0, BUDGET);
    let bh = round_half_up(bw * height / width).max(90.0);
    let square = |slot: usize| palette.ink(slot).map(|c| f64::from(c) * f64::from(c));
    let neon = square(0);
    let mut rng = Xorshift::new(tool_seed);
    let ground: Vec<([f64; 3], (f64, f64))> = (1..INKS)
        .map(|slot| (square(slot), (rng.next() * 37.0, rng.next() * 43.0)))
        .collect();
    let (sox, soy) = (rng.next() * 53.0, rng.next() * 29.0);
    let noise = Noise(tool_seed ^ 0x51ed_270b);
    let th = 1.0 - params.cover * 0.9;
    let del = 0.06 + params.soft * 0.24;
    let stf = 2.6 * params.streaks + 1.2;
    let bsc = 0.9 + params.blot * 2.2;
    let gr = params.grain * 13.0;
    let short_edge = bw.min(bh);

    let mut rgba = Vec::with_capacity((bw * bh) as usize * 4);
    for y in 0..bh as u32 {
        let ny = (f64::from(y) - bh / 2.0) / short_edge * 1.7;
        for x in 0..bw as u32 {
            let nx = (f64::from(x) - bw / 2.0) / short_edge * 1.7;
            let (gx, gy) = (nx * bsc, ny * bsc);
            let (mut total, mut mix) = (0.0, [0.0; 3]);
            for (channel, (ink, (ox, oy))) in (60..).zip(&ground) {
                let nz = noise.fbm2(gx + ox, gy + oy, channel);
                let weight = nz.max(0.002).powf(3.4);
                total += weight;
                for (sum, square) in mix.iter_mut().zip(ink) {
                    *sum += weight * square;
                }
            }
            let q = noise.fbm(nx * 1.9 + 3.1, ny * 1.9 + 8.7, 91);
            let sx = nx * stf + 0.5 * (q - 0.5) + sox;
            let sy = ny * 0.5 + soy;
            let st = noise.fbm(sx, sy, 77) * 0.72 + noise.fbm(sx * 2.4, sy * 2.1, 78) * 0.28;
            let m = smoothstep(th - del, th + del, st);
            let g = (hash(x as i32, y as i32, 7, noise.0) - 0.5) * gr;
            let [r, gg, b] = std::array::from_fn(|i| {
                let c = mix[i] / total;
                store((c + (neon[i] - c) * m).sqrt() + g)
            });
            rgba.extend([r, gg, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, bw as u32, bh as u32);
}
