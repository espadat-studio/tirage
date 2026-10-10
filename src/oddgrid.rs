use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::Cap;
use crate::surface::Join;
use crate::surface::Path2D;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::terrain::{fbm, hash};
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "oddgrid";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const GROUND: [u8; 3] = [0x10, 0x18, 0x20];
const MARK_INK: [u8; 3] = [0xf2, 0xed, 0xe4];

const DEFAULT_PALETTE: [&str; 5] = ["#ff5e5b", "#ffed66", "#00cecb", "#f2ede4", "#9b5de5"];

const COLS: Param = Param::new(SLUG, "cols", 12, 110, 1);
const SCALE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "scale", 20, 600, 10)
};
const DENSITY: Param = Param {
    taste: (40, 100),
    ..Param::new(SLUG, "density", 0, 100, 100)
};
const GRAIN: Param = Param::new(SLUG, "grain", 0, 140, 100);
const VARIETY: Param = Param::new(SLUG, "variety", 0, 160, 100);
const BLOCK: Param = Param::new(SLUG, "block", 0, 100, 100);
const BSIZE: Param = Param::new(SLUG, "bsize", 2, 16, 1);
const SPECK: Param = Param {
    taste: (0, 45),
    ..Param::new(SLUG, "speck", 0, 60, 100)
};
const MOTIF_AMT: Param = Param::new(SLUG, "motifAmt", 0, 100, 100);
const MARK_SIZE: Param = Param::new(SLUG, "markSize", 15, 100, 100);
const BALANCE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "balance", -120, 120, 100)
};

pub(crate) const PARAMS: &[Param] = &[
    COLS, SCALE, DENSITY, GRAIN, VARIETY, BLOCK, BSIZE, SPECK, MOTIF_AMT, MARK_SIZE, BALANCE,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Motif {
    None,
    Dot,
    Ring,
    Square,
    Wedge,
    Mixed,
}

impl Motif {
    pub const ALL: &[Motif] = &[
        Self::None,
        Self::Dot,
        Self::Ring,
        Self::Square,
        Self::Wedge,
        Self::Mixed,
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OddgridParams {
    cols: u32,
    scale: f64,
    density: f64,
    grain: f64,
    variety: f64,
    block: f64,
    bsize: u32,
    speck: f64,
    motifs: Motif,
    #[serde(rename = "motifAmt")]
    motif_amt: f64,
    #[serde(rename = "markSize")]
    mark_size: f64,
    balance: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for OddgridParams {
    fn default() -> Self {
        Self {
            cols: 44,
            scale: 10.0,
            density: 0.55,
            grain: 0.35,
            variety: 0.5,
            block: 0.4,
            bsize: 6,
            speck: 0.08,
            motifs: Motif::None,
            motif_amt: 0.0,
            mark_size: 0.52,
            balance: 0.0,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl OddgridParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn density(&self) -> f64 {
        self.density
    }

    pub fn set_density(&mut self, density: f64) -> Result<(), Error> {
        self.density = DENSITY.check(density)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn variety(&self) -> f64 {
        self.variety
    }

    pub fn set_variety(&mut self, variety: f64) -> Result<(), Error> {
        self.variety = VARIETY.check(variety)?;
        Ok(())
    }

    pub fn block(&self) -> f64 {
        self.block
    }

    pub fn set_block(&mut self, block: f64) -> Result<(), Error> {
        self.block = BLOCK.check(block)?;
        Ok(())
    }

    pub fn bsize(&self) -> u32 {
        self.bsize
    }

    pub fn set_bsize(&mut self, bsize: u32) -> Result<(), Error> {
        BSIZE.check(f64::from(bsize))?;
        self.bsize = bsize;
        Ok(())
    }

    pub fn speck(&self) -> f64 {
        self.speck
    }

    pub fn set_speck(&mut self, speck: f64) -> Result<(), Error> {
        self.speck = SPECK.check(speck)?;
        Ok(())
    }

    pub fn motifs(&self) -> Motif {
        self.motifs
    }

    pub fn set_motifs(&mut self, motifs: Motif) {
        self.motifs = motifs;
    }

    pub fn motif_amt(&self) -> f64 {
        self.motif_amt
    }

    pub fn set_motif_amt(&mut self, motif_amt: f64) -> Result<(), Error> {
        self.motif_amt = MOTIF_AMT.check(motif_amt)?;
        Ok(())
    }

    pub fn mark_size(&self) -> f64 {
        self.mark_size
    }

    pub fn set_mark_size(&mut self, mark_size: f64) -> Result<(), Error> {
        self.mark_size = MARK_SIZE.check(mark_size)?;
        Ok(())
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn set_balance(&mut self, balance: f64) -> Result<(), Error> {
        self.balance = BALANCE.check(balance)?;
        Ok(())
    }

    pub fn dither(&self) -> &Dither {
        &self.dither
    }

    pub fn dither_mut(&mut self) -> &mut Dither {
        &mut self.dither
    }

    pub fn grain_pass(&self) -> &Grain {
        &self.chassis_grain
    }

    pub fn grain_pass_mut(&mut self) -> &mut Grain {
        &mut self.chassis_grain
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unchecked {
    cols: u32,
    scale: f64,
    density: f64,
    grain: f64,
    variety: f64,
    block: f64,
    bsize: u32,
    speck: f64,
    motifs: Motif,
    #[serde(rename = "motifAmt")]
    motif_amt: f64,
    #[serde(rename = "markSize")]
    mark_size: f64,
    balance: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<OddgridParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = OddgridParams::default();
    params.set_cols(raw.cols)?;
    params.set_scale(raw.scale)?;
    params.set_density(raw.density)?;
    params.set_grain(raw.grain)?;
    params.set_variety(raw.variety)?;
    params.set_block(raw.block)?;
    params.set_bsize(raw.bsize)?;
    params.set_speck(raw.speck)?;
    params.set_motifs(raw.motifs);
    params.set_motif_amt(raw.motif_amt)?;
    params.set_mark_size(raw.mark_size)?;
    params.set_balance(raw.balance)?;
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
        SCALE.parameter(),
        DENSITY.parameter(),
        GRAIN.parameter(),
        VARIETY.parameter(),
        BLOCK.parameter(),
        BSIZE.parameter(),
        SPECK.parameter(),
        Parameter::choice(
            "motifs",
            &["None", "Dot", "Ring", "Square", "Wedge", "Mixed"],
        ),
        MOTIF_AMT.parameter(),
        MARK_SIZE.parameter(),
        BALANCE.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> OddgridParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    OddgridParams {
        cols: pick(&COLS) as u32,
        scale: pick(&SCALE),
        density: pick(&DENSITY),
        grain: pick(&GRAIN),
        variety: pick(&VARIETY),
        block: pick(&BLOCK),
        bsize: pick(&BSIZE) as u32,
        speck: pick(&SPECK),
        motifs: Motif::ALL[(draw("motifs") % Motif::ALL.len() as u64) as usize],
        motif_amt: pick(&MOTIF_AMT),
        mark_size: pick(&MARK_SIZE),
        balance: pick(&BALANCE),
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Dot,
    Ring,
    Square,
    Wedge,
}

struct Cell {
    ink: Option<usize>,
    marked: bool,
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn stretch(v: f64, k: f64) -> f64 {
    ((v - 0.5) * k + 0.5).clamp(0.0, 1.0)
}

fn grid(params: &OddgridParams, n: usize, seed: u32, cols: u32, rows: u32) -> Vec<Cell> {
    let b = params.bsize.max(2) as i32;
    let bf = f64::from(b);
    let s = params.scale;
    let power = 2f64.powf(-params.balance);
    let at = |salt: u32| seed.wrapping_add(salt);
    let mut cells = Vec::with_capacity((cols * rows) as usize);
    for y in 0..rows as i32 {
        let ry = y.div_euclid(b);
        for x in 0..cols as i32 {
            let rx = x.div_euclid(b);
            let (xc, yc) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let bx = lerp(xc, f64::from(rx) * bf + bf / 2.0, params.block);
            let by = lerp(yc, f64::from(ry) * bf + bf / 2.0, params.block);
            let smooth = fbm(bx / s, by / s, seed, 2);
            let tone = smooth * 0.5 + hash(rx, ry, 0, 13, at(404)) * 0.5;
            let base = stretch(lerp(smooth, tone, params.block), 2.0);
            let mask = stretch(
                fbm(xc / (s * 1.15) + 91.0, yc / (s * 1.15) - 17.0, at(9111), 3),
                1.8,
            );
            let det = fbm(xc / (s * 0.26) + 37.0, yc / (s * 0.26) - 11.0, at(7717), 2);
            let bias = (hash(rx, ry, 0, 11, at(808)) - 0.5) * params.variety * 0.6;
            let level = 0.5 + (params.density - 0.5) * 1.1 + bias;
            let local = params.grain
                * lerp(
                    1.0,
                    0.15 + hash(rx, ry, 0, 17, at(909)) * 2.1,
                    (params.variety * 0.8).clamp(0.0, 1.0),
                );
            if mask + (det - 0.5) * local * 1.15 >= level {
                cells.push(Cell {
                    ink: None,
                    marked: false,
                });
                continue;
            }
            let mut ink = (n - 1).min((base.powf(power) * n as f64).floor() as usize);
            if params.speck > 0.0 && hash(x, y, 0, 3, at(21)) < params.speck {
                ink = (hash(x, y, 0, 4, at(33)) * n as f64).floor() as usize % n;
            }
            let marked = params.motifs != Motif::None
                && params.motif_amt > 0.0
                && hash(x, y, 0, 5, at(64)) < params.motif_amt;
            cells.push(Cell {
                ink: Some(ink),
                marked,
            });
        }
    }
    cells
}

fn shape(motif: Motif, x: i32, y: i32, seed: u32) -> Shape {
    match motif {
        Motif::Dot => Shape::Dot,
        Motif::Ring => Shape::Ring,
        Motif::Square => Shape::Square,
        Motif::Wedge => Shape::Wedge,
        Motif::None | Motif::Mixed => {
            let k = hash(x, y, 0, 9, seed.wrapping_add(404));
            if k < 0.4 {
                Shape::Dot
            } else if k < 0.65 {
                Shape::Square
            } else if k < 0.85 {
                Shape::Ring
            } else {
                Shape::Wedge
            }
        }
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &OddgridParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &OddgridParams, palette: &Palette, seed: u32) {
    let (width, height) = (surface.width(), surface.height());
    let cols = params.cols;
    let rows =
        (round_half_up(f64::from(cols) * f64::from(height) / f64::from(width)) as u32).max(4);
    let inks = palette.inks();
    let cells = grid(params, inks.len(), seed, cols, rows);
    let edge = |i: u32, total: u32, n: u32| {
        round_half_up(f64::from(i) * f64::from(total) / f64::from(n)) as i32
    };
    let xs: Vec<i32> = (0..=cols).map(|i| edge(i, width, cols)).collect();
    let ys: Vec<i32> = (0..=rows).map(|j| edge(j, height, rows)).collect();
    surface.fill(GROUND);
    for y in 0..rows as usize {
        let (top, tall) = (ys[y], (ys[y + 1] - ys[y]) as u32);
        let row = &cells[y * cols as usize..(y + 1) * cols as usize];
        let mut x = 0;
        while x < row.len() {
            let Some(ink) = row[x].ink else {
                x += 1;
                continue;
            };
            if row[x].marked {
                let wide = (xs[x + 1] - xs[x]) as u32;
                fill(surface, xs[x], top, wide, tall, inks[ink]);
                let kind = shape(params.motifs, x as i32, y as i32, seed);
                mark(surface, kind, params.mark_size, xs[x], top, wide, tall);
                x += 1;
                continue;
            }
            let run = 1 + row[x + 1..]
                .iter()
                .take_while(|c| c.ink == Some(ink) && !c.marked)
                .count();
            fill(
                surface,
                xs[x],
                top,
                (xs[x + run] - xs[x]) as u32,
                tall,
                inks[ink],
            );
            x += run;
        }
    }
}

fn fill(surface: &mut Surface, x: i32, y: i32, width: u32, height: u32, ink: [u8; 3]) {
    if width > 0 && height > 0 {
        surface.fill_rect(x, y, width, height, ink);
    }
}

fn mark(surface: &mut Surface, kind: Shape, size: f64, px: i32, py: i32, cw: u32, ch: u32) {
    let (px, py, cw, ch) = (f64::from(px), f64::from(py), f64::from(cw), f64::from(ch));
    let cs = cw.min(ch);
    let (mx, my) = (px + cw / 2.0, py + ch / 2.0);
    match kind {
        Shape::Dot => {
            let r = cs * size / 2.0;
            if r > 0.0 {
                surface.fill_circle(mx, my, r, MARK_INK);
            }
        }
        Shape::Ring => {
            let lw = (cs * size * 0.28).max(1.0);
            let mut ring = Path2D::default();
            ring.push_circle(mx, my, (lw * 0.6).max(cs * size / 2.0 - lw / 2.0));
            surface.stroke(&ring, MARK_INK, lw, Cap::Butt, Join::Round);
        }
        Shape::Square => {
            let side = cs * size;
            surface.fill_box(mx - side / 2.0, my - side / 2.0, side, side, MARK_INK);
        }
        Shape::Wedge => {
            let mut wedge = Path2D::default();
            wedge.move_to(px, py + ch);
            wedge.line_to(px + cw, py + ch);
            wedge.line_to(px + cw, py);
            wedge.close();
            surface.fill_path(&wedge.into_path(), MARK_INK);
        }
    }
}
