use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, fbm, hash, round_half_up};
use crate::kiosk::CharacterSet;
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::text::{Face, Type};
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "atlas";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 8] = [
    "#0d2b45", "#203c56", "#544e68", "#8d697a", "#d08159", "#ffaa5e", "#ffd4a3", "#ffecd6",
];

const SCALE: Param = Param::new(SLUG, "scale", 10, 90, 10);
const WARP: Param = Param::new(SLUG, "warp", 0, 160, 100);
const BANDS: Param = Param {
    taste: (3, 14),
    ..Param::new(SLUG, "bands", 2, 14, 1)
};
const MIX: Param = Param::new(SLUG, "mix", 0, 100, 100);
const COLS: Param = Param::new(SLUG, "cols", 24, 260, 1);
const DENSITY: Param = Param::new(SLUG, "density", 0, 100, 100);
const WEIGHT: Param = Param::new(SLUG, "weight", 0, 150, 100);

pub(crate) const PARAMS: &[Param] = &[SCALE, WARP, BANDS, MIX, COLS, DENSITY, WEIGHT];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AtlasParams {
    scale: f64,
    warp: f64,
    bands: u32,
    mix: f64,
    sets: CharacterSet,
    cols: u32,
    density: f64,
    weight: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for AtlasParams {
    fn default() -> Self {
        Self {
            scale: 2.4,
            warp: 0.5,
            bands: 6,
            mix: 0.55,
            sets: CharacterSet::Dos,
            cols: 110,
            density: 0.62,
            weight: 0.5,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl AtlasParams {
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

    pub fn sets(&self) -> CharacterSet {
        self.sets
    }

    pub fn set_sets(&mut self, sets: CharacterSet) {
        self.sets = sets;
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
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
    scale: f64,
    warp: f64,
    bands: u32,
    mix: f64,
    sets: CharacterSet,
    cols: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<AtlasParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = AtlasParams::default();
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
    params.set_bands(raw.bands)?;
    params.set_mix(raw.mix)?;
    params.set_sets(raw.sets);
    params.set_cols(raw.cols)?;
    params.set_density(raw.density)?;
    params.set_weight(raw.weight)?;
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
    let [scale, warp, bands, mix, rest @ ..] = PARAMS else {
        unreachable!("atlas has 7 ranges")
    };
    [scale, warp, bands, mix]
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> AtlasParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    let sets = CharacterSet::ALL[(draw("sets") % CharacterSet::ALL.len() as u64) as usize];
    AtlasParams {
        scale: pick(&SCALE),
        warp: pick(&WARP),
        bands: pick(&BANDS) as u32,
        mix: pick(&MIX),
        sets,
        cols: pick(&COLS) as u32,
        density: pick(&DENSITY),
        weight: pick(&WEIGHT),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

struct Cell {
    ground: [u8; 3],
    ink: [u8; 3],
    ch: Option<char>,
}

fn lightness([r, g, b]: [u8; 3]) -> f64 {
    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114
}

fn grid(params: &AtlasParams, palette: &Palette, seed: u32, aspect: f64) -> (u32, u32, Vec<Cell>) {
    let cols = params.cols.max(8);
    let rows = (round_half_up(f64::from(cols) * aspect * 0.6) as u32).max(6);
    let bands = params.bands.max(2);
    let glyphs: Vec<char> = params.sets.chars().chars().collect();
    let len = glyphs.len();
    let inks = palette.inks();
    let (mut hi, mut lo) = (inks[0], inks[0]);
    for &ink in inks {
        if lightness(ink) > lightness(hi) {
            hi = ink;
        }
        if lightness(ink) < lightness(lo) {
            lo = ink;
        }
    }
    let span = round_half_up(1.0 + (params.weight / 1.5) * (len - 1) as f64).max(1.0);
    let n = f64::from(bands);
    let mut cells = Vec::with_capacity((cols * rows) as usize);
    for y in 0..rows as i32 {
        for x in 0..cols as i32 {
            let u = f64::from(x) / f64::from(cols) * params.scale;
            let v = (f64::from(y) / f64::from(rows)) * params.scale * aspect;
            let wx = u + params.warp
                * (fbm(u * 1.6 + 19.0, v * 1.6, seed.wrapping_add(41), 3) - 0.5)
                * 2.2;
            let wy = v + params.warp
                * (fbm(u * 1.6, v * 1.6 + 7.0, seed.wrapping_add(83), 3) - 0.5)
                * 2.2;
            let t =
                ((fbm(wx, wy, seed, 4) - 0.5) * (0.7 + params.mix * 1.6) + 0.5).clamp(0.0, 0.999);
            let band = (t * n).floor() as u32;
            let ground = palette.ink(band as usize);
            let ink = if (lightness(ground) - lightness(hi)).abs()
                > (lightness(ground) - lightness(lo)).abs()
            {
                hi
            } else {
                lo
            };
            let start = (f64::from(band) / n * len as f64).floor() as usize;
            let jitter = hash(x * 7 + 1, y * 13 + 3, seed.wrapping_add(band * 97));
            let on = hash(x * 3 + 5, y * 5 + 11, seed.wrapping_add(band * 57))
                < params.density * (0.35 + 0.9 * (f64::from(band + 1) / n));
            let ch = on.then(|| glyphs[(start + (jitter * span).floor() as usize) % len]);
            cells.push(Cell { ground, ink, ch });
        }
    }
    (cols, rows, cells)
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &AtlasParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &AtlasParams, palette: &Palette, seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let (cols, rows, cells) = grid(params, palette, seed, height / width);
    let (cw, ch) = (width / f64::from(cols), height / f64::from(rows));
    for (y, row) in cells.chunks(cols as usize).enumerate() {
        let top = (y as f64 * ch).floor() as i32;
        let tall = ch.ceil() as u32 + 1;
        let mut x = 0;
        while x < row.len() {
            let ground = row[x].ground;
            let end = x + row[x..].iter().take_while(|c| c.ground == ground).count();
            let left = (x as f64 * cw).floor() as i32;
            let wide = ((end - x) as f64 * cw).ceil() as u32 + 1;
            surface.fill_rect(left, top, wide, tall, ground);
            x = end;
        }
    }
    let size = (ch * 0.98) as f32;
    let mut text = Type::new(Face::Book);
    for (i, cell) in cells.iter().enumerate() {
        let Some(glyph) = cell.ch else { continue };
        let (x, y) = ((i % cols as usize) as f64, (i / cols as usize) as f64);
        text.fill_centred(
            surface,
            glyph,
            ((x + 0.5) * cw) as f32,
            ((y + 0.55) * ch) as f32,
            size,
            cell.ink,
        );
    }
}
