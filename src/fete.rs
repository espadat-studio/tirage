use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "fete";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 20;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 4] = ["#7b2cbf", "#ff6d00", "#00bbf9", "#ffd60a"];
const LINE: [u8; 3] = [0xff, 0xff, 0xff];
const DOT: [u8; 3] = [0x0a, 0x0a, 0x0a];

const SCALE: Param = Param::new(SLUG, "scale", 12, 60, 10);
const RES: Param = Param {
    step: 4,
    ..Param::new(SLUG, "res", 36, 120, 1)
};
const LW: Param = Param {
    step: 5,
    ..Param::new(SLUG, "lw", 40, 220, 100)
};
const DOTS: Param = Param {
    step: 2,
    ..Param::new(SLUG, "dots", 0, 80, 1)
};

pub(crate) const PARAMS: &[Param] = &[SCALE, RES, LW, DOTS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeteMotif {
    Auto,
    Rings,
    Spiral,
    Burst,
    Atom,
    Wave,
    Scribble,
}

const MOTIFS: [FeteMotif; 7] = [
    FeteMotif::Auto,
    FeteMotif::Rings,
    FeteMotif::Spiral,
    FeteMotif::Burst,
    FeteMotif::Atom,
    FeteMotif::Wave,
    FeteMotif::Scribble,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FeteParams {
    #[serde(rename = "motifs")]
    motif: FeteMotif,
    scale: f64,
    res: u32,
    lw: f64,
    dots: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for FeteParams {
    fn default() -> Self {
        Self {
            motif: FeteMotif::Auto,
            scale: 2.6,
            res: 64,
            lw: 1.0,
            dots: 32,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl FeteParams {
    pub fn motif(&self) -> FeteMotif {
        self.motif
    }

    pub fn set_motif(&mut self, motif: FeteMotif) {
        self.motif = motif;
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn res(&self) -> u32 {
        self.res
    }

    pub fn set_res(&mut self, res: u32) -> Result<(), Error> {
        RES.check(f64::from(res))?;
        self.res = res;
        Ok(())
    }

    pub fn lw(&self) -> f64 {
        self.lw
    }

    pub fn set_lw(&mut self, lw: f64) -> Result<(), Error> {
        self.lw = LW.check(lw)?;
        Ok(())
    }

    pub fn dots(&self) -> u32 {
        self.dots
    }

    pub fn set_dots(&mut self, dots: u32) -> Result<(), Error> {
        DOTS.check(f64::from(dots))?;
        self.dots = dots;
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
    #[serde(rename = "motifs")]
    motif: FeteMotif,
    scale: f64,
    res: u32,
    lw: f64,
    dots: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<FeteParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = FeteParams::default();
    params.set_motif(raw.motif);
    params.set_scale(raw.scale)?;
    params.set_res(raw.res)?;
    params.set_lw(raw.lw)?;
    params.set_dots(raw.dots)?;
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
        "motifs",
        &[
            "Auto", "Rings", "Spiral", "Burst", "Atom", "Wave", "Scribble",
        ],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> FeteParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    FeteParams {
        motif: MOTIFS[(draw("motifs") % MOTIFS.len() as u64) as usize],
        scale: pick(&SCALE),
        res: pick(&RES) as u32,
        lw: pick(&LW),
        dots: pick(&DOTS) as u32,
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &FeteParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

const VIEW: f64 = 1000.0;

type Point = (f64, f64);

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn value_noise(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let fade = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash(xi + dx, yi + dy, 0, seed);
    lerp(
        lerp(corner(0, 0), corner(1, 0), u),
        lerp(corner(0, 1), corner(1, 1), u),
        v,
    )
}

fn fbm(x: f64, y: f64, seed: u32) -> f64 {
    let (mut weight, mut frequency, mut sum, mut total) = (0.5, 1.0, 0.0, 0.0);
    for octave in 0..3 {
        sum += weight
            * value_noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 1319),
            );
        total += weight;
        weight *= 0.5;
        frequency *= 2.0;
    }
    sum / total
}

fn patches(params: &FeteParams, palette: &Palette, columns: u32, rows: u32, seed: u32) -> Vec<u8> {
    let inks = palette.len();
    let mut rgba = Vec::with_capacity((columns * rows * 4) as usize);
    for v in 0..rows {
        for u in 0..columns {
            let x = f64::from(u) / f64::from(columns) * params.scale;
            let y = f64::from(v) / f64::from(columns) * params.scale;
            let n = fbm(x, y, seed);
            let slot = (((n - 0.5) * 1.9 + 0.5) * inks as f64).floor();
            let [r, g, b] = palette.ink(slot.clamp(0.0, (inks - 1) as f64) as usize);
            rgba.extend([r, g, b, 255]);
        }
    }
    rgba
}

fn curve(steps: u32, closed: bool, at: impl Fn(f64) -> Point) -> Vec<Point> {
    let mut points: Vec<Point> = (0..=steps)
        .map(|i| at(f64::from(i) / f64::from(steps)))
        .collect();
    if closed {
        points.push(at(0.0));
    }
    points
}

fn doodle(motif: FeteMotif, view_height: f64, tool_seed: u32) -> Vec<Vec<Point>> {
    let h = view_height;
    let mut rng = Xorshift::from_state(tool_seed ^ 0x51ed_270b);
    let motif = match motif {
        FeteMotif::Auto => MOTIFS[1 + (rng.next() * 6.0) as usize],
        motif => motif,
    };
    let cx = VIEW * (0.32 + rng.next() * 0.36);
    let cy = h * (0.3 + rng.next() * 0.4);
    let wobble = VIEW * 0.004;
    let phase = rng.next() * 7.0;
    let jitter = |points: Vec<Point>| -> Vec<Point> {
        points
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let i = i as f64;
                (
                    x + (i * 1.7 + phase).sin() * wobble,
                    y + (i * 1.3 + phase).cos() * wobble,
                )
            })
            .collect()
    };
    let mut lines = Vec::new();
    match motif {
        FeteMotif::Rings => {
            let count = 2 + (rng.next() * 2.0) as u32;
            for _ in 0..count {
                let radius = VIEW * (0.16 + rng.next() * 0.18);
                let ox = cx + (rng.next() - 0.5) * VIEW * 0.3;
                let oy = cy + (rng.next() - 0.5) * h * 0.2;
                lines.push(jitter(curve(80, true, |t| {
                    (ox + (t * TAU).cos() * radius, oy + (t * TAU).sin() * radius)
                })));
            }
        }
        FeteMotif::Spiral => {
            let arms = 4 + (rng.next() * 3.0) as u32;
            let sign = if rng.next() < 0.5 { -1.0 } else { 1.0 };
            let curl = sign * (1.6 + rng.next() * 1.6);
            for i in 0..arms {
                let start = f64::from(i) / f64::from(arms) * TAU;
                lines.push(jitter(curve(40, false, |t| {
                    let radius = VIEW * (0.05 + t * 0.33);
                    let angle = start + t * curl;
                    (cx + angle.cos() * radius, cy + angle.sin() * radius)
                })));
            }
        }
        FeteMotif::Burst => {
            let rays = 9 + (rng.next() * 8.0) as u32;
            for i in 0..rays {
                let angle = f64::from(i) / f64::from(rays) * TAU + (rng.next() - 0.5) * 0.2;
                let radius = VIEW * (0.18 + rng.next() * 0.34);
                lines.push(vec![
                    (cx, cy),
                    (cx + angle.cos() * radius, cy + angle.sin() * radius),
                ]);
            }
        }
        FeteMotif::Atom => {
            for i in 0..3 {
                let rot = f64::from(i) / 3.0 * PI + (rng.next() - 0.5) * 0.25;
                let rx = VIEW * (0.3 + rng.next() * 0.12);
                let ry = rx * (0.3 + rng.next() * 0.15);
                lines.push(jitter(curve(80, true, |t| {
                    let angle = t * TAU;
                    let (x0, y0) = (angle.cos() * rx, angle.sin() * ry);
                    (
                        cx + x0 * rot.cos() - y0 * rot.sin(),
                        cy + x0 * rot.sin() + y0 * rot.cos(),
                    )
                })));
            }
        }
        FeteMotif::Wave => {
            let n = 7 + (rng.next() * 7.0) as u32;
            let line = (0..=n)
                .map(|i| {
                    let sign = if i % 2 == 1 { -1.0 } else { 1.0 };
                    let ends = if i == 0 || i == n { 0.2 } else { 1.0 };
                    (
                        VIEW * 0.06 + f64::from(i) / f64::from(n) * VIEW * 0.88,
                        cy + sign * h * (0.05 + rng.next() * 0.22) * ends,
                    )
                })
                .collect();
            lines.push(line);
        }
        FeteMotif::Scribble | FeteMotif::Auto => {
            let loops = 4 + (rng.next() * 3.0) as u32;
            let (mut x, mut y) = (VIEW * 0.15, cy);
            let mut line = vec![(x, y)];
            for _ in 0..loops {
                let nx = (x + VIEW * (0.1 + rng.next() * 0.22)).clamp(0.0, VIEW * 0.92);
                let ny = (cy + (rng.next() - 0.5) * h * 0.5).clamp(h * 0.08, h * 0.92);
                let mx = (x + nx) / 2.0 + (rng.next() - 0.5) * VIEW * 0.3;
                let my = (y + ny) / 2.0 + (rng.next() - 0.5) * h * 0.4;
                for k in 1..=18 {
                    let t = f64::from(k) / 18.0;
                    line.push((
                        lerp(lerp(x, mx, t), lerp(mx, nx, t), t),
                        lerp(lerp(y, my, t), lerp(my, ny, t), t),
                    ));
                }
                (x, y) = (nx, ny);
            }
            lines.push(line);
        }
    }
    lines
}

fn dots(count: u32, lines: &[Vec<Point>], view_height: f64, tool_seed: u32) -> Vec<Point> {
    let mut rng = Xorshift::from_state(tool_seed ^ 0x2545_f491);
    let vertices: Vec<Point> = lines
        .iter()
        .flat_map(|line| line.iter().skip(1).copied())
        .collect();
    (0..count)
        .map(|_| {
            if rng.next() < 0.4 {
                vertices[(rng.next() * vertices.len() as f64) as usize]
            } else {
                let x = VIEW * (0.04 + rng.next() * 0.92);
                (x, view_height * (0.04 + rng.next() * 0.92))
            }
        })
        .collect()
}

fn paint(surface: &mut Surface, params: &FeteParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let columns = params.res;
    let rows = (round_half_up(f64::from(columns) * height / width) as u32).max(8);
    let rgba = patches(params, palette, columns, rows, tool_seed);
    surface.draw_smooth(&rgba, columns, rows, Smoothing::Nearest);

    let view_height = VIEW * height / width;
    let k = width / VIEW;
    let lines = doodle(params.motif, view_height, tool_seed);
    let line_width = VIEW * 0.009 * params.lw * k;
    for line in &lines {
        let mut path = Path2D::default();
        path.move_to(line[0].0 * k, line[0].1 * k);
        for &(x, y) in &line[1..] {
            path.line_to(x * k, y * k);
        }
        surface.stroke(&path, LINE, line_width, Cap::Round, Join::Round);
    }
    let radius = VIEW * 0.0085 * k;
    for (x, y) in dots(params.dots, &lines, view_height, tool_seed) {
        surface.fill_circle(x * k, y * k, radius, DOT);
    }
}
