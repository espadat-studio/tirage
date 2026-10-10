use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "totem";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 8;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 5] = ["#1b1b1b", "#ff6b35", "#f7b32b", "#2e86ab", "#f72585"];

const BORDER: Param = Param {
    step: 5,
    taste: (0, 300),
    ..Param::new(SLUG, "border", 0, 400, 1000)
};
const MAT: Param = Param::new(SLUG, "mat", 0, 70, 100);
const MAT_GRAIN: Param = Param::new(SLUG, "matGrain", 1, 6, 1);
const KEYLINE: Param = Param {
    taste: (0, 6),
    ..Param::new(SLUG, "keyline", 0, 10, 1)
};
const REGIONS: Param = Param {
    taste: (3, 30),
    ..Param::new(SLUG, "regions", 1, 30, 1)
};
const GRAIN: Param = Param {
    taste: (60, 220),
    ..Param::new(SLUG, "grain", 16, 220, 1)
};
const MIRROR: Param = Param::new(SLUG, "mirror", 0, 100, 100);
const VARIETY: Param = Param::new(SLUG, "variety", 0, 100, 100);
const CORE: Param = Param::new(SLUG, "core", 0, 60, 100);
const CORE_RINGS: Param = Param::new(SLUG, "coreRings", 0, 8, 1);

pub(crate) const PARAMS: &[Param] = &[
    BORDER, MAT, MAT_GRAIN, KEYLINE, REGIONS, GRAIN, MIRROR, VARIETY, CORE, CORE_RINGS,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TotemParams {
    border: f64,
    mat: f64,
    #[serde(rename = "matGrain")]
    mat_grain: u32,
    keyline: u32,
    regions: u32,
    grain: u32,
    mirror: f64,
    variety: f64,
    core: f64,
    #[serde(rename = "coreRings")]
    core_rings: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for TotemParams {
    fn default() -> Self {
        Self {
            border: 0.15,
            mat: 0.36,
            mat_grain: 2,
            keyline: 3,
            regions: 14,
            grain: 110,
            mirror: 1.0,
            variety: 0.7,
            core: 0.22,
            core_rings: 3,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl TotemParams {
    pub fn border(&self) -> f64 {
        self.border
    }

    pub fn set_border(&mut self, border: f64) -> Result<(), Error> {
        self.border = BORDER.check(border)?;
        Ok(())
    }

    pub fn mat(&self) -> f64 {
        self.mat
    }

    pub fn set_mat(&mut self, mat: f64) -> Result<(), Error> {
        self.mat = MAT.check(mat)?;
        Ok(())
    }

    pub fn mat_grain(&self) -> u32 {
        self.mat_grain
    }

    pub fn set_mat_grain(&mut self, mat_grain: u32) -> Result<(), Error> {
        MAT_GRAIN.check(f64::from(mat_grain))?;
        self.mat_grain = mat_grain;
        Ok(())
    }

    pub fn keyline(&self) -> u32 {
        self.keyline
    }

    pub fn set_keyline(&mut self, keyline: u32) -> Result<(), Error> {
        KEYLINE.check(f64::from(keyline))?;
        self.keyline = keyline;
        Ok(())
    }

    pub fn regions(&self) -> u32 {
        self.regions
    }

    pub fn set_regions(&mut self, regions: u32) -> Result<(), Error> {
        REGIONS.check(f64::from(regions))?;
        self.regions = regions;
        Ok(())
    }

    pub fn grain(&self) -> u32 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: u32) -> Result<(), Error> {
        GRAIN.check(f64::from(grain))?;
        self.grain = grain;
        Ok(())
    }

    pub fn mirror(&self) -> f64 {
        self.mirror
    }

    pub fn set_mirror(&mut self, mirror: f64) -> Result<(), Error> {
        self.mirror = MIRROR.check(mirror)?;
        Ok(())
    }

    pub fn variety(&self) -> f64 {
        self.variety
    }

    pub fn set_variety(&mut self, variety: f64) -> Result<(), Error> {
        self.variety = VARIETY.check(variety)?;
        Ok(())
    }

    pub fn core(&self) -> f64 {
        self.core
    }

    pub fn set_core(&mut self, core: f64) -> Result<(), Error> {
        self.core = CORE.check(core)?;
        Ok(())
    }

    pub fn core_rings(&self) -> u32 {
        self.core_rings
    }

    pub fn set_core_rings(&mut self, core_rings: u32) -> Result<(), Error> {
        CORE_RINGS.check(f64::from(core_rings))?;
        self.core_rings = core_rings;
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
    border: f64,
    mat: f64,
    #[serde(rename = "matGrain")]
    mat_grain: u32,
    keyline: u32,
    regions: u32,
    grain: u32,
    mirror: f64,
    variety: f64,
    core: f64,
    #[serde(rename = "coreRings")]
    core_rings: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<TotemParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = TotemParams::default();
    params.set_border(raw.border)?;
    params.set_mat(raw.mat)?;
    params.set_mat_grain(raw.mat_grain)?;
    params.set_keyline(raw.keyline)?;
    params.set_regions(raw.regions)?;
    params.set_grain(raw.grain)?;
    params.set_mirror(raw.mirror)?;
    params.set_variety(raw.variety)?;
    params.set_core(raw.core)?;
    params.set_core_rings(raw.core_rings)?;
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
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> TotemParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    TotemParams {
        border: pick(&BORDER),
        mat: pick(&MAT),
        mat_grain: pick(&MAT_GRAIN) as u32,
        keyline: pick(&KEYLINE) as u32,
        regions: pick(&REGIONS) as u32,
        grain: pick(&GRAIN) as u32,
        mirror: pick(&MIRROR),
        variety: pick(&VARIETY),
        core: pick(&CORE),
        core_rings: pick(&CORE_RINGS) as u32,
        ..TotemParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &TotemParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Motif {
    Solid,
    Check,
    Hline,
    Vline,
    Diag,
    DiagB,
    Brick,
    Dash,
    Grid,
    Rings,
    Noise,
}

const MOTIFS: [Motif; 11] = [
    Motif::Solid,
    Motif::Check,
    Motif::Hline,
    Motif::Vline,
    Motif::Diag,
    Motif::DiagB,
    Motif::Brick,
    Motif::Dash,
    Motif::Grid,
    Motif::Rings,
    Motif::Noise,
];

#[derive(Clone, Copy)]
struct Print {
    kind: Motif,
    cs: i32,
    a: [u8; 3],
    b: [u8; 3],
}

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

fn index(rng: &mut Xorshift, len: usize) -> usize {
    (rng.next() * len as f64) as usize
}

fn marked(kind: Motif, i: i32, j: i32, cols: i32, rows: i32, rng: &mut Xorshift) -> bool {
    match kind {
        Motif::Solid => true,
        Motif::Check => (i + j) % 2 == 0,
        Motif::Hline => j % 2 == 0,
        Motif::Vline => i % 2 == 0,
        Motif::Diag => (i + j) % 4 < 2,
        Motif::DiagB => (i - j).rem_euclid(4) < 2,
        Motif::Brick => (i + 2 * (j % 2)) % 4 < 2,
        Motif::Dash => j % 2 == 0 && i % 3 < 2,
        Motif::Grid => i % 3 == 0 || j % 3 == 0,
        Motif::Rings => i.min(j).min(cols - 1 - i).min(rows - 1 - j) % 2 == 0,
        Motif::Noise => rng.next() < 0.5,
    }
}

fn deal_print(rng: &mut Xorshift, bag: &[Motif], inks: &[[u8; 3]], dark: [u8; 3], u: i32) -> Print {
    let mut kind = bag[index(rng, bag.len())];
    if kind == Motif::Solid && rng.next() < 0.55 {
        kind = bag[index(rng, bag.len())];
    }
    let a = inks[index(rng, inks.len())];
    let pool: Vec<[u8; 3]> = inks
        .iter()
        .copied()
        .chain([dark, dark])
        .filter(|&c| c != a)
        .collect();
    let b = if pool.is_empty() {
        dark
    } else {
        pool[index(rng, pool.len())]
    };
    let cs = if rng.next() < 0.3 { 2 * u } else { u };
    Print { kind, cs, a, b }
}

fn print(
    surface: &mut Surface,
    rng: &mut Xorshift,
    print: Print,
    at: Rect,
    mirror_axis: Option<i32>,
) {
    let mut fill = |x: i32, y: i32, w: i32, h: i32, ink: [u8; 3]| {
        let x = mirror_axis.map_or(x, |axis| axis - x - w);
        surface.fill_rect(x, y, w as u32, h as u32, ink);
    };
    let Rect { x, y, w, h } = at;
    fill(x, y, w, h, print.b);
    if print.kind == Motif::Solid {
        fill(x, y, w, h, print.a);
        return;
    }
    let cs = print.cs;
    let cols = (round_half_up(f64::from(w) / f64::from(cs)) as i32).max(1);
    let rows = (round_half_up(f64::from(h) / f64::from(cs)) as i32).max(1);
    for j in 0..rows {
        for i in 0..cols {
            if !marked(print.kind, i, j, cols, rows, rng) {
                continue;
            }
            let (px, py) = (x + i * cs, y + j * cs);
            fill(px, py, cs.min(x + w - px), cs.min(y + h - py), print.a);
        }
    }
}

fn carve(whole: Rect, n: usize, u: i32, rng: &mut Xorshift) -> Vec<Rect> {
    let mut list = vec![whole];
    let mut guard = 0;
    while list.len() < n && guard < 400 {
        guard += 1;
        let mut best = 0;
        for (i, r) in list.iter().enumerate() {
            let best_area = i64::from(list[best].w) * i64::from(list[best].h);
            if i64::from(r.w) * i64::from(r.h) > best_area {
                best = i;
            }
        }
        let t = list[best];
        let (can_v, can_h) = (t.w >= u * 6, t.h >= u * 6);
        if !can_v && !can_h {
            break;
        }
        let (fw, fh) = (f64::from(t.w), f64::from(t.h));
        let vert = if can_v && can_h {
            if fw / fh > 1.1 {
                true
            } else if fh / fw > 1.1 {
                false
            } else {
                rng.next() < 0.5
            }
        } else {
            can_v
        };
        let frac = 0.3 + rng.next() * 0.4;
        list.remove(best);
        let side = if vert { t.w } else { t.h };
        let cut = (u * 2).max(round_half_up(f64::from(side) * frac / f64::from(u)) as i32 * u);
        if cut <= 0 || cut >= side {
            list.push(t);
            break;
        }
        if vert {
            list.push(Rect { w: cut, ..t });
            list.push(Rect {
                x: t.x + cut,
                w: t.w - cut,
                ..t
            });
        } else {
            list.push(Rect { h: cut, ..t });
            list.push(Rect {
                y: t.y + cut,
                h: t.h - cut,
                ..t
            });
        }
    }
    list
}

fn luma([r, g, b]: [u8; 3]) -> f64 {
    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114
}

fn paint(surface: &mut Surface, p: &TotemParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width() as i32, surface.height() as i32);
    let all = palette.inks();
    let mut di = 0;
    for (i, &ink) in all.iter().enumerate().skip(1) {
        if luma(ink) < luma(all[di]) {
            di = i;
        }
    }
    let dark = all[di];
    let mut inks: Vec<[u8; 3]> = all
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != di)
        .map(|(_, &c)| c)
        .collect();
    if inks.is_empty() {
        inks.push(dark);
    }
    let mat = inks[0];
    let mark = if inks.len() > 1 {
        inks[inks.len() - 2]
    } else {
        dark
    };
    let mut rng = Xorshift::from_state(tool_seed);

    let short = w.min(h);
    let u = (round_half_up(f64::from(short) / f64::from(p.grain)) as i32).max(2);
    let snap = |v: f64| round_half_up(v / f64::from(u)) as i32 * u;

    surface.fill(mat);
    let bw = snap(f64::from(short) * p.border);
    let panel = Rect {
        x: bw,
        y: bw,
        w: snap(f64::from(w - bw * 2)),
        h: snap(f64::from(h - bw * 2)),
    };

    if bw > 0 && p.mat > 0.01 {
        let mu = u * p.mat_grain as i32;
        let (gw, gh) = ((w + mu - 1) / mu + 1, (h + mu - 1) / mu + 1);
        let at = |x: i32, y: i32| (y * gw + x) as usize;
        let mut g: Vec<bool> = (0..gw * gh).map(|_| rng.next() < p.mat).collect();
        for _ in 0..2 {
            let mut next = vec![false; g.len()];
            for y in 0..gh {
                for x in 0..gw {
                    let mut sum = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let (xx, yy) = (x + dx, y + dy);
                            let on = if xx < 0 || yy < 0 || xx >= gw || yy >= gh {
                                g[at(x, y)]
                            } else {
                                g[at(xx, yy)]
                            };
                            sum += i32::from(on);
                        }
                    }
                    next[at(x, y)] = match sum {
                        5.. => true,
                        4 => g[at(x, y)],
                        _ => false,
                    };
                }
            }
            g = next;
        }
        for y in 0..gh {
            for x in 0..gw {
                let (rx, ry) = (x * mu, y * mu);
                let inside = rx + mu > panel.x
                    && rx < panel.x + panel.w
                    && ry + mu > panel.y
                    && ry < panel.y + panel.h;
                if g[at(x, y)] && !inside {
                    surface.fill_rect(rx, ry, mu as u32, mu as u32, mark);
                }
            }
        }
    }

    if panel.w < u * 4 || panel.h < u * 4 {
        return;
    }

    surface.fill_rect(panel.x, panel.y, panel.w as u32, panel.h as u32, dark);
    let k = p.keyline as i32 * u;
    let (cx, cy) = (panel.x + k, panel.y + k);
    let (cw, ch) = (u.max(panel.w - k * 2), u.max(panel.h - k * 2));

    let n = (round_half_up(2.0 + p.variety * 9.0) as usize).max(2);
    let mut pool = MOTIFS.to_vec();
    let bag: Vec<Motif> = (0..n.min(MOTIFS.len()))
        .map(|_| pool.remove(index(&mut rng, pool.len())))
        .collect();

    let half_w = u.max((f64::from(cw) / f64::from(u) / 2.0).ceil() as i32 * u);
    let whole = Rect {
        x: cx,
        y: cy,
        w: half_w,
        h: ch,
    };
    for g in carve(whole, p.regions as usize, u, &mut rng) {
        let left = deal_print(&mut rng, &bag, &inks, dark, u);
        let draw_w = g.w.min(cx + cw - g.x);
        let original = Rect { w: draw_w, ..g };
        if draw_w > 0 {
            print(surface, &mut rng, left, original, None);
        }
        let mx = cx + cw - (g.x - cx) - g.w;
        let twin = if rng.next() < p.mirror {
            left
        } else {
            deal_print(&mut rng, &bag, &inks, dark, u)
        };
        let clip_x = mx.max(cx + cw - half_w);
        let tw = (g.w - (clip_x - mx)).min(cx + cw - clip_x);
        if tw > 0 {
            print(surface, &mut rng, twin, Rect { x: 0, ..g }, Some(mx + g.w));
            if draw_w > 0 {
                print(surface, &mut rng, left, original, None);
            }
        }
    }

    if p.core > 0.01 {
        let mut kw = snap(f64::from(cw.min(ch)) * p.core);
        let mut kh = snap(f64::from(kw) * (1.2 + 0.6 * f64::from(tool_seed % 5) / 5.0));
        kw = (u * 2).max(kw.min(cw - u * 2));
        kh = (u * 2).max(kh.min(ch - u * 2));
        let mut kx = snap(f64::from(cx) + f64::from(cw - kw) / 2.0);
        let mut ky = snap(f64::from(cy) + f64::from(ch - kh) / 2.0);
        for i in 0..p.core_rings as usize {
            let ink = if i % 2 == 1 {
                inks[(i + 1) % inks.len()]
            } else {
                dark
            };
            surface.fill_rect(kx, ky, kw as u32, kh as u32, ink);
            kx += u;
            ky += u;
            kw -= u * 2;
            kh -= u * 2;
            if kw < u * 2 || kh < u * 2 {
                kw = kw.max(u);
                kh = kh.max(u);
                break;
            }
        }
        let emblem = deal_print(&mut rng, &bag, &inks, dark, u);
        let at = Rect {
            x: kx,
            y: ky,
            w: kw,
            h: kh,
        };
        print(
            surface,
            &mut rng,
            Print {
                cs: u,
                b: dark,
                ..emblem
            },
            at,
            None,
        );
    }
}
