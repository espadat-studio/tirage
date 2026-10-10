use tirage::{Frame, Palette, Params, Recipe, StitchParams, Tool, ToolPin, derive};

const STITCH: ToolPin = ToolPin::Tool(Tool::Stitch);

fn stitch(recipe: &Recipe) -> &StitchParams {
    let Params::Stitch(params) = recipe.params() else {
        panic!("not a stitch Recipe");
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
            r##"{"tirage":0,"tool":"stitch","tool_seed":9611518,"palette":["#111111","#f6f23a","#f01fd0","#62f19e","#f5532b","#ded7eb"],"params":{"cols":67,"gutter":0.06,"streak":0.52,"scale":0.24,"slip":0.96,"band":12,"blobs":0.89,"blobsize":0.66,"steps":7,"ground":0.5,"stray":0.58,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"stitch","tool_seed":3724278653,"palette":["#111111","#f6f23a","#f01fd0","#62f19e","#f5532b","#ded7eb"],"params":{"cols":35,"gutter":0.15,"streak":0.99,"scale":0.17,"slip":0.15,"band":2,"blobs":0.39,"blobsize":0.94,"steps":5,"ground":0.25,"stray":0.17,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"stitch","tool_seed":3565986084,"palette":["#111111","#f6f23a","#f01fd0","#62f19e","#f5532b","#ded7eb"],"params":{"cols":40,"gutter":0.18,"streak":0.67,"scale":0.05,"slip":0.26,"band":12,"blobs":0.72,"blobsize":0.49,"steps":6,"ground":0.27,"stray":0.64,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, STITCH).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let palette = Palette::from_hex(&[
        "#111111", "#f6f23a", "#f01fd0", "#62f19e", "#f5532b", "#ded7eb",
    ])
    .unwrap();
    for seed in 0..2000 {
        let recipe = derive(seed, STITCH);
        let p = stitch(&recipe);
        assert_eq!(recipe.palette().unwrap(), &palette);
        assert!(
            (16..=96).contains(&p.cols()),
            "seed {seed}: cols {}",
            p.cols()
        );
        let value = p.gutter();
        assert!((0.0..=0.45).contains(&value), "seed {seed}: gutter {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: gutter {value}"
        );
        let value = p.streak();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: streak {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: streak {value}"
        );
        let value = p.scale();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: scale {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: scale {value}"
        );
        let value = p.slip();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: slip {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: slip {value}"
        );
        assert!(
            (1..=16).contains(&p.band()),
            "seed {seed}: band {}",
            p.band()
        );
        let value = p.blobs();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: blobs {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: blobs {value}"
        );
        let value = p.blobsize();
        assert!(
            (0.0..=1.0).contains(&value),
            "seed {seed}: blobsize {value}"
        );
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: blobsize {value}"
        );
        assert!(
            (2..=12).contains(&p.steps()),
            "seed {seed}: steps {}",
            p.steps()
        );
        let value = p.ground();
        assert!((0.0..=0.7).contains(&value), "seed {seed}: ground {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: ground {value}"
        );
        let value = p.stray();
        assert!((0.0..=1.0).contains(&value), "seed {seed}: stray {value}");
        let ticks = (value - 0.0) / 0.01;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: stray {value}"
        );
    }
}

#[test]
fn stitch_is_a_still() {
    assert_eq!(Tool::Stitch.frames(), 1);
    let recipe = derive(7, STITCH);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, STITCH);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = StitchParams::default();
    assert_eq!(error(p.set_cols(15)), "stitch: cols 15 is outside 16..=96");
    assert_eq!(error(p.set_cols(97)), "stitch: cols 97 is outside 16..=96");
    assert_eq!(
        error(p.set_gutter(1.5)),
        "stitch: gutter 1.5 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_streak(1.5)),
        "stitch: streak 1.5 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_scale(1.5)),
        "stitch: scale 1.5 is outside 0..=1"
    );
    assert_eq!(error(p.set_slip(1.5)), "stitch: slip 1.5 is outside 0..=1");
    assert_eq!(error(p.set_band(0)), "stitch: band 0 is outside 1..=16");
    assert_eq!(error(p.set_band(17)), "stitch: band 17 is outside 1..=16");
    assert_eq!(
        error(p.set_blobs(1.5)),
        "stitch: blobs 1.5 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_blobsize(1.5)),
        "stitch: blobsize 1.5 is outside 0..=1"
    );
    assert_eq!(error(p.set_steps(1)), "stitch: steps 1 is outside 2..=12");
    assert_eq!(error(p.set_steps(13)), "stitch: steps 13 is outside 2..=12");
    assert_eq!(
        error(p.set_ground(1.5)),
        "stitch: ground 1.5 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_stray(1.5)),
        "stitch: stray 1.5 is outside 0..=1"
    );
    assert_eq!(p, StitchParams::default());
}
