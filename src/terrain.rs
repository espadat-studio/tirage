use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "terrain";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#264653", "#2a9d8f", "#8ab17d", "#e9c46a", "#f4a261", "#e76f51",
];

const SCALE: Param = Param {
    taste: (15, 90),
    ..Param::new(SLUG, "scale", 6, 90, 10)
};
const WARP: Param = Param {
    step: 2,
    ..Param::new(SLUG, "warp", 0, 300, 100)
};
const OCT: Param = Param::new(SLUG, "oct", 1, 7, 1);
const CONTRAST: Param = Param {
    step: 5,
    taste: (100, 400),
    ..Param::new(SLUG, "contrast", 50, 400, 100)
};
const BALANCE: Param = Param {
    step: 5,
    ..Param::new(SLUG, "balance", -120, 120, 100)
};
const GRAIN: Param = Param::new(SLUG, "grain", 0, 120, 100);
const BLOCK: Param = Param::new(SLUG, "block", 0, 14, 1);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TerrainParams {
    scale: f64,
    warp: f64,
    oct: u32,
    contrast: f64,
    balance: f64,
    grain: f64,
    block: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            scale: 2.6,
            warp: 1.1,
            oct: 5,
            contrast: 1.7,
            balance: 0.0,
            grain: 0.3,
            block: 0,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl TerrainParams {
    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn warp(&self) -> f64 {
        self.warp
    }

    pub fn set_warp(&mut self, warp: f64) -> Result<(), Error> {
        self.warp = WARP.check(warp)?;
        Ok(())
    }

    pub fn oct(&self) -> u32 {
        self.oct
    }

    pub fn set_oct(&mut self, oct: u32) -> Result<(), Error> {
        OCT.check(f64::from(oct))?;
        self.oct = oct;
        Ok(())
    }

    pub fn contrast(&self) -> f64 {
        self.contrast
    }

    pub fn set_contrast(&mut self, contrast: f64) -> Result<(), Error> {
        self.contrast = CONTRAST.check(contrast)?;
        Ok(())
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn set_balance(&mut self, balance: f64) -> Result<(), Error> {
        self.balance = BALANCE.check(balance)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn block(&self) -> u32 {
        self.block
    }

    pub fn set_block(&mut self, block: u32) -> Result<(), Error> {
        BLOCK.check(f64::from(block))?;
        self.block = block;
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
    scale: f64,
    warp: f64,
    oct: u32,
    contrast: f64,
    balance: f64,
    grain: f64,
    block: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<TerrainParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = TerrainParams::default();
    params.set_scale(raw.scale)?;
    params.set_warp(raw.warp)?;
    params.set_oct(raw.oct)?;
    params.set_contrast(raw.contrast)?;
    params.set_balance(raw.balance)?;
    params.set_grain(raw.grain)?;
    params.set_block(raw.block)?;
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

pub(crate) const PARAMS: &[Param] = &[SCALE, WARP, OCT, CONTRAST, BALANCE, BLOCK];

pub(crate) fn parameters() -> Vec<Parameter> {
    [
        SCALE.parameter(),
        WARP.parameter(),
        OCT.parameter(),
        CONTRAST.parameter(),
        BALANCE.parameter(),
        GRAIN.parameter(),
        BLOCK.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> TerrainParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    TerrainParams {
        scale: pick(&SCALE),
        warp: pick(&WARP),
        oct: pick(&OCT) as u32,
        contrast: pick(&CONTRAST),
        balance: pick(&BALANCE),
        block: pick(&BLOCK) as u32,
        ..TerrainParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &TerrainParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

const STEP: usize = 5;

fn hash(x: i32, y: i32, z: i32, w: i32, seed: u32) -> f64 {
    let mut n = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(1_440_662_683)
        ^ (w as u32).wrapping_mul(1_274_126_177)
        ^ seed.wrapping_mul(1_013_904_223);
    n = (n ^ (n >> 15)).wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(3_266_489_917);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn noise(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let fade = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash(xi + dx, yi + dy, 0, 0, seed);
    lerp(
        lerp(corner(0, 0), corner(1, 0), u),
        lerp(corner(0, 1), corner(1, 1), u),
        v,
    )
}

fn fbm(x: f64, y: f64, seed: u32, octaves: u32) -> f64 {
    let (mut weight, mut frequency, mut sum, mut total) = (0.5, 1.0, 0.0, 0.0);
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

fn paint(surface: &mut Surface, params: &TerrainParams, palette: &Palette, tool_seed: u32) {
    let (width, height) = (surface.width() as usize, surface.height() as usize);
    let (w, h) = (width as f64, height as f64);
    let fw = width.div_ceil(STEP) + 2;
    let fh = height.div_ceil(STEP) + 2;
    let mut field = vec![0.0_f32; fw * fh];
    for j in 0..fh {
        let v = (j * STEP) as f64 / h * params.scale * (h / w);
        for i in 0..fw {
            let u = (i * STEP) as f64 / w * params.scale;
            let wx = fbm(u + 5.2, v + 1.3, tool_seed.wrapping_add(11), 2);
            let wy = fbm(u + 9.1, v + 7.7, tool_seed.wrapping_add(29), 2);
            let val = fbm(
                u + params.warp * (wx - 0.5) * 2.0,
                v + params.warp * (wy - 0.5) * 2.0,
                tool_seed,
                params.oct,
            );
            field[j * fw + i] = ((val - 0.5) * params.contrast + 0.5).clamp(0.0, 1.0) as f32;
        }
    }
    let inks = palette.inks();
    let n = inks.len();
    let power = 2.0_f64.powf(-params.balance);
    let block = params.block as usize;
    let at = |i: usize| f64::from(field[i]);
    let snap = |c: usize| c.checked_div(block).map_or(c, |q| q * block);
    surface.edit_rgba(|rgba, _, _| {
        for (y, row) in rgba.chunks_exact_mut(width * 4).enumerate() {
            for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let (bx, by) = (snap(x), snap(y));
                let (fx, fy) = (bx as f64 / STEP as f64, by as f64 / STEP as f64);
                let (i0, j0) = (fx as usize, fy as usize);
                let (sx, sy) = (fx - i0 as f64, fy - j0 as f64);
                let top = j0 * fw + i0;
                let bottom = top + fw;
                let mut val = lerp(
                    lerp(at(top), at(top + 1), sx),
                    lerp(at(bottom), at(bottom + 1), sx),
                    sy,
                );
                val += (hash(x as i32, y as i32, 0, 1, tool_seed) - 0.5) * params.grain;
                let q = val.clamp(0.0, 1.0).powf(power);
                let ink = inks[((q * n as f64).floor() as usize).min(n - 1)];
                px[..3].copy_from_slice(&ink);
                px[3] = 255;
            }
        }
    });
}
