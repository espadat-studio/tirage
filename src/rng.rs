pub(crate) struct XorShift(u32);

impl XorShift {
    pub(crate) fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    pub(crate) fn knuth(seed: u32) -> Self {
        Self::new(seed.wrapping_mul(2_654_435_761))
    }

    pub(crate) fn murmur(seed: u32) -> Self {
        Self::new(seed.wrapping_mul(2_246_822_519) ^ 0x27d4_eb2f)
    }

    pub(crate) fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        f64::from(self.0) / 4_294_967_296.0
    }
}

pub(crate) struct XorShiftArithmetic(u32);

impl XorShiftArithmetic {
    pub(crate) fn from_seed(tool_seed: u32) -> Self {
        let start = (f64::from(tool_seed) * 2_654_435_761.0).rem_euclid(4_294_967_296.0) as u32;
        Self(start.max(1))
    }

    pub(crate) fn from_state(state: u32) -> Self {
        Self(state.max(1))
    }

    pub(crate) fn next(&mut self) -> f64 {
        let mut a = self.0;
        a ^= a << 13;
        a ^= ((a as i32) >> 17) as u32;
        a ^= a << 5;
        self.0 = a;
        f64::from(a) / 4_294_967_296.0
    }
}
