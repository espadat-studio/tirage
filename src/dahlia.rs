use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, XorShift, hash};
use crate::param::Param;
use crate::surface::{Cap, Join, Path2D, Surface};
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "dahlia";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 12;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 5] = ["#f6eedc", "#3fd3f0", "#ff6fd8", "#fff23a", "#f52a2a"];

const RAYS: Param = Param::new(SLUG, "rays", 8, 300, 1);
const SIZE: Param = Param::new(SLUG, "size", 0, 100, 100);
const RAGGED: Param = Param::new(SLUG, "ragged", 0, 100, 100);
const CX: Param = Param::new(SLUG, "cx", 10, 90, 100);
const CY: Param = Param::new(SLUG, "cy", 10, 90, 100);
const WIDTH: Param = Param::new(SLUG, "width", 0, 100, 100);
const DASH: Param = Param::new(SLUG, "dash", 0, 100, 100);
const GAP: Param = Param::new(SLUG, "gap", 0, 100, 100);
const BEND: Param = Param::new(SLUG, "bend", 0, 100, 100);
const CAPS: Param = Param::new(SLUG, "caps", 0, 100, 100);

pub(crate) const PARAMS: &[Param] = &[RAYS, SIZE, RAGGED, CX, CY, WIDTH, DASH, GAP, BEND, CAPS];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DahliaParams {
    rays: u32,
    size: f64,
    ragged: f64,
    cx: f64,
    cy: f64,
    width: f64,
    dash: f64,
    gap: f64,
    bend: f64,
    caps: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    grain: Grain,
}

impl Default for DahliaParams {
    fn default() -> Self {
        Self {
            rays: 160,
            size: 0.85,
            ragged: 0.4,
            cx: 0.5,
            cy: 0.5,
            width: 0.6,
            dash: 0.5,
            gap: 0.4,
            bend: 0.3,
            caps: 0.8,
            dither: Dither::default(),
            grain: Grain::poster(),
        }
    }
}

impl DahliaParams {
    pub fn rays(&self) -> u32 {
        self.rays
    }

    pub fn set_rays(&mut self, rays: u32) -> Result<(), Error> {
        RAYS.check(f64::from(rays))?;
        self.rays = rays;
        Ok(())
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) -> Result<(), Error> {
        self.size = SIZE.check(size)?;
        Ok(())
    }

    pub fn ragged(&self) -> f64 {
        self.ragged
    }

    pub fn set_ragged(&mut self, ragged: f64) -> Result<(), Error> {
        self.ragged = RAGGED.check(ragged)?;
        Ok(())
    }

    pub fn cx(&self) -> f64 {
        self.cx
    }

    pub fn set_cx(&mut self, cx: f64) -> Result<(), Error> {
        self.cx = CX.check(cx)?;
        Ok(())
    }

    pub fn cy(&self) -> f64 {
        self.cy
    }

    pub fn set_cy(&mut self, cy: f64) -> Result<(), Error> {
        self.cy = CY.check(cy)?;
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) -> Result<(), Error> {
        self.width = WIDTH.check(width)?;
        Ok(())
    }

    pub fn dash(&self) -> f64 {
        self.dash
    }

    pub fn set_dash(&mut self, dash: f64) -> Result<(), Error> {
        self.dash = DASH.check(dash)?;
        Ok(())
    }

    pub fn gap(&self) -> f64 {
        self.gap
    }

    pub fn set_gap(&mut self, gap: f64) -> Result<(), Error> {
        self.gap = GAP.check(gap)?;
        Ok(())
    }

    pub fn bend(&self) -> f64 {
        self.bend
    }

    pub fn set_bend(&mut self, bend: f64) -> Result<(), Error> {
        self.bend = BEND.check(bend)?;
        Ok(())
    }

    pub fn caps(&self) -> f64 {
        self.caps
    }

    pub fn set_caps(&mut self, caps: f64) -> Result<(), Error> {
        self.caps = CAPS.check(caps)?;
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
    rays: u32,
    size: f64,
    ragged: f64,
    cx: f64,
    cy: f64,
    width: f64,
    dash: f64,
    gap: f64,
    bend: f64,
    caps: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<DahliaParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = DahliaParams::default();
    params.set_rays(raw.rays)?;
    params.set_size(raw.size)?;
    params.set_ragged(raw.ragged)?;
    params.set_cx(raw.cx)?;
    params.set_cy(raw.cy)?;
    params.set_width(raw.width)?;
    params.set_dash(raw.dash)?;
    params.set_gap(raw.gap)?;
    params.set_bend(raw.bend)?;
    params.set_caps(raw.caps)?;
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
    PARAMS
        .iter()
        .map(Param::parameter)
        .chain(chassis::parameters())
        .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> DahliaParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    DahliaParams {
        rays: pick(&RAYS) as u32,
        size: pick(&SIZE),
        ragged: pick(&RAGGED),
        cx: pick(&CX),
        cy: pick(&CY),
        width: pick(&WIDTH),
        dash: pick(&DASH),
        gap: pick(&GAP),
        bend: pick(&BEND),
        caps: pick(&CAPS),
        ..DahliaParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &DahliaParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.grain, &params.dither);
}

fn paint(surface: &mut Surface, p: &DahliaParams, palette: &Palette, tool_seed: u32) {
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let inks = palette.inks();
    let (paper, accent) = (inks[0], inks[inks.len() - 1]);
    let rays: &[[u8; 3]] = if inks.len() > 2 {
        &inks[1..inks.len() - 1]
    } else {
        std::slice::from_ref(&inks[inks.len() - 1])
    };
    surface.fill(paper);

    let unit = (w * h).sqrt();
    let (cx, cy) = (p.cx * w, p.cy * h);
    let radius = unit * (0.3 + p.size * 0.55);
    let n = p.rays;
    let w0 = unit * (0.012 + p.width * 0.03);
    let dash = unit * (0.03 + p.dash * 0.07);
    let period = dash + dash * (0.15 + p.gap * 1.1);
    let mut rng = XorShift::new(tool_seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f);
    let lean = (rng.next() - 0.5) * 0.5;
    let h = |i: u32, k: u32| hash(i as i32, k as i32, tool_seed);

    for i in 0..n {
        let angle = (f64::from(i) + 0.5) / f64::from(n) * TAU
            + (h(i, 1) - 0.5) * (TAU / f64::from(n)) * 1.6;
        let len = radius * (1.0 - p.ragged * h(i, 2) * 0.9) * (0.55 + 0.45 * h(i, 3));
        let (ca, sa) = (angle.cos(), angle.sin());
        let bow = (h(i, 4) - 0.5 + lean * 0.5) * p.bend * len * 0.9;
        let (mx, my) = (
            cx + ca * len * 0.5 - sa * bow,
            cy + sa * len * 0.5 + ca * bow,
        );
        let (ex, ey) = (cx + ca * len, cy + sa * len);
        let at = |t: f64| {
            let u = 1.0 - t;
            (
                u * u * cx + 2.0 * u * t * mx + t * t * ex,
                u * u * cy + 2.0 * u * t * my + t * t * ey,
            )
        };
        let mut s = h(i, 5) * period - period;
        let mut k = 0;
        while s < len {
            let t0 = s.max(0.0) / len;
            let t1 = len.min(s + dash * (0.45 + 0.8 * t0)) / len;
            if t1 > t0 && t1 > 0.0 {
                let wd = w0 * (0.4 + t0);
                let (hk, hk2) = (h(i, k + 11), h(i, k + 31));
                let first = if hk < 0.55 {
                    0
                } else {
                    (hk * rays.len() as f64) as usize
                };
                let layers = rays.len().min(1 + (hk2 * 2.5) as usize);
                for l in 0..layers {
                    let (off, line) = [(0.0, 1.0), (0.32, 0.62), (-0.28, 0.36)][l];
                    let mut path = Path2D::default();
                    for q in 0..=4 {
                        let t = t0 + (t1 - t0) * f64::from(q) / 4.0;
                        let (x, y) = at(t);
                        let u = 1.0 - t;
                        let dx = 2.0 * u * (mx - cx) + 2.0 * t * (ex - mx);
                        let dy = 2.0 * u * (my - cy) + 2.0 * t * (ey - my);
                        let d = match dx.hypot(dy) {
                            0.0 => 1.0,
                            d => d,
                        };
                        let (x, y) = (x - dy / d * off * wd, y + dx / d * off * wd);
                        if q == 0 {
                            path.move_to(x, y);
                        } else {
                            path.line_to(x, y);
                        }
                    }
                    let ink = rays[(first + l) % rays.len()];
                    surface.stroke(&path, ink, wd * line, Cap::Round, Join::Round);
                }
                if h(i, k + 51) < p.caps {
                    let (x, y) = at(t1);
                    let bead = wd * 0.5 * (0.8 + 0.5 * h(i, k + 61));
                    surface.fill_circle(x, y, bead, accent);
                    if h(i, k + 71) < p.caps * 0.35 {
                        let (x, y) = at(t0);
                        surface.fill_circle(x, y, bead * 0.8, accent);
                    }
                }
            }
            s += period;
            k += 1;
        }
    }
}
