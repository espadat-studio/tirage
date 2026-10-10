use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store, value_noise,
};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "crowd";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 3] = ["#b5bab6", "#d6ee3a", "#f08ae6"];

const CROWD: Param = Param {
    taste: (8, 48),
    ..Param::new(SLUG, "crowd", 1, 48, 1)
};
const SCALE: Param = Param {
    taste: (0, 60),
    ..Param::new(SLUG, "scale", 0, 100, 100)
};
const WOBBLE: Param = Param::new(SLUG, "wobble", 0, 100, 100);
const BLUR: Param = Param::new(SLUG, "blur", 0, 100, 100);
const HALO: Param = Param::new(SLUG, "halo", 0, 100, 100);
const EDGE: Param = Param::new(SLUG, "edge", 0, 100, 100);
const ARROWS: Param = Param::new(SLUG, "arrows", 0, 100, 100);
const ARROW_SIZE: Param = Param::new(SLUG, "arrowSize", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[CROWD, SCALE, WOBBLE, BLUR, HALO, EDGE, ARROWS, ARROW_SIZE];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrowdParams {
    crowd: u32,
    scale: f64,
    wobble: f64,
    blur: f64,
    halo: f64,
    edge: f64,
    arrows: f64,
    #[serde(rename = "arrowSize")]
    arrow_size: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for CrowdParams {
    fn default() -> Self {
        Self {
            crowd: 30,
            scale: 0.4,
            wobble: 0.35,
            blur: 0.5,
            halo: 0.5,
            edge: 0.3,
            arrows: 0.5,
            arrow_size: 0.5,
            dither: Dither::default(),
            grain: Grain::thermal(),
        }
    }
}

impl CrowdParams {
    pub fn crowd(&self) -> u32 {
        self.crowd
    }

    pub fn set_crowd(&mut self, crowd: u32) -> Result<(), Error> {
        CROWD.check(f64::from(crowd))?;
        self.crowd = crowd;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn wobble(&self) -> f64 {
        self.wobble
    }

    pub fn set_wobble(&mut self, wobble: f64) -> Result<(), Error> {
        self.wobble = WOBBLE.check(wobble)?;
        Ok(())
    }

    pub fn blur(&self) -> f64 {
        self.blur
    }

    pub fn set_blur(&mut self, blur: f64) -> Result<(), Error> {
        self.blur = BLUR.check(blur)?;
        Ok(())
    }

    pub fn halo(&self) -> f64 {
        self.halo
    }

    pub fn set_halo(&mut self, halo: f64) -> Result<(), Error> {
        self.halo = HALO.check(halo)?;
        Ok(())
    }

    pub fn edge(&self) -> f64 {
        self.edge
    }

    pub fn set_edge(&mut self, edge: f64) -> Result<(), Error> {
        self.edge = EDGE.check(edge)?;
        Ok(())
    }

    pub fn arrows(&self) -> f64 {
        self.arrows
    }

    pub fn set_arrows(&mut self, arrows: f64) -> Result<(), Error> {
        self.arrows = ARROWS.check(arrows)?;
        Ok(())
    }

    pub fn arrow_size(&self) -> f64 {
        self.arrow_size
    }

    pub fn set_arrow_size(&mut self, arrow_size: f64) -> Result<(), Error> {
        self.arrow_size = ARROW_SIZE.check(arrow_size)?;
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
    crowd: u32,
    scale: f64,
    wobble: f64,
    blur: f64,
    halo: f64,
    edge: f64,
    arrows: f64,
    #[serde(rename = "arrowSize")]
    arrow_size: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<CrowdParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = CrowdParams::default();
    params.set_crowd(raw.crowd)?;
    params.set_scale(raw.scale)?;
    params.set_wobble(raw.wobble)?;
    params.set_blur(raw.blur)?;
    params.set_halo(raw.halo)?;
    params.set_edge(raw.edge)?;
    params.set_arrows(raw.arrows)?;
    params.set_arrow_size(raw.arrow_size)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> CrowdParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    CrowdParams {
        crowd: pick(&CROWD) as u32,
        scale: pick(&SCALE),
        wobble: pick(&WOBBLE),
        blur: pick(&BLUR),
        halo: pick(&HALO),
        edge: pick(&EDGE),
        arrows: pick(&ARROWS),
        arrow_size: pick(&ARROW_SIZE),
        ..CrowdParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &CrowdParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

type Rgb = [f64; 3];

const RAMP: usize = 1024;
const CORE_AT: f64 = 0.62;

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

fn ramp(palette: &Palette, edge: f64) -> Vec<[f32; 3]> {
    let inks: Vec<Rgb> = palette
        .inks()
        .iter()
        .map(|ink| ink.map(f64::from))
        .collect();
    let (ground, core) = (inks[0], inks[1]);
    let halos = if inks.len() > 2 {
        &inks[2..]
    } else {
        &inks[1..2]
    };
    let dark = mix(halos[0], [30.0, 0.0, 70.0], 0.55);
    let rim = mix(halos[0], dark, edge);
    let mut stops = vec![(0.0, ground), (0.05, ground), (0.2, rim)];
    let (start, bands) = (0.32, halos.len() as f64);
    let span = (CORE_AT - start).max(0.06);
    for (k, &halo) in halos.iter().enumerate() {
        stops.push((start + k as f64 * span / bands, halo));
    }
    stops.extend([(CORE_AT + 0.12, core), (1.0, core)]);
    let mut k = 0;
    (0..RAMP)
        .map(|i| {
            let m = i as f64 / (RAMP - 1) as f64;
            while k < stops.len() - 2 && m >= stops[k + 1].0 {
                k += 1;
            }
            let ((ma, ca), (mb, cb)) = (stops[k], stops[k + 1]);
            let t = ((m - ma) / (mb - ma)).clamp(0.0, 1.0);
            mix(ca, cb, t * t * (3.0 - 2.0 * t)).map(|c| c as f32)
        })
        .collect()
}

struct Figure {
    x: f64,
    y: f64,
    head: f64,
    body_height: f64,
    body_width: f64,
    corner: f64,
    seed: u32,
}

fn figures(params: &CrowdParams, width: f64, height: f64, unit: f64, seed: u32) -> Vec<Figure> {
    let mut rng = XorShift::new(seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    (0..params.crowd)
        .map(|k| {
            let big = rng.next();
            let head = unit * (0.018 + params.scale * 0.06) * (0.5 + big.powf(1.6) * 1.3);
            let body_height = head * (2.4 + rng.next() * 3.2);
            let body_width = head * (1.5 + rng.next() * 0.8);
            let rows = 3 + (rng.next() * 3.0) as u32;
            let row = (rng.next() * f64::from(rows)) as u32;
            let x = (-0.06 + rng.next() * 1.12) * width;
            let y = (f64::from(row) + rng.next() * 1.2 - 0.3) / f64::from(rows) * height;
            rng.next();
            rng.next();
            Figure {
                x,
                y,
                head,
                body_height,
                body_width,
                corner: head * (0.6 + rng.next() * 0.5),
                seed: seed.wrapping_mul(13).wrapping_add(k * 7),
            }
        })
        .collect()
}

fn cell_range(low: f64, high: f64, cell: f64, cells: usize) -> std::ops::Range<usize> {
    let low = ((low / cell) as i64).max(0);
    let high = ((high / cell) as i64).min(cells as i64 - 1);
    low as usize..(high + 1).max(low) as usize
}

fn box_blur(field: &mut [f32], scratch: &mut [f32], width: usize, height: usize, radius: f64) {
    if radius < 1.0 {
        return;
    }
    let r = radius as i64;
    let n = (2 * r + 1) as f64;
    let (w, h) = (width as i64, height as i64);
    let at = |i: i64, edge: i64| i.clamp(0, edge - 1) as usize;
    for _ in 0..3 {
        for y in 0..height {
            let row = y * width;
            let mut sum: f64 = (-r..=r).map(|i| f64::from(field[row + at(i, w)])).sum();
            for x in 0..w {
                scratch[row + x as usize] = (sum / n) as f32;
                sum +=
                    f64::from(field[row + at(x + r + 1, w)]) - f64::from(field[row + at(x - r, w)]);
            }
        }
        for x in 0..width {
            let mut sum: f64 = (-r..=r)
                .map(|i| f64::from(scratch[at(i, h) * width + x]))
                .sum();
            for y in 0..h {
                field[y as usize * width + x] = (sum / n) as f32;
                sum += f64::from(scratch[at(y + r + 1, h) * width + x])
                    - f64::from(scratch[at(y - r, h) * width + x]);
            }
        }
    }
}

fn mask(
    params: &CrowdParams,
    figures: &[Figure],
    unit: f64,
    cell: f64,
    fw: usize,
    fh: usize,
) -> Vec<f32> {
    let mut field = vec![0.0f32; fw * fh];
    let halo = unit * (0.012 + params.halo * 0.06);
    let margin = halo * 2.0 + unit * 0.03;
    for g in figures {
        let reach = g.body_width * 0.5 + g.head + margin;
        let columns = cell_range(g.x - reach, g.x + reach, cell, fw);
        let rows = cell_range(
            g.y - g.head * 1.2 - margin,
            g.y + g.head + g.body_height + margin,
            cell,
            fh,
        );
        let (hw, hh) = (
            g.body_width * 0.5 - g.corner,
            g.body_height * 0.5 - g.corner,
        );
        let by = g.y + g.head * 0.9 + g.body_height * 0.5;
        let wf = 1.0 / (g.head * 1.4);
        for j in rows {
            let py = (j as f64 - 1.0) * cell + cell * 0.5;
            for i in columns.clone() {
                let px = (i as f64 - 1.0) * cell + cell * 0.5;
                let mut d = (px - g.x).hypot(py - g.y) - g.head;
                let (qx, qy) = ((px - g.x).abs() - hw, (py - by).abs() - hh);
                let body = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - g.corner;
                if body < d {
                    d = body;
                }
                d += (value_noise(px * wf + 3.0, py * wf + 7.0, g.seed) - 0.5)
                    * params.wobble
                    * g.head
                    * 1.6;
                let v = (0.5 - d / (halo * 2.0)).clamp(0.0, 1.0);
                let q = j * fw + i;
                if v > f64::from(field[q]) {
                    field[q] = v as f32;
                }
            }
        }
    }
    let mut scratch = vec![0.0f32; fw * fh];
    box_blur(
        &mut field,
        &mut scratch,
        fw,
        fh,
        (0.004 + params.blur * 0.04) * unit / cell,
    );
    field
}

fn arrows(params: &CrowdParams, width: f64, height: f64, unit: f64, seed: u32) -> (Path2D, f64) {
    let mut path = Path2D::default();
    let density = params.arrows;
    let [sd1, sd2, sd3] = [1, 2, 3].map(|k| seed.wrapping_mul(7).wrapping_add(k));
    let spacing = unit * 0.019 * (0.7 + params.arrow_size * 0.9);
    let size = spacing * 0.55;
    let columns = (width / spacing).ceil() as i32 + 1;
    let rows = (height / spacing).ceil() as i32 + 1;
    for j in 0..rows {
        for i in 0..columns {
            let x = (f64::from(i) + 0.5) * spacing;
            let y = (f64::from(j) + 0.5) * spacing;
            let patch = value_noise(x / unit * 3.2 + 1.0, y / unit * 3.2 + 5.0, sd1) * 0.7
                + value_noise(x / unit * 9.0 + 4.0, y / unit * 9.0 + 2.0, sd2) * 0.3;
            if patch < 1.0 - density * 0.95 || hash(i, j, sd3) < 0.3 {
                continue;
            }
            let jx = (hash(i, j, sd3.wrapping_add(1)) - 0.5) * spacing * 0.6;
            let jy = (hash(i, j, sd3.wrapping_add(2)) - 0.5) * spacing * 0.6;
            let lean = PI * 0.75
                + (value_noise(x / unit * 2.2 + 9.0, y / unit * 2.2 + 3.0, sd2) - 0.5) * PI * 1.6;
            let a = lean + (hash(i, j, sd3.wrapping_add(3)) - 0.5) * 0.9;
            let (ca, sa) = (a.cos(), a.sin());
            let (ax, ay) = (x + jx, y + jy);
            let (tx, ty) = (ax + ca * size * 0.5, ay + sa * size * 0.5);
            path.move_to(ax - ca * size * 0.5, ay - sa * size * 0.5);
            path.line_to(tx, ty);
            path.move_to(
                tx - ca * size * 0.42 - sa * size * 0.34,
                ty - sa * size * 0.42 + ca * size * 0.34,
            );
            path.line_to(tx, ty);
            path.line_to(
                tx - ca * size * 0.42 + sa * size * 0.34,
                ty - sa * size * 0.42 - ca * size * 0.34,
            );
        }
    }
    (path, size)
}

fn paint(surface: &mut Surface, params: &CrowdParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let unit = (width * height).sqrt();
    let cell = round_half_up(unit / 420.0).max(2.0);
    let fw = (width / cell).ceil() as usize + 2;
    let fh = (height / cell).ceil() as usize + 2;
    let figures = figures(params, width, height, unit, tool_seed);
    let field = mask(params, &figures, unit, cell, fw, fh);
    let ramp = ramp(palette, params.edge);
    surface.edit_rgba(|rgba, columns, rows| {
        for y in 0..rows as usize {
            let gy = (y as f64 + cell) / cell - 0.5;
            let (yi, yf) = (gy as usize, gy - gy.trunc());
            let (row0, row1) = (yi * fw, (yi + 1).min(fh - 1) * fw);
            for x in 0..columns as usize {
                let gx = (x as f64 + cell) / cell - 0.5;
                let (xi, xf) = (gx as usize, gx - gx.trunc());
                let xj = (xi + 1).min(fw - 1);
                let f = |q: usize| f64::from(field[q]);
                let m = ((f(row0 + xi) * (1.0 - xf) + f(row0 + xj) * xf) * (1.0 - yf)
                    + (f(row1 + xi) * (1.0 - xf) + f(row1 + xj) * xf) * yf)
                    .min(1.0);
                let ink = ramp[(m * (RAMP - 1) as f64) as usize];
                let p = (y * columns as usize + x) * 4;
                for c in 0..3 {
                    rgba[p + c] = store(f64::from(ink[c]));
                }
                rgba[p + 3] = 255;
            }
        }
    });
    if params.arrows > 0.0 {
        let (path, size) = arrows(params, width, height, unit, tool_seed);
        if !path.is_empty() {
            let core = palette.inks()[1];
            let line = (size * 0.15).max(0.8);
            surface.with_alpha(0.9, |surface| {
                surface.stroke(&path, core, line, Cap::Round, Join::Round);
            });
        }
    }
}
