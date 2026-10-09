use tirage::{Frame, Palette, Params, Recipe, Tool, ToolPin, WeaveParams, derive, render};

const WEAVE: ToolPin = ToolPin::Tool(Tool::Weave);

fn weave(recipe: &Recipe) -> &WeaveParams {
    let Params::Weave(params) = recipe.params() else {
        panic!("not a weave Recipe");
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
            r##"{"tirage":0,"tool":"weave","tool_seed":9611518,"palette":["#22223b","#f2e9e4","#c9ada7","#9a8c98","#4a4e69","#e63946","#f4d35e"],"params":{"bands":6,"stripe":2.2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"weave","tool_seed":3724278653,"palette":["#22223b","#f2e9e4","#c9ada7","#9a8c98","#4a4e69","#e63946","#f4d35e"],"params":{"bands":12,"stripe":0.95,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"weave","tool_seed":3565986084,"palette":["#22223b","#f2e9e4","#c9ada7","#9a8c98","#4a4e69","#e63946","#f4d35e"],"params":{"bands":9,"stripe":0.95,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, WEAVE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, WEAVE);
        let p = weave(&recipe);
        assert!((3..=12).contains(&p.bands()), "seed {seed}: {}", p.bands());
        let value = p.stripe();
        assert!((0.5..=2.2).contains(&value), "seed {seed}: stripe {value}");
        let ticks = (value - 0.5) / 0.05;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: stripe {value}"
        );
    }
}

#[test]
fn weave_is_a_still() {
    assert_eq!(Tool::Weave.frames(), 1);
    let recipe = derive(7, WEAVE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn a_two_ink_palette_paints_both_inks() {
    let mut recipe = derive(7, WEAVE);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 54, 96, 0).unwrap());
    let reds = image.rgba().chunks(4).filter(|px| px[0] == 255).count();
    let blacks = image.rgba().chunks(4).filter(|px| px[0] == 0).count();
    assert_eq!(reds + blacks, 54 * 96);
    assert!(reds > 0 && blacks > 0, "{reds} {blacks}");
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, WEAVE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = WeaveParams::default();
    assert_eq!(error(p.set_bands(2)), "weave: bands 2 is outside 3..=12");
    assert_eq!(error(p.set_bands(13)), "weave: bands 13 is outside 3..=12");
    assert_eq!(
        error(p.set_stripe(2.3)),
        "weave: stripe 2.3 is outside 0.5..=2.2"
    );
    assert_eq!(p, WeaveParams::default());
    p.set_bands(12).unwrap();
    p.set_stripe(0.5).unwrap();
    assert_eq!((p.bands(), p.stripe()), (12, 0.5));
}
