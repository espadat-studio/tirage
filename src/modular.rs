use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "modular";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 6;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 4] = ["#e63946", "#1d3557", "#f1c40f", "#1b1b1b"];
const GROUND: [u8; 3] = [0xf4, 0xef, 0xe3];
const RULE: [u8; 3] = [0x1b, 0x1b, 0x1b];

const GCOLS: Param = Param::new(SLUG, "gcols", 2, 12, 1);
const UNIT: Param = Param::new(SLUG, "unit", 2, 12, 1);
const MERGE: Param = Param::new(SLUG, "merge", 0, 100, 100);
const W_EMPTY: Param = Param::new(SLUG, "wEmpty", 0, 100, 1);
const W_SOLID: Param = Param::new(SLUG, "wSolid", 0, 100, 1);
const W_BLOCKS: Param = Param::new(SLUG, "wBlocks", 0, 100, 1);
const W_DOTS: Param = Param::new(SLUG, "wDots", 0, 100, 1);
const W_LINES: Param = Param::new(SLUG, "wLines", 0, 100, 1);
const W_GRAD: Param = Param::new(SLUG, "wGrad", 0, 100, 1);
const BLOCK_FILL: Param = Param::new(SLUG, "blockFill", 10, 90, 100);
const DOT: Param = Param::new(SLUG, "dot", 20, 100, 100);
const RULES: Param = Param::new(SLUG, "rules", 0, 100, 100);
const RULE_W: Param = Param::new(SLUG, "ruleW", 1, 8, 2);

pub(crate) const PARAMS: &[Param] = &[
    GCOLS, UNIT, MERGE, W_EMPTY, W_SOLID, W_BLOCKS, W_DOTS, W_LINES, W_GRAD, BLOCK_FILL, DOT,
    RULES, RULE_W,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModularParams {
    gcols: u32,
    unit: u32,
    merge: f64,
    #[serde(rename = "wEmpty")]
    w_empty: u32,
    #[serde(rename = "wSolid")]
    w_solid: u32,
    #[serde(rename = "wBlocks")]
    w_blocks: u32,
    #[serde(rename = "wDots")]
    w_dots: u32,
    #[serde(rename = "wLines")]
    w_lines: u32,
    #[serde(rename = "wGrad")]
    w_grad: u32,
    #[serde(rename = "blockFill")]
    block_fill: f64,
    dot: f64,
    rules: f64,
    #[serde(rename = "ruleW")]
    rule_w: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for ModularParams {
    fn default() -> Self {
        Self {
            gcols: 6,
            unit: 4,
            merge: 0.45,
            w_empty: 34,
            w_solid: 20,
            w_blocks: 24,
            w_dots: 14,
            w_lines: 12,
            w_grad: 10,
            block_fill: 0.5,
            dot: 0.62,
            rules: 0.22,
            rule_w: 1.0,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

macro_rules! whole {
    ($($get:ident, $set:ident, $param:ident;)*) => {
        $(
            pub fn $get(&self) -> u32 {
                self.$get
            }

            pub fn $set(&mut self, value: u32) -> Result<(), Error> {
                $param.check(f64::from(value))?;
                self.$get = value;
                Ok(())
            }
        )*
    };
}

macro_rules! share {
    ($($get:ident, $set:ident, $param:ident;)*) => {
        $(
            pub fn $get(&self) -> f64 {
                self.$get
            }

            pub fn $set(&mut self, value: f64) -> Result<(), Error> {
                self.$get = $param.check(value)?;
                Ok(())
            }
        )*
    };
}

impl ModularParams {
    whole! {
        gcols, set_gcols, GCOLS;
        unit, set_unit, UNIT;
        w_empty, set_w_empty, W_EMPTY;
        w_solid, set_w_solid, W_SOLID;
        w_blocks, set_w_blocks, W_BLOCKS;
        w_dots, set_w_dots, W_DOTS;
        w_lines, set_w_lines, W_LINES;
        w_grad, set_w_grad, W_GRAD;
    }

    share! {
        merge, set_merge, MERGE;
        block_fill, set_block_fill, BLOCK_FILL;
        dot, set_dot, DOT;
        rules, set_rules, RULES;
    }

    pub fn rule_w(&self) -> f64 {
        self.rule_w
    }

    pub fn set_rule_w(&mut self, rule_w: f64) -> Result<(), Error> {
        RULE_W.ticks(rule_w)?;
        self.rule_w = rule_w;
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
    gcols: u32,
    unit: u32,
    merge: f64,
    #[serde(rename = "wEmpty")]
    w_empty: u32,
    #[serde(rename = "wSolid")]
    w_solid: u32,
    #[serde(rename = "wBlocks")]
    w_blocks: u32,
    #[serde(rename = "wDots")]
    w_dots: u32,
    #[serde(rename = "wLines")]
    w_lines: u32,
    #[serde(rename = "wGrad")]
    w_grad: u32,
    #[serde(rename = "blockFill")]
    block_fill: f64,
    dot: f64,
    rules: f64,
    #[serde(rename = "ruleW")]
    rule_w: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<ModularParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = ModularParams::default();
    params.set_gcols(raw.gcols)?;
    params.set_unit(raw.unit)?;
    params.set_merge(raw.merge)?;
    params.set_w_empty(raw.w_empty)?;
    params.set_w_solid(raw.w_solid)?;
    params.set_w_blocks(raw.w_blocks)?;
    params.set_w_dots(raw.w_dots)?;
    params.set_w_lines(raw.w_lines)?;
    params.set_w_grad(raw.w_grad)?;
    params.set_block_fill(raw.block_fill)?;
    params.set_dot(raw.dot)?;
    params.set_rules(raw.rules)?;
    params.set_rule_w(raw.rule_w)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> ModularParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    ModularParams {
        gcols: pick(&GCOLS) as u32,
        unit: pick(&UNIT) as u32,
        merge: pick(&MERGE),
        w_empty: pick(&W_EMPTY) as u32,
        w_solid: pick(&W_SOLID) as u32,
        w_blocks: pick(&W_BLOCKS) as u32,
        w_dots: pick(&W_DOTS) as u32,
        w_lines: pick(&W_LINES) as u32,
        w_grad: pick(&W_GRAD) as u32,
        block_fill: pick(&BLOCK_FILL),
        dot: pick(&DOT),
        rules: pick(&RULES),
        rule_w: pick(&RULE_W),
        ..ModularParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &ModularParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Empty,
    Solid,
    Blocks,
    Dots,
    Lines,
    Grad,
}

struct Module {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    kind: Kind,
    ci: usize,
    ci2: usize,
    on_bg: bool,
    inset: u32,
    corner: u32,
    angle: u32,
}

fn noise(x: f64, y: f64, key: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let corner = |dx: i32, dy: i32| hash(xi + dx, yi + dy, 0, key);
    lerp(
        lerp(corner(0, 0), corner(1, 0), u),
        lerp(corner(0, 1), corner(1, 1), u),
        v,
    )
}

fn lit(gx: u32, gy: u32, key: u32, block_fill: f64, tool_seed: u32) -> bool {
    let q = tool_seed
        .wrapping_add(key.wrapping_mul(131))
        .wrapping_add(9);
    let (x, y) = (f64::from(gx) * 0.55, f64::from(gy) * 0.55);
    let v = (0.5 * noise(x, y, q) + 0.25 * noise(x * 2.0, y * 2.0, q.wrapping_add(1319))) / 0.75;
    ((v - 0.5) * 1.9 + 0.5).clamp(0.0, 1.0) < block_fill
}

fn edges(n: u32, a: i32, b: i32) -> Vec<i32> {
    (0..=n)
        .map(|i| {
            round_half_up(f64::from(a) + f64::from(b - a) * f64::from(i) / f64::from(n)) as i32
        })
        .collect()
}

fn layout(p: &ModularParams, columns: u32, rows: u32, inks: usize, tool_seed: u32) -> Vec<Module> {
    let start = (f64::from(tool_seed) * 2_654_435_761.0 + 17.0).rem_euclid(4_294_967_296.0);
    let mut rng = Xorshift::from_state(start as u32);
    let mut taken = vec![false; (columns * rows) as usize];
    let free = |taken: &[bool], x: u32, y: u32, w: u32, h: u32| {
        x + w <= columns
            && y + h <= rows
            && (y..y + h).all(|j| (x..x + w).all(|i| !taken[(j * columns + i) as usize]))
    };
    let weights = [
        (Kind::Empty, p.w_empty),
        (Kind::Solid, p.w_solid),
        (Kind::Blocks, p.w_blocks),
        (Kind::Dots, p.w_dots),
        (Kind::Lines, p.w_lines),
        (Kind::Grad, p.w_grad),
    ];
    let total = match weights.iter().map(|&(_, w)| w).sum::<u32>() {
        0 => 1,
        total => total,
    };
    let mut modules = Vec::new();
    for y in 0..rows {
        for x in 0..columns {
            if taken[(y * columns + x) as usize] {
                continue;
            }
            let k = rng.next();
            let (w, h) = if k < p.merge * 0.32 && free(&taken, x, y, 2, 2) {
                (2, 2)
            } else if k < p.merge * 0.66 && free(&taken, x, y, 2, 1) {
                (2, 1)
            } else if k < p.merge && free(&taken, x, y, 1, 2) {
                (1, 2)
            } else {
                (1, 1)
            };
            for j in y..y + h {
                for i in x..x + w {
                    taken[(j * columns + i) as usize] = true;
                }
            }
            let mut n = rng.next() * f64::from(total);
            let kind = weights
                .iter()
                .find(|&&(_, w)| {
                    n -= f64::from(w);
                    n <= 0.0
                })
                .map_or(Kind::Empty, |&(kind, _)| kind);
            let ci = (rng.next() * inks as f64) as usize;
            let ci2 = (ci + 1 + (rng.next() * (inks - 1) as f64) as usize) % inks;
            let sub = rng.next();
            let on_bg = rng.next() < 0.45;
            let corner = (rng.next() * 4.0) as u32;
            let angle = (rng.next() * 4.0) as u32;
            rng.next();
            modules.push(Module {
                x,
                y,
                w,
                h,
                kind,
                ci,
                ci2,
                on_bg,
                inset: if sub < 0.55 {
                    0
                } else if sub < 0.8 {
                    1
                } else {
                    2
                },
                corner,
                angle,
            });
        }
    }
    modules
}

fn rules(xs: &[i32], ys: &[i32], (x0, y0, x1, y1): (i32, i32, i32, i32)) -> Path2D {
    let mut path = Path2D::default();
    for &x in &xs[1..xs.len() - 1] {
        path.move_to(f64::from(x) + 0.5, f64::from(y0));
        path.line_to(f64::from(x) + 0.5, f64::from(y1));
    }
    for &y in &ys[1..ys.len() - 1] {
        path.move_to(f64::from(x0), f64::from(y) + 0.5);
        path.line_to(f64::from(x1), f64::from(y) + 0.5);
    }
    path
}

fn paint(surface: &mut Surface, p: &ModularParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (surface.width() as i32, surface.height() as i32);
    let inks = palette.inks();
    let columns = p.gcols;
    let rows =
        2_u32.max(round_half_up(f64::from(columns) * f64::from(height) / f64::from(width)) as u32);
    surface.fill(GROUND);
    let mx = edges(columns, 0, width);
    let my = edges(rows, 0, height);

    for (idx, m) in layout(p, columns, rows, inks.len(), tool_seed)
        .iter()
        .enumerate()
    {
        let idx = idx as u32;
        let (x0, x1) = (mx[m.x as usize], mx[(m.x + m.w) as usize]);
        let (y0, y1) = (my[m.y as usize], my[(m.y + m.h) as usize]);
        let (rw, rh) = ((x1 - x0) as u32, (y1 - y0) as u32);
        let (col, col2) = (inks[m.ci], inks[m.ci2]);
        let (uw, uh) = (m.w * p.unit, m.h * p.unit);
        let ux = edges(uw, x0, x1);
        let uy = edges(uh, y0, y1);
        if m.kind != Kind::Empty && m.kind != Kind::Solid && m.kind != Kind::Grad && !m.on_bg {
            surface.fill_rect(x0, y0, rw, rh, col2);
        }
        match m.kind {
            Kind::Empty => {}
            Kind::Solid => surface.fill_rect(x0, y0, rw, rh, col),
            Kind::Grad => {
                let (x0f, y0f, x1f, y1f) =
                    (f64::from(x0), f64::from(y0), f64::from(x1), f64::from(y1));
                let (from, to) = [
                    ((x0f, y0f), (x1f, y0f)),
                    ((x0f, y0f), (x0f, y1f)),
                    ((x0f, y0f), (x1f, y1f)),
                    ((x1f, y0f), (x0f, y1f)),
                ][m.angle as usize];
                surface.fill_rect_linear(x0, y0, rw, rh, from, to, &[(0.0, col), (1.0, col2)]);
            }
            Kind::Blocks => {
                let on = |i: u32, j: u32| {
                    lit(
                        m.x * p.unit + i,
                        m.y * p.unit + j,
                        idx,
                        p.block_fill,
                        tool_seed,
                    )
                };
                for j in 0..uh {
                    let mut i = 0;
                    while i < uw {
                        if !on(i, j) {
                            i += 1;
                            continue;
                        }
                        let mut run = 1;
                        while i + run < uw && on(i + run, j) {
                            run += 1;
                        }
                        let (left, right) = (ux[i as usize], ux[(i + run) as usize]);
                        let (top, bottom) = (uy[j as usize], uy[j as usize + 1]);
                        surface.fill_rect(
                            left,
                            top,
                            (right - left) as u32,
                            (bottom - top) as u32,
                            col,
                        );
                        i += run;
                    }
                }
            }
            Kind::Dots => {
                let inset = |n: u32| if n > 2 { 2 * m.inset } else { 0 };
                let cw = 1.max(uw.saturating_sub(inset(uw)));
                let ch = 1.max(uh.saturating_sub(inset(uh)));
                let ox = if m.corner & 1 == 1 { uw - cw } else { 0 };
                let oy = if m.corner & 2 == 2 { uh - ch } else { 0 };
                for j in oy..oy + ch {
                    for i in ox..ox + cw {
                        if !lit(
                            m.x * p.unit + i,
                            m.y * p.unit + j,
                            idx + 77,
                            p.block_fill,
                            tool_seed,
                        ) {
                            continue;
                        }
                        let (l, r) = (ux[i as usize], ux[i as usize + 1]);
                        let (t, b) = (uy[j as usize], uy[j as usize + 1]);
                        let radius = f64::from((r - l).min(b - t)) * p.dot / 2.0;
                        surface.fill_circle(
                            f64::from(l + r) / 2.0,
                            f64::from(t + b) / 2.0,
                            radius,
                            col,
                        );
                    }
                }
            }
            Kind::Lines => {
                let path = rules(&ux, &uy, (x0, y0, x1, y1));
                if !path.is_empty() {
                    surface.with_alpha(0.85, |s| {
                        s.stroke(&path, col, p.rule_w, Cap::Butt, Join::Miter)
                    });
                }
            }
        }
    }

    if p.rules > 0.0 {
        let path = rules(&mx, &my, (0, 0, width, height));
        if !path.is_empty() {
            surface.with_alpha(p.rules as f32, |s| {
                s.stroke(&path, RULE, p.rule_w, Cap::Butt, Join::Miter)
            });
        }
    }
}
