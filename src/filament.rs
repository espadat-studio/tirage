use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::aura::{Xorshift, hash};
use crate::chassis::{self, Blend, Dither, DitherKind, Grain};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "filament";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 16;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 3] = ["#ff5da2", "#ff9acb", "#ffe6f1"];
const GROUND: [u8; 3] = [0x12, 0x0a, 0x1e];
const ACCENTS: [[u8; 3]; 3] = [[0xf9, 0xf8, 0x71], [0x4c, 0xe0, 0xd2], [0xb0, 0x9c, 0xff]];
const VB: f64 = 1000.0;

const ZOOM: Param = Param {
    step: 5,
    ..Param::new(SLUG, "zoom", 30, 600, 100)
};
const TURN: Param = Param {
    step: 5,
    ..Param::new(SLUG, "turn", 20, 500, 100)
};
const OCT: Param = Param::new(SLUG, "oct", 1, 6, 1);
const CURL: Param = Param::new(SLUG, "curl", 0, 100, 100);
const TANGLE: Param = Param::new(SLUG, "tangle", 0, 100, 100);
const BUNDLES: Param = Param {
    taste: (23, 90),
    ..Param::new(SLUG, "bundles", 1, 90, 1)
};
const PER: Param = Param::new(SLUG, "per", 1, 40, 1);
const CLUMP: Param = Param::new(SLUG, "clump", 0, 100, 100);
const TIGHT: Param = Param {
    step: 5,
    ..Param::new(SLUG, "tight", 5, 200, 1000)
};
const LEN: Param = Param {
    step: 10,
    taste: (160, 600),
    ..Param::new(SLUG, "len", 20, 600, 1)
};
const STEP: Param = Param {
    step: 2,
    ..Param::new(SLUG, "step", 10, 100, 10)
};
const WGT: Param = Param::new(SLUG, "wgt", 2, 80, 10);
const WVAR: Param = Param::new(SLUG, "wvar", 0, 100, 100);
const HIER: Param = Param::new(SLUG, "hier", 0, 100, 100);
const HAIR: Param = Param::new(SLUG, "hair", 0, 100, 100);
const MARK: Param = Param::new(SLUG, "mark", 0, 100, 100);
const BEAD: Param = Param::new(SLUG, "bead", 0, 100, 100);
const BGAP: Param = Param::new(SLUG, "bgap", 2, 14, 1);
const MSIZE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "msize", 20, 260, 10)
};

pub(crate) const PARAMS: &[Param] = &[
    ZOOM, TURN, OCT, CURL, TANGLE, BUNDLES, PER, CLUMP, TIGHT, LEN, STEP, WGT, WVAR, HIER, HAIR,
    MARK, BEAD, BGAP, MSIZE,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilamentMarks {
    Symbols,
    Beads,
    Both,
}

const MSTYLES: [FilamentMarks; 3] = [
    FilamentMarks::Symbols,
    FilamentMarks::Beads,
    FilamentMarks::Both,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FilamentParams {
    zoom: f64,
    turn: f64,
    oct: u32,
    curl: f64,
    tangle: f64,
    bundles: u32,
    per: u32,
    clump: f64,
    tight: f64,
    len: u32,
    step: f64,
    wgt: f64,
    wvar: f64,
    hier: f64,
    hair: f64,
    mstyles: FilamentMarks,
    mark: f64,
    bead: f64,
    bgap: u32,
    msize: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for FilamentParams {
    fn default() -> Self {
        Self {
            zoom: 1.9,
            turn: 1.9,
            oct: 3,
            curl: 0.45,
            tangle: 0.0,
            bundles: 26,
            per: 12,
            clump: 0.6,
            tight: 0.04,
            len: 230,
            step: 3.4,
            wgt: 1.6,
            wvar: 0.6,
            hier: 0.55,
            hair: 0.22,
            mstyles: FilamentMarks::Symbols,
            mark: 0.3,
            bead: 0.28,
            bgap: 4,
            msize: 9.0,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

macro_rules! real {
    ($($get:ident, $set:ident, $param:ident;)*) => {$(
        pub fn $get(&self) -> f64 {
            self.$get
        }

        pub fn $set(&mut self, value: f64) -> Result<(), Error> {
            self.$get = $param.check(value)?;
            Ok(())
        }
    )*};
}

macro_rules! whole {
    ($($get:ident, $set:ident, $param:ident;)*) => {$(
        pub fn $get(&self) -> u32 {
            self.$get
        }

        pub fn $set(&mut self, value: u32) -> Result<(), Error> {
            $param.check(f64::from(value))?;
            self.$get = value;
            Ok(())
        }
    )*};
}

#[expect(
    clippy::len_without_is_empty,
    reason = "len is the site's strand length slider"
)]
impl FilamentParams {
    real! {
        zoom, set_zoom, ZOOM;
        turn, set_turn, TURN;
        curl, set_curl, CURL;
        tangle, set_tangle, TANGLE;
        clump, set_clump, CLUMP;
        tight, set_tight, TIGHT;
        step, set_step, STEP;
        wgt, set_wgt, WGT;
        wvar, set_wvar, WVAR;
        hier, set_hier, HIER;
        hair, set_hair, HAIR;
        mark, set_mark, MARK;
        bead, set_bead, BEAD;
        msize, set_msize, MSIZE;
    }

    whole! {
        oct, set_oct, OCT;
        bundles, set_bundles, BUNDLES;
        per, set_per, PER;
        len, set_len, LEN;
        bgap, set_bgap, BGAP;
    }

    pub fn mstyles(&self) -> FilamentMarks {
        self.mstyles
    }

    pub fn set_mstyles(&mut self, mstyles: FilamentMarks) {
        self.mstyles = mstyles;
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
    zoom: f64,
    turn: f64,
    oct: u32,
    curl: f64,
    tangle: f64,
    bundles: u32,
    per: u32,
    clump: f64,
    tight: f64,
    len: u32,
    step: f64,
    wgt: f64,
    wvar: f64,
    hier: f64,
    hair: f64,
    mstyles: FilamentMarks,
    mark: f64,
    bead: f64,
    bgap: u32,
    msize: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<FilamentParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = FilamentParams::default();
    params.set_zoom(raw.zoom)?;
    params.set_turn(raw.turn)?;
    params.set_oct(raw.oct)?;
    params.set_curl(raw.curl)?;
    params.set_tangle(raw.tangle)?;
    params.set_bundles(raw.bundles)?;
    params.set_per(raw.per)?;
    params.set_clump(raw.clump)?;
    params.set_tight(raw.tight)?;
    params.set_len(raw.len)?;
    params.set_step(raw.step)?;
    params.set_wgt(raw.wgt)?;
    params.set_wvar(raw.wvar)?;
    params.set_hier(raw.hier)?;
    params.set_hair(raw.hair)?;
    params.set_mstyles(raw.mstyles);
    params.set_mark(raw.mark)?;
    params.set_bead(raw.bead)?;
    params.set_bgap(raw.bgap)?;
    params.set_msize(raw.msize)?;
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
    let (before, after) = PARAMS.split_at(15);
    before
        .iter()
        .map(Param::parameter)
        .chain(std::iter::once(Parameter::choice(
            "mstyles",
            &["Symbols", "Beads", "Both"],
        )))
        .chain(after.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> FilamentParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    FilamentParams {
        zoom: pick(&ZOOM),
        turn: pick(&TURN),
        oct: pick(&OCT) as u32,
        curl: pick(&CURL),
        tangle: pick(&TANGLE),
        bundles: pick(&BUNDLES) as u32,
        per: pick(&PER) as u32,
        clump: pick(&CLUMP),
        tight: pick(&TIGHT),
        len: pick(&LEN) as u32,
        step: pick(&STEP),
        wgt: pick(&WGT),
        wvar: pick(&WVAR),
        hier: pick(&HIER),
        hair: pick(&HAIR),
        mstyles: MSTYLES[(draw("mstyles") % MSTYLES.len() as u64) as usize],
        mark: pick(&MARK),
        bead: pick(&BEAD),
        bgap: pick(&BGAP) as u32,
        msize: pick(&MSIZE),
        ..FilamentParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &FilamentParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Kind {
    Bead,
    Arrow,
    Ring,
    Cross,
    Hook,
    Tick,
}

const KINDS: [Kind; 5] = [Kind::Arrow, Kind::Ring, Kind::Cross, Kind::Hook, Kind::Tick];

struct Strand {
    points: Vec<(f64, f64)>,
    ink: [u8; 3],
    width: f64,
}

struct Mark {
    x: f64,
    y: f64,
    heading: f64,
    kind: Kind,
    ink: [u8; 3],
    size: f64,
}

fn noise(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let fade = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash(xi.wrapping_add(dx), yi.wrapping_add(dy), 0, seed);
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    lerp(
        lerp(corner(0, 0), corner(1, 0), u),
        lerp(corner(0, 1), corner(1, 1), u),
        v,
    )
}

fn fbm(x: f64, y: f64, seed: u32, octaves: u32) -> f64 {
    let (mut sum, mut weight, mut total, mut frequency) = (0.0, 0.5, 0.0, 1.0);
    for octave in 0..octaves {
        sum += weight
            * noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 1319),
            );
        total += weight;
        weight *= 0.5;
        frequency *= 2.0;
    }
    sum / total
}

fn trace(p: &FilamentParams, palette: &Palette, h: f64, s: u32) -> (Vec<Strand>, Vec<Mark>) {
    let start = (f64::from(s) * 2_654_435_761.0 + 13.0).rem_euclid(4_294_967_296.0) as u32;
    let mut rng = Xorshift::from_state(start);
    let mut r = || rng.next();
    let angle_at = |x: f64, y: f64| {
        let (u, v) = (x / VB * p.zoom, y / h * p.zoom * (h / VB));
        let a = fbm(u, v, s, p.oct);
        let b = fbm(u + 31.7, v - 17.3, s.wrapping_add(404), p.oct);
        a * TAU * p.turn + (b - 0.5) * TAU * p.curl
    };
    let inks = palette.len();
    let symbols = matches!(p.mstyles, FilamentMarks::Symbols | FilamentMarks::Both);
    let beads = matches!(p.mstyles, FilamentMarks::Beads | FilamentMarks::Both);
    let density = if symbols { p.mark } else { 0.0 };
    let margin = VB.max(h) * 0.12;
    let spread = p.tight * VB.min(h);
    let (mut strands, mut marks) = (Vec::new(), Vec::new());
    let (mut placed, mut tries) = (0, 0);
    while placed < p.bundles && tries < p.bundles * 40 {
        tries += 1;
        let bx = (-0.1 + 1.2 * r()) * VB;
        let by = (-0.1 + 1.2 * r()) * h;
        if p.clump > 0.0 {
            let fv = fbm(bx / VB * 2.2, by / h * 2.2, s.wrapping_add(733), 2);
            if r() > fv.powf(1.0 + p.clump * 5.0) {
                continue;
            }
        }
        placed += 1;
        r();
        for strand in 0..p.per {
            let (mut x, mut y) = (
                bx + (r() - 0.5) * 2.0 * spread,
                by + (r() - 0.5) * 2.0 * spread,
            );
            let mut points = vec![(x, y)];
            let t = p.tangle;
            let (ox, oy) = ((r() - 0.5) * 900.0 * t, (r() - 0.5) * 900.0 * t);
            let (bias, lean) = (r() * TAU, t * (0.3 + r() * 0.9));
            let sid = (placed * 997 + strand) as i32;
            for i in 0..p.len {
                let mut a = angle_at(x + ox, y + oy);
                if t > 0.0 {
                    let jitter = hash(i as i32, sid, 3, s.wrapping_add(55)) - 0.5;
                    a = (a.sin() + bias.sin() * lean).atan2(a.cos() + bias.cos() * lean)
                        + jitter * t * 0.55;
                }
                x += a.cos() * p.step;
                y += a.sin() * p.step;
                if x < -margin || y < -margin || x > VB + margin || y > h + margin {
                    break;
                }
                points.push((x, y));
            }
            if points.len() < 6 {
                continue;
            }
            let hairline = r() < p.hair;
            let ink = if hairline {
                inks - 1
            } else {
                (r().powf(1.7) * inks as f64) as usize
            };
            let heavy = r().powf(1.0 + p.hier * 4.0);
            let width = if hairline { 0.45 } else { 1.0 }
                * p.wgt
                * (1.0 - p.wvar * 0.5 + p.wvar * r())
                * (1.0 + p.hier * 2.4 * heavy);
            r();
            r();
            r();
            let n = points.len();
            if beads && r() < p.bead {
                let gap = p.bgap.max(2) as usize;
                for i in (2..n - 1).step_by(gap) {
                    let ((qx, qy), (nx, ny)) = (points[i], points[i + 1]);
                    let (dx, dy) = (nx - qx, ny - qy);
                    let length = match dx.hypot(dy) {
                        0.0 => 1.0,
                        length => length,
                    };
                    let off = (r() - 0.5) * p.msize * 0.5;
                    let ink = palette.ink((r() * inks as f64) as usize);
                    marks.push(Mark {
                        x: qx - dy / length * off,
                        y: qy + dx / length * off,
                        heading: dy.atan2(dx),
                        kind: Kind::Bead,
                        ink,
                        size: p.msize * (0.3 + r() * 0.55),
                    });
                    r();
                }
            }
            let symbol =
                |(x, y): (f64, f64), heading: f64, size: (f64, f64), r: &mut dyn FnMut() -> f64| {
                    let kind = KINDS[(r() * KINDS.len() as f64) as usize];
                    let ink = ACCENTS[(r() * ACCENTS.len() as f64) as usize];
                    let size = p.msize * (size.0 + r() * size.1);
                    r();
                    Mark {
                        x,
                        y,
                        heading,
                        kind,
                        ink,
                        size,
                    }
                };
            for i in (4..n).step_by(6) {
                if r() > density * (0.25 + i as f64 / n as f64) {
                    continue;
                }
                let (q, next) = (points[i], points[(i + 1).min(n - 1)]);
                marks.push(symbol(
                    q,
                    (next.1 - q.1).atan2(next.0 - q.0),
                    (0.6, 0.8),
                    &mut r,
                ));
            }
            if symbols && r() < density * 1.4 {
                let (q, before) = (points[n - 1], points[n - 2]);
                marks.push(symbol(
                    q,
                    (q.1 - before.1).atan2(q.0 - before.0),
                    (0.9, 0.9),
                    &mut r,
                ));
            }
            strands.push(Strand {
                points,
                ink: palette.ink(ink.min(inks - 1)),
                width: width.max(0.15),
            });
        }
    }
    (strands, marks)
}

fn paint(surface: &mut Surface, p: &FilamentParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let k = w / VB;
    let (strands, marks) = trace(p, palette, VB * h / w, tool_seed);
    surface.fill(GROUND);
    for strand in &strands {
        let mut path = Path2D::default();
        let (x, y) = strand.points[0];
        path.move_to(x * k, y * k);
        for &(x, y) in &strand.points[1..] {
            path.line_to(x * k, y * k);
        }
        let width = (strand.width * k).max(0.25);
        surface.stroke(&path, strand.ink, width, Cap::Round, Join::Round);
    }
    let line = (p.wgt * 0.9 * k).max(0.5);
    for mark in &marks {
        let (x, y, size, a) = (mark.x * k, mark.y * k, mark.size * k, mark.heading);
        let (ca, sa) = (a.cos(), a.sin());
        let at = |dx: f64, dy: f64| (x + dx * ca - dy * sa, y + dx * sa + dy * ca);
        let mut path = Path2D::default();
        let mut segment = |from: (f64, f64), to: (f64, f64)| {
            path.move_to(from.0, from.1);
            path.line_to(to.0, to.1);
        };
        match mark.kind {
            Kind::Bead => {
                surface.fill_circle(x, y, (size * 0.5).max(0.4), mark.ink);
                continue;
            }
            Kind::Arrow => {
                segment(at(size * 0.5, 0.0), at(-size * 0.2, size * 0.38));
                let (x3, y3) = at(-size * 0.2, -size * 0.38);
                path.line_to(x3, y3);
                path.close();
                surface.fill_transformed(&path, mark.ink, Transform::IDENTITY);
                continue;
            }
            Kind::Ring => path.push_circle(x, y, size * 0.34),
            Kind::Cross => {
                segment(at(-size * 0.4, -size * 0.4), at(size * 0.4, size * 0.4));
                segment(at(-size * 0.4, size * 0.4), at(size * 0.4, -size * 0.4));
            }
            Kind::Hook => {
                let radius = size * 0.36;
                path.ellipse(x, y, radius, radius, 0.0, a - 0.4, a + 2.4);
            }
            Kind::Tick => segment(at(0.0, -size * 0.45), at(0.0, size * 0.45)),
        }
        surface.stroke(&path, mark.ink, line, Cap::Round, Join::Round);
    }
}
