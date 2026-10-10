use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "prism";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const FIELD: [u8; 3] = [0x08, 0x08, 0x0a];
const DOT: [u8; 3] = [0xff, 0xff, 0xff];
const RAMP: [&[[u8; 3]]; 5] = [
    &[[0xff, 0xff, 0xff]],
    &[[0xff, 0xe8, 0x00], [0xff, 0x9e, 0x00], [0xff, 0xc4, 0x00]],
    &[
        [0xff, 0x2d, 0x55],
        [0xff, 0x00, 0xa8],
        [0xe6, 0x00, 0xd8],
        [0xff, 0x3b, 0x00],
    ],
    &[
        [0x2e, 0x7c, 0xff],
        [0x00, 0xc8, 0xff],
        [0x6a, 0x2b, 0xd9],
        [0x00, 0x38, 0xff],
    ],
    &[[0x3a, 0x1e, 0xd8], [0x5a, 0x0f, 0xb0], [0x2b, 0x0f, 0xa8]],
];
const RINGS: [f64; 5] = [0.55, 0.75, 0.95, 1.12, 1.24];
const SPINE: usize = 29;

const COLS: Param = Param {
    step: 2,
    ..Param::new(SLUG, "cols", 24, 72, 1)
};
const STREAMS: Param = Param::new(SLUG, "streams", 1, 5, 1);
const REACH: Param = Param::new(SLUG, "reach", 30, 85, 100);
const FRINGE: Param = Param::new(SLUG, "fringe", 50, 160, 100);
const DOTS: Param = Param::new(SLUG, "dots", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[COLS, STREAMS, REACH, FRINGE, DOTS];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PrismParams {
    cols: u32,
    streams: u32,
    reach: f64,
    fringe: f64,
    dots: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for PrismParams {
    fn default() -> Self {
        Self {
            cols: 44,
            streams: 3,
            reach: 0.55,
            fringe: 1.0,
            dots: 0.55,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl PrismParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.ticks(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn streams(&self) -> u32 {
        self.streams
    }

    pub fn set_streams(&mut self, streams: u32) -> Result<(), Error> {
        STREAMS.check(f64::from(streams))?;
        self.streams = streams;
        Ok(())
    }

    pub fn reach(&self) -> f64 {
        self.reach
    }

    pub fn set_reach(&mut self, reach: f64) -> Result<(), Error> {
        self.reach = REACH.check(reach)?;
        Ok(())
    }

    pub fn fringe(&self) -> f64 {
        self.fringe
    }

    pub fn set_fringe(&mut self, fringe: f64) -> Result<(), Error> {
        self.fringe = FRINGE.check(fringe)?;
        Ok(())
    }

    pub fn dots(&self) -> f64 {
        self.dots
    }

    pub fn set_dots(&mut self, dots: f64) -> Result<(), Error> {
        self.dots = DOTS.check(dots)?;
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
    cols: u32,
    streams: u32,
    reach: f64,
    fringe: f64,
    dots: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<PrismParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = PrismParams::default();
    params.set_cols(raw.cols)?;
    params.set_streams(raw.streams)?;
    params.set_reach(raw.reach)?;
    params.set_fringe(raw.fringe)?;
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
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Option<Palette> {
    None
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> PrismParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    PrismParams {
        cols: pick(&COLS) as u32,
        streams: pick(&STREAMS) as u32,
        reach: pick(&REACH),
        fringe: pick(&FRINGE),
        dots: pick(&DOTS),
        ..PrismParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &PrismParams,
    _palette: Option<&Palette>,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

struct Stream {
    spine: [(f64, f64); SPINE],
    width: f64,
}

fn streams(p: &PrismParams, w: f64, h: f64, rng: &mut Xorshift) -> Vec<Stream> {
    let md = w.min(h);
    let dg = w.hypot(h);
    let mut anchors = [
        (0.0, 0.0),
        (w, 0.0),
        (0.0, h),
        (w, h),
        (w / 2.0, 0.0),
        (w / 2.0, h),
        (0.0, h / 2.0),
        (w, h / 2.0),
    ];
    for i in (1..anchors.len()).rev() {
        let j = (rng.next() * (i + 1) as f64) as usize;
        anchors.swap(i, j);
    }
    (0..p.streams as usize)
        .map(|i| {
            let (ax, ay) = anchors[i % anchors.len()];
            let mut tangents = Vec::with_capacity(2);
            let along_x = if ax < w / 2.0 { 0.0 } else { PI };
            let along_y = if ay < h / 2.0 { PI / 2.0 } else { -PI / 2.0 };
            if ay <= 0.0 {
                tangents.push(along_x);
            }
            if ay >= h {
                tangents.push(along_x);
            }
            if ax <= 0.0 {
                tangents.push(along_y);
            }
            if ax >= w {
                tangents.push(along_y);
            }
            let tangent = tangents[(rng.next() * tangents.len() as f64) as usize];
            let inward = (h / 2.0 - ay).atan2(w / 2.0 - ax);
            let tw = 0.35 + 0.45 * rng.next();
            let dx = tangent.cos() * tw + inward.cos() * (1.0 - tw);
            let dy = tangent.sin() * tw + inward.sin() * (1.0 - tw);
            let norm = match dx.hypot(dy) {
                0.0 => 1.0,
                n => n,
            };
            let aim = (dy / norm).atan2(dx / norm) + (rng.next() - 0.5) * 0.5;
            let length = dg * p.reach * (0.7 + 0.4 * rng.next());
            let (ca, sa) = (aim.cos(), aim.sin());
            let p0 = (ax - ca * md * 0.15, ay - sa * md * 0.15);
            let p2 = (ax + ca * length, ay + sa * length);
            let perp = aim + PI / 2.0;
            let bend = (rng.next() - 0.5) * length * 1.1;
            let p1 = (
                (p0.0 + p2.0) / 2.0 + perp.cos() * bend,
                (p0.1 + p2.1) / 2.0 + perp.sin() * bend,
            );
            let spine = std::array::from_fn(|q| {
                let t = q as f64 / (SPINE - 1) as f64;
                let u = 1.0 - t;
                (
                    u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                    u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
                )
            });
            let width = md * (0.16 + 0.08 * rng.next()) * p.fringe;
            rng.next();
            Stream { spine, width }
        })
        .collect()
}

fn cell(streams: &[Stream], (cx, cy): (f64, f64), gx: i32, gy: i32, key: u32) -> Option<[u8; 3]> {
    let (mut best, mut bq, mut bi) = (1e9, 0, 0);
    for (i, stream) in streams.iter().enumerate() {
        for (q, &(px, py)) in stream.spine.iter().enumerate() {
            let d =
                (cx - px).hypot(cy - py) / (stream.width * (1.15 - 0.45 * q as f64 / SPINE as f64));
            if d < best {
                (best, bq, bi) = (d, q, i);
            }
        }
    }
    let h = |x: i32, y: i32, z: u32| hash(x, y, z, key);
    let jitter = (h(gx, gy, 1) - 0.5) * 0.32 + (h(gx >> 1, gy >> 1, 15) - 0.5) * 0.42;
    let dn = best + jitter;
    let ring = match RINGS.iter().position(|&edge| dn < edge) {
        Some(ring) => ring,
        None if dn < 1.7 && h(gx, gy, 7) < 0.07 => 2 + (h(gx, gy, 8) * 2.4) as usize,
        None => return None,
    };
    let inks = RAMP[ring];
    let n = inks.len() as f64;
    let mut ink = (h(bi as i32 * 7 + ring as i32, (bq / 5) as i32, 21) * n) as usize;
    if h(gx, gy, 3 + ring as u32) < 0.22 {
        ink = (h(gx, gy, 9 + ring as u32) * n) as usize;
    }
    let dim = h(gx, gy, 5) * 0.06;
    Some(inks[ink].map(|c| {
        let c = f64::from(c);
        round_half_up(c - c * dim) as u8
    }))
}

fn paint(surface: &mut Surface, p: &PrismParams, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let mut rng = Xorshift::new(tool_seed);
    let streams = streams(p, w, h, &mut rng);
    let key = (rng.next() * 1e9) as u32;

    let gw = p.cols.max(8);
    let gh = ((f64::from(gw) * h / w).round() as u32).max(8);
    let (cw, ch) = (w / f64::from(gw), h / f64::from(gh));
    surface.fill(FIELD);
    for gy in 0..gh {
        for gx in 0..gw {
            let (fx, fy) = (f64::from(gx), f64::from(gy));
            let centre = ((fx + 0.5) * cw, (fy + 0.5) * ch);
            let Some(ink) = cell(&streams, centre, gx as i32, gy as i32, key) else {
                continue;
            };
            let (x0, x1) = ((fx * cw).round(), ((fx + 1.0) * cw).round());
            let (y0, y1) = ((fy * ch).round(), ((fy + 1.0) * ch).round());
            surface.fill_rect(
                x0 as i32,
                y0 as i32,
                (x1 - x0) as u32,
                (y1 - y0) as u32,
                ink,
            );
        }
    }
    if p.dots > 0.0 {
        let side = (cw.min(ch) * 0.07).max(1.0);
        surface.with_alpha((p.dots * 0.8) as f32, |surface| {
            for gy in 0..gh {
                for gx in 0..gw {
                    let x = (f64::from(gx) + 0.5) * cw - side / 2.0;
                    let y = (f64::from(gy) + 0.5) * ch - side / 2.0;
                    surface.fill_box(x, y, side, side, DOT);
                }
            }
        });
    }
}
