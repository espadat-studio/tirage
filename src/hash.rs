pub(crate) fn sum(x: i32, y: i32, seed: u32) -> f64 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(1_274_126_177));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}
