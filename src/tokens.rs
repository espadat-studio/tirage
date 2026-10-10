use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "tokens";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 14;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 9] = [
    "#1b1b1b", "#e63946", "#3a86ff", "#ffbe0b", "#2a9d8f", "#ff006e", "#8338ec", "#fb5607",
    "#00b4d8",
];
const BOARD: [u8; 3] = [0xf4, 0xf1, 0xea];
const RULE: [u8; 3] = [0xd9, 0xd4, 0xc7];
const VB: f64 = 1000.0;

const COLS: Param = Param {
    taste: (4, 48),
    ..Param::new(SLUG, "cols", 4, 120, 1)
};
const RULE_EVERY: Param = Param::new(SLUG, "ruleEvery", 1, 10, 1);
const GRID: Param = Param::new(SLUG, "grid", 0, 100, 100);
const COUNT: Param = Param {
    taste: (8, 400),
    ..Param::new(SLUG, "count", 1, 400, 1)
};
const W_RUN: Param = Param::new(SLUG, "wRun", 0, 100, 1);
const W_BLOCK: Param = Param::new(SLUG, "wBlock", 0, 100, 1);
const W_PLUS: Param = Param::new(SLUG, "wPlus", 0, 100, 1);
const W_ONE: Param = Param::new(SLUG, "wOne", 0, 100, 1);
const CLUMP: Param = Param::new(SLUG, "clump", 0, 100, 100);
const ALIGN: Param = Param::new(SLUG, "align", 0, 100, 100);
const RUN_LEN: Param = Param::new(SLUG, "runLen", 2, 12, 1);
const UPRIGHT: Param = Param::new(SLUG, "upright", 0, 100, 100);
const SIZE_T: Param = Param {
    taste: (20, 100),
    ..Param::new(SLUG, "sizeT", 3, 100, 100)
};
const SVAR: Param = Param::new(SLUG, "svar", 0, 100, 100);
const HIER: Param = Param::new(SLUG, "hier", 0, 100, 100);
const SQUARE: Param = Param::new(SLUG, "square", 0, 100, 100);
const STROKE_W: Param = Param {
    taste: (1, 16),
    ..Param::new(SLUG, "strokeW", 0, 16, 2)
};
const HOLLOW: Param = Param::new(SLUG, "hollow", 0, 100, 100);
const KIN: Param = Param::new(SLUG, "kin", 0, 100, 100);
const BREAK: Param = Param::new(SLUG, "break", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    COLS, RULE_EVERY, GRID, COUNT, W_RUN, W_BLOCK, W_PLUS, W_ONE, CLUMP, ALIGN, RUN_LEN, UPRIGHT,
    SIZE_T, SVAR, HIER, SQUARE, STROKE_W, HOLLOW, KIN, BREAK,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokensParams {
    cols: u32,
    #[serde(rename = "ruleEvery")]
    rule_every: u32,
    grid: f64,
    count: u32,
    #[serde(rename = "wRun")]
    w_run: u32,
    #[serde(rename = "wBlock")]
    w_block: u32,
    #[serde(rename = "wPlus")]
    w_plus: u32,
    #[serde(rename = "wOne")]
    w_one: u32,
    clump: f64,
    align: f64,
    #[serde(rename = "runLen")]
    run_len: u32,
    upright: f64,
    #[serde(rename = "sizeT")]
    size_t: f64,
    svar: f64,
    hier: f64,
    square: f64,
    #[serde(rename = "strokeW")]
    stroke_w: f64,
    hollow: f64,
    kin: f64,
    #[serde(rename = "break")]
    brk: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for TokensParams {
    fn default() -> Self {
        Self {
            cols: 16,
            rule_every: 4,
            grid: 0.6,
            count: 14,
            w_run: 44,
            w_block: 18,
            w_plus: 10,
            w_one: 28,
            clump: 0.7,
            align: 0.55,
            run_len: 5,
            upright: 0.18,
            size_t: 0.5,
            svar: 0.45,
            hier: 0.65,
            square: 0.34,
            stroke_w: 1.5,
            hollow: 0.18,
            kin: 0.2,
            brk: 0.3,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl TokensParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn rule_every(&self) -> u32 {
        self.rule_every
    }

    pub fn set_rule_every(&mut self, rule_every: u32) -> Result<(), Error> {
        RULE_EVERY.check(f64::from(rule_every))?;
        self.rule_every = rule_every;
        Ok(())
    }

    pub fn grid(&self) -> f64 {
        self.grid
    }

    pub fn set_grid(&mut self, grid: f64) -> Result<(), Error> {
        self.grid = GRID.check(grid)?;
        Ok(())
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn w_run(&self) -> u32 {
        self.w_run
    }

    pub fn set_w_run(&mut self, w_run: u32) -> Result<(), Error> {
        W_RUN.check(f64::from(w_run))?;
        self.w_run = w_run;
        Ok(())
    }

    pub fn w_block(&self) -> u32 {
        self.w_block
    }

    pub fn set_w_block(&mut self, w_block: u32) -> Result<(), Error> {
        W_BLOCK.check(f64::from(w_block))?;
        self.w_block = w_block;
        Ok(())
    }

    pub fn w_plus(&self) -> u32 {
        self.w_plus
    }

    pub fn set_w_plus(&mut self, w_plus: u32) -> Result<(), Error> {
        W_PLUS.check(f64::from(w_plus))?;
        self.w_plus = w_plus;
        Ok(())
    }

    pub fn w_one(&self) -> u32 {
        self.w_one
    }

    pub fn set_w_one(&mut self, w_one: u32) -> Result<(), Error> {
        W_ONE.check(f64::from(w_one))?;
        self.w_one = w_one;
        Ok(())
    }

    pub fn clump(&self) -> f64 {
        self.clump
    }

    pub fn set_clump(&mut self, clump: f64) -> Result<(), Error> {
        self.clump = CLUMP.check(clump)?;
        Ok(())
    }

    pub fn align(&self) -> f64 {
        self.align
    }

    pub fn set_align(&mut self, align: f64) -> Result<(), Error> {
        self.align = ALIGN.check(align)?;
        Ok(())
    }

    pub fn run_len(&self) -> u32 {
        self.run_len
    }

    pub fn set_run_len(&mut self, run_len: u32) -> Result<(), Error> {
        RUN_LEN.check(f64::from(run_len))?;
        self.run_len = run_len;
        Ok(())
    }

    pub fn upright(&self) -> f64 {
        self.upright
    }

    pub fn set_upright(&mut self, upright: f64) -> Result<(), Error> {
        self.upright = UPRIGHT.check(upright)?;
        Ok(())
    }

    pub fn size_t(&self) -> f64 {
        self.size_t
    }

    pub fn set_size_t(&mut self, size_t: f64) -> Result<(), Error> {
        self.size_t = SIZE_T.check(size_t)?;
        Ok(())
    }

    pub fn svar(&self) -> f64 {
        self.svar
    }

    pub fn set_svar(&mut self, svar: f64) -> Result<(), Error> {
        self.svar = SVAR.check(svar)?;
        Ok(())
    }

    pub fn hier(&self) -> f64 {
        self.hier
    }

    pub fn set_hier(&mut self, hier: f64) -> Result<(), Error> {
        self.hier = HIER.check(hier)?;
        Ok(())
    }

    pub fn square(&self) -> f64 {
        self.square
    }

    pub fn set_square(&mut self, square: f64) -> Result<(), Error> {
        self.square = SQUARE.check(square)?;
        Ok(())
    }

    pub fn stroke_w(&self) -> f64 {
        self.stroke_w
    }

    pub fn set_stroke_w(&mut self, stroke_w: f64) -> Result<(), Error> {
        self.stroke_w = STROKE_W.check(stroke_w)?;
        Ok(())
    }

    pub fn hollow(&self) -> f64 {
        self.hollow
    }

    pub fn set_hollow(&mut self, hollow: f64) -> Result<(), Error> {
        self.hollow = HOLLOW.check(hollow)?;
        Ok(())
    }

    pub fn kin(&self) -> f64 {
        self.kin
    }

    pub fn set_kin(&mut self, kin: f64) -> Result<(), Error> {
        self.kin = KIN.check(kin)?;
        Ok(())
    }

    pub fn brk(&self) -> f64 {
        self.brk
    }

    pub fn set_brk(&mut self, brk: f64) -> Result<(), Error> {
        self.brk = BREAK.check(brk)?;
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
    cols: u32,
    #[serde(rename = "ruleEvery")]
    rule_every: u32,
    grid: f64,
    count: u32,
    #[serde(rename = "wRun")]
    w_run: u32,
    #[serde(rename = "wBlock")]
    w_block: u32,
    #[serde(rename = "wPlus")]
    w_plus: u32,
    #[serde(rename = "wOne")]
    w_one: u32,
    clump: f64,
    align: f64,
    #[serde(rename = "runLen")]
    run_len: u32,
    upright: f64,
    #[serde(rename = "sizeT")]
    size_t: f64,
    svar: f64,
    hier: f64,
    square: f64,
    #[serde(rename = "strokeW")]
    stroke_w: f64,
    hollow: f64,
    kin: f64,
    #[serde(rename = "break")]
    brk: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<TokensParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = TokensParams::default();
    params.set_cols(raw.cols)?;
    params.set_rule_every(raw.rule_every)?;
    params.set_grid(raw.grid)?;
    params.set_count(raw.count)?;
    params.set_w_run(raw.w_run)?;
    params.set_w_block(raw.w_block)?;
    params.set_w_plus(raw.w_plus)?;
    params.set_w_one(raw.w_one)?;
    params.set_clump(raw.clump)?;
    params.set_align(raw.align)?;
    params.set_run_len(raw.run_len)?;
    params.set_upright(raw.upright)?;
    params.set_size_t(raw.size_t)?;
    params.set_svar(raw.svar)?;
    params.set_hier(raw.hier)?;
    params.set_square(raw.square)?;
    params.set_stroke_w(raw.stroke_w)?;
    params.set_hollow(raw.hollow)?;
    params.set_kin(raw.kin)?;
    params.set_brk(raw.brk)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> TokensParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    TokensParams {
        cols: pick(&COLS) as u32,
        rule_every: pick(&RULE_EVERY) as u32,
        grid: pick(&GRID),
        count: pick(&COUNT) as u32,
        w_run: pick(&W_RUN) as u32,
        w_block: pick(&W_BLOCK) as u32,
        w_plus: pick(&W_PLUS) as u32,
        w_one: pick(&W_ONE) as u32,
        clump: pick(&CLUMP),
        align: pick(&ALIGN),
        run_len: pick(&RUN_LEN) as u32,
        upright: pick(&UPRIGHT),
        size_t: pick(&SIZE_T),
        svar: pick(&SVAR),
        hier: pick(&HIER),
        square: pick(&SQUARE),
        stroke_w: pick(&STROKE_W),
        hollow: pick(&HOLLOW),
        kin: pick(&KIN),
        brk: pick(&BREAK),
        ..TokensParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &TokensParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Run,
    Block,
    Plus,
    One,
}

struct Token {
    x: i32,
    y: i32,
    fill: usize,
    outline: usize,
    square: bool,
    hollow: bool,
}

struct Cluster {
    size: f64,
    tokens: Vec<Token>,
}

struct Board {
    columns: i32,
    rows: i32,
    cell: f64,
}

fn board(p: &TokensParams, w: f64, h: f64) -> Board {
    let columns = p.cols.max(3) as i32;
    let rows = 3.max((f64::from(columns) * (VB * h / w) / VB).round() as i32);
    Board {
        columns,
        rows,
        cell: VB / f64::from(columns),
    }
}

fn hue(&[r, g, b]: &[u8; 3]) -> f64 {
    let [r, g, b] = [r, g, b].map(|c| f64::from(c) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let d = max - min;
    if d == 0.0 {
        return 0.0;
    }
    let h = if max == r {
        ((g - b) / d) % 6.0
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    ((h * 60.0) % 360.0 + 360.0) % 360.0
}

fn hue_gap(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

fn field(x: f64, y: f64, seed: u32) -> f64 {
    let (xi, yi) = (x.floor(), y.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (smooth(x - xi), smooth(y - yi));
    let (xi, yi) = (xi as i32, yi as i32);
    let a = hash(xi, yi, 0, seed);
    let b = hash(xi + 1, yi, 0, seed);
    let c = hash(xi, yi + 1, 0, seed);
    let d = hash(xi + 1, yi + 1, 0, seed);
    let top = a + (b - a) * u;
    top + ((c + (d - c) * u) - top) * v
}

fn place(p: &TokensParams, board: &Board, palette: &Palette, tool_seed: u32) -> Vec<Cluster> {
    let Board { columns, rows, .. } = *board;
    let start = (f64::from(tool_seed) * 2_654_435_761.0 + 17.0) as u64 as u32;
    let mut rng = Xorshift::from_state(start);
    let mut r = || rng.next();
    let n = palette.len();
    let hues: Vec<f64> = palette.inks().iter().map(hue).collect();
    let near: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut list: Vec<usize> = (0..n).filter(|&j| j != i).collect();
            list.sort_by(|&a, &b| hue_gap(hues[i], hues[a]).total_cmp(&hue_gap(hues[i], hues[b])));
            list
        })
        .collect();
    let outline = |fill: usize, r: &mut dyn FnMut() -> f64| {
        let list = &near[fill];
        let reach = 1.max((1.0 + p.kin * (list.len() - 1) as f64).round() as usize);
        let pick = list[(r() * reach as f64) as usize];
        if pick == fill { (pick + 1) % n } else { pick }
    };
    let kinds = [
        (Kind::Run, p.w_run),
        (Kind::Block, p.w_block),
        (Kind::Plus, p.w_plus),
        (Kind::One, p.w_one),
    ];
    let total = match kinds.iter().map(|&(_, w)| w).sum::<u32>() {
        0 => 1,
        total => total,
    };

    let guides = 2.max((2.0 + (1.0 - p.align) * 4.0).round() as usize);
    let (mut guide_rows, mut guide_columns) = (Vec::new(), Vec::new());
    for _ in 0..guides {
        guide_rows.push((r() * f64::from(rows)) as i32);
        guide_columns.push((r() * f64::from(columns)) as i32);
    }

    let mut taken = vec![false; (columns * rows) as usize];
    let at = |x: i32, y: i32| (y * columns + x) as usize;
    let mut clusters = Vec::new();
    let mut tries = 0;
    while clusters.len() < p.count as usize && tries < p.count * 140 {
        tries += 1;
        let mut left = r() * f64::from(total);
        let mut kind = Kind::One;
        for (k, w) in kinds {
            left -= f64::from(w);
            if left <= 0.0 {
                kind = k;
                break;
            }
        }
        let cells: Vec<(i32, i32)> = match kind {
            Kind::One => vec![(0, 0)],
            Kind::Plus => vec![(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1)],
            Kind::Block => {
                let w = 2 + i32::from(r() < 0.4);
                let h = 2 + i32::from(r() < 0.4);
                (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).collect()
            }
            Kind::Run => {
                let len = 2 + (r() * f64::from(p.run_len.saturating_sub(1).max(1))) as i32;
                let upright = r() < p.upright;
                (0..len)
                    .map(|i| if upright { (0, i) } else { (i, 0) })
                    .collect()
            }
        };
        let mut ox = (r() * f64::from(columns)) as i32;
        let mut oy = (r() * f64::from(rows)) as i32;
        if p.align > 0.0 && r() < p.align {
            if r() < 0.5 {
                oy = guide_rows[(r() * guides as f64) as usize];
            } else {
                ox = guide_columns[(r() * guides as f64) as usize];
            }
        }
        if p.clump > 0.0 {
            let fv = field(
                f64::from(ox) / f64::from(columns) * 2.6 + 0.5,
                f64::from(oy) / f64::from(rows) * 2.6 + 0.5,
                tool_seed.wrapping_add(301),
            );
            if r() > fv.powf(1.0 + p.clump * 6.0) {
                continue;
            }
        }
        let free = cells.iter().all(|&(dx, dy)| {
            let (x, y) = (ox + dx, oy + dy);
            (0..columns).contains(&x)
                && (0..rows).contains(&y)
                && (-1..=1).all(|jy| {
                    (-1..=1).all(|jx| {
                        let (nx, ny) = (x + jx, y + jy);
                        !(0..columns).contains(&nx)
                            || !(0..rows).contains(&ny)
                            || !taken[at(nx, ny)]
                    })
                })
        });
        if !free {
            continue;
        }
        for &(dx, dy) in &cells {
            taken[at(ox + dx, oy + dy)] = true;
        }
        let fill = (r() * n as f64) as usize;
        let edge = outline(fill, &mut r);
        let square = r() < p.square;
        let big = r().powf(1.0 + p.hier * 3.2);
        let size = p.size_t * (1.0 - p.svar * 0.5 + p.svar * r()) * (1.0 + p.hier * 2.6 * big);
        let tokens = cells
            .iter()
            .map(|&(dx, dy)| {
                let (mut f, mut e) = (fill, edge);
                if r() < p.brk {
                    f = (r() * n as f64) as usize;
                    e = outline(f, &mut r);
                }
                let flip = r() < 0.12;
                let hollow = r() < p.hollow;
                r();
                Token {
                    x: ox + dx,
                    y: oy + dy,
                    fill: f,
                    outline: e,
                    square: square != flip,
                    hollow,
                }
            })
            .collect();
        r();
        r();
        r();
        clusters.push(Cluster { size, tokens });
    }
    clusters
}

fn paint(surface: &mut Surface, p: &TokensParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let k = w / VB;
    let board = board(p, w, h);
    surface.fill(BOARD);
    if p.grid > 0.0 {
        let mut path = Path2D::default();
        let step = p.rule_every.max(1) as i32;
        for i in (step..board.columns).step_by(step as usize) {
            let x = (f64::from(i) * board.cell * k).round() + 0.5;
            path.move_to(x, 0.0);
            path.line_to(x, h);
        }
        for j in 1..board.rows {
            let y = (f64::from(j) * board.cell * k).round() + 0.5;
            path.move_to(0.0, y);
            path.line_to(w, y);
        }
        if !path.is_empty() {
            surface.with_alpha(p.grid as f32, |surface| {
                surface.stroke(&path, RULE, (1.1 * k).max(0.5), Cap::Butt, Join::Miter);
            });
        }
    }
    let cell = board.cell;
    let room = 0.05_f64.max(cell / 2.0 - p.stroke_w.max(0.0));
    for cluster in place(p, &board, palette, tool_seed) {
        let radius = 0.05_f64.max((cluster.size * cell / 2.0).min(room));
        for token in &cluster.tokens {
            let (cx, cy) = (
                (f64::from(token.x) + 0.5) * cell * k,
                (f64::from(token.y) + 0.5) * cell * k,
            );
            let shape = |surface: &mut Surface, radius: f64, ink: [u8; 3]| {
                let r = radius * k;
                if token.square {
                    surface.fill_box(cx - r, cy - r, r * 2.0, r * 2.0, ink);
                } else {
                    surface.fill_circle(cx, cy, r, ink);
                }
            };
            if p.stroke_w > 0.0 {
                shape(surface, radius + p.stroke_w, palette.ink(token.outline));
            }
            let fill = if token.hollow {
                BOARD
            } else {
                palette.ink(token.fill)
            };
            shape(surface, radius, fill);
        }
    }
}
