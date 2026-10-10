use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "sampler";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 6;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 7] = [
    "#101010", "#d6ff3c", "#6f6a1c", "#ff7a1f", "#f4786e", "#c9c4ca", "#a2218e",
];

const GRID: Param = Param::new(SLUG, "grid", 4, 48, 1);
const BANDS: Param = Param::new(SLUG, "bands", 1, 14, 1);
const MIX: Param = Param::new(SLUG, "mix", 0, 100, 100);
const TURN: Param = Param::new(SLUG, "turn", 0, 100, 100);
const DENSITY: Param = Param {
    taste: (55, 100),
    ..Param::new(SLUG, "density", 10, 100, 100)
};
const WEIGHT: Param = Param::new(SLUG, "weight", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[GRID, BANDS, MIX, TURN, DENSITY, WEIGHT];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SamplerParams {
    grid: u32,
    bands: u32,
    mix: f64,
    turn: f64,
    density: f64,
    weight: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            grid: 16,
            bands: 6,
            mix: 0.55,
            turn: 0.75,
            density: 0.95,
            weight: 0.5,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl SamplerParams {
    pub fn grid(&self) -> u32 {
        self.grid
    }

    pub fn set_grid(&mut self, grid: u32) -> Result<(), Error> {
        GRID.check(f64::from(grid))?;
        self.grid = grid;
        Ok(())
    }

    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn mix(&self) -> f64 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f64) -> Result<(), Error> {
        self.mix = MIX.check(mix)?;
        Ok(())
    }

    pub fn turn(&self) -> f64 {
        self.turn
    }

    pub fn set_turn(&mut self, turn: f64) -> Result<(), Error> {
        self.turn = TURN.check(turn)?;
        Ok(())
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) -> Result<(), Error> {
        self.density = DENSITY.check(density)?;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
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
    grid: u32,
    bands: u32,
    mix: f64,
    turn: f64,
    density: f64,
    weight: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<SamplerParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = SamplerParams::default();
    params.set_grid(raw.grid)?;
    params.set_bands(raw.bands)?;
    params.set_mix(raw.mix)?;
    params.set_turn(raw.turn)?;
    params.set_density(raw.density)?;
    params.set_weight(raw.weight)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> SamplerParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    SamplerParams {
        grid: pick(&GRID) as u32,
        bands: pick(&BANDS) as u32,
        mix: pick(&MIX),
        turn: pick(&TURN),
        density: pick(&DENSITY),
        weight: pick(&WEIGHT),
        ..SamplerParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &SamplerParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Tile {
    Triangle,
    Diagonal,
    Bar,
    Elbow,
    Step,
    Half,
    Notch,
    Arrow,
}

const TILES: [Tile; 8] = [
    Tile::Triangle,
    Tile::Diagonal,
    Tile::Bar,
    Tile::Elbow,
    Tile::Step,
    Tile::Half,
    Tile::Notch,
    Tile::Arrow,
];

impl Tile {
    fn points(self) -> &'static [(f64, f64)] {
        match self {
            Self::Triangle => &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            Self::Diagonal => &[
                (0.0, 0.34),
                (0.66, 0.0),
                (1.0, 0.0),
                (1.0, 0.66),
                (0.34, 1.0),
                (0.0, 1.0),
            ],
            Self::Bar => &[(0.0, 0.3), (1.0, 0.3), (1.0, 0.7), (0.0, 0.7)],
            Self::Elbow => &[
                (0.0, 0.3),
                (0.7, 0.3),
                (0.7, 1.0),
                (0.3, 1.0),
                (0.3, 0.7),
                (0.0, 0.7),
            ],
            Self::Step => &[
                (0.0, 0.5),
                (0.5, 0.5),
                (0.5, 0.0),
                (1.0, 0.0),
                (1.0, 0.5),
                (0.5, 0.5),
                (0.5, 1.0),
                (0.0, 1.0),
            ],
            Self::Half => &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.5), (0.0, 0.5)],
            Self::Notch => &[
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.5, 1.0),
                (0.5, 0.5),
                (0.0, 0.5),
            ],
            Self::Arrow => &[
                (0.0, 0.0),
                (0.5, 0.5),
                (0.0, 1.0),
                (0.34, 1.0),
                (0.84, 0.5),
                (0.34, 0.0),
            ],
        }
    }
}

struct Band {
    y0: u32,
    y1: u32,
    kinds: Vec<Tile>,
    a: [u8; 3],
    b: [u8; 3],
    seed: u32,
}

fn hash(x: u32, y: u32, s: u32) -> f64 {
    let mut n = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(s.wrapping_mul(1_274_126_177));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

fn pick<T: Copy>(items: &[T], draw: f64) -> T {
    items[(draw * items.len() as f64) as usize]
}

fn luma([r, g, b]: [u8; 3]) -> f64 {
    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114
}

fn bands(p: &SamplerParams, rows: u32, inks: &[[u8; 3]], seed: u32) -> (Vec<Band>, [u8; 3]) {
    let ground = inks
        .iter()
        .copied()
        .reduce(|dark, ink| if luma(ink) < luma(dark) { ink } else { dark })
        .expect("a Palette has an ink");
    let tile_inks: Vec<[u8; 3]> = inks.iter().copied().filter(|&c| c != ground).collect();
    let tile_inks = if tile_inks.is_empty() {
        inks.to_vec()
    } else {
        tile_inks
    };
    let want = f64::from(p.bands);
    let y0 = (hash(0, 71, seed.wrapping_add(3)) * f64::from(rows)) as u32;
    let mut out = Vec::new();
    let (mut y, mut i) = (y0, 0u32);
    while y - y0 < rows {
        let grown = round_half_up(
            f64::from(rows) / want * (0.55 + hash(i, 11, seed.wrapping_add(13)) * 0.95),
        )
        .max(1.0) as u32;
        let h = grown.min(rows - (y - y0));
        let nk = 1 + (hash(i, 17, seed.wrapping_add(19)) * (1.0 + p.mix * 2.2)) as u32;
        let kinds = (0..nk)
            .map(|k| pick(&TILES, hash(i * 7 + k, 23, seed.wrapping_add(29))))
            .collect();
        let a = pick(&tile_inks, hash(i, 31, seed.wrapping_add(37)));
        let others: Vec<[u8; 3]> = tile_inks.iter().copied().filter(|&c| c != a).collect();
        let b = if others.is_empty() {
            ground
        } else {
            pick(&others, hash(i, 41, seed.wrapping_add(43)))
        };
        out.push(Band {
            y0: y,
            y1: y + h,
            kinds,
            a,
            b,
            seed: seed.wrapping_add(i.wrapping_mul(257)),
        });
        y += h;
        i += 1;
    }
    (out, ground)
}

fn paint(surface: &mut Surface, p: &SamplerParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let cols = p.grid;
    let rows = round_half_up(f64::from(cols) * h / w).max(3.0) as u32;
    let (cw, ch) = (w / f64::from(cols), h / f64::from(rows));
    let (bands, ground) = bands(p, rows, palette.inks(), tool_seed);

    surface.fill(ground);
    for band in &bands {
        let s = band.seed;
        for ry in band.y0..band.y1 {
            let y = ry % rows;
            for x in 0..cols {
                if hash(x * 3 + 1, ry * 5 + 7, s) > p.density {
                    continue;
                }
                let kind = band.kinds[(hash(x, ry, s.wrapping_add(61)) * band.kinds.len() as f64)
                    as usize
                    % band.kinds.len()];
                let rq = hash(x * 11, ry * 13, s.wrapping_add(67));
                let quarter = (rq * 4.0 * (0.25 + p.turn * 0.75)) as u32 & 3;
                let ink =
                    if hash(x * 17, ry * 19, s.wrapping_add(73)) < 0.5 + 0.3 * (p.weight - 0.5) {
                        band.a
                    } else {
                        band.b
                    };
                let transform = Transform::IDENTITY
                    .translate(f64::from(x) * cw, f64::from(y) * ch)
                    .scale(cw, ch)
                    .translate(0.5, 0.5)
                    .rotate(f64::from(quarter) * PI / 2.0)
                    .translate(-0.5, -0.5);
                let mut path = Path2D::default();
                let points = kind.points();
                path.move_to(points[0].0, points[0].1);
                for &(px, py) in &points[1..] {
                    path.line_to(px, py);
                }
                path.close();
                surface.fill_transformed(&path, ink, transform);
            }
        }
    }
}
