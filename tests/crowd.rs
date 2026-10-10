use tirage::{CrowdParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const CROWD: ToolPin = ToolPin::Tool(Tool::Crowd);

fn crowd(recipe: &Recipe) -> &CrowdParams {
    let Params::Crowd(params) = recipe.params() else {
        panic!("not a crowd Recipe");
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
            r##"{"tirage":0,"tool":"crowd","tool_seed":9611518,"palette":["#b5bab6","#d6ee3a","#f08ae6"],"params":{"crowd":31,"scale":0.19,"wobble":0.21,"blur":0.58,"halo":0.92,"edge":0.87,"arrows":0.46,"arrowSize":0.97,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.35,"grnSize":1.0,"grnSpecks":0.3,"grnVignette":0.2}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"crowd","tool_seed":3724278653,"palette":["#b5bab6","#d6ee3a","#f08ae6"],"params":{"crowd":8,"scale":0.15,"wobble":0.35,"blur":0.81,"halo":0.39,"edge":0.5,"arrows":1.0,"arrowSize":1.0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.35,"grnSize":1.0,"grnSpecks":0.3,"grnVignette":0.2}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"crowd","tool_seed":3565986084,"palette":["#b5bab6","#d6ee3a","#f08ae6"],"params":{"crowd":13,"scale":0.3,"wobble":0.43,"blur":0.06,"halo":0.38,"edge":0.09,"arrows":0.69,"arrowSize":0.85,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.35,"grnSize":1.0,"grnSpecks":0.3,"grnVignette":0.2}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CROWD).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, CROWD);
        let p = crowd(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.crowd()), 8.0, 48.0, 1.0),
            (p.scale(), 0.0, 0.6, 0.01),
            (p.wobble(), 0.0, 1.0, 0.01),
            (p.blur(), 0.0, 1.0, 0.01),
            (p.halo(), 0.0, 1.0, 0.01),
            (p.edge(), 0.0, 1.0, 0.01),
            (p.arrows(), 0.0, 1.0, 0.01),
            (p.arrow_size(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-9, "seed {seed}: {value}");
        }
        assert!(p.grain().on(), "seed {seed}");
    }
}

#[test]
fn crowd_is_a_still() {
    assert_eq!(Tool::Crowd.frames(), 1);
    let recipe = derive(7, CROWD);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CROWD);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = CrowdParams::default();
    assert_eq!(error(p.set_crowd(0)), "crowd: crowd 0 is outside 1..=48");
    assert_eq!(
        error(p.set_scale(1.01)),
        "crowd: scale 1.01 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_arrow_size(-0.01)),
        "crowd: arrowSize -0.01 is outside 0..=1"
    );
    assert_eq!(p, CrowdParams::default());
    p.set_arrows(0.0).unwrap();
    assert_eq!(p.arrows(), 0.0);
}

fn inks_drawn(arrows: f64) -> Vec<[u8; 3]> {
    let palette = Palette::from_hex(&["#000000", "#ff0000", "#0000ff"]).unwrap();
    let mut recipe = derive(7, CROWD);
    recipe.set_palette(palette).unwrap();
    let Params::Crowd(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.grain_mut().set_on(false);
    params.set_crowd(48).unwrap();
    params.set_scale(1.0).unwrap();
    params.set_halo(0.0).unwrap();
    params.set_blur(0.0).unwrap();
    params.set_arrows(arrows).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    image
        .rgba()
        .chunks(4)
        .map(|px| [px[0], px[1], px[2]])
        .collect()
}

#[test]
fn the_ramp_paints_ground_halo_and_core_and_arrows_blend_core_over_them() {
    let plain = inks_drawn(0.0);
    for ink in [[0, 0, 0], [255, 0, 0], [0, 0, 255]] {
        assert!(plain.contains(&ink), "lacks {ink:?}");
    }
    let arrowed = inks_drawn(1.0);
    assert!(
        arrowed.contains(&[230, 0, 0]),
        "no core arrow at 0.9 over ground"
    );
}
