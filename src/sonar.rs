use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, fbm, hash, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "sonar";
pub(crate) const FRAMES: u32 = 24;
pub(crate) const FPS: u32 = 10;
pub(crate) const MAX_INKS: Option<usize> = None;
const AMOUNT: f64 = 0.6;

const DEFAULT_PALETTE: [&str; 6] = [
    "#0a0f1c", "#3ddc97", "#4361ee", "#ffd166", "#ef476f", "#f1faee",
];

const LEVEL: Param = Param {
    taste: (25, 70),
    ..Param::new(SLUG, "level", 0, 100, 100)
};
const SCALE: Param = Param::new(SLUG, "scale", 10, 100, 10);
const WARP: Param = Param::new(SLUG, "warp", 0, 100, 100);
const GRID: Param = Param {
    step: 2,
    taste: (140, 300),
    ..Param::new(SLUG, "grid", 40, 320, 1)
};
const DEPTH: Param = Param::new(SLUG, "depth", 0, 100, 100);
const FRINGE: Param = Param::new(SLUG, "fringe", 0, 100, 100);
const SPARK: Param = Param::new(SLUG, "spark", 0, 100, 100);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SonarParams {
    level: f64,
    scale: f64,
    warp: f64,
    grid: u32,
    depth: f64,
    fringe: f64,
    spark: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for SonarParams {
    fn default() -> Self {
        Self {
            level: 0.5,
            scale: 2.2,
            warp: 0.4,
            grid: 150,
            depth: 0.7,
            fringe: 0.45,
            spark: 0.6,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl SonarParams {
    pub fn level(&self) -> f64 {
        self.level
    }

    pub fn set_level(&mut self, level: f64) -> Result<(), Error> {
        self.level = LEVEL.check(level)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn grid(&self) -> u32 {
        self.grid
    }

    pub fn set_grid(&mut self, grid: u32) -> Result<(), Error> {
        GRID.check(f64::from(grid))?;
        self.grid = grid;
        Ok(())
    }

    pub fn depth(&self) -> f64 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: f64) -> Result<(), Error> {
        self.depth = DEPTH.check(depth)?;
        Ok(())
    }

    pub fn fringe(&self) -> f64 {
        self.fringe
    }

    pub fn set_fringe(&mut self, fringe: f64) -> Result<(), Error> {
        self.fringe = FRINGE.check(fringe)?;
        Ok(())
    }

    pub fn spark(&self) -> f64 {
        self.spark
    }

    pub fn set_spark(&mut self, spark: f64) -> Result<(), Error> {
        self.spark = SPARK.check(spark)?;
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
    level: f64,
    scale: f64,
    warp: f64,
    grid: u32,
    depth: f64,
    fringe: f64,
    spark: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SonarParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SonarParams::default();
    params.set_level(raw.level)?;
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
    params.set_grid(raw.grid)?;
    params.set_depth(raw.depth)?;
    params.set_fringe(raw.fringe)?;
    params.set_spark(raw.spark)?;
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

pub(crate) const PARAMS: &[Param] = &[LEVEL, SCALE, WARP, GRID, DEPTH, FRINGE, SPARK];

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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SonarParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SonarParams {
        level: pick(&LEVEL),
        scale: pick(&SCALE),
        warp: pick(&WARP),
        grid: pick(&GRID) as u32,
        depth: pick(&DEPTH),
        fringe: pick(&FRINGE),
        spark: pick(&SPARK),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

const WATER: usize = 0;
const LAND: usize = 1;
const SPECK_A: usize = 2;
const SPECK_B: usize = 3;

pub(crate) fn render(
    surface: &mut Surface,
    params: &SonarParams,
    palette: &Palette,
    tool_seed: u32,
    t: u32,
) {
    paint(surface, params, palette, tool_seed, t);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &SonarParams, palette: &Palette, tool_seed: u32, t: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let cols = params.grid.max(12);
    let cell_width = width / f64::from(cols);
    let rows = round_half_up(height / cell_width).max(6.0) as u32;
    let cell_height = height / f64::from(rows);
    let aspect = width / height;
    let scale = params.scale.max(0.5);
    let phase = f64::from(t) / f64::from(FRAMES);
    let tide = (phase * std::f64::consts::TAU).sin() * 0.1 * AMOUNT;
    let level = (params.level * 0.7 + 0.15).clamp(0.05, 0.95) + tide;
    let band = 0.012 + params.fringe * 0.07;

    surface.fill(palette.ink(WATER));
    for row in 0..rows {
        for col in 0..cols {
            let u = (f64::from(col) + 0.5) / f64::from(cols);
            let v = (f64::from(row) + 0.5) / f64::from(rows);
            let (x, y) = (u * scale * aspect, v * scale);
            let push_x = fbm(x * 0.6 + 11.3, y * 0.6 + 3.7, tool_seed.wrapping_add(7), 3) - 0.5;
            let push_y = fbm(x * 0.6 + 5.1, y * 0.6 + 19.9, tool_seed.wrapping_add(13), 3) - 0.5;
            let field = fbm(
                x + push_x * params.warp * 2.4,
                y + push_y * params.warp * 2.4,
                tool_seed,
                4,
            );
            let above = field - level;
            let dot = |surface: &mut Surface, side: f64, ink: usize| {
                let corner = |cell: u32, cell_size: f64| {
                    round_half_up(f64::from(cell) * cell_size + (cell_size - side) / 2.0) as i32
                };
                let side_px = round_half_up(side).max(1.0) as u32;
                surface.fill_rect(
                    corner(col, cell_width),
                    corner(row, cell_height),
                    side_px,
                    side_px,
                    palette.ink(ink),
                );
            };
            let (col_i, row_i) = (col as i32, row as i32);
            if above > band {
                let inland = ((above - band) / (0.2 * (1.05 - params.depth * 0.85))).min(1.0);
                dot(surface, cell_width * (0.3 + 0.68 * inland), LAND);
            } else if above > -band {
                let near = 1.0 - above.abs() / band;
                if hash(col_i, row_i, tool_seed.wrapping_add(53))
                    < near * near * params.spark * 1.35
                {
                    let ink = if hash(col_i, row_i, tool_seed.wrapping_add(59)) < 0.5 {
                        SPECK_A
                    } else {
                        SPECK_B
                    };
                    dot(surface, cell_width * (0.34 + 0.42 * near), ink);
                }
            }
        }
    }
}
