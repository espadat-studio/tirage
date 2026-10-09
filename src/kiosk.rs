use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, hash, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::text::Type;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "kiosk";
pub(crate) const FRAMES: u32 = 24;
pub(crate) const FPS: u32 = 6;
const SHUFFLE: u32 = 7919;

const DEFAULT_PALETTE: [&str; 8] = [
    "#f3efe0", "#141414", "#d7263d", "#1b4079", "#f4b942", "#3c887e", "#f26430", "#8e44ad",
];

const SPLIT: Param = Param::new(SLUG, "split", 15, 95, 100);
const RINGS: Param = Param::new(SLUG, "rings", 4, 40, 1);
const STRIPES: Param = Param::new(SLUG, "stripes", 2, 24, 1);
const GRID: Param = Param::new(SLUG, "grid", 4, 30, 1);
const BIG_SIZE: Param = Param {
    taste: (30, 63),
    ..Param::new(SLUG, "bigSize", 30, 160, 100)
};
const DENSITY: Param = Param {
    taste: (0, 50),
    ..Param::new(SLUG, "density", 0, 100, 100)
};
const SMALL_SIZE: Param = Param {
    taste: (10, 50),
    ..Param::new(SLUG, "smallSize", 10, 90, 100)
};
const SMALL: Param = Param {
    taste: (0, 50),
    ..Param::new(SLUG, "small", 0, 100, 100)
};
const BLOCKS: Param = Param::new(SLUG, "blocks", 0, 100, 100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterSet {
    #[serde(rename = "DOS")]
    Dos,
    Stipple,
    Blocks,
    Code,
    Digits,
    Runes,
}

impl CharacterSet {
    pub const ALL: &[CharacterSet] = &[
        Self::Dos,
        Self::Stipple,
        Self::Blocks,
        Self::Code,
        Self::Digits,
        Self::Runes,
    ];

    pub fn chars(self) -> &'static str {
        match self {
            Self::Dos => "0369#%&@!;,'()",
            Self::Stipple => ".,;:'\"^~*",
            Self::Blocks => "\u{2591}\u{2592}\u{2593}\u{2588}\u{25a0}\u{25aa}\u{00b7}",
            Self::Code => "/\\|_-+=<>[]{}",
            Self::Digits => "0123456789",
            Self::Runes => {
                "\u{2020}\u{2021}\u{00a7}\u{00b6}\u{00ae}\u{00a9}\u{2260}\u{221e}\u{2248}"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KioskParams {
    split: f64,
    rings: u32,
    stripes: u32,
    sets: CharacterSet,
    grid: u32,
    #[serde(rename = "bigSize")]
    big_size: f64,
    density: f64,
    #[serde(rename = "smallSize")]
    small_size: f64,
    small: f64,
    blocks: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for KioskParams {
    fn default() -> Self {
        Self {
            split: 0.62,
            rings: 15,
            stripes: 9,
            sets: CharacterSet::Dos,
            grid: 13,
            big_size: 0.5,
            density: 0.84,
            small_size: 0.44,
            small: 0.6,
            blocks: 0.62,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl KioskParams {
    pub fn split(&self) -> f64 {
        self.split
    }

    pub fn set_split(&mut self, split: f64) -> Result<(), Error> {
        self.split = SPLIT.check(split)?;
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

    pub fn stripes(&self) -> u32 {
        self.stripes
    }

    pub fn set_stripes(&mut self, stripes: u32) -> Result<(), Error> {
        STRIPES.check(f64::from(stripes))?;
        self.stripes = stripes;
        Ok(())
    }

    pub fn sets(&self) -> CharacterSet {
        self.sets
    }

    pub fn set_sets(&mut self, sets: CharacterSet) {
        self.sets = sets;
    }

    pub fn grid(&self) -> u32 {
        self.grid
    }

    pub fn set_grid(&mut self, grid: u32) -> Result<(), Error> {
        GRID.check(f64::from(grid))?;
        self.grid = grid;
        Ok(())
    }

    pub fn big_size(&self) -> f64 {
        self.big_size
    }

    pub fn set_big_size(&mut self, big_size: f64) -> Result<(), Error> {
        self.big_size = BIG_SIZE.check(big_size)?;
        Ok(())
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) -> Result<(), Error> {
        self.density = DENSITY.check(density)?;
        Ok(())
    }

    pub fn small_size(&self) -> f64 {
        self.small_size
    }

    pub fn set_small_size(&mut self, small_size: f64) -> Result<(), Error> {
        self.small_size = SMALL_SIZE.check(small_size)?;
        Ok(())
    }

    pub fn small(&self) -> f64 {
        self.small
    }

    pub fn set_small(&mut self, small: f64) -> Result<(), Error> {
        self.small = SMALL.check(small)?;
        Ok(())
    }

    pub fn blocks(&self) -> f64 {
        self.blocks
    }

    pub fn set_blocks(&mut self, blocks: f64) -> Result<(), Error> {
        self.blocks = BLOCKS.check(blocks)?;
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
    split: f64,
    rings: u32,
    stripes: u32,
    sets: CharacterSet,
    grid: u32,
    #[serde(rename = "bigSize")]
    big_size: f64,
    density: f64,
    #[serde(rename = "smallSize")]
    small_size: f64,
    small: f64,
    blocks: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<KioskParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = KioskParams::default();
    params.set_split(raw.split)?;
    params.set_rings(raw.rings)?;
    params.set_stripes(raw.stripes)?;
    params.set_sets(raw.sets);
    params.set_grid(raw.grid)?;
    params.set_big_size(raw.big_size)?;
    params.set_density(raw.density)?;
    params.set_small_size(raw.small_size)?;
    params.set_small(raw.small)?;
    params.set_blocks(raw.blocks)?;
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

pub(crate) const PARAMS: &[Param] = &[
    SPLIT, RINGS, STRIPES, GRID, BIG_SIZE, DENSITY, SMALL_SIZE, SMALL, BLOCKS,
];

pub(crate) fn parameters() -> Vec<Parameter> {
    let [split, rings, stripes, rest @ ..] = PARAMS else {
        unreachable!("kiosk has 9 ranges")
    };
    [split, rings, stripes]
        .into_iter()
        .map(Param::parameter)
        .chain([Parameter::choice(
            "sets",
            &["DOS", "Stipple", "Blocks", "Code", "Digits", "Runes"],
        )])
        .chain(rest.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> KioskParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    let sets = CharacterSet::ALL[(draw("sets") % CharacterSet::ALL.len() as u64) as usize];
    KioskParams {
        split: pick(&SPLIT),
        rings: pick(&RINGS) as u32,
        stripes: pick(&STRIPES) as u32,
        sets,
        grid: pick(&GRID) as u32,
        big_size: pick(&BIG_SIZE),
        density: pick(&DENSITY),
        small_size: pick(&SMALL_SIZE),
        small: pick(&SMALL),
        blocks: pick(&BLOCKS),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

struct Roles {
    dark: [u8; 3],
    paper: [u8; 3],
    ink: [u8; 3],
    bands: Vec<[u8; 3]>,
}

impl Roles {
    fn new(palette: &Palette) -> Self {
        let inks = palette.inks();
        let lightness =
            |[r, g, b]: [u8; 3]| f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114;
        let (mut dark, mut paper) = (inks[0], inks[0]);
        for &ink in inks {
            if lightness(ink) < lightness(dark) {
                dark = ink;
            }
            if lightness(ink) > lightness(paper) {
                paper = ink;
            }
        }
        let mut rest: Vec<_> = inks
            .iter()
            .copied()
            .filter(|&ink| ink != dark && ink != paper)
            .collect();
        if rest.is_empty() {
            rest.push(paper);
        }
        let ink = rest[0];
        let bands = if rest.len() > 1 {
            rest.split_off(1)
        } else {
            rest
        };
        Self {
            dark,
            paper,
            ink,
            bands,
        }
    }

    fn band(&self, i: usize) -> [u8; 3] {
        self.bands[i % self.bands.len()]
    }
}

pub(crate) fn paint(
    surface: &mut Surface,
    params: &KioskParams,
    palette: &Palette,
    tool_seed: u32,
    t: u32,
) {
    let roles = Roles::new(palette);
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let at = |x: i32, y: i32, salt: u32| hash(x, y, tool_seed.wrapping_add(salt));

    surface.fill_box(0.0, 0.0, width, height, roles.band(0));
    let cx = width * (-0.22 + at(1, 1, 11) * 0.30);
    let cy = height * (-0.06 + at(2, 2, 13) * 0.36);
    let rings = params.rings;
    let reach = (cx.abs().max((width - cx).abs())).hypot(cy.abs().max((height - cy).abs())) * 1.02;
    let step = reach / f64::from(rings);
    for k in (1..=rings).rev() {
        surface.fill_circle(cx, cy, f64::from(k) * step, roles.band(k as usize));
    }

    let split = round_half_up(width * params.split);
    let stripes = params.stripes as i32;
    let weights: Vec<f64> = (0..stripes).map(|i| 0.45 + at(i, 7, 31)).collect();
    let total: f64 = weights.iter().sum();
    let mut x = split;
    for (i, weight) in (0..stripes).zip(&weights) {
        let w = weight / total * (width - split);
        let (x0, x1) = (round_half_up(x), round_half_up(x + w));
        let band = (at(i, 9, 37) * roles.bands.len() as f64).floor() as usize;
        surface.fill_box(x0, 0.0, x1 - x0, height, roles.band(band));
        x += w;
    }

    let tile_width = width * 0.115;
    let left = width - tile_width;
    let tile_height = height / 8.0;
    let flip = i32::from(at(0, 19, 47) < 0.5);
    for i in 0..8 {
        if at(i, 13, 41) > params.blocks {
            continue;
        }
        let top = round_half_up(f64::from(i) * tile_height);
        let bottom = round_half_up(f64::from(i + 1) * tile_height);
        let solid = (i + flip) & 1 == 1;
        let ink = if solid { roles.dark } else { roles.paper };
        surface.fill_box(left, top, tile_width, bottom - top, ink);
        if solid {
            continue;
        }
        let pitch = tile_width / 6.0;
        for row in 0..((bottom - top) / pitch).ceil() as u32 {
            let dy = top + (f64::from(row) + 0.5) * pitch;
            if dy > bottom {
                continue;
            }
            for col in 0..6 {
                let dx = left + (f64::from(col) + 0.5) * pitch;
                surface.fill_circle(dx, dy, pitch * 0.23, roles.dark);
            }
        }
    }

    let panel = [
        width * (0.26 + at(5, 5, 61) * 0.26),
        height * (0.56 + at(6, 6, 67) * 0.22),
        width * (0.26 + at(3, 3, 53) * 0.18),
        height * (0.14 + at(4, 4, 59) * 0.11),
    ]
    .map(round_half_up);
    surface.fill_box(panel[0], panel[1], panel[2], panel[3], roles.dark);

    let text_seed = tool_seed.wrapping_add(t.wrapping_mul(SHUFFLE));
    let cols = params.grid;
    let cell_width = width / f64::from(cols);
    let rows = round_half_up(height / cell_width).max(2.0) as u32;
    let cell_height = height / f64::from(rows);
    let chars: Vec<char> = params.sets.chars().chars().collect();
    let mut text = Type::new();
    let mut put = |surface: &mut Surface, i: i32, j: i32, x: f64, y: f64, size: f64, salt: u32| {
        let draw = |s: u32| hash(i, j, text_seed.wrapping_add(salt + s));
        let ch = chars[(draw(3) * chars.len() as f64).floor() as usize];
        let ink = if draw(7) < 0.28 {
            roles.dark
        } else {
            roles.ink
        };
        text.fill_centred(surface, ch, x as f32, y as f32, to_fixed_1(size), ink);
    };
    let pass = |cutoff: f64, salt: u32| {
        (0..rows as i32)
            .flat_map(move |j| (0..cols as i32).map(move |i| (i, j)))
            .filter(move |&(i, j)| hash(i, j, text_seed.wrapping_add(salt)) <= cutoff)
    };
    for (i, j) in pass(params.density, 71) {
        let x = (f64::from(i) + 0.5) * cell_width;
        let y = (f64::from(j) + 0.52) * cell_height;
        put(surface, i, j, x, y, cell_height * params.big_size, 71);
    }
    for (i, j) in pass(params.small, 91) {
        let x = f64::from(i + 1) * cell_width;
        let y = f64::from(j + 1) * cell_height;
        put(surface, i, j, x, y, cell_height * params.small_size, 91);
    }
}

fn to_fixed_1(value: f64) -> f32 {
    let quarters = value * 4.0;
    if quarters.fract() == 0.0 && quarters as u64 % 2 == 1 {
        return ((value * 10.0).ceil() / 10.0) as f32;
    }
    format!("{value:.1}")
        .parse::<f64>()
        .expect("a formatted float parses") as f32
}

#[cfg(test)]
mod tests {
    use super::to_fixed_1;

    #[test]
    fn font_sizes_round_to_one_decimal_with_ties_up() {
        assert_eq!(to_fixed_1(20.25), 20.3);
        assert_eq!(to_fixed_1(20.75), 20.8);
        assert_eq!(to_fixed_1(0.15), 0.1);
        assert_eq!(to_fixed_1(20.869_565_2), 20.9);
        assert_eq!(to_fixed_1(18.0), 18.0);
    }
}
