use std::time::Instant;

fn main() {
    let mut r = Renderer::new();
    for f in 0..10 {
        r.frame(f);
    }
    let n = 120;
    let mut paths = Vec::with_capacity(n);
    let mut grain = Vec::with_capacity(n);
    let mut sum = 0u32;
    for f in 0..n as u32 {
        let t0 = Instant::now();
        r.paths(f);
        let t1 = Instant::now();
        r.grain(f);
        let t2 = Instant::now();
        sum = sum.wrapping_add(scene::checksum(unsafe { std::slice::from_raw_parts(r.ptr(), (scene::W * scene::H * 4) as usize) }));
        paths.push((t1 - t0).as_secs_f64() * 1e3);
        grain.push((t2 - t1).as_secs_f64() * 1e3);
    }
    let med = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let (p, g) = (med(&mut paths), med(&mut grain));
    println!("paths_ms={p:.2} grain_ms={g:.2} total_ms={:.2} checksum={sum}", p + g);
    if let Some(out) = std::env::args().nth(1) {
        r.frame(0);
        let px = unsafe { std::slice::from_raw_parts(r.ptr(), (scene::W * scene::H * 4) as usize) };
        std::fs::write(out, px).unwrap();
    }
}
