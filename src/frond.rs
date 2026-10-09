use std::f64::consts::{FRAC_PI_2, PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, value_noise,
};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "frond";
pub(crate) const FRAMES: u32 = 36;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;
const BOIL: i32 = 911;

const DEFAULT_PALETTE: [&str; 5] = ["#ede4d3", "#ff8c42", "#0b3d91", "#1a1a1a", "#a89f8c"];

const MASSES: Param = Param::new(SLUG, "masses", 0, 6, 1);
const SIZE: Param = Param {
    taste: (0, 75),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const ROUND: Param = Param {
    taste: (25, 100),
    ..Param::new(SLUG, "round", 0, 100, 100)
};
const GROWTH: Param = Param {
    taste: (25, 75),
    ..Param::new(SLUG, "growth", 0, 100, 100)
};
const DETAIL: Param = Param::new(SLUG, "detail", 0, 100, 100);
const COARSE: Param = Param {
    taste: (25, 75),
    ..Param::new(SLUG, "coarse", 0, 100, 100)
};
const BREAKUP: Param = Param {
    taste: (25, 50),
    ..Param::new(SLUG, "breakup", 0, 100, 100)
};
const NOISE: Param = Param::new(SLUG, "noise", 0, 100, 100);
const PATCH: Param = Param::new(SLUG, "patch", 0, 100, 100);
const CIRCLES: Param = Param::new(SLUG, "circles", 0, 100, 100);
const RULES: Param = Param::new(SLUG, "rules", 0, 100, 100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Plant {
    Potted,
    Bouquet,
    Fronds,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrondParams {
    #[serde(rename = "dirs")]
    plant: Plant,
    masses: u32,
    size: f64,
    round: f64,
    growth: f64,
    detail: f64,
    coarse: f64,
    breakup: f64,
    noise: f64,
    patch: f64,
    circles: f64,
    rules: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for FrondParams {
    fn default() -> Self {
        Self {
            plant: Plant::Potted,
            masses: 3,
            size: 0.62,
            round: 0.55,
            growth: 0.55,
            detail: 0.5,
            coarse: 0.42,
            breakup: 0.35,
            noise: 0.45,
            patch: 0.5,
            circles: 0.5,
            rules: 0.45,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl FrondParams {
    pub fn plant(&self) -> Plant {
        self.plant
    }

    pub fn set_plant(&mut self, plant: Plant) {
        self.plant = plant;
    }

    pub fn masses(&self) -> u32 {
        self.masses
    }

    pub fn set_masses(&mut self, masses: u32) -> Result<(), Error> {
        MASSES.check(f64::from(masses))?;
        self.masses = masses;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn round(&self) -> f64 {
        self.round
    }

    pub fn set_round(&mut self, round: f64) -> Result<(), Error> {
        self.round = ROUND.check(round)?;
        Ok(())
    }

    pub fn growth(&self) -> f64 {
        self.growth
    }

    pub fn set_growth(&mut self, growth: f64) -> Result<(), Error> {
        self.growth = GROWTH.check(growth)?;
        Ok(())
    }

    pub fn detail(&self) -> f64 {
        self.detail
    }

    pub fn set_detail(&mut self, detail: f64) -> Result<(), Error> {
        self.detail = DETAIL.check(detail)?;
        Ok(())
    }

    pub fn coarse(&self) -> f64 {
        self.coarse
    }

    pub fn set_coarse(&mut self, coarse: f64) -> Result<(), Error> {
        self.coarse = COARSE.check(coarse)?;
        Ok(())
    }

    pub fn breakup(&self) -> f64 {
        self.breakup
    }

    pub fn set_breakup(&mut self, breakup: f64) -> Result<(), Error> {
        self.breakup = BREAKUP.check(breakup)?;
        Ok(())
    }

    pub fn noise(&self) -> f64 {
        self.noise
    }

    pub fn set_noise(&mut self, noise: f64) -> Result<(), Error> {
        self.noise = NOISE.check(noise)?;
        Ok(())
    }

    pub fn patch(&self) -> f64 {
        self.patch
    }

    pub fn set_patch(&mut self, patch: f64) -> Result<(), Error> {
        self.patch = PATCH.check(patch)?;
        Ok(())
    }

    pub fn circles(&self) -> f64 {
        self.circles
    }

    pub fn set_circles(&mut self, circles: f64) -> Result<(), Error> {
        self.circles = CIRCLES.check(circles)?;
        Ok(())
    }

    pub fn rules(&self) -> f64 {
        self.rules
    }

    pub fn set_rules(&mut self, rules: f64) -> Result<(), Error> {
        self.rules = RULES.check(rules)?;
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
    dirs: Plant,
    masses: u32,
    size: f64,
    round: f64,
    growth: f64,
    detail: f64,
    coarse: f64,
    breakup: f64,
    noise: f64,
    patch: f64,
    circles: f64,
    rules: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<FrondParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = FrondParams::default();
    params.set_plant(raw.dirs);
    params.set_masses(raw.masses)?;
    params.set_size(raw.size)?;
    params.set_round(raw.round)?;
    params.set_growth(raw.growth)?;
    params.set_detail(raw.detail)?;
    params.set_coarse(raw.coarse)?;
    params.set_breakup(raw.breakup)?;
    params.set_noise(raw.noise)?;
    params.set_patch(raw.patch)?;
    params.set_circles(raw.circles)?;
    params.set_rules(raw.rules)?;
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
    MASSES, SIZE, ROUND, GROWTH, DETAIL, COARSE, BREAKUP, NOISE, PATCH, CIRCLES, RULES,
];

pub(crate) fn parameters() -> Vec<Parameter> {
    std::iter::once(Parameter::choice("dirs", &["Potted", "Bouquet", "Fronds"]))
        .chain(PARAMS.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> FrondParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    const PLANTS: [Plant; 3] = [Plant::Potted, Plant::Bouquet, Plant::Fronds];
    FrondParams {
        plant: PLANTS[(draw("dirs") % 3) as usize],
        masses: pick(&MASSES) as u32,
        size: pick(&SIZE),
        round: pick(&ROUND),
        growth: pick(&GROWTH),
        detail: pick(&DETAIL),
        coarse: pick(&COARSE),
        breakup: pick(&BREAKUP),
        noise: pick(&NOISE),
        patch: pick(&PATCH),
        circles: pick(&CIRCLES),
        rules: pick(&RULES),
        ..FrondParams::default()
    }
}

type Points = Vec<(f64, f64)>;

const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

struct Pen {
    ink: [u8; 3],
    mark: f64,
    gap: f64,
    jitter: f64,
    skip: f64,
    boil: i32,
}

impl Pen {
    fn trace(&self, surface: &mut Surface, points: &[(f64, f64)], closed: bool, seed: i32) {
        let n = points.len();
        if n < 2 {
            return;
        }
        let seed = seed + self.boil;
        let roll = |k: i32, key: u32| hash(k, seed, key);
        let (mut need, mut k) = (0.0, seed);
        for i in 0..if closed { n } else { n - 1 } {
            let ((ax, ay), (bx, by)) = (points[i], points[(i + 1) % n]);
            let (dx, dy) = (bx - ax, by - ay);
            let length = dx.hypot(dy);
            if length < 1e-6 {
                continue;
            }
            let (dx, dy) = (dx / length, dy / length);
            let mut pos = 0.0;
            while need <= length - pos {
                pos += need;
                k += 1;
                if hash(k * 17, seed, seed as u32 + 7) >= self.skip {
                    let jx = (roll(k, 3) - 0.5) * self.jitter;
                    let jy = (roll(k, 9) - 0.5) * self.jitter;
                    let size = self.mark * (0.55 + roll(k, 13));
                    mark(
                        surface,
                        ax + dx * pos + jx,
                        ay + dy * pos + jy,
                        size,
                        (k ^ seed) & 1 == 1,
                        self.ink,
                    );
                }
                need = self.gap * (0.7 + roll(k, 21) * 0.7);
            }
            need -= length - pos;
        }
    }
}

fn mark(surface: &mut Surface, x: f64, y: f64, size: f64, diamond: bool, ink: [u8; 3]) {
    if !diamond {
        let half = size * 0.78;
        surface.fill_box(x - half, y - half, size * 1.56, size * 1.56, ink);
        return;
    }
    let mut path = Path2D::default();
    path.move_to(x, y - size);
    path.line_to(x + size, y);
    path.line_to(x, y + size);
    path.line_to(x - size, y);
    path.close();
    surface.fill_path(&path.into_path(), ink);
}

fn walk(
    (x, y): (f64, f64),
    angle: f64,
    length: f64,
    steps: usize,
    turn: impl Fn(usize) -> f64,
) -> (Points, Points, (f64, f64)) {
    let (mut a, mut px, mut py) = (angle, x, y);
    let (mut spine, mut normals) = (Vec::new(), Vec::new());
    let n = steps as f64;
    for i in 0..=steps {
        spine.push((px, py));
        normals.push((-a.sin(), a.cos()));
        a += turn(i);
        px += a.cos() * length / n;
        py += a.sin() * length / n;
    }
    (spine, normals, (px, py))
}

fn offset(
    spine: &[(f64, f64)],
    normals: &[(f64, f64)],
    widths: impl Fn(usize) -> f64,
    sign: f64,
) -> Points {
    spine
        .iter()
        .zip(normals)
        .enumerate()
        .map(|(i, (&(x, y), &(nx, ny)))| {
            let w = widths(i);
            (x + sign * nx * w, y + sign * ny * w)
        })
        .collect()
}

struct Leaf {
    outline: Points,
    spine: Points,
}

#[allow(clippy::too_many_arguments)]
fn leaf(
    base: (f64, f64),
    angle: f64,
    length: f64,
    width: f64,
    curl: f64,
    wobble: f64,
    seed: i32,
) -> Leaf {
    const N: usize = 26;
    let n = N as f64;
    let s = f64::from(seed);
    let at = |i: usize| i as f64 / n;
    let (spine, normals, _) = walk(base, angle, length, N, |i| {
        curl / n + (value_noise(at(i) * 3.3 + s * 0.11, s * 0.07, seed as u32) - 0.5) * 0.5 * wobble
    });
    let widths = |i: usize| {
        let t = at(i);
        width
            * (PI * t).sin().powf(0.62)
            * (1.0 + (value_noise(t * 4.1 + s * 0.2, 3.7, seed as u32 + 5) - 0.5) * 0.5 * wobble)
    };
    let mut outline = offset(&spine, &normals, widths, 1.0);
    outline.extend(offset(&spine, &normals, widths, -1.0).into_iter().rev());
    Leaf { outline, spine }
}

struct Stem {
    left: Points,
    right: Points,
    tip: (f64, f64),
}

fn stem(base: (f64, f64), angle: f64, length: f64, width: f64, wobble: f64, seed: i32) -> Stem {
    let s = f64::from(seed);
    let (spine, normals, tip) = walk(base, angle, length, 22, |i| {
        (value_noise(i as f64 * 0.29 + s * 0.13, 2.9, seed as u32) - 0.5) * 0.30 * wobble
    });
    Stem {
        left: offset(&spine, &normals, |_| width, 1.0),
        right: offset(&spine, &normals, |_| width, -1.0),
        tip,
    }
}

fn ring((cx, cy): (f64, f64), rx: f64, ry: f64, rotation: f64, wobble: f64, seed: i32) -> Points {
    let (cos_r, sin_r) = (rotation.cos(), rotation.sin());
    (0..14)
        .map(|i| {
            let th = f64::from(i) / 14.0 * TAU;
            let r = 1.0
                + (value_noise(th.cos() * 2.1 + 3.1, th.sin() * 2.1 + 7.7, seed as u32) - 0.5)
                    * 0.9
                    * wobble;
            let (x, y) = (th.cos() * rx * r, th.sin() * ry * r);
            (cx + x * cos_r - y * sin_r, cy + x * sin_r + y * cos_r)
        })
        .collect()
}

fn rounded_rect(x: f64, y: f64, w: f64, h: f64, radius: f64) -> Path2D {
    let r = radius.min(w.abs() * 0.5).min(h.abs() * 0.5);
    let mut path = Path2D::default();
    path.move_to(x + r, y);
    path.line_to(x + w - r, y);
    path.arc_to(x + w, y, x + w, y + r, r);
    path.line_to(x + w, y + h - r);
    path.arc_to(x + w, y + h, x + w - r, y + h, r);
    path.line_to(x + r, y + h);
    path.arc_to(x, y + h, x, y + h - r, r);
    path.line_to(x, y + r);
    path.arc_to(x, y, x + r, y, r);
    path.close();
    path
}

fn pot(cx: f64, foot: f64, w: f64, h: f64, r: f64) -> Path2D {
    let (top, bottom) = (w * 0.5, w * 0.5 * 0.72);
    let mut path = Path2D::default();
    path.move_to(cx - top, foot - h);
    path.line_to(cx + top, foot - h);
    path.line_to(cx + bottom, foot - r);
    path.quad_to(cx + bottom, foot, cx + bottom - r, foot);
    path.line_to(cx - bottom + r, foot);
    path.quad_to(cx - bottom, foot, cx - bottom, foot - r);
    path.close();
    path
}

#[allow(clippy::too_many_arguments)]
fn noise_block(
    surface: &mut Surface,
    (x, y): (f64, f64),
    (w, h): (f64, f64),
    scale: f64,
    seed: i32,
    grit: [u8; 3],
    paper: [u8; 3],
) {
    let (x, y) = (
        round_half_up(x).max(0.0) as u32,
        round_half_up(y).max(0.0) as u32,
    );
    let (w, h) = (round_half_up(w), round_half_up(h));
    if w < 2.0 || h < 2.0 {
        return;
    }
    let (w, h) = (w as u32, h as u32);
    let fine = scale * 0.34;
    for j in 0..h.min(surface.height().saturating_sub(y)) {
        for i in 0..w.min(surface.width().saturating_sub(x)) {
            let (px, py) = (f64::from(x + i), f64::from(y + j));
            let f = value_noise(px / scale, py / scale, seed as u32) * 0.62
                + value_noise(px / fine, py / fine, seed as u32 + 91) * 0.38;
            let v = (f - 0.5) * 2.3 + 0.42 - f64::from(j) / f64::from(h) * 0.34;
            let threshold = (f64::from(BAYER[((j & 3) * 4 + (i & 3)) as usize]) + 0.5) / 16.0;
            surface.set_pixel(x + i, y + j, if v < threshold { grit } else { paper });
        }
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &FrondParams,
    palette: &Palette,
    tool_seed: u32,
    t: u32,
) {
    paint(surface, params, palette, tool_seed, t);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &FrondParams, palette: &Palette, tool_seed: u32, t: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let unit = (width * height).sqrt();
    let boil = t as i32 * BOIL;
    let role = |slot: usize, missing: [u8; 3]| {
        if slot < palette.len() {
            palette.ink(slot)
        } else {
            missing
        }
    };
    let (paper, mass) = (palette.ink(0), palette.ink(1));
    let ink = role(2, mass);
    let (grit, shade) = (role(3, ink), role(4, mass));

    let base = tool_seed.wrapping_add(1).wrapping_mul(2_654_435_761);
    let mut masses = XorShift::new(base ^ 0x9e37_79b9);
    let mut plant = XorShift::new(base ^ 0x85eb_ca6b);
    let mut noise = XorShift::new(base ^ 0xc2b2_ae35);
    let mut geometry = XorShift::new(base ^ 0x27d4_eb2f);

    surface.fill(paper);
    for _ in 0..3 {
        let cx = width * (0.2 + masses.next() * 0.6);
        let cy = height * (0.2 + masses.next() * 0.6);
        let r = unit * (0.18 + masses.next() * 0.26);
        surface.fill_radial_fade(cx, cy, r, shade, 0.22);
    }

    let big = unit * (0.22 + params.size * 0.42);
    for _ in 0..params.masses {
        let w = big * (0.7 + masses.next() * 0.9);
        let h = big * (0.45 + masses.next() * 0.85);
        let x = width * (0.5 + (masses.next() - 0.5) * 0.95) - w * 0.5;
        let y = height * (0.10 + masses.next() * 0.50) - h * 0.5;
        let r = w.min(h) * 0.5 * params.round;
        let lobes = 1 + (masses.next() * 3.0) as u32;
        for _ in 0..lobes {
            let ox = (masses.next() - 0.5) * w * 0.5;
            let oy = (masses.next() - 0.5) * h * 0.5;
            let lw = w * (0.55 + masses.next() * 0.6);
            let lh = h * (0.55 + masses.next() * 0.6);
            surface.fill_path(&rounded_rect(x + ox, y + oy, lw, lh, r).into_path(), mass);
        }
    }
    let pot_w = unit * (0.34 + params.size * 0.30);
    let pot_h = unit * (0.20 + params.size * 0.20);
    let (pot_x, pot_y) = (width * 0.5, height * 0.94);
    if params.plant == Plant::Potted {
        let r = pot_w * 0.16 * (0.3 + params.round);
        surface.fill_path(&pot(pot_x, pot_y, pot_w, pot_h, r).into_path(), mass);
    }

    for _ in 0..round_half_up(params.noise * 10.0) as u32 {
        let pw = unit * (0.06 + params.patch * 0.20) * (0.5 + noise.next() * 1.1);
        let ph = unit * (0.05 + params.patch * 0.17) * (0.5 + noise.next() * 1.1);
        let (px, py) = if noise.next() < 0.5 {
            let px = if noise.next() < 0.5 {
                -pw * 0.35
            } else {
                width - pw * 0.65
            };
            (px, noise.next() * (height - ph))
        } else {
            let px = noise.next() * (width - pw);
            (
                px,
                if noise.next() < 0.5 {
                    -ph * 0.3
                } else {
                    height - ph * 0.7
                },
            )
        };
        let size = (pw.min(width - px.max(0.0)), ph.min(height - py.max(0.0)));
        let scale = unit * (0.010 + params.patch * 0.030);
        let seed = (noise.next() * 9973.0) as i32 + boil;
        noise_block(surface, (px, py), size, scale, seed, grit, paper);
    }

    let mark_size = unit * (0.0013 + params.coarse * 0.0060);
    let pen = Pen {
        ink,
        mark: mark_size,
        gap: mark_size * (1.35 + params.breakup * 1.5),
        jitter: mark_size * (0.5 + params.breakup * 3.2),
        skip: params.breakup * 0.42,
        boil,
    };
    let wobble = 0.5 + params.breakup * 0.7;
    let leaves = round_half_up(2.0 + params.growth * 9.0).max(1.0);
    let head = round_half_up(14.0 + params.detail * 92.0);
    let seed = |stream: &mut XorShift| (stream.next() * 9973.0) as i32;

    if params.plant == Plant::Fronds {
        for _ in 0..leaves as u32 {
            let bx = width * (0.12 + plant.next() * 0.76);
            let by = height * (0.86 + plant.next() * 0.16);
            let angle = -FRAC_PI_2 + (plant.next() - 0.5) * 1.5;
            let length = unit * (0.34 + params.growth * 0.50) * (0.6 + plant.next() * 0.7);
            let blade = unit * (0.055 + params.size * 0.115);
            let curl = (plant.next() - 0.5) * 2.4;
            let leaf = leaf(
                (bx, by),
                angle,
                length,
                blade,
                curl,
                wobble,
                seed(&mut plant),
            );
            pen.trace(surface, &leaf.outline, true, seed(&mut plant));
            if plant.next() < params.detail {
                pen.trace(surface, &leaf.spine, false, seed(&mut plant));
            }
        }
    } else {
        let bouquet = params.plant == Plant::Bouquet;
        let stems = if bouquet {
            2 + round_half_up(params.growth * 5.0) as u32
        } else {
            1
        };
        for s in 0..stems {
            let bx = pot_x
                + if bouquet {
                    (plant.next() - 0.5) * unit * 0.10
                } else {
                    0.0
                };
            let by = if bouquet {
                height * 0.86
            } else {
                pot_y - pot_h * 0.92
            };
            let spread = if bouquet {
                (f64::from(s) / f64::from(stems - 1) - 0.5) * 1.5 * (0.4 + params.growth)
            } else {
                0.0
            };
            let length = unit * (0.24 + params.growth * 0.28);
            let half = unit * 0.006 * (0.6 + params.size);
            let stem = stem(
                (bx, by),
                -FRAC_PI_2 + spread,
                length,
                half,
                wobble,
                seed(&mut plant),
            );
            pen.trace(surface, &stem.left, false, seed(&mut plant));
            pen.trace(surface, &stem.right, false, seed(&mut plant));

            let per = round_half_up(leaves / f64::from(stems)).max(1.0);
            for i in 0..per as u32 {
                let t = 0.12 + (f64::from(i) + 0.5) / per * 0.7;
                let base = stem.left[round_half_up(t * 22.0) as usize];
                let side = if i & 1 == 1 { 1.0 } else { -1.0 };
                let drop = 1.75 - t;
                let angle = -FRAC_PI_2 + side * (0.7 + plant.next() * 0.7) + spread;
                let length =
                    unit * (0.19 + params.growth * 0.32) * (0.55 + plant.next() * 0.7) * drop;
                let blade = unit * (0.032 + params.size * 0.080) * drop;
                let curl = side * (0.6 + plant.next() * 1.5);
                let leaf = leaf(base, angle, length, blade, curl, wobble, seed(&mut plant));
                pen.trace(surface, &leaf.outline, true, seed(&mut plant));
                if plant.next() < params.detail * 0.7 {
                    pen.trace(surface, &leaf.spine, false, seed(&mut plant));
                }
            }

            let (hx, hy) = stem.tip;
            let hw = unit * (0.10 + params.detail * 0.16);
            let hh = hw * 1.05;
            let rings = round_half_up(head / f64::from(stems).sqrt()).max(6.0);
            for _ in 0..rings as u32 {
                let a = plant.next() * TAU;
                let reach = plant.next().sqrt();
                let centre = (
                    hx + a.cos() * reach * hw,
                    hy + a.sin() * reach * hh - hh * 0.45,
                );
                let rx = unit * (0.008 + plant.next() * 0.020) * (0.5 + params.detail);
                let ry = rx * (0.5 + plant.next() * 0.8);
                let rotation = plant.next() * TAU;
                let ring = ring(centre, rx, ry, rotation, wobble, seed(&mut plant));
                pen.trace(surface, &ring, true, seed(&mut plant));
            }
        }
    }
    for i in 0..round_half_up(params.detail * 90.0) as i32 {
        let x = plant.next() * width;
        let y = plant.next() * height * 0.85;
        let size = mark_size * (0.6 + plant.next() * 1.2);
        mark(surface, x, y, size, (i + boil) & 1 == 1, ink);
    }

    let line = (unit * 0.0013).max(0.6);
    for _ in 0..round_half_up(params.circles * 4.0) as u32 {
        let r = unit * (0.24 + geometry.next() * 0.30);
        let cx = width * (0.2 + geometry.next() * 0.6);
        let cy = height * (0.2 + geometry.next() * 0.6);
        let mut path = Path2D::default();
        path.push_circle(cx, cy, r);
        surface.stroke(&path, ink, line);
    }
    for _ in 0..round_half_up(params.rules * 9.0) as u32 {
        let y = height * (0.05 + geometry.next() * 0.9);
        let x0 = width * geometry.next() * 0.35;
        let x1 = width * (0.65 + geometry.next() * 0.4);
        let mut path = Path2D::default();
        path.move_to(x0, y);
        path.line_to(x1, y);
        surface.stroke(&path, ink, line);
        if geometry.next() < 0.35 {
            let side = unit * (0.006 + geometry.next() * 0.008);
            let x = x0 + (x1 - x0) * (0.15 + geometry.next() * 0.7) - side * 0.5;
            surface.fill_box(x, y - side * 0.5, side, side, ink);
        }
    }
}
