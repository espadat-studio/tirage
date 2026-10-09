use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::chassis::{
    self, Blend, Dither, DitherKind, Grain, XorShift, hash, round_half_up, store, value_noise,
};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "pith";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 5] = ["#fbf8ef", "#f26ca7", "#7b2cbf", "#2b1b4a", "#141414"];

const COUNT: Param = Param::new(SLUG, "count", 1, 60, 1);
const SIZE: Param = Param {
    taste: (20, 100),
    ..Param::new(SLUG, "size", 0, 100, 100)
};
const ZOOM: Param = Param {
    taste: (10, 30),
    ..Param::new(SLUG, "zoom", 10, 80, 10)
};
const ROUND: Param = Param::new(SLUG, "round", 0, 100, 100);
const WOBBLE: Param = Param::new(SLUG, "wobble", 0, 100, 100);
const BAND: Param = Param::new(SLUG, "band", 0, 100, 100);
const DITHER: Param = Param::new(SLUG, "dither", 0, 100, 100);
const VEINS: Param = Param::new(SLUG, "veins", 0, 100, 100);
const THICK: Param = Param::new(SLUG, "thick", 0, 100, 100);
const GRAIN: Param = Param::new(SLUG, "grain", 0, 100, 100);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PithParams {
    count: u32,
    size: f64,
    zoom: f64,
    round: f64,
    wobble: f64,
    band: f64,
    dither: f64,
    veins: f64,
    thick: f64,
    grain: f64,
    #[serde(flatten)]
    chassis_dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for PithParams {
    fn default() -> Self {
        Self {
            count: 28,
            size: 0.45,
            zoom: 1.0,
            round: 0.55,
            wobble: 0.5,
            band: 0.45,
            dither: 0.55,
            veins: 0.5,
            thick: 0.45,
            grain: 0.4,
            chassis_dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl PithParams {
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

    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f64) -> Result<(), Error> {
        self.zoom = ZOOM.check(zoom)?;
        Ok(())
    }

    pub fn round(&self) -> f64 {
        self.round
    }

    pub fn set_round(&mut self, round: f64) -> Result<(), Error> {
        self.round = ROUND.check(round)?;
        Ok(())
    }

    pub fn wobble(&self) -> f64 {
        self.wobble
    }

    pub fn set_wobble(&mut self, wobble: f64) -> Result<(), Error> {
        self.wobble = WOBBLE.check(wobble)?;
        Ok(())
    }

    pub fn band(&self) -> f64 {
        self.band
    }

    pub fn set_band(&mut self, band: f64) -> Result<(), Error> {
        self.band = BAND.check(band)?;
        Ok(())
    }

    pub fn dither(&self) -> f64 {
        self.dither
    }

    pub fn set_dither(&mut self, dither: f64) -> Result<(), Error> {
        self.dither = DITHER.check(dither)?;
        Ok(())
    }

    pub fn veins(&self) -> f64 {
        self.veins
    }

    pub fn set_veins(&mut self, veins: f64) -> Result<(), Error> {
        self.veins = VEINS.check(veins)?;
        Ok(())
    }

    pub fn thick(&self) -> f64 {
        self.thick
    }

    pub fn set_thick(&mut self, thick: f64) -> Result<(), Error> {
        self.thick = THICK.check(thick)?;
        Ok(())
    }

    pub fn grain(&self) -> f64 {
        self.grain
    }

    pub fn set_grain(&mut self, grain: f64) -> Result<(), Error> {
        self.grain = GRAIN.check(grain)?;
        Ok(())
    }

    pub fn dither_pass(&self) -> &Dither {
        &self.chassis_dither
    }

    pub fn dither_pass_mut(&mut self) -> &mut Dither {
        &mut self.chassis_dither
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
    zoom: f64,
    round: f64,
    wobble: f64,
    band: f64,
    dither: f64,
    veins: f64,
    thick: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<PithParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = PithParams::default();
    params.set_count(raw.count)?;
    params.set_size(raw.size)?;
    params.set_zoom(raw.zoom)?;
    params.set_round(raw.round)?;
    params.set_wobble(raw.wobble)?;
    params.set_band(raw.band)?;
    params.set_dither(raw.dither)?;
    params.set_veins(raw.veins)?;
    params.set_thick(raw.thick)?;
    params.set_grain(raw.grain)?;
    let dither = params.dither_pass_mut();
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

pub(crate) const PARAMS: &[Param] = &[COUNT, SIZE, ZOOM, ROUND, WOBBLE, BAND, VEINS, THICK];

pub(crate) fn parameters() -> Vec<Parameter> {
    [
        COUNT.parameter(),
        SIZE.parameter(),
        ZOOM.parameter(),
        ROUND.parameter(),
        WOBBLE.parameter(),
        BAND.parameter(),
        DITHER.parameter(),
        VEINS.parameter(),
        THICK.parameter(),
        GRAIN.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> PithParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    PithParams {
        count: pick(&COUNT) as u32,
        size: pick(&SIZE),
        zoom: pick(&ZOOM),
        round: pick(&ROUND),
        wobble: pick(&WOBBLE),
        band: pick(&BAND),
        veins: pick(&VEINS),
        thick: pick(&THICK),
        ..PithParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &PithParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.chassis_dither);
}

struct RoundBox {
    cx: f64,
    cy: f64,
    hw: f64,
    hh: f64,
    r: f64,
}

impl RoundBox {
    fn distance(&self, x: f64, y: f64) -> f64 {
        let r = self.r.min(self.hw).min(self.hh);
        let dx = (x - self.cx).abs() - (self.hw - r);
        let dy = (y - self.cy).abs() - (self.hh - r);
        let (ax, ay) = (dx.max(0.0), dy.max(0.0));
        (ax * ax + ay * ay).sqrt() + dx.max(dy).min(0.0) - r
    }
}

struct Cell {
    boxes: Vec<RoundBox>,
    veins: Vec<[f64; 4]>,
    bounds: [f64; 4],
    seed: u32,
}

fn segment_distance(x: f64, y: f64, [x0, y0, x1, y1]: [f64; 4]) -> f64 {
    let (vx, vy, wx, wy) = (x1 - x0, y1 - y0, x - x0, y - y0);
    let length = vx * vx + vy * vy;
    let t = if length > 0.0 {
        ((wx * vx + wy * vy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (wx - vx * t, wy - vy * t);
    (dx * dx + dy * dy).sqrt()
}

struct Walk<'a> {
    rnd: XorShift,
    step: f64,
    veins: &'a mut Vec<[f64; 4]>,
}

impl Walk<'_> {
    fn walk(&mut self, mut x: f64, mut y: f64, mut a: f64, steps: u32, depth: u32) {
        for _ in 0..steps {
            let (nx, ny) = (x + a.cos() * self.step, y + a.sin() * self.step);
            self.veins.push([x, y, nx, ny]);
            (x, y) = (nx, ny);
            a += (self.rnd.next() - 0.5) * 0.9;
            if depth < 2 && self.rnd.next() < 0.42 {
                let sign = if self.rnd.next() < 0.5 { -1.0 } else { 1.0 };
                let turn = a + sign * (0.6 + self.rnd.next() * 0.6);
                self.walk(
                    x,
                    y,
                    turn,
                    ((f64::from(steps) * 0.6) as u32).max(1),
                    depth + 1,
                );
            }
        }
    }
}

fn layout(params: &PithParams, tool_seed: u32, w: f64, h: f64) -> (Vec<Cell>, f64) {
    let mut rnd = XorShift::new(tool_seed.wrapping_add(3).wrapping_mul(2_654_435_761));
    let margin = w.min(h) * 0.085;
    let (iw, ih) = (w - 2.0 * margin, h - 2.0 * margin);
    let count = params.count.max(1);
    let base = (iw * ih / f64::from(count)).sqrt() * (0.28 + params.size * 0.95);
    let zoom = params.zoom.max(1.0);
    let (zx, zy) = (w * 0.5, h * 0.5);
    let mut cells = Vec::new();
    for _ in 0..count {
        let cw = base * (0.75 + rnd.next() * 1.40);
        let ch = base * (0.75 + rnd.next() * 1.25);
        let x0 = margin + rnd.next() * (iw - cw).max(1.0);
        let y0 = margin + rnd.next() * (ih - ch).max(1.0);
        let count = 3 + (rnd.next() * 2.0) as usize;
        let mut boxes: Vec<RoundBox> = (0..count)
            .map(|k| {
                let bw = cw
                    * if k > 0 {
                        0.34 + rnd.next() * 0.44
                    } else {
                        0.62 + rnd.next() * 0.32
                    };
                let bh = ch
                    * if k > 0 {
                        0.34 + rnd.next() * 0.46
                    } else {
                        0.58 + rnd.next() * 0.34
                    };
                let bx = x0 + rnd.next() * (cw - bw);
                let by = y0 + rnd.next() * (ch - bh);
                RoundBox {
                    cx: bx + bw * 0.5,
                    cy: by + bh * 0.5,
                    hw: bw * 0.5,
                    hh: bh * 0.5,
                    r: bw.min(bh) * 0.5 * params.round,
                }
            })
            .collect();
        let vein_seed = (rnd.next() * 4_294_967_295.0) as u32;
        let seed = (rnd.next() * 9973.0) as u32;
        for b in &mut boxes {
            b.cx = zx + (b.cx - zx) * zoom;
            b.cy = zy + (b.cy - zy) * zoom;
            b.hw *= zoom;
            b.hh *= zoom;
            b.r *= zoom;
        }
        let bounds = boxes
            .iter()
            .fold([1e9_f64, 1e9, -1e9, -1e9], |[x0, y0, x1, y1], b| {
                [
                    x0.min(b.cx - b.hw),
                    y0.min(b.cy - b.hh),
                    x1.max(b.cx + b.hw),
                    y1.max(b.cy + b.hh),
                ]
            });
        let slack = base * zoom;
        if bounds[2] < -slack
            || bounds[0] > w + slack
            || bounds[3] < -slack
            || bounds[1] > h + slack
        {
            continue;
        }
        let mut veins = Vec::new();
        let mut walk = Walk {
            rnd: XorShift::new(vein_seed),
            step: cw.min(ch) * zoom * 0.095,
            veins: &mut veins,
        };
        for _ in 0..round_half_up(params.veins * 6.0) as u32 {
            let b = &boxes[(walk.rnd.next() * boxes.len() as f64) as usize];
            let x = b.cx + (walk.rnd.next() - 0.5) * b.hw * 1.1;
            let y = b.cy + (walk.rnd.next() - 0.5) * b.hh * 1.1;
            let a = walk.rnd.next() * PI * 2.0;
            let steps = 4 + (walk.rnd.next() * 5.0) as u32;
            walk.walk(x, y, a, steps, 0);
        }
        cells.push(Cell {
            boxes,
            veins,
            bounds,
            seed,
        });
    }
    (cells, base * zoom)
}

fn span(low: f64, high: f64, last: usize) -> std::ops::Range<usize> {
    let low = low.floor().max(0.0);
    let high = high.ceil().min(last as f64);
    if high < low {
        return 0..0;
    }
    low as usize..high as usize + 1
}

fn paint(surface: &mut Surface, params: &PithParams, palette: &Palette, tool_seed: u32) {
    let ink = |slots: &[usize]| {
        let slot = slots
            .iter()
            .copied()
            .find(|&slot| slot < palette.len())
            .unwrap_or(1);
        palette.ink(slot).map(f64::from)
    };
    let ground = ink(&[0]);
    let edge = ink(&[1]);
    let rim = ink(&[2, 1]);
    let core = ink(&[3, 2, 1]);
    let fleck = ink(&[4, 1]);
    let (width, height) = (surface.width() as usize, surface.height() as usize);
    let (w, h) = (width as f64, height as f64);
    let (cells, cs) = layout(params, tool_seed, w, h);

    let ns = cs * (0.10 + params.wobble * 0.18);
    let amp = cs * 0.12 * params.wobble;
    let pad = (amp * 2.2 + cs * 0.04).ceil();
    let far = amp * 1.5 + 2.0;
    let vt = cs * (0.0067 + params.thick * 0.033);
    let mut depth = vec![1e9_f32; width * height];
    let mut vein = vec![1e9_f32; width * height];
    for cell in &cells {
        let [bx0, by0, bx1, by1] = cell.bounds;
        for y in span(by0 - pad, by1 + pad, height - 1) {
            for x in span(bx0 - pad, bx1 + pad, width - 1) {
                let (xf, yf) = (x as f64, y as f64);
                let mut d = cell
                    .boxes
                    .iter()
                    .fold(1e9, |d: f64, b| d.min(b.distance(xf, yf)));
                let i = y * width + x;
                if d <= far {
                    d += (value_noise(xf / ns, yf / ns, cell.seed) - 0.5) * amp * 2.0
                        + (value_noise(xf / (ns * 0.28), yf / (ns * 0.28), cell.seed + 31) - 0.5)
                            * amp
                            * 0.8;
                }
                if d < f64::from(depth[i]) {
                    depth[i] = d as f32;
                }
            }
        }
        for &segment in &cell.veins {
            let [x0, y0, x1, y1] = segment;
            for y in span(y0.min(y1) - vt - 1.0, y0.max(y1) + vt + 1.0, height - 1) {
                for x in span(x0.min(x1) - vt - 1.0, x0.max(x1) + vt + 1.0, width - 1) {
                    let q = segment_distance(x as f64, y as f64, segment) - vt;
                    let i = y * width + x;
                    if q < f64::from(vein[i]) {
                        vein[i] = q as f32;
                    }
                }
            }
        }
    }

    let band = cs * (0.033 + params.band * 0.190);
    let rim_width = cs * (0.020 + params.band * 0.070);
    let nb = cs * 0.31;
    let gsc = ((w * h).sqrt() / 620.0).max(1.0);
    let dth = band * (0.12 + params.dither * 1.5);
    let grain = params.grain;
    surface.edit_rgba(|rgba, _, _| {
        for (y, row) in rgba.chunks_exact_mut(width * 4).enumerate() {
            for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let i = y * width + x;
                let (xf, yf) = (x as f64, y as f64);
                let (gx, gy) = ((xf / gsc) as i32, (yf / gsc) as i32);
                let mut c = ground;
                let d = f64::from(depth[i]);
                if d < 0.0 {
                    let t = -d
                        + (hash(gx, gy, tool_seed) - 0.5) * dth
                        + (value_noise(
                            xf / (6.5 * gsc),
                            yf / (6.5 * gsc),
                            tool_seed.wrapping_add(17),
                        ) - 0.5)
                            * dth
                            * 1.4;
                    let ev = band
                        * (0.40
                            + value_noise(xf / nb, yf / nb, tool_seed.wrapping_add(101)) * 1.35);
                    if vein[i] >= 0.0 && t >= 0.0 {
                        if t < ev {
                            c = edge;
                        } else if t < ev + rim_width {
                            c = rim;
                        } else {
                            let k = (value_noise(
                                xf / (2.6 * gsc),
                                yf / (2.6 * gsc),
                                tool_seed.wrapping_add(53),
                            ) - 0.5)
                                * 46.0
                                * grain;
                            c = core.map(|channel| channel + k);
                        }
                    }
                }
                if grain > 0.0 {
                    let draw = hash(gx, gy, tool_seed.wrapping_add(7919));
                    if draw > 1.0 - grain * 0.016 {
                        for ch in 0..3 {
                            c[ch] = (c[ch] + fleck[ch]) * 0.5;
                        }
                    } else if draw < grain * 0.010 {
                        c = c.map(|channel| channel * 0.55);
                    }
                }
                for (out, channel) in px.iter_mut().zip(c) {
                    *out = store(channel);
                }
                px[3] = 255;
            }
        }
    });
}
