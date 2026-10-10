use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, hash, store, value_noise};
use crate::param::Param;
use crate::surface::{Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "strand";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const INKS: usize = 3;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#ffd166", "#1b1b1b", "#ef476f"];
const ROD_CAP: usize = 7000;
const STATIONS: usize = 18;
const TURN: f64 = 1.75;

const COUNT: Param = Param {
    taste: (4, 20),
    ..Param::new(SLUG, "count", 1, 40, 1)
};
const LEN: Param = Param {
    taste: (6, 32),
    ..Param::new(SLUG, "len", 3, 90, 1)
};
const WANDER: Param = Param::new(SLUG, "wander", 0, 100, 100);
const BRANCH: Param = Param::new(SLUG, "branch", 0, 100, 100);
const THICK: Param = Param {
    taste: (0, 80),
    ..Param::new(SLUG, "thick", 0, 100, 100)
};
const ROD: Param = Param::new(SLUG, "rod", 0, 100, 100);
const NOTCH: Param = Param::new(SLUG, "notch", 0, 100, 100);
const ROUGH: Param = Param::new(SLUG, "rough", 0, 100, 100);
const OFFSET: Param = Param::new(SLUG, "offset", 0, 100, 100);
const EDGE: Param = Param::new(SLUG, "edge", 0, 100, 100);
const TEX: Param = Param::new(SLUG, "tex", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    COUNT, LEN, WANDER, BRANCH, THICK, ROD, NOTCH, ROUGH, OFFSET, EDGE, TEX,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrandTexture {
    Stipple,
    Drag,
    Screen,
}

const TEXTURES: [StrandTexture; 3] = [
    StrandTexture::Stipple,
    StrandTexture::Drag,
    StrandTexture::Screen,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StrandParams {
    count: u32,
    len: u32,
    wander: f64,
    branch: f64,
    thick: f64,
    rod: f64,
    notch: f64,
    rough: f64,
    offset: f64,
    edge: f64,
    tex: f64,
    #[serde(rename = "texKinds")]
    texture: StrandTexture,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for StrandParams {
    fn default() -> Self {
        Self {
            count: 8,
            len: 22,
            wander: 0.6,
            branch: 0.22,
            thick: 1.0,
            rod: 0.5,
            notch: 0.18,
            rough: 0.55,
            offset: 0.5,
            edge: 0.35,
            tex: 0.4,
            texture: StrandTexture::Stipple,
            grain: 0.42,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl StrandParams {
    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    #[expect(
        clippy::len_without_is_empty,
        reason = "len is the rods-per-chain slider"
    )]
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn set_len(&mut self, len: u32) -> Result<(), Error> {
        LEN.check(f64::from(len))?;
        self.len = len;
        Ok(())
    }

    pub fn wander(&self) -> f64 {
        self.wander
    }

    pub fn set_wander(&mut self, wander: f64) -> Result<(), Error> {
        self.wander = WANDER.check(wander)?;
        Ok(())
    }

    pub fn branch(&self) -> f64 {
        self.branch
    }

    pub fn set_branch(&mut self, branch: f64) -> Result<(), Error> {
        self.branch = BRANCH.check(branch)?;
        Ok(())
    }

    pub fn thick(&self) -> f64 {
        self.thick
    }

    pub fn set_thick(&mut self, thick: f64) -> Result<(), Error> {
        self.thick = THICK.check(thick)?;
        Ok(())
    }

    pub fn rod(&self) -> f64 {
        self.rod
    }

    pub fn set_rod(&mut self, rod: f64) -> Result<(), Error> {
        self.rod = ROD.check(rod)?;
        Ok(())
    }

    pub fn notch(&self) -> f64 {
        self.notch
    }

    pub fn set_notch(&mut self, notch: f64) -> Result<(), Error> {
        self.notch = NOTCH.check(notch)?;
        Ok(())
    }

    pub fn rough(&self) -> f64 {
        self.rough
    }

    pub fn set_rough(&mut self, rough: f64) -> Result<(), Error> {
        self.rough = ROUGH.check(rough)?;
        Ok(())
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: f64) -> Result<(), Error> {
        self.offset = OFFSET.check(offset)?;
        Ok(())
    }

    pub fn edge(&self) -> f64 {
        self.edge
    }

    pub fn set_edge(&mut self, edge: f64) -> Result<(), Error> {
        self.edge = EDGE.check(edge)?;
        Ok(())
    }

    pub fn tex(&self) -> f64 {
        self.tex
    }

    pub fn set_tex(&mut self, tex: f64) -> Result<(), Error> {
        self.tex = TEX.check(tex)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn texture(&self) -> StrandTexture {
        self.texture
    }

    pub fn set_texture(&mut self, texture: StrandTexture) {
        self.texture = texture;
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
    count: u32,
    len: u32,
    wander: f64,
    branch: f64,
    thick: f64,
    rod: f64,
    notch: f64,
    rough: f64,
    offset: f64,
    edge: f64,
    tex: f64,
    #[serde(rename = "texKinds")]
    texture: StrandTexture,
    grain: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<StrandParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = StrandParams::default();
    params.set_count(raw.count)?;
    params.set_len(raw.len)?;
    params.set_wander(raw.wander)?;
    params.set_branch(raw.branch)?;
    params.set_thick(raw.thick)?;
    params.set_rod(raw.rod)?;
    params.set_notch(raw.notch)?;
    params.set_rough(raw.rough)?;
    params.set_offset(raw.offset)?;
    params.set_edge(raw.edge)?;
    params.set_tex(raw.tex)?;
    params.set_texture(raw.texture);
    params.set_grain(raw.grain)?;
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
        .chain(std::iter::once(Parameter::choice(
            "texKinds",
            &["Stipple", "Drag", "Screen"],
        )))
        .chain(std::iter::once(GRAIN.parameter()))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> StrandParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    StrandParams {
        count: pick(&COUNT) as u32,
        len: pick(&LEN) as u32,
        wander: pick(&WANDER),
        branch: pick(&BRANCH),
        thick: pick(&THICK),
        rod: pick(&ROD),
        notch: pick(&NOTCH),
        rough: pick(&ROUGH),
        offset: pick(&OFFSET),
        edge: pick(&EDGE),
        tex: pick(&TEX),
        texture: TEXTURES[(draw("texKinds") % TEXTURES.len() as u64) as usize],
        ..StrandParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &StrandParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

struct Rod {
    x: f64,
    y: f64,
    a: f64,
    sd: u32,
}

struct Chain {
    x: f64,
    y: f64,
    a: f64,
    left: u32,
    depth: u32,
}

fn rods(p: &StrandParams, (fw, fh): (f64, f64), tool_seed: u32) -> Vec<Rod> {
    let mut rnd = XorShift::new(tool_seed.wrapping_mul(2_654_435_761));
    let wid = 0.02 + p.thick * 0.09;
    let seg = wid * (0.4 + p.rod * 2.2);
    let step = seg + wid + p.notch * wid * 0.95;
    let mut stack: Vec<Chain> = (0..p.count.max(1))
        .map(|_| Chain {
            x: (-0.12 + rnd.next() * 1.24) * fw,
            y: (-0.12 + rnd.next() * 1.24) * fh,
            a: rnd.next() * TAU,
            left: p.len.max(1),
            depth: 0,
        })
        .collect();
    let mut rods = Vec::new();
    while rods.len() < ROD_CAP {
        let Some(Chain {
            mut x,
            mut y,
            mut a,
            mut left,
            depth,
        }) = stack.pop()
        else {
            break;
        };
        while left > 0 && rods.len() < ROD_CAP {
            left -= 1;
            rods.push(Rod {
                x,
                y,
                a,
                sd: (rnd.next() * 9973.0) as u32,
            });
            x += a.cos() * step;
            y += a.sin() * step;
            a += (rnd.next() - 0.5) * p.wander * 0.9;
            if depth < 3 && left > 4 && rnd.next() < p.branch * 0.22 {
                let side = if rnd.next() < 0.5 { -1.0 } else { 1.0 };
                let turn = a + side * (0.5 + rnd.next() * 0.6);
                let share = f64::from(left) * (0.4 + rnd.next() * 0.5);
                stack.push(Chain {
                    x,
                    y,
                    a: turn,
                    left: (share as u32).max(3),
                    depth: depth + 1,
                });
            }
            if x < -0.32 * fw || x > 1.32 * fw || y < -0.32 * fh || y > 1.32 * fh {
                let mut da = (fh * 0.5 - y).atan2(fw * 0.5 - x) - a;
                while da > PI {
                    da -= TAU;
                }
                while da < -PI {
                    da += TAU;
                }
                a += da * 0.2;
            }
        }
    }
    rods
}

fn rod_outline(
    path: &mut Path2D,
    rod: &Rod,
    (x, y): (f64, f64),
    (seg, wid): (f64, f64),
    rough: f64,
    grow: f64,
) {
    let sd = rod.sd;
    let s = sd as i32;
    let sdf = f64::from(sd);
    let a = rod.a + (hash(s, 17, 3) - 0.5) * 0.42 * rough;
    let (ca, sa) = (a.cos(), a.sin());
    let (px, py) = (-sa, ca);
    let full = seg + wid;
    let half = (wid * 0.05).max(wid * 0.5 + grow);
    let e0 = 0.06 + hash(s, 3, 7) * 0.34 * rough;
    let e1 = 0.06 + hash(s, 9, 7) * 0.6 * rough;
    let (mut left, mut right) = (Vec::new(), Vec::new());
    for i in 0..=STATIONS {
        let u = i as f64 / STATIONS as f64;
        let cap = (u / e0).min((1.0 - u) / e1).clamp(0.0, 1.0).powf(0.45);
        let wl = 1.0 + (value_noise(u * 6.1 + sdf * 0.11, sdf * 0.07, sd) - 0.5) * rough * 1.3;
        let wr =
            1.0 + (value_noise(u * 5.3 + sdf * 0.17, sdf * 0.05, sd + 911) - 0.5) * rough * 1.3;
        let off = (value_noise(u * 2.2 + sdf * 0.09, 7.3, sd + 41) - 0.5) * wid * 0.55 * rough;
        let t = u * full - wid * 0.5;
        let (bx, by) = (x + ca * t + px * off, y + sa * t + py * off);
        let (hl, hr) = (half * cap * wl, half * cap * wr);
        left.push((bx + px * hl, by + py * hl));
        right.push((bx - px * hr, by - py * hr));
    }
    path.move_to(left[0].0, left[0].1);
    for &(lx, ly) in &left[1..] {
        path.line_to(lx, ly);
    }
    for &(rx, ry) in right.iter().rev() {
        path.line_to(rx, ry);
    }
    path.close();
}

fn paint(surface: &mut Surface, p: &StrandParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let aspect = hf / wf;
    let rods = rods(p, (1.0 / aspect.sqrt(), aspect.sqrt()), tool_seed);
    let area = (wf * hf).sqrt();
    let wid = 0.02 + p.thick * 0.09;
    let seg = wid * (0.4 + p.rod * 2.2);
    let wide = wid * area;
    let slide = p.offset * 0.055 * area;
    let (ox, oy) = ((PI * 0.75).cos() * slide, (PI * 0.75).sin() * slide);
    let fat = wide * (1.0 + p.edge * 0.55);

    let (mw, mh) = (
        (wf / 2.0).round().max(2.0) as u32,
        (hf / 2.0).round().max(2.0) as u32,
    );
    let mu = (f64::from(mw) * f64::from(mh)).sqrt();
    let layer = |shift: (f64, f64), grow: f64| {
        let mut path = Path2D::default();
        for rod in &rods {
            let at = (rod.x * mu + shift.0 * 0.5, rod.y * mu + shift.1 * 0.5);
            rod_outline(&mut path, rod, at, (seg * mu, wide * 0.5), p.rough, grow);
        }
        path.into_path()
    };
    let mut mask = Surface::new(mw, mh);
    mask.fill([0, 0, 0]);
    mask.fill_path(&layer((ox, oy), (fat - wide) * 0.25), [255, 0, 0]);
    mask.fill_path(&layer((0.0, 0.0), 0.0), [0, 255, 0]);
    let textured = p.tex > 0.004;
    if textured {
        let eat = wide * 0.5 * 0.5 * 0.38;
        let core = layer((0.0, 0.0), -eat);
        let sigma = (eat * 0.8 * 100.0).round() / 100.0;
        mask.with_plus(|mask| mask.with_blur(sigma, |mask| mask.fill_path(&core, [0, 0, 255])));
    }
    let mask = mask.into_image();
    let src = mask.rgba();
    let (mw, mh) = (mw as usize, mh as usize);

    let inks = palette.inks();
    let (ground, plate, fill) = (inks[0], inks[1], inks[2.min(inks.len() - 1)]);
    let seed = tool_seed;
    let spread = 0.8 * p.grain + 0.06;
    let speck = p.grain * 46.0;
    let cell = (area * 0.0026).max(1.4);
    let (tc, along, across, pitch) = (cell * 1.7, cell * 13.0, cell * 1.5, cell * 4.4);
    let (tca, tsa) = (TURN.cos(), TURN.sin());

    surface.edit_rgba(|rgba, w, h| {
        for y in 0..h as usize {
            let gy = y as f64 * 0.5;
            let y0 = (gy as usize).min(mh - 1);
            let fy = gy - y0 as f64;
            let y1 = (y0 + 1).min(mh - 1);
            for x in 0..w as usize {
                let gx = x as f64 * 0.5;
                let x0 = (gx as usize).min(mw - 1);
                let fx = gx - x0 as f64;
                let x1 = if x0 + 1 > mw - 1 { 0 } else { x0 + 1 };
                let sample = |c: usize| {
                    let at = |sx: usize, sy: usize| f64::from(src[(sy * mw + sx) * 4 + c]);
                    let a = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
                    let b = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
                    (a + (b - a) * fy) / 255.0
                };
                let (xf, yf) = (x as f64, y as f64);
                let n = (value_noise(xf / cell, yf / cell, seed.wrapping_add(29)) - 0.5) * spread;
                let mut ink = sample(1) > 0.5 + n;
                if ink && textured {
                    let miss = p.tex * (0.12 + 0.88 * (1.0 - sample(2)));
                    let (u, v) = (xf * tca + yf * tsa, yf * tca - xf * tsa);
                    let gap = match p.texture {
                        StrandTexture::Screen => {
                            let (u, v) = (u / pitch, v / pitch);
                            let (fu, fv) = (u - u.floor() - 0.5, v - v.floor() - 0.5);
                            let rad = 0.72 * (1.0 - miss).max(0.0).sqrt();
                            fu * fu + fv * fv > rad * rad
                        }
                        StrandTexture::Drag => {
                            let (u, v) = ((u / along).floor() as i32, (v / across).floor() as i32);
                            hash(u, v, seed.wrapping_add(17)) < miss
                        }
                        StrandTexture::Stipple => {
                            let (u, v) = ((xf / tc).floor() as i32, (yf / tc).floor() as i32);
                            hash(u, v, seed.wrapping_add(13)) < miss
                        }
                    };
                    ink = !gap;
                }
                let col = if ink {
                    fill
                } else if sample(0) > 0.5 + n {
                    plate
                } else {
                    ground
                };
                let q = if speck > 0.002 {
                    (hash(x as i32, y as i32, seed.wrapping_add(71)) - 0.5) * speck
                } else {
                    0.0
                };
                let i = (y * w as usize + x) * 4;
                for c in 0..3 {
                    rgba[i + c] = store(f64::from(col[c]) + q);
                }
                rgba[i + 3] = 255;
            }
        }
    });
}
