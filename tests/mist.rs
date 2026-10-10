use tirage::{Frame, MistParams, Palette, Params, Recipe, Tool, ToolPin, derive};

const MIST: ToolPin = ToolPin::Tool(Tool::Mist);

fn mist(recipe: &Recipe) -> &MistParams {
    let Params::Mist(params) = recipe.params() else {
        panic!("not a mist Recipe");
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
            r##"{"tirage":0,"tool":"mist","tool_seed":9611518,"palette":["#dcff3a","#1e1b3a","#3b2a6e","#5a2e7a"],"params":{"streaks":1.45,"cover":0.45,"soft":0.38,"blot":0.28,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"mist","tool_seed":3724278653,"palette":["#dcff3a","#1e1b3a","#3b2a6e","#5a2e7a"],"params":{"streaks":1.2,"cover":0.5,"soft":0.85,"blot":0.69,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"mist","tool_seed":3565986084,"palette":["#dcff3a","#1e1b3a","#3b2a6e","#5a2e7a"],"params":{"streaks":0.55,"cover":0.58,"soft":0.81,"blot":0.73,"grain":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, MIST).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, MIST);
        let p = mist(&recipe);
        assert_eq!(p.grain(), 0.4, "seed {seed}");
        for (value, min, max, unit) in [
            (p.streaks(), 0.4, 2.0, 20.0),
            (p.cover(), 0.3, 0.6, 100.0),
            (p.soft(), 0.1, 1.0, 100.0),
            (p.blot(), 0.2, 1.0, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
    }
}

#[test]
fn mist_is_a_still() {
    assert_eq!(Tool::Mist.frames(), 1);
    let recipe = derive(7, MIST);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, MIST);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = MistParams::default();
    assert_eq!(
        error(p.set_streaks(0.3)),
        "mist: streaks 0.3 is outside 0.4..=2"
    );
    assert_eq!(
        error(p.set_cover(0.9)),
        "mist: cover 0.9 is outside 0.2..=0.85"
    );
    assert_eq!(
        error(p.set_grain(f64::NAN)),
        "mist: grain NaN is outside 0..=1"
    );
    assert_eq!(p, MistParams::default());
    p.set_cover(0.85).unwrap();
    assert_eq!(p.cover(), 0.85);
}

#[test]
fn a_palette_over_four_inks_is_refused() {
    let five = Palette::from_hex(&["#ff8fcb", "#b79cff", "#ffe066", "#6ee7c8", "#7cc6ff"]).unwrap();
    let mut recipe = derive(7, MIST);
    assert_eq!(
        error(recipe.set_palette(five)),
        "mist: palette has 5 inks, mist draws at most 4"
    );
    assert_eq!(recipe, derive(7, MIST));
}
