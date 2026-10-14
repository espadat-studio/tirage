pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub(crate) fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}
