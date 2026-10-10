use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store,
};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "splice";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 7] = [
    "#1b1b1b", "#ff6b35", "#004e89", "#f7c59f", "#efefd0", "#1a936f", "#ff3cac",
];

const SLICES: Param = Param::new(SLUG, "slices", 1, 40, 1);
const SHIFT: Param = Param::new(SLUG, "shift", 0, 100, 100);
const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const TONES: Param = Param::new(SLUG, "tones", 2, 14, 1);
const KEY: Param = Param::new(SLUG, "key", 0, 100, 100);
const SCREEN: Param = Param::new(SLUG, "screen", 0, 100, 100);
const MIX: Param = Param::new(SLUG, "mix", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);
const ARCS: Param = Param::new(SLUG, "arcs", 0, 24, 1);
const WEIGHT: Param = Param::new(SLUG, "weight", 0, 100, 100);
const BEND: Param = Param::new(SLUG, "bend", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    SLICES, SHIFT, SCALE, TONES, KEY, SCREEN, MIX, ARCS, WEIGHT, BEND,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpliceCut {
    Columns,
    Rows,
    Blocks,
}

const CUTS: [SpliceCut; 3] = [SpliceCut::Columns, SpliceCut::Rows, SpliceCut::Blocks];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpliceParams {
    #[serde(rename = "dirs")]
    cut: SpliceCut,
    slices: u32,
    shift: f64,
    scale: f64,
    tones: u32,
    key: f64,
    screen: f64,
    mix: f64,
    grain: f64,
    arcs: u32,
    weight: f64,
    bend: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for SpliceParams {
    fn default() -> Self {
        Self {
            cut: SpliceCut::Columns,
            slices: 1,
            shift: 0.35,
            scale: 0.5,
            tones: 5,
            key: 0.4,
            screen: 0.4,
            mix: 0.7,
            grain: 0.18,
            arcs: 5,
            weight: 0.45,
            bend: 0.55,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl SpliceParams {
    pub fn cut(&self) -> SpliceCut {
        self.cut
    }

    pub fn set_cut(&mut self, cut: SpliceCut) {
        self.cut = cut;
    }

    pub fn slices(&self) -> u32 {
        self.slices
    }

    pub fn set_slices(&mut self, slices: u32) -> Result<(), Error> {
        SLICES.check(f64::from(slices))?;
        self.slices = slices;
        Ok(())
    }

    pub fn shift(&self) -> f64 {
        self.shift
    }

    pub fn set_shift(&mut self, shift: f64) -> Result<(), Error> {
        self.shift = SHIFT.check(shift)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn tones(&self) -> u32 {
        self.tones
    }

    pub fn set_tones(&mut self, tones: u32) -> Result<(), Error> {
        TONES.check(f64::from(tones))?;
        self.tones = tones;
        Ok(())
    }

    pub fn key(&self) -> f64 {
        self.key
    }

    pub fn set_key(&mut self, key: f64) -> Result<(), Error> {
        self.key = KEY.check(key)?;
        Ok(())
    }

    pub fn screen(&self) -> f64 {
        self.screen
    }

    pub fn set_screen(&mut self, screen: f64) -> Result<(), Error> {
        self.screen = SCREEN.check(screen)?;
        Ok(())
    }

    pub fn mix(&self) -> f64 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f64) -> Result<(), Error> {
        self.mix = MIX.check(mix)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn arcs(&self) -> u32 {
        self.arcs
    }

    pub fn set_arcs(&mut self, arcs: u32) -> Result<(), Error> {
        ARCS.check(f64::from(arcs))?;
        self.arcs = arcs;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
        Ok(())
    }

    pub fn bend(&self) -> f64 {
        self.bend
    }

    pub fn set_bend(&mut self, bend: f64) -> Result<(), Error> {
        self.bend = BEND.check(bend)?;
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
    #[serde(rename = "dirs")]
    cut: SpliceCut,
    slices: u32,
    shift: f64,
    scale: f64,
    tones: u32,
    key: f64,
    screen: f64,
    mix: f64,
    grain: f64,
    arcs: u32,
    weight: f64,
    bend: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SpliceParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SpliceParams::default();
    params.set_cut(raw.cut);
    params.set_slices(raw.slices)?;
    params.set_shift(raw.shift)?;
    params.set_scale(raw.scale)?;
    params.set_tones(raw.tones)?;
    params.set_key(raw.key)?;
    params.set_screen(raw.screen)?;
    params.set_mix(raw.mix)?;
    params.set_grain(raw.grain)?;
    params.set_arcs(raw.arcs)?;
    params.set_weight(raw.weight)?;
    params.set_bend(raw.bend)?;
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
    let (before, after) = PARAMS.split_at(7);
    std::iter::once(Parameter::choice("dirs", &["Columns", "Rows", "Blocks"]))
        .chain(
            before
                .iter()
                .chain([&GRAIN])
                .chain(after)
                .map(Param::parameter),
        )
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SpliceParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SpliceParams {
        cut: CUTS[(draw("dirs") % CUTS.len() as u64) as usize],
        slices: pick(&SLICES) as u32,
        shift: pick(&SHIFT),
        scale: pick(&SCALE),
        tones: pick(&TONES) as u32,
        key: pick(&KEY),
        screen: pick(&SCREEN),
        mix: pick(&MIX),
        grain: 0.18,
        arcs: pick(&ARCS) as u32,
        weight: pick(&WEIGHT),
        bend: pick(&BEND),
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &SpliceParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Dots,
    Lines,
    Noise,
    Flat,
}

const SCREENS: [Screen; 4] = [Screen::Dots, Screen::Lines, Screen::Noise, Screen::Flat];

struct Piece {
    bottom: f64,
    off: f64,
    rot: usize,
    sc: f64,
    screen: Screen,
}

struct Column {
    right: f64,
    pieces: Vec<Piece>,
}

struct Arc {
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    rot: f64,
    a0: f64,
    a1: f64,
    k: usize,
}

fn splits(rng: &mut XorShift, m: u32) -> Vec<f64> {
    let gaps: Vec<f64> = (0..m).map(|_| 0.45 + rng.next()).collect();
    let total: f64 = gaps.iter().sum();
    let mut at = 0.0;
    let mut cuts: Vec<f64> = gaps[..gaps.len() - 1]
        .iter()
        .map(|gap| {
            at += gap / total;
            at
        })
        .collect();
    cuts.push(1.0);
    cuts
}

fn cuts(params: &SpliceParams, tool_seed: u32) -> (Vec<Column>, Vec<Arc>) {
    let mut rng = XorShift::new(tool_seed.wrapping_mul(2_654_435_761));
    let n = params.slices;
    let base = SCREENS[(rng.next() * 3.0) as usize];
    let mix = params.mix;
    let piece = |rng: &mut XorShift, bottom: f64| {
        let off = (rng.next() - 0.5) * 2.0;
        let rot = (rng.next() * 97.0) as usize;
        let sc = 0.6 + rng.next() * 1.5;
        let screen = if rng.next() < mix {
            SCREENS[(rng.next() * 4.0) as usize]
        } else {
            base
        };
        Piece {
            bottom,
            off,
            rot,
            sc,
            screen,
        }
    };
    let columns = match params.cut {
        SpliceCut::Rows => {
            let ys = splits(&mut rng, n);
            vec![Column {
                right: 1.0,
                pieces: ys.into_iter().map(|y| piece(&mut rng, y)).collect(),
            }]
        }
        SpliceCut::Blocks => {
            let nx = (round_half_up((f64::from(n) * 1.3).sqrt()) as u32).max(1);
            splits(&mut rng, nx)
                .into_iter()
                .map(|right| {
                    let ny =
                        (round_half_up(f64::from(n) / f64::from(nx) * (0.6 + rng.next() * 0.9))
                            as u32)
                            .max(1);
                    let ys = splits(&mut rng, ny);
                    Column {
                        right,
                        pieces: ys.into_iter().map(|y| piece(&mut rng, y)).collect(),
                    }
                })
                .collect()
        }
        SpliceCut::Columns => splits(&mut rng, n)
            .into_iter()
            .map(|right| Column {
                right,
                pieces: vec![piece(&mut rng, 1.0)],
            })
            .collect(),
    };
    let bend = 0.15 + params.bend * 0.85;
    let arcs = (0..params.arcs)
        .map(|_| {
            let a0 = rng.next() * TAU;
            let cx = -0.3 + rng.next() * 1.6;
            let cy = -0.3 + rng.next() * 1.6;
            let rx = 0.35 + rng.next() * 1.1;
            let ry = (0.35 + rng.next() * 1.1) * bend;
            let rot = rng.next() * PI;
            let a1 = a0 + 0.9 + rng.next() * 2.6;
            let k = (rng.next() * 97.0) as usize;
            Arc {
                cx,
                cy,
                rx,
                ry,
                rot,
                a0,
                a1,
                k,
            }
        })
        .collect();
    (columns, arcs)
}

fn paint(surface: &mut Surface, params: &SpliceParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (width, height) = (f64::from(w), f64::from(h));
    let (columns, arcs) = cuts(params, tool_seed);
    let inks = palette.len();
    let u = (width * height).sqrt();
    let tones = params.tones;
    let fs = u * (0.09 + params.scale * 0.42);
    let shift = params.shift * 0.55 * u;
    let cell = u * (0.004 + params.screen * 0.022);
    let kw = params.key * 0.085;
    let gl = params.grain * 54.0;
    let seed = |offset: u32| tool_seed.wrapping_add(offset);
    let column_of: Vec<&Column> = (0..w)
        .map(|x| {
            let at = (f64::from(x) + 0.5) / width;
            columns
                .iter()
                .find(|column| at < column.right)
                .expect("the last column ends at 1")
        })
        .collect();
    let octaves = [
        (1.0, 0.45, seed(3)),
        (0.42, 0.26, seed(17)),
        (0.17, 0.18, seed(41)),
        (0.065, 0.11, seed(83)),
    ];
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let v = (f64::from(y) + 0.5) / height;
        let fy = f64::from(y);
        for (x, column) in column_of.iter().enumerate() {
            let fx = x as f64;
            let pc = column
                .pieces
                .iter()
                .find(|piece| v < piece.bottom)
                .expect("the last piece ends at 1");
            let (sx, sy) = match params.cut {
                SpliceCut::Rows => (fx + pc.off * shift, fy),
                _ => (fx, fy + pc.off * shift),
            };
            let field: f64 = octaves
                .iter()
                .map(|&(size, weight, seed)| {
                    weight * chassis::value_noise(sx / (fs * size), sy / (fs * size), seed)
                })
                .sum();
            let g = field.clamp(0.0, 0.999_999) * f64::from(tones);
            let t = (g as u32).min(tones - 1);
            let cov = g - f64::from(t);
            let near = if cov < 0.5 { cov } else { 1.0 - cov };
            let ink = if kw > 0.0005 && near < kw {
                palette.ink(0)
            } else {
                let cs = (cell * pc.sc).max(1.0);
                let (cx, cy) = ((fx / cs).floor(), (fy / cs).floor());
                let on = match pc.screen {
                    Screen::Flat => cov > 0.5,
                    Screen::Dots => {
                        let (dx, dy) = (fx / cs - cx - 0.5, fy / cs - cy - 0.5);
                        let r = cov.sqrt() * 0.72;
                        dx * dx + dy * dy < r * r
                    }
                    Screen::Lines => fy / cs - cy < cov,
                    Screen::Noise => hash(cx as i32, cy as i32, seed(7)) < cov,
                };
                palette.ink((t as usize + usize::from(on) + pc.rot) % inks)
            };
            let q = if gl > 0.002 {
                (hash(x as i32, y as i32, seed(71)) - 0.5) * gl
            } else {
                0.0
            };
            let [r, g, b] = ink.map(|c| store(f64::from(c) + q));
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.edit_rgba(|pixels, _, _| pixels.copy_from_slice(&rgba));

    if arcs.is_empty() || params.weight <= 0.004 {
        return;
    }
    let line = (u * (0.0009 + params.weight * 0.0042)).max(0.9);
    surface.with_alpha(0.85, |surface| {
        for arc in &arcs {
            let mut path = Path2D::default();
            path.ellipse(
                arc.cx * width,
                arc.cy * height,
                arc.rx * u,
                arc.ry * u,
                arc.rot,
                arc.a0,
                arc.a1,
            );
            surface.stroke(
                &path,
                palette.ink(arc.k % inks),
                line,
                Cap::Round,
                Join::Round,
            );
        }
    });
}
