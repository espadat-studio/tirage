use crate::ease::smooth;
use crate::hash;

pub(crate) fn value_sum(x: f64, y: f64, seed: u32) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (u, v) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| hash::sum(xi.wrapping_add(dx), yi.wrapping_add(dy), seed);
    (corner(0, 0) * (1.0 - u) + corner(1, 0) * u) * (1.0 - v)
        + (corner(0, 1) * (1.0 - u) + corner(1, 1) * u) * v
}
