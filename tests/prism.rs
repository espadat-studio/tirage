use tirage::{Frame, Palette, Params, PrismParams, Recipe, Tool, ToolPin, derive, render};

const PRISM: ToolPin = ToolPin::Tool(Tool::Prism);

fn prism(recipe: &Recipe) -> &PrismParams {
    let Params::Prism(params) = recipe.params() else {
        panic!("not a prism Recipe");
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
            r##"{"tirage":0,"tool":"prism","tool_seed":9611518,"params":{"cols":48,"streams":5,"reach":0.37,"fringe":1.12,"dots":0.83,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"prism","tool_seed":3724278653,"params":{"cols":60,"streams":2,"reach":0.36,"fringe":1.54,"dots":0.0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"prism","tool_seed":3565986084,"params":{"cols":44,"streams":1,"reach":0.8,"fringe":0.94,"dots":0.81,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PRISM).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, PRISM);
        let p = prism(&recipe);
        assert!((24..=72).contains(&p.cols()), "seed {seed}");
        assert!(p.cols().is_multiple_of(2), "seed {seed}");
        assert!((1..=5).contains(&p.streams()), "seed {seed}");
        for (value, min, max) in [
            (p.reach(), 0.3, 0.85),
            (p.fringe(), 0.5, 1.6),
            (p.dots(), 0.0, 1.0),
        ] {
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
fn prism_is_a_still() {
    assert_eq!(Tool::Prism.frames(), 1);
    let recipe = derive(7, PRISM);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, PRISM);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = PrismParams::default();
    assert_eq!(error(p.set_cols(74)), "prism: cols 74 is outside 24..=72");
    assert_eq!(
        error(p.set_cols(25)),
        "prism: cols 25 is not a slider value, 24..=72 step 2"
    );
    assert_eq!(error(p.set_streams(0)), "prism: streams 0 is outside 1..=5");
    assert_eq!(
        error(p.set_fringe(1.7)),
        "prism: fringe 1.7 is outside 0.5..=1.6"
    );
    assert_eq!(p, PrismParams::default());
    p.set_cols(72).unwrap();
    p.set_dots(0.0).unwrap();
    assert_eq!((p.cols(), p.dots()), (72, 0.0));
}

#[test]
fn prism_takes_no_palette() {
    let mut recipe = derive(7, PRISM);
    assert_eq!(recipe.palette(), None);
    assert!(!recipe.to_json().contains("palette"));
    let inks = Palette::from_hex(&["#000000", "#ffffff"]).unwrap();
    assert_eq!(error(recipe.set_palette(inks)), "prism: takes no Palette");
    let mut value: serde_json::Value = serde_json::from_str(&recipe.to_json()).unwrap();
    value["palette"] = serde_json::json!(["#000000", "#ffffff"]);
    assert_eq!(
        error(Recipe::from_json(&value.to_string())),
        "prism: takes no Palette"
    );
}

fn inks_drawn(edit: impl FnOnce(&mut PrismParams)) -> Vec<[u8; 3]> {
    let mut recipe = derive(7, PRISM);
    let Params::Prism(params) = recipe.params_mut() else {
        unreachable!()
    };
    edit(params);
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let mut inks: Vec<[u8; 3]> = Vec::new();
    for px in image.rgba().chunks(4) {
        let ink = [px[0], px[1], px[2]];
        if !inks.contains(&ink) {
            inks.push(ink);
        }
    }
    inks
}

#[test]
fn the_field_shows_and_dots_off_leaves_only_cell_inks() {
    let inks = inks_drawn(|p| p.set_dots(0.0).unwrap());
    assert!(inks.contains(&[0x08, 0x08, 0x0a]));
    assert!(inks.len() > 4, "{inks:?}");
    let dotted = inks_drawn(|p| p.set_dots(1.0).unwrap());
    assert!(dotted.len() > inks.len());
}
