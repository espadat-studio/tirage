mod fidelity;

use std::fs;
use std::path::Path;

use serde_json::Value;
use tirage::{Frame, Recipe, render};

use fidelity::assert_fidelity;

const CEILING: f64 = 0.016;
const WIDTH: u32 = 540;
const HEIGHT: u32 = 960;

#[test]
fn every_fixture_is_within_its_threshold() {
    let refs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/refs");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(refs.join("manifest.json")).unwrap()).unwrap();
    let mut checked = 0;
    for (slug, entry) in manifest["tools"].as_object().unwrap() {
        let ceiling = entry["threshold"]["max"].as_f64().unwrap_or(CEILING);
        for fixture in entry["fixtures"].as_array().unwrap() {
            let name = format!("{slug}-{}", fixture["name"].as_str().unwrap());
            let recipe = Recipe::from_json(&fixture["recipe"].to_string()).unwrap();
            let t = fixture["frame"].as_u64().unwrap_or(0) as u32;
            let frame = Frame::new(&recipe, WIDTH, HEIGHT, t).unwrap();
            let image = render(&recipe, &frame);
            let ours = image::RgbaImage::from_raw(WIDTH, HEIGHT, image.rgba().to_vec()).unwrap();
            let reference = image::open(
                refs.join(slug)
                    .join(format!("{}.png", fixture["name"].as_str().unwrap())),
            )
            .unwrap()
            .to_rgba8();
            assert_fidelity(&name, &ours, &reference, ceiling);
            checked += 1;
        }
    }
    assert!(checked > 0, "manifest has no fixtures");
}
