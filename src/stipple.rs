use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::terrain::{fbm, hash};
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "stipple";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 14;
pub(crate) const MAX_INKS: Option<usize> = None;

const GROUND: [u8; 3] = [0x0b, 0x13, 0x2b];
const ACCENT: [u8; 3] = [0xff, 0xd1, 0x66];
const PAGE: f64 = 1000.0;

const DEFAULT_PALETTE: [&str; 3] = ["#3a86ff", "#8ab6ff", "#d6e6ff"];

const RES: Param = Param::new(SLUG, "res", 12, 200, 1);
const DOT: Param = Param {
    taste: (75, 140),
    ..Param::new(SLUG, "dot", 5, 140, 100)
};
const VARY: Param = Param::new(SLUG, "vary", 0, 100, 100);
const CUT: Param = Param::new(SLUG, "cut", 0, 95, 100);
const JIT: Param = Param::new(SLUG, "jit", 0, 100, 100);
const EDGE: Param = Param::new(SLUG, "edge", 0, 100, 100);
const LOOSE: Param = Param::new(SLUG, "loose", 0, 60, 100);
const ZOOM: Param = Param::new(SLUG, "zoom", 5, 90, 10);
const WARP: Param = Param {
    step: 2,
    ..Param::new(SLUG, "warp", 0, 300, 100)
};
const OCT: Param = Param::new(SLUG, "oct", 1, 7, 1);
const CONTRAST: Param = Param {
    step: 5,
    ..Param::new(SLUG, "contrast", 40, 400, 100)
};
const FOLDS: Param = Param::new(SLUG, "folds", 2, 16, 1);
const ACC_RATE: Param = Param::new(SLUG, "accRate", 0, 150, 1000);

pub(crate) const PARAMS: &[Param] = &[
    RES, DOT, VARY, CUT, JIT, EDGE, LOOSE, ZOOM, WARP, OCT, CONTRAST, FOLDS, ACC_RATE,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DotMode {
    Lattice,
    Contour,
}

impl DotMode {
    pub const ALL: &[DotMode] = &[Self::Lattice, Self::Contour];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lattice {
    Square,
    Hex,
}

impl Lattice {
    pub const ALL: &[Lattice] = &[Self::Square, Self::Hex];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DotShape {
    Circle,
    Square,
}

impl DotShape {
    pub const ALL: &[DotShape] = &[Self::Circle, Self::Square];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Symmetry {
    None,
    Mirror,
    Quad,
    Radial,
}

impl Symmetry {
    pub const ALL: &[Symmetry] = &[Self::None, Self::Mirror, Self::Quad, Self::Radial];
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StippleParams {
    #[serde(rename = "modesDot")]
    modes_dot: DotMode,
    lattices: Lattice,
    shapes: DotShape,
    res: u32,
    dot: f64,
    vary: f64,
    cut: f64,
    jit: f64,
    edge: f64,
    loose: f64,
    zoom: f64,
    warp: f64,
    oct: u32,
    contrast: f64,
    syms: Symmetry,
    folds: u32,
    #[serde(rename = "accRate")]
    acc_rate: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for StippleParams {
    fn default() -> Self {
        Self {
            modes_dot: DotMode::Contour,
            lattices: Lattice::Square,
            shapes: DotShape::Circle,
            res: 72,
            dot: 0.62,
            vary: 0.7,
            cut: 0.34,
            jit: 0.0,
            edge: 0.85,
            loose: 0.12,
            zoom: 2.4,
            warp: 0.8,
            oct: 4,
            contrast: 1.6,
            syms: Symmetry::None,
            folds: 6,
            acc_rate: 0.055,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl StippleParams {
    pub fn modes_dot(&self) -> DotMode {
        self.modes_dot
    }

    pub fn set_modes_dot(&mut self, modes_dot: DotMode) {
        self.modes_dot = modes_dot;
    }

    pub fn lattices(&self) -> Lattice {
        self.lattices
    }

    pub fn set_lattices(&mut self, lattices: Lattice) {
        self.lattices = lattices;
    }

    pub fn shapes(&self) -> DotShape {
        self.shapes
    }

    pub fn set_shapes(&mut self, shapes: DotShape) {
        self.shapes = shapes;
    }

    pub fn res(&self) -> u32 {
        self.res
    }

    pub fn set_res(&mut self, res: u32) -> Result<(), Error> {
        RES.check(f64::from(res))?;
        self.res = res;
        Ok(())
    }

    pub fn dot(&self) -> f64 {
        self.dot
    }

    pub fn set_dot(&mut self, dot: f64) -> Result<(), Error> {
        self.dot = DOT.check(dot)?;
        Ok(())
    }

    pub fn vary(&self) -> f64 {
        self.vary
    }

    pub fn set_vary(&mut self, vary: f64) -> Result<(), Error> {
        self.vary = VARY.check(vary)?;
        Ok(())
    }

    pub fn cut(&self) -> f64 {
        self.cut
    }

    pub fn set_cut(&mut self, cut: f64) -> Result<(), Error> {
        self.cut = CUT.check(cut)?;
        Ok(())
    }

    pub fn jit(&self) -> f64 {
        self.jit
    }

    pub fn set_jit(&mut self, jit: f64) -> Result<(), Error> {
        self.jit = JIT.check(jit)?;
        Ok(())
    }

    pub fn edge(&self) -> f64 {
        self.edge
    }

    pub fn set_edge(&mut self, edge: f64) -> Result<(), Error> {
        self.edge = EDGE.check(edge)?;
        Ok(())
    }

    pub fn loose(&self) -> f64 {
        self.loose
    }

    pub fn set_loose(&mut self, loose: f64) -> Result<(), Error> {
        self.loose = LOOSE.check(loose)?;
        Ok(())
    }

    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f64) -> Result<(), Error> {
        self.zoom = ZOOM.check(zoom)?;
        Ok(())
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn oct(&self) -> u32 {
        self.oct
    }

    pub fn set_oct(&mut self, oct: u32) -> Result<(), Error> {
        OCT.check(f64::from(oct))?;
        self.oct = oct;
        Ok(())
    }

    pub fn contrast(&self) -> f64 {
        self.contrast
    }

    pub fn set_contrast(&mut self, contrast: f64) -> Result<(), Error> {
        self.contrast = CONTRAST.check(contrast)?;
        Ok(())
    }

    pub fn syms(&self) -> Symmetry {
        self.syms
    }

    pub fn set_syms(&mut self, syms: Symmetry) {
        self.syms = syms;
    }

    pub fn folds(&self) -> u32 {
        self.folds
    }

    pub fn set_folds(&mut self, folds: u32) -> Result<(), Error> {
        FOLDS.check(f64::from(folds))?;
        self.folds = folds;
        Ok(())
    }

    pub fn acc_rate(&self) -> f64 {
        self.acc_rate
    }

    pub fn set_acc_rate(&mut self, acc_rate: f64) -> Result<(), Error> {
        self.acc_rate = ACC_RATE.check(acc_rate)?;
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
    #[serde(rename = "modesDot")]
    modes_dot: DotMode,
    lattices: Lattice,
    shapes: DotShape,
    res: u32,
    dot: f64,
    vary: f64,
    cut: f64,
    jit: f64,
    edge: f64,
    loose: f64,
    zoom: f64,
    warp: f64,
    oct: u32,
    contrast: f64,
    syms: Symmetry,
    folds: u32,
    #[serde(rename = "accRate")]
    acc_rate: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<StippleParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = StippleParams::default();
    params.set_modes_dot(raw.modes_dot);
    params.set_lattices(raw.lattices);
    params.set_shapes(raw.shapes);
    params.set_res(raw.res)?;
    params.set_dot(raw.dot)?;
    params.set_vary(raw.vary)?;
    params.set_cut(raw.cut)?;
    params.set_jit(raw.jit)?;
    params.set_edge(raw.edge)?;
    params.set_loose(raw.loose)?;
    params.set_zoom(raw.zoom)?;
    params.set_warp(raw.warp)?;
    params.set_oct(raw.oct)?;
    params.set_contrast(raw.contrast)?;
    params.set_syms(raw.syms);
    params.set_folds(raw.folds)?;
    params.set_acc_rate(raw.acc_rate)?;
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
        Parameter::choice("modesDot", &["Lattice", "Contour"]),
        Parameter::choice("lattices", &["Square", "Hex"]),
        Parameter::choice("shapes", &["Circle", "Square"]),
        RES.parameter(),
        DOT.parameter(),
        VARY.parameter(),
        CUT.parameter(),
        JIT.parameter(),
        EDGE.parameter(),
        LOOSE.parameter(),
        ZOOM.parameter(),
        WARP.parameter(),
        OCT.parameter(),
        CONTRAST.parameter(),
        Parameter::choice("syms", &["None", "Mirror", "Quad", "Radial"]),
        FOLDS.parameter(),
        ACC_RATE.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> StippleParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    StippleParams {
        modes_dot: DotMode::ALL[(draw("modesDot") % DotMode::ALL.len() as u64) as usize],
        lattices: Lattice::ALL[(draw("lattices") % Lattice::ALL.len() as u64) as usize],
        shapes: DotShape::ALL[(draw("shapes") % DotShape::ALL.len() as u64) as usize],
        res: pick(&RES) as u32,
        dot: pick(&DOT),
        vary: pick(&VARY),
        cut: pick(&CUT),
        jit: pick(&JIT),
        edge: pick(&EDGE),
        loose: pick(&LOOSE),
        zoom: pick(&ZOOM),
        warp: pick(&WARP),
        oct: pick(&OCT) as u32,
        contrast: pick(&CONTRAST),
        syms: Symmetry::ALL[(draw("syms") % Symmetry::ALL.len() as u64) as usize],
        folds: pick(&FOLDS) as u32,
        acc_rate: pick(&ACC_RATE),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

struct Field<'a> {
    params: &'a StippleParams,
    seed: u32,
    aspect: f64,
}

impl Field<'_> {
    fn fold(&self, nx: f64, ny: f64) -> (f64, f64) {
        let (mut u, mut v) = (nx - 0.5, ny - 0.5);
        match self.params.syms {
            Symmetry::None => {}
            Symmetry::Mirror => u = u.abs(),
            Symmetry::Quad => (u, v) = (u.abs(), v.abs()),
            Symmetry::Radial => {
                let r = u.hypot(v);
                let wedge = TAU / f64::from(self.params.folds.max(2));
                let mut a = ((v.atan2(u) % wedge) + wedge) % wedge;
                if a > wedge / 2.0 {
                    a = wedge - a;
                }
                (u, v) = (a.cos() * r, a.sin() * r);
            }
        }
        (u + 0.5, v + 0.5)
    }

    fn level(&self, nx: f64, ny: f64) -> f64 {
        let p = self.params;
        let (u, v) = self.fold(nx, ny);
        let su = u * p.zoom;
        let sv = v * p.zoom * self.aspect;
        let wx = fbm(su + 5.2, sv + 1.3, self.seed.wrapping_add(11), 2);
        let wy = fbm(su + 9.1, sv + 7.7, self.seed.wrapping_add(29), 2);
        let f = fbm(
            su + p.warp * (wx - 0.5) * 2.0,
            sv + p.warp * (wy - 0.5) * 2.0,
            self.seed,
            p.oct,
        );
        ((f - 0.5) * p.contrast + 0.5).clamp(0.0, 1.0)
    }
}

struct Dot {
    x: f64,
    y: f64,
    r: f64,
    ink: [u8; 3],
}

fn dots(params: &StippleParams, palette: &Palette, seed: u32, page_h: f64) -> Vec<Dot> {
    let aspect = page_h / PAGE;
    let field = Field {
        params,
        seed,
        aspect,
    };
    let hex = params.lattices == Lattice::Hex;
    let cols = params.res.max(2);
    let stretch = if hex { 1.1547 } else { 1.0 };
    let rows = (round_half_up(f64::from(cols) * aspect * stretch) as u32).max(2);
    let (cw, ch) = (PAGE / f64::from(cols), page_h / f64::from(rows));
    let cell = cw.min(ch);
    let eps = 1.0 / f64::from(cols.max(rows));
    let contour = params.modes_dot == DotMode::Contour;
    let mut points = Vec::new();
    let (mut low, mut high, mut g_low, mut g_high) = (1.0_f64, 0.0_f64, 1.0_f64, 0.0_f64);
    for j in 0..rows as i32 {
        for i in 0..cols as i32 {
            let offset = if hex && j & 1 == 1 { 0.5 } else { 0.0 };
            let cx = (f64::from(i) + 0.5 + offset) * cw;
            let cy = (f64::from(j) + 0.5) * ch;
            if cx > PAGE {
                continue;
            }
            let (nx, ny) = (cx / PAGE, cy / page_h);
            let level = field.level(nx, ny);
            low = low.min(level);
            high = high.max(level);
            let mut g = 0.0;
            if contour {
                let gx = field.level(nx + eps, ny) - field.level(nx - eps, ny);
                let gy = field.level(nx, ny + eps) - field.level(nx, ny - eps);
                g = gx.hypot(gy);
                g_low = g_low.min(g);
                g_high = g_high.max(g);
            }
            points.push((i, j, cx, cy, level, g));
        }
    }
    let span = (high - low).max(1e-6);
    let g_span = (g_high - g_low).max(1e-6);
    let inks = palette.inks();
    let n = inks.len();
    let mut dots = Vec::new();
    for (i, j, mut cx, mut cy, level, g) in points {
        let v = (level - low) / span;
        let (keep, r) = if contour {
            let g = ((g - g_low) / g_span).clamp(0.0, 1.0);
            let keep = hash(i, j, 0, 3, seed.wrapping_add(61)) < params.loose + params.edge * g;
            let r = cell * 0.5 * params.dot * (1.0 - params.vary * 0.5 + params.vary * 0.5 * g);
            (keep, r)
        } else {
            let cut = params.cut;
            let q = (v - cut) / (1.0 - cut).max(0.001);
            let r = cell * 0.5 * params.dot * (1.0 - params.vary + params.vary * q.clamp(0.0, 1.0));
            (v > cut, r)
        };
        if !keep || r <= 0.05 {
            continue;
        }
        if params.jit > 0.0 {
            cx += (hash(i, j, 0, 7, seed.wrapping_add(3)) - 0.5) * cw * params.jit;
            cy += (hash(i, j, 0, 8, seed.wrapping_add(4)) - 0.5) * ch * params.jit;
        }
        let ink =
            if params.acc_rate > 0.0 && hash(i, j, 0, 9, seed.wrapping_add(17)) < params.acc_rate {
                ACCENT
            } else {
                inks[(n - 1).min((v * n as f64).floor() as usize)]
            };
        dots.push(Dot {
            x: cx,
            y: cy,
            r,
            ink,
        });
    }
    dots
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &StippleParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &StippleParams, palette: &Palette, seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let k = width / PAGE;
    surface.fill(GROUND);
    for dot in dots(params, palette, seed, PAGE * height / width) {
        match params.shapes {
            DotShape::Circle => surface.fill_circle(dot.x * k, dot.y * k, dot.r * k, dot.ink),
            DotShape::Square => surface.fill_box(
                (dot.x - dot.r) * k,
                (dot.y - dot.r) * k,
                dot.r * 2.0 * k,
                dot.r * 2.0 * k,
                dot.ink,
            ),
        }
    }
}
