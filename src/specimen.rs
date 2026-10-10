use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::terrain::{fbm, hash};
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "specimen";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 7] = [
    "#e63946", "#457b9d", "#f4a261", "#2a9d8f", "#e9c46a", "#8338ec", "#1b1b1b",
];
const PAGE: [u8; 3] = [0xf5, 0xf1, 0xe8];
const INK: [u8; 3] = [0x1b, 0x1b, 0x1b];

const COLS: Param = Param::new(SLUG, "cols", 2, 16, 1);
const FILL: Param = Param::new(SLUG, "fill", 10, 100, 100);
const SPAN: Param = Param::new(SLUG, "span", 0, 100, 100);
const GUT: Param = Param::new(SLUG, "gut", 0, 60, 200);
const RULES: Param = Param::new(SLUG, "rules", 0, 100, 100);
const W_MARBLE: Param = Param::new(SLUG, "wMarble", 0, 100, 1);
const W_TENDRIL: Param = Param::new(SLUG, "wTendril", 0, 100, 1);
const W_LOOP: Param = Param::new(SLUG, "wLoop", 0, 100, 1);
const W_FLAT: Param = Param::new(SLUG, "wFlat", 0, 100, 1);
const W_RING: Param = Param::new(SLUG, "wRing", 0, 100, 1);
const W_FAN: Param = Param::new(SLUG, "wFan", 0, 100, 1);
const W_ARC: Param = Param::new(SLUG, "wArc", 0, 100, 1);
const W_SPIKE: Param = Param::new(SLUG, "wSpike", 0, 100, 1);
const M_SCALE: Param = Param::new(SLUG, "mScale", 6, 60, 10);
const M_BANDS: Param = Param::new(SLUG, "mBands", 2, 7, 1);
const DENSITY: Param = Param::new(SLUG, "density", 10, 100, 100);
const LINE_W: Param = Param::new(SLUG, "lineW", 2, 50, 5);
const INK_MIX: Param = Param::new(SLUG, "inkMix", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    COLS, FILL, SPAN, GUT, RULES, W_MARBLE, W_TENDRIL, W_LOOP, W_FLAT, W_RING, W_FAN, W_ARC,
    W_SPIKE, M_SCALE, M_BANDS, DENSITY, LINE_W, INK_MIX,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpecimenParams {
    cols: u32,
    fill: f64,
    span: f64,
    gut: f64,
    rules: f64,
    #[serde(rename = "wMarble")]
    w_marble: u32,
    #[serde(rename = "wTendril")]
    w_tendril: u32,
    #[serde(rename = "wLoop")]
    w_loop: u32,
    #[serde(rename = "wFlat")]
    w_flat: u32,
    #[serde(rename = "wRing")]
    w_ring: u32,
    #[serde(rename = "wFan")]
    w_fan: u32,
    #[serde(rename = "wArc")]
    w_arc: u32,
    #[serde(rename = "wSpike")]
    w_spike: u32,
    #[serde(rename = "mScale")]
    m_scale: f64,
    #[serde(rename = "mBands")]
    m_bands: u32,
    density: f64,
    #[serde(rename = "lineW")]
    line_w: f64,
    #[serde(rename = "inkMix")]
    ink_mix: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for SpecimenParams {
    fn default() -> Self {
        Self {
            cols: 5,
            fill: 0.26,
            span: 0.45,
            gut: 0.0,
            rules: 0.14,
            w_marble: 26,
            w_tendril: 16,
            w_loop: 12,
            w_flat: 14,
            w_ring: 10,
            w_fan: 10,
            w_arc: 10,
            w_spike: 8,
            m_scale: 2.2,
            m_bands: 4,
            density: 0.55,
            line_w: 2.0,
            ink_mix: 0.35,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl SpecimenParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn fill(&self) -> f64 {
        self.fill
    }

    pub fn set_fill(&mut self, fill: f64) -> Result<(), Error> {
        self.fill = FILL.check(fill)?;
        Ok(())
    }

    pub fn span(&self) -> f64 {
        self.span
    }

    pub fn set_span(&mut self, span: f64) -> Result<(), Error> {
        self.span = SPAN.check(span)?;
        Ok(())
    }

    pub fn gut(&self) -> f64 {
        self.gut
    }

    pub fn set_gut(&mut self, gut: f64) -> Result<(), Error> {
        self.gut = GUT.check(gut)?;
        Ok(())
    }

    pub fn rules(&self) -> f64 {
        self.rules
    }

    pub fn set_rules(&mut self, rules: f64) -> Result<(), Error> {
        self.rules = RULES.check(rules)?;
        Ok(())
    }

    pub fn w_marble(&self) -> u32 {
        self.w_marble
    }

    pub fn set_w_marble(&mut self, w_marble: u32) -> Result<(), Error> {
        W_MARBLE.check(f64::from(w_marble))?;
        self.w_marble = w_marble;
        Ok(())
    }

    pub fn w_tendril(&self) -> u32 {
        self.w_tendril
    }

    pub fn set_w_tendril(&mut self, w_tendril: u32) -> Result<(), Error> {
        W_TENDRIL.check(f64::from(w_tendril))?;
        self.w_tendril = w_tendril;
        Ok(())
    }

    pub fn w_loop(&self) -> u32 {
        self.w_loop
    }

    pub fn set_w_loop(&mut self, w_loop: u32) -> Result<(), Error> {
        W_LOOP.check(f64::from(w_loop))?;
        self.w_loop = w_loop;
        Ok(())
    }

    pub fn w_flat(&self) -> u32 {
        self.w_flat
    }

    pub fn set_w_flat(&mut self, w_flat: u32) -> Result<(), Error> {
        W_FLAT.check(f64::from(w_flat))?;
        self.w_flat = w_flat;
        Ok(())
    }

    pub fn w_ring(&self) -> u32 {
        self.w_ring
    }

    pub fn set_w_ring(&mut self, w_ring: u32) -> Result<(), Error> {
        W_RING.check(f64::from(w_ring))?;
        self.w_ring = w_ring;
        Ok(())
    }

    pub fn w_fan(&self) -> u32 {
        self.w_fan
    }

    pub fn set_w_fan(&mut self, w_fan: u32) -> Result<(), Error> {
        W_FAN.check(f64::from(w_fan))?;
        self.w_fan = w_fan;
        Ok(())
    }

    pub fn w_arc(&self) -> u32 {
        self.w_arc
    }

    pub fn set_w_arc(&mut self, w_arc: u32) -> Result<(), Error> {
        W_ARC.check(f64::from(w_arc))?;
        self.w_arc = w_arc;
        Ok(())
    }

    pub fn w_spike(&self) -> u32 {
        self.w_spike
    }

    pub fn set_w_spike(&mut self, w_spike: u32) -> Result<(), Error> {
        W_SPIKE.check(f64::from(w_spike))?;
        self.w_spike = w_spike;
        Ok(())
    }

    pub fn m_scale(&self) -> f64 {
        self.m_scale
    }

    pub fn set_m_scale(&mut self, m_scale: f64) -> Result<(), Error> {
        self.m_scale = M_SCALE.check(m_scale)?;
        Ok(())
    }

    pub fn m_bands(&self) -> u32 {
        self.m_bands
    }

    pub fn set_m_bands(&mut self, m_bands: u32) -> Result<(), Error> {
        M_BANDS.check(f64::from(m_bands))?;
        self.m_bands = m_bands;
        Ok(())
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) -> Result<(), Error> {
        self.density = DENSITY.check(density)?;
        Ok(())
    }

    pub fn line_w(&self) -> f64 {
        self.line_w
    }

    pub fn set_line_w(&mut self, line_w: f64) -> Result<(), Error> {
        self.line_w = LINE_W.check(line_w)?;
        Ok(())
    }

    pub fn ink_mix(&self) -> f64 {
        self.ink_mix
    }

    pub fn set_ink_mix(&mut self, ink_mix: f64) -> Result<(), Error> {
        self.ink_mix = INK_MIX.check(ink_mix)?;
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
    fill: f64,
    span: f64,
    gut: f64,
    rules: f64,
    #[serde(rename = "wMarble")]
    w_marble: u32,
    #[serde(rename = "wTendril")]
    w_tendril: u32,
    #[serde(rename = "wLoop")]
    w_loop: u32,
    #[serde(rename = "wFlat")]
    w_flat: u32,
    #[serde(rename = "wRing")]
    w_ring: u32,
    #[serde(rename = "wFan")]
    w_fan: u32,
    #[serde(rename = "wArc")]
    w_arc: u32,
    #[serde(rename = "wSpike")]
    w_spike: u32,
    #[serde(rename = "mScale")]
    m_scale: f64,
    #[serde(rename = "mBands")]
    m_bands: u32,
    density: f64,
    #[serde(rename = "lineW")]
    line_w: f64,
    #[serde(rename = "inkMix")]
    ink_mix: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SpecimenParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SpecimenParams::default();
    params.set_cols(raw.cols)?;
    params.set_fill(raw.fill)?;
    params.set_span(raw.span)?;
    params.set_gut(raw.gut)?;
    params.set_rules(raw.rules)?;
    params.set_w_marble(raw.w_marble)?;
    params.set_w_tendril(raw.w_tendril)?;
    params.set_w_loop(raw.w_loop)?;
    params.set_w_flat(raw.w_flat)?;
    params.set_w_ring(raw.w_ring)?;
    params.set_w_fan(raw.w_fan)?;
    params.set_w_arc(raw.w_arc)?;
    params.set_w_spike(raw.w_spike)?;
    params.set_m_scale(raw.m_scale)?;
    params.set_m_bands(raw.m_bands)?;
    params.set_density(raw.density)?;
    params.set_line_w(raw.line_w)?;
    params.set_ink_mix(raw.ink_mix)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SpecimenParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SpecimenParams {
        cols: pick(&COLS) as u32,
        fill: pick(&FILL),
        span: pick(&SPAN),
        gut: pick(&GUT),
        rules: pick(&RULES),
        w_marble: pick(&W_MARBLE) as u32,
        w_tendril: pick(&W_TENDRIL) as u32,
        w_loop: pick(&W_LOOP) as u32,
        w_flat: pick(&W_FLAT) as u32,
        w_ring: pick(&W_RING) as u32,
        w_fan: pick(&W_FAN) as u32,
        w_arc: pick(&W_ARC) as u32,
        w_spike: pick(&W_SPIKE) as u32,
        m_scale: pick(&M_SCALE),
        m_bands: pick(&M_BANDS) as u32,
        density: pick(&DENSITY),
        line_w: pick(&LINE_W),
        ink_mix: pick(&INK_MIX),
        ..SpecimenParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &SpecimenParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Motif {
    Marble,
    Tendril,
    Loop,
    Flat,
    Ring,
    Fan,
    Arc,
    Spike,
}

struct Block {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    motif: Motif,
    ci: usize,
    ci2: usize,
    on_ink: bool,
    rot: f64,
    v: f64,
    v2: f64,
}

const SHAPES: [(usize, usize); 13] = [
    (2, 1),
    (1, 2),
    (2, 2),
    (2, 1),
    (1, 2),
    (2, 2),
    (3, 1),
    (1, 3),
    (2, 3),
    (3, 2),
    (3, 3),
    (4, 1),
    (1, 4),
];

fn blocks(
    p: &SpecimenParams,
    columns: usize,
    rows: usize,
    inks: usize,
    tool_seed: u32,
) -> Vec<Block> {
    let start = (f64::from(tool_seed) * 2_654_435_761.0 + 7.0).rem_euclid(4_294_967_296.0) as u32;
    let mut rng = Xorshift::from_state(start);
    let weights = [
        (Motif::Marble, p.w_marble),
        (Motif::Tendril, p.w_tendril),
        (Motif::Loop, p.w_loop),
        (Motif::Flat, p.w_flat),
        (Motif::Ring, p.w_ring),
        (Motif::Fan, p.w_fan),
        (Motif::Arc, p.w_arc),
        (Motif::Spike, p.w_spike),
    ];
    let total = match weights.iter().map(|&(_, w)| w).sum::<u32>() {
        0 => 1,
        total => total,
    };
    let mut taken = vec![false; columns * rows];
    let mut out = Vec::new();
    for y in 0..rows {
        for x in 0..columns {
            if taken[y * columns + x] {
                continue;
            }
            if rng.next() > p.fill {
                taken[y * columns + x] = true;
                continue;
            }
            let (mut w, mut h) = (1, 1);
            if rng.next() < p.span {
                let mut order = SHAPES.map(|shape| (shape, rng.next()));
                order.sort_by(|a, b| a.1.total_cmp(&b.1));
                let free = |(sw, sh): (usize, usize)| {
                    x + sw <= columns
                        && y + sh <= rows
                        && (y..y + sh).all(|j| (x..x + sw).all(|i| !taken[j * columns + i]))
                };
                if let Some(&(shape, _)) = order.iter().find(|(shape, _)| free(*shape)) {
                    (w, h) = shape;
                }
            }
            for j in y..y + h {
                for i in x..x + w {
                    taken[j * columns + i] = true;
                }
            }
            let mut left = rng.next() * f64::from(total);
            let motif = weights
                .iter()
                .find(|&&(_, weight)| {
                    left -= f64::from(weight);
                    left <= 0.0
                })
                .map_or(Motif::Flat, |&(motif, _)| motif);
            let mut index = || (rng.next() * inks as f64) as usize;
            let (ci, ci2) = (index(), index());
            let _ = (index(), index());
            let on_ink = rng.next() < p.ink_mix;
            rng.next();
            let rot = rng.next() * TAU;
            rng.next();
            let (v, v2) = (rng.next(), rng.next());
            out.push(Block {
                x,
                y,
                w,
                h,
                motif,
                ci,
                ci2,
                on_ink,
                rot,
                v,
                v2,
            });
        }
    }
    out
}

fn tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn contours(field: &[f32], nx: usize, ny: usize, level: f64) -> Vec<Vec<(f64, f64)>> {
    let get = |i: usize, j: usize| f64::from(field[j * (nx + 1) + i]);
    let mut segments = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            let (a, b, c, d) = (get(i, j), get(i + 1, j), get(i + 1, j + 1), get(i, j + 1));
            let case = (usize::from(a > level) << 3)
                | (usize::from(b > level) << 2)
                | (usize::from(c > level) << 1)
                | usize::from(d > level);
            if case == 0 || case == 15 {
                continue;
            }
            let (fi, fj) = (i as f64, j as f64);
            let cross = |x1: f64, y1: f64, v1: f64, x2: f64, y2: f64, v2: f64| {
                let diff = match v2 - v1 {
                    0.0 => 1e-9,
                    diff => diff,
                };
                let t = (level - v1) / diff;
                (x1 + (x2 - x1) * t, y1 + (y2 - y1) * t)
            };
            let top = || cross(fi, fj, a, fi + 1.0, fj, b);
            let right = || cross(fi + 1.0, fj, b, fi + 1.0, fj + 1.0, c);
            let bottom = || cross(fi, fj + 1.0, d, fi + 1.0, fj + 1.0, c);
            let left = || cross(fi, fj, a, fi, fj + 1.0, d);
            match case {
                1 | 14 => segments.push([left(), bottom()]),
                2 | 13 => segments.push([bottom(), right()]),
                3 | 12 => segments.push([left(), right()]),
                4 | 11 => segments.push([top(), right()]),
                6 | 9 => segments.push([top(), bottom()]),
                7 | 8 => segments.push([left(), top()]),
                5 => {
                    segments.push([left(), top()]);
                    segments.push([bottom(), right()]);
                }
                _ => {
                    segments.push([top(), right()]);
                    segments.push([left(), bottom()]);
                }
            }
        }
    }
    let key = |(x, y): (f64, f64)| format!("{x:.4},{y:.4}");
    let mut ends: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, segment) in segments.iter().enumerate() {
        for &point in segment {
            ends.entry(key(point)).or_default().push(index);
        }
    }
    let mut used = vec![false; segments.len()];
    let mut loops = Vec::new();
    for first in 0..segments.len() {
        if used[first] {
            continue;
        }
        used[first] = true;
        let mut chain = segments[first].to_vec();
        let head = key(chain[0]);
        loop {
            let tail = key(chain[chain.len() - 1]);
            let Some(&next) = ends[&tail].iter().find(|&&s| !used[s]) else {
                break;
            };
            used[next] = true;
            let [p0, p1] = segments[next];
            let point = if key(p0) == tail { p1 } else { p0 };
            chain.push(point);
            if key(point) == head {
                break;
            }
        }
        if chain.len() > 3 {
            loops.push(chain);
        }
    }
    loops
}

fn polyline(points: impl IntoIterator<Item = (f64, f64)>, k: f64) -> Path2D {
    let mut path = Path2D::default();
    for (q, (x, y)) in points.into_iter().enumerate() {
        if q == 0 {
            path.move_to(x * k, y * k);
        } else {
            path.line_to(x * k, y * k);
        }
    }
    path
}

fn paint(surface: &mut Surface, p: &SpecimenParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let k = w / 1000.0;
    let sheet = 1000.0 * h / w;
    let inks = palette.inks();
    let n = inks.len();
    let columns = p.cols as usize;
    let rows = (round_half_up(f64::from(p.cols) * sheet / 1000.0) as usize).max(1);
    let (cw, ch) = (1000.0 / columns as f64, sheet / rows as f64);
    let gutter = p.gut * cw.min(ch) * 0.5;
    let line = (p.line_w * k).max(0.3);
    let count = |base: f64, more: f64| round_half_up(base + p.density * more) as usize;
    surface.fill(PAGE);

    for (id, b) in blocks(p, columns, rows, n, tool_seed).iter().enumerate() {
        let id = id as i32;
        let (bx, by) = (b.x as f64 * cw + gutter, b.y as f64 * ch + gutter);
        let (bw, bh) = (
            b.w as f64 * cw - 2.0 * gutter,
            b.h as f64 * ch - 2.0 * gutter,
        );
        let col = |i: usize| inks[i % n];
        let ground = if b.on_ink { INK } else { col(b.ci) };
        let stroke = if b.on_ink { col(b.ci2) } else { INK };
        let (m, big) = (bw.min(bh), bw.max(bh));
        let (cx, cy) = (bx + bw / 2.0, by + bh / 2.0);
        let mut clip = polyline(
            [(bx, by), (bx + bw, by), (bx + bw, by + bh), (bx, by + bh)],
            k,
        );
        clip.close();
        surface.with_clip(&clip, |surface| {
            let base = match b.motif {
                Motif::Marble | Motif::Flat => col(b.ci),
                _ => ground,
            };
            surface.fill_box(bx * k, by * k, bw * k, bh * k, base);
            match b.motif {
                Motif::Flat => {}
                Motif::Marble => {
                    let nx = (round_half_up(28.0 * (bw / bh).sqrt()) as usize).max(8);
                    let ny = (round_half_up(28.0 * (bh / bw).sqrt()) as usize).max(8);
                    let pad = 0.16;
                    let seed = tool_seed.wrapping_add((id as u32).wrapping_mul(97));
                    let mut field = Vec::with_capacity((nx + 1) * (ny + 1));
                    for j in 0..=ny {
                        for i in 0..=nx {
                            let u = (i as f64 / nx as f64) * (1.0 + 2.0 * pad) - pad;
                            let v = (j as f64 / ny as f64) * (1.0 + 2.0 * pad) - pad;
                            field.push(fbm(
                                u * p.m_scale * (bw / m) + b.v * 40.0,
                                v * p.m_scale * (bh / m) + b.v2 * 40.0,
                                seed,
                                3,
                            ) as f32);
                        }
                    }
                    let lo = field.iter().fold(1.0_f32, |lo, &v| lo.min(v));
                    let hi = field.iter().fold(0.0_f32, |hi, &v| hi.max(v));
                    let (lo, hi) = (f64::from(lo), f64::from(hi));
                    for band in 1..p.m_bands {
                        let level = lo + (hi - lo) * (f64::from(band) / f64::from(p.m_bands));
                        let loops = contours(&field, nx, ny, level);
                        if loops.is_empty() {
                            continue;
                        }
                        let mut path = Path2D::default();
                        for chain in loops {
                            for (q, &(u, v)) in chain.iter().enumerate() {
                                let x =
                                    tenth(bx + ((u / nx as f64) * (1.0 + 2.0 * pad) - pad) * bw);
                                let y =
                                    tenth(by + ((v / ny as f64) * (1.0 + 2.0 * pad) - pad) * bh);
                                if q == 0 {
                                    path.move_to(x * k, y * k);
                                } else {
                                    path.line_to(x * k, y * k);
                                }
                            }
                            path.close();
                        }
                        surface.fill_even_odd(&path, col(b.ci + band as usize));
                    }
                }
                Motif::Tendril => {
                    let strands = count(8.0, 40.0);
                    let steps = count(30.0, 70.0);
                    let step = m / steps as f64 * 1.3;
                    let seed = tool_seed
                        .wrapping_add((id as u32).wrapping_mul(13))
                        .wrapping_add(5);
                    let width = (p.line_w * 0.45 * k).max(0.3);
                    for i in 0..strands as i32 {
                        let mut x = bx + hash(id, i, 0, 1, tool_seed) * bw;
                        let mut y = by + hash(id, i, 0, 2, tool_seed) * bh;
                        let mut points = vec![(x, y)];
                        for _ in 0..steps {
                            let noise = fbm(x / m * 7.0 + f64::from(i) * 3.0, y / m * 7.0, seed, 3);
                            let angle = (noise - 0.5) * 2.6 * TAU;
                            x += angle.cos() * step;
                            y += angle.sin() * step;
                            points.push((x, y));
                        }
                        surface.stroke(
                            &polyline(points, k),
                            stroke,
                            width,
                            Cap::Round,
                            Join::Round,
                        );
                    }
                }
                Motif::Loop => {
                    for i in 0..count(2.0, 6.0) as i32 {
                        let rr = m * (0.18 + 0.42 * hash(id, i, 3, 0, tool_seed));
                        let rot = b.rot + f64::from(i) * 0.7;
                        let squash = 0.35 + 0.6 * hash(id, i, 4, 0, tool_seed);
                        let (cr, sr) = (rot.cos(), rot.sin());
                        let path = polyline(
                            (0..=64).map(|q| {
                                let a = f64::from(q) / 64.0 * TAU;
                                let (x, y) = (a.cos() * rr, a.sin() * rr * squash);
                                (cx + x * cr - y * sr, cy + x * sr + y * cr)
                            }),
                            k,
                        );
                        surface.stroke(&path, stroke, line, Cap::Round, Join::Round);
                    }
                }
                Motif::Ring => {
                    let rings = count(3.0, 12.0);
                    let rmax = m * 0.46;
                    for i in 0..rings {
                        let q = (i + 1) as f64 / rings as f64;
                        let rot = (b.rot + i as f64 * 0.12).rem_euclid(TAU);
                        let mut path = Path2D::default();
                        path.ellipse(
                            cx * k,
                            cy * k,
                            (rmax * q * k).max(0.1),
                            (rmax * q * (0.35 + 0.65 * b.v) * k).max(0.1),
                            rot,
                            0.0,
                            TAU,
                        );
                        surface.stroke(&path, stroke, line, Cap::Butt, Join::Miter);
                    }
                }
                Motif::Fan => {
                    let blades = count(3.0, 10.0);
                    let (px, py) = (
                        bx + bw * (0.5 + 0.4 * (b.v - 0.5)),
                        by + bh * (0.5 + 0.4 * (b.v2 - 0.5)),
                    );
                    let len = big * 0.62;
                    let wd = m * 0.09;
                    let spread = PI * (0.4 + 0.5 * b.v);
                    for i in 0..blades {
                        let share = if blades < 2 {
                            0.5
                        } else {
                            i as f64 / (blades - 1) as f64
                        };
                        let a = b.rot - spread / 2.0 + spread * share;
                        let (tx, ty) = (px + a.cos() * len, py + a.sin() * len);
                        let (nx, ny) = (-a.sin() * wd / 2.0, a.cos() * wd / 2.0);
                        let mut path = polyline(
                            [
                                (px + nx * 0.4, py + ny * 0.4),
                                (tx + nx, ty + ny),
                                (tx - nx, ty - ny),
                                (px - nx * 0.4, py - ny * 0.4),
                            ],
                            k,
                        );
                        path.close();
                        surface.fill_path(&path.into_path(), col(b.ci2 + i));
                    }
                }
                Motif::Arc => {
                    let arcs = count(3.0, 12.0);
                    let (px, py) = (cx, by + bh * 1.05);
                    let rmax = big * 1.15;
                    for i in 0..arcs {
                        let rr = rmax * (0.25 + 0.75 * (i + 1) as f64 / arcs as f64);
                        let path = polyline(
                            (0..=40).map(|q| {
                                let a = PI + f64::from(q) / 40.0 * PI;
                                (px + a.cos() * rr, py + a.sin() * rr * 0.9)
                            }),
                            k,
                        );
                        surface.stroke(&path, stroke, line, Cap::Round, Join::Round);
                    }
                }
                Motif::Spike => {
                    let wd = m * 0.045;
                    for i in 0..count(3.0, 14.0) as i32 {
                        let (ax, ay) = (
                            bx + hash(id, i, 7, 0, tool_seed) * bw,
                            by + hash(id, i, 8, 0, tool_seed) * bh,
                        );
                        let a = b.rot + hash(id, i, 9, 0, tool_seed) * TAU;
                        let len = m * (0.3 + 0.6 * hash(id, i, 10, 0, tool_seed));
                        let (nx, ny) = (-a.sin() * wd / 2.0, a.cos() * wd / 2.0);
                        let mut path = polyline(
                            [
                                (ax + nx, ay + ny),
                                (ax + a.cos() * len, ay + a.sin() * len),
                                (ax - nx, ay - ny),
                            ],
                            k,
                        );
                        path.close();
                        surface.fill_path(&path.into_path(), stroke);
                    }
                }
            }
        });
    }

    if p.rules > 0.0 {
        let mut path = Path2D::default();
        for i in 1..columns {
            let x = tenth(i as f64 * cw) * k;
            path.move_to(x, 0.0);
            path.line_to(x, tenth(sheet) * k);
        }
        for j in 1..rows {
            let y = tenth(j as f64 * ch) * k;
            path.move_to(0.0, y);
            path.line_to(w, y);
        }
        surface.with_alpha(p.rules as f32, |surface| {
            surface.stroke(&path, INK, k.max(0.5), Cap::Butt, Join::Miter);
        });
    }
}
