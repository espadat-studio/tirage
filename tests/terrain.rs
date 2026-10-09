use tirage::{Frame, Palette, Params, Recipe, TerrainParams, Tool, ToolPin, derive};

const TERRAIN: ToolPin = ToolPin::Tool(Tool::Terrain);

fn terrain(recipe: &Recipe) -> &TerrainParams {
    let Params::Terrain(params) = recipe.params() else {
        panic!("not a terrain Recipe");
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
            r##"{"tirage":0,"tool":"terrain","tool_seed":9611518,"palette":["#264653","#2a9d8f","#8ab17d","#e9c46a","#f4a261","#e76f51"],"params":{"scale":2.0,"warp":0.4,"oct":6,"contrast":2.9,"balance":-0.2,"grain":0.3,"block":2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"terrain","tool_seed":3724278653,"palette":["#264653","#2a9d8f","#8ab17d","#e9c46a","#f4a261","#e76f51"],"params":{"scale":7.8,"warp":0.02,"oct":6,"contrast":3.35,"balance":-0.3,"grain":0.3,"block":10,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"terrain","tool_seed":3565986084,"palette":["#264653","#2a9d8f","#8ab17d","#e9c46a","#f4a261","#e76f51"],"params":{"scale":2.0,"warp":0.78,"oct":1,"contrast":1.6,"balance":0.1,"grain":0.3,"block":9,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, TERRAIN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let palette = Palette::from_hex(&[
        "#264653", "#2a9d8f", "#8ab17d", "#e9c46a", "#f4a261", "#e76f51",
    ])
    .unwrap();
    for seed in 0..2000 {
        let recipe = derive(seed, TERRAIN);
        let p = terrain(&recipe);
        assert_eq!(recipe.palette(), &palette);
        assert_eq!(p.grain(), 0.3, "seed {seed}");
        let value = p.scale();
        assert!((1.5..=9.0).contains(&value), "seed {seed}: scale {value}");
        let ticks = (value - 0.6) / 0.1;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: scale {value}"
        );
        let value = p.warp();
        assert!((0.0..=3.0).contains(&value), "seed {seed}: warp {value}");
        let ticks = (value - 0.0) / 0.02;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: warp {value}"
        );
        assert!((1..=7).contains(&p.oct()), "seed {seed}: oct {}", p.oct());
        let value = p.contrast();
        assert!(
            (1.0..=4.0).contains(&value),
            "seed {seed}: contrast {value}"
        );
        let ticks = (value - 0.5) / 0.05;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: contrast {value}"
        );
        let value = p.balance();
        assert!(
            (-1.2..=1.2).contains(&value),
            "seed {seed}: balance {value}"
        );
        let ticks = (value - -1.2) / 0.05;
        assert!(
            (ticks - ticks.round()).abs() < 1e-6,
            "seed {seed}: balance {value}"
        );
        assert!(
            (0..=14).contains(&p.block()),
            "seed {seed}: block {}",
            p.block()
        );
    }
}

#[test]
fn terrain_is_a_still() {
    assert_eq!(Tool::Terrain.frames(), 1);
    let recipe = derive(7, TERRAIN);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, TERRAIN);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = TerrainParams::default();
    assert_eq!(
        error(p.set_scale(9.5)),
        "terrain: scale 9.5 is outside 0.6..=9"
    );
    assert_eq!(error(p.set_warp(3.5)), "terrain: warp 3.5 is outside 0..=3");
    assert_eq!(error(p.set_oct(0)), "terrain: oct 0 is outside 1..=7");
    assert_eq!(error(p.set_oct(8)), "terrain: oct 8 is outside 1..=7");
    assert_eq!(
        error(p.set_contrast(4.5)),
        "terrain: contrast 4.5 is outside 0.5..=4"
    );
    assert_eq!(
        error(p.set_balance(1.7)),
        "terrain: balance 1.7 is outside -1.2..=1.2"
    );
    assert_eq!(
        error(p.set_grain(1.7)),
        "terrain: grain 1.7 is outside 0..=1.2"
    );
    assert_eq!(
        error(p.set_block(15)),
        "terrain: block 15 is outside 0..=14"
    );
    assert_eq!(p, TerrainParams::default());
}
