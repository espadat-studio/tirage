use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, value_noise,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "stitch";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 6] = [
    "#111111", "#f6f23a", "#f01fd0", "#62f19e", "#f5532b", "#ded7eb",
];

const COLS: Param = Param::new(SLUG, "cols", 16, 96, 1);
const GUTTER: Param = Param {
    taste: (0, 45),
    ..Param::new(SLUG, "gutter", 0, 100, 100)
};
const STREAK: Param = Param::new(SLUG, "streak", 0, 100, 100);
const SCALE: Param = Param::new(SLUG, "scale", 0, 100, 100);
const SLIP: Param = Param::new(SLUG, "slip", 0, 100, 100);
const BAND: Param = Param::new(SLUG, "band", 1, 16, 1);
const BLOBS: Param = Param::new(SLUG, "blobs", 0, 100, 100);
const BLOBSIZE: Param = Param::new(SLUG, "blobsize", 0, 100, 100);
const STEPS: Param = Param::new(SLUG, "steps", 2, 12, 1);
const GROUND: Param = Param {
    taste: (0, 70),
    ..Param::new(SLUG, "ground", 0, 100, 100)
};
const STRAY: Param = Param::new(SLUG, "stray", 0, 100, 100);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StitchParams {
    cols: u32,
    gutter: f64,
    streak: f64,
    scale: f64,
    slip: f64,
    band: u32,
    blobs: f64,
    blobsize: f64,
    steps: u32,
    ground: f64,
    stray: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for StitchParams {
    fn default() -> Self {
        Self {
            cols: 48,
            gutter: 0.2,
            streak: 0.8,
            scale: 0.5,
            slip: 0.4,
            band: 5,
            blobs: 0.35,
            blobsize: 0.5,
            steps: 7,
            ground: 0.35,
            stray: 0.3,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl StitchParams {
    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn gutter(&self) -> f64 {
        self.gutter
    }

    pub fn set_gutter(&mut self, gutter: f64) -> Result<(), Error> {
        self.gutter = GUTTER.check(gutter)?;
        Ok(())
    }

    pub fn streak(&self) -> f64 {
        self.streak
    }

    pub fn set_streak(&mut self, streak: f64) -> Result<(), Error> {
        self.streak = STREAK.check(streak)?;
        Ok(())
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f64) -> Result<(), Error> {
        self.scale = SCALE.check(scale)?;
        Ok(())
    }

    pub fn slip(&self) -> f64 {
        self.slip
    }

    pub fn set_slip(&mut self, slip: f64) -> Result<(), Error> {
        self.slip = SLIP.check(slip)?;
        Ok(())
    }

    pub fn band(&self) -> u32 {
        self.band
    }

    pub fn set_band(&mut self, band: u32) -> Result<(), Error> {
        BAND.check(f64::from(band))?;
        self.band = band;
        Ok(())
    }

    pub fn blobs(&self) -> f64 {
        self.blobs
    }

    pub fn set_blobs(&mut self, blobs: f64) -> Result<(), Error> {
        self.blobs = BLOBS.check(blobs)?;
        Ok(())
    }

    pub fn blobsize(&self) -> f64 {
        self.blobsize
    }

    pub fn set_blobsize(&mut self, blobsize: f64) -> Result<(), Error> {
        self.blobsize = BLOBSIZE.check(blobsize)?;
        Ok(())
    }

    pub fn steps(&self) -> u32 {
        self.steps
    }

    pub fn set_steps(&mut self, steps: u32) -> Result<(), Error> {
        STEPS.check(f64::from(steps))?;
        self.steps = steps;
        Ok(())
    }

    pub fn ground(&self) -> f64 {
        self.ground
    }

    pub fn set_ground(&mut self, ground: f64) -> Result<(), Error> {
        self.ground = GROUND.check(ground)?;
        Ok(())
    }

    pub fn stray(&self) -> f64 {
        self.stray
    }

    pub fn set_stray(&mut self, stray: f64) -> Result<(), Error> {
        self.stray = STRAY.check(stray)?;
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
    cols: u32,
    gutter: f64,
    streak: f64,
    scale: f64,
    slip: f64,
    band: u32,
    blobs: f64,
    blobsize: f64,
    steps: u32,
    ground: f64,
    stray: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<StitchParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = StitchParams::default();
    params.set_cols(raw.cols)?;
    params.set_gutter(raw.gutter)?;
    params.set_streak(raw.streak)?;
    params.set_scale(raw.scale)?;
    params.set_slip(raw.slip)?;
    params.set_band(raw.band)?;
    params.set_blobs(raw.blobs)?;
    params.set_blobsize(raw.blobsize)?;
    params.set_steps(raw.steps)?;
    params.set_ground(raw.ground)?;
    params.set_stray(raw.stray)?;
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

pub(crate) const PARAMS: &[Param] = &[
    COLS, GUTTER, STREAK, SCALE, SLIP, BAND, BLOBS, BLOBSIZE, STEPS, GROUND, STRAY,
];

pub(crate) fn parameters() -> Vec<Parameter> {
    [
        COLS.parameter(),
        GUTTER.parameter(),
        STREAK.parameter(),
        SCALE.parameter(),
        SLIP.parameter(),
        BAND.parameter(),
        BLOBS.parameter(),
        BLOBSIZE.parameter(),
        STEPS.parameter(),
        GROUND.parameter(),
        STRAY.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> StitchParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    StitchParams {
        cols: pick(&COLS) as u32,
        gutter: pick(&GUTTER),
        streak: pick(&STREAK),
        scale: pick(&SCALE),
        slip: pick(&SLIP),
        band: pick(&BAND) as u32,
        blobs: pick(&BLOBS),
        blobsize: pick(&BLOBSIZE),
        steps: pick(&STEPS) as u32,
        ground: pick(&GROUND),
        stray: pick(&STRAY),
        ..StitchParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &StitchParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

fn luma([r, g, b]: [u8; 3]) -> f64 {
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}

fn paint(surface: &mut Surface, params: &StitchParams, palette: &Palette, tool_seed: u32) {
    let inks = palette.inks();
    let n = inks.len();
    let seed = |k: u32| tool_seed.wrapping_mul(7).wrapping_add(k);
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let cols = params.cols.max(4);
    let cw = w / f64::from(cols);
    let rows = round_half_up(h / cw).max(1.0) as u32;
    let ch = h / f64::from(rows);
    let gutter = round_half_up(params.gutter * cw * 0.35);
    let kx = 3.0 + (1.0 - params.scale) * 6.0;
    let ky = kx * (1.0 - params.streak * 0.92) * (h / w);
    let sw = 0.25 + params.streak * 0.4;
    let band_rows = params.band.max(1);
    let slip = params.slip * f64::from(cols) * 0.3;
    let kb = 3.0 + (1.0 - params.blobsize) * 6.0;
    let blob_cut = 1.0 - params.blobs * 0.42;
    let stray = params.stray * 0.06;

    let steps = params.steps.max(2);
    let mut rnd = XorShift::new(tool_seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let mut mids: Vec<usize> = (2..n).collect();
    if mids.is_empty() {
        mids.push(if n > 1 { 1 } else { 0 });
    }
    for k in (1..mids.len()).rev() {
        let j = (rnd.next() * (k + 1) as f64) as usize;
        mids.swap(k, j);
    }
    let grounded = (steps - 1).min(round_half_up(f64::from(steps) * params.ground) as u32);
    let levels: Vec<usize> = (0..steps)
        .map(|s| {
            if s < grounded {
                0
            } else {
                mids[(s - grounded) as usize % mids.len()]
            }
        })
        .collect();
    let accent = if n > 1 { 1 } else { 0 };
    let gutter_ink = (1..n).fold(0, |darkest, k| {
        if luma(inks[k]) < luma(inks[darkest]) {
            k
        } else {
            darkest
        }
    });

    surface.fill(inks[gutter_ink]);
    for j in 0..rows {
        let band = (j / band_rows) as i32;
        let offset = (hash(band, 3, seed(1)) - 0.5) * 2.0 * slip;
        let y0 = round_half_up(f64::from(j) * ch) + gutter;
        let y1 = round_half_up(f64::from(j + 1) * ch);
        if y1 - y0 <= 0.0 {
            continue;
        }
        for i in 0..cols {
            let u = (f64::from(i) + offset) / f64::from(cols);
            let v = f64::from(j) / f64::from(rows);
            let base = value_noise(u * kx, v * ky, seed(2));
            let stripe = value_noise(u * kx * 6.0 + 7.0, v * ky * 1.5 + 3.0, seed(3));
            let mixed = base * (1.0 - sw) + stripe * sw;
            let val = ((mixed - 0.5) * 2.2 + 0.5).clamp(0.0, 0.9999);
            let mut ink = levels[(val * f64::from(steps)) as usize];
            let blob = value_noise(
                f64::from(i) / f64::from(cols) * kb + 11.0,
                f64::from(j) / f64::from(rows) * kb * (h / w) + 5.0,
                seed(4),
            );
            if blob > blob_cut {
                ink = accent;
            }
            let draw = hash(i as i32, j as i32, tool_seed.wrapping_add(9));
            if draw < stray {
                ink = (draw / stray * n as f64) as usize % n;
            }
            let x0 = round_half_up(f64::from(i) * cw) + gutter;
            let x1 = round_half_up(f64::from(i + 1) * cw);
            if x1 - x0 > 0.0 {
                surface.fill_rect(
                    x0 as i32,
                    y0 as i32,
                    (x1 - x0) as u32,
                    (y1 - y0) as u32,
                    inks[ink],
                );
            }
        }
    }
}
