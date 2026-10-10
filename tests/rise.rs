use tirage::{
    Frame, Palette, Params, Recipe, RiseAnchor, RiseParams, Tool, ToolPin, derive, render,
};

const RISE: ToolPin = ToolPin::Tool(Tool::Rise);

fn rise(recipe: &Recipe) -> &RiseParams {
    let Params::Rise(params) = recipe.params() else {
        panic!("not a rise Recipe");
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
            r##"{"tirage":0,"tool":"rise","tool_seed":9611518,"palette":["#ff9f1c","#2ec4b6","#011627"],"params":{"styles":"Left","count":17,"weight":0.55,"bands":4,"depth":0.34,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"rise","tool_seed":3724278653,"palette":["#ff9f1c","#2ec4b6","#011627"],"params":{"styles":"Right","count":8,"weight":0.5,"bands":3,"depth":0.2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"rise","tool_seed":3565986084,"palette":["#ff9f1c","#2ec4b6","#011627"],"params":{"styles":"Left","count":13,"weight":0.3,"bands":5,"depth":0.35,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, RISE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, RISE);
        let p = rise(&recipe);
        assert!((6..=28).contains(&p.count()), "seed {seed}");
        assert!((1..=8).contains(&p.bands()), "seed {seed}");
        for (value, min, max) in [(p.weight(), 0.3, 0.7), (p.depth(), 0.1, 0.4)] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(!p.grain().on(), "seed {seed}");
    }
}

#[test]
fn rise_is_a_still() {
    assert_eq!(Tool::Rise.frames(), 1);
    let recipe = derive(7, RISE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, RISE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = RiseParams::default();
    assert_eq!(error(p.set_count(5)), "rise: count 5 is outside 6..=28");
    assert_eq!(
        error(p.set_weight(0.71)),
        "rise: weight 0.71 is outside 0.3..=0.7"
    );
    assert_eq!(error(p.set_bands(0)), "rise: bands 0 is outside 1..=8");
    assert_eq!(
        error(p.set_depth(0.05)),
        "rise: depth 0.05 is outside 0.1..=0.4"
    );
    assert_eq!(p, RiseParams::default());
    p.set_count(28).unwrap();
    p.set_depth(0.4).unwrap();
    assert_eq!((p.count(), p.depth()), (28, 0.4));
}

#[test]
fn a_palette_over_three_inks_is_refused() {
    let four = Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap();
    let mut recipe = derive(7, RISE);
    assert_eq!(
        error(recipe.set_palette(four)),
        "rise: palette has 4 inks, rise draws at most 3"
    );
    assert_eq!(recipe, derive(7, RISE));
}

fn ink_at(anchor: RiseAnchor, x: u32, y: u32) -> [u8; 3] {
    let mut recipe = derive(7, RISE);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let Params::Rise(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_anchor(anchor);
    params.set_bands(1).unwrap();
    params.set_depth(0.1).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    let i = ((y * 90 + x) * 4) as usize;
    [image.rgba()[i], image.rgba()[i + 1], image.rgba()[i + 2]]
}

#[test]
fn the_band_clips_to_a_disc_at_its_anchor() {
    assert_eq!(ink_at(RiseAnchor::Bottom, 45, 158), [0, 0, 255]);
    assert_eq!(ink_at(RiseAnchor::Bottom, 45, 2), [0, 255, 0]);
    assert_eq!(ink_at(RiseAnchor::Top, 45, 2), [0, 0, 255]);
    assert_eq!(ink_at(RiseAnchor::Top, 45, 158), [0, 255, 0]);
}
