use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, hash, store, value_noise};
use crate::param::Param;
use crate::surface::{Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "chaff";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const INKS: usize = 2;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#0e3b43", "#f2c14e"];
const BLADE_CAP: f64 = 1200.0;
const STEPS: usize = 26;

const COUNT: Param = Param {
    taste: (4, 250),
    ..Param::new(SLUG, "count", 4, 400, 1)
};
const SIZE: Param = Param {
    taste: (10, 100),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const VARY: Param = Param::new(SLUG, "vary", 0, 100, 100);
const APART: Param = Param::new(SLUG, "apart", 0, 100, 100);
const CURVE: Param = Param::new(SLUG, "curve", 0, 100, 100);
const SLIM: Param = Param::new(SLUG, "slim", 0, 100, 100);
const TAPER: Param = Param::new(SLUG, "taper", 0, 100, 100);
const MOTTLE: Param = Param {
    taste: (0, 75),
    ..Param::new(SLUG, "mottle", 0, 100, 100)
};
const COARSE: Param = Param::new(SLUG, "coarse", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[COUNT, SIZE, VARY, APART, CURVE, SLIM, TAPER, MOTTLE, COARSE];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChaffShape {
    Crescent,
    Leaf,
    Bar,
}

const SHAPES: [ChaffShape; 3] = [ChaffShape::Crescent, ChaffShape::Leaf, ChaffShape::Bar];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChaffParams {
    count: u32,
    size: f64,
    vary: f64,
    apart: f64,
    shapes: ChaffShape,
    curve: f64,
    slim: f64,
    taper: f64,
    mottle: f64,
    coarse: f64,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain_pass: Grain,
}

impl Default for ChaffParams {
    fn default() -> Self {
        Self {
            count: 40,
            size: 0.5,
            vary: 0.55,
            apart: 0.0,
            shapes: ChaffShape::Crescent,
            curve: 0.7,
            slim: 0.45,
            taper: 0.3,
            mottle: 0.62,
            coarse: 0.4,
            grain: 0.3,
            dither: Dither::default(),
            grain_pass: Grain::default(),
        }
    }
}

impl ChaffParams {
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

    pub fn apart(&self) -> f64 {
        self.apart
    }

    pub fn set_apart(&mut self, apart: f64) -> Result<(), Error> {
        self.apart = APART.check(apart)?;
        Ok(())
    }

    pub fn shapes(&self) -> ChaffShape {
        self.shapes
    }

    pub fn set_shapes(&mut self, shapes: ChaffShape) {
        self.shapes = shapes;
    }

    pub fn curve(&self) -> f64 {
        self.curve
    }

    pub fn set_curve(&mut self, curve: f64) -> Result<(), Error> {
        self.curve = CURVE.check(curve)?;
        Ok(())
    }

    pub fn slim(&self) -> f64 {
        self.slim
    }

    pub fn set_slim(&mut self, slim: f64) -> Result<(), Error> {
        self.slim = SLIM.check(slim)?;
        Ok(())
    }

    pub fn taper(&self) -> f64 {
        self.taper
    }

    pub fn set_taper(&mut self, taper: f64) -> Result<(), Error> {
        self.taper = TAPER.check(taper)?;
        Ok(())
    }

    pub fn mottle(&self) -> f64 {
        self.mottle
    }

    pub fn set_mottle(&mut self, mottle: f64) -> Result<(), Error> {
        self.mottle = MOTTLE.check(mottle)?;
        Ok(())
    }

    pub fn coarse(&self) -> f64 {
        self.coarse
    }

    pub fn set_coarse(&mut self, coarse: f64) -> Result<(), Error> {
        self.coarse = COARSE.check(coarse)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn dither(&self) -> &Dither {
        &self.dither
    }

    pub fn dither_mut(&mut self) -> &mut Dither {
        &mut self.dither
    }

    pub fn grain_pass(&self) -> &Grain {
        &self.grain_pass
    }

    pub fn grain_pass_mut(&mut self) -> &mut Grain {
        &mut self.grain_pass
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    count: u32,
    size: f64,
    vary: f64,
    apart: f64,
    shapes: ChaffShape,
    curve: f64,
    slim: f64,
    taper: f64,
    mottle: f64,
    coarse: f64,
    grain: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<ChaffParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = ChaffParams::default();
    params.set_count(raw.count)?;
    params.set_size(raw.size)?;
    params.set_vary(raw.vary)?;
    params.set_apart(raw.apart)?;
    params.set_shapes(raw.shapes);
    params.set_curve(raw.curve)?;
    params.set_slim(raw.slim)?;
    params.set_taper(raw.taper)?;
    params.set_mottle(raw.mottle)?;
    params.set_coarse(raw.coarse)?;
    params.set_grain(raw.grain)?;
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
    [
        COUNT.parameter(),
        SIZE.parameter(),
        VARY.parameter(),
        APART.parameter(),
        Parameter::choice("shapes", &["Crescent", "Leaf", "Bar"]),
        CURVE.parameter(),
        SLIM.parameter(),
        TAPER.parameter(),
        MOTTLE.parameter(),
        COARSE.parameter(),
        GRAIN.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> ChaffParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    ChaffParams {
        count: pick(&COUNT) as u32,
        size: pick(&SIZE),
        vary: pick(&VARY),
        apart: pick(&APART),
        shapes: SHAPES[(draw("shapes") % SHAPES.len() as u64) as usize],
        curve: pick(&CURVE),
        slim: pick(&SLIM),
        taper: pick(&TAPER),
        mottle: pick(&MOTTLE),
        coarse: pick(&COARSE),
        ..ChaffParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &ChaffParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain_pass, &params.dither);
}

struct Blade {
    x: f64,
    y: f64,
    a: f64,
    length: f64,
    turn: f64,
    sd: u32,
}

fn blades(p: &ChaffParams, aspect: f64, tool_seed: u32) -> Vec<Blade> {
    let (fw, fh) = (1.0 / aspect.sqrt(), aspect.sqrt());
    let mut rnd = XorShift::new(tool_seed.wrapping_mul(2_246_822_519));
    let base = 0.06 + p.size * 0.38;
    let n = f64::from(p.count).round().clamp(1.0, BLADE_CAP) as usize;
    let mut out: Vec<Blade> = (0..n)
        .map(|_| {
            let u = rnd.next();
            let k = 1.0 + (u * u * 3.2 - 0.5) * p.vary;
            Blade {
                x: (-0.25 + rnd.next() * 1.5) * fw,
                y: (-0.25 + rnd.next() * 1.5) * fh,
                a: rnd.next() * TAU,
                length: base * k.max(0.12),
                turn: if rnd.next() < 0.5 { -1.0 } else { 1.0 },
                sd: (rnd.next() * 9973.0) as u32,
            }
        })
        .collect();
    out.sort_by(|p, q| q.length.total_cmp(&p.length));
    out
}

fn profile(t: f64, shape: ChaffShape, tp: f64) -> f64 {
    match shape {
        ChaffShape::Bar => {
            let e = 0.04 + tp * 0.2;
            (t / e).sqrt().min(((1.0 - t) / e).sqrt()).min(1.0)
        }
        ChaffShape::Leaf => {
            let e = 0.05 + tp * 0.1;
            (t / e).sqrt().min(1.0) * (1.0 - t).powf(0.55 + tp * 1.5)
        }
        ChaffShape::Crescent => (PI * t).sin().powf(0.45 + tp * 1.4),
    }
}

fn outline(path: &mut Path2D, b: &Blade, unit: f64, scale: f64, slim: f64, p: &ChaffParams) {
    let length = b.length * scale * unit;
    let wide = length * (0.1 + slim * 0.42);
    let th = p.curve * 2.3 * b.turn;
    let a0 = b.a - th * 0.5;
    let step = length / STEPS as f64;
    let mut centre = [(0.0, 0.0); STEPS + 1];
    let (mut px, mut py, mut sx, mut sy) = (0.0, 0.0, 0.0, 0.0);
    for (i, point) in centre.iter_mut().enumerate() {
        *point = (px, py);
        sx += px;
        sy += py;
        let h = a0 + th * (i as f64 / STEPS as f64);
        px += h.cos() * step;
        py += h.sin() * step;
    }
    let count = (STEPS + 1) as f64;
    let (bx, by) = (b.x * unit - sx / count, b.y * unit - sy / count);
    let sd = f64::from(b.sd);
    let edges: Vec<((f64, f64), (f64, f64))> = centre
        .iter()
        .enumerate()
        .map(|(i, &(cx, cy))| {
            let t = i as f64 / STEPS as f64;
            let h = a0 + th * t;
            let (nx, ny) = (-h.sin(), h.cos());
            let wob = 1.0 + (value_noise(t * 2.9 + sd * 0.11, sd * 0.07, b.sd) - 0.5) * 0.34;
            let w = profile(t, p.shapes, p.taper) * wide * 0.5 * wob;
            let (x, y) = (bx + cx, by + cy);
            ((x + nx * w, y + ny * w), (x - nx * w, y - ny * w))
        })
        .collect();
    path.move_to(edges[0].0.0, edges[0].0.1);
    for &(left, _) in &edges[1..] {
        path.line_to(left.0, left.1);
    }
    for &(_, right) in edges.iter().rev() {
        path.line_to(right.0, right.1);
    }
    path.close();
}

fn mask(p: &ChaffParams, blades: &[Blade], width: u32, height: u32) -> Vec<u8> {
    let mut mask = Surface::new(width, height);
    let unit = f64::from(width * height).sqrt();
    let green = [0, 255, 0];
    mask.fill([0, 0, 0]);
    if p.apart > 0.004 {
        let pad = 1.0 + p.apart * 0.5;
        for blade in blades {
            let mut knockout = Path2D::default();
            outline(&mut knockout, blade, unit, pad, p.slim * pad, p);
            mask.fill_transformed(&knockout, [0, 0, 0], Transform::IDENTITY);
            let mut path = Path2D::default();
            outline(&mut path, blade, unit, 1.0, p.slim, p);
            mask.fill_transformed(&path, green, Transform::IDENTITY);
        }
    } else {
        let mut path = Path2D::default();
        for blade in blades {
            outline(&mut path, blade, unit, 1.0, p.slim, p);
        }
        mask.fill_transformed(&path, green, Transform::IDENTITY);
    }
    mask.into_image().rgba().chunks(4).map(|px| px[1]).collect()
}

fn paint(surface: &mut Surface, p: &ChaffParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (ground, ink) = (palette.ink(0), palette.ink(1));
    let blades = blades(p, f64::from(h) / f64::from(w), tool_seed);
    let half = |edge: u32| (f64::from(edge) * 0.5).round().max(2.0) as u32;
    let (mw, mh) = (half(w), half(h));
    let green = mask(p, &blades, mw, mh);
    let unit = f64::from(w * h).sqrt();
    let fine = unit * (0.0022 + p.coarse * 0.007);
    let big = fine * 7.5;
    let amp = p.mottle * 2.6;
    let gl = p.grain * 52.0;
    let cell = |at: u32, edge: u32| {
        let g = f64::from(at) * 0.5;
        let lower = (g as u32).min(edge - 1);
        (lower, (lower + 1).min(edge - 1), g - f64::from(lower))
    };
    surface.edit_rgba(|rgba, _, _| {
        for y in 0..h {
            let (y0, y1, fy) = cell(y, mh);
            for x in 0..w {
                let (x0, x1, fx) = cell(x, mw);
                let at = |cx: u32, cy: u32| f64::from(green[(cy * mw + cx) as usize]);
                let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
                let bottom = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
                let g = (top + (bottom - top) * fy) / 255.0;
                let (fx, fy) = (f64::from(x), f64::from(y));
                let n = ((value_noise(fx / fine, fy / fine, tool_seed.wrapping_add(41)) - 0.5)
                    * 0.66
                    + (value_noise(fx / big, fy / big, tool_seed.wrapping_add(13)) - 0.5) * 0.34)
                    * amp;
                let colour = if g + n > 0.5 { ink } else { ground };
                let q = if gl > 0.002 {
                    (hash(x as i32, y as i32, tool_seed.wrapping_add(71)) - 0.5) * gl
                } else {
                    0.0
                };
                let i = ((y * w + x) * 4) as usize;
                for c in 0..3 {
                    rgba[i + c] = store(f64::from(colour[c]) + q);
                }
                rgba[i + 3] = 255;
            }
        }
    });
}
