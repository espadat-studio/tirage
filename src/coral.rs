use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store, value_noise,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "coral";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

#[expect(
    clippy::approx_constant,
    reason = "the site's rib phase is 6.28, not 2π"
)]
const RIB_PHASE: f64 = 6.28;

const DEFAULT_PALETTE: [&str; 5] = ["#33081c", "#e8449e", "#f7b8dc", "#45be6e", "#c9f24e"];

const BRANCHES: Param = Param::new(SLUG, "branches", 2, 12, 1);
const SPACING: Param = Param::new(SLUG, "spacing", 0, 100, 100);
const SPREAD: Param = Param::new(SLUG, "spread", 0, 100, 100);
const WIDTH: Param = Param::new(SLUG, "width", 10, 100, 100);
const WOBBLE: Param = Param::new(SLUG, "wobble", 0, 100, 100);
const COVER: Param = Param::new(SLUG, "cover", 0, 100, 100);
const SIZE: Param = Param::new(SLUG, "size", 0, 100, 100);
const BENEATH: Param = Param::new(SLUG, "beneath", 0, 100, 100);
const RIBS: Param = Param::new(SLUG, "ribs", 0, 100, 100);
const DOTS: Param = Param::new(SLUG, "dots", 0, 100, 100);
const SHADOW: Param = Param::new(SLUG, "shadow", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[
    BRANCHES, SPACING, SPREAD, WIDTH, WOBBLE, COVER, SIZE, BENEATH, RIBS, DOTS, SHADOW,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoralParams {
    branches: u32,
    spacing: f64,
    spread: f64,
    width: f64,
    wobble: f64,
    cover: f64,
    size: f64,
    beneath: f64,
    ribs: f64,
    dots: f64,
    shadow: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for CoralParams {
    fn default() -> Self {
        Self {
            branches: 6,
            spacing: 0.5,
            spread: 0.7,
            width: 0.6,
            wobble: 0.4,
            cover: 0.5,
            size: 0.5,
            beneath: 0.35,
            ribs: 0.8,
            dots: 0.7,
            shadow: 0.6,
            dither: Dither::default(),
            chassis_grain: Grain::textile(),
        }
    }
}

impl CoralParams {
    pub fn branches(&self) -> u32 {
        self.branches
    }

    pub fn set_branches(&mut self, branches: u32) -> Result<(), Error> {
        BRANCHES.check(f64::from(branches))?;
        self.branches = branches;
        Ok(())
    }

    pub fn spacing(&self) -> f64 {
        self.spacing
    }

    pub fn set_spacing(&mut self, spacing: f64) -> Result<(), Error> {
        self.spacing = SPACING.check(spacing)?;
        Ok(())
    }

    pub fn spread(&self) -> f64 {
        self.spread
    }

    pub fn set_spread(&mut self, spread: f64) -> Result<(), Error> {
        self.spread = SPREAD.check(spread)?;
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) -> Result<(), Error> {
        self.width = WIDTH.check(width)?;
        Ok(())
    }

    pub fn wobble(&self) -> f64 {
        self.wobble
    }

    pub fn set_wobble(&mut self, wobble: f64) -> Result<(), Error> {
        self.wobble = WOBBLE.check(wobble)?;
        Ok(())
    }

    pub fn cover(&self) -> f64 {
        self.cover
    }

    pub fn set_cover(&mut self, cover: f64) -> Result<(), Error> {
        self.cover = COVER.check(cover)?;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn beneath(&self) -> f64 {
        self.beneath
    }

    pub fn set_beneath(&mut self, beneath: f64) -> Result<(), Error> {
        self.beneath = BENEATH.check(beneath)?;
        Ok(())
    }

    pub fn ribs(&self) -> f64 {
        self.ribs
    }

    pub fn set_ribs(&mut self, ribs: f64) -> Result<(), Error> {
        self.ribs = RIBS.check(ribs)?;
        Ok(())
    }

    pub fn dots(&self) -> f64 {
        self.dots
    }

    pub fn set_dots(&mut self, dots: f64) -> Result<(), Error> {
        self.dots = DOTS.check(dots)?;
        Ok(())
    }

    pub fn shadow(&self) -> f64 {
        self.shadow
    }

    pub fn set_shadow(&mut self, shadow: f64) -> Result<(), Error> {
        self.shadow = SHADOW.check(shadow)?;
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
    branches: u32,
    spacing: f64,
    spread: f64,
    width: f64,
    wobble: f64,
    cover: f64,
    size: f64,
    beneath: f64,
    ribs: f64,
    dots: f64,
    shadow: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<CoralParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = CoralParams::default();
    params.set_branches(raw.branches)?;
    params.set_spacing(raw.spacing)?;
    params.set_spread(raw.spread)?;
    params.set_width(raw.width)?;
    params.set_wobble(raw.wobble)?;
    params.set_cover(raw.cover)?;
    params.set_size(raw.size)?;
    params.set_beneath(raw.beneath)?;
    params.set_ribs(raw.ribs)?;
    params.set_dots(raw.dots)?;
    params.set_shadow(raw.shadow)?;
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

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> CoralParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    CoralParams {
        branches: pick(&BRANCHES) as u32,
        spacing: pick(&SPACING),
        spread: pick(&SPREAD),
        width: pick(&WIDTH),
        wobble: pick(&WOBBLE),
        cover: pick(&COVER),
        size: pick(&SIZE),
        beneath: pick(&BENEATH),
        ribs: pick(&RIBS),
        dots: pick(&DOTS),
        shadow: pick(&SHADOW),
        dither: Dither::default(),
        chassis_grain: Grain::textile(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &CoralParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

type Rgb = [f64; 3];

fn lift(c: Rgb, k: f64) -> Rgb {
    c.map(|v| v + (255.0 - v) * k)
}

fn toward(c: &mut Rgb, target: Rgb, k: f64) {
    for (v, t) in c.iter_mut().zip(target) {
        *v += (t - *v) * k;
    }
}

struct Grid {
    values: Vec<f32>,
    width: usize,
    cell: f64,
}

impl Grid {
    fn new(cols: usize, rows: usize, cell: f64, value: impl Fn(f64, f64) -> f64) -> Self {
        let values = (0..rows)
            .flat_map(|j| (0..cols).map(move |i| (i as f64 * cell, j as f64 * cell)))
            .map(|(x, y)| value(x, y) as f32)
            .collect();
        Self {
            values,
            width: cols,
            cell,
        }
    }

    fn at(&self, x: f64, y: f64) -> f64 {
        let (gx, gy) = (x / self.cell, y / self.cell);
        let (xi, yi) = (gx as usize, gy as usize);
        let (xf, yf) = (gx - xi as f64, gy - yi as f64);
        let k = yi * self.width + xi;
        let v = |k: usize| f64::from(self.values[k]);
        let (a, b, c, d) = (v(k), v(k + 1), v(k + self.width), v(k + self.width + 1));
        (a * (1.0 - xf) + b * xf) * (1.0 - yf) + (c * (1.0 - xf) + d * xf) * yf
    }
}

fn paint(surface: &mut Surface, params: &CoralParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let u_size = (wf * hf).sqrt();
    let seed = tool_seed as i32;
    let sd = |i: i32| seed.wrapping_mul(7).wrapping_add(i) as u32;

    let inks: Vec<Rgb> = palette.inks().iter().map(|c| c.map(f64::from)).collect();
    let n = inks.len();
    let ground = inks[0];
    let branch = inks[1.min(n - 1)];
    let rib = if n > 2 { inks[2] } else { lift(branch, 0.55) };
    let lobe = if n > 3 { inks[3] } else { inks[1.min(n - 1)] };
    let dot = if n > 4 { inks[4] } else { lift(lobe, 0.5) };
    let dark = ground.map(|c| c * 0.55);

    let mut rng = XorShift::new(seed.wrapping_mul(-2_048_144_777) as u32 ^ 0x27d4_eb2f);
    let side = rng.next();
    let (rx, ry) = if side < 0.6 {
        let rx = wf * (0.2 + rng.next() * 0.6);
        (rx, hf + u_size * (0.1 + rng.next() * 0.25))
    } else if side < 0.8 {
        let rx = -u_size * (0.1 + rng.next() * 0.25);
        (rx, hf * (0.3 + rng.next() * 0.5))
    } else {
        let rx = wf + u_size * (0.1 + rng.next() * 0.25);
        (rx, hf * (0.3 + rng.next() * 0.5))
    };
    let th0 = (hf * 0.5 - ry).atan2(wf * 0.5 - rx);
    let range = (0.35 + params.spread * 0.65) * PI;
    let th_start = th0 - range * 0.5;
    let s = (0.032 + params.spacing * 0.075) * u_size;
    let n0 = f64::from(params.branches.max(2));
    let hw_max = params.width * s * 0.5 * 0.62;
    let wob = params.wobble;
    let rib_p = (s * 0.11).max(1.5);
    let sh_k = params.shadow;
    let sh_off = u_size * 0.012 * sh_k;
    let gap = s * 0.1 * sh_k;

    let lsz = u_size * (0.08 + params.size * 0.16);
    let thr = 0.62 - params.cover * 0.16;
    let cell = (u_size * 0.0095).max(2.0);

    let (cs0, sn0) = (th0.cos(), th0.sin());
    let lobes = Grid::new((w >> 1) as usize + 2, (h >> 1) as usize + 2, 2.0, |x, y| {
        let xr = (x * cs0 + y * sn0) / (lsz * 1.9);
        let yr = (-x * sn0 + y * cs0) / lsz;
        0.58 * value_noise(xr, yr, sd(2))
            + 0.3 * value_noise(xr * 2.3 + 3.1, yr * 2.3 + 1.7, sd(3))
            + 0.12 * value_noise(xr * 6.1 + 9.0, yr * 6.1 + 4.0, sd(3))
    });
    let slow = |k: f64, ox: f64, oy: f64, s: u32| {
        Grid::new(
            w.div_ceil(8) as usize + 1,
            h.div_ceil(8) as usize + 1,
            8.0,
            move |x, y| value_noise(x / k + ox, y / k + oy, s),
        )
    };
    let under_field = slow(lsz * 3.0, 11.0, 5.0, sd(4));
    let dot_field = slow(u_size * 0.16, 23.0, 9.0, sd(5));
    let off_i = round_half_up(sh_off);
    let off_j = round_half_up(sh_off * 0.7);
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);

    let mut rgba = Vec::with_capacity((w * h) as usize * 4);
    for y in 0..h {
        let y = f64::from(y);
        for x in 0..w {
            let x = f64::from(x);
            let (dx, dy) = (x - rx, y - ry);
            let rr = (dx * dx + dy * dy).sqrt();
            let mut th = dy.atan2(dx);
            if wob > 0.0 {
                let far = (rr / (u_size * 0.6)).min(1.0);
                th += wob
                    * 0.55
                    * far
                    * (value_noise(rr / (u_size * 0.38) + 7.0, th * 1.4, sd(1)) - 0.5);
                th += wob
                    * 0.22
                    * far
                    * (value_noise(rr / (u_size * 0.16) + 3.0, th * 4.2 + 5.0, sd(1)) - 0.5);
            }
            let a = ((th - th_start) % TAU + TAU) % TAU;
            let (mut in_b, mut arc, mut hw, mut side1, mut bid) = (false, 1e9, 1.0, 0, 0);
            if a <= range && rr > 1.0 {
                let l = (rr * range / (n0 * s)).log2();
                let (nf, ease, fine, n) = if l < 0.0 {
                    (n0, 1.0, false, 0)
                } else {
                    let n = l.floor();
                    (n0 * 2f64.powf(n + 1.0), smooth(l - n), true, n as i32)
                };
                let u = a / range * nf;
                let i0 = u.floor() as i64;
                let sp = rr * range / nf;
                let mut best = 1e9;
                for k in i0 - 1..=i0 + 2 {
                    if k < 0 || k as f64 > nf {
                        continue;
                    }
                    let (mut centre, mut sc) = (k as f64, 1.0);
                    if fine && k & 1 == 1 {
                        let parent = if (k >> 1) & 1 == 1 { k - 1 } else { k + 1 };
                        centre = parent as f64 + (k - parent) as f64 * ease;
                        sc = 0.22 + 0.78 * ease;
                    }
                    let id = k as f64 / nf;
                    let d = (u - centre).abs() * sp;
                    if d >= best * hw_max * 1.2 {
                        continue;
                    }
                    if fine {
                        let (mut q, mut lvl) = (k, n + 1);
                        while lvl > 0 && q & 1 == 0 {
                            q >>= 1;
                            lvl -= 1;
                        }
                        if lvl > 0 {
                            let hq = hash(round_half_up(id * 8192.0) as i32, 9, seed as u32);
                            if hq < 0.28 {
                                let rs = n0 * s / range * 2f64.powi(lvl - 1);
                                let rend = rs * (1.5 + hq * 6.0);
                                let e = (rend - rr) / (hw_max * sc).max(1.0);
                                if e < 0.0 {
                                    continue;
                                }
                                if e < 1.0 {
                                    sc *= (1.0 - (1.0 - e) * (1.0 - e)).max(0.0).sqrt();
                                }
                            }
                        }
                    }
                    let tube = hw_max * sc * (1.0 + 0.2 * (rr / (s * 0.85) + id * 137.5).sin());
                    let ratio = d / tube.max(0.5);
                    if ratio < best {
                        best = ratio;
                        arc = d;
                        hw = tube;
                        side1 = if u - centre >= 0.0 { 1 } else { -1 };
                        bid = round_half_up(id * 8192.0) as i32;
                    }
                }
                in_b = best < 1.0;
            }

            let fv = lobes.at(x, y);
            let in_l = fv > thr;
            let under = under_field.at(x, y) < params.beneath;
            let (sx, sy) = (x - off_i, y - off_j);
            let fs = if sx >= 0.0 && sy >= 0.0 {
                lobes.at(sx, sy)
            } else {
                0.0
            };
            let lobe_sh = sh_k > 0.0 && !in_l && fs > thr;
            let lobe_sh_under = lobe_sh && under_field.at(sx, sy) < params.beneath;

            let mut c = ground;
            let mut on_lobe = false;
            if lobe_sh_under {
                toward(&mut c, dark, 0.55 * sh_k);
            }
            if in_l && under {
                c = lobe;
                on_lobe = true;
            }
            if in_b {
                c = branch;
                on_lobe = false;
                if params.ribs > 0.0 {
                    let rv = 0.5
                        + 0.5 * (rr / rib_p * TAU + hash(bid, 3, seed as u32) * RIB_PHASE).sin();
                    if rv > 0.7 && arc < hw * 0.86 {
                        toward(&mut c, rib, params.ribs * ((rv - 0.7) * 8.0).min(1.0));
                    }
                }
                if sh_k > 0.0 && side1 > 0 && arc > hw * 0.58 {
                    let k = sh_k * 0.55 * ((arc - hw * 0.58) / (hw * 0.42)).min(1.0);
                    toward(&mut c, dark, k);
                }
            } else if sh_k > 0.0 && side1 > 0 && arc < hw + gap {
                toward(&mut c, dark, 0.5 * sh_k);
            }
            if lobe_sh && !lobe_sh_under {
                toward(&mut c, dark, 0.6 * sh_k);
            }
            if in_l && !under {
                c = lobe;
                on_lobe = true;
            }
            if on_lobe && params.dots > 0.0 {
                let (gx, gy) = (x / cell, y / cell);
                let (ddx, ddy) = (gx - gx.floor() - 0.5, gy - gy.floor() - 0.5);
                let edge = ((thr + 0.13 - fv) / 0.13).clamp(0.0, 1.0);
                let rad = 0.5 * params.dots * (0.3 + 0.4 * edge + 0.3 * dot_field.at(x, y));
                if ddx * ddx + ddy * ddy < rad * rad {
                    c = dot;
                }
            }
            let [r, g, b] = c.map(store);
            rgba.extend([r, g, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, w, h);
}
