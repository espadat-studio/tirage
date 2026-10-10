use tirage::{Frame, MoshParams, Palette, Params, Recipe, Tool, ToolPin, derive};

const MOSH: ToolPin = ToolPin::Tool(Tool::Mosh);

fn mosh(recipe: &Recipe) -> &MoshParams {
    let Params::Mosh(params) = recipe.params() else {
        panic!("not a mosh Recipe");
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
            r##"{"tirage":0,"tool":"mosh","tool_seed":9611518,"palette":["#000000","#ffffff","#ff3131","#31ff6b","#3164ff","#31f0ff","#ff31e0","#ffee31"],"params":{"bands":3,"cols":284,"mix":0.01,"tears":0.1,"runs":0.74,"bright":0.86,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"mosh","tool_seed":3724278653,"palette":["#000000","#ffffff","#ff3131","#31ff6b","#3164ff","#31f0ff","#ff31e0","#ffee31"],"params":{"bands":9,"cols":144,"mix":0.84,"tears":0.89,"runs":0.68,"bright":0.33,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"mosh","tool_seed":3565986084,"palette":["#000000","#ffffff","#ff3131","#31ff6b","#3164ff","#31f0ff","#ff31e0","#ffee31"],"params":{"bands":12,"cols":286,"mix":0.72,"tears":0.86,"runs":0.27,"bright":0.01,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, MOSH).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let palette = Palette::from_hex(&[
        "#000000", "#ffffff", "#ff3131", "#31ff6b", "#3164ff", "#31f0ff", "#ff31e0", "#ffee31",
    ])
    .unwrap();
    for seed in 0..2000 {
        let recipe = derive(seed, MOSH);
        let p = mosh(&recipe);
        assert_eq!(recipe.palette().unwrap(), &palette);
        assert!(
            (3..=14).contains(&p.bands()),
            "seed {seed}: bands {}",
            p.bands()
        );
        assert!(
            (24..=420).contains(&p.cols()),
            "seed {seed}: cols {}",
            p.cols()
        );
        let value = p.mix();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: mix {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: mix {value}"
        );
        let value = p.tears();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: tears {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: tears {value}"
        );
        let value = p.runs();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: runs {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: runs {value}"
        );
        let value = p.bright();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: bright {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: bright {value}"
        );
    }
}

#[test]
fn mosh_is_a_still() {
    assert_eq!(Tool::Mosh.frames(), 1);
    let recipe = derive(7, MOSH);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, MOSH);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = MoshParams::default();
    assert_eq!(error(p.set_bands(0)), "mosh: bands 0 is outside 1..=14");
    assert_eq!(error(p.set_bands(15)), "mosh: bands 15 is outside 1..=14");
    assert_eq!(error(p.set_cols(23)), "mosh: cols 23 is outside 24..=420");
    assert_eq!(error(p.set_cols(421)), "mosh: cols 421 is outside 24..=420");
    assert_eq!(error(p.set_mix(1.5)), "mosh: mix 1.5 is outside 0..=1");
    assert_eq!(error(p.set_tears(1.5)), "mosh: tears 1.5 is outside 0..=1");
    assert_eq!(error(p.set_runs(1.5)), "mosh: runs 1.5 is outside 0..=1");
    assert_eq!(
        error(p.set_bright(1.5)),
        "mosh: bright 1.5 is outside 0..=1"
    );
    assert_eq!(p, MoshParams::default());
}
