use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, hash, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "relief";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 6;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#1f1f1f", "#f4d35e", "#ee964b", "#f95738", "#0d3b66", "#faf0ca",
];

const SCALE: Param = Param {
    taste: (12, 120),
    ..Param::new(SLUG, "scale", 12, 180, 1)
};
const REPEAT: Param = Param::new(SLUG, "repeat", 1, 12, 1);
const VARIETY: Param = Param::new(SLUG, "variety", 0, 100, 100);
const SOLIDS: Param = Param {
    taste: (0, 75),
    ..Param::new(SLUG, "solids", 0, 100, 100)
};
const RELIEF: Param = Param::new(SLUG, "relief", 0, 100, 100);
const ACCENT: Param = Param::new(SLUG, "accent", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[SCALE, REPEAT, VARIETY, SOLIDS, RELIEF, ACCENT];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReliefParams {
    scale: u32,
    repeat: u32,
    variety: f64,
    solids: f64,
    relief: f64,
    accent: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for ReliefParams {
    fn default() -> Self {
        Self {
            scale: 89,
            repeat: 3,
            variety: 0.38,
            solids: 0.14,
            relief: 0.9,
            accent: 0.34,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl ReliefParams {
    pub fn scale(&self) -> u32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: u32) -> Result<(), Error> {
        SCALE.check(f64::from(scale))?;
        self.scale = scale;
        Ok(())
    }

    pub fn repeat(&self) -> u32 {
        self.repeat
    }

    pub fn set_repeat(&mut self, repeat: u32) -> Result<(), Error> {
        REPEAT.check(f64::from(repeat))?;
        self.repeat = repeat;
        Ok(())
    }

    pub fn variety(&self) -> f64 {
        self.variety
    }

    pub fn set_variety(&mut self, variety: f64) -> Result<(), Error> {
        self.variety = VARIETY.check(variety)?;
        Ok(())
    }

    pub fn solids(&self) -> f64 {
        self.solids
    }

    pub fn set_solids(&mut self, solids: f64) -> Result<(), Error> {
        self.solids = SOLIDS.check(solids)?;
        Ok(())
    }

    pub fn relief(&self) -> f64 {
        self.relief
    }

    pub fn set_relief(&mut self, relief: f64) -> Result<(), Error> {
        self.relief = RELIEF.check(relief)?;
        Ok(())
    }

    pub fn accent(&self) -> f64 {
        self.accent
    }

    pub fn set_accent(&mut self, accent: f64) -> Result<(), Error> {
        self.accent = ACCENT.check(accent)?;
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
    scale: u32,
    repeat: u32,
    variety: f64,
    solids: f64,
    relief: f64,
    accent: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<ReliefParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = ReliefParams::default();
    params.set_scale(raw.scale)?;
    params.set_repeat(raw.repeat)?;
    params.set_variety(raw.variety)?;
    params.set_solids(raw.solids)?;
    params.set_relief(raw.relief)?;
    params.set_accent(raw.accent)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> ReliefParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    ReliefParams {
        scale: pick(&SCALE) as u32,
        repeat: pick(&REPEAT) as u32,
        variety: pick(&VARIETY),
        solids: pick(&SOLIDS),
        relief: pick(&RELIEF),
        accent: pick(&ACCENT),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &ReliefParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

struct Cube {
    rot: usize,
    solid: bool,
    tones: [[u8; 3]; 3],
}

fn paint(surface: &mut Surface, params: &ReliefParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let sq3 = 3.0_f64.sqrt();
    let repeat = params.repeat.max(1);
    let turns = round_half_up(1.0 + params.variety * 2.0).max(1.0);
    let (lead, accents) = palette.inks().split_first().expect("a Palette has inks");
    let cubes: Vec<Cube> = (0..repeat)
        .flat_map(|j| (0..repeat).map(move |i| (i as i32, j as i32)))
        .map(|(i, j)| {
            let draw = |offset: u32| hash(i, j, tool_seed.wrapping_add(offset));
            let rot = (draw(11) * turns).floor() as usize % 3;
            let solid = draw(29) < params.solids;
            let ink = if !accents.is_empty() && draw(41) < params.accent {
                accents[(draw(53) * accents.len() as f64).floor() as usize]
            } else {
                *lead
            };
            Cube {
                rot,
                solid,
                tones: tones(ink, params.relief),
            }
        })
        .collect();

    let want = f64::from(params.scale).max(6.0);
    let mut cubes_x = round_half_up(w / (sq3 * want)).max(1.0) as u32;
    let mut cubes_y = round_half_up(h / (1.5 * want)).max(1.0) as u32;
    let p = repeat.min(cubes_x);
    let q = repeat.min(cubes_y);
    cubes_x = cubes_x.div_ceil(p) * p;
    cubes_y = cubes_y.div_ceil(q) * q;
    if cubes_y % 2 == 1 {
        cubes_y += q;
    }
    let (p, q, stride) = (p as i32, q as i32, repeat as i32);
    let sx = w / (sq3 * f64::from(cubes_x));
    let sy = h / (1.5 * f64::from(cubes_y));
    let (hx, hy) = (sq3 / 2.0 * sx, 0.5 * sy);

    surface.fill(cubes[0].tones[2]);
    for j in -1..cubes_y as i32 + 2 {
        for i in -1..cubes_x as i32 + 2 {
            let shift = if j & 1 == 1 { 0.5 } else { 0.0 };
            let cx = (f64::from(i) + shift) * sq3 * sx;
            let cy = f64::from(j) * 1.5 * sy;
            let cube = &cubes[(j.rem_euclid(q) * stride + i.rem_euclid(p)) as usize];
            let faces = [
                [
                    (cx, cy - sy),
                    (cx + hx, cy - hy),
                    (cx, cy),
                    (cx - hx, cy - hy),
                ],
                [
                    (cx, cy),
                    (cx + hx, cy - hy),
                    (cx + hx, cy + hy),
                    (cx, cy + sy),
                ],
                [
                    (cx, cy),
                    (cx - hx, cy - hy),
                    (cx - hx, cy + hy),
                    (cx, cy + sy),
                ],
            ];
            for (k, corners) in faces.iter().enumerate() {
                let tone = if cube.solid {
                    cube.tones[1]
                } else {
                    cube.tones[(k + cube.rot) % 3]
                };
                let mut path = Path2D::default();
                path.move_to(corners[0].0, corners[0].1);
                for &(x, y) in &corners[1..] {
                    path.line_to(x, y);
                }
                path.close();
                surface.fill_even_odd(&path, tone);
                surface.stroke(&path, tone, 1.0);
            }
        }
    }
}

fn tones(ink: [u8; 3], relief: f64) -> [[u8; 3]; 3] {
    let (h, s, l) = to_hsl(ink);
    let base = l.clamp(0.38, 0.62);
    let sat = s.clamp(0.55, 1.0);
    [
        from_hsl(h + relief * 26.0, sat, (base + relief * 0.22).min(0.92)),
        from_hsl(h, sat, base),
        from_hsl(
            h - relief * 22.0,
            (sat * 1.06).min(1.0),
            (base - relief * 0.16).max(0.16),
        ),
    ]
}

fn to_hsl(ink: [u8; 3]) -> (f64, f64, f64) {
    let [r, g, b] = ink.map(|c| f64::from(c) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    let l = (max + min) / 2.0;
    if d == 0.0 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn from_hsl(h: f64, s: f64, l: f64) -> [u8; 3] {
    let h = (h % 360.0 + 360.0) % 360.0;
    let (s, l) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
    let a = s * l.min(1.0 - l);
    [0.0, 8.0, 4.0].map(|n: f64| {
        let k = (n + h / 30.0) % 12.0;
        let v = l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        round_half_up(v * 255.0) as u8
    })
}
