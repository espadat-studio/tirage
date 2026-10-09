use tirage::{BendayParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive};

const BENDAY: ToolPin = ToolPin::Tool(Tool::Benday);

fn benday(recipe: &Recipe) -> &BendayParams {
    let Params::Benday(params) = recipe.params() else {
        panic!("not a benday Recipe");
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
            r##"{"tirage":0,"tool":"benday","tool_seed":9611518,"palette":["#0a0a0c","#5b1e9e","#2c4be8","#31c6f0","#8fe23a","#f5f03a","#ff8a1e","#ff2a1e"],"params":{"turb":0.31,"streak":0.59,"dir":0.02,"scale":0.64,"black":0.51,"bands":10,"rims":0.78,"dot":0.66,"ring":0.67,"angle":0.59,"bite":0.42,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"benday","tool_seed":3724278653,"palette":["#0a0a0c","#5b1e9e","#2c4be8","#31c6f0","#8fe23a","#f5f03a","#ff8a1e","#ff2a1e"],"params":{"turb":0.39,"streak":0.64,"dir":-0.43,"scale":0.88,"black":0.32,"bands":8,"rims":0.55,"dot":0.9,"ring":0.84,"angle":0.37,"bite":0.05,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"benday","tool_seed":3565986084,"palette":["#0a0a0c","#5b1e9e","#2c4be8","#31c6f0","#8fe23a","#f5f03a","#ff8a1e","#ff2a1e"],"params":{"turb":0.39,"streak":0.45,"dir":0.4,"scale":0.69,"black":0.34,"bands":16,"rims":0.54,"dot":0.16,"ring":0.98,"angle":0.52,"bite":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, BENDAY).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, BENDAY);
        let p = benday(&recipe);
        assert_eq!(recipe.palette().to_hex().len(), 8);
        assert!(
            (2..=16).contains(&p.bands()),
            "seed {seed}: bands {}",
            p.bands()
        );
        for (value, min, max) in [
            (p.turb(), 0.0, 1.0),
            (p.streak(), 0.0, 1.0),
            (p.dir(), -1.0, 1.0),
            (p.scale(), 0.0, 1.0),
            (p.black(), 0.0, 0.6),
            (p.rims(), 0.0, 1.0),
            (p.dot(), 0.0, 1.0),
            (p.ring(), 0.0, 1.0),
            (p.angle(), 0.0, 1.0),
            (p.bite(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn benday_is_a_still() {
    assert_eq!(Tool::Benday.frames(), 1);
    let recipe = derive(7, BENDAY);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, BENDAY);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    let Params::Benday(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_dir(-0.35).unwrap();
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = BendayParams::default();
    assert_eq!(error(p.set_bands(1)), "benday: bands 1 is outside 2..=16");
    assert_eq!(error(p.set_bands(17)), "benday: bands 17 is outside 2..=16");
    assert_eq!(error(p.set_dir(-1.1)), "benday: dir -1.1 is outside -1..=1");
    assert_eq!(
        error(p.set_bite(f64::NAN)),
        "benday: bite NaN is outside 0..=1"
    );
    assert_eq!(p, BendayParams::default());
}
