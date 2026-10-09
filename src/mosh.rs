use serde::{Deserialize, Serialize};

use crate::chassis::{self, Blend, Dither, DitherKind, Grain, hash, round_half_up};
use crate::param::Param;
use crate::surface::Surface;
use crate::taste::Taste;
use crate::{Error, Palette, Parameter};

pub(crate) const SLUG: &str = "mosh";
pub(crate) const FRAMES: u32 = 1;
pub(crate) const FPS: u32 = 1;
pub(crate) const MAX_INKS: Option<usize> = None;

const DEFAULT_PALETTE: [&str; 8] = [
    "#000000", "#ffffff", "#ff3131", "#31ff6b", "#3164ff", "#31f0ff", "#ff31e0", "#ffee31",
];

const BANDS: Param = Param {
    taste: (3, 14),
    ..Param::new(SLUG, "bands", 1, 14, 1)
};
const COLS: Param = Param {
    step: 2,
    ..Param::new(SLUG, "cols", 24, 420, 1)
};
const MIX: Param = Param::new(SLUG, "mix", 0, 100, 100);
const TEARS: Param = Param::new(SLUG, "tears", 0, 100, 100);
const RUNS: Param = Param::new(SLUG, "runs", 0, 100, 100);
const BRIGHT: Param = Param::new(SLUG, "bright", 0, 100, 100);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MoshParams {
    bands: u32,
    cols: u32,
    mix: f64,
    tears: f64,
    runs: f64,
    bright: f64,
    #[serde(flatten)]
    dither: Dither,
    #[serde(flatten)]
    chassis_grain: Grain,
}

impl Default for MoshParams {
    fn default() -> Self {
        Self {
            bands: 6,
            cols: 150,
            mix: 0.62,
            tears: 0.55,
            runs: 0.5,
            bright: 0.3,
            dither: Dither::default(),
            chassis_grain: Grain::default(),
        }
    }
}

impl MoshParams {
    pub fn bands(&self) -> u32 {
        self.bands
    }

    pub fn set_bands(&mut self, bands: u32) -> Result<(), Error> {
        BANDS.check(f64::from(bands))?;
        self.bands = bands;
        Ok(())
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn set_cols(&mut self, cols: u32) -> Result<(), Error> {
        COLS.check(f64::from(cols))?;
        self.cols = cols;
        Ok(())
    }

    pub fn mix(&self) -> f64 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f64) -> Result<(), Error> {
        self.mix = MIX.check(mix)?;
        Ok(())
    }

    pub fn tears(&self) -> f64 {
        self.tears
    }

    pub fn set_tears(&mut self, tears: f64) -> Result<(), Error> {
        self.tears = TEARS.check(tears)?;
        Ok(())
    }

    pub fn runs(&self) -> f64 {
        self.runs
    }

    pub fn set_runs(&mut self, runs: f64) -> Result<(), Error> {
        self.runs = RUNS.check(runs)?;
        Ok(())
    }

    pub fn bright(&self) -> f64 {
        self.bright
    }

    pub fn set_bright(&mut self, bright: f64) -> Result<(), Error> {
        self.bright = BRIGHT.check(bright)?;
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
    bands: u32,
    cols: u32,
    mix: f64,
    tears: f64,
    runs: f64,
    bright: f64,
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

pub(crate) fn from_json(params: serde_json::Value) -> Result<MoshParams, Error> {
    let raw = Unchecked::deserialize(params).map_err(|e| Error::Json(format!("params: {e}")))?;
    let mut params = MoshParams::default();
    params.set_bands(raw.bands)?;
    params.set_cols(raw.cols)?;
    params.set_mix(raw.mix)?;
    params.set_tears(raw.tears)?;
    params.set_runs(raw.runs)?;
    params.set_bright(raw.bright)?;
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

pub(crate) const PARAMS: &[Param] = &[BANDS, COLS, MIX, TEARS, RUNS, BRIGHT];

pub(crate) fn parameters() -> Vec<Parameter> {
    [
        BANDS.parameter(),
        COLS.parameter(),
        MIX.parameter(),
        TEARS.parameter(),
        RUNS.parameter(),
        BRIGHT.parameter(),
    ]
    .into_iter()
    .chain(chassis::parameters())
    .collect()
}

pub(crate) fn palette() -> Palette {
    Palette::from_hex(&DEFAULT_PALETTE).expect("the default Palette is valid")
}

pub(crate) fn deal(draw: impl Fn(&str) -> u64, taste: &Taste) -> MoshParams {
    let pick = |param: &Param| param.deal(taste.bounds(param), draw(param.id));
    MoshParams {
        bands: pick(&BANDS) as u32,
        cols: pick(&COLS) as u32,
        mix: pick(&MIX),
        tears: pick(&TEARS),
        runs: pick(&RUNS),
        bright: pick(&BRIGHT),
        ..MoshParams::default()
    }
}

pub(crate) fn render(
    surface: &mut Surface,
    params: &MoshParams,
    palette: &Palette,
    tool_seed: u32,
    _t: u32,
) {
    paint(surface, params, palette, tool_seed);
    chassis::finish(surface, &params.chassis_grain, &params.dither);
}

#[derive(Clone, Copy)]
enum Kind {
    Confetti,
    Mosaic,
    Smear,
    Scan,
    Chevron,
}

fn luma([r, g, b]: [u8; 3]) -> f64 {
    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114
}

struct Picker<'a> {
    inks: &'a [[u8; 3]],
    dark: [u8; 3],
    mix: f64,
}

impl Picker<'_> {
    fn pick(&self, v: f64, bias: u32) -> [u8; 3] {
        if v < 0.16 {
            return self.dark;
        }
        let len = self.inks.len();
        let n = round_half_up(len as f64 * (0.25 + self.mix * 0.75)).max(1.0);
        self.inks[((v * n) as usize + bias as usize) % len]
    }

    fn bright(&self) -> [u8; 3] {
        self.inks[1 % self.inks.len()]
    }
}

fn put(surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, ink: [u8; 3]) {
    if width > 0.0 && height > 0.0 {
        surface.fill_rect(x as i32, y as i32, width as u32, height as u32, ink);
    }
}

struct Sheet<'a> {
    surface: &'a mut Surface,
    params: &'a MoshParams,
    picker: Picker<'a>,
    columns: u32,
    cw: f64,
}

impl Sheet<'_> {
    fn run_rows(
        surface: &mut Surface,
        (columns, cw): (u32, f64),
        (top, bh, rows): (f64, f64, u32),
        run: impl Fn(u32, u32) -> (u32, [u8; 3]),
    ) {
        let rh = bh / f64::from(rows);
        let n = f64::from(columns);
        for j in 0..rows {
            let y = round_half_up(top + f64::from(j) * rh);
            let mut i = 0;
            while i < columns {
                let (length, ink) = run(i, j);
                let x0 = round_half_up(f64::from(i) * cw);
                let x1 = round_half_up(n.min(f64::from(i + length)) * cw);
                put(surface, x0, y, x1 - x0, rh.ceil() + 1.0, ink);
                i += length;
            }
        }
    }

    fn band(&mut self, top: f64, bh: f64, kind: Kind, b: u32, bs: u32) {
        let seed = |k: u32| bs.wrapping_add(k);
        let (w, cw, n) = (f64::from(self.surface.width()), self.cw, self.columns);
        match kind {
            Kind::Confetti => {
                let rows = round_half_up(bh / cw).max(1.0) as u32;
                let picker = &self.picker;
                Self::run_rows(self.surface, (n, cw), (top, bh, rows), |i, j| {
                    let (x, y) = (i as i32, j as i32);
                    let long = hash(x, y, seed(101));
                    let length = if long < 0.10 {
                        5 + (long * 260.0) as u32
                    } else {
                        1 + (hash(x, y, seed(17)) * 3.0) as u32
                    };
                    (length, picker.pick(hash(x, y, seed(19)), j))
                });
            }
            Kind::Mosaic => {
                let step = round_half_up(4.0 + hash(b as i32, 5, seed(23)) * 5.0).max(2.0) as u32;
                let cols = n.div_ceil(step);
                let rows = round_half_up(bh / (cw * f64::from(step))).max(1.0) as u32;
                let rh = bh / f64::from(rows);
                let count = if self.params.tears <= 0.0 {
                    0
                } else {
                    1 + (hash(b as i32, 9, seed(29)) * self.params.tears * 2.6) as i32
                };
                let tears: Vec<(f64, f64, bool)> = (0..count)
                    .map(|t| {
                        (
                            hash(t, 11, seed(31)) * f64::from(rows),
                            (hash(t, 13, seed(37)) * 2.0 - 1.0) * f64::from(rows) / f64::from(cols)
                                * 1.8,
                            hash(t, 15, seed(41)) < 0.5,
                        )
                    })
                    .collect();
                for j in 0..rows {
                    let row = f64::from(j);
                    for i in 0..cols {
                        let torn = !tears.is_empty()
                            && tears.iter().all(|&(offset, slope, up)| {
                                let edge = offset + slope * f64::from(i);
                                if up { row < edge } else { row > edge }
                            });
                        let v = hash((i * step) as i32, j as i32, seed(43));
                        let ink = if torn {
                            if v < 0.82 {
                                self.picker.dark
                            } else {
                                self.picker.pick(v, b)
                            }
                        } else {
                            self.picker.pick(v, b + j)
                        };
                        let x0 = round_half_up(f64::from(i * step) * cw);
                        let x1 = round_half_up(f64::from(n.min((i + 1) * step)) * cw);
                        put(
                            self.surface,
                            x0,
                            round_half_up(top + row * rh),
                            x1 - x0,
                            rh.ceil() + 1.0,
                            ink,
                        );
                    }
                }
            }
            Kind::Smear => {
                let rows = round_half_up(bh / (cw * 0.6).max(2.0)).max(2.0) as u32;
                let long = 6.0 + self.params.runs * 46.0;
                let bright = self.params.bright * 0.35;
                let picker = &self.picker;
                Self::run_rows(self.surface, (n, cw), (top, bh, rows), |i, j| {
                    let (x, y) = (i as i32, j as i32);
                    let length = round_half_up(long * (0.25 + hash(x, y * 3, seed(47)) * 1.5))
                        .max(1.0) as u32;
                    let ink = if hash(x, y, seed(59)) < bright {
                        picker.bright()
                    } else {
                        picker.pick(hash(x, y, seed(53)), j)
                    };
                    (length, ink)
                });
            }
            Kind::Scan => {
                let (mut yy, bottom) = (top, top + bh);
                let mut j = 0;
                while yy < bottom {
                    let row = j as i32;
                    let rh = round_half_up(cw * (0.4 + hash(0, row, seed(61)) * 3.2)).max(1.0);
                    let hh = rh.min(bottom - yy);
                    let ink = self.picker.pick(hash(1, row, seed(67)), j);
                    put(
                        self.surface,
                        0.0,
                        round_half_up(yy),
                        w,
                        hh.ceil() + 1.0,
                        ink,
                    );
                    if hash(2, row, seed(71)) < self.params.bright * 0.8 {
                        let width = w * (0.08 + hash(3, row, seed(73)) * 0.5);
                        let x0 = w * hash(4, row, seed(79)) * (1.0 - 0.08);
                        let bright = self.picker.bright();
                        put(
                            self.surface,
                            round_half_up(x0),
                            round_half_up(yy),
                            round_half_up(width.min(w - x0)),
                            hh.ceil() + 1.0,
                            bright,
                        );
                    }
                    yy += hh;
                    j += 1;
                }
            }
            Kind::Chevron => {
                let rows = round_half_up(bh / cw).max(1.0) as u32;
                let rh = bh / f64::from(rows);
                let per = round_half_up(4.0 + hash(b as i32, 17, seed(83)) * 10.0).max(3.0) as u32;
                let amp = round_half_up(2.0 + hash(b as i32, 19, seed(89)) * 6.0).max(1.0);
                for j in 0..rows {
                    let t = j % (per * 2);
                    let tri = if t < per { t } else { per * 2 - t };
                    let shift = round_half_up(f64::from(tri) / f64::from(per) * amp) as u32;
                    let y = round_half_up(top + f64::from(j) * rh);
                    for i in 0..n {
                        let ink = self
                            .picker
                            .pick(hash(((i + shift) % per) as i32, 0, seed(97)), b);
                        put(
                            self.surface,
                            round_half_up(f64::from(i) * cw),
                            y,
                            cw.ceil() + 1.0,
                            rh.ceil() + 1.0,
                            ink,
                        );
                    }
                }
            }
        }
    }
}

fn paint(surface: &mut Surface, params: &MoshParams, palette: &Palette, tool_seed: u32) {
    let inks = palette.inks();
    let dark = inks.iter().copied().fold(
        inks[0],
        |dark, ink| if luma(ink) < luma(dark) { ink } else { dark },
    );
    let (w, h) = (f64::from(surface.width()), f64::from(surface.height()));
    let columns = params.cols.max(6);
    let count = params.bands.max(1);
    let weights: Vec<f64> = (0..count)
        .map(|i| 0.4 + hash(i as i32, 3, tool_seed.wrapping_add(11)) * 1.6)
        .collect();
    let total: f64 = weights.iter().sum();
    let mut deck = [
        Kind::Confetti,
        Kind::Mosaic,
        Kind::Smear,
        Kind::Scan,
        Kind::Chevron,
    ];
    for i in (1..deck.len()).rev() {
        let j = (hash(i as i32, 7, tool_seed.wrapping_add(13)) * (i + 1) as f64) as usize;
        deck.swap(i, j);
    }
    let mut sheet = Sheet {
        surface,
        params,
        picker: Picker {
            inks,
            dark,
            mix: params.mix,
        },
        columns,
        cw: w / f64::from(columns),
    };
    let mut y = 0.0;
    for (b, weight) in (0..count).zip(&weights) {
        let bh = weight / total * h;
        let y0 = round_half_up(y);
        let y1 = if b == count - 1 {
            h
        } else {
            round_half_up(y + bh)
        };
        y += bh;
        let bs = tool_seed.wrapping_add(b.wrapping_mul(101));
        sheet.band(y0, y1 - y0, deck[b as usize % deck.len()], b, bs);
    }
}
