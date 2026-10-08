pub const W: u32 = 1080;
pub const H: u32 = 1920;
pub const N_PATHS: usize = 200;

#[derive(Clone, Copy)]
pub enum Cmd {
    M(f32, f32),
    L(f32, f32),
    C(f32, f32, f32, f32, f32, f32),
    Z,
}

pub struct Shape {
    pub cmds: Vec<Cmd>,
    pub rgb: [u8; 3],
    pub alpha: f32,
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / 16_777_216.0
    }
}

fn arc(cmds: &mut Vec<Cmd>, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, start: bool) {
    let segs = ((a1 - a0).abs() / core::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let step = (a1 - a0) / segs as f32;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let (mut x, mut y) = (cx + r * a0.cos(), cy + r * a0.sin());
    cmds.push(if start { Cmd::M(x, y) } else { Cmd::L(x, y) });
    for i in 0..segs {
        let a = a0 + step * i as f32;
        let b = a + step;
        let (nx, ny) = (cx + r * b.cos(), cy + r * b.sin());
        cmds.push(Cmd::C(
            x - k * r * a.sin(),
            y + k * r * a.cos(),
            nx + k * r * b.sin(),
            ny - k * r * b.cos(),
            nx,
            ny,
        ));
        x = nx;
        y = ny;
    }
}

pub fn build(seed: u32, t: f32) -> Vec<Shape> {
    let mut rng = Rng(seed | 1);
    let tau = core::f32::consts::TAU;
    (0..N_PATHS)
        .map(|i| {
            let cx = rng.next() * W as f32 + (t * tau + i as f32).sin() * 40.0;
            let cy = rng.next() * H as f32 + (t * tau + i as f32).cos() * 40.0;
            let r = 20.0 + rng.next() * 180.0;
            let mut cmds = Vec::new();
            match i % 3 {
                0 => arc(&mut cmds, cx, cy, r, 0.0, tau, true),
                1 => {
                    let a0 = rng.next() * tau;
                    cmds.push(Cmd::M(cx, cy));
                    arc(&mut cmds, cx, cy, r, a0, a0 + 0.5 + rng.next() * 4.0, false);
                }
                _ => {
                    cmds.push(Cmd::M(cx - r, cy));
                    for _ in 0..4 {
                        cmds.push(Cmd::C(
                            cx + (rng.next() - 0.5) * 3.0 * r,
                            cy + (rng.next() - 0.5) * 3.0 * r,
                            cx + (rng.next() - 0.5) * 3.0 * r,
                            cy + (rng.next() - 0.5) * 3.0 * r,
                            cx + (rng.next() - 0.5) * 2.0 * r,
                            cy + (rng.next() - 0.5) * 2.0 * r,
                        ));
                    }
                    cmds.push(Cmd::L(cx, cy + r));
                }
            }
            cmds.push(Cmd::Z);
            let rgb = [(rng.next() * 255.0) as u8, (rng.next() * 255.0) as u8, (rng.next() * 255.0) as u8];
            Shape { cmds, rgb, alpha: 0.25 + rng.next() * 0.6 }
        })
        .collect()
}

pub fn grain(px: &mut [u8], frame: u32, amount: i32) {
    let mut s = frame.wrapping_mul(0x9E37_79B9) | 1;
    for p in px.chunks_exact_mut(4) {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let n = (((s & 0xff) as i32 - 128) * amount) >> 7;
        let a = p[3] as i32;
        p[0] = (p[0] as i32 + n).clamp(0, a) as u8;
        p[1] = (p[1] as i32 + n).clamp(0, a) as u8;
        p[2] = (p[2] as i32 + n).clamp(0, a) as u8;
    }
}

pub fn checksum(px: &[u8]) -> u32 {
    px.iter().step_by(97).fold(0u32, |h, &b| h.wrapping_mul(31).wrapping_add(b as u32))
}
