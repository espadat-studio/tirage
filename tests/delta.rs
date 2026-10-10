use tirage::{DeltaParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const DELTA: ToolPin = ToolPin::Tool(Tool::Delta);

fn delta(recipe: &Recipe) -> &DeltaParams {
    let Params::Delta(params) = recipe.params() else {
        panic!("not a delta Recipe");
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
            r##"{"tirage":0,"tool":"delta","tool_seed":9611518,"palette":["#ff6f00","#0057ff","#ffea00","#00d68f","#ff2e88","#2a2a2a"],"params":{"scale":2.5,"warp":0.65,"contrast":0.25,"bleach":0.56,"patches":13,"cell":77,"fill":0.99,"quant":0.77,"blocks":10,"bands":3,"slice":0.32,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"delta","tool_seed":3724278653,"palette":["#ff6f00","#0057ff","#ffea00","#00d68f","#ff2e88","#2a2a2a"],"params":{"scale":1.1,"warp":1.18,"contrast":0.72,"bleach":0.05,"patches":12,"cell":67,"fill":0.37,"quant":0.92,"blocks":3,"bands":4,"slice":0.52,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"delta","tool_seed":3565986084,"palette":["#ff6f00","#0057ff","#ffea00","#00d68f","#ff2e88","#2a2a2a"],"params":{"scale":7.6,"warp":0.32,"contrast":0.13,"bleach":0.35,"patches":18,"cell":14,"fill":0.16,"quant":0.66,"blocks":8,"bands":2,"slice":0.25,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, DELTA).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_the_site_range_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, DELTA);
        let p = delta(&recipe);
        assert!((1.0..=9.0).contains(&p.scale()), "seed {seed}");
        assert_eq!(p.scale(), (p.scale() * 10.0).round() / 10.0, "seed {seed}");
        assert!((0.0..=1.6).contains(&p.warp()), "seed {seed}");
        assert!((0.05..=1.0).contains(&p.fill()), "seed {seed}");
        for value in [p.contrast(), p.bleach(), p.quant(), p.slice()] {
            assert!((0.0..=1.0).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(
            p.patches() <= 18 && p.blocks() <= 10 && p.bands() <= 6,
            "seed {seed}"
        );
        assert!((8..=90).contains(&p.cell()), "seed {seed}");
        assert!(!p.grain().on() && !p.dither().on(), "seed {seed}");
    }
}

#[test]
fn delta_is_a_still() {
    assert_eq!(Tool::Delta.frames(), 1);
    let recipe = derive(7, DELTA);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, DELTA);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = DeltaParams::default();
    assert_eq!(error(p.set_scale(0.9)), "delta: scale 0.9 is outside 1..=9");
    assert_eq!(error(p.set_cell(7)), "delta: cell 7 is outside 8..=90");
    assert_eq!(
        error(p.set_warp(1.61)),
        "delta: warp 1.61 is outside 0..=1.6"
    );
    assert_eq!(p, DeltaParams::default());
    p.set_bands(6).unwrap();
    p.set_fill(0.05).unwrap();
    assert_eq!((p.bands(), p.fill()), (6, 0.05));
}

#[test]
fn with_no_marks_the_frame_is_the_field_ramp_between_silt_and_shore() {
    let mut recipe = derive(7, DELTA);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ffffff"]).unwrap())
        .unwrap();
    let Params::Delta(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_quant(0.0).unwrap();
    params.set_patches(0).unwrap();
    params.set_blocks(0).unwrap();
    params.set_bands(0).unwrap();
    params.set_slice(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    for px in image.rgba().chunks(4) {
        assert!(px[0] >= 38 && px[0] <= 236, "{px:?}");
        assert_eq!(px[3], 255);
    }
}
