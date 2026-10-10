use tirage::{Frame, Palette, Params, PithParams, Recipe, Tool, ToolPin, derive};

const PITH: ToolPin = ToolPin::Tool(Tool::Pith);

fn pith(recipe: &Recipe) -> &PithParams {
    let Params::Pith(params) = recipe.params() else {
        panic!("not a pith Recipe");
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
            r##"{"tirage":0,"tool":"pith","tool_seed":9611518,"palette":["#fbf8ef","#f26ca7","#7b2cbf","#2b1b4a","#141414"],"params":{"count":40,"size":0.45,"zoom":1.1,"round":0.7,"wobble":0.33,"band":0.99,"dither":0.55,"veins":0.82,"thick":0.25,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"pith","tool_seed":3724278653,"palette":["#fbf8ef","#f26ca7","#7b2cbf","#2b1b4a","#141414"],"params":{"count":57,"size":0.91,"zoom":2.6,"round":0.05,"wobble":0.31,"band":0.36,"dither":0.55,"veins":0.46,"thick":0.84,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"pith","tool_seed":3565986084,"palette":["#fbf8ef","#f26ca7","#7b2cbf","#2b1b4a","#141414"],"params":{"count":27,"size":0.67,"zoom":1.3,"round":0.59,"wobble":0.8,"band":0.45,"dither":0.55,"veins":0.43,"thick":0.45,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PITH).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let palette =
        Palette::from_hex(&["#fbf8ef", "#f26ca7", "#7b2cbf", "#2b1b4a", "#141414"]).unwrap();
    for seed in 0..2000 {
        let recipe = derive(seed, PITH);
        let p = pith(&recipe);
        assert_eq!(recipe.palette().unwrap(), &palette);
        assert_eq!(p.dither(), 0.55, "seed {seed}");
        assert_eq!(p.grain(), 0.4, "seed {seed}");
        assert!(
            (1..=60).contains(&p.count()),
            "seed {seed}: count {}",
            p.count()
        );
        let value = p.size();
        assert!((0.2..=1.0).contains(&value), "seed {seed}: size {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: size {value}"
        );
        let value = p.zoom();
        assert!((1.0..=3.0).contains(&value), "seed {seed}: zoom {value}");
        let ticks = (value - 1.0) / 0.1;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: zoom {value}"
        );
        let value = p.round();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: round {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: round {value}"
        );
        let value = p.wobble();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: wobble {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: wobble {value}"
        );
        let value = p.band();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: band {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: band {value}"
        );
        let value = p.veins();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: veins {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: veins {value}"
        );
        let value = p.thick();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: thick {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: thick {value}"
        );
    }
}

#[test]
fn pith_is_a_still() {
    assert_eq!(Tool::Pith.frames(), 1);
    let recipe = derive(7, PITH);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, PITH);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = PithParams::default();
    assert_eq!(error(p.set_count(0)), "pith: count 0 is outside 1..=60");
    assert_eq!(error(p.set_count(61)), "pith: count 61 is outside 1..=60");
    assert_eq!(error(p.set_size(1.5)), "pith: size 1.5 is outside 0..=1");
    assert_eq!(error(p.set_zoom(8.5)), "pith: zoom 8.5 is outside 1..=8");
    assert_eq!(error(p.set_round(1.5)), "pith: round 1.5 is outside 0..=1");
    assert_eq!(
        error(p.set_wobble(1.5)),
        "pith: wobble 1.5 is outside 0..=1"
    );
    assert_eq!(error(p.set_band(1.5)), "pith: band 1.5 is outside 0..=1");
    assert_eq!(
        error(p.set_dither(1.5)),
        "pith: dither 1.5 is outside 0..=1"
    );
    assert_eq!(error(p.set_veins(1.5)), "pith: veins 1.5 is outside 0..=1");
    assert_eq!(error(p.set_thick(1.5)), "pith: thick 1.5 is outside 0..=1");
    assert_eq!(error(p.set_grain(1.5)), "pith: grain 1.5 is outside 0..=1");
    assert_eq!(p, PithParams::default());
}
