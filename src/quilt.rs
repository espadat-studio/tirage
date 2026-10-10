use std::f64::consts::{FRAC_PI_2, TAU};

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "quilt";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 10;
pub(crate) const MAX_INKS: Option<usize> = Some(4);

const DEFAULT_PALETTE: [&str; 4] = ["#f3e9dc", "#2a6f97", "#c8553d", "#f4d35e"];

const CELLS: Param = Param {
    step: 2,
    ..Param::new(SLUG, "cells", 28, 72, 1)
};
const CHUNK: Param = Param {
    step: 5,
    ..Param::new(SLUG, "chunk", 70, 180, 100)
};

pub(crate) const PARAMS: &[Param] = &[CELLS, CHUNK];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuiltStyle {
    Auto,
    Bands,
    Tabs,
    Plaid,
    Dither,
    Steps,
    Zigzag,
    Diamond,
    Cross,
    Basket,
    Rings,
    Star,
    Waves,
    Gingham,
    Burst,
}

const STYLES: [QuiltStyle; 15] = [
    QuiltStyle::Auto,
    QuiltStyle::Bands,
    QuiltStyle::Tabs,
    QuiltStyle::Plaid,
    QuiltStyle::Dither,
    QuiltStyle::Steps,
    QuiltStyle::Zigzag,
    QuiltStyle::Diamond,
    QuiltStyle::Cross,
    QuiltStyle::Basket,
    QuiltStyle::Rings,
    QuiltStyle::Star,
    QuiltStyle::Waves,
    QuiltStyle::Gingham,
    QuiltStyle::Burst,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuiltParams {
    #[serde(rename = "styles")]
    style: QuiltStyle,
    cells: u32,
    chunk: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for QuiltParams {
    fn default() -> Self {
        Self {
            style: QuiltStyle::Auto,
            cells: 44,
            chunk: 1.0,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl QuiltParams {
    pub fn style(&self) -> QuiltStyle {
        self.style
    }

    pub fn set_style(&mut self, style: QuiltStyle) {
        self.style = style;
    }

    pub fn cells(&self) -> u32 {
        self.cells
    }

    pub fn set_cells(&mut self, cells: u32) -> Result<(), Error> {
        CELLS.check(f64::from(cells))?;
        self.cells = cells;
        Ok(())
    }

    pub fn chunk(&self) -> f64 {
        self.chunk
    }

    pub fn set_chunk(&mut self, chunk: f64) -> Result<(), Error> {
        self.chunk = CHUNK.check(chunk)?;
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
    style: QuiltStyle,
    cells: u32,
    chunk: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<QuiltParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = QuiltParams::default();
    params.set_style(raw.style);
    params.set_cells(raw.cells)?;
    params.set_chunk(raw.chunk)?;
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
        &[
            "Auto", "Bands", "Tabs", "Plaid", "Dither", "Steps", "Zigzag", "Diamond", "Cross",
            "Basket", "Rings", "Star", "Waves", "Gingham", "Burst",
        ],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> QuiltParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    QuiltParams {
        style: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        cells: pick(&CELLS) as u32,
        chunk: pick(&CHUNK),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &QuiltParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

const POOL: [QuiltStyle; 14] = [
    QuiltStyle::Bands,
    QuiltStyle::Tabs,
    QuiltStyle::Plaid,
    QuiltStyle::Dither,
    QuiltStyle::Steps,
    QuiltStyle::Zigzag,
    QuiltStyle::Diamond,
    QuiltStyle::Cross,
    QuiltStyle::Basket,
    QuiltStyle::Rings,
    QuiltStyle::Star,
    QuiltStyle::Waves,
    QuiltStyle::Gingham,
    QuiltStyle::Burst,
];

const BAYER: [f64; 16] = [
    0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0,
];

struct Pattern {
    style: QuiltStyle,
    rs: u32,
    bh: f64,
    l: f64,
    tw: f64,
    th: f64,
    gx: f64,
    gy: f64,
    per: f64,
    pw: f64,
    f1: f64,
    f2: f64,
    warp: f64,
    ph0: f64,
    rw: f64,
    rh: f64,
    off: f64,
    steps: Vec<u8>,
    zper: f64,
    zslope: f64,
    zigzag: Vec<u8>,
    dw: f64,
    dh: f64,
    ct: f64,
    ca: f64,
    bb: f64,
    bs: f64,
    rings: Vec<u8>,
    stt: f64,
    wper: f64,
    wamp: f64,
    waves: Vec<u8>,
    gg: f64,
    bn: f64,
    bcore: f64,
}

fn size(base: f64, chunk: f64, min: f64) -> f64 {
    round_half_up(base * chunk).max(min)
}

fn run(entries: &[(u8, f64)]) -> Vec<u8> {
    entries
        .iter()
        .flat_map(|&(role, width)| std::iter::repeat_n(role, width as usize))
        .collect()
}

fn pattern(params: &QuiltParams, tool_seed: u32) -> Pattern {
    let mut rng = Xorshift::new(tool_seed);
    let ch = params.chunk;
    let style = match params.style {
        QuiltStyle::Auto => POOL[(rng.next() * POOL.len() as f64) as usize],
        style => style,
    };
    let rs = (rng.next() * 1e9) as u32;
    let bh = size(2.2, ch, 2.0);
    let l = size(6.0 + rng.next() * 4.0, ch, 4.0);
    let tw = size(3.0 + rng.next() * 2.0, ch, 2.0);
    let th = size(1.6, ch, 1.0);
    let gx = size(2.0, ch, 1.0);
    let gy = size(1.4, ch, 1.0);
    let per = size(9.0 + rng.next() * 4.0, ch, 6.0);
    let pw = size(1.8, ch, 1.0);
    let f1 = 0.22 + rng.next() * 0.25;
    let f2 = 0.09 + rng.next() * 0.08;
    let warp = 1.2 + rng.next() * 2.2;
    let ph0 = rng.next() * TAU;
    let rw = size(3.0 + rng.next() * 2.0, ch, 2.0);
    let rh = size(2.0, ch, 1.0);
    let off = (rng.next() * 97.0).floor();
    let wide = size(7.0 + rng.next() * 5.0, ch, 5.0);
    let t1 = size(1.2, ch, 1.0);
    let w2 = size(2.5 + rng.next() * 2.0, ch, 2.0);
    let w3 = size(2.0 + rng.next() * 2.0, ch, 2.0);
    let steps = if rng.next() < 0.5 {
        run(&[(0, wide), (3, t1), (2, w2), (1, w3), (3, t1)])
    } else {
        run(&[(0, wide), (1, w3), (2, w2), (3, t1)])
    };
    let zper = size(10.0 + rng.next() * 6.0, ch, 6.0);
    let zslope = 0.5 + rng.next() * 0.5;
    let zrh = size(2.5, ch, 2.0);
    let zigzag = if rng.next() < 0.5 {
        run(&[(1, zrh), (0, zrh), (2, zrh), (0, zrh)])
    } else {
        run(&[(1, zrh), (0, zrh), (2, zrh), (3, t1), (0, zrh)])
    };
    let dw = size(9.0 + rng.next() * 5.0, ch, 6.0);
    let dh = size(7.0 + rng.next() * 4.0, ch, 6.0);
    let ct = size(7.0 + rng.next() * 3.0, ch, 5.0);
    let ca = size(1.1, ch, 1.0);
    let bb = size(5.0 + rng.next() * 3.0, ch, 4.0);
    let bs = size(1.4, ch, 1.0);
    let rw1 = size(2.0 + rng.next() * 2.0, ch, 2.0);
    let rings = if rng.next() < 0.5 {
        run(&[(0, rw1 + 1.0), (1, rw1), (0, rw1), (2, rw1), (3, t1)])
    } else {
        run(&[(0, rw1 + 1.0), (2, rw1), (3, t1), (1, rw1)])
    };
    let stt = size(10.0 + rng.next() * 4.0, ch, 7.0);
    let wper = size(8.0 + rng.next() * 5.0, ch, 6.0);
    let wamp = size(2.6, ch, 2.0);
    let wh = size(2.4, ch, 2.0);
    let waves = if rng.next() < 0.5 {
        run(&[(1, wh), (0, wh), (2, wh), (0, wh)])
    } else {
        run(&[(1, wh), (0, wh), (3, t1), (2, wh), (0, wh)])
    };
    let gg = size(3.0 + rng.next() * 2.0, ch, 2.0);
    let bn = 3.0 + (rng.next() * 3.0).floor();
    let bcore = size(3.0, ch, 2.0);
    Pattern {
        style,
        rs,
        bh,
        l,
        tw,
        th,
        gx,
        gy,
        per,
        pw,
        f1,
        f2,
        warp,
        ph0,
        rw,
        rh,
        off,
        steps,
        zper,
        zslope,
        zigzag,
        dw,
        dh,
        ct,
        ca,
        bb,
        bs,
        rings,
        stt,
        wper,
        wamp,
        waves,
        gg,
        bn,
        bcore,
    }
}

fn lookup(run: &[u8], d: f64) -> u8 {
    run[(d as i64).rem_euclid(run.len() as i64) as usize]
}

fn odd(value: f64) -> bool {
    (value as i64) & 1 == 1
}

fn half(value: f64) -> f64 {
    f64::from((value as i32) >> 1)
}

impl Pattern {
    fn h(&self, x: f64, y: f64, channel: u32) -> f64 {
        hash(x as i32, y as i32, channel, self.rs)
    }

    fn role(&self, u: f64, v: f64, (mx, my): (f64, f64), (gw, gh): (f64, f64)) -> u8 {
        let (ax, ay) = (u - mx, v - my);
        let (ux, vy) = (ax.abs(), ay.abs());
        let spills = |cx: f64, cy: f64, rx: f64, ry: f64| {
            cx - rx < -0.5 || cx + rx > gw - 0.5 || cy - ry < -0.5 || cy + ry > gh - 0.5
        };
        match self.style {
            QuiltStyle::Bands => {
                let b = (vy / self.bh).floor();
                let uu = ux + (self.h(0.0, b, 2) * self.l).floor();
                let k = (uu / self.l).floor();
                let mut role = (self.h(k, b, 5) * 3.0).floor() as u8;
                if self.h(k, b, 9) < 0.22 {
                    let lx = (uu % self.l + self.l) % self.l;
                    let start = half(self.l - self.bh);
                    if lx >= start && lx < start + self.bh {
                        role = 3;
                    }
                }
                role
            }
            QuiltStyle::Tabs => {
                let (py, px) = (self.th + self.gy, self.tw + self.gx);
                let row = (vy / py).floor();
                if vy - row * py >= self.th {
                    return 0;
                }
                let uu = ux + if odd(row) { half(px) } else { 0.0 };
                if (uu % px + px) % px >= self.tw {
                    return 0;
                }
                if self.h((uu / px).floor(), row, 3) < 0.12 {
                    2
                } else {
                    1
                }
            }
            QuiltStyle::Zigzag => {
                let uu = (ux % self.zper + self.zper) % self.zper;
                let tri = (uu * 2.0 - self.zper).abs() / 2.0;
                lookup(&self.zigzag, (vy + tri * self.zslope).floor())
            }
            QuiltStyle::Diamond => {
                let (tx, ty) = (round_half_up(ax / self.dw), round_half_up(ay / self.dh));
                let (cx, cy) = (mx + tx * self.dw, my + ty * self.dh);
                if spills(cx, cy, self.dw / 2.0, self.dh / 2.0) {
                    return 0;
                }
                let du = (ax - tx * self.dw).abs() / (self.dw / 2.0);
                let dv = (ay - ty * self.dh).abs() / (self.dh / 2.0);
                let (md, p) = (du + dv, odd(tx + ty));
                if md <= 0.4 {
                    if p { 3 } else { 1 }
                } else if md <= 0.95 {
                    if p { 1 } else { 2 }
                } else {
                    0
                }
            }
            QuiltStyle::Cross => {
                let t = self.ct;
                let (tx, ty) = (round_half_up(ax / t), round_half_up(ay / t));
                let arm = (t * 0.38).floor().max(2.0);
                let (cx, cy) = (mx + tx * t, my + ty * t);
                if spills(cx, cy, arm, arm) {
                    return 0;
                }
                let (du, dv) = ((ax - tx * t).abs(), (ay - ty * t).abs());
                if (du < self.ca && dv <= arm) || (dv < self.ca && du <= arm) {
                    if self.h(tx.abs(), ty.abs(), 4) < 0.15 {
                        3
                    } else if odd(tx + ty) {
                        1
                    } else {
                        2
                    }
                } else {
                    0
                }
            }
            QuiltStyle::Basket => {
                let (bx, by) = ((ux / self.bb).floor(), (vy / self.bb).floor());
                let stripe = self.bs * 2.0;
                if odd(bx + by) {
                    u8::from(vy.floor() % stripe < self.bs)
                } else if ux.floor() % stripe < self.bs {
                    2
                } else {
                    0
                }
            }
            QuiltStyle::Rings => lookup(&self.rings, (ux * (gh / gw)).max(vy).floor()),
            QuiltStyle::Star => {
                let t = self.stt;
                let (tx, ty) = (round_half_up(ax / t), round_half_up(ay / t));
                let (cx, cy) = (mx + tx * t, my + ty * t);
                let r = t * 0.46;
                if spills(cx, cy, r, r) {
                    return 0;
                }
                let (du, dv) = ((ax - tx * t).abs(), (ay - ty * t).abs());
                let p = odd(tx + ty);
                if du + dv <= r * 0.95 || du.max(dv) <= r * 0.55 {
                    if du + dv <= r * 0.4 && du.max(dv) <= r * 0.4 {
                        if p { 3 } else { 2 }
                    } else if p {
                        1
                    } else {
                        2
                    }
                } else {
                    0
                }
            }
            QuiltStyle::Waves => {
                let uu = (ux % self.wper + self.wper) % self.wper;
                let x = uu / self.wper * 2.0 - 1.0;
                let bump = (1.0 - x * x).max(0.0).sqrt() * self.wamp;
                lookup(&self.waves, (vy + bump).floor())
            }
            QuiltStyle::Gingham => {
                let hs = odd((vy / self.gg).floor());
                let vs = odd((ux / self.gg).floor());
                match (hs, vs) {
                    (true, true) => 2,
                    (false, false) => 0,
                    _ => 1,
                }
            }
            QuiltStyle::Burst => {
                if (ux * (gh / gw)).max(vy) < self.bcore {
                    return 3;
                }
                let angle = (vy + 0.5).atan2(ux + 0.5);
                let k = (angle / FRAC_PI_2 * self.bn).floor();
                [1, 0, 2, 0][(k as i64).rem_euclid(4) as usize]
            }
            QuiltStyle::Plaid => {
                let (uu, vv) = (ux % self.per, vy % self.per);
                let (hs, vs) = (vv < self.pw, uu < self.pw);
                if hs && vs {
                    2
                } else if hs || vs {
                    1
                } else {
                    let mid = self.pw + half(self.per - self.pw);
                    if uu.floor() == mid || vv.floor() == mid {
                        3
                    } else {
                        0
                    }
                }
            }
            QuiltStyle::Dither => {
                let s = 0.5
                    + 0.5
                        * ((ux + vy) * self.f1 + (ux * self.f2).sin() * self.warp + self.ph0).sin();
                let cell = ((vy.floor() as usize) & 3) * 4 + ((ux.floor() as usize) & 3);
                u8::from(s > (BAYER[cell] + 0.5) / 16.0)
            }
            QuiltStyle::Auto => unreachable!("Auto resolves to a style"),
            QuiltStyle::Steps => lookup(
                &self.steps,
                (vy + ((ux + self.off) / self.rw).floor() * self.rh).floor(),
            ),
        }
    }
}

fn paint(surface: &mut Surface, params: &QuiltParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let columns = params.cells;
    let rows = (round_half_up(f64::from(columns) * height / width) as u32).max(8);
    let pattern = pattern(params, tool_seed);
    let (gw, gh) = (f64::from(columns), f64::from(rows));
    let mid = ((gw - 1.0) / 2.0, (gh - 1.0) / 2.0);
    let mut rgba = Vec::with_capacity((columns * rows * 4) as usize);
    for v in 0..rows {
        for u in 0..columns {
            let role = pattern.role(f64::from(u), f64::from(v), mid, (gw, gh));
            let [r, g, b] = palette.ink(usize::from(role));
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, columns, rows, Smoothing::Nearest);
}
