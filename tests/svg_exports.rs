mod fidelity;

use std::fs;
use std::path::Path;

use resvg::{tiny_skia, usvg};
use serde_json::Value;
use tirage::{Frame, Recipe, Tool, render, render_svg};

use fidelity::assert_svg_fidelity;

const MAX_LUMA: u8 = 1;
const MAX_BYTES: usize = 10_000_000;
const OVERRIDES: &[(Tool, u8)] = &[(Tool::Riso, 2), (Tool::Specimen, 3)];
const WIDTH: u32 = 540;
const HEIGHT: u32 = 960;

fn rasterize(svg: &str) -> image::RgbaImage {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(WIDTH, HEIGHT).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let c = pixel.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    image::RgbaImage::from_raw(WIDTH, HEIGHT, rgba).unwrap()
}

fn without_grain_or_dither(mut recipe: Value) -> Recipe {
    let params = recipe["params"].as_object_mut().unwrap();
    for toggle in ["grainTog", "ditherTog"] {
        if params.contains_key(toggle) {
            params.insert(toggle.to_owned(), Value::Bool(false));
        }
    }
    Recipe::from_json(&recipe.to_string()).unwrap()
}

#[test]
fn every_fixture_of_an_svg_tool_rasterizes_to_its_png() {
    let refs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/refs");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(refs.join("manifest.json")).unwrap()).unwrap();
    for &tool in Tool::ALL.iter().filter(|tool| tool.supports_svg()) {
        let bar = OVERRIDES
            .iter()
            .find(|(t, _)| *t == tool)
            .map_or(MAX_LUMA, |&(_, bar)| bar);
        let fixtures = manifest["tools"][tool.slug()]["fixtures"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: no fixtures in the manifest", tool.slug()));
        assert!(!fixtures.is_empty(), "{}: no fixtures", tool.slug());
        for fixture in fixtures {
            let name = format!("svg-{}-{}", tool.slug(), fixture["name"].as_str().unwrap());
            let recipe = without_grain_or_dither(fixture["recipe"].clone());
            let frame = Frame::still(WIDTH, HEIGHT).unwrap();
            let image = render(&recipe, &frame);
            let png = image::RgbaImage::from_raw(WIDTH, HEIGHT, image.rgba().to_vec()).unwrap();
            let svg = render_svg(&recipe, WIDTH, HEIGHT).unwrap();
            assert!(svg.len() <= MAX_BYTES, "{name}: {} bytes", svg.len());
            assert_svg_fidelity(&name, &png, &rasterize(&svg), bar);
        }
    }
}
