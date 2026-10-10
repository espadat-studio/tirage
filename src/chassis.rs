use serde::{Deserialize, Serialize};

use crate::param::Param;
use crate::surface::Surface;
use crate::{Error, Parameter};

pub(crate) fn hash(x: i32, y: i32, seed: u32) -> f64 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(1_274_126_177));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

pub(crate) fn value_noise(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash(xi.wrapping_add(dx), yi.wrapping_add(dy), seed);
    (corner(0, 0) * (1.0 - u) + corner(1, 0) * u) * (1.0 - v)
        + (corner(0, 1) * (1.0 - u) + corner(1, 1) * u) * v
}

pub(crate) fn fbm(x: f64, y: f64, seed: u32, octaves: u32) -> f64 {
    let (mut sum, mut weight, mut total, mut frequency) = (0.0, 0.5, 0.0, 1.0);
    for octave in 0..octaves {
        sum += weight
            * value_noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 131),
            );
        total += weight;
        frequency *= 2.0;
        weight *= 0.5;
    }
    sum / total
}

pub(crate) struct XorShift(u32);

impl XorShift {
    pub(crate) fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    pub(crate) fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        f64::from(self.0) / 4_294_967_296.0
    }
}

pub(crate) fn round_half_up(value: f64) -> f64 {
    let floor = value.floor();
    if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    }
}

pub(crate) fn store(value: f64) -> u8 {
    value.clamp(0.0, 255.0).round_ties_even() as u8
}

pub(crate) fn finish(surface: &mut Surface, grain: &Grain, dither: &Dither) {
    if !grain.is_active() && !dither.is_active() {
        return;
    }
    surface.edit_rgba(|rgba, width, height| {
        grain.apply(rgba, width, height);
        dither.apply(rgba, width, height);
    });
}

pub(crate) fn parameters() -> [Parameter; 11] {
    [
        Parameter::toggle("ditherTog"),
        Parameter::choice("dthKinds", &["Bayer 8", "Bayer 4", "Noise"]),
        DITHER_SIZE.parameter(),
        DITHER_LEVELS.parameter(),
        DITHER_AMOUNT.parameter(),
        Parameter::toggle("grainTog"),
        Parameter::choice(
            "grnBlends",
            &["Add", "Overlay", "Soft light", "Multiply", "Screen"],
        ),
        GRAIN_AMOUNT.parameter(),
        GRAIN_SIZE.parameter(),
        GRAIN_SPECKS.parameter(),
        GRAIN_VIGNETTE.parameter(),
    ]
}

const GRAIN: &str = "grain";
const GRAIN_AMOUNT: Param = Param {
    step: 5,
    ..Param::new(GRAIN, "grnAmount", 0, 100, 100)
};
const GRAIN_SIZE: Param = Param::new(GRAIN, "grnSize", 5, 40, 10);
const GRAIN_SPECKS: Param = Param {
    step: 5,
    ..Param::new(GRAIN, "grnSpecks", 0, 100, 100)
};
const GRAIN_VIGNETTE: Param = Param {
    step: 5,
    ..Param::new(GRAIN, "grnVignette", 0, 100, 100)
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blend {
    Add,
    Overlay,
    #[serde(rename = "Soft light")]
    SoftLight,
    Multiply,
    Screen,
}

impl Blend {
    fn target(self, c: f64, k: f64) -> f64 {
        let n = (128.0 + k) / 255.0;
        match self {
            Self::Add => c + k,
            Self::Multiply => c * n,
            Self::Screen => 255.0 - (255.0 - c) * (1.0 - n),
            Self::Overlay if c < 128.0 => 2.0 * c * n,
            Self::Overlay => 255.0 - 2.0 * (255.0 - c) * (1.0 - n),
            Self::SoftLight => (1.0 - 2.0 * n) * c * c / 255.0 + 2.0 * n * c,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Grain {
    #[serde(rename = "grainTog")]
    on: bool,
    #[serde(rename = "grnBlends")]
    blend: Blend,
    #[serde(rename = "grnAmount")]
    amount: f64,
    #[serde(rename = "grnSize")]
    size: f64,
    #[serde(rename = "grnSpecks")]
    specks: f64,
    #[serde(rename = "grnVignette")]
    vignette: f64,
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            on: false,
            blend: Blend::Add,
            amount: 0.55,
            size: 1.0,
            specks: 0.5,
            vignette: 0.5,
        }
    }
}

impl Grain {
    pub(crate) fn textile() -> Self {
        Self {
            on: true,
            blend: Blend::Multiply,
            amount: 0.45,
            size: 1.0,
            specks: 0.4,
            vignette: 0.3,
        }
    }

    pub(crate) fn printed() -> Self {
        Self {
            on: true,
            blend: Blend::Overlay,
            amount: 0.4,
            size: 1.0,
            specks: 0.4,
            vignette: 0.25,
        }
    }

    pub fn on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn blend(&self) -> Blend {
        self.blend
    }

    pub fn set_blend(&mut self, blend: Blend) {
        self.blend = blend;
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn set_amount(&mut self, amount: f64) -> Result<(), Error> {
        self.amount = GRAIN_AMOUNT.check(amount)?;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = GRAIN_SIZE.check(size)?;
        Ok(())
    }

    pub fn specks(&self) -> f64 {
        self.specks
    }

    pub fn set_specks(&mut self, specks: f64) -> Result<(), Error> {
        self.specks = GRAIN_SPECKS.check(specks)?;
        Ok(())
    }

    pub fn vignette(&self) -> f64 {
        self.vignette
    }

    pub fn set_vignette(&mut self, vignette: f64) -> Result<(), Error> {
        self.vignette = GRAIN_VIGNETTE.check(vignette)?;
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.on && self.amount > 0.0
    }

    fn apply(&self, rgba: &mut [u8], width: u32, height: u32) {
        if !self.is_active() {
            return;
        }
        let (w, h) = (f64::from(width), f64::from(height));
        let cell = ((w * h).sqrt() / 640.0 * self.size).max(1.0);
        let clump_scale = 11.0 * cell;
        let speck_cut = self.specks * 0.035;
        let vignette = self.vignette * 0.35;
        let amount = self.amount;
        let mut clumps = vec![0.0_f32; (w / cell).ceil() as usize + 1];
        let mut clump_row = None;
        for (y, row) in rgba.chunks_exact_mut(width as usize * 4).enumerate() {
            let y = y as f64;
            let gy = (y / cell) as i32;
            if clump_row != Some(gy) {
                for (gx, clump) in clumps.iter_mut().enumerate() {
                    *clump =
                        value_noise(gx as f64 * cell / clump_scale, y / clump_scale, 41) as f32;
                }
                clump_row = Some(gy);
            }
            let ey = (y / h - 0.5).abs() * 2.0;
            for (x, pixel) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let x = x as f64;
                let gx = (x / cell) as usize;
                let draw = hash(gx as i32, gy, 3);
                let k = (draw - 0.5) * 120.0 * (0.5 + f64::from(clumps[gx]));
                let ex = (x / w - 0.5).abs() * 2.0;
                let e = if ex > ey { ex } else { ey };
                let shade = 1.0 - vignette * amount * e * e * e;
                for channel in &mut pixel[..3] {
                    let c = f64::from(*channel);
                    let mut c = c + (self.blend.target(c, k) - c) * amount;
                    if draw > 1.0 - speck_cut {
                        c += 95.0 * amount;
                    }
                    *channel = store(c * shade);
                }
            }
        }
    }
}

const DITHER: &str = "dither";
const DITHER_SIZE: Param = Param::new(DITHER, "dthSize", 1, 10, 1);
const DITHER_LEVELS: Param = Param::new(DITHER, "dthLevels", 2, 8, 1);
const DITHER_AMOUNT: Param = Param {
    step: 5,
    ..Param::new(DITHER, "dthAmount", 0, 100, 100)
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DitherKind {
    #[serde(rename = "Bayer 8")]
    Bayer8,
    #[serde(rename = "Bayer 4")]
    Bayer4,
    Noise,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dither {
    #[serde(rename = "ditherTog")]
    on: bool,
    #[serde(rename = "dthKinds")]
    kind: DitherKind,
    #[serde(rename = "dthSize")]
    size: u32,
    #[serde(rename = "dthLevels")]
    levels: u32,
    #[serde(rename = "dthAmount")]
    amount: f64,
}

impl Default for Dither {
    fn default() -> Self {
        Self {
            on: false,
            kind: DitherKind::Bayer8,
            size: 2,
            levels: 3,
            amount: 1.0,
        }
    }
}

impl Dither {
    pub fn on(&self) -> bool {
        self.on
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn kind(&self) -> DitherKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: DitherKind) {
        self.kind = kind;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_size(&mut self, size: u32) -> Result<(), Error> {
        DITHER_SIZE.check(f64::from(size))?;
        self.size = size;
        Ok(())
    }

    pub fn levels(&self) -> u32 {
        self.levels
    }

    pub fn set_levels(&mut self, levels: u32) -> Result<(), Error> {
        DITHER_LEVELS.check(f64::from(levels))?;
        self.levels = levels;
        Ok(())
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn set_amount(&mut self, amount: f64) -> Result<(), Error> {
        self.amount = DITHER_AMOUNT.check(amount)?;
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.on && self.amount > 0.0
    }

    fn apply(&self, rgba: &mut [u8], width: u32, height: u32) {
        if !self.is_active() {
            return;
        }
        let block = round_half_up(f64::from(self.size) * f64::from(width.min(height)) / 700.0)
            .max(1.0) as usize;
        let side = if self.kind == DitherKind::Bayer4 {
            4
        } else {
            8
        };
        let cells = side * side;
        let steps = f64::from(self.levels - 1);
        let lut: Vec<[u8; 256]> = (0..cells)
            .map(|cell| {
                let threshold = (cell as f64 + 0.5) / cells as f64;
                std::array::from_fn(|v| {
                    let q = v as f64 / 255.0 * steps;
                    let floor = q.floor();
                    let step = (floor + if q - floor > threshold { 1.0 } else { 0.0 }).min(steps);
                    round_half_up(step / steps * 255.0) as u8
                })
            })
            .collect();
        let (w, h) = (width as usize, height as usize);
        for by in (0..h).step_by(block) {
            let rows = by..(by + block).min(h);
            for bx in (0..w).step_by(block) {
                let cols = bx..(bx + block).min(w);
                let (gx, gy) = (bx / block, by / block);
                let pixels = || {
                    rows.clone()
                        .flat_map(|y| cols.clone().map(move |x| (y * w + x) * 4))
                };
                let mut sum = [0_u32; 3];
                for p in pixels() {
                    for (total, &channel) in sum.iter_mut().zip(&rgba[p..p + 3]) {
                        *total += u32::from(channel);
                    }
                }
                let count = pixels().count() as u32;
                let cell = match self.kind {
                    DitherKind::Noise => {
                        ((noise_hash(gx, gy) * cells as f64) as usize).min(cells - 1)
                    }
                    _ => bayer(gx % side, gy % side, side),
                };
                let ink = sum.map(|total| lut[cell][(total / count) as usize]);
                for p in pixels() {
                    for (channel, &ink) in rgba[p..p + 3].iter_mut().zip(&ink) {
                        *channel = if self.amount >= 1.0 {
                            ink
                        } else {
                            let c = f64::from(*channel);
                            store(c + (f64::from(ink) - c) * self.amount)
                        };
                    }
                }
            }
        }
    }
}

fn bayer(x: usize, y: usize, side: usize) -> usize {
    if side == 1 {
        return 0;
    }
    let half = side / 2;
    let quadrant = match (x >= half, y >= half) {
        (false, false) => 0,
        (true, false) => 2,
        (false, true) => 3,
        (true, true) => 1,
    };
    bayer(x % half, y % half, half) * 4 + quadrant
}

fn noise_hash(x: usize, y: usize) -> f64 {
    let a = to_int32(x as f64 * 374_761_393.0 + y as f64 * 668_265_263.0);
    let b = to_int32(f64::from(a ^ (a >> 13)) * 1_274_126_177.0);
    f64::from((b ^ (b >> 16)) as u32) / 4_294_967_296.0
}

fn to_int32(value: f64) -> i32 {
    value.rem_euclid(4_294_967_296.0) as u32 as i32
}
