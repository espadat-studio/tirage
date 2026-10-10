use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "optic";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const INKS: usize = 2;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#f5f0e6", "#7b2cbf"];
const SIZES: [f64; 3] = [1.0, 0.55, 0.22];
const MOVES: [(f64, f64); 4] = [
    (0.0, 0.5),
    (FRAC_PI_2, 0.0),
    (FRAC_PI_4, 0.0),
    (-FRAC_PI_4, 0.0),
];
const POOL: [usize; 5] = [0, 0, 1, 2, 3];
const LAP: f64 = 2.0;

const COUNT: Param = Param::new(SLUG, "count", 6, 28, 1);
const WEIGHT: Param = Param::new(SLUG, "weight", 30, 70, 100);
const SIZE_F: Param = Param::new(SLUG, "sizeF", 40, 100, 100);
const LEVELS: Param = Param::new(SLUG, "levels", 1, 3, 1);

pub(crate) const PARAMS: &[Param] = &[COUNT, WEIGHT, SIZE_F, LEVELS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpticStyle {
    Auto,
    Diamond,
    Circle,
    Peak,
    Square,
}

const STYLES: [OpticStyle; 5] = [
    OpticStyle::Auto,
    OpticStyle::Diamond,
    OpticStyle::Circle,
    OpticStyle::Peak,
    OpticStyle::Square,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpticParams {
    styles: OpticStyle,
    count: u32,
    weight: f64,
    #[serde(rename = "sizeF")]
    size_f: f64,
    levels: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for OpticParams {
    fn default() -> Self {
        Self {
            styles: OpticStyle::Auto,
            count: 14,
            weight: 0.5,
            size_f: 0.88,
            levels: 2,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl OpticParams {
    pub fn styles(&self) -> OpticStyle {
        self.styles
    }

    pub fn set_styles(&mut self, styles: OpticStyle) {
        self.styles = styles;
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
        Ok(())
    }

    pub fn size_f(&self) -> f64 {
        self.size_f
    }

    pub fn set_size_f(&mut self, size_f: f64) -> Result<(), Error> {
        self.size_f = SIZE_F.check(size_f)?;
        Ok(())
    }

    pub fn levels(&self) -> u32 {
        self.levels
    }

    pub fn set_levels(&mut self, levels: u32) -> Result<(), Error> {
        LEVELS.check(f64::from(levels))?;
        self.levels = levels;
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
    styles: OpticStyle,
    count: u32,
    weight: f64,
    #[serde(rename = "sizeF")]
    size_f: f64,
    levels: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<OpticParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = OpticParams::default();
    params.set_styles(raw.styles);
    params.set_count(raw.count)?;
    params.set_weight(raw.weight)?;
    params.set_size_f(raw.size_f)?;
    params.set_levels(raw.levels)?;
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
        &["Auto", "Diamond", "Circle", "Peak", "Square"],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> OpticParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    OpticParams {
        styles: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        count: pick(&COUNT) as u32,
        weight: pick(&WEIGHT),
        size_f: pick(&SIZE_F),
        levels: pick(&LEVELS) as u32,
        ..OpticParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &OpticParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Figure {
    Diamond,
    Circle,
    Peak,
    Square,
}

fn plan(p: &OpticParams, tool_seed: u32) -> (Figure, Vec<(f64, f64)>) {
    let mut rng = Xorshift::new(tool_seed);
    let figure = match p.styles {
        OpticStyle::Auto => [
            Figure::Diamond,
            Figure::Circle,
            Figure::Peak,
            Figure::Square,
        ][(rng.next() * 4.0) as usize],
        OpticStyle::Diamond => Figure::Diamond,
        OpticStyle::Circle => Figure::Circle,
        OpticStyle::Peak => Figure::Peak,
        OpticStyle::Square => Figure::Square,
    };
    let mut previous = None;
    let moves = (0..p.levels)
        .map(|_| {
            let mut idx = POOL[(rng.next() * POOL.len() as f64) as usize];
            if previous == Some(idx) {
                idx = (idx + 1) % MOVES.len();
            }
            previous = Some(idx);
            MOVES[idx]
        })
        .collect();
    (figure, moves)
}

fn outline(figure: Figure, cx: f64, cy: f64, s: f64) -> Path2D {
    let mut path = Path2D::default();
    let mut polygon = |points: &[(f64, f64)]| {
        path.move_to(points[0].0, points[0].1);
        for &(x, y) in &points[1..] {
            path.line_to(x, y);
        }
        path.close();
    };
    match figure {
        Figure::Circle => path.push_circle(cx, cy, s),
        Figure::Square => {
            let q = s * 0.9;
            path.rect(cx - q, cy - q, 2.0 * q, 2.0 * q);
        }
        Figure::Diamond => polygon(&[(cx, cy - s), (cx + s, cy), (cx, cy + s), (cx - s, cy)]),
        Figure::Peak => polygon(&[
            (cx, cy - s * 0.92),
            (cx + s * 1.05, cy + s * 0.8),
            (cx - s * 1.05, cy + s * 0.8),
        ]),
    }
    path
}

fn paint(surface: &mut Surface, p: &OpticParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let (cx, cy) = (w / 2.0, h / 2.0);
    let md = w.min(h);
    let period = md / f64::from(p.count.max(4));
    let bar = period * p.weight;
    let reach = w.hypot(h) / 2.0 + period * (LAP + 2.0);
    let (base, ink) = (palette.ink(0), palette.ink(1));
    let bars = |surface: &mut Surface, turn: f64, phase: f64| {
        let first = ((-reach - phase - bar) / period).ceil() as i64;
        let last = ((reach - phase + bar) / period).floor() as i64;
        let offsets = (first..=last).map(|k| k as f64 * period + phase - bar / 2.0);
        if turn == 0.0 {
            for x in offsets {
                let (x0, x1) = (round_half_up(cx + x), round_half_up(cx + x + bar));
                surface.fill_box(x0, -2.0, x1 - x0, h + 4.0, ink);
            }
            return;
        }
        let turned = Transform::IDENTITY.translate(cx, cy).rotate(turn);
        for x in offsets {
            let mut rect = Path2D::default();
            rect.rect(x, -reach, bar, 2.0 * reach);
            surface.fill_transformed(&rect, ink, turned);
        }
    };
    let (figure, moves) = plan(p, tool_seed);
    surface.fill(base);
    bars(surface, 0.0, 0.0);
    for (i, (turn, phase)) in moves.into_iter().enumerate() {
        let s = p.size_f * md / 2.0 * SIZES[i];
        surface.with_clip(
            &outline(figure, cx, cy, s),
            Transform::IDENTITY,
            |surface| {
                surface.fill(base);
                bars(surface, turn, phase * period);
            },
        );
    }
}
