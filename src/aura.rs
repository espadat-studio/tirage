use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up, store};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "aura";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const BUDGET: f64 = 700.0;
const INKS: usize = 4;

const DEFAULT_PALETTE: [&str; INKS] = ["#ff8a5b", "#ffc15e", "#f4a7d6", "#8e7cff"];

const SCALE: Param = Param {
    step: 5,
    taste: (90, 200),
    ..Param::new(SLUG, "scale", 50, 200, 100)
};
const CHURN: Param = Param {
    taste: (25, 100),
    ..Param::new(SLUG, "churn", 0, 100, 100)
};
const PUNCH: Param = Param {
    taste: (25, 100),
    ..Param::new(SLUG, "punch", 0, 100, 100)
};

pub(crate) const PARAMS: &[Param] = &[SCALE, CHURN, PUNCH];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuraStyle {
    Auto,
    Clouds,
    Mesh,
    Sweep,
}

const STYLES: [AuraStyle; 4] = [
    AuraStyle::Auto,
    AuraStyle::Clouds,
    AuraStyle::Mesh,
    AuraStyle::Sweep,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuraParams {
    #[serde(rename = "styles")]
    style: AuraStyle,
    scale: f64,
    churn: f64,
    punch: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for AuraParams {
    fn default() -> Self {
        Self {
            style: AuraStyle::Clouds,
            scale: 2.0,
            churn: 1.0,
            punch: 1.0,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl AuraParams {
    pub fn style(&self) -> AuraStyle {
        self.style
    }

    pub fn set_style(&mut self, style: AuraStyle) {
        self.style = style;
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn churn(&self) -> f64 {
        self.churn
    }

    pub fn set_churn(&mut self, churn: f64) -> Result<(), Error> {
        self.churn = CHURN.check(churn)?;
        Ok(())
    }

    pub fn punch(&self) -> f64 {
        self.punch
    }

    pub fn set_punch(&mut self, punch: f64) -> Result<(), Error> {
        self.punch = PUNCH.check(punch)?;
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
    #[serde(rename = "styles")]
    style: AuraStyle,
    scale: f64,
    churn: f64,
    punch: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<AuraParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = AuraParams::default();
    params.set_style(raw.style);
    params.set_scale(raw.scale)?;
    params.set_churn(raw.churn)?;
    params.set_punch(raw.punch)?;
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
    std::iter::once(Parameter::choice(
        "styles",
        &["Auto", "Clouds", "Mesh", "Sweep"],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> AuraParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    AuraParams {
        style: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        scale: pick(&SCALE),
        churn: pick(&CHURN),
        punch: pick(&PUNCH),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

struct Xorshift(u32);

impl Xorshift {
    fn new(tool_seed: u32) -> Self {
        let start = (f64::from(tool_seed) * 2_654_435_761.0).rem_euclid(4_294_967_296.0) as u32;
        Self(start.max(1))
    }

    fn next(&mut self) -> f64 {
        let mut a = self.0;
        a ^= a << 13;
        a ^= ((a as i32) >> 17) as u32;
        a ^= a << 5;
        self.0 = a;
        f64::from(a) / 4_294_967_296.0
    }
}

fn hash(x: i32, y: i32, channel: u32, seed: u32) -> f64 {
    let mut n = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ channel.wrapping_mul(1_440_662_683)
        ^ seed.wrapping_mul(1_013_904_223);
    n = (n ^ (n >> 15)).wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(3_266_489_917);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

struct Noise(u32);

impl Noise {
    fn value(&self, x: f64, y: f64, channel: u32) -> f64 {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let u = fx * fx * (3.0 - 2.0 * fx);
        let v = fy * fy * (3.0 - 2.0 * fy);
        let (xi, yi) = (x0 as i32, y0 as i32);
        let corner = |dx: i32, dy: i32| hash(xi + dx, yi + dy, channel, self.0);
        let (a, b, c, d) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    fn fbm(&self, x: f64, y: f64, channel: u32) -> f64 {
        self.value(x, y, channel) * 0.55
            + self.value(x * 2.13, y * 2.13, channel + 11) * 0.28
            + self.value(x * 4.31, y * 4.31, channel + 23) * 0.17
    }

    fn fbm2(&self, x: f64, y: f64, channel: u32) -> f64 {
        self.value(x, y, channel) * 0.62 + self.value(x * 2.13, y * 2.13, channel + 11) * 0.38
    }
}

struct Ink {
    square: [f64; 3],
    offset: (f64, f64),
    centre: (f64, f64),
    direction: (f64, f64),
}

pub(crate) fn paint(surface: &mut Surface, params: &AuraParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let bw = round_half_up(width).clamp(90.0, BUDGET);
    let bh = round_half_up(bw * height / width).max(90.0);
    let mut rng = Xorshift::new(tool_seed);
    let style = match params.style {
        AuraStyle::Auto => STYLES[1 + (rng.next() * 3.0) as usize],
        style => style,
    };
    let inks: Vec<Ink> = (0..INKS)
        .map(|slot| {
            let square = palette.ink(slot).map(|c| f64::from(c) * f64::from(c));
            let offset = (rng.next() * 37.0, rng.next() * 43.0);
            let centre = ((rng.next() - 0.5) * 1.5, (rng.next() - 0.5) * 1.5);
            let angle = rng.next() * TAU;
            Ink {
                square,
                offset,
                centre,
                direction: (angle.cos(), angle.sin()),
            }
        })
        .collect();
    let noise = Noise(tool_seed ^ 0x51ed_270b);
    let exponent = 1.3 + params.punch * 5.2;
    let warp = 0.4 + params.churn * 2.4;
    let short = bw.min(bh);
    let zoom = 1.7 * params.scale;

    let mut rgba = Vec::with_capacity((bw * bh) as usize * 4);
    for y in 0..bh as u32 {
        let ny = (f64::from(y) - bh / 2.0) / short * zoom;
        for x in 0..bw as u32 {
            let nx = (f64::from(x) - bw / 2.0) / short * zoom;
            let q1 = noise.fbm(nx + 11.3, ny + 7.9, 81);
            let q2 = noise.fbm(nx + 3.7, ny + 19.1, 82);
            let (wx, wy) = (nx + warp * (q1 - 0.5), ny + warp * (q2 - 0.5));
            let (mut total, mut mix) = (0.0, [0.0; 3]);
            for (channel, ink) in (60..).zip(&inks) {
                let nz = noise.fbm2(wx * 1.15 + ink.offset.0, wy * 1.15 + ink.offset.1, channel);
                let strength = match style {
                    AuraStyle::Mesh => {
                        let dx = wx * 0.8 - ink.centre.0;
                        let dy = wy * 0.8 - ink.centre.1;
                        (-(dx * dx + dy * dy) * 2.6).exp() * 0.72 + nz * 0.34
                    }
                    AuraStyle::Sweep => {
                        let along = wx * ink.direction.0 + wy * ink.direction.1;
                        (0.5 + along * 0.5).clamp(0.0, 1.0) * 0.66 + nz * 0.4
                    }
                    _ => nz,
                };
                let weight = strength.max(0.002).powf(exponent);
                total += weight;
                for (sum, square) in mix.iter_mut().zip(ink.square) {
                    *sum += weight * square;
                }
            }
            let [r, g, b] = mix.map(|sum| store((sum / total).sqrt()));
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, bw as u32, bh as u32);
}
