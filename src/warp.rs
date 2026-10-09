use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "warp";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const INKS: usize = 5;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#f7f3ea", "#f72585", "#4361ee", "#f7b32b", "#1b1b1b"];

const SCALE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "scale", 50, 200, 100)
};
const WARP: Param = Param::new(SLUG, "warp", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[SCALE, WARP];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpStyle {
    Checker,
    Slash,
}

const STYLES: [WarpStyle; 2] = [WarpStyle::Checker, WarpStyle::Slash];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WarpParams {
    #[serde(rename = "styles")]
    style: WarpStyle,
    scale: f64,
    warp: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for WarpParams {
    fn default() -> Self {
        Self {
            style: WarpStyle::Checker,
            scale: 1.0,
            warp: 0.7,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl WarpParams {
    pub fn style(&self) -> WarpStyle {
        self.style
    }

    pub fn set_style(&mut self, style: WarpStyle) {
        self.style = style;
    }

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
    style: WarpStyle,
    scale: f64,
    warp: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<WarpParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = WarpParams::default();
    params.set_style(raw.style);
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
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
    std::iter::once(Parameter::choice("styles", &["Checker", "Slash"]))
        .chain(PARAMS.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> WarpParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    WarpParams {
        style: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        scale: pick(&SCALE),
        warp: pick(&WARP),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &WarpParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Bend {
    Wave,
    Taper,
    Bulge,
}

struct Spec {
    checker: bool,
    ink: usize,
    cell: f64,
    angle: f64,
    duty: f64,
    bend: Bend,
    strength: f64,
    frequency: f64,
    phase: f64,
    bars: f64,
    inverted: bool,
}

fn spec(style: WarpStyle, tool_seed: u32) -> Spec {
    let start = (f64::from(tool_seed) * 2_654_435_761.0) % 4_294_967_296.0;
    let mut state = (start as u32).max(1);
    let mut draw = || {
        state ^= state << 13;
        state ^= ((state as i32) >> 17) as u32;
        state ^= state << 5;
        f64::from(state) / 4_294_967_296.0
    };
    draw();
    let checker = style == WarpStyle::Checker;
    let ink = 1 + (draw() * 4.0) as usize;
    let cell = 0.09 + draw() * 0.09;
    let sign = if draw() < 0.5 { -1.0 } else { 1.0 };
    let angle = sign * (0.35 + draw() * 0.55);
    let duty = 0.45 + draw() * 0.18;
    draw();
    draw();
    let bend = [Bend::Wave, Bend::Taper, Bend::Bulge][(draw() * 3.0) as usize];
    let strength = 0.5 + draw() * 0.8;
    let frequency = 2.0 + draw() * 3.5;
    let phase = draw() * TAU;
    let bars = 2.5 + draw() * 3.0;
    let inverted = draw() < 0.4;
    Spec {
        checker,
        ink,
        cell,
        angle,
        duty,
        bend,
        strength,
        frequency,
        phase,
        bars,
        inverted,
    }
}

fn shift_hash(x: i32, y: i32, z: i32, seed: u32) -> f64 {
    let mut n = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(1_440_662_683)
        ^ seed.wrapping_mul(1_013_904_223);
    n = (n ^ (n >> 15)).wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(3_266_489_917);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

fn paint(surface: &mut Surface, params: &WarpParams, palette: &Palette, tool_seed: u32) {
    let spec = spec(params.style, tool_seed);
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let md = w.min(h);
    let strength = params.warp * spec.strength;
    let cs = spec.cell * params.scale * md;
    let margin = md * 0.35;
    let bend = |u: f64, v: f64| match spec.bend {
        Bend::Wave => (
            u + strength * 0.15 * (v / md * spec.frequency * 2.1 + spec.phase).sin() * md,
            v + strength * 0.08 * (u / md * spec.frequency * 1.6 + spec.phase * 1.7).sin() * md,
        ),
        Bend::Taper => {
            let g = 1.0 + strength * 0.9 * (v / h - 0.5);
            ((u - w / 2.0) * g + w / 2.0, v)
        }
        Bend::Bulge => {
            let (cx, cy) = (w / 2.0, h / 2.0);
            let (dx, dy) = ((u - cx) / md, (v - cy) / md);
            let k = (dx.hypot(dy) + 1e-6) / 0.72;
            let f = k.powf(1.0 + strength * 1.2) / k;
            (cx + dx * f * md, cy + dy * f * md)
        }
    };
    let mut path = Path2D::default();
    let mut quad = |corners: [(f64, f64); 4]| {
        for side in 0..4 {
            let (a, b) = (corners[side], corners[(side + 1) % 4]);
            for step in 0..5 {
                let t = f64::from(step) / 5.0;
                let (x, y) = bend(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                if side == 0 && step == 0 {
                    path.move_to(x, y);
                } else {
                    path.line_to(x, y);
                }
            }
        }
        path.close();
    };
    if spec.checker {
        let (i0, i1) = (
            (-margin / cs).floor() as i32,
            ((w + margin) / cs).ceil() as i32,
        );
        let (j0, j1) = (
            (-margin / cs).floor() as i32,
            ((h + margin) / cs).ceil() as i32,
        );
        for j in j0..j1 {
            for i in i0..i1 {
                if (i + j).rem_euclid(2) == 1 {
                    continue;
                }
                let (x, y) = (f64::from(i) * cs, f64::from(j) * cs);
                quad([(x, y), (x + cs, y), (x + cs, y + cs), (x, y + cs)]);
            }
        }
    } else {
        let (cos, sin) = (spec.angle.cos(), spec.angle.sin());
        let band = cs * spec.bars;
        let reach = w.hypot(h) + margin;
        let place = |q: f64, r: f64| (q * cos - r * sin, q * sin + r * cos);
        let shift_seed = tool_seed ^ 0x2c1b_3c6d;
        for b in (-reach / band).floor() as i32..(reach / band).ceil() as i32 {
            let shift = shift_hash(b, 3, 7, shift_seed) * cs * 2.0;
            let (r0, r1) = (f64::from(b) * band, f64::from(b + 1) * band);
            for k in ((-reach - shift) / cs).floor() as i32..((reach - shift) / cs).ceil() as i32 {
                let q0 = f64::from(k) * cs + shift;
                let q1 = q0 + cs * spec.duty;
                quad([place(q0, r0), place(q1, r0), place(q1, r1), place(q0, r1)]);
            }
        }
    }
    let (base, ink) = (palette.ink(0), palette.ink(spec.ink));
    let (ground, shapes) = if spec.inverted {
        (ink, base)
    } else {
        (base, ink)
    };
    surface.fill(ground);
    surface.fill_path(&path.into_path(), shapes);
}
