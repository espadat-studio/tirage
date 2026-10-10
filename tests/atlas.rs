use tirage::{AtlasParams, CharacterSet, Frame, Params, Recipe, Tool, ToolPin, derive};

const ATLAS: ToolPin = ToolPin::Tool(Tool::Atlas);

fn atlas(recipe: &Recipe) -> &AtlasParams {
    let Params::Atlas(params) = recipe.params() else {
        panic!("not an atlas Recipe");
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
            r##"{"tirage":0,"tool":"atlas","tool_seed":9611518,"palette":["#0d2b45","#203c56","#544e68","#8d697a","#d08159","#ffaa5e","#ffd4a3","#ffecd6"],"params":{"scale":2.4,"warp":0.42,"bands":3,"mix":0.92,"sets":"DOS","cols":115,"density":0.5,"weight":1.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"atlas","tool_seed":3724278653,"palette":["#0d2b45","#203c56","#544e68","#8d697a","#d08159","#ffaa5e","#ffd4a3","#ffecd6"],"params":{"scale":1.0,"warp":0.47,"bands":14,"mix":0.85,"sets":"Blocks","cols":26,"density":0.31,"weight":0.57,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"atlas","tool_seed":3565986084,"palette":["#0d2b45","#203c56","#544e68","#8d697a","#d08159","#ffaa5e","#ffd4a3","#ffecd6"],"params":{"scale":7.1,"warp":1.16,"bands":9,"mix":0.47,"sets":"Runes","cols":158,"density":0.16,"weight":1.19,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, ATLAS).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut sets = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, ATLAS);
        let p = atlas(&recipe);
        for (value, min, max, unit) in [
            (p.scale(), 1.0, 9.0, 10.0),
            (p.warp(), 0.0, 1.6, 100.0),
            (f64::from(p.bands()), 3.0, 14.0, 1.0),
            (p.mix(), 0.0, 1.0, 100.0),
            (f64::from(p.cols()), 24.0, 260.0, 1.0),
            (p.density(), 0.0, 1.0, 100.0),
            (p.weight(), 0.0, 1.5, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !sets.contains(&p.sets()) {
            sets.push(p.sets());
        }
    }
    assert_eq!(sets.len(), CharacterSet::ALL.len());
}

#[test]
fn atlas_is_a_still() {
    assert_eq!(Tool::Atlas.frames(), 1);
    let recipe = derive(7, ATLAS);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, ATLAS);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = AtlasParams::default();
    assert_eq!(error(p.set_scale(0.9)), "atlas: scale 0.9 is outside 1..=9");
    assert_eq!(error(p.set_warp(1.7)), "atlas: warp 1.7 is outside 0..=1.6");
    assert_eq!(error(p.set_bands(1)), "atlas: bands 1 is outside 2..=14");
    assert_eq!(
        error(p.set_cols(261)),
        "atlas: cols 261 is outside 24..=260"
    );
    assert_eq!(
        error(p.set_weight(f64::NAN)),
        "atlas: weight NaN is outside 0..=1.5"
    );
    assert_eq!(p, AtlasParams::default());
    p.set_bands(2).unwrap();
    assert_eq!(p.bands(), 2);
}

#[test]
fn any_palette_length_renders() {
    let mut recipe = derive(7, ATLAS);
    recipe
        .set_palette(tirage::Palette::from_hex(&["#000000", "#ffffff"]).unwrap())
        .unwrap();
    let frame = Frame::new(&recipe, 90, 160, 0).unwrap();
    let image = tirage::render(&recipe, &frame);
    assert!(image.rgba().chunks(4).any(|px| px[0] == 0));
    assert!(image.rgba().chunks(4).any(|px| px[0] == 255));
}
