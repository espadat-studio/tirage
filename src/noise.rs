use crate::ease::{lerp, smooth};
use crate::hash;

pub(crate) fn value_sum(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash::sum(xi.wrapping_add(dx), yi.wrapping_add(dy), seed);
    (corner(0, 0) * (1.0 - u) + corner(1, 0) * u) * (1.0 - v)
        + (corner(0, 1) * (1.0 - u) + corner(1, 1) * u) * v
}

pub(crate) fn value_lerp(x: f64, y: f64, seed: u32) -> f64 {
    value_lerp_zw(x, y, None, seed)
}

pub(crate) fn value_lerp_zw(x: f64, y: f64, zw: Option<(f64, f64)>, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let face = |z: i32, w: i32| {
        let corner =
            |dx: i32, dy: i32| hash::xor_w(xi.wrapping_add(dx), yi.wrapping_add(dy), z, w, seed);
        lerp(
            lerp(corner(0, 0), corner(1, 0), u),
            lerp(corner(0, 1), corner(1, 1), u),
            v,
        )
    };
    let Some((z, w)) = zw else {
        return face(0, 0);
    };
    let (z0, w0) = (z.floor(), w.floor());
    let (s, t) = (smooth(z - z0), smooth(w - w0));
    let (zi, wi) = (z0 as i32, w0 as i32);
    lerp(
        lerp(face(zi, wi), face(zi, wi.wrapping_add(1)), t),
        lerp(
            face(zi.wrapping_add(1), wi),
            face(zi.wrapping_add(1), wi.wrapping_add(1)),
            t,
        ),
        s,
    )
}

pub(crate) fn value_poly(x: f64, y: f64, channel: u32, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner =
        |dx: i32, dy: i32| hash::xor(xi.wrapping_add(dx), yi.wrapping_add(dy), channel, seed);
    let (a, b, c, d) = (corner(0, 0), corner(1, 0), corner(0, 1), corner(1, 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

#[derive(Clone, Copy)]
pub(crate) struct Fbm {
    pub(crate) stride: u32,
    pub(crate) lacunarity: f64,
    pub(crate) gain: f64,
}

impl Fbm {
    pub(crate) const STANDARD: Self = Self {
        stride: 131,
        lacunarity: 2.0,
        gain: 0.5,
    };
    pub(crate) const LERP: Self = Self {
        stride: 1319,
        lacunarity: 2.0,
        gain: 0.5,
    };
}

pub(crate) fn fbm(
    shape: Fbm,
    noise: impl Fn(f64, f64, u32) -> f64,
    x: f64,
    y: f64,
    seed: u32,
    octaves: u32,
) -> f64 {
    let (mut sum, mut weight, mut total, mut frequency) = (0.0, 0.5, 0.0, 1.0);
    for octave in 0..octaves {
        sum += weight
            * noise(
                x * frequency,
                y * frequency,
                seed.wrapping_add(octave * shape.stride),
            );
        total += weight;
        frequency *= shape.lacunarity;
        weight *= shape.gain;
    }
    sum / total
}

pub(crate) fn fbm_zw(x: f64, y: f64, zw: Option<(f64, f64)>, seed: u32, octaves: u32) -> f64 {
    fbm(
        Fbm::LERP,
        |x, y, seed| value_lerp_zw(x, y, zw, seed),
        x,
        y,
        seed,
        octaves,
    )
}
