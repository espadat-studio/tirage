use tirage::{Frame, Palette, Params, Recipe, StaticParams, Tool, ToolPin, derive, render};

const STATIC: ToolPin = ToolPin::Tool(Tool::Static);

fn static_params(recipe: &Recipe) -> &StaticParams {
    let Params::Static(params) = recipe.params() else {
        panic!("not a static Recipe");
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
            r##"{"tirage":0,"tool":"static","tool_seed":9611518,"palette":["#ff3cac","#1b1b2f"],"params":{"regions":4,"res":160,"glitch":0.56,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"static","tool_seed":3724278653,"palette":["#ff3cac","#1b1b2f"],"params":{"regions":4,"res":148,"glitch":0.36,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"static","tool_seed":3565986084,"palette":["#ff3cac","#1b1b2f"],"params":{"regions":3,"res":140,"glitch":0.39,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, STATIC).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, STATIC);
        let p = static_params(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.regions()), 2.0, 5.0, 1.0),
            (f64::from(p.res()), 48.0, 160.0, 4.0),
            (p.glitch(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-6, "seed {seed}: {value}");
        }
    }
}

#[test]
fn static_is_a_still() {
    assert_eq!(Tool::Static.frames(), 1);
    let recipe = derive(7, STATIC);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, STATIC);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = StaticParams::default();
    assert_eq!(
        error(p.set_regions(6)),
        "static: regions 6 is outside 2..=5"
    );
    assert_eq!(error(p.set_res(44)), "static: res 44 is outside 48..=160");
    assert_eq!(
        error(p.set_glitch(1.01)),
        "static: glitch 1.01 is outside 0..=1"
    );
    assert_eq!(p, StaticParams::default());
    p.set_glitch(0.0).unwrap();
    assert_eq!(p.glitch(), 0.0);
}

#[test]
fn the_palette_is_an_ink_and_a_ground_and_nothing_more() {
    let mut recipe = derive(7, STATIC);
    let three = Palette::from_hex(&["#000000", "#ffffff", "#ff0000"]).unwrap();
    assert!(recipe.set_palette(three).is_err());
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#0000ff"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    let reds = image
        .rgba()
        .chunks(4)
        .filter(|px| px[..3] == [255, 0, 0])
        .count();
    let blues = image
        .rgba()
        .chunks(4)
        .filter(|px| px[..3] == [0, 0, 255])
        .count();
    assert_eq!(reds + blues, 90 * 160);
    assert!(reds > 0 && blues > 0, "{reds} {blues}");
}

#[test]
fn glitch_changes_the_bits() {
    let mut recipe = derive(7, STATIC);
    let frame = Frame::new(&recipe, 90, 160, 0).unwrap();
    let mut with = |glitch: f64| {
        let Params::Static(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_glitch(glitch).unwrap();
        render(&recipe, &frame)
    };
    assert_ne!(with(0.0), with(1.0));
}
