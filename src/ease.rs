use std::f64::consts::TAU;

pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub(crate) fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn circle(t: f64, amount: f64) -> (f64, f64) {
    ((TAU * t).cos() * amount, (TAU * t).sin() * amount)
}
