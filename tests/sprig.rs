use tirage::{Frame, MotifSet, Params, Recipe, SprigParams, Tool, ToolPin, derive};

const PIN: ToolPin = ToolPin::Tool(Tool::Sprig);

fn params(recipe: &Recipe) -> &SprigParams {
    let Params::Sprig(params) = recipe.params() else {
        panic!("not a sprig Recipe");
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
            r##"{"tirage":0,"tool":"sprig","tool_seed":9611518,"palette":["#f9e4c8","#2d6a4f"],"params":{"dirs":"Leaves","count":96,"size":0.49,"vary":0.01,"solids":0.2,"weight":0.4,"rough":0.55,"wobble":0.21,"detail":1.0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"sprig","tool_seed":3724278653,"palette":["#f9e4c8","#2d6a4f"],"params":{"dirs":"Blooms","count":33,"size":0.23,"vary":0.98,"solids":0.54,"weight":0.59,"rough":0.5,"wobble":0.96,"detail":0.87,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"sprig","tool_seed":3565986084,"palette":["#f9e4c8","#2d6a4f"],"params":{"dirs":"Blooms","count":70,"size":0.07,"vary":0.7,"solids":0.86,"weight":0.09,"rough":0.24,"wobble":0.0,"detail":0.55,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
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
            (f64::from(p.count()), 20.0, 100.0, 1.0),
            (p.size(), 0.0, 0.6, 100.0),
            (p.vary(), 0.0, 1.0, 100.0),
            (p.solids(), 0.0, 1.0, 100.0),
            (p.weight(), 0.0, 0.8, 100.0),
            (p.rough(), 0.0, 1.0, 100.0),
            (p.wobble(), 0.0, 1.0, 100.0),
            (p.detail(), 0.0, 1.0, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !seen.contains(&p.dirs()) {
            seen.push(p.dirs());
        }
    }
    assert_eq!(seen.len(), MotifSet::ALL.len());
}

#[test]
fn sprig_is_a_still() {
    assert_eq!(Tool::Sprig.frames(), 1);
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
    let mut p = SprigParams::default();
    assert_eq!(
        error(p.set_count(121)),
        "sprig: count 121 is outside 2..=120"
    );
    assert_eq!(p, SprigParams::default());
}
