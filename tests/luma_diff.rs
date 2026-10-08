mod fidelity;

use std::{fs, panic};

use image::{Rgba, RgbaImage};

use fidelity::{assert_fidelity, luma_diff, out_dir};

const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

fn black() -> RgbaImage {
    RgbaImage::from_pixel(100, 100, BLACK)
}

fn with_region(grey: u8, width: u32, height: u32) -> RgbaImage {
    let mut image = black();
    for y in 0..height {
        for x in 0..width {
            image.put_pixel(x, y, Rgba([grey, grey, grey, 255]));
        }
    }
    image
}

#[test]
fn identical_images_differ_by_zero() {
    assert_eq!(luma_diff(&black(), &black()).share, 0.0);
}

#[test]
fn changed_region_gives_its_share_of_the_frame() {
    assert_eq!(luma_diff(&with_region(255, 10, 20), &black()).share, 0.02);
}

#[test]
fn change_at_the_luma_cutoff_is_ignored() {
    assert_eq!(luma_diff(&with_region(48, 100, 100), &black()).share, 0.0);
}

#[test]
fn change_just_above_the_luma_cutoff_counts() {
    assert_eq!(luma_diff(&with_region(49, 50, 100), &black()).share, 0.5);
}

#[test]
fn diff_ignores_alpha() {
    let transparent = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 0]));
    assert_eq!(luma_diff(&transparent, &black()).share, 0.0);
}

#[test]
fn failing_fidelity_writes_heatmap_and_our_render() {
    let name = "luma-diff-self-test";
    for suffix in ["heatmap", "ours"] {
        let _ = fs::remove_file(out_dir().join(format!("{name}-{suffix}.png")));
    }
    let result =
        panic::catch_unwind(|| assert_fidelity(name, &with_region(255, 10, 20), &black(), 0.016));
    let message = *result.unwrap_err().downcast::<String>().unwrap();
    assert_eq!(message, "luma-diff-self-test: diff 2.00% > ceiling 1.60%");
    assert!(out_dir().join(format!("{name}-heatmap.png")).is_file());
    assert!(out_dir().join(format!("{name}-ours.png")).is_file());
}

#[test]
fn passing_fidelity_writes_nothing() {
    let name = "luma-diff-self-test-pass";
    assert_fidelity(name, &with_region(255, 10, 10), &black(), 0.016);
    assert!(!out_dir().join(format!("{name}-heatmap.png")).exists());
    assert!(!out_dir().join(format!("{name}-ours.png")).exists());
}
