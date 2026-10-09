use serde::{Deserialize, Serialize};

use crate::surface::Surface;
use crate::{Error, Palette};

pub(crate) const SLUG: &str = "sonar";
pub(crate) const FRAMES: u32 = 24;
pub(crate) const FPS: u32 = 10;
const AMOUNT: f64 = 0.6;

const DEFAULT_PALETTE: [&str; 6] = [
    "#0a0f1c", "#3ddc97", "#4361ee", "#ffd166", "#ef476f", "#f1faee",
];

struct Param {
    id: &'static str,
    min: u32,
    max: u32,
    step: u32,
    unit: u32,
    taste: (u32, u32),
}

impl Param {
    const fn unit(id: &'static str, taste: (u32, u32)) -> Self {
        Self {
            id,
            min: 0,
            max: 100,
            step: 1,
            unit: 100,
            taste,
        }
    }

    fn deal(&self, draw: u64) -> f64 {
        let (low, high) = self.taste;
        let steps = u64::from((high - low) / self.step + 1);
        let ticks = low + (draw % steps) as u32 * self.step;
        f64::from(ticks) / f64::from(self.unit)
    }

    fn check(&self, value: f64) -> Result<f64, Error> {
        let (min, max) = (
            f64::from(self.min) / f64::from(self.unit),
            f64::from(self.max) / f64::from(self.unit),
        );
        if (min..=max).contains(&value) {
            return Ok(value);
        }
        Err(Error::OutOfRange {
            tool: SLUG,
            param: self.id,
            value,
            min,
            max,
        })
    }
}

const LEVEL: Param = Param::unit("level", (25, 70));
const SCALE: Param = Param {
    id: "scale",
    min: 10,
    max: 100,
    step: 1,
    unit: 10,
    taste: (10, 100),
};
const WARP: Param = Param::unit("warp", (0, 100));
const GRID: Param = Param {
    id: "grid",
    min: 40,
    max: 320,
    step: 2,
    unit: 1,
    taste: (140, 300),
};
const DEPTH: Param = Param::unit("depth", (0, 100));
const FRINGE: Param = Param::unit("fringe", (0, 100));
const SPARK: Param = Param::unit("spark", (0, 100));

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SonarParams {
    level: f64,
    scale: f64,
    warp: f64,
    grid: u32,
    depth: f64,
    fringe: f64,
    spark: f64,
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
    Ok(params)
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64) -> SonarParams {
    let pick = |param: &Param| param.deal(draw(param.id));
    SonarParams {
        level: pick(&LEVEL),
        scale: pick(&SCALE),
        warp: pick(&WARP),
        grid: pick(&GRID) as u32,
        depth: pick(&DEPTH),
        fringe: pick(&FRINGE),
        spark: pick(&SPARK),
    }
}

const WATER: usize = 0;
const LAND: usize = 1;
const SPECK_A: usize = 2;
const SPECK_B: usize = 3;

pub(crate) fn paint(
    surface: &mut Surface,
    params: &SonarParams,
    palette: &Palette,
    tool_seed: u32,
    t: u32,
) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let cols = params.grid.max(12);
    let cell_width = width / f64::from(cols);
    let rows = round_half_up(height / cell_width).max(6.0) as u32;
    let cell_height = height / f64::from(rows);
    let aspect = width / height;
    let scale = params.scale.max(0.5);
    let cycles = round_half_up(AMOUNT * 2.0).max(1.0);
    let phase = f64::from(t) / f64::from(FRAMES);
    let tide = (phase * std::f64::consts::TAU * cycles).sin() * 0.1 * AMOUNT.min(1.6);
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

fn round_half_up(value: f64) -> f64 {
    let floor = value.floor();
    if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    }
}

fn hash(x: i32, y: i32, seed: u32) -> f64 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(1_274_126_177));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

fn value_noise(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash(xi.wrapping_add(dx), yi.wrapping_add(dy), seed);
    (corner(0, 0) * (1.0 - u) + corner(1, 0) * u) * (1.0 - v)
        + (corner(0, 1) * (1.0 - u) + corner(1, 1) * u) * v
}

fn fbm(x: f64, y: f64, seed: u32, octaves: u32) -> f64 {
    let (mut sum, mut weight, mut total, mut frequency) = (0.0, 0.5, 0.0, 1.0);
    for octave in 0..octaves {
        sum += weight
            * value_noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 131),
            );
        total += weight;
        frequency *= 2.0;
        weight *= 0.5;
    }
    sum / total
}
