use tirage::{Frame, Palette, Params, Recipe, ReliefParams, Tool, ToolPin, derive, render};

const RELIEF: ToolPin = ToolPin::Tool(Tool::Relief);

fn relief(recipe: &Recipe) -> &ReliefParams {
    let Params::Relief(params) = recipe.params() else {
        panic!("not a relief Recipe");
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
            r##"{"tirage":0,"tool":"relief","tool_seed":9611518,"palette":["#1f1f1f","#f4d35e","#ee964b","#f95738","#0d3b66","#faf0ca"],"params":{"scale":120,"repeat":12,"variety":0.02,"solids":0.28,"relief":0.84,"accent":0.33,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"relief","tool_seed":3724278653,"palette":["#1f1f1f","#f4d35e","#ee964b","#f95738","#0d3b66","#faf0ca"],"params":{"scale":49,"repeat":7,"variety":0.48,"solids":0.23,"relief":0.73,"accent":0.86,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"relief","tool_seed":3565986084,"palette":["#1f1f1f","#f4d35e","#ee964b","#f95738","#0d3b66","#faf0ca"],"params":{"scale":114,"repeat":8,"variety":0.25,"solids":0.72,"relief":0.38,"accent":0.93,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, RELIEF).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, RELIEF);
        let p = relief(&recipe);
        assert!((12..=120).contains(&p.scale()), "seed {seed}");
        assert!((1..=12).contains(&p.repeat()), "seed {seed}");
        for (value, min, max) in [
            (p.variety(), 0.0, 1.0),
            (p.solids(), 0.0, 0.75),
            (p.relief(), 0.0, 1.0),
            (p.accent(), 0.0, 1.0),
        ] {
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
fn relief_is_a_still() {
    assert_eq!(Tool::Relief.frames(), 1);
    let recipe = derive(7, RELIEF);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, RELIEF);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = ReliefParams::default();
    assert_eq!(
        error(p.set_scale(181)),
        "relief: scale 181 is outside 12..=180"
    );
    assert_eq!(error(p.set_repeat(0)), "relief: repeat 0 is outside 1..=12");
    assert_eq!(p, ReliefParams::default());
    p.set_scale(180).unwrap();
    p.set_solids(1.0).unwrap();
    assert_eq!((p.scale(), p.solids()), (180, 1.0));
}

#[test]
fn a_flat_relief_paints_every_face_in_the_mid_tone() {
    let mut recipe = derive(7, RELIEF);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#ff0000"]).unwrap())
        .unwrap();
    let Params::Relief(p) = recipe.params_mut() else {
        unreachable!()
    };
    p.set_relief(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    assert!(image.rgba().chunks(4).all(|px| px == [255, 0, 0, 255]));
}
