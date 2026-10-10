use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "static";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 10;
pub(crate) const MAX_INKS: Option<usize> = Some(2);

const DEFAULT_PALETTE: [&str; 2] = ["#ff3cac", "#1b1b2f"];

const REGIONS: Param = Param::new(SLUG, "regions", 2, 5, 1);
const RES: Param = Param {
    step: 4,
    ..Param::new(SLUG, "res", 48, 160, 1)
};
const GLITCH: Param = Param::new(SLUG, "glitch", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[REGIONS, RES, GLITCH];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StaticParams {
    regions: u32,
    res: u32,
    glitch: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for StaticParams {
    fn default() -> Self {
        Self {
            regions: 3,
            res: 84,
            glitch: 0.5,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl StaticParams {
    pub fn regions(&self) -> u32 {
        self.regions
    }

    pub fn set_regions(&mut self, regions: u32) -> Result<(), Error> {
        REGIONS.check(f64::from(regions))?;
        self.regions = regions;
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

    pub fn glitch(&self) -> f64 {
        self.glitch
    }

    pub fn set_glitch(&mut self, glitch: f64) -> Result<(), Error> {
        self.glitch = GLITCH.check(glitch)?;
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
    regions: u32,
    res: u32,
    glitch: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<StaticParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = StaticParams::default();
    params.set_regions(raw.regions)?;
    params.set_res(raw.res)?;
    params.set_glitch(raw.glitch)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> StaticParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    StaticParams {
        regions: pick(&REGIONS) as u32,
        res: pick(&RES) as u32,
        glitch: pick(&GLITCH),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &StaticParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Kind {
    Moire,
    Bars,
    Static,
    Blocks,
    Rings,
    Zigzag,
}

const KINDS: [Kind; 6] = [
    Kind::Moire,
    Kind::Bars,
    Kind::Static,
    Kind::Blocks,
    Kind::Rings,
    Kind::Zigzag,
];

struct Region {
    kind: Kind,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    a: f64,
    b: f64,
    warp: f64,
    th: f64,
    cx: f64,
    cy: f64,
    rk: f64,
    ph: f64,
    dens: f64,
    grad: f64,
    q: f64,
    duty: f64,
    p: f64,
    slope: f64,
    vert: bool,
    rs: u32,
}

fn region(
    rng: &mut Xorshift,
    (x0, x1, y0, y1): (f64, f64, f64, f64),
    (gw, gh): (f64, f64),
) -> Region {
    let kind = KINDS[(rng.next() * KINDS.len() as f64) as usize];
    let a = 0.15 + rng.next() * 0.75;
    let b = 0.06 + rng.next() * 0.4;
    let warp = rng.next() * 6.0;
    let th = (rng.next() - 0.5) * 0.8;
    let cx = (rng.next() * 1.6 - 0.3) * gw;
    let cy = (rng.next() * 1.6 - 0.3) * gh;
    let rk = 0.12 + rng.next() * 0.4;
    let ph = rng.next() * TAU;
    let dens = 0.2 + rng.next() * 0.5;
    let grad = (rng.next() - 0.5) * 0.9;
    let q = 2.0 + (rng.next() * 5.0).floor();
    let duty = 0.35 + rng.next() * 0.35;
    let p = 5.0 + (rng.next() * 10.0).floor();
    let sign = if rng.next() < 0.5 { 1.0 } else { -1.0 };
    let slope = sign * (0.3 + rng.next() * 1.2);
    let vert = rng.next() < 0.35;
    let rs = (rng.next() * 1e9) as u32;
    Region {
        kind,
        x0,
        x1,
        y0,
        y1,
        a,
        b,
        warp,
        th,
        cx,
        cy,
        rk,
        ph,
        dens,
        grad,
        q,
        duty,
        p,
        slope,
        vert,
        rs,
    }
}

fn js_hypot(x: f64, y: f64) -> f64 {
    let (x, y) = (x.abs(), y.abs());
    let max = x.max(y);
    if max == 0.0 {
        return 0.0;
    }
    let (mut sum, mut compensation) = (0.0, 0.0);
    for n in [x / max, y / max] {
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

fn modulo(value: f64, by: f64) -> f64 {
    (value % by + by) % by
}

impl Region {
    fn h(&self, x: f64, y: f64, channel: u32) -> f64 {
        hash(x as i32, y as i32, channel, self.rs)
    }

    fn on(&self, u: f64, v: f64) -> bool {
        match self.kind {
            Kind::Moire => {
                let w1 = (u * self.a + (v * self.b).sin() * self.warp + self.ph).sin();
                let w2 = (js_hypot(u - self.cx, v - self.cy) * self.rk).sin();
                w1 + w2 > self.th
            }
            Kind::Rings => (js_hypot(u - self.cx, v - self.cy) * self.rk + self.ph).sin() > self.th,
            Kind::Bars if self.vert => {
                let vv = modulo(v - self.y0, self.y1 - self.y0);
                let seg = self.h(u, (vv / self.q).floor(), 1);
                self.h(u, 0.0, 7) < 0.8 && seg < self.duty + (vv * 0.2).sin() * 0.1
            }
            Kind::Bars => {
                let uu = modulo(u - self.x0, self.x1 - self.x0);
                let seg = self.h((uu / self.q).floor(), v, 1);
                self.h(0.0, v, 7) < 0.8 && seg < self.duty + (u * 0.15).sin() * 0.08
            }
            Kind::Static => {
                let rows = (self.y1 - self.y0).max(1.0);
                let d =
                    (self.dens + self.grad * ((v - self.y0) / rows - 0.5) * 2.0).clamp(0.02, 0.95);
                self.h(u, v, 0) < d
            }
            Kind::Blocks => {
                let (small, large) = (self.q * 2.0, self.q * 5.0);
                let a = self.h((u / small).floor(), (v / small).floor(), 0) < 0.5;
                let b = self.h((u / large).floor(), (v / large).floor(), 3) < 0.5;
                a != b
            }
            Kind::Zigzag => modulo(u + (v * self.slope).floor(), self.p) < self.p * self.duty,
        }
    }
}

fn bits(params: &StaticParams, gw: usize, gh: usize, tool_seed: u32) -> Vec<bool> {
    let grid = (gw as f64, gh as f64);
    let mut rng = Xorshift::new(tool_seed);
    let weights: Vec<f64> = (0..params.regions)
        .map(|_| 0.5 + rng.next() * 1.2)
        .collect();
    let sum: f64 = weights.iter().sum();
    let mut bands = Vec::with_capacity(weights.len());
    let mut y = 0.0;
    for (i, weight) in weights.iter().enumerate() {
        let h = round_half_up(weight / sum * grid.1);
        let y1 = if i == weights.len() - 1 {
            grid.1
        } else {
            grid.1.min(y + h)
        };
        bands.push(region(&mut rng, (0.0, grid.0, y, y1), grid));
        y = y1;
    }
    let inset = (rng.next() < 0.4).then(|| {
        let x0 = round_half_up(grid.0 * (0.1 + rng.next() * 0.25));
        let x1 = round_half_up(grid.0 * (0.65 + rng.next() * 0.3));
        let y0 = round_half_up(grid.1 * (0.15 + rng.next() * 0.3));
        let y1 = round_half_up(grid.1 * (0.55 + rng.next() * 0.35));
        region(&mut rng, (x0, x1, y0, y1), grid)
    });
    let mut bits = Vec::with_capacity(gw * gh);
    for v in 0..gh {
        let v = v as f64;
        let band = bands
            .iter()
            .find(|band| v >= band.y0 && v < band.y1)
            .unwrap_or(&bands[0]);
        for u in 0..gw {
            let u = u as f64;
            let region = match &inset {
                Some(inset) if u >= inset.x0 && u < inset.x1 && v >= inset.y0 && v < inset.y1 => {
                    inset
                }
                _ => band,
            };
            bits.push(region.on(u, v));
        }
    }
    glitch(&mut bits, params.glitch, gw, gh, tool_seed);
    bits
}

fn glitch(bits: &mut [bool], amount: f64, gw: usize, gh: usize, tool_seed: u32) {
    let mut rng = Xorshift::from_state(tool_seed ^ 0x9e37_79b9);
    let (w, h) = (gw as f64, gh as f64);
    for _ in 0..round_half_up(h * 0.3 * amount) as u32 {
        let y = (rng.next() * h) as usize;
        let x0 = (rng.next() * w) as usize;
        let len = (1.0 + rng.next() * w * 0.5) as usize;
        let dx = (1.0 + rng.next() * 7.0) as usize;
        let row = bits[y * gw..(y + 1) * gw].to_vec();
        for x in x0..gw.min(x0 + len) {
            bits[y * gw + x] = row[(x + dx) % gw];
        }
    }
    for _ in 0..round_half_up(h * 0.08 * amount) as u32 {
        let y = 1 + (rng.next() * (h - 1.0)) as usize;
        let reps = (1.0 + rng.next() * 3.0) as usize;
        for k in (0..reps).take_while(|k| y + k < gh) {
            bits.copy_within((y - 1) * gw..y * gw, (y + k) * gw);
        }
    }
    for _ in 0..round_half_up(6.0 * amount) as u32 {
        let x0 = (rng.next() * w) as usize;
        let y0 = (rng.next() * h) as usize;
        let width = (2.0 + rng.next() * w * 0.2) as usize;
        let height = (1.0 + rng.next() * h * 0.06) as usize;
        let value = rng.next() >= 0.5;
        for y in y0..gh.min(y0 + height) {
            bits[y * gw + x0..y * gw + gw.min(x0 + width)].fill(value);
        }
    }
}

fn paint(surface: &mut Surface, params: &StaticParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let columns = params.res;
    let rows = (round_half_up(f64::from(columns) * height / width) as u32).max(8);
    let (ink, ground) = (palette.ink(0), palette.ink(1));
    let rgba: Vec<u8> = bits(params, columns as usize, rows as usize, tool_seed)
        .into_iter()
        .flat_map(|on| {
            let [r, g, b] = if on { ink } else { ground };
            [r, g, b, 255]
        })
        .collect();
    surface.draw_smooth(&rgba, columns, rows, Smoothing::Nearest);
}
