use tirage::{
    Frame, Palette, Params, Recipe, Tool, ToolPin, WhorlParams, WhorlWarp, derive, render,
};

const WHORL: ToolPin = ToolPin::Tool(Tool::Whorl);

fn whorl(recipe: &Recipe) -> &WhorlParams {
    let Params::Whorl(params) = recipe.params() else {
        panic!("not a whorl Recipe");
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
            r##"{"tirage":0,"tool":"whorl","tool_seed":9611518,"palette":["#003049","#f77f00"],"params":{"centres":13,"pull":0.76,"push":0.21,"stripes":26,"weight":0.75,"dirs":"Smooth","warp":0.33,"detail":0.03,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"whorl","tool_seed":3724278653,"palette":["#003049","#f77f00"],"params":{"centres":2,"pull":0.63,"push":0.86,"stripes":23,"weight":0.45,"dirs":"Ripple","warp":0.65,"detail":0.84,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"whorl","tool_seed":3565986084,"palette":["#003049","#f77f00"],"params":{"centres":12,"pull":0.56,"push":0.05,"stripes":12,"weight":0.77,"dirs":"Ripple","warp":0.19,"detail":0.22,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, WHORL).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut warps = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, WHORL);
        let p = whorl(&recipe);
        assert!((1..=14).contains(&p.centres()), "seed {seed}");
        assert!((2..=30).contains(&p.stripes()), "seed {seed}");
        for (value, min, max) in [
            (p.pull(), 0.0, 1.0),
            (p.push(), 0.0, 1.0),
            (p.weight(), 0.2, 0.8),
            (p.warp(), 0.0, 1.0),
            (p.detail(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        if !warps.contains(&p.kind()) {
            warps.push(p.kind());
        }
    }
    assert_eq!(warps.len(), 3, "{warps:?}");
}

#[test]
fn whorl_is_a_still() {
    assert_eq!(Tool::Whorl.frames(), 1);
    let recipe = derive(7, WHORL);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, WHORL);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = WhorlParams::default();
    assert_eq!(
        error(p.set_stripes(91)),
        "whorl: stripes 91 is outside 2..=90"
    );
    assert_eq!(
        error(p.set_centres(0)),
        "whorl: centres 0 is outside 1..=14"
    );
    assert_eq!(p, WhorlParams::default());
    p.set_kind(WhorlWarp::Ripple);
    p.set_stripes(90).unwrap();
    assert_eq!((p.kind(), p.stripes()), (WhorlWarp::Ripple, 90));
}

#[test]
fn stripe_inks_cycle_after_the_ground() {
    let mut recipe = derive(7, WHORL);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    for ink in [[255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(
            image.rgba().chunks(4).any(|px| px[..3] == ink),
            "{ink:?} never drawn"
        );
    }
}
