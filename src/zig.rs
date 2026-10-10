use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "zig";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 24;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#1b1b1b", "#f94144", "#f8961e", "#f9c74f", "#43aa8b", "#577590",
];

const WIDTH: Param = Param {
    step: 5,
    ..Param::new(SLUG, "width", 50, 200, 100)
};
const DEPTH: Param = Param {
    step: 2,
    ..Param::new(SLUG, "depth", 20, 140, 100)
};
const TOOTH: Param = Param {
    step: 5,
    ..Param::new(SLUG, "tooth", 50, 200, 100)
};
const ROUND: Param = Param::new(SLUG, "round", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[WIDTH, DEPTH, TOOTH, ROUND];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigStyle {
    Teeth,
    Chevron,
    Stairs,
    Ricrac,
    Waves,
    Scales,
}

const STYLES: [ZigStyle; 6] = [
    ZigStyle::Teeth,
    ZigStyle::Chevron,
    ZigStyle::Stairs,
    ZigStyle::Ricrac,
    ZigStyle::Waves,
    ZigStyle::Scales,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZigParams {
    #[serde(rename = "styles")]
    style: ZigStyle,
    width: f64,
    depth: f64,
    tooth: f64,
    round: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for ZigParams {
    fn default() -> Self {
        Self {
            style: ZigStyle::Teeth,
            width: 1.0,
            depth: 0.9,
            tooth: 1.0,
            round: 0.8,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl ZigParams {
    pub fn style(&self) -> ZigStyle {
        self.style
    }

    pub fn set_style(&mut self, style: ZigStyle) {
        self.style = style;
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) -> Result<(), Error> {
        self.width = WIDTH.check(width)?;
        Ok(())
    }

    pub fn depth(&self) -> f64 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: f64) -> Result<(), Error> {
        self.depth = DEPTH.check(depth)?;
        Ok(())
    }

    pub fn tooth(&self) -> f64 {
        self.tooth
    }

    pub fn set_tooth(&mut self, tooth: f64) -> Result<(), Error> {
        self.tooth = TOOTH.check(tooth)?;
        Ok(())
    }

    pub fn round(&self) -> f64 {
        self.round
    }

    pub fn set_round(&mut self, round: f64) -> Result<(), Error> {
        self.round = ROUND.check(round)?;
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
    style: ZigStyle,
    width: f64,
    depth: f64,
    tooth: f64,
    round: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<ZigParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = ZigParams::default();
    params.set_style(raw.style);
    params.set_width(raw.width)?;
    params.set_depth(raw.depth)?;
    params.set_tooth(raw.tooth)?;
    params.set_round(raw.round)?;
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
        &["Teeth", "Chevron", "Stairs", "Ricrac", "Waves", "Scales"],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> ZigParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    ZigParams {
        style: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        width: pick(&WIDTH),
        depth: pick(&DEPTH),
        tooth: pick(&TOOTH),
        round: pick(&ROUND),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &ZigParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

const BOX: f64 = 1000.0;

type Point = (f64, f64);

fn hash(x: i32, y: i32, z: i32, seed: u32) -> f64 {
    let mut n = (x.wrapping_mul(374_761_393)
        ^ y.wrapping_mul(668_265_263)
        ^ z.wrapping_mul(1_440_662_683)
        ^ (seed as i32).wrapping_mul(1_013_904_223)) as u32;
    n = (n ^ (n >> 15)).wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(3_266_489_917);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

fn paint(surface: &mut Surface, params: &ZigParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let tall = BOX * height / width;
    let k = width / BOX;
    let n = palette.len();
    let ground = (hash(0, 7, 1, tool_seed) * n as f64) as usize;
    let mut stripe = (hash(0, 7, 2, tool_seed) * n as f64) as usize;
    if stripe == ground && n > 1 {
        stripe = (ground + 1) % n;
    }
    surface.fill(palette.ink(ground));
    let lines = lines(params, tall);
    let radius = corner_radius(params, tall);
    for pair in lines.windows(2).step_by(2) {
        let polygon: Vec<Point> = pair[0]
            .iter()
            .chain(pair[1].iter().rev())
            .map(|&(x, y)| match params.style {
                ZigStyle::Chevron => (y, x),
                _ => (x, y),
            })
            .collect();
        surface.fill_path(
            &rounded(&polygon, radius, k).into_path(),
            palette.ink(stripe),
        );
    }
}

fn lines(params: &ZigParams, tall: f64) -> Vec<Vec<Point>> {
    match params.style {
        ZigStyle::Teeth | ZigStyle::Chevron => teeth(params, tall),
        ZigStyle::Stairs => stairs(params, tall),
        ZigStyle::Ricrac | ZigStyle::Waves | ZigStyle::Scales => rows(params, tall),
    }
}

fn spans(params: &ZigParams, tall: f64) -> (f64, f64) {
    match params.style {
        ZigStyle::Teeth => (BOX, tall),
        _ => (tall, BOX),
    }
}

fn teeth(params: &ZigParams, tall: f64) -> Vec<Vec<Point>> {
    let (span, cross) = spans(params, tall);
    let stripe = span * 0.125 * params.width;
    let reach = stripe * params.depth * 0.5;
    let half = span * 0.055 * params.tooth;
    let period = 2.0 * half;
    let count = (span / stripe).ceil() as i32 + 3;
    let last = (cross / period).ceil() as i32 + 8;
    (-1..count)
        .map(|j| {
            let base = f64::from(j) * stripe;
            let offset = if j % 2 != 0 { half } else { 0.0 };
            (-8..=last)
                .flat_map(|i| {
                    let y0 = f64::from(i) * period + offset;
                    let x = base + if i.rem_euclid(2) == 0 { reach } else { -reach };
                    [(x, y0), (x, y0 + period)]
                })
                .collect()
        })
        .collect()
}

fn stairs(params: &ZigParams, tall: f64) -> Vec<Vec<Point>> {
    let step = BOX * 0.07 * params.tooth * params.width;
    let spacing = round_half_up(2.0 * params.width).max(1.0) * step * 2.0;
    let count = ((BOX + tall + spacing * 4.0) / spacing).ceil() as i32 + 2;
    (-2..count)
        .map(|j| {
            let mut points = Vec::new();
            let mut x = -tall - spacing * 3.0 + f64::from(j) * spacing;
            let mut y = -step * 4.0;
            while y < tall + step * 4.0 {
                points.push((x, y));
                x += step;
                points.push((x, y));
                y += step;
            }
            points
        })
        .collect()
}

fn rows(params: &ZigParams, tall: f64) -> Vec<Vec<Point>> {
    let style = params.style;
    let row = tall * 0.115 * params.width;
    let period = BOX * 0.14 * params.tooth;
    let amp = match style {
        ZigStyle::Scales => period * 0.5,
        ZigStyle::Ricrac => row * 0.42 * 1.2,
        _ => row * 0.42,
    } * params.depth;
    let count = (tall / row).ceil() as i32 + 3;
    (-2..count)
        .map(|j| {
            let base = f64::from(j) * row;
            if style == ZigStyle::Ricrac {
                let half = period / 2.0;
                let first = (-half * 4.0 / half).floor() as i32;
                let last = ((BOX + half * 4.0) / half).ceil() as i32;
                return (first..=last)
                    .map(|i| {
                        let y = if i.rem_euclid(2) == 0 { -amp } else { amp };
                        (f64::from(i) * half, base + y)
                    })
                    .collect();
            }
            let offset = if style == ZigStyle::Scales && j.rem_euclid(2) != 0 {
                period / 2.0
            } else {
                0.0
            };
            let mut points = Vec::new();
            let mut x = -period * 3.0;
            while x <= BOX + period * 3.0 {
                let u = (x + offset) / period * TAU;
                let y = match style {
                    ZigStyle::Waves => u.sin() * amp,
                    _ => -(u / 2.0).sin().abs() * amp,
                };
                points.push((x, base + y));
                x += period / 14.0;
            }
            points
        })
        .collect()
}

fn corner_radius(params: &ZigParams, tall: f64) -> f64 {
    let base = match params.style {
        ZigStyle::Ricrac => BOX * 0.07 * params.tooth,
        ZigStyle::Waves | ZigStyle::Scales => return 6.0,
        ZigStyle::Stairs => BOX * 0.07 * params.tooth * params.width,
        ZigStyle::Teeth | ZigStyle::Chevron => (BOX * 0.125 * params.width * params.depth * 0.5)
            .min(spans(params, tall).0 * 0.055 * params.tooth),
    };
    (base * 0.9 * params.round).max(0.0)
}

fn rounded(polygon: &[Point], radius: f64, k: f64) -> Path2D {
    let n = polygon.len();
    let at = |i: usize| polygon[i % n];
    let mut path = Path2D::default();
    for i in 0..n {
        let (p0, p1, p2) = (at(i), at(i + 1), at(i + 2));
        let l1 = nonzero((p1.0 - p0.0).hypot(p1.1 - p0.1));
        let l2 = nonzero((p2.0 - p1.0).hypot(p2.1 - p1.1));
        let cos = ((p0.0 - p1.0) * (p2.0 - p1.0) + (p0.1 - p1.1) * (p2.1 - p1.1)) / (l1 * l2);
        let tan = nonzero((cos.clamp(-1.0, 1.0).acos() / 2.0).tan());
        let trim = (radius / tan).min(l1 / 2.0).min(l2 / 2.0);
        let safe = (trim * tan).max(0.0);
        if i == 0 {
            path.move_to((p0.0 + p1.0) / 2.0 * k, (p0.1 + p1.1) / 2.0 * k);
        }
        path.arc_to(p1.0 * k, p1.1 * k, p2.0 * k, p2.1 * k, safe * k);
    }
    path.close();
    path
}

fn nonzero(value: f64) -> f64 {
    if value == 0.0 { 1.0 } else { value }
}
