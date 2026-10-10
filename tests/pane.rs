use tirage::{Frame, Palette, PaneParams, Params, Recipe, Tool, ToolPin, derive, render};

const PANE: ToolPin = ToolPin::Tool(Tool::Pane);

fn pane(recipe: &Recipe) -> &PaneParams {
    let Params::Pane(params) = recipe.params() else {
        panic!("not a pane Recipe");
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
            r##"{"tirage":0,"tool":"pane","tool_seed":9611518,"palette":["#22223b","#4a4e69","#9a8c98","#c9ada7","#f2e9e4","#ff7b54","#ffb26b","#ffd56f"],"params":{"rows":1,"cells":14,"vary":0.58,"diag":0.25,"soft":0.32,"spread":0.66,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"pane","tool_seed":3724278653,"palette":["#22223b","#4a4e69","#9a8c98","#c9ada7","#f2e9e4","#ff7b54","#ffb26b","#ffd56f"],"params":{"rows":10,"cells":15,"vary":0.8,"diag":0.53,"soft":0.32,"spread":0.44,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"pane","tool_seed":3565986084,"palette":["#22223b","#4a4e69","#9a8c98","#c9ada7","#f2e9e4","#ff7b54","#ffb26b","#ffd56f"],"params":{"rows":9,"cells":6,"vary":0.34,"diag":0.52,"soft":0.62,"spread":0.22,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PANE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, PANE);
        let p = pane(&recipe);
        assert!((1..=10).contains(&p.rows()), "seed {seed}");
        assert!((1..=16).contains(&p.cells()), "seed {seed}");
        for value in [p.vary(), p.diag(), p.soft(), p.spread()] {
            assert!((0.0..=1.0).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(!p.grain().on() && !p.dither().on(), "seed {seed}");
    }
}

#[test]
fn pane_is_a_still() {
    assert_eq!(Tool::Pane.frames(), 1);
    let recipe = derive(7, PANE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, PANE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = PaneParams::default();
    assert_eq!(error(p.set_rows(0)), "pane: rows 0 is outside 1..=10");
    assert_eq!(error(p.set_cells(17)), "pane: cells 17 is outside 1..=16");
    assert_eq!(error(p.set_soft(1.01)), "pane: soft 1.01 is outside 0..=1");
    assert_eq!(p, PaneParams::default());
    p.set_rows(10).unwrap();
    p.set_spread(0.0).unwrap();
    assert_eq!((p.rows(), p.spread()), (10, 0.0));
}

#[test]
fn a_hard_ramp_shows_both_inks_and_walks_the_hue_between_them() {
    let mut recipe = derive(7, PANE);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    let Params::Pane(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_soft(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let pixels: Vec<&[u8]> = image.rgba().chunks(4).collect();
    for ink in [[0, 0, 0], [255, 0, 0]] {
        assert!(
            pixels.iter().any(|px| px[..3] == ink),
            "{ink:?} never drawn"
        );
    }
    assert!(
        pixels.iter().all(|px| px[1].abs_diff(px[2]) <= 1),
        "black takes red's hue, so the ramp stays on the red axis"
    );
}
