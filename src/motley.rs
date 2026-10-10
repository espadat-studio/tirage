use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, value_noise,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "motley";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 9] = [
    "#111111", "#f4f1ea", "#1e7a46", "#e5352b", "#f5c400", "#f28ab8", "#2c7ac2", "#f0801e",
    "#f3e2b0",
];

const CELLS: Param = Param::new(SLUG, "cells", 16, 120, 1);
const MARK: Param = Param {
    taste: (75, 100),
    ..Param::new(SLUG, "mark", 40, 100, 100)
};
const PATCHES: Param = Param::new(SLUG, "patches", 1, 60, 1);
const SIZE: Param = Param::new(SLUG, "size", 0, 100, 100);
const RAGGED: Param = Param::new(SLUG, "ragged", 0, 100, 100);
const BLOCKS: Param = Param::new(SLUG, "blocks", 0, 100, 100);
const SPARSE: Param = Param {
    taste: (0, 60),
    ..Param::new(SLUG, "sparse", 0, 100, 100)
};
const STITCH: Param = Param::new(SLUG, "stitch", 0, 100, 100);
const GAPS: Param = Param {
    taste: (0, 30),
    ..Param::new(SLUG, "gaps", 0, 50, 100)
};

pub(crate) const PARAMS: &[Param] = &[
    CELLS, MARK, PATCHES, SIZE, RAGGED, BLOCKS, SPARSE, STITCH, GAPS,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MotleyParams {
    cells: u32,
    mark: f64,
    patches: u32,
    size: f64,
    ragged: f64,
    blocks: f64,
    sparse: f64,
    stitch: f64,
    gaps: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for MotleyParams {
    fn default() -> Self {
        Self {
            cells: 56,
            mark: 0.85,
            patches: 24,
            size: 0.5,
            ragged: 0.55,
            blocks: 0.4,
            sparse: 0.35,
            stitch: 0.4,
            gaps: 0.1,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl MotleyParams {
    pub fn cells(&self) -> u32 {
        self.cells
    }

    pub fn set_cells(&mut self, cells: u32) -> Result<(), Error> {
        CELLS.check(f64::from(cells))?;
        self.cells = cells;
        Ok(())
    }

    pub fn mark(&self) -> f64 {
        self.mark
    }

    pub fn set_mark(&mut self, mark: f64) -> Result<(), Error> {
        self.mark = MARK.check(mark)?;
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

    pub fn ragged(&self) -> f64 {
        self.ragged
    }

    pub fn set_ragged(&mut self, ragged: f64) -> Result<(), Error> {
        self.ragged = RAGGED.check(ragged)?;
        Ok(())
    }

    pub fn blocks(&self) -> f64 {
        self.blocks
    }

    pub fn set_blocks(&mut self, blocks: f64) -> Result<(), Error> {
        self.blocks = BLOCKS.check(blocks)?;
        Ok(())
    }

    pub fn sparse(&self) -> f64 {
        self.sparse
    }

    pub fn set_sparse(&mut self, sparse: f64) -> Result<(), Error> {
        self.sparse = SPARSE.check(sparse)?;
        Ok(())
    }

    pub fn stitch(&self) -> f64 {
        self.stitch
    }

    pub fn set_stitch(&mut self, stitch: f64) -> Result<(), Error> {
        self.stitch = STITCH.check(stitch)?;
        Ok(())
    }

    pub fn gaps(&self) -> f64 {
        self.gaps
    }

    pub fn set_gaps(&mut self, gaps: f64) -> Result<(), Error> {
        self.gaps = GAPS.check(gaps)?;
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
    cells: u32,
    mark: f64,
    patches: u32,
    size: f64,
    ragged: f64,
    blocks: f64,
    sparse: f64,
    stitch: f64,
    gaps: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<MotleyParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = MotleyParams::default();
    params.set_cells(raw.cells)?;
    params.set_mark(raw.mark)?;
    params.set_patches(raw.patches)?;
    params.set_size(raw.size)?;
    params.set_ragged(raw.ragged)?;
    params.set_blocks(raw.blocks)?;
    params.set_sparse(raw.sparse)?;
    params.set_stitch(raw.stitch)?;
    params.set_gaps(raw.gaps)?;
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
        CELLS.parameter(),
        MARK.parameter(),
        PATCHES.parameter(),
        SIZE.parameter(),
        RAGGED.parameter(),
        BLOCKS.parameter(),
        SPARSE.parameter(),
        STITCH.parameter(),
        GAPS.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> MotleyParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    MotleyParams {
        cells: pick(&CELLS) as u32,
        mark: pick(&MARK),
        patches: pick(&PATCHES) as u32,
        size: pick(&SIZE),
        ragged: pick(&RAGGED),
        blocks: pick(&BLOCKS),
        sparse: pick(&SPARSE),
        stitch: pick(&STITCH),
        gaps: pick(&GAPS),
        dither: Dither::default(),
        grain: Grain::default(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Glyph {
    Disc,
    Plus,
    Stripes,
    Square,
    Ring,
    Dot,
}

const GLYPHS: [Glyph; 6] = [
    Glyph::Disc,
    Glyph::Plus,
    Glyph::Stripes,
    Glyph::Square,
    Glyph::Ring,
    Glyph::Dot,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sparse {
    Full,
    Checker,
    Rows,
}

struct Patch {
    big: bool,
    block: bool,
    r: f64,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    ink: [u8; 3],
    stitch_ink: [u8; 3],
    dot_ink: [u8; 3],
    glyph: Glyph,
    sparse: Sparse,
    stitched: bool,
    rank: f64,
    empty: bool,
    salt: u32,
}

impl Patch {
    fn inside(&self, i: i32, j: i32, ragged: f64) -> bool {
        let dx = f64::from(i) + 0.5 - self.cx;
        let dy = f64::from(j) + 0.5 - self.cy;
        if self.block {
            return dx.abs() <= self.w * 0.5 && dy.abs() <= self.h * 0.5;
        }
        let d = (dx * dx + dy * dy).sqrt() / self.r;
        let wobble = value_noise(
            f64::from(i) / (self.r * 0.55) + 7.0,
            f64::from(j) / (self.r * 0.55) + 3.0,
            self.salt,
        );
        d < 0.55 + ragged * 0.8 * (wobble - 0.5) + 0.2
    }
}

fn patches(params: &MotleyParams, inks: &[[u8; 3]], seed: u32, cols: u32, rows: u32) -> Vec<Patch> {
    let mut rng = XorShift::new(seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let m = inks.len();
    let (cols, rows) = (f64::from(cols), f64::from(rows));
    let mut patches: Vec<Patch> = (0..params.patches.max(1) + 8)
        .map(|k| {
            let big = k < 8;
            let block = !big && rng.next() < params.blocks;
            let r = if big {
                0.45 + rng.next() * 0.4
            } else {
                0.1 + params.size * 0.35 * (0.4 + rng.next() * 0.8)
            } * cols;
            let cx = rng.next() * cols;
            let cy = rng.next() * rows;
            let ink = (rng.next() * m as f64).floor() as usize;
            let mut stitch = (rng.next() * m as f64).floor() as usize;
            if stitch == ink {
                stitch = (stitch + 1) % m;
            }
            let glyph = GLYPHS[(rng.next() * 6.0).floor() as usize];
            let sparse = if rng.next() < params.sparse {
                if rng.next() < 0.5 {
                    Sparse::Checker
                } else {
                    Sparse::Rows
                }
            } else {
                Sparse::Full
            };
            let stitched = rng.next() < params.stitch;
            let rank = rng.next();
            let w = r * (0.6 + rng.next() * 0.9);
            let h = r * (0.5 + rng.next() * 0.9);
            let dot = (ink + 1 + (rng.next() * (m - 1) as f64).floor() as usize) % m;
            Patch {
                big,
                block,
                r,
                cx,
                cy,
                w,
                h,
                ink: inks[ink],
                stitch_ink: inks[stitch],
                dot_ink: inks[dot],
                glyph,
                sparse,
                stitched,
                rank,
                empty: false,
                salt: seed.wrapping_mul(13).wrapping_add(k * 7),
            }
        })
        .collect();
    let mut small: Vec<&mut Patch> = patches.iter_mut().filter(|p| !p.big).collect();
    small.sort_by(|a, b| a.rank.total_cmp(&b.rank));
    let gaps = round_half_up(params.gaps * small.len() as f64) as usize;
    for patch in small.into_iter().take(gaps) {
        patch.empty = true;
    }
    patches
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &MotleyParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &MotleyParams, palette: &Palette, seed: u32) {
    let (width, height) = (f64::from(surface.width()), f64::from(surface.height()));
    let ground = palette.ink(0);
    let inks = &palette.inks()[1..];
    let cols = params.cells.max(8);
    let cs = width / f64::from(cols);
    let rows = (height / cs).ceil() as u32;
    let g = cs * params.mark;
    surface.fill(ground);
    let glyph =
        |surface: &mut Surface, kind: Glyph, x: f64, y: f64, ink: [u8; 3], dot: [u8; 3]| match kind
        {
            Glyph::Disc => surface.fill_circle(x, y, g * 0.47, ink),
            Glyph::Plus => {
                let (t, l) = (g * 0.24, g * 0.92);
                surface.fill_box(x - l * 0.5, y - t * 0.5, l, t, ink);
                surface.fill_box(x - t * 0.5, y - l * 0.5, t, l, ink);
            }
            Glyph::Stripes => {
                let (t, l) = (g * 0.17, g * 0.9);
                for q in -1..=1 {
                    surface.fill_box(
                        x - l * 0.5,
                        y + f64::from(q) * g * 0.32 - t * 0.5,
                        l,
                        t,
                        ink,
                    );
                }
            }
            Glyph::Square => surface.fill_box(x - g * 0.46, y - g * 0.46, g * 0.92, g * 0.92, ink),
            Glyph::Ring => {
                surface.fill_circle(x, y, g * 0.47, ink);
                surface.fill_circle(x, y, g * 0.2, dot);
            }
            Glyph::Dot => surface.fill_circle(x, y, g * 0.17, ink),
        };
    let ragged = params.ragged;
    for p in patches(params, inks, seed, cols, rows) {
        let reach = if p.block {
            p.w.max(p.h) * 0.5 + 1.0
        } else {
            p.r * 1.3 + 1.0
        };
        let i0 = (p.cx - reach).floor().max(0.0) as i32;
        let i1 = (p.cx + reach).ceil().min(f64::from(cols - 1)) as i32;
        let j0 = (p.cy - reach).floor().max(0.0) as i32;
        let j1 = (p.cy + reach).ceil().min(f64::from(rows - 1)) as i32;
        for j in j0..=j1 {
            for i in i0..=i1 {
                if !p.inside(i, j, ragged) {
                    continue;
                }
                let (x, y) = ((f64::from(i) + 0.5) * cs, (f64::from(j) + 0.5) * cs);
                let (left, top) = (f64::from(i) * cs, f64::from(j) * cs);
                let edge = p.stitched
                    && (!p.inside(i - 1, j, ragged)
                        || !p.inside(i + 1, j, ragged)
                        || !p.inside(i, j - 1, ragged)
                        || !p.inside(i, j + 1, ragged));
                if edge {
                    surface.fill_box(left, top, cs + 0.5, cs + 0.5, ground);
                    let kind = if hash(i, j, p.salt) < 0.5 {
                        Glyph::Dot
                    } else {
                        Glyph::Plus
                    };
                    glyph(surface, kind, x, y, p.stitch_ink, p.stitch_ink);
                    continue;
                }
                match p.sparse {
                    Sparse::Checker if (i + j) & 1 == 1 => continue,
                    Sparse::Rows if j & 1 == 1 => continue,
                    Sparse::Checker | Sparse::Rows if hash(i, j, p.salt) < 0.12 => continue,
                    _ => {}
                }
                surface.fill_box(left, top, cs + 0.5, cs + 0.5, ground);
                if p.empty {
                    continue;
                }
                glyph(surface, p.glyph, x, y, p.ink, p.dot_ink);
            }
        }
    }
}
