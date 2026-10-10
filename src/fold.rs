use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, round_half_up, store};
use crate::param::Param;
use crate::surface::{Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "fold";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#1a1a1a", "#f5f1e8", "#ff5c39", "#3b5bff", "#ffd23f", "#7b2cbf",
];

const FOLDX: Param = Param::new(SLUG, "foldx", 20, 80, 100);
const FOLDY: Param = Param::new(SLUG, "foldy", 20, 80, 100);
const PATCHES: Param = Param::new(SLUG, "patches", 1, 16, 1);
const SIZE: Param = Param::new(SLUG, "size", 0, 100, 100);
const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const WAVE: Param = Param::new(SLUG, "wave", 0, 100, 100);
const LEVELS: Param = Param::new(SLUG, "levels", 2, 8, 1);
const TILT: Param = Param::new(SLUG, "tilt", 0, 100, 100);
const PX: Param = Param::new(SLUG, "px", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[FOLDX, FOLDY, PATCHES, SIZE, SCALE, WAVE, LEVELS, TILT, PX];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldMirror {
    #[serde(rename = "Four ways")]
    FourWays,
    Kaleidoscope,
    Across,
    Down,
    None,
}

const MIRRORS: [FoldMirror; 5] = [
    FoldMirror::FourWays,
    FoldMirror::Kaleidoscope,
    FoldMirror::Across,
    FoldMirror::Down,
    FoldMirror::None,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldKind {
    Mixed,
    Zebra,
    Staircases,
    #[serde(rename = "Dot rows")]
    DotRows,
    Bursts,
    Chevrons,
    Checks,
}

const KINDS: [FoldKind; 7] = [
    FoldKind::Mixed,
    FoldKind::Zebra,
    FoldKind::Staircases,
    FoldKind::DotRows,
    FoldKind::Bursts,
    FoldKind::Chevrons,
    FoldKind::Checks,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoldParams {
    #[serde(rename = "folds")]
    mirror: FoldMirror,
    #[serde(rename = "kinds")]
    kind: FoldKind,
    foldx: f64,
    foldy: f64,
    patches: u32,
    size: f64,
    scale: f64,
    wave: f64,
    levels: u32,
    tilt: f64,
    px: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for FoldParams {
    fn default() -> Self {
        Self {
            mirror: FoldMirror::FourWays,
            kind: FoldKind::Mixed,
            foldx: 0.5,
            foldy: 0.5,
            patches: 9,
            size: 0.5,
            scale: 0.5,
            wave: 0.5,
            levels: 5,
            tilt: 0.3,
            px: 0.35,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl FoldParams {
    pub fn mirror(&self) -> FoldMirror {
        self.mirror
    }

    pub fn set_mirror(&mut self, mirror: FoldMirror) {
        self.mirror = mirror;
    }

    pub fn kind(&self) -> FoldKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: FoldKind) {
        self.kind = kind;
    }

    pub fn foldx(&self) -> f64 {
        self.foldx
    }

    pub fn set_foldx(&mut self, foldx: f64) -> Result<(), Error> {
        self.foldx = FOLDX.check(foldx)?;
        Ok(())
    }

    pub fn foldy(&self) -> f64 {
        self.foldy
    }

    pub fn set_foldy(&mut self, foldy: f64) -> Result<(), Error> {
        self.foldy = FOLDY.check(foldy)?;
        Ok(())
    }

    pub fn patches(&self) -> u32 {
        self.patches
    }

    pub fn set_patches(&mut self, patches: u32) -> Result<(), Error> {
        PATCHES.check(f64::from(patches))?;
        self.patches = patches;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn wave(&self) -> f64 {
        self.wave
    }

    pub fn set_wave(&mut self, wave: f64) -> Result<(), Error> {
        self.wave = WAVE.check(wave)?;
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

    pub fn tilt(&self) -> f64 {
        self.tilt
    }

    pub fn set_tilt(&mut self, tilt: f64) -> Result<(), Error> {
        self.tilt = TILT.check(tilt)?;
        Ok(())
    }

    pub fn px(&self) -> f64 {
        self.px
    }

    pub fn set_px(&mut self, px: f64) -> Result<(), Error> {
        self.px = PX.check(px)?;
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
    #[serde(rename = "folds")]
    mirror: FoldMirror,
    #[serde(rename = "kinds")]
    kind: FoldKind,
    foldx: f64,
    foldy: f64,
    patches: u32,
    size: f64,
    scale: f64,
    wave: f64,
    levels: u32,
    tilt: f64,
    px: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<FoldParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = FoldParams::default();
    params.set_mirror(raw.mirror);
    params.set_kind(raw.kind);
    params.set_foldx(raw.foldx)?;
    params.set_foldy(raw.foldy)?;
    params.set_patches(raw.patches)?;
    params.set_size(raw.size)?;
    params.set_scale(raw.scale)?;
    params.set_wave(raw.wave)?;
    params.set_levels(raw.levels)?;
    params.set_tilt(raw.tilt)?;
    params.set_px(raw.px)?;
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
    [
        Parameter::choice(
            "folds",
            &["Four ways", "Kaleidoscope", "Across", "Down", "None"],
        ),
        Parameter::choice(
            "kinds",
            &[
                "Mixed",
                "Zebra",
                "Staircases",
                "Dot rows",
                "Bursts",
                "Chevrons",
                "Checks",
            ],
        ),
    ]
    .into_iter()
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> FoldParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    FoldParams {
        mirror: MIRRORS[(draw("folds") % MIRRORS.len() as u64) as usize],
        kind: KINDS[(draw("kinds") % KINDS.len() as u64) as usize],
        foldx: pick(&FOLDX),
        foldy: pick(&FOLDY),
        patches: pick(&PATCHES) as u32,
        size: pick(&SIZE),
        scale: pick(&SCALE),
        wave: pick(&WAVE),
        levels: pick(&LEVELS) as u32,
        tilt: pick(&TILT),
        px: pick(&PX),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &FoldParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Pattern {
    Zebra,
    Stairs,
    Dots,
    Burst,
    Chevron,
    Checks,
}

const MIXED: [Pattern; 6] = [
    Pattern::Zebra,
    Pattern::Stairs,
    Pattern::Dots,
    Pattern::Burst,
    Pattern::Chevron,
    Pattern::Checks,
];

struct Patch {
    pattern: Pattern,
    centre: (f64, f64),
    half: (f64, f64),
    cos: f64,
    sin: f64,
    a: [u8; 3],
    b: [u8; 3],
    freq: f64,
    oval: bool,
    phase: f64,
}

fn lum([r, g, b]: [u8; 3]) -> f64 {
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}

fn mix(a: [u8; 3], b: [u8; 3], t: f64) -> [u8; 3] {
    std::array::from_fn(|c| store(f64::from(a[c]) + (f64::from(b[c]) - f64::from(a[c])) * t))
}

fn patches(
    params: &FoldParams,
    inks: &[[u8; 3]],
    others: &[usize],
    aw: f64,
    ah: f64,
    tool_seed: u32,
) -> Vec<Patch> {
    let mut rng = XorShift::new(tool_seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let pool: &[Pattern] = match params.kind {
        FoldKind::Mixed => &MIXED,
        FoldKind::Zebra => &[Pattern::Zebra],
        FoldKind::Staircases => &[Pattern::Stairs],
        FoldKind::DotRows => &[Pattern::Dots],
        FoldKind::Bursts => &[Pattern::Burst],
        FoldKind::Chevrons => &[Pattern::Chevron],
        FoldKind::Checks => &[Pattern::Checks],
    };
    let pick = |rng: &mut XorShift| (rng.next() * others.len() as f64) as usize;
    (0..params.patches)
        .map(|_| {
            let pattern = pool[(rng.next() * pool.len() as f64) as usize];
            let cx = rng.next() * aw;
            let cy = rng.next() * ah;
            let size = (0.18 + rng.next() * 0.32) * (0.5 + params.size);
            let w = size * (0.7 + rng.next() * 0.8);
            let h = size * (0.7 + rng.next() * 0.8);
            let snap = rng.next();
            let base = if snap < 0.55 {
                0.0
            } else if snap < 0.8 {
                FRAC_PI_2
            } else {
                FRAC_PI_4
            };
            let angle = base + (rng.next() - 0.5) * params.tilt * 0.8;
            let ia = pick(&mut rng);
            let mut ib = pick(&mut rng);
            if others[ib] == others[ia] {
                ib = (ia + 1) % others.len();
            }
            let freq = (4.0 + rng.next() * 8.0) * (0.5 + params.scale);
            let oval = rng.next() >= 0.75;
            let phase = rng.next() * TAU;
            Patch {
                pattern,
                centre: (cx, cy),
                half: (w * 0.5, h * 0.5),
                cos: angle.cos(),
                sin: angle.sin(),
                a: inks[others[ia]],
                b: inks[others[ib]],
                freq,
                oval,
                phase,
            }
        })
        .collect()
}

fn even(value: f64) -> bool {
    (value.floor() as i64) & 1 == 0
}

fn shade(
    patch: &Patch,
    (u, v): (f64, f64),
    params: &FoldParams,
    dark: [u8; 3],
    light: [u8; 3],
) -> Option<[u8; 3]> {
    let (dx, dy) = (u - patch.centre.0, v - patch.centre.1);
    let lx = dx * patch.cos + dy * patch.sin;
    let ly = -dx * patch.sin + dy * patch.cos;
    let (nx, ny) = (lx / patch.half.0, ly / patch.half.1);
    let outside = if patch.oval {
        nx * nx + ny * ny > 1.0
    } else {
        !(-1.0..=1.0).contains(&nx) || !(-1.0..=1.0).contains(&ny)
    };
    if outside {
        return None;
    }
    let f = patch.freq;
    let on = match patch.pattern {
        Pattern::Zebra => {
            let pinch = 1.0 + 0.6 * ny;
            let stripe =
                (nx * pinch + params.wave * 0.35 * (ny * 3.1 + patch.phase).sin()) * f * PI;
            return Some(if stripe.sin() > 0.0 { dark } else { light });
        }
        Pattern::Stairs => {
            let q = ((nx + 1.0) * f * 0.5).floor() + ((ny + 1.0) * f * 0.5).floor();
            even(q / 2.0)
        }
        Pattern::Dots => {
            let g = f * 0.5;
            let (gx, gy) = ((nx + 1.0) * g, (ny + 1.0) * g);
            let radius = 0.18 + 0.32 * (0.5 + 0.5 * ((nx + 1.0) * 2.2 + patch.phase).sin());
            let (ddx, ddy) = (gx - gx.floor() - 0.5, gy - gy.floor() - 0.5);
            (ddx * ddx + ddy * ddy).sqrt() < radius
        }
        Pattern::Burst => {
            let tone = (nx * nx + ny * ny).sqrt().clamp(0.0, 1.0);
            return Some(mix(patch.a, patch.b, step(tone, params.levels)));
        }
        Pattern::Chevron => even((nx.abs() + ny + 1.0) * f * 0.5),
        Pattern::Checks => even(((nx + 1.0) * f * 0.5).floor() + ((ny + 1.0) * f * 0.5).floor()),
    };
    Some(if on { patch.a } else { patch.b })
}

fn step(tone: f64, levels: u32) -> f64 {
    let levels = f64::from(levels);
    (tone * levels).floor() / (levels - 1.0)
}

fn fold(x: f64, line: f64) -> f64 {
    (if x < line { line - x } else { x - line }) / line.max(1.0 - line)
}

fn paint(surface: &mut Surface, params: &FoldParams, palette: &Palette, tool_seed: u32) {
    let inks = palette.inks();
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let unit = (width * height).sqrt();
    let (aw, ah) = (width / unit, height / unit);
    let (mut dark, mut light) = (0, 0);
    for (k, &ink) in inks.iter().enumerate().skip(1) {
        if lum(ink) < lum(inks[dark]) {
            dark = k;
        }
        if lum(ink) > lum(inks[light]) {
            light = k;
        }
    }
    let mut others: Vec<usize> = (2..inks.len()).collect();
    if others.len() < 2 {
        others = (0..inks.len()).collect();
    }
    let patches = patches(params, inks, &others, aw, ah, tool_seed);
    let (burst_a, burst_b) = (inks[others[0]], inks[0]);
    let cell = (unit * (0.005 + params.px * 0.02)).max(2.0);
    let columns = (round_half_up(width / cell) as u32).max(8);
    let rows = (round_half_up(height / cell) as u32).max(8);
    let across = matches!(
        params.mirror,
        FoldMirror::Across | FoldMirror::FourWays | FoldMirror::Kaleidoscope
    );
    let down = matches!(
        params.mirror,
        FoldMirror::Down | FoldMirror::FourWays | FoldMirror::Kaleidoscope
    );
    let mut rgba = Vec::with_capacity((columns * rows * 4) as usize);
    for j in 0..rows {
        let y = (f64::from(j) + 0.5) / f64::from(rows);
        for i in 0..columns {
            let x = (f64::from(i) + 0.5) / f64::from(columns);
            let mut sx = if across { fold(x, params.foldx) } else { x };
            let mut sy = if down { fold(y, params.foldy) } else { y };
            if params.mirror == FoldMirror::Kaleidoscope && sx > sy {
                (sx, sy) = (sy, sx);
            }
            let point = (sx * aw, sy * ah);
            let ink = patches
                .iter()
                .rev()
                .find_map(|patch| shade(patch, point, params, inks[dark], inks[light]))
                .unwrap_or_else(|| {
                    let d =
                        (point.0 * point.0 + point.1 * point.1).sqrt() / (aw * aw + ah * ah).sqrt();
                    mix(burst_a, burst_b, step(d.min(0.999), params.levels))
                });
            rgba.extend([ink[0], ink[1], ink[2], 255]);
        }
    }
    surface.draw_smooth(&rgba, columns, rows, Smoothing::Nearest);
}
