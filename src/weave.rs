use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "weave";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 24;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 7] = [
    "#22223b", "#f2e9e4", "#c9ada7", "#9a8c98", "#4a4e69", "#e63946", "#f4d35e",
];

const BANDS: Param = Param::new(SLUG, "bands", 3, 12, 1);
const STRIPE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "stripe", 50, 220, 100)
};

pub(crate) const PARAMS: &[Param] = &[BANDS, STRIPE];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeaveParams {
    bands: u32,
    stripe: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for WeaveParams {
    fn default() -> Self {
        Self {
            bands: 6,
            stripe: 1.0,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl WeaveParams {
    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn stripe(&self) -> f64 {
        self.stripe
    }

    pub fn set_stripe(&mut self, stripe: f64) -> Result<(), Error> {
        self.stripe = STRIPE.check(stripe)?;
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
    bands: u32,
    stripe: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<WeaveParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = WeaveParams::default();
    params.set_bands(raw.bands)?;
    params.set_stripe(raw.stripe)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> WeaveParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    WeaveParams {
        bands: pick(&BANDS) as u32,
        stripe: pick(&STRIPE),
        ..WeaveParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &WeaveParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

const VIEW: f64 = 1000.0;

enum Align {
    Top,
    Bottom,
    Centre,
}

struct Band {
    top: f64,
    bottom: f64,
    ground: usize,
    threads: Vec<usize>,
    pitch: f64,
    duty: f64,
    share: f64,
    align: Align,
    phase: f64,
}

fn bands(params: &WeaveParams, inks: usize, view_height: f64, tool_seed: u32) -> Vec<Band> {
    let mut rng = Xorshift::new(tool_seed);
    let n = params.bands as usize;
    let weights: Vec<f64> = (0..n).map(|_| 0.55 + rng.next() * 1.3).collect();
    let sum: f64 = weights.iter().sum();
    let pick = |rng: &mut Xorshift| (rng.next() * inks as f64) as usize;
    let mut top = 0.0;
    let mut out = Vec::with_capacity(n);
    for (i, weight) in weights.iter().enumerate() {
        let bottom = if i == n - 1 {
            view_height
        } else {
            view_height.min(top + weight / sum * view_height)
        };
        let ground = pick(&mut rng);
        let mut first = pick(&mut rng);
        while first == ground && inks > 1 {
            first = pick(&mut rng);
        }
        let two = rng.next() < 0.4 && inks > 2;
        let mut threads = vec![first];
        if two {
            let mut second = pick(&mut rng);
            while second == ground || second == first {
                second = pick(&mut rng);
            }
            threads.push(second);
        }
        let thin = rng.next() < 0.45;
        let pitch = if thin {
            VIEW * (0.012 + rng.next() * 0.02)
        } else {
            VIEW * (0.04 + rng.next() * 0.07)
        } * params.stripe;
        let duty = if thin {
            0.28 + rng.next() * 0.3
        } else {
            0.42 + rng.next() * 0.35
        };
        let extent = rng.next();
        let full = extent < 0.55;
        let share = if full { 1.0 } else { 0.6 + rng.next() * 0.3 };
        let align = if full || rng.next() < 0.5 {
            Align::Top
        } else if rng.next() < 0.5 {
            Align::Bottom
        } else {
            Align::Centre
        };
        rng.next();
        rng.next();
        let phase = rng.next();
        out.push(Band {
            top,
            bottom,
            ground,
            threads,
            pitch,
            duty,
            share,
            align,
            phase,
        });
        top = bottom;
    }
    out
}

fn paint(surface: &mut Surface, params: &WeaveParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (surface.width(), surface.height());
    let view_height = VIEW * f64::from(height) / f64::from(width);
    let k = f64::from(width) / VIEW;
    for band in bands(params, palette.len(), view_height, tool_seed) {
        surface.fill_rect(
            0,
            (band.top * k).floor() as i32,
            width,
            ((band.bottom - band.top) * k).ceil() as u32 + 1,
            palette.ink(band.ground),
        );
        let span = band.bottom - band.top;
        let stripe_height = span * band.share;
        let stripe_top = match band.align {
            Align::Top => band.top,
            Align::Bottom => band.bottom - stripe_height,
            Align::Centre => band.top + (span - stripe_height) / 2.0,
        };
        let rows = round_half_up(stripe_height * k) as u32;
        if rows == 0 {
            continue;
        }
        let y = round_half_up(stripe_top * k) as i32;
        let threads = band.threads.len() as i64;
        let offset = band.phase * band.pitch * threads as f64;
        let stripe_width = band.pitch * band.duty;
        let cols = (round_half_up(stripe_width * k) as u32).max(1);
        let first = ((-offset - stripe_width) / band.pitch).floor() as i64 - 1;
        let last = ((VIEW - offset) / band.pitch).ceil() as i64 + 1;
        for stripe in first..=last {
            let x = stripe as f64 * band.pitch + offset;
            if x + stripe_width < 0.0 || x > VIEW {
                continue;
            }
            let thread = band.threads[stripe.rem_euclid(threads) as usize];
            surface.fill_rect(
                round_half_up(x * k) as i32,
                y,
                cols,
                rows,
                palette.ink(thread),
            );
        }
    }
}
