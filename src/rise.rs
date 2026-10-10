use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "rise";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
const INKS: usize = 3;
pub(crate) const MAX_INKS: Option<usize> = Some(INKS);

const DEFAULT_PALETTE: [&str; INKS] = ["#ff9f1c", "#2ec4b6", "#011627"];

const COUNT: Param = Param::new(SLUG, "count", 6, 28, 1);
const WEIGHT: Param = Param::new(SLUG, "weight", 30, 70, 100);
const BANDS: Param = Param::new(SLUG, "bands", 1, 8, 1);
const DEPTH: Param = Param::new(SLUG, "depth", 10, 40, 100);

pub(crate) const PARAMS: &[Param] = &[COUNT, WEIGHT, BANDS, DEPTH];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiseAnchor {
    Bottom,
    Top,
    Left,
    Right,
    Centre,
}

const ANCHORS: [RiseAnchor; 5] = [
    RiseAnchor::Bottom,
    RiseAnchor::Top,
    RiseAnchor::Left,
    RiseAnchor::Right,
    RiseAnchor::Centre,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RiseParams {
    #[serde(rename = "styles")]
    anchor: RiseAnchor,
    count: u32,
    weight: f64,
    bands: u32,
    depth: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for RiseParams {
    fn default() -> Self {
        Self {
            anchor: RiseAnchor::Bottom,
            count: 14,
            weight: 0.5,
            bands: 3,
            depth: 0.24,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl RiseParams {
    pub fn anchor(&self) -> RiseAnchor {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: RiseAnchor) {
        self.anchor = anchor;
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) -> Result<(), Error> {
        COUNT.check(f64::from(count))?;
        self.count = count;
        Ok(())
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) -> Result<(), Error> {
        self.weight = WEIGHT.check(weight)?;
        Ok(())
    }

    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn depth(&self) -> f64 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: f64) -> Result<(), Error> {
        self.depth = DEPTH.check(depth)?;
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
    #[serde(rename = "styles")]
    anchor: RiseAnchor,
    count: u32,
    weight: f64,
    bands: u32,
    depth: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<RiseParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = RiseParams::default();
    params.set_anchor(raw.anchor);
    params.set_count(raw.count)?;
    params.set_weight(raw.weight)?;
    params.set_bands(raw.bands)?;
    params.set_depth(raw.depth)?;
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
    std::iter::once(Parameter::choice(
        "styles",
        &["Bottom", "Top", "Left", "Right", "Centre"],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> RiseParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    RiseParams {
        anchor: ANCHORS[(draw("styles") % ANCHORS.len() as u64) as usize],
        count: pick(&COUNT) as u32,
        weight: pick(&WEIGHT),
        bands: pick(&BANDS) as u32,
        depth: pick(&DEPTH),
        ..RiseParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &RiseParams,
    palette: &Palette,
    _tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette);
    chassis::finish(surface, &params.grain, &params.dither);
}

const PAIRS: [(usize, usize); 4] = [(1, 2), (2, 0), (1, 0), (2, 1)];

fn paint(surface: &mut Surface, p: &RiseParams, palette: &Palette) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let md = w.min(h);
    let period = md / f64::from(p.count.max(4));
    let bar = period * p.weight;
    let band = p.depth * md;
    let (ax, ay) = match p.anchor {
        RiseAnchor::Bottom => (w / 2.0, h),
        RiseAnchor::Top => (w / 2.0, 0.0),
        RiseAnchor::Left => (0.0, h / 2.0),
        RiseAnchor::Right => (w, h / 2.0),
        RiseAnchor::Centre => (w / 2.0, h / 2.0),
    };
    let bars = |surface: &mut Surface, ink: [u8; 3]| {
        let cx = w / 2.0;
        let first = ((-cx - bar) / period).ceil() as i64;
        let last = ((w - cx + bar) / period).floor() as i64;
        for k in first..=last {
            let x = cx + k as f64 * period - bar / 2.0;
            let (x0, x1) = (round_half_up(x), round_half_up(x + bar));
            surface.fill_box(x0, -2.0, x1 - x0, h + 4.0, ink);
        }
    };
    surface.fill(palette.ink(0));
    bars(surface, palette.ink(1));
    for i in (0..p.bands.max(1)).rev() {
        let (ground, ink) = PAIRS[i as usize % 4];
        let mut disc = Path2D::default();
        disc.push_circle(ax, ay, (f64::from(i) + 0.8) * band);
        surface.with_clip(&disc, Transform::IDENTITY, |surface| {
            surface.fill(palette.ink(ground));
            bars(surface, palette.ink(ink));
        });
    }
}
