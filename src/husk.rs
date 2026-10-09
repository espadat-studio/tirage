use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, hash, round_half_up, store, value_noise,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "husk";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 3] = ["#e0c3fc", "#1b1b1e", "#f9f871"];

const COUNT: Param = Param {
    taste: (18, 70),
    ..Param::new(SLUG, "count", 1, 70, 1)
};
const SIZE: Param = Param {
    taste: (10, 100),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const VARY: Param = Param::new(SLUG, "vary", 0, 100, 100);
const LUMP: Param = Param::new(SLUG, "lump", 0, 100, 100);
const EAT: Param = Param {
    taste: (20, 100),
    ..Param::new(SLUG, "eat", 0, 100, 100)
};
const TEX: Param = Param::new(SLUG, "tex", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bite {
    Crumble,
    Dots,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HuskParams {
    count: u32,
    size: f64,
    vary: f64,
    lump: f64,
    #[serde(rename = "bites")]
    bite: Bite,
    eat: f64,
    tex: f64,
    grain: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for HuskParams {
    fn default() -> Self {
        Self {
            count: 30,
            size: 0.62,
            vary: 0.5,
            lump: 0.42,
            bite: Bite::Crumble,
            eat: 0.55,
            tex: 0.45,
            grain: 0.3,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl HuskParams {
    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn vary(&self) -> f64 {
        self.vary
    }

    pub fn set_vary(&mut self, vary: f64) -> Result<(), Error> {
        self.vary = VARY.check(vary)?;
        Ok(())
    }

    pub fn lump(&self) -> f64 {
        self.lump
    }

    pub fn set_lump(&mut self, lump: f64) -> Result<(), Error> {
        self.lump = LUMP.check(lump)?;
        Ok(())
    }

    pub fn bite(&self) -> Bite {
        self.bite
    }

    pub fn set_bite(&mut self, bite: Bite) {
        self.bite = bite;
    }

    pub fn eat(&self) -> f64 {
        self.eat
    }

    pub fn set_eat(&mut self, eat: f64) -> Result<(), Error> {
        self.eat = EAT.check(eat)?;
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
    size: f64,
    vary: f64,
    lump: f64,
    bites: Bite,
    eat: f64,
    tex: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<HuskParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = HuskParams::default();
    params.set_count(raw.count)?;
    params.set_size(raw.size)?;
    params.set_vary(raw.vary)?;
    params.set_lump(raw.lump)?;
    params.set_bite(raw.bites);
    params.set_eat(raw.eat)?;
    params.set_tex(raw.tex)?;
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

pub(crate) const PARAMS: &[Param] = &[COUNT, SIZE, VARY, LUMP, EAT, TEX];

pub(crate) fn parameters() -> Vec<Parameter> {
    [
        COUNT.parameter(),
        SIZE.parameter(),
        VARY.parameter(),
        LUMP.parameter(),
        Parameter::choice("bites", &["Crumble", "Dots"]),
        EAT.parameter(),
        TEX.parameter(),
        GRAIN.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> HuskParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    HuskParams {
        count: pick(&COUNT) as u32,
        size: pick(&SIZE),
        vary: pick(&VARY),
        lump: pick(&LUMP),
        eat: pick(&EAT),
        tex: pick(&TEX),
        ..HuskParams::default()
    }
}

const GROUND: usize = 0;
const SILHOUETTE: usize = 1;
const FILL: usize = 2;

struct Field {
    depth: Vec<f32>,
    width: usize,
    height: usize,
}

struct Xorshift(u32);

impl Xorshift {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        f64::from(self.0) / 4_294_967_296.0
    }
}

fn field(params: &HuskParams, tool_seed: u32, aspect: f64) -> Field {
    let width = if aspect > 1.0 {
        round_half_up(560.0 / aspect).max(8.0)
    } else {
        560.0
    };
    let height = round_half_up(width * aspect).max(8.0);
    let (width, height) = (width as usize, height as usize);
    let (fw, fh) = (width as f64, height as f64);
    let mut depth = vec![0.0_f32; width * height];
    let mut rnd = Xorshift(match tool_seed.wrapping_mul(2_654_435_761) {
        0 => 1,
        state => state,
    });
    let base = (0.045 + params.size * 0.115) * (fw * fh).sqrt();
    let lump = params.lump * 0.55;

    for _ in 0..params.count.max(1) {
        let cx = (-0.12 + rnd.next() * 1.24) * fw;
        let cy = (-0.12 + rnd.next() * 1.24) * fh;
        let radius = base * (1.0 + (rnd.next() - 0.5) * 2.0 * params.vary * 0.7);
        let ex = 0.78 + rnd.next() * 0.5;
        let ey = 0.78 + rnd.next() * 0.5;
        let rot = rnd.next() * PI * 2.0;
        let _wobble = rnd.next() * PI * 2.0;
        let phases = [rnd.next(), rnd.next(), rnd.next()].map(|r| r * PI * 2.0);
        let harmonics = [
            2.0 + (rnd.next() * 2.0).trunc(),
            4.0 + (rnd.next() * 3.0).trunc(),
            7.0 + (rnd.next() * 3.0).trunc(),
        ];
        let reach = radius * (1.0 + lump) * ex.max(ey) + 2.0;
        let x0 = (cx - reach).floor().max(0.0);
        let x1 = (cx + reach).ceil().min(fw - 1.0);
        let y0 = (cy - reach).floor().max(0.0);
        let y1 = (cy + reach).ceil().min(fh - 1.0);
        if x1 < x0 || y1 < y0 {
            continue;
        }
        let (ca, sa) = (rot.cos(), rot.sin());
        for j in y0 as usize..=y1 as usize {
            let dy = j as f64 - cy;
            for i in x0 as usize..=x1 as usize {
                let dx = i as f64 - cx;
                let ux = (dx * ca + dy * sa) / ex;
                let uy = (-dx * sa + dy * ca) / ey;
                let dist = (ux * ux + uy * uy).sqrt();
                if dist > radius * (1.0 + lump) {
                    continue;
                }
                let ang = uy.atan2(ux);
                let wobble = (harmonics[0] * ang + phases[0]).sin() * 0.5
                    + (harmonics[1] * ang + phases[1]).sin() * 0.33
                    + (harmonics[2] * ang + phases[2]).sin() * 0.2;
                let edge = radius * (1.0 + lump * wobble);
                if edge <= 0.001 || dist >= edge {
                    continue;
                }
                let d = 1.0 - dist / edge;
                let cell = &mut depth[j * width + i];
                if d > f64::from(*cell) {
                    *cell = d as f32;
                }
            }
        }
    }
    Field {
        depth,
        width,
        height,
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &HuskParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

fn paint(surface: &mut Surface, params: &HuskParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (surface.width(), surface.height());
    let field = field(params, tool_seed, f64::from(height) / f64::from(width));
    let ground = palette.ink(GROUND);
    let silhouette = palette.ink(SILHOUETTE);
    let fill = palette.ink(if palette.len() > FILL {
        FILL
    } else {
        SILHOUETTE
    });
    let kx = field.width as f64 / f64::from(width);
    let ky = field.height as f64 / f64::from(height);
    let pitch = ((0.004 + params.tex * 0.03) * f64::from(width.min(height))).max(3.0);
    let noise_scale = 1.0 / (2.0 + params.tex * 18.0);
    let eat = params.eat;
    let grain = params.grain * 40.0;
    let lerp_cells = |at: f64, last: usize| {
        let low = (at as usize).min(last);
        (low, (low + 1).min(last), at - low as f64)
    };
    let stored = |i: usize| f64::from(field.depth[i]);

    surface.edit_rgba(|rgba, _, _| {
        for (y, row) in rgba.chunks_exact_mut(width as usize * 4).enumerate() {
            let (y0, y1, fy) = lerp_cells(y as f64 * ky, field.height - 1);
            let (r0, r1) = (y0 * field.width, y1 * field.width);
            for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let (x0, x1, fx) = lerp_cells(x as f64 * kx, field.width - 1);
                let a = stored(r0 + x0) + (stored(r0 + x1) - stored(r0 + x0)) * fx;
                let b = stored(r1 + x0) + (stored(r1 + x1) - stored(r1 + x0)) * fx;
                let depth = a + (b - a) * fy;
                let (xf, yf) = (x as f64, y as f64);
                let ink = if depth <= 0.001 {
                    ground
                } else {
                    let keep = match params.bite {
                        Bite::Dots => {
                            let dx = xf / pitch - ((xf / pitch).trunc() + 0.5);
                            let dy = yf / pitch - ((yf / pitch).trunc() + 0.5);
                            let rad = (dx * dx + dy * dy).sqrt();
                            let v = ((depth - 0.05) / 0.95).clamp(0.0, 1.0);
                            rad < 0.52 * v.sqrt() * (1.35 - eat * 0.85)
                        }
                        Bite::Crumble => {
                            let n = fbm(
                                xf * noise_scale,
                                yf * noise_scale,
                                tool_seed.wrapping_add(13),
                            );
                            let n = n * n * (3.0 - 2.0 * n);
                            depth > 0.02 + eat * (0.06 + 1.5 * n)
                        }
                    };
                    if keep { fill } else { silhouette }
                };
                let g = if grain > 0.002 {
                    (hash(x as i32, y as i32, tool_seed.wrapping_add(71)) - 0.5) * grain
                } else {
                    0.0
                };
                for (out, channel) in px.iter_mut().zip(ink) {
                    *out = store(f64::from(channel) + g);
                }
                px[3] = 255;
            }
        }
    });
}

fn fbm(x: f64, y: f64, seed: u32) -> f64 {
    let (mut sum, mut weight, mut frequency, mut total) = (0.0, 0.5, 1.0, 0.0);
    for octave in 0..3 {
        sum += weight
            * value_noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * 131),
            );
        total += weight;
        frequency *= 2.07;
        weight *= 0.55;
    }
    sum / total
}
