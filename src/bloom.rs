use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, fbm, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "bloom";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#1b2a49", "#2e5c8a", "#3f9bc4", "#7fd1d8", "#beefdc", "#f3fbeb",
];

const COLS: Param = Param::new(SLUG, "cols", 8, 96, 1);
const RINGS: Param = Param {
    taste: (4, 14),
    ..Param::new(SLUG, "rings", 1, 14, 1)
};
const WARP: Param = Param::new(SLUG, "warp", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);
const STEPS: Param = Param {
    taste: (2, 8),
    ..Param::new(SLUG, "steps", 2, 14, 1)
};
const CALM: Param = Param {
    taste: (0, 50),
    ..Param::new(SLUG, "calm", 0, 100, 100)
};

pub(crate) const PARAMS: &[Param] = &[COLS, RINGS, WARP, STEPS, CALM];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BloomParams {
    cols: u32,
    rings: u32,
    warp: f64,
    grain: f64,
    steps: u32,
    calm: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain_pass: Grain,
}

impl Default for BloomParams {
    fn default() -> Self {
        Self {
            cols: 40,
            rings: 7,
            warp: 0.5,
            grain: 0.3,
            steps: 6,
            calm: 0.35,
            dither: Dither::default(),
            grain_pass: Grain::default(),
        }
    }
}

impl BloomParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn rings(&self) -> u32 {
        self.rings
    }

    pub fn set_rings(&mut self, rings: u32) -> Result<(), Error> {
        RINGS.check(f64::from(rings))?;
        self.rings = rings;
        Ok(())
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn steps(&self) -> u32 {
        self.steps
    }

    pub fn set_steps(&mut self, steps: u32) -> Result<(), Error> {
        STEPS.check(f64::from(steps))?;
        self.steps = steps;
        Ok(())
    }

    pub fn calm(&self) -> f64 {
        self.calm
    }

    pub fn set_calm(&mut self, calm: f64) -> Result<(), Error> {
        self.calm = CALM.check(calm)?;
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
    cols: u32,
    rings: u32,
    warp: f64,
    grain: f64,
    steps: u32,
    calm: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<BloomParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = BloomParams::default();
    params.set_cols(raw.cols)?;
    params.set_rings(raw.rings)?;
    params.set_warp(raw.warp)?;
    params.set_grain(raw.grain)?;
    params.set_steps(raw.steps)?;
    params.set_calm(raw.calm)?;
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
        COLS.parameter(),
        RINGS.parameter(),
        WARP.parameter(),
        GRAIN.parameter(),
        STEPS.parameter(),
        CALM.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> BloomParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    BloomParams {
        cols: pick(&COLS) as u32,
        rings: pick(&RINGS) as u32,
        warp: pick(&WARP),
        steps: pick(&STEPS) as u32,
        calm: pick(&CALM),
        ..BloomParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &BloomParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain_pass, &params.dither);
}

fn paint(surface: &mut Surface, params: &BloomParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (surface.width(), surface.height());
    let (w, h) = (f64::from(width), f64::from(height));
    let cols = params.cols.max(4);
    let rows = (round_half_up(h / (w / f64::from(cols))) as u32).max(3);
    let aspect = w / h;
    let steps = i64::from(params.steps.max(2));
    let calm = params.calm * 0.55;
    let corner = (0.5 * aspect).hypot(0.5);
    let edges_x = edges(cols, width);
    let edges_y = edges(rows, height);
    let inks = palette.len() as i64;

    for j in 0..rows {
        for i in 0..cols {
            let mi = i.min(cols - 1 - i);
            let mj = j.min(rows - 1 - j);
            let u = (f64::from(mi) + 0.5) / f64::from(cols);
            let v = (f64::from(mj) + 0.5) / f64::from(rows);
            let mut r = ((0.5 - u) * aspect).hypot(0.5 - v) / corner;
            let slow = fbm(u * 3.2 + 3.1, v * 3.2 + 7.7, tool_seed.wrapping_add(11), 3) - 0.5;
            let fast = fbm(u * 9.5 + 13.7, v * 9.5 + 2.3, tool_seed.wrapping_add(17), 2) - 0.5;
            r += (slow * 1.35 + fast * 0.5) * params.warp;
            let level = if r <= calm {
                0.0
            } else {
                let rough = (fbm(
                    u * 21.0 + 1.3,
                    v * 21.0 + 9.1,
                    tool_seed.wrapping_add(29),
                    2,
                ) - 0.5)
                    * params.grain
                    * 2.4;
                (r - calm) / (1.0 - calm) * f64::from(params.rings.max(1)) + rough
            };
            let k = (level.floor() as i64).rem_euclid(steps);
            let ink = (k as f64 / steps as f64 * inks as f64).floor() as usize;
            let (x, y) = (edges_x[i as usize], edges_y[j as usize]);
            let (cell_w, cell_h) = (edges_x[i as usize + 1] - x, edges_y[j as usize + 1] - y);
            if cell_w > 0 && cell_h > 0 {
                surface.fill_rect(x as i32, y as i32, cell_w, cell_h, palette.ink(ink));
            }
        }
    }
}

fn edges(n: u32, total: u32) -> Vec<u32> {
    let step = f64::from(total) / f64::from(n);
    (0..=n)
        .map(|k| {
            let near = round_half_up(f64::from(k.min(n - k)) * step) as u32;
            if k <= n - k { near } else { total - near }
        })
        .collect()
}
