use std::fs;
use std::path::{Path, PathBuf};

use image::{GrayImage, Pixel, Rgb, RgbaImage};

const LUMA_CUTOFF: u8 = 48;

pub struct Diff {
    pub share: f64,
    pub heatmap: GrayImage,
}

pub fn luma_diff(ours: &RgbaImage, reference: &RgbaImage) -> Diff {
    assert_eq!(
        ours.dimensions(),
        reference.dimensions(),
        "ours and reference differ in size"
    );
    let (width, height) = ours.dimensions();
    let heatmap = GrayImage::from_fn(width, height, |x, y| {
        let (a, b) = (ours.get_pixel(x, y), reference.get_pixel(x, y));
        Rgb([
            a[0].abs_diff(b[0]),
            a[1].abs_diff(b[1]),
            a[2].abs_diff(b[2]),
        ])
        .to_luma()
    });
    let over = heatmap
        .pixels()
        .filter(|luma| luma[0] > LUMA_CUTOFF)
        .count();
    Diff {
        share: over as f64 / f64::from(width * height),
        heatmap,
    }
}

pub fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .parent()
        .unwrap()
        .join("fidelity")
}

pub fn assert_fidelity(name: &str, ours: &RgbaImage, reference: &RgbaImage, ceiling: f64) {
    let diff = luma_diff(ours, reference);
    if diff.share <= ceiling {
        return;
    }
    let dir = out_dir();
    fs::create_dir_all(&dir).unwrap();
    diff.heatmap
        .save(dir.join(format!("{name}-heatmap.png")))
        .unwrap();
    ours.save(dir.join(format!("{name}-ours.png"))).unwrap();
    panic!(
        "{name}: diff {:.2}% > ceiling {:.2}%",
        diff.share * 100.0,
        ceiling * 100.0
    );
}
