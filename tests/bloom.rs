use tirage::{BloomParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const BLOOM: ToolPin = ToolPin::Tool(Tool::Bloom);

fn bloom(recipe: &Recipe) -> &BloomParams {
    let Params::Bloom(params) = recipe.params() else {
        panic!("not a bloom Recipe");
    };
    params
}

fn error<T: std::fmt::Debug>(result: Result<T, tirage::Error>) -> String {
    result.unwrap_err().to_string()
}

#[test]
fn derive_gives_golden_recipes() {
    let golden = [
        (
            1,
            r##"{"tirage":0,"tool":"bloom","tool_seed":9611518,"palette":["#1b2a49","#2e5c8a","#3f9bc4","#7fd1d8","#beefdc","#f3fbeb"],"params":{"cols":54,"rings":14,"warp":0.78,"grain":0.3,"steps":3,"calm":0.17,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"bloom","tool_seed":3724278653,"palette":["#1b2a49","#2e5c8a","#3f9bc4","#7fd1d8","#beefdc","#f3fbeb"],"params":{"cols":12,"rings":4,"warp":0.66,"grain":0.3,"steps":6,"calm":0.41,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"bloom","tool_seed":3565986084,"palette":["#1b2a49","#2e5c8a","#3f9bc4","#7fd1d8","#beefdc","#f3fbeb"],"params":{"cols":38,"rings":11,"warp":0.15,"grain":0.3,"steps":7,"calm":0.1,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, BLOOM).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, BLOOM);
        let p = bloom(&recipe);
        assert!((8..=96).contains(&p.cols()), "seed {seed}");
        assert!((4..=14).contains(&p.rings()), "seed {seed}");
        assert!((2..=8).contains(&p.steps()), "seed {seed}");
        assert_eq!(p.grain(), 0.3, "seed {seed}");
        for (value, min, max) in [(p.warp(), 0.0, 1.0), (p.calm(), 0.0, 0.5)] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn bloom_is_a_still() {
    assert_eq!(Tool::Bloom.frames(), 1);
    let recipe = derive(7, BLOOM);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, BLOOM);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = BloomParams::default();
    assert_eq!(error(p.set_cols(7)), "bloom: cols 7 is outside 8..=96");
    assert_eq!(error(p.set_steps(15)), "bloom: steps 15 is outside 2..=14");
    assert_eq!(p, BloomParams::default());
    p.set_rings(14).unwrap();
    p.set_calm(1.0).unwrap();
    assert_eq!((p.rings(), p.calm()), (14, 1.0));
}

#[test]
fn the_frame_is_mirrored_on_both_axes() {
    for seed in 0..5 {
        let recipe = derive(seed, BLOOM);
        let (width, height) = (90, 160);
        let image = render(&recipe, &Frame::new(&recipe, width, height, 0).unwrap());
        let px = |x: u32, y: u32| {
            let at = ((y * width + x) * 4) as usize;
            &image.rgba()[at..at + 4]
        };
        for y in 0..height {
            for x in 0..width {
                assert_eq!(px(x, y), px(width - 1 - x, y), "seed {seed} at {x},{y}");
                assert_eq!(px(x, y), px(x, height - 1 - y), "seed {seed} at {x},{y}");
            }
        }
    }
}

#[test]
fn the_ramp_reaches_every_ink() {
    let mut recipe = derive(7, BLOOM);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    for ink in [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(
            image.rgba().chunks(4).any(|px| px[..3] == ink),
            "{ink:?} never drawn"
        );
    }
}
