use tirage::{Frame, HissParams, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const HISS: ToolPin = ToolPin::Tool(Tool::Hiss);

fn hiss(recipe: &Recipe) -> &HissParams {
    let Params::Hiss(params) = recipe.params() else {
        panic!("not a hiss Recipe");
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
            r##"{"tirage":0,"tool":"hiss","tool_seed":9611518,"palette":["#ff1e8c","#ff7a1a","#2b4bff","#ffd400","#e8262b"],"params":{"leaves":1,"size":0.75,"overlap":0.21,"tilt":-0.1,"shift":0.41,"swirl":0.68,"black":0.94,"scale":0.82,"checkers":2,"steps":0.02,"comb":0.6,"marks":18,"flecks":0.54,"coarse":0.75,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"hiss","tool_seed":3724278653,"palette":["#ff1e8c","#ff7a1a","#2b4bff","#ffd400","#e8262b"],"params":{"leaves":2,"size":0.49,"overlap":0.46,"tilt":-0.7,"shift":0.1,"swirl":0.7,"black":0.97,"scale":0.78,"checkers":3,"steps":0.74,"comb":0.53,"marks":24,"flecks":0.26,"coarse":0.33,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"hiss","tool_seed":3565986084,"palette":["#ff1e8c","#ff7a1a","#2b4bff","#ffd400","#e8262b"],"params":{"leaves":2,"size":1.02,"overlap":0.42,"tilt":-0.53,"shift":0.92,"swirl":0.92,"black":0.29,"scale":0.43,"checkers":4,"steps":0.17,"comb":0.45,"marks":19,"flecks":0.96,"coarse":0.72,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, HISS).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, HISS);
        let p = hiss(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.leaves()), 1.0, 5.0, 1.0),
            (p.size(), 0.4, 1.4, 0.01),
            (p.overlap(), 0.0, 0.5, 0.01),
            (p.tilt(), -1.0, 1.0, 0.01),
            (p.shift(), 0.0, 1.0, 0.01),
            (p.swirl(), 0.0, 1.0, 0.01),
            (p.black(), 0.0, 1.0, 0.01),
            (p.scale(), 0.0, 1.0, 0.01),
            (f64::from(p.checkers()), 0.0, 4.0, 1.0),
            (p.steps(), 0.0, 1.0, 0.01),
            (p.comb(), 0.0, 1.0, 0.01),
            (f64::from(p.marks()), 0.0, 24.0, 1.0),
            (p.flecks(), 0.0, 1.0, 0.01),
            (p.coarse(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-6, "seed {seed}: {value}");
        }
    }
}

#[test]
fn hiss_is_a_still() {
    assert_eq!(Tool::Hiss.frames(), 1);
    let recipe = derive(7, HISS);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, HISS);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = HissParams::default();
    assert_eq!(error(p.set_leaves(0)), "hiss: leaves 0 is outside 1..=5");
    assert_eq!(
        error(p.set_tilt(-1.01)),
        "hiss: tilt -1.01 is outside -1..=1"
    );
    assert_eq!(error(p.set_marks(25)), "hiss: marks 25 is outside 0..=24");
    assert_eq!(p, HissParams::default());
    p.set_checkers(0).unwrap();
    assert_eq!(p.checkers(), 0);
}

#[test]
fn marble_takes_the_palette_and_snow_and_marks_their_own_inks() {
    let mut recipe = derive(7, HISS);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00"]).unwrap())
        .unwrap();
    let Params::Hiss(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_black(0.0).unwrap();
    params.set_comb(0.0).unwrap();
    params.set_flecks(0.0).unwrap();
    params.set_marks(24).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let has = |test: &dyn Fn(&[u8]) -> bool| image.rgba().chunks(4).any(test);
    assert!(
        has(&|px| px[0] > 100 && px[1] < 10 && px[2] == 0),
        "red marble"
    );
    assert!(
        has(&|px| px[1] > 100 && px[0] < 10 && px[2] == 0),
        "green marble"
    );
    assert!(has(&|px| px[..3] == [0xf4, 0xf4, 0xf2]), "white marks");
    assert!(has(&|px| px[..3] == [245, 245, 245]), "checker lenses");
}
