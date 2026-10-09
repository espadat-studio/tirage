use tirage::{CultureParams, CultureTexture, Frame, Params, Recipe, Tool, ToolPin, derive};

const CULTURE: ToolPin = ToolPin::Tool(Tool::Culture);

fn culture(recipe: &Recipe) -> &CultureParams {
    let Params::Culture(params) = recipe.params() else {
        panic!("not a culture Recipe");
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
            r##"{"tirage":0,"tool":"culture","tool_seed":9611518,"palette":["#f6d8b0","#e4572e","#17bebb"],"params":{"count":48,"size":0.28,"fuse":0.3,"cover":0.32,"spread":1.87,"soft":0.34,"textures":"Fine","grain":0.55,"dot":1,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"culture","tool_seed":3724278653,"palette":["#f6d8b0","#e4572e","#17bebb"],"params":{"count":42,"size":0.48,"fuse":0.47,"cover":0.4,"spread":1.37,"soft":0.84,"textures":"Fine","grain":0.82,"dot":8,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"culture","tool_seed":3565986084,"palette":["#f6d8b0","#e4572e","#17bebb"],"params":{"count":41,"size":0.4,"fuse":0.07,"cover":0.78,"spread":1.55,"soft":0.84,"textures":"Stipple","grain":0.09,"dot":4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CULTURE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut textures = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, CULTURE);
        let p = culture(&recipe);
        assert!((15..=70).contains(&p.count()), "seed {seed}");
        assert!((1..=10).contains(&p.dot()), "seed {seed}");
        for (value, min, max) in [
            (p.size(), 0.15, 0.6),
            (p.fuse(), 0.0, 0.8),
            (p.cover(), 0.0, 1.0),
            (p.spread(), 0.05, 2.0),
            (p.soft(), 0.02, 1.0),
            (p.grain(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        if !textures.contains(&p.texture()) {
            textures.push(p.texture());
        }
    }
    assert_eq!(textures.len(), 2, "{textures:?}");
}

#[test]
fn culture_is_a_still() {
    assert_eq!(Tool::Culture.frames(), 1);
    let recipe = derive(7, CULTURE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CULTURE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = CultureParams::default();
    assert_eq!(
        error(p.set_count(91)),
        "culture: count 91 is outside 1..=90"
    );
    assert_eq!(
        error(p.set_spread(0.01)),
        "culture: spread 0.01 is outside 0.05..=2"
    );
    assert_eq!(p, CultureParams::default());
    p.set_texture(CultureTexture::Stipple);
    p.set_count(1).unwrap();
    assert_eq!((p.texture(), p.count()), (CultureTexture::Stipple, 1));
}
