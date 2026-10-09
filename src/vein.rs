use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, value_noise,
};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "vein";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 8] = [
    "#0e1a2b", "#2e63b8", "#7fa8e0", "#e9dcc3", "#f2892b", "#e0362f", "#2fa39a", "#8b5a2b",
];

const SCALE: Param = Param {
    taste: (25, 75),
    ..Param::new(SLUG, "scale", 0, 100, 100)
};
const CURVE: Param = Param::new(SLUG, "curve", 0, 100, 100);
const LEVELS: Param = Param {
    taste: (9, 11),
    ..Param::new(SLUG, "levels", 3, 14, 1)
};
const TIGER: Param = Param::new(SLUG, "tiger", 0, 100, 100);
const EDGES: Param = Param {
    taste: (37, 75),
    ..Param::new(SLUG, "edges", 0, 100, 100)
};
const STARS: Param = Param::new(SLUG, "stars", 0, 100, 100);
const TINTS: Param = Param::new(SLUG, "tints", 0, 100, 100);
const RUNS: Param = Param {
    taste: (50, 75),
    ..Param::new(SLUG, "runs", 0, 100, 100)
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flow {
    Marble,
    Swirl,
    Ripple,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VeinParams {
    #[serde(rename = "flows")]
    flow: Flow,
    scale: f64,
    curve: f64,
    levels: u32,
    tiger: f64,
    edges: f64,
    stars: f64,
    tints: f64,
    runs: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for VeinParams {
    fn default() -> Self {
        Self {
            flow: Flow::Marble,
            scale: 0.5,
            curve: 0.6,
            levels: 9,
            tiger: 0.5,
            edges: 0.4,
            stars: 0.5,
            tints: 0.5,
            runs: 0.5,
            dither: Dither::default(),
            grain: Grain::printed(),
        }
    }
}

impl VeinParams {
    pub fn flow(&self) -> Flow {
        self.flow
    }

    pub fn set_flow(&mut self, flow: Flow) {
        self.flow = flow;
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn curve(&self) -> f64 {
        self.curve
    }

    pub fn set_curve(&mut self, curve: f64) -> Result<(), Error> {
        self.curve = CURVE.check(curve)?;
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

    pub fn tiger(&self) -> f64 {
        self.tiger
    }

    pub fn set_tiger(&mut self, tiger: f64) -> Result<(), Error> {
        self.tiger = TIGER.check(tiger)?;
        Ok(())
    }

    pub fn edges(&self) -> f64 {
        self.edges
    }

    pub fn set_edges(&mut self, edges: f64) -> Result<(), Error> {
        self.edges = EDGES.check(edges)?;
        Ok(())
    }

    pub fn stars(&self) -> f64 {
        self.stars
    }

    pub fn set_stars(&mut self, stars: f64) -> Result<(), Error> {
        self.stars = STARS.check(stars)?;
        Ok(())
    }

    pub fn tints(&self) -> f64 {
        self.tints
    }

    pub fn set_tints(&mut self, tints: f64) -> Result<(), Error> {
        self.tints = TINTS.check(tints)?;
        Ok(())
    }

    pub fn runs(&self) -> f64 {
        self.runs
    }

    pub fn set_runs(&mut self, runs: f64) -> Result<(), Error> {
        self.runs = RUNS.check(runs)?;
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
    flows: Flow,
    scale: f64,
    curve: f64,
    levels: u32,
    tiger: f64,
    edges: f64,
    stars: f64,
    tints: f64,
    runs: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<VeinParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = VeinParams::default();
    params.set_flow(raw.flows);
    params.set_scale(raw.scale)?;
    params.set_curve(raw.curve)?;
    params.set_levels(raw.levels)?;
    params.set_tiger(raw.tiger)?;
    params.set_edges(raw.edges)?;
    params.set_stars(raw.stars)?;
    params.set_tints(raw.tints)?;
    params.set_runs(raw.runs)?;
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

pub(crate) const PARAMS: &[Param] = &[SCALE, CURVE, LEVELS, TIGER, EDGES, STARS, TINTS, RUNS];

pub(crate) fn parameters() -> Vec<Parameter> {
    std::iter::once(Parameter::choice("flows", &["Marble", "Swirl", "Ripple"]))
        .chain(PARAMS.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> VeinParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    const FLOWS: [Flow; 3] = [Flow::Marble, Flow::Swirl, Flow::Ripple];
    VeinParams {
        flow: FLOWS[(draw("flows") % 3) as usize],
        scale: pick(&SCALE),
        curve: pick(&CURVE),
        levels: pick(&LEVELS) as u32,
        tiger: pick(&TIGER),
        edges: pick(&EDGES),
        stars: pick(&STARS),
        tints: pick(&TINTS),
        runs: pick(&RUNS),
        ..VeinParams::default()
    }
}

fn tint(ink: [u8; 3], t: f64) -> [u8; 3] {
    ink.map(|c| {
        let c = f64::from(c);
        let moved = if t >= 0.0 {
            c + (255.0 - c) * t.min(1.0)
        } else {
            c * (1.0 - (-t).min(1.0) * 0.9)
        };
        round_half_up(moved).clamp(0.0, 255.0) as u8
    })
}

fn luma([r, g, b]: [u8; 3]) -> f64 {
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}

struct Grid {
    values: Vec<f32>,
    width: usize,
    height: usize,
    cell: f64,
}

impl Grid {
    fn at_pixel(&self, x: f64, y: f64) -> f32 {
        let node = |p: f64, len: usize| {
            (round_half_up(p / self.cell) + 1.0).clamp(1.0, (len - 2) as f64) as usize
        };
        self.values[node(y, self.height) * self.width + node(x, self.width)]
    }

    fn node(&self, i: usize, j: usize) -> f64 {
        f64::from(self.values[j * self.width + i])
    }

    fn cut(&self, level: f64) -> Path2D {
        let edges = self.width * self.height * 2;
        let mut points = vec![(0.0, 0.0); edges];
        let mut links = vec![[usize::MAX; 2]; edges];
        let mut order = Vec::new();
        let mut link = |a: usize, b: usize| {
            for (from, to) in [(a, b), (b, a)] {
                let slot = &mut links[from];
                if slot[0] == usize::MAX {
                    slot[0] = to;
                    order.push(from);
                } else {
                    slot[1] = to;
                }
            }
        };
        let cell = self.cell;
        let along = |a: f64, b: f64| {
            let span = b - a;
            (level - a) / if span == 0.0 { 1e-9 } else { span }
        };
        let place = |i: f64, j: f64| (i * cell - cell, j * cell - cell);
        for j in 0..self.height - 1 {
            for i in 0..self.width - 1 {
                let (a, b) = (self.node(i, j), self.node(i + 1, j));
                let (c, d) = (self.node(i + 1, j + 1), self.node(i, j + 1));
                let code = [a, b, c, d]
                    .iter()
                    .fold(0, |code, &v| code << 1 | u8::from(v >= level));
                if code == 0 || code == 15 {
                    continue;
                }
                let edge = |i: usize, j: usize, side: usize| (j * self.width + i) * 2 + side;
                let (fi, fj) = (i as f64, j as f64);
                let top = edge(i, j, 0);
                let right = edge(i + 1, j, 1);
                let bottom = edge(i, j + 1, 0);
                let left = edge(i, j, 1);
                points[top] = place(fi + along(a, b), fj);
                points[right] = place(fi + 1.0, fj + along(b, c));
                points[bottom] = place(fi + along(d, c), fj + 1.0);
                points[left] = place(fi, fj + along(a, d));
                let middle_inside = (a + b + c + d) * 0.25 >= level;
                let pairs: &[(usize, usize)] = match code {
                    1 | 14 => &[(left, bottom)],
                    2 | 13 => &[(bottom, right)],
                    3 | 12 => &[(left, right)],
                    4 | 11 => &[(top, right)],
                    6 | 9 => &[(top, bottom)],
                    7 | 8 => &[(top, left)],
                    5 if middle_inside => &[(top, left), (bottom, right)],
                    5 => &[(top, right), (left, bottom)],
                    _ if middle_inside => &[(top, right), (left, bottom)],
                    _ => &[(top, left), (bottom, right)],
                };
                for &(from, to) in pairs {
                    link(from, to);
                }
            }
        }

        let mut path = Path2D::default();
        let mut seen = vec![false; edges];
        let tenth = |v: f64| round_half_up(v * 10.0) / 10.0;
        for &start in &order {
            if seen[start] {
                continue;
            }
            let mut ring = Vec::new();
            let (mut current, mut previous) = (start, usize::MAX);
            while !seen[current] {
                seen[current] = true;
                ring.push(points[current]);
                let [first, second] = links[current];
                let next = if first != previous { first } else { second };
                previous = current;
                current = next;
                if current == start || current == usize::MAX {
                    break;
                }
            }
            let n = ring.len();
            if n < 3 {
                continue;
            }
            let mid = |k: usize| {
                let ((px, py), (qx, qy)) = (ring[k % n], ring[(k + 1) % n]);
                ((px + qx) * 0.5, (py + qy) * 0.5)
            };
            let (sx, sy) = mid(0);
            path.move_to(tenth(sx), tenth(sy));
            for k in 1..=n {
                let ((px, py), (mx, my)) = (ring[k % n], mid(k));
                path.quad_to(tenth(px), tenth(py), tenth(mx), tenth(my));
            }
            path.close();
        }
        path
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &VeinParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &VeinParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let unit = (width * height).sqrt();
    let inks = palette.len();
    let field_seed = |offset: u32| tool_seed.wrapping_mul(7).wrapping_add(offset);
    let (warp_seed, detail_seed, mask_seed) = (field_seed(1), field_seed(2), field_seed(3));

    let mut stream = XorShift::new(tool_seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let angle = stream.next() * TAU;
    let (cos_a, sin_a) = (angle.cos(), angle.sin());
    let centre_x = (0.3 + stream.next() * 0.4) * width / unit;
    let centre_y = (0.3 + stream.next() * 0.4) * height / unit;
    let frequency = 0.6 + params.scale * 1.6;
    let curve = params.curve;

    let field = |u: f64, v: f64| {
        let wu =
            u + curve * 0.55 * (value_noise(u * 1.3 + 3.0, v * 1.3 + 1.0, warp_seed) - 0.5) * 2.0;
        let wv =
            v + curve * 0.55 * (value_noise(u * 1.3 + 8.0, v * 1.3 + 5.0, warp_seed) - 0.5) * 2.0;
        let detail = |k: f64, x: f64, y: f64| {
            value_noise(wu * k * frequency + x, wv * k * frequency + y, detail_seed)
        };
        let n = 0.6 * detail(2.4, 7.0, 2.0)
            + 0.3 * detail(5.0, 3.0, 9.0)
            + 0.1 * detail(11.0, 4.0, 6.0);
        let stripes = |phase: f64| 0.5 + 0.5 * (phase * PI).sin();
        let mut value = match params.flow {
            Flow::Marble => {
                let p = wu * cos_a + wv * sin_a;
                stripes(p * 1.6 * frequency + 2.4 * (n - 0.5))
            }
            Flow::Swirl => {
                let (dx, dy) = (u - centre_x, v - centre_y);
                let d = (dx * dx + dy * dy).sqrt();
                let turn = curve * 4.0 * (-d * d / 0.16).exp();
                let (cos_t, sin_t) = (turn.cos(), turn.sin());
                let rx = centre_x + dx * cos_t - dy * sin_t;
                let ry = centre_y + dx * sin_t + dy * cos_t;
                let p = rx * cos_a + ry * sin_a;
                stripes(p * 1.6 * frequency + 2.0 * (n - 0.5))
            }
            Flow::Ripple => {
                let p = wv * 0.7 + wu * 0.15;
                stripes(p * 2.6 * frequency + 1.4 * (n - 0.5))
            }
        };
        if params.tiger > 0.0 {
            let mask = value_noise(u * 1.5 + 9.0, v * 1.5 + 4.0, mask_seed);
            let k = ((mask - 0.45) / 0.25).clamp(0.0, 1.0);
            value += params.tiger * k * 0.09 * (value * TAU * 5.0).sin();
        }
        value
    };

    let cell = unit / 150.0;
    let grid_width = (width / cell).ceil() as usize + 3;
    let grid_height = (height / cell).ceil() as usize + 3;
    let mut values = vec![-1.0_f32; grid_width * grid_height];
    for j in 1..grid_height - 1 {
        for i in 1..grid_width - 1 {
            let at = |n: usize| (n - 1) as f64 * cell / unit;
            values[j * grid_width + i] = field(at(i), at(j)) as f32;
        }
    }
    let grid = Grid {
        values,
        width: grid_width,
        height: grid_height,
        cell,
    };

    let levels = params.levels;
    let dark = round_half_up(f64::from(levels) * 0.3) as u32;
    let run_length = 1.0 + params.runs * 3.0;
    let (mut ink, mut run_end) = (1 % inks, 0);
    surface.fill(palette.ink(0));
    for k in 0..levels {
        let roll = |key: i32| hash(k as i32, key, tool_seed);
        let (fill, outline) = if k < dark {
            (
                tint(palette.ink(0), if k % 2 == 1 { 0.12 } else { -0.2 }),
                None,
            )
        } else {
            if k >= run_end {
                let mut next = 1 + (roll(4) * (inks - 1) as f64) as usize;
                if next == ink && inks > 2 {
                    next = 1 + next % (inks - 1);
                }
                ink = next;
                run_end = k + 1 + (roll(5) * run_length) as u32;
            }
            let shift = ((roll(6) - 0.5) * 2.0) * params.tints * 0.45;
            let outline = (roll(7) < params.edges).then(|| {
                let other = 1 + (ink + 1 + (roll(8) * (inks - 2) as f64) as usize) % (inks - 1);
                tint(palette.ink(other), 0.25)
            });
            (tint(palette.ink(ink), shift), outline)
        };
        let path = grid.cut((f64::from(k) + 0.5) / f64::from(levels));
        if path.is_empty() {
            continue;
        }
        surface.fill_even_odd(&path, fill);
        if let Some(outline) = outline {
            surface.stroke(&path, outline, (unit * 0.0035).max(0.6));
        }
    }

    let light = (1..inks).fold(1, |light, k| {
        if luma(palette.ink(k)) > luma(palette.ink(light)) {
            k
        } else {
            light
        }
    });
    let ground = f64::from(dark) / f64::from(levels);
    for k in 0..round_half_up(params.stars * 600.0) as i32 {
        let roll = |key: i32| hash(k, key, tool_seed);
        let (x, y) = (roll(11) * width, roll(12) * height);
        if f64::from(grid.at_pixel(x, y)) >= ground {
            continue;
        }
        let star = if roll(13) < 0.7 {
            light
        } else {
            1 + (roll(14) * (inks - 1) as f64) as usize
        };
        let radius = unit * (0.0012 + roll(15) * 0.002);
        surface.fill_circle(x, y, radius, tint(palette.ink(star), 0.4));
    }
}
