use tirage::{DotMode, Frame, Params, Recipe, StippleParams, Tool, ToolPin, derive};

const PIN: ToolPin = ToolPin::Tool(Tool::Stipple);

fn params(recipe: &Recipe) -> &StippleParams {
    let Params::Stipple(params) = recipe.params() else {
        panic!("not a stipple Recipe");
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
            r##"{"tirage":0,"tool":"stipple","tool_seed":9611518,"palette":["#3a86ff","#8ab6ff","#d6e6ff"],"params":{"modesDot":"Lattice","lattices":"Square","shapes":"Circle","res":165,"dot":1.07,"vary":0.62,"cut":0.24,"jit":0.38,"edge":0.41,"loose":0.26,"zoom":4.3,"warp":1.02,"oct":1,"contrast":1.4,"syms":"Mirror","folds":10,"accRate":0.066,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"stipple","tool_seed":3724278653,"palette":["#3a86ff","#8ab6ff","#d6e6ff"],"params":{"modesDot":"Contour","lattices":"Hex","shapes":"Square","res":142,"dot":0.97,"vary":0.03,"cut":0.07,"jit":0.78,"edge":0.53,"loose":0.21,"zoom":6.1,"warp":2.74,"oct":4,"contrast":2.1,"syms":"Quad","folds":12,"accRate":0.032,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"stipple","tool_seed":3565986084,"palette":["#3a86ff","#8ab6ff","#d6e6ff"],"params":{"modesDot":"Contour","lattices":"Hex","shapes":"Square","res":50,"dot":1.26,"vary":0.99,"cut":0.17,"jit":0.21,"edge":0.84,"loose":0.15,"zoom":4.4,"warp":2.7,"oct":1,"contrast":2.35,"syms":"None","folds":9,"accRate":0.084,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PIN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut seen = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, PIN);
        let p = params(&recipe);
        for (value, min, max, unit) in [
            (f64::from(p.res()), 12.0, 200.0, 1.0),
            (p.dot(), 0.75, 1.4, 100.0),
            (p.vary(), 0.0, 1.0, 100.0),
            (p.cut(), 0.0, 0.95, 100.0),
            (p.jit(), 0.0, 1.0, 100.0),
            (p.edge(), 0.0, 1.0, 100.0),
            (p.loose(), 0.0, 0.6, 100.0),
            (p.zoom(), 0.5, 9.0, 10.0),
            (p.warp(), 0.0, 3.0, 100.0),
            (f64::from(p.oct()), 1.0, 7.0, 1.0),
            (p.contrast(), 0.4, 4.0, 100.0),
            (f64::from(p.folds()), 2.0, 16.0, 1.0),
            (p.acc_rate(), 0.0, 0.15, 1000.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !seen.contains(&p.modes_dot()) {
            seen.push(p.modes_dot());
        }
    }
    assert_eq!(seen.len(), DotMode::ALL.len());
}

#[test]
fn stipple_is_a_still() {
    assert_eq!(Tool::Stipple.frames(), 1);
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
    let mut p = StippleParams::default();
    assert_eq!(
        error(p.set_res(201)),
        "stipple: res 201 is outside 12..=200"
    );
    assert_eq!(p, StippleParams::default());
}
