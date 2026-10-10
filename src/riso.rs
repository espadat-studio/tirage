use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "riso";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#3a86ff", "#ff006e", "#fb5607", "#1b1b2f", "#ffbe0b", "#f7f3ea",
];
const INK: [u8; 3] = [0x1a, 0x1a, 0x1a];
const VB: f64 = 1000.0;

const BANDS: Param = Param::new(SLUG, "bands", 2, 5, 1);
const ROUGH: Param = Param::new(SLUG, "rough", 10, 100, 100);
const SCRIB: Param = Param::new(SLUG, "scrib", 0, 100, 100);
const DOTP: Param = Param::new(SLUG, "dotp", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[BANDS, ROUGH, SCRIB, DOTP];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RisoParams {
    bands: u32,
    rough: f64,
    scrib: f64,
    dotp: f64,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for RisoParams {
    fn default() -> Self {
        Self {
            bands: 3,
            rough: 0.5,
            scrib: 0.6,
            dotp: 0.5,
            grain: 0.5,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl RisoParams {
    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn rough(&self) -> f64 {
        self.rough
    }

    pub fn set_rough(&mut self, rough: f64) -> Result<(), Error> {
        self.rough = ROUGH.check(rough)?;
        Ok(())
    }

    pub fn scrib(&self) -> f64 {
        self.scrib
    }

    pub fn set_scrib(&mut self, scrib: f64) -> Result<(), Error> {
        self.scrib = SCRIB.check(scrib)?;
        Ok(())
    }

    pub fn dotp(&self) -> f64 {
        self.dotp
    }

    pub fn set_dotp(&mut self, dotp: f64) -> Result<(), Error> {
        self.dotp = DOTP.check(dotp)?;
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
        &self.chassis_grain
    }

    pub fn grain_pass_mut(&mut self) -> &mut Grain {
        &mut self.chassis_grain
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    bands: u32,
    rough: f64,
    scrib: f64,
    dotp: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<RisoParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = RisoParams::default();
    params.set_bands(raw.bands)?;
    params.set_rough(raw.rough)?;
    params.set_scrib(raw.scrib)?;
    params.set_dotp(raw.dotp)?;
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
    PARAMS
        .iter()
        .chain([&GRAIN])
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> RisoParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    RisoParams {
        bands: pick(&BANDS) as u32,
        rough: pick(&ROUGH),
        scrib: pick(&SCRIB),
        dotp: pick(&DOTP),
        ..RisoParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &RisoParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Blocks,
    Streaks,
    Static,
}

struct Band {
    y0: f64,
    y1: f64,
    kind: Kind,
    bg: usize,
    fg: usize,
    v: f64,
}

struct Dot {
    cx: f64,
    cy: f64,
    radius: f64,
    ink: usize,
}

fn luminance([r, g, b]: [u8; 3]) -> f64 {
    let linear = |c: u8| {
        let v = f64::from(c) / 255.0;
        if v <= 0.039_28 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn against(inks: &[[u8; 3]], base: usize, draw: f64) -> usize {
    let n = inks.len();
    let start = (draw * n as f64) as usize;
    (0..n)
        .map(|q| (start + q) % n)
        .find(|&c| c != base && (luminance(inks[c]) - luminance(inks[base])).abs() > 0.09)
        .unwrap_or((base + 1) % n)
}

type Poly = Vec<(f64, f64)>;

#[expect(clippy::too_many_arguments, reason = "a torn rect needs its keys")]
fn torn(x: f64, y: f64, w: f64, h: f64, rough: f64, k1: i32, k2: i32, sw: u32) -> Poly {
    let j = |q: i32, edge: i32| (hash(k1, q, (edge + k2 * 7) as u32, sw) - 0.5) * rough;
    let at = |q: i32| f64::from(q) / 5.0;
    let mut pts = Vec::with_capacity(20);
    pts.extend((0..=5).map(|q| (x + w * at(q), y + j(q, 1))));
    pts.extend((1..=5).map(|q| (x + w + j(q, 2), y + h * at(q))));
    pts.extend((0..=4).rev().map(|q| (x + w * at(q), y + h + j(q, 3))));
    pts.extend((1..=4).rev().map(|q| (x + j(q, 4), y + h * at(q))));
    pts
}

fn shapes(band: &Band, id: i32, rough: f64, sw: u32) -> Vec<Poly> {
    let bh = band.y1 - band.y0;
    let g = VB * 0.016 * (0.4 + rough * 1.6);
    let h = |x: i32, y: i32, z: i32| hash(x, y, z as u32, sw);
    let mut out = Vec::new();
    match band.kind {
        Kind::Blocks => {
            let cols = 5 + (band.v * 3.0) as i32;
            let rows = round_half_up(bh / (VB * 0.16)).max(1.0) as i32;
            let (cw, ch) = (VB / f64::from(cols), bh / f64::from(rows));
            for cy in 0..rows {
                for cx in 0..cols {
                    if h(cx, cy, id * 3 + 1) >= 0.58 {
                        continue;
                    }
                    let jx = (h(cx, cy, id + 11) - 0.5) * cw * 0.4;
                    let jy = (h(cx, cy, id + 12) - 0.5) * ch * 0.3;
                    let w = cw * (0.7 + h(cx, cy, id + 13) * 0.55);
                    let hh = ch * (0.75 + h(cx, cy, id + 14) * 0.5);
                    out.push(torn(
                        f64::from(cx) * cw + jx,
                        band.y0 + f64::from(cy) * ch + jy,
                        w,
                        hh,
                        g,
                        cx + cy * 31,
                        id,
                        sw,
                    ));
                }
            }
        }
        Kind::Streaks => {
            let cols = 7 + (band.v * 3.0) as i32;
            let cw = VB / f64::from(cols);
            for i in 0..cols {
                if h(i, 0, id * 7 + 2) < 0.16 {
                    continue;
                }
                let (mut left, mut right) = (Vec::new(), Vec::new());
                for q in 0..=6 {
                    let yy = band.y0 + bh * f64::from(q) / 6.0;
                    let sway = (h(i, q, id * 7 + 3) - 0.5) * cw * 1.1 * (0.4 + rough);
                    let wd = cw * (0.34 + h(i, q, id * 7 + 4) * 0.5);
                    let mid = f64::from(i) * cw + cw / 2.0 + sway;
                    left.push((mid - wd / 2.0, yy));
                    right.push((mid + wd / 2.0, yy));
                }
                left.extend(right.into_iter().rev());
                out.push(left);
            }
        }
        Kind::Static => {
            let rows = round_half_up(bh / (VB * 0.028)).max(3.0) as i32;
            let rh = bh / f64::from(rows);
            for ry in 0..rows {
                let dashes = 2 + (h(ry, 1, id * 9 + 1) * 6.0) as i32;
                for d in 0..dashes {
                    let x0 = h(ry, d * 2, id * 9 + 2) * VB;
                    let len = VB * (0.03 + h(ry, d * 2 + 1, id * 9 + 3) * 0.16);
                    out.push(torn(
                        x0,
                        band.y0 + f64::from(ry) * rh + rh * 0.2,
                        len,
                        rh * 0.55,
                        g * 0.5,
                        ry * 17 + d,
                        id,
                        sw,
                    ));
                }
            }
        }
    }
    out
}

fn scribble(rng: &mut Xorshift, i: i32, vh: f64, tool_seed: u32) -> Poly {
    let (mut x, mut y) = (rng.next() * VB * 0.3, rng.next() * vh * 0.5);
    let mut angle = rng.next() * TAU;
    let step = VB * 0.02 + rng.next() * VB * 0.02;
    let mut walk = Vec::with_capacity(70);
    for q in 0..70 {
        walk.push((x, y));
        angle += (hash(i, q, 3, tool_seed) - 0.5) * 1.4;
        if !(0.0..=VB).contains(&x) {
            angle = PI - angle;
        }
        if !(0.0..=vh).contains(&y) {
            angle = -angle;
        }
        x += angle.cos() * step;
        y += angle.sin() * step;
    }
    let mut smooth = vec![walk[0]];
    smooth.extend(walk.windows(3).map(|w| {
        (
            (w[0].0 + w[1].0 * 2.0 + w[2].0) / 4.0,
            (w[0].1 + w[1].1 * 2.0 + w[2].1) / 4.0,
        )
    }));
    smooth.push(walk[69]);
    smooth
}

fn trace(pts: &[(f64, f64)], k: f64) -> Path2D {
    let mut path = Path2D::default();
    path.move_to(pts[0].0 * k, pts[0].1 * k);
    for &(x, y) in &pts[1..] {
        path.line_to(x * k, y * k);
    }
    path
}

fn fill_poly(surface: &mut Surface, pts: &[(f64, f64)], k: f64, ink: [u8; 3]) {
    let mut path = trace(pts, k);
    path.close();
    surface.fill_path(&path.into_path(), ink);
}

fn paint(surface: &mut Surface, p: &RisoParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let inks = palette.inks();
    let nc = inks.len();
    let vh = VB * h / w;
    let k = w / VB;
    let mut rng = Xorshift::new(tool_seed);

    let n = p.bands as usize;
    let weights: Vec<f64> = (0..n).map(|_| 0.6 + rng.next() * 1.2).collect();
    let sum: f64 = weights.iter().sum();
    let mut kinds = [
        Kind::Blocks,
        Kind::Streaks,
        Kind::Static,
        Kind::Blocks,
        Kind::Streaks,
    ];
    for i in (1..kinds.len()).rev() {
        let j = (rng.next() * (i + 1) as f64) as usize;
        kinds.swap(i, j);
    }
    let mut bands = Vec::with_capacity(n);
    let mut y = 0.0;
    for (i, weight) in weights.iter().enumerate() {
        let bh = weight / sum * vh;
        let bg = (rng.next() * nc as f64) as usize;
        let fg = against(inks, bg, rng.next());
        let v = rng.next();
        rng.next();
        bands.push(Band {
            y0: y,
            y1: y + bh,
            kind: kinds[i],
            bg,
            fg,
            v,
        });
        y += bh;
    }
    let scribbles: Vec<Poly> = (0..round_half_up(p.scrib * 3.0) as i32)
        .map(|i| scribble(&mut rng, i, vh, tool_seed))
        .collect();
    let dot = (rng.next() < p.dotp).then(|| {
        let band = &bands[(rng.next() * n as f64) as usize];
        let radius = VB.min(vh) * (0.14 + rng.next() * 0.1);
        let cx = VB * (0.3 + rng.next() * 0.4);
        let cy = (band.y0 + (band.y1 - band.y0) * rng.next())
            .max(band.y0 + radius * 0.3)
            .min(band.y1 - radius * 0.3);
        let ink = against(inks, band.bg, rng.next());
        rng.next();
        Dot {
            cx,
            cy,
            radius,
            ink,
        }
    });
    let sw = (rng.next() * 1e9) as u32;

    for (id, band) in bands.iter().enumerate() {
        let mut clip = Path2D::default();
        let (top, bottom) = (band.y0 * k - 0.5, band.y1 * k + 0.5);
        clip.move_to(0.0, top);
        clip.line_to(w, top);
        clip.line_to(w, bottom);
        clip.line_to(0.0, bottom);
        clip.close();
        let shapes = shapes(band, id as i32, p.rough, sw);
        surface.with_clip(&clip, |surface| {
            surface.fill_box(
                0.0,
                band.y0 * k - 1.0,
                w,
                (band.y1 - band.y0) * k + 2.0,
                inks[band.bg % nc],
            );
            for shape in &shapes {
                fill_poly(surface, shape, k, inks[band.fg % nc]);
            }
        });
    }

    if let Some(dot) = dot {
        let pts: Poly = (0..36)
            .map(|q| {
                let a = f64::from(q) / 36.0 * TAU;
                let r = dot.radius * (1.0 + (hash(q, 1, 77, sw) - 0.5) * 0.05 * p.rough);
                (dot.cx + a.cos() * r, dot.cy + a.sin() * r)
            })
            .collect();
        fill_poly(surface, &pts, k, inks[dot.ink % nc]);
    }

    let line = (VB * 0.0035 * k).max(1.0);
    for walk in &scribbles {
        surface.stroke(&trace(walk, k), INK, line, Cap::Round, Join::Round);
    }

    let side = (VB * 0.0016 * k).max(1.0);
    for i in 0..round_half_up(p.grain * 700.0) as i32 {
        let x = hash(i, 1, 91, sw) * VB;
        let y = hash(i, 2, 92, sw) * vh;
        let (ink, alpha) = if hash(i, 3, 93, sw) < 0.55 {
            ([10, 10, 10], 0.5)
        } else {
            ([255, 255, 255], 0.4)
        };
        surface.with_alpha(alpha, |surface| {
            surface.fill_box(x * k, y * k, side, side, ink);
        });
    }
}
