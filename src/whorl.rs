use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, store, value_noise};
use crate::param::Param;
use crate::surface::{Smoothing, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "whorl";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 2] = ["#003049", "#f77f00"];

const CENTRES: Param = Param::new(SLUG, "centres", 1, 14, 1);
const PULL: Param = Param::new(SLUG, "pull", 0, 100, 100);
const PUSH: Param = Param::new(SLUG, "push", 0, 100, 100);
const STRIPES: Param = Param {
    taste: (2, 30),
    ..Param::new(SLUG, "stripes", 2, 90, 1)
};
const WEIGHT: Param = Param {
    taste: (20, 80),
    ..Param::new(SLUG, "weight", 0, 100, 100)
};
const WARP: Param = Param::new(SLUG, "warp", 0, 100, 100);
const DETAIL: Param = Param::new(SLUG, "detail", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[CENTRES, PULL, PUSH, STRIPES, WEIGHT, WARP, DETAIL];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhorlWarp {
    Smooth,
    Turbulent,
    Ripple,
}

const WARPS: [WhorlWarp; 3] = [WhorlWarp::Smooth, WhorlWarp::Turbulent, WhorlWarp::Ripple];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhorlParams {
    centres: u32,
    pull: f64,
    push: f64,
    stripes: u32,
    weight: f64,
    #[serde(rename = "dirs")]
    kind: WhorlWarp,
    warp: f64,
    detail: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for WhorlParams {
    fn default() -> Self {
        Self {
            centres: 4,
            pull: 0.62,
            push: 0.35,
            stripes: 16,
            weight: 0.5,
            kind: WhorlWarp::Smooth,
            warp: 0.5,
            detail: 0.3,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl WhorlParams {
    pub fn centres(&self) -> u32 {
        self.centres
    }

    pub fn set_centres(&mut self, centres: u32) -> Result<(), Error> {
        CENTRES.check(f64::from(centres))?;
        self.centres = centres;
        Ok(())
    }

    pub fn pull(&self) -> f64 {
        self.pull
    }

    pub fn set_pull(&mut self, pull: f64) -> Result<(), Error> {
        self.pull = PULL.check(pull)?;
        Ok(())
    }

    pub fn push(&self) -> f64 {
        self.push
    }

    pub fn set_push(&mut self, push: f64) -> Result<(), Error> {
        self.push = PUSH.check(push)?;
        Ok(())
    }

    pub fn stripes(&self) -> u32 {
        self.stripes
    }

    pub fn set_stripes(&mut self, stripes: u32) -> Result<(), Error> {
        STRIPES.check(f64::from(stripes))?;
        self.stripes = stripes;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
        Ok(())
    }

    pub fn kind(&self) -> WhorlWarp {
        self.kind
    }

    pub fn set_kind(&mut self, kind: WhorlWarp) {
        self.kind = kind;
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn detail(&self) -> f64 {
        self.detail
    }

    pub fn set_detail(&mut self, detail: f64) -> Result<(), Error> {
        self.detail = DETAIL.check(detail)?;
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
    centres: u32,
    pull: f64,
    push: f64,
    stripes: u32,
    weight: f64,
    #[serde(rename = "dirs")]
    kind: WhorlWarp,
    warp: f64,
    detail: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<WhorlParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = WhorlParams::default();
    params.set_centres(raw.centres)?;
    params.set_pull(raw.pull)?;
    params.set_push(raw.push)?;
    params.set_stripes(raw.stripes)?;
    params.set_weight(raw.weight)?;
    params.set_kind(raw.kind);
    params.set_warp(raw.warp)?;
    params.set_detail(raw.detail)?;
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
    let (before, after) = PARAMS.split_at(5);
    before
        .iter()
        .map(Param::parameter)
        .chain(std::iter::once(Parameter::choice(
            "dirs",
            &["Smooth", "Turbulent", "Ripple"],
        )))
        .chain(after.iter().map(Param::parameter))
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> WhorlParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    WhorlParams {
        centres: pick(&CENTRES) as u32,
        pull: pick(&PULL),
        push: pick(&PUSH),
        stripes: pick(&STRIPES) as u32,
        weight: pick(&WEIGHT),
        kind: WARPS[(draw("dirs") % WARPS.len() as u64) as usize],
        warp: pick(&WARP),
        detail: pick(&DETAIL),
        dither: Dither::default(),
        chassis_grain: Grain::default(),
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &WhorlParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

struct Centre {
    x: f64,
    y: f64,
    weight: f64,
}

fn paint(surface: &mut Surface, params: &WhorlParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (surface.width(), surface.height());
    let (wf, hf) = (f64::from(w), f64::from(h));
    let seed = tool_seed as i32;
    let noise_seed = |k: i32| seed.wrapping_add(k) as u32;
    let ground = palette.ink(0).map(f64::from);
    let inks: Vec<[f64; 3]> = palette.inks()[1..]
        .iter()
        .map(|c| c.map(f64::from))
        .collect();

    let mut rng = XorShift::new(tool_seed.wrapping_mul(2_654_435_761));
    let centres: Vec<Centre> = (0..params.centres.max(1))
        .map(|i| {
            let negative = i > 0 && rng.next() < params.push;
            let x = -0.25 + rng.next() * 1.5;
            let y = -0.25 + rng.next() * 1.5;
            let weight = (if negative { -1.0 } else { 1.0 }) * (0.45 + rng.next() * 0.9);
            rng.next();
            Centre { x, y, weight }
        })
        .collect();

    let aspect = hf / wf;
    let wamp = params.warp * 0.42;
    let wfrq = 1.1 + params.detail * 5.5;
    let freq = f64::from(params.stripes.max(1));
    let soft = 0.36 * (1.0 - params.pull);
    let soft2 = soft * soft;
    let duty = 0.06 + params.weight * 0.88;
    let field = |x: f64, y: f64| {
        let (mut wx, mut wy) = (x, y);
        if wamp > 0.001 {
            if params.kind == WhorlWarp::Ripple {
                wx += wamp * 0.30 * (y * wfrq * 6.0 + x * 1.7).sin();
                wy += wamp * 0.30 * (x * wfrq * 5.1 + y * 1.3).sin();
            } else {
                wx += wamp * (value_noise(x * wfrq, y * wfrq, noise_seed(11)) - 0.5);
                wy += wamp * (value_noise(x * wfrq + 7.3, y * wfrq + 3.1, noise_seed(29)) - 0.5);
                if params.kind == WhorlWarp::Turbulent {
                    wx += wamp
                        * 0.45
                        * (value_noise(x * wfrq * 2.7 + 2.2, y * wfrq * 2.7, noise_seed(53)) - 0.5);
                    wy += wamp
                        * 0.45
                        * (value_noise(x * wfrq * 2.7, y * wfrq * 2.7 + 5.5, noise_seed(71)) - 0.5);
                }
            }
        }
        centres.iter().fold(0.0, |sum, c| {
            let (dx, dy) = (wx - c.x, wy - c.y);
            sum + c.weight * (dx * dx + dy * dy + soft2).sqrt()
        })
    };

    let mut up = vec![0.0_f32; w as usize];
    let mut rgba = Vec::with_capacity((w * h) as usize * 4);
    for y in 0..h {
        let fy = (f64::from(y) + 0.5) / hf * aspect;
        let mut left: Option<f64> = None;
        for x in 0..w {
            let fx = (f64::from(x) + 0.5) / wf;
            let u = field(fx, fy) * freq;
            let mut g: f64 = 0.0025;
            if let Some(left) = left {
                g = g.max((u - left).abs());
            }
            if y > 0 {
                g = g.max((u - f64::from(up[x as usize])).abs());
            }
            left = Some(u);
            up[x as usize] = u as f32;
            let v = u - u.floor();
            let sd = if v < duty {
                v.min(duty - v)
            } else {
                -(v - duty).min(1.0 - v)
            };
            let a = (0.5 + sd / g).clamp(0.0, 1.0);
            let ink = inks[(u.floor() as i64).rem_euclid(inks.len() as i64) as usize];
            let [r, gg, b] = std::array::from_fn(|i| store(ground[i] + (ink[i] - ground[i]) * a));
            rgba.extend([r, gg, b, 255]);
        }
    }
    surface.draw_smooth(&rgba, w, h, Smoothing::Bicubic);
}
