use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store,
};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "carve";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#1a1a1a", "#f5f2ec", "#ff5c00", "#00a3ff", "#ff00a8", "#a8ff00",
];

const CUTS: Param = Param::new(SLUG, "cuts", 1, 16, 1);
const UNEVEN: Param = Param::new(SLUG, "uneven", 0, 100, 100);
const GAP: Param = Param::new(SLUG, "gap", 0, 100, 100);
const MIX: Param = Param::new(SLUG, "mix", 0, 100, 100);
const PITCH: Param = Param::new(SLUG, "pitch", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);
const NODES: Param = Param::new(SLUG, "nodes", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[CUTS, UNEVEN, GAP, MIX, PITCH, NODES];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CarveParams {
    cuts: u32,
    uneven: f64,
    gap: f64,
    mix: f64,
    pitch: f64,
    grain: f64,
    nodes: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for CarveParams {
    fn default() -> Self {
        Self {
            cuts: 7,
            uneven: 0.55,
            gap: 0.0,
            mix: 0.7,
            pitch: 0.4,
            grain: 0.5,
            nodes: 0.5,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl CarveParams {
    pub fn cuts(&self) -> u32 {
        self.cuts
    }

    pub fn set_cuts(&mut self, cuts: u32) -> Result<(), Error> {
        CUTS.check(f64::from(cuts))?;
        self.cuts = cuts;
        Ok(())
    }

    pub fn uneven(&self) -> f64 {
        self.uneven
    }

    pub fn set_uneven(&mut self, uneven: f64) -> Result<(), Error> {
        self.uneven = UNEVEN.check(uneven)?;
        Ok(())
    }

    pub fn gap(&self) -> f64 {
        self.gap
    }

    pub fn set_gap(&mut self, gap: f64) -> Result<(), Error> {
        self.gap = GAP.check(gap)?;
        Ok(())
    }

    pub fn mix(&self) -> f64 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f64) -> Result<(), Error> {
        self.mix = MIX.check(mix)?;
        Ok(())
    }

    pub fn pitch(&self) -> f64 {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: f64) -> Result<(), Error> {
        self.pitch = PITCH.check(pitch)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn nodes(&self) -> f64 {
        self.nodes
    }

    pub fn set_nodes(&mut self, nodes: f64) -> Result<(), Error> {
        self.nodes = NODES.check(nodes)?;
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
    cuts: u32,
    uneven: f64,
    gap: f64,
    mix: f64,
    pitch: f64,
    grain: f64,
    nodes: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<CarveParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = CarveParams::default();
    params.set_cuts(raw.cuts)?;
    params.set_uneven(raw.uneven)?;
    params.set_gap(raw.gap)?;
    params.set_mix(raw.mix)?;
    params.set_pitch(raw.pitch)?;
    params.set_grain(raw.grain)?;
    params.set_nodes(raw.nodes)?;
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
    [&CUTS, &UNEVEN, &GAP, &MIX, &PITCH, &GRAIN, &NODES]
        .into_iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> CarveParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    CarveParams {
        cuts: pick(&CUTS) as u32,
        uneven: pick(&UNEVEN),
        gap: pick(&GAP),
        mix: pick(&MIX),
        pitch: pick(&PITCH),
        nodes: pick(&NODES),
        ..CarveParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &CarveParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Flat,
    Stripe,
    Chevron,
    Ramp,
    Grid,
}

struct Panel {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    kind: Kind,
    a: usize,
    b: usize,
    dir: bool,
    brk: f64,
    seed: u32,
}

fn layout(p: &CarveParams, n: usize, aspect: f64, tool_seed: u32) -> Vec<Panel> {
    let mut rnd = XorShift::new(tool_seed.wrapping_mul(2_654_435_761));
    let mut rects: Vec<(f64, f64, f64, f64)> = vec![(0.0, 0.0, 1.0, 1.0)];
    for _ in 0..p.cuts.max(1) {
        let mut order: Vec<usize> = (0..rects.len()).collect();
        order.sort_by(|&i, &j| {
            let area = |k: usize| rects[k].2 * rects[k].3;
            area(j).total_cmp(&area(i))
        });
        let rank = (rnd.next() * rnd.next() * 3.0) as usize;
        let pick = order[rank.min(order.len() - 1)];
        let (x, y, w, h) = rects.remove(pick);
        let wide = w >= h * aspect;
        let vertical = if rnd.next() < 0.18 { !wide } else { wide };
        let t = 0.5 + (rnd.next() - 0.5) * p.uneven.min(0.92) * 0.86;
        if vertical {
            rects.push((x, y, w * t, h));
            rects.push((x + w * t, y, w * (1.0 - t), h));
        } else {
            rects.push((x, y, w, h * t));
            rects.push((x, y + h * t, w, h * (1.0 - t)));
        }
    }
    rects.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));

    let mut grid_left = true;
    rects
        .into_iter()
        .enumerate()
        .map(|(i, (x, y, w, h))| {
            let start = tool_seed.wrapping_add((i as u32).wrapping_mul(7919));
            let mut rng = XorShift::new(start.wrapping_mul(2_246_822_519));
            let mut kind = Kind::Flat;
            if rng.next() < p.mix {
                let roll = rng.next();
                kind = if grid_left && roll > 0.86 && w * h < 0.3 {
                    grid_left = false;
                    Kind::Grid
                } else if roll < 0.34 {
                    Kind::Stripe
                } else if roll < 0.6 {
                    Kind::Chevron
                } else {
                    Kind::Ramp
                };
            }
            let a = (rng.next() * n as f64) as usize;
            let b = (a + 1 + (rng.next() * (n - 1) as f64) as usize) % n;
            Panel {
                x,
                y,
                w,
                h,
                kind,
                a,
                b,
                dir: rng.next() >= 0.5,
                brk: 0.3 + rng.next() * 0.45,
                seed: (rng.next() * 99999.0) as u32,
            }
        })
        .collect()
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Path2D {
    let mut path = Path2D::default();
    path.move_to(x, y);
    path.line_to(x + w, y);
    path.line_to(x + w, y + h);
    path.line_to(x, y + h);
    path.close();
    path
}

fn paint(surface: &mut Surface, p: &CarveParams, palette: &Palette, tool_seed: u32) {
    let (fw, fh) = (f64::from(surface.width()), f64::from(surface.height()));
    let inks = palette.inks();
    surface.fill(inks[0]);
    let gap = p.gap * fw.min(fh) * 0.02;
    for panel in layout(p, inks.len(), fh / fw, tool_seed) {
        let x = panel.x * fw + gap / 2.0;
        let y = panel.y * fh + gap / 2.0;
        let w = (panel.w * fw - gap).max(1.0);
        let h = (panel.h * fh - gap).max(1.0);
        let (a, b) = (inks[panel.a], inks[panel.b]);
        match panel.kind {
            Kind::Flat => surface.fill_box(x, y, w, h, a),
            Kind::Stripe => stripe(surface, p, &panel, (x, y, w, h), a, b),
            Kind::Chevron => chevron(surface, &panel, (x, y, w, h), a, b),
            Kind::Ramp => ramp(surface, p, &panel, (x, y, w, h), a, b),
            Kind::Grid => grid(surface, p, &panel, (x, y, w, h), a, b),
        }
    }
}

type Area = (f64, f64, f64, f64);

fn stripe(
    surface: &mut Surface,
    p: &CarveParams,
    panel: &Panel,
    (x, y, w, h): Area,
    a: [u8; 3],
    b: [u8; 3],
) {
    surface.fill_box(x, y, w, h, a);
    let span = if panel.dir { h } else { w };
    let base = ((0.006 + p.pitch * 0.075) * w.min(h) + 2.0).max(3.0);
    let brk = if panel.dir { w } else { h } * panel.brk;
    for pass in 0..2 {
        let pitch = base * if pass == 1 { 1.9 } else { 1.0 };
        let n = (span / (pitch * 2.0)).ceil() as i32 + 2;
        for i in -1..n {
            let o = f64::from(i) * pitch * 2.0 + f64::from(panel.seed % 7) / 7.0 * pitch;
            if o + pitch < 0.0 {
                continue;
            }
            if panel.dir {
                let yy = y + o;
                if yy > y + h {
                    break;
                }
                let (x0, x1) = if pass == 1 {
                    (x + brk, x + w)
                } else {
                    (x, x + brk)
                };
                if x1 > x0 {
                    surface.fill_box(x0, yy, x1 - x0, pitch.min(y + h - yy), b);
                }
            } else {
                let xx = x + o;
                if xx > x + w {
                    break;
                }
                let (y0, y1) = if pass == 1 {
                    (y + brk, y + h)
                } else {
                    (y, y + brk)
                };
                if y1 > y0 {
                    surface.fill_box(xx, y0, pitch.min(x + w - xx), y1 - y0, b);
                }
            }
        }
    }
}

fn chevron(surface: &mut Surface, panel: &Panel, (x, y, w, h): Area, a: [u8; 3], b: [u8; 3]) {
    surface.fill_box(x, y, w, h, a);
    let rows = round_half_up(h / 14.0_f64.max(w.min(h) * 0.55)).max(2.0);
    let step = h / rows;
    let (near, far) = if panel.dir { (x, x + w) } else { (x + w, x) };
    let t = 0.42;
    surface.with_clip(&rect(x, y, w, h), Transform::IDENTITY, |surface| {
        for j in 0..rows as u32 {
            let y0 = y + f64::from(j) * step;
            let mut path = Path2D::default();
            path.move_to(near, y0);
            path.line_to(far, y0 + step * 0.5);
            path.line_to(near, y0 + step);
            path.line_to(near, y0 + step * (1.0 - t));
            path.line_to(far - (far - near) * (1.0 - t), y0 + step * 0.5);
            path.line_to(near, y0 + step * t);
            path.close();
            surface.fill_path(&path.into_path(), b);
        }
    });
}

fn ramp(
    surface: &mut Surface,
    p: &CarveParams,
    panel: &Panel,
    (x, y, w, h): Area,
    a: [u8; 3],
    b: [u8; 3],
) {
    let (bw, bh) = (round_half_up(w).max(1.0), round_half_up(h).max(1.0));
    let (bx, by) = (round_half_up(x), round_half_up(y));
    let angle = f64::from(panel.seed % 360) * PI / 180.0;
    let (ux, uy) = (angle.cos(), angle.sin());
    let sd = f64::from(panel.seed);
    let (width, height) = (surface.width(), surface.height());
    for j in 0..bh as u32 {
        let py = by + f64::from(j);
        for i in 0..bw as u32 {
            let px = bx + f64::from(i);
            let u = f64::from(i) / bw - 0.5;
            let v = f64::from(j) / bh - 0.5;
            let mut t = 0.5 + (u * ux + v * uy);
            t += ((u * uy - v * ux) * 11.0 + sd).sin() * 0.06;
            let t = t.clamp(0.0, 1.0);
            let noise = if p.grain > 0.002 {
                (hash(i as i32, j as i32, panel.seed) - 0.5)
                    * p.grain
                    * 104.0
                    * (1.0 - (t - 0.5).abs() * 1.1)
            } else {
                0.0
            };
            if px < 0.0 || py < 0.0 || px >= f64::from(width) || py >= f64::from(height) {
                continue;
            }
            let ink = std::array::from_fn(|c| {
                let (ca, cb) = (f64::from(a[c]), f64::from(b[c]));
                store(ca + (cb - ca) * t + noise)
            });
            surface.set_pixel(px as u32, py as u32, ink);
        }
    }
}

fn grid(
    surface: &mut Surface,
    p: &CarveParams,
    panel: &Panel,
    (x, y, w, h): Area,
    a: [u8; 3],
    b: [u8; 3],
) {
    surface.fill_box(x, y, w, h, a);
    let side = w.min(h);
    let step = 14.0_f64.max(side / (2.0 + round_half_up(p.nodes * 6.0)));
    let lines = |from: f64, to: f64| {
        let mut at = Vec::new();
        let mut g = from;
        while g <= to + 0.5 {
            at.push(g);
            g += step;
        }
        at
    };
    let (xs, ys) = (lines(x, x + w), lines(y, y + h));
    surface.with_clip(&rect(x, y, w, h), Transform::IDENTITY, |surface| {
        let mut path = Path2D::default();
        for &gx in &xs {
            path.move_to(gx, y);
            path.line_to(gx, y + h);
        }
        for &gy in &ys {
            path.move_to(x, gy);
            path.line_to(x + w, gy);
        }
        surface.stroke(&path, b, (side * 0.004).max(1.0), Cap::Butt, Join::Miter);
        let radius = (side * 0.016).max(1.6);
        for (i, &gx) in xs.iter().enumerate() {
            for (j, &gy) in ys.iter().enumerate() {
                if hash(i as i32, j as i32, panel.seed) > 0.82 {
                    surface.fill_circle(gx, gy, radius, b);
                }
            }
        }
    });
}
