use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, value_noise};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "sprig";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 2] = ["#f9e4c8", "#2d6a4f"];

const COUNT: Param = Param {
    taste: (20, 100),
    ..Param::new(SLUG, "count", 2, 120, 1)
};
const SIZE: Param = Param {
    taste: (0, 60),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const VARY: Param = Param::new(SLUG, "vary", 0, 100, 100);
const SOLIDS: Param = Param::new(SLUG, "solids", 0, 100, 100);
const WEIGHT: Param = Param {
    taste: (0, 80),
    ..Param::new(SLUG, "weight", 0, 100, 100)
};
const ROUGH: Param = Param::new(SLUG, "rough", 0, 100, 100);
const WOBBLE: Param = Param::new(SLUG, "wobble", 0, 100, 100);
const DETAIL: Param = Param::new(SLUG, "detail", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[COUNT, SIZE, VARY, SOLIDS, WEIGHT, ROUGH, WOBBLE, DETAIL];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotifSet {
    Garden,
    Blooms,
    Leaves,
}

impl MotifSet {
    pub const ALL: &[MotifSet] = &[Self::Garden, Self::Blooms, Self::Leaves];

    fn kinds(self) -> &'static [Kind] {
        use Kind::*;
        match self {
            Self::Garden => &[
                Bloom, Leaf, Leaf, Petal, Daisy, Dot, Stem, Bloom, Leaf, Bloom,
            ],
            Self::Blooms => &[Bloom, Bloom, Petal, Daisy, Dot],
            Self::Leaves => &[Leaf, Leaf, Leaf, Leaf, Stem, Petal],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SprigParams {
    dirs: MotifSet,
    count: u32,
    size: f64,
    vary: f64,
    solids: f64,
    weight: f64,
    rough: f64,
    wobble: f64,
    detail: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for SprigParams {
    fn default() -> Self {
        Self {
            dirs: MotifSet::Garden,
            count: 42,
            size: 0.62,
            vary: 0.5,
            solids: 0.06,
            weight: 0.5,
            rough: 0.45,
            wobble: 0.5,
            detail: 0.55,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl SprigParams {
    pub fn dirs(&self) -> MotifSet {
        self.dirs
    }

    pub fn set_dirs(&mut self, dirs: MotifSet) {
        self.dirs = dirs;
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn vary(&self) -> f64 {
        self.vary
    }

    pub fn set_vary(&mut self, vary: f64) -> Result<(), Error> {
        self.vary = VARY.check(vary)?;
        Ok(())
    }

    pub fn solids(&self) -> f64 {
        self.solids
    }

    pub fn set_solids(&mut self, solids: f64) -> Result<(), Error> {
        self.solids = SOLIDS.check(solids)?;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
        Ok(())
    }

    pub fn rough(&self) -> f64 {
        self.rough
    }

    pub fn set_rough(&mut self, rough: f64) -> Result<(), Error> {
        self.rough = ROUGH.check(rough)?;
        Ok(())
    }

    pub fn wobble(&self) -> f64 {
        self.wobble
    }

    pub fn set_wobble(&mut self, wobble: f64) -> Result<(), Error> {
        self.wobble = WOBBLE.check(wobble)?;
        Ok(())
    }

    pub fn detail(&self) -> f64 {
        self.detail
    }

    pub fn set_detail(&mut self, detail: f64) -> Result<(), Error> {
        self.detail = DETAIL.check(detail)?;
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
    dirs: MotifSet,
    count: u32,
    size: f64,
    vary: f64,
    solids: f64,
    weight: f64,
    rough: f64,
    wobble: f64,
    detail: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SprigParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SprigParams::default();
    params.set_dirs(raw.dirs);
    params.set_count(raw.count)?;
    params.set_size(raw.size)?;
    params.set_vary(raw.vary)?;
    params.set_solids(raw.solids)?;
    params.set_weight(raw.weight)?;
    params.set_rough(raw.rough)?;
    params.set_wobble(raw.wobble)?;
    params.set_detail(raw.detail)?;
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
    [Parameter::choice("dirs", &["Garden", "Blooms", "Leaves"])]
        .into_iter()
        .chain(PARAMS.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SprigParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SprigParams {
        dirs: MotifSet::ALL[(draw("dirs") % MotifSet::ALL.len() as u64) as usize],
        count: pick(&COUNT) as u32,
        size: pick(&SIZE),
        vary: pick(&VARY),
        solids: pick(&SOLIDS),
        weight: pick(&WEIGHT),
        rough: pick(&ROUGH),
        wobble: pick(&WOBBLE),
        detail: pick(&DETAIL),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bloom,
    Leaf,
    Petal,
    Daisy,
    Dot,
    Stem,
}

struct Shape {
    points: Vec<(f64, f64)>,
    closed: bool,
    filled: bool,
}

impl Shape {
    fn ring(points: Vec<(f64, f64)>) -> Self {
        Self {
            points,
            closed: true,
            filled: false,
        }
    }

    fn line(points: Vec<(f64, f64)>) -> Self {
        Self {
            points,
            closed: false,
            filled: false,
        }
    }

    fn filled(points: Vec<(f64, f64)>) -> Self {
        Self {
            points,
            closed: true,
            filled: true,
        }
    }
}

struct Motif {
    x: f64,
    y: f64,
    angle: f64,
    k: f64,
    solid: bool,
    shapes: Vec<Shape>,
    salt: u32,
}

fn salt(rng: &mut XorShift) -> u32 {
    (rng.next() * 9973.0).floor() as u32
}

fn ring(n: u32, rx: f64, ry: f64, pointy: f64, wobble: f64, salt: u32) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let theta = f64::from(i) / f64::from(n) * TAU;
            let (s, c) = theta.sin_cos();
            let r = 1.0 + (value_noise(c * 1.9 + 3.1, s * 1.9 + 7.7, salt) - 0.5) * 0.5 * wobble;
            let y = if pointy > 0.0 {
                (if s < 0.0 { -1.0 } else { 1.0 }) * s.abs().powf(1.0 + pointy * 0.9)
            } else {
                s
            };
            (c * rx * r, y * ry * r)
        })
        .collect()
}

fn shapes(kind: Kind, rng: &mut XorShift, wobble: f64, detail: f64) -> Vec<Shape> {
    let sd = salt(rng);
    match kind {
        Kind::Bloom => {
            let mut shapes = vec![Shape::ring(ring(
                40,
                1.0,
                0.78 + rng.next() * 0.3,
                0.0,
                wobble,
                sd,
            ))];
            if rng.next() < detail {
                shapes.push(Shape::filled(ring(16, 0.16, 0.11, 0.0, wobble, sd + 7)));
            }
            shapes
        }
        Kind::Leaf => {
            let ry = 0.30 + rng.next() * 0.22;
            let mut shapes = vec![Shape::ring(ring(38, 1.0, ry, 1.0, wobble, sd))];
            let d = rng.next();
            if d < detail * 0.5 {
                let vein = (0..=10)
                    .map(|i| {
                        let t = -0.72 + 1.44 * f64::from(i) / 10.0;
                        let y = (value_noise(t * 2.2 + f64::from(sd) * 0.3, 1.7, sd + 3) - 0.5)
                            * ry
                            * 0.5
                            * wobble;
                        (t, y)
                    })
                    .collect();
                shapes.push(Shape::line(vein));
            } else if d < detail {
                let m = 3 + (rng.next() * 4.0).floor() as u32;
                for j in 0..m {
                    let t = -0.62 + 1.24 * (f64::from(j) + 0.5) / f64::from(m);
                    let h = ry * (1.0 - t.abs().powf(1.9)) * 0.82;
                    shapes.push(Shape::line(vec![(t - 0.06, -h), (t + 0.06, h)]));
                }
            }
            shapes
        }
        Kind::Petal => vec![Shape::ring(ring(26, 0.55, 0.22, 1.0, wobble, sd))],
        Kind::Daisy => {
            let m = 5 + (rng.next() * 4.0).floor() as u32;
            let mut shapes: Vec<Shape> = (0..m)
                .map(|j| {
                    let a = f64::from(j) / f64::from(m) * TAU + rng.next() * 0.2;
                    let (sa, ca) = a.sin_cos();
                    let petal = ring(22, 0.62, 0.2, 1.0, wobble, sd + 13 * j)
                        .into_iter()
                        .map(|(x, y)| {
                            let x = 0.62 + x;
                            (x * ca - y * sa, x * sa + y * ca)
                        })
                        .collect();
                    Shape::ring(petal)
                })
                .collect();
            shapes.push(Shape::ring(ring(18, 0.2, 0.16, 0.0, wobble, sd + 91)));
            shapes
        }
        Kind::Dot => vec![Shape::filled(ring(24, 0.42, 0.40, 0.0, wobble, sd))],
        Kind::Stem => {
            let mut heading = rng.next() * TAU;
            let (mut x, mut y) = (0.0, 0.0);
            let stem = (0..15)
                .map(|i| {
                    let point = (x, y);
                    heading += (value_noise(f64::from(i) * 0.31 + f64::from(sd) * 0.2, 4.4, sd)
                        - 0.5)
                        * 0.9
                        * wobble;
                    x += heading.cos() * 0.13;
                    y += heading.sin() * 0.13;
                    point
                })
                .collect();
            vec![Shape::line(stem)]
        }
    }
}

fn motifs(params: &SprigParams, seed: u32, fw: f64, fh: f64) -> Vec<Motif> {
    let mut rng = XorShift::new(seed.wrapping_mul(2_654_435_761));
    let kinds = params.dirs.kinds();
    let mut motifs: Vec<Motif> = Vec::new();
    for _ in 0..params.count.max(1) {
        let (mut bx, mut by, mut best) = (0.0, 0.0, -1.0);
        for _ in 0..12 {
            let cx = rng.next() * fw;
            let cy = rng.next() * fh;
            let near = motifs.iter().fold(1e9, |near: f64, m| {
                let mut dx = (m.x - cx).abs();
                let mut dy = (m.y - cy).abs();
                if dx > fw * 0.5 {
                    dx = fw - dx;
                }
                if dy > fh * 0.5 {
                    dy = fh - dy;
                }
                near.min(dx * dx + dy * dy)
            });
            if near > best {
                (best, bx, by) = (near, cx, cy);
            }
        }
        let kind = kinds[(rng.next() * kinds.len() as f64).floor() as usize];
        let u = rng.next();
        let angle = rng.next() * TAU;
        let solid = kind == Kind::Dot || rng.next() < params.solids;
        let shapes = shapes(kind, &mut rng, params.wobble, params.detail);
        motifs.push(Motif {
            x: bx,
            y: by,
            angle,
            k: 1.0 + (u * u * 2.2 - 0.45) * params.vary,
            solid,
            shapes,
            salt: salt(&mut rng),
        });
    }
    motifs
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &SprigParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &SprigParams, palette: &Palette, seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let aspect = height / width;
    let (fw, fh) = (1.0 / aspect.sqrt(), aspect.sqrt());
    let unit = (width * height).sqrt();
    let (ground, line) = (palette.ink(0), palette.ink(1));
    surface.fill(ground);
    let base = 0.035 + params.size * 0.11;
    let width_px = unit * (0.004 + params.weight * 0.020);
    for motif in motifs(params, seed, fw, fh) {
        let scale = base * motif.k.max(0.18) * unit;
        let (sa, ca) = motif.angle.sin_cos();
        for gx in -1..=1 {
            for gy in -1..=1 {
                let px = (motif.x + f64::from(gx) * fw) * unit;
                let py = (motif.y + f64::from(gy) * fh) * unit;
                if px < -scale * 2.0
                    || px > width + scale * 2.0
                    || py < -scale * 2.0
                    || py > height + scale * 2.0
                {
                    continue;
                }
                for shape in &motif.shapes {
                    let points: Vec<(f64, f64)> = shape
                        .points
                        .iter()
                        .map(|&(x, y)| {
                            (
                                px + (x * ca - y * sa) * scale,
                                py + (x * sa + y * ca) * scale,
                            )
                        })
                        .collect();
                    if shape.filled || (shape.closed && motif.solid) {
                        surface.fill_path(&polygon(&points).into_path(), line);
                    } else {
                        ink(
                            surface,
                            &points,
                            shape.closed,
                            width_px,
                            params.rough,
                            motif.salt,
                            line,
                        );
                    }
                }
            }
        }
    }
}

fn polygon(points: &[(f64, f64)]) -> Path2D {
    let mut path = Path2D::default();
    path.move_to(points[0].0, points[0].1);
    for &(x, y) in &points[1..] {
        path.line_to(x, y);
    }
    path.close();
    path
}

fn ink(
    surface: &mut Surface,
    points: &[(f64, f64)],
    closed: bool,
    width: f64,
    rough: f64,
    salt: u32,
    colour: [u8; 3],
) {
    let n = points.len();
    let (mut left, mut right) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for (i, &(x, y)) in points.iter().enumerate() {
        let (dx, dy) = if !closed && i == 0 {
            (points[1].0 - points[0].0, points[1].1 - points[0].1)
        } else if !closed && i == n - 1 {
            (
                points[n - 1].0 - points[n - 2].0,
                points[n - 1].1 - points[n - 2].1,
            )
        } else {
            let (a, b) = (points[(i + n - 1) % n], points[(i + 1) % n]);
            (b.0 - a.0, b.1 - a.1)
        };
        let length = (dx * dx + dy * dy).sqrt();
        let length = if length == 0.0 { 1.0 } else { length };
        let (nx, ny) = (-dy / length, dx / length);
        let theta = i as f64 / n as f64 * TAU;
        let q = f64::from(salt);
        let mut h = width
            * 0.5
            * (1.0
                + (value_noise(
                    theta.cos() * 2.6 + q * 0.11,
                    theta.sin() * 2.6 + q * 0.07,
                    salt,
                ) - 0.5)
                    * rough
                    * 1.25);
        if !closed {
            let t = i as f64 / (n - 1) as f64;
            let e = t.min(1.0 - t) * 7.0 + 0.3;
            if e < 1.0 {
                h *= e;
            }
        }
        let h = h.max(0.2);
        left.push((x + nx * h, y + ny * h));
        right.push((x - nx * h, y - ny * h));
    }
    let mut path = Path2D::default();
    path.move_to(left[0].0, left[0].1);
    for &(x, y) in &left[1..] {
        path.line_to(x, y);
    }
    if closed {
        path.close();
        path.move_to(right[n - 1].0, right[n - 1].1);
        for &(x, y) in right[..n - 1].iter().rev() {
            path.line_to(x, y);
        }
        path.close();
        surface.fill_even_odd(&path, colour);
    } else {
        for &(x, y) in right.iter().rev() {
            path.line_to(x, y);
        }
        path.close();
        surface.fill_path(&path.into_path(), colour);
    }
}
