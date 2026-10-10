use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, hash, store, value_noise};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "hiss";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 5] = ["#ff1e8c", "#ff7a1a", "#2b4bff", "#ffd400", "#e8262b"];
const BLACK: [f64; 3] = [8.0, 8.0, 10.0];
const MARK: [u8; 3] = [0xf4, 0xf4, 0xf2];

const LEAVES: Param = Param::new(SLUG, "leaves", 1, 5, 1);
const SIZE: Param = Param::new(SLUG, "size", 40, 140, 100);
const OVERLAP: Param = Param::new(SLUG, "overlap", 0, 50, 100);
const TILT: Param = Param::new(SLUG, "tilt", -100, 100, 100);
const SHIFT: Param = Param::new(SLUG, "shift", 0, 100, 100);
const SWIRL: Param = Param::new(SLUG, "swirl", 0, 100, 100);
const BLACKS: Param = Param::new(SLUG, "black", 0, 100, 100);
const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const CHECKERS: Param = Param::new(SLUG, "checkers", 0, 4, 1);
const STEPS: Param = Param::new(SLUG, "steps", 0, 100, 100);
const COMB: Param = Param::new(SLUG, "comb", 0, 100, 100);
const MARKS: Param = Param::new(SLUG, "marks", 0, 24, 1);
const FLECKS: Param = Param::new(SLUG, "flecks", 0, 100, 100);
const COARSE: Param = Param::new(SLUG, "coarse", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    LEAVES, SIZE, OVERLAP, TILT, SHIFT, SWIRL, BLACKS, SCALE, CHECKERS, STEPS, COMB, MARKS, FLECKS,
    COARSE,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HissParams {
    leaves: u32,
    size: f64,
    overlap: f64,
    tilt: f64,
    shift: f64,
    swirl: f64,
    black: f64,
    scale: f64,
    checkers: u32,
    steps: f64,
    comb: f64,
    marks: u32,
    flecks: f64,
    coarse: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for HissParams {
    fn default() -> Self {
        Self {
            leaves: 3,
            size: 0.9,
            overlap: 0.15,
            tilt: 0.2,
            shift: 0.5,
            swirl: 0.6,
            black: 0.45,
            scale: 0.5,
            checkers: 2,
            steps: 0.35,
            comb: 0.4,
            marks: 8,
            flecks: 0.4,
            coarse: 0.15,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl HissParams {
    pub fn leaves(&self) -> u32 {
        self.leaves
    }

    pub fn set_leaves(&mut self, leaves: u32) -> Result<(), Error> {
        LEAVES.check(f64::from(leaves))?;
        self.leaves = leaves;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn overlap(&self) -> f64 {
        self.overlap
    }

    pub fn set_overlap(&mut self, overlap: f64) -> Result<(), Error> {
        self.overlap = OVERLAP.check(overlap)?;
        Ok(())
    }

    pub fn tilt(&self) -> f64 {
        self.tilt
    }

    pub fn set_tilt(&mut self, tilt: f64) -> Result<(), Error> {
        self.tilt = TILT.check(tilt)?;
        Ok(())
    }

    pub fn shift(&self) -> f64 {
        self.shift
    }

    pub fn set_shift(&mut self, shift: f64) -> Result<(), Error> {
        self.shift = SHIFT.check(shift)?;
        Ok(())
    }

    pub fn swirl(&self) -> f64 {
        self.swirl
    }

    pub fn set_swirl(&mut self, swirl: f64) -> Result<(), Error> {
        self.swirl = SWIRL.check(swirl)?;
        Ok(())
    }

    pub fn black(&self) -> f64 {
        self.black
    }

    pub fn set_black(&mut self, black: f64) -> Result<(), Error> {
        self.black = BLACKS.check(black)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn checkers(&self) -> u32 {
        self.checkers
    }

    pub fn set_checkers(&mut self, checkers: u32) -> Result<(), Error> {
        CHECKERS.check(f64::from(checkers))?;
        self.checkers = checkers;
        Ok(())
    }

    pub fn steps(&self) -> f64 {
        self.steps
    }

    pub fn set_steps(&mut self, steps: f64) -> Result<(), Error> {
        self.steps = STEPS.check(steps)?;
        Ok(())
    }

    pub fn comb(&self) -> f64 {
        self.comb
    }

    pub fn set_comb(&mut self, comb: f64) -> Result<(), Error> {
        self.comb = COMB.check(comb)?;
        Ok(())
    }

    pub fn marks(&self) -> u32 {
        self.marks
    }

    pub fn set_marks(&mut self, marks: u32) -> Result<(), Error> {
        MARKS.check(f64::from(marks))?;
        self.marks = marks;
        Ok(())
    }

    pub fn flecks(&self) -> f64 {
        self.flecks
    }

    pub fn set_flecks(&mut self, flecks: f64) -> Result<(), Error> {
        self.flecks = FLECKS.check(flecks)?;
        Ok(())
    }

    pub fn coarse(&self) -> f64 {
        self.coarse
    }

    pub fn set_coarse(&mut self, coarse: f64) -> Result<(), Error> {
        self.coarse = COARSE.check(coarse)?;
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
    leaves: u32,
    size: f64,
    overlap: f64,
    tilt: f64,
    shift: f64,
    swirl: f64,
    black: f64,
    scale: f64,
    checkers: u32,
    steps: f64,
    comb: f64,
    marks: u32,
    flecks: f64,
    coarse: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<HissParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = HissParams::default();
    params.set_leaves(raw.leaves)?;
    params.set_size(raw.size)?;
    params.set_overlap(raw.overlap)?;
    params.set_tilt(raw.tilt)?;
    params.set_shift(raw.shift)?;
    params.set_swirl(raw.swirl)?;
    params.set_black(raw.black)?;
    params.set_scale(raw.scale)?;
    params.set_checkers(raw.checkers)?;
    params.set_steps(raw.steps)?;
    params.set_comb(raw.comb)?;
    params.set_marks(raw.marks)?;
    params.set_flecks(raw.flecks)?;
    params.set_coarse(raw.coarse)?;
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
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> HissParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    HissParams {
        leaves: pick(&LEAVES) as u32,
        size: pick(&SIZE),
        overlap: pick(&OVERLAP),
        tilt: pick(&TILT),
        shift: pick(&SHIFT),
        swirl: pick(&SWIRL),
        black: pick(&BLACKS),
        scale: pick(&SCALE),
        checkers: pick(&CHECKERS) as u32,
        steps: pick(&STEPS),
        comb: pick(&COMB),
        marks: pick(&MARKS) as u32,
        flecks: pick(&FLECKS),
        coarse: pick(&COARSE),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &HissParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

struct Leaf {
    cx: f64,
    cy: f64,
    half_length: f64,
    half_width: f64,
    cos: f64,
    sin: f64,
}

struct Lens {
    cx: f64,
    cy: f64,
    half_height: f64,
    half_width: f64,
}

fn taper(along: f64, half_length: f64) -> f64 {
    let t = along.abs() / half_length;
    (1.0 - t * t.sqrt()).powf(0.6667)
}

fn ink(palette: &Palette, slot: usize) -> [f64; 3] {
    palette.ink(slot).map(f64::from)
}

fn marble(palette: &Palette, m: f64) -> [f64; 3] {
    let inks = palette.len();
    let p = (m * inks as f64 * 1.25 * 6.0).floor() / 6.0;
    let k = p.floor();
    let f = p - k;
    let first = (k as i64).rem_euclid(inks as i64) as usize;
    let (a, b) = (ink(palette, first), ink(palette, (first + 1) % inks));
    let w = f * f * (3.0 - 2.0 * f);
    std::array::from_fn(|c| a[c] + (b[c] - a[c]) * w)
}

fn leaves(params: &HissParams, w: f64, h: f64, rng: &mut XorShift) -> (Vec<Leaf>, f64) {
    let n = params.leaves as usize;
    let o = params.overlap;
    let leaf = h * 1.32 / (2.0 * (n as f64 - o * (n as f64 - 1.0)));
    let leaves = (0..n)
        .map(|i| {
            let sign = if i % 2 == 1 { -1.0 } else { 1.0 };
            let cy = -h * 0.16 + leaf + i as f64 * 2.0 * leaf * (1.0 - o);
            let cx = w * 0.5 + (params.shift - 0.5) * w * 0.4 * sign;
            let size = params.size * (0.9 + rng.next() * 0.25);
            let angle = params.tilt * 0.25 * sign * (0.6 + rng.next() * 0.8);
            Leaf {
                cx,
                cy,
                half_length: leaf,
                half_width: w * 0.5 * size,
                cos: angle.cos(),
                sin: angle.sin(),
            }
        })
        .collect();
    (leaves, leaf)
}

fn lenses(count: u32, leaves: &[Leaf], leaf: f64, w: f64, rng: &mut XorShift) -> Vec<Lens> {
    let n = leaves.len();
    (0..count as usize)
        .map(|k| {
            if k + 1 < n {
                let cy = (leaves[k].cy + leaves[k + 1].cy) / 2.0;
                let cx = w * 0.5 + (rng.next() - 0.5) * w * 0.2;
                let half_height = leaf * (0.14 + rng.next() * 0.1);
                let half_width = w * (0.22 + rng.next() * 0.18);
                Lens {
                    cx,
                    cy,
                    half_height,
                    half_width,
                }
            } else {
                let host = &leaves[(k - (n - 1)) % n];
                let cy = host.cy + (rng.next() - 0.5) * leaf * 0.3;
                let cx = host.cx + (rng.next() - 0.5) * w * 0.2;
                let half_height = leaf * (0.1 + rng.next() * 0.08);
                let half_width = w * (0.12 + rng.next() * 0.12);
                Lens {
                    cx,
                    cy,
                    half_height,
                    half_width,
                }
            }
        })
        .collect()
}

fn paint(surface: &mut Surface, params: &HissParams, palette: &Palette, seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let u = (w * h).sqrt();
    let inks = palette.len();

    let mut rng = XorShift::new(seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let (leaves, leaf) = leaves(params, w, h, &mut rng);
    let lenses = lenses(params.checkers, &leaves, leaf, w, &mut rng);
    let comb_a = ink(palette, (rng.next() * inks as f64) as usize);
    let comb_b = ink(palette, (rng.next() * inks as f64) as usize);
    let sd = |k: u32| seed.wrapping_mul(7).wrapping_add(k);

    let cell = (params.steps > 0.0).then(|| (params.steps * u * 0.04).max(1.0));
    let mc = (u * 0.008).max(1.0);
    let sc = u * (0.3 + params.scale * 0.8);
    let warp = params.swirl * 1.4;
    let black = params.black * 0.85;
    let sp = (u * 0.007).max(1.0);
    let comb = params.comb * 0.9;
    let ch = (u * 0.02).max(2.0);
    let gsc = (u / 640.0 * (1.0 + params.coarse * 3.0)).max(1.0);
    let fleck = params.flecks * 0.03;
    let step = |at: f64| cell.map_or(at, |cell| ((at / cell).floor() + 0.5) * cell);

    surface.edit_rgba(|rgba, width, _| {
        for (y, row) in rgba.chunks_exact_mut(width as usize * 4).enumerate() {
            let yf = y as f64;
            let qy = step(yf);
            let my = (yf / mc).floor() * mc;
            for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let xf = x as f64;
                let qx = step(xf);
                let (p0, p1) = ((xf / mc).floor() * mc / sc, my / sc);
                let bn = value_noise(p0 * 0.8 + 40.0, p1 * 0.8 + 7.0, sd(4));
                let mut rgb = if (bn - 0.5) * 2.2 + 0.5 < black {
                    BLACK
                } else {
                    let q1 = value_noise(p0 * 2.0 + 1.7, p1 * 2.0 + 9.1, sd(1));
                    let q2 = value_noise(p0 * 2.0 + 5.3, p1 * 2.0 + 2.2, sd(2));
                    let m = value_noise(
                        p0 * 2.0 + warp * (q1 - 0.5) * 4.0,
                        p1 * 2.0 + warp * (q2 - 0.5) * 4.0,
                        sd(3),
                    );
                    let shade = 0.55 + 0.45 * q1;
                    marble(palette, m).map(|c| c * shade)
                };
                if comb > 0.0 {
                    let (ax, ay) = (qx / w, qy / h);
                    let si = (qx / sp).floor() as i32;
                    let sj = (qy / sp).floor() as i32;
                    let edge = |stripe: i32, row: i32, salt: u32| {
                        1.0 - comb + (hash(stripe, row, seed.wrapping_add(salt)) - 0.5) * 0.14
                    };
                    if ax - ay > edge(si, 0, 5) {
                        rgb = if si & 1 == 0 { comb_a } else { BLACK };
                    } else if ay - ax > edge(sj, 1, 6) {
                        rgb = if sj & 1 == 0 { comb_b } else { BLACK };
                    }
                }
                let in_leaf = leaves.iter().any(|leaf| {
                    let (ex, ey) = (qx - leaf.cx, qy - leaf.cy);
                    let across = ex * leaf.cos - ey * leaf.sin;
                    let along = ex * leaf.sin + ey * leaf.cos;
                    along > -leaf.half_length
                        && along < leaf.half_length
                        && across.abs() < leaf.half_width * taper(along, leaf.half_length)
                });
                if in_leaf {
                    let (gx, gy) = ((xf / gsc) as i32, (yf / gsc) as i32);
                    rgb = [12.0 + hash(gx, gy, 3).powf(1.6) * 220.0; 3];
                    let fleck_at = hash(gx, gy, 77);
                    if fleck > 0.0 && fleck_at < fleck {
                        rgb = ink(palette, (fleck_at / fleck * inks as f64) as usize % inks);
                    }
                }
                let in_lens = lenses.iter().any(|lens| {
                    let (across, along) = (qx - lens.cx, qy - lens.cy);
                    across > -lens.half_width
                        && across < lens.half_width
                        && along.abs() < lens.half_height * taper(across, lens.half_width)
                });
                if in_lens {
                    let on = ((qx / ch).floor() + (qy / ch).floor()) as i64 & 1 == 0;
                    rgb = [if on { 245.0 } else { 10.0 }; 3];
                }
                *px = [store(rgb[0]), store(rgb[1]), store(rgb[2]), 255];
            }
        }
    });

    marks(surface, params.marks, w, h, u, seed);
}

fn marks(surface: &mut Surface, count: u32, w: f64, h: f64, u: f64, seed: u32) {
    let mut rng = XorShift::new(seed.wrapping_mul(3_266_489_917) ^ 0x1656_67b1);
    for _ in 0..count {
        let x = (0.04 + rng.next() * 0.92) * w;
        let y = (0.04 + rng.next() * 0.92) * h;
        let size = u * 0.016 * (0.7 + rng.next() * 0.8);
        let arms: u32 = if rng.next() < 0.5 { 3 } else { 4 };
        let mut path = Path2D::default();
        for a in 0..arms {
            let angle = f64::from(a) / f64::from(arms) * PI + PI / 2.0;
            let (dx, dy) = (angle.cos() * size, angle.sin() * size);
            path.move_to(x - dx, y - dy);
            path.line_to(x + dx, y + dy);
        }
        surface.stroke(&path, MARK, (size * 0.2).max(1.0), Cap::Round, Join::Round);
        for a in 0..arms * 2 {
            let angle = f64::from(a) / f64::from(arms * 2) * TAU + PI / 2.0;
            surface.fill_circle(
                x + angle.cos() * size * 1.05,
                y + angle.sin() * size * 1.05,
                (size * 0.16).max(0.8),
                MARK,
            );
        }
    }
}
