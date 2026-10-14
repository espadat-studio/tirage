pub(crate) fn luma_601([r, g, b]: [u8; 3]) -> f64 {
    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114
}

pub(crate) fn luma_709([r, g, b]: [u8; 3]) -> f64 {
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}
