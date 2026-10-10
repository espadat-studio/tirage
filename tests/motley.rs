use tirage::{Frame, MotleyParams, Params, Recipe, Tool, ToolPin, derive};

const PIN: ToolPin = ToolPin::Tool(Tool::Motley);

fn params(recipe: &Recipe) -> &MotleyParams {
    let Params::Motley(params) = recipe.params() else {
        panic!("not a motley Recipe");
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
            r##"{"tirage":0,"tool":"motley","tool_seed":9611518,"palette":["#111111","#f4f1ea","#1e7a46","#e5352b","#f5c400","#f28ab8","#2c7ac2","#f0801e","#f3e2b0"],"params":{"cells":24,"mark":0.88,"patches":10,"size":0.76,"ragged":0.48,"blocks":0.43,"sparse":0.45,"stitch":0.98,"gaps":0.14,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"motley","tool_seed":3724278653,"palette":["#111111","#f4f1ea","#1e7a46","#e5352b","#f5c400","#f28ab8","#2c7ac2","#f0801e","#f3e2b0"],"params":{"cells":66,"mark":0.85,"patches":3,"size":0.32,"ragged":0.46,"blocks":0.69,"sparse":0.42,"stitch":0.86,"gaps":0.13,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"motley","tool_seed":3565986084,"palette":["#111111","#f4f1ea","#1e7a46","#e5352b","#f5c400","#f28ab8","#2c7ac2","#f0801e","#f3e2b0"],"params":{"cells":116,"mark":0.92,"patches":44,"size":0.63,"ragged":0.63,"blocks":0.07,"sparse":0.43,"stitch":0.9,"gaps":0.22,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PIN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, PIN);
        let p = params(&recipe);
        for (value, min, max, unit) in [
            (f64::from(p.cells()), 16.0, 120.0, 1.0),
            (p.mark(), 0.75, 1.0, 100.0),
            (f64::from(p.patches()), 1.0, 60.0, 1.0),
            (p.size(), 0.0, 1.0, 100.0),
            (p.ragged(), 0.0, 1.0, 100.0),
            (p.blocks(), 0.0, 1.0, 100.0),
            (p.sparse(), 0.0, 0.6, 100.0),
            (p.stitch(), 0.0, 1.0, 100.0),
            (p.gaps(), 0.0, 0.3, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
    }
}

#[test]
fn motley_is_a_still() {
    assert_eq!(Tool::Motley.frames(), 1);
    let recipe = derive(7, PIN);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, PIN);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = MotleyParams::default();
    assert_eq!(
        error(p.set_cells(121)),
        "motley: cells 121 is outside 16..=120"
    );
    assert_eq!(p, MotleyParams::default());
}
