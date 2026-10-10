use serde::{Deserialize, Serialize};

use crate::aura::Xorshift;
use crate::chassis::{self, Blend, Dither, DitherKind, Grain, round_half_up};
use crate::param::Param;
use crate::surface::{Path2D, Surface, Transform};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "vee";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = Some(4);

const DEFAULT_PALETTE: [&str; 4] = ["#f4efe3", "#d62828", "#003049", "#1b1b1b"];

const ANGLE: Param = Param::new(SLUG, "angle", 15, 75, 1);
const WIDTH: Param = Param {
    step: 5,
    ..Param::new(SLUG, "width", 40, 220, 100)
};
const WEIGHT: Param = Param::new(SLUG, "weight", 15, 85, 100);
const BANDS: Param = Param::new(SLUG, "bands", 1, 4, 1);

pub(crate) const PARAMS: &[Param] = &[ANGLE, WIDTH, WEIGHT, BANDS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VeeStyle {
    Auto,
    Chevron,
    Quad,
    Diagonal,
    Cross,
}

const STYLES: [VeeStyle; 5] = [
    VeeStyle::Auto,
    VeeStyle::Chevron,
    VeeStyle::Quad,
    VeeStyle::Diagonal,
    VeeStyle::Cross,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VeeParams {
    #[serde(rename = "styles")]
    style: VeeStyle,
    angle: u32,
    width: f64,
    weight: f64,
    bands: u32,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for VeeParams {
    fn default() -> Self {
        Self {
            style: VeeStyle::Auto,
            angle: 45,
            width: 1.0,
            weight: 0.5,
            bands: 2,
            dither: Dither::default(),
            grain: Grain::default(),
        }
    }
}

impl VeeParams {
    pub fn style(&self) -> VeeStyle {
        self.style
    }

    pub fn set_style(&mut self, style: VeeStyle) {
        self.style = style;
    }

    pub fn angle(&self) -> u32 {
        self.angle
    }

    pub fn set_angle(&mut self, angle: u32) -> Result<(), Error> {
        ANGLE.check(f64::from(angle))?;
        self.angle = angle;
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) -> Result<(), Error> {
        self.width = WIDTH.check(width)?;
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
    style: VeeStyle,
    angle: u32,
    width: f64,
    weight: f64,
    bands: u32,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<VeeParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = VeeParams::default();
    params.set_style(raw.style);
    params.set_angle(raw.angle)?;
    params.set_width(raw.width)?;
    params.set_weight(raw.weight)?;
    params.set_bands(raw.bands)?;
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
        &["Auto", "Chevron", "Quad", "Diagonal", "Cross"],
    ))
    .chain(PARAMS.iter().map(Param::parameter))
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> VeeParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    VeeParams {
        style: STYLES[(draw("styles") % STYLES.len() as u64) as usize],
        angle: pick(&ANGLE) as u32,
        width: pick(&WIDTH),
        weight: pick(&WEIGHT),
        bands: pick(&BANDS) as u32,
        ..VeeParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &VeeParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Layout {
    Chevron,
    Quad,
    Diagonal,
    Cross,
}

struct Stripes {
    transform: Transform,
    rects: Path2D,
}

fn stripes(
    (x0, y0, x1, y1): (f64, f64, f64, f64),
    angle: f64,
    per: f64,
    duty: f64,
    ph: f64,
) -> Stripes {
    let reach = (x1 - x0).hypot(y1 - y0) / 2.0 + per * 2.0;
    let first = ((-reach - ph) / per).ceil() as i64;
    let last = ((reach - ph) / per).floor() as i64;
    let mut rects = Path2D::default();
    for k in first..=last {
        rects.rect(-reach, k as f64 * per + ph, reach * 2.0, per * duty);
    }
    Stripes {
        transform: Transform::IDENTITY
            .translate((x0 + x1) / 2.0, (y0 + y1) / 2.0)
            .rotate(angle),
        rects,
    }
}

fn fill_stripes(surface: &mut Surface, set: &Stripes, ink: [u8; 3]) {
    surface.fill_transformed(&set.rects, ink, set.transform);
}

fn region_clip((x0, y0, x1, y1): (f64, f64, f64, f64)) -> Path2D {
    let mut clip = Path2D::default();
    clip.rect(x0, y0, x1 - x0 + 1.0, y1 - y0 + 1.0);
    clip
}

fn paint(surface: &mut Surface, p: &VeeParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let per = w.min(h) * 0.055 * p.width;
    let a = f64::from(p.angle).to_radians();
    let ink = |i: usize| palette.ink(1 + i % 3);
    let mut rng = Xorshift::new(tool_seed);

    let n = p.bands as usize;
    let weights: Vec<f64> = (0..n).map(|_| 0.6 + rng.next() * 0.9).collect();
    let sum: f64 = weights.iter().sum();
    let mut cuts = vec![0.0];
    for (i, weight) in weights.iter().enumerate() {
        cuts.push(round_half_up(cuts[i] + weight / sum * h));
    }
    cuts[n] = h;

    surface.fill(palette.ink(0));
    let mx = w / 2.0;
    for b in 0..n {
        let (y0, y1) = (cuts[b], cuts[b + 1]);
        let layout = match p.style {
            VeeStyle::Auto => [
                Layout::Chevron,
                Layout::Quad,
                Layout::Diagonal,
                Layout::Cross,
            ][(rng.next() * 4.0) as usize],
            VeeStyle::Chevron => Layout::Chevron,
            VeeStyle::Quad => Layout::Quad,
            VeeStyle::Diagonal => Layout::Diagonal,
            VeeStyle::Cross => Layout::Cross,
        };
        let flip = if rng.next() < 0.5 { 1.0 } else { -1.0 };
        let ph = rng.next() * per;
        let base = (rng.next() * 2.0) as usize;
        let angle = a * flip;
        let (ca, cb, cc) = (ink(base), ink(base + 1), ink(base + 2));
        let single = |surface: &mut Surface, rect: (f64, f64, f64, f64), angle: f64| {
            let set = stripes(rect, angle, per, p.weight, ph);
            surface.with_clip(&region_clip(rect), Transform::IDENTITY, |surface| {
                fill_stripes(surface, &set, ca);
            });
        };
        match layout {
            Layout::Diagonal => single(surface, (0.0, y0, w, y1), angle),
            Layout::Chevron => {
                single(surface, (0.0, y0, mx, y1), angle);
                single(surface, (mx, y0, w, y1), -angle);
            }
            Layout::Quad => {
                let my = (y0 + y1) / 2.0;
                single(surface, (0.0, y0, mx, my), angle);
                single(surface, (mx, y0, w, my), -angle);
                single(surface, (0.0, my, mx, y1), -angle);
                single(surface, (mx, my, w, y1), angle);
            }
            Layout::Cross => {
                for (x0, x1, sign) in [(0.0, mx, 1.0), (mx, w, -1.0)] {
                    let rect = (x0, y0, x1, y1);
                    let one = stripes(rect, angle * sign, per, p.weight, ph);
                    let two = stripes(rect, -angle * sign, per, p.weight, ph);
                    surface.with_clip(&region_clip(rect), Transform::IDENTITY, |surface| {
                        fill_stripes(surface, &one, ca);
                        fill_stripes(surface, &two, cb);
                        surface.with_clip(&one.rects, one.transform, |surface| {
                            fill_stripes(surface, &two, cc);
                        });
                    });
                }
            }
        }
    }
}
