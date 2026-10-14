pub(crate) fn sum(x: i32, y: i32, seed: u32) -> f64 {
    let mut n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(1_274_126_177));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

pub(crate) fn xor(x: i32, y: i32, channel: u32, seed: u32) -> f64 {
    xor_w(x, y, channel as i32, 0, seed)
}

pub(crate) fn xor_w(x: i32, y: i32, z: i32, w: i32, seed: u32) -> f64 {
    let mut n = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(1_440_662_683)
        ^ (w as u32).wrapping_mul(1_274_126_177)
        ^ seed.wrapping_mul(1_013_904_223);
    n = (n ^ (n >> 15)).wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(3_266_489_917);
    n ^= n >> 16;
    f64::from(n) / 4_294_967_296.0
}

pub(crate) fn sub_seed(seed: u32, k: u32) -> u32 {
    seed.wrapping_mul(7).wrapping_add(k)
}
