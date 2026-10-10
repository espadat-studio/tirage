use tirage::{
    FoldKind, FoldMirror, FoldParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const FOLD: ToolPin = ToolPin::Tool(Tool::Fold);

fn fold(recipe: &Recipe) -> &FoldParams {
    let Params::Fold(params) = recipe.params() else {
        panic!("not a fold Recipe");
    };
    params
}

fn error<T: std::fmt::Debug>(result: Result<T, tirage::Error>) -> String {
    result.unwrap_err().to_string()
}

const MIRRORS: [FoldMirror; 5] = [
    FoldMirror::FourWays,
    FoldMirror::Kaleidoscope,
    FoldMirror::Across,
    FoldMirror::Down,
    FoldMirror::None,
];

const KINDS: [FoldKind; 7] = [
    FoldKind::Mixed,
    FoldKind::Zebra,
    FoldKind::Staircases,
    FoldKind::DotRows,
    FoldKind::Bursts,
    FoldKind::Chevrons,
    FoldKind::Checks,
];

#[test]
fn derive_gives_golden_recipes() {
    let golden = [
        (
            1,
            r##"{"tirage":0,"tool":"fold","tool_seed":9611518,"palette":["#1a1a1a","#f5f1e8","#ff5c39","#3b5bff","#ffd23f","#7b2cbf"],"params":{"folds":"Four ways","kinds":"Checks","foldx":0.64,"foldy":0.3,"patches":4,"size":0.01,"scale":0.46,"wave":0.79,"levels":4,"tilt":0.73,"px":0.61,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"fold","tool_seed":3724278653,"palette":["#1a1a1a","#f5f1e8","#ff5c39","#3b5bff","#ffd23f","#7b2cbf"],"params":{"folds":"Down","kinds":"Bursts","foldx":0.39,"foldy":0.34,"patches":13,"size":0.28,"scale":0.73,"wave":0.61,"levels":7,"tilt":0.97,"px":0.12,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"fold","tool_seed":3565986084,"palette":["#1a1a1a","#f5f1e8","#ff5c39","#3b5bff","#ffd23f","#7b2cbf"],"params":{"folds":"Four ways","kinds":"Checks","foldx":0.5,"foldy":0.75,"patches":13,"size":0.61,"scale":0.81,"wave":0.88,"levels":7,"tilt":0.22,"px":0.89,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, FOLD).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let (mut mirrors, mut kinds) = (Vec::new(), Vec::new());
    for seed in 0..2000 {
        let recipe = derive(seed, FOLD);
        let p = fold(&recipe);
        for (value, min, max, step) in [
            (p.foldx(), 0.2, 0.8, 0.01),
            (p.foldy(), 0.2, 0.8, 0.01),
            (f64::from(p.patches()), 1.0, 16.0, 1.0),
            (p.size(), 0.0, 1.0, 0.01),
            (p.scale(), 0.0, 1.0, 0.01),
            (p.wave(), 0.0, 1.0, 0.01),
            (f64::from(p.levels()), 2.0, 8.0, 1.0),
            (p.tilt(), 0.0, 1.0, 0.01),
            (p.px(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-6, "seed {seed}: {value}");
        }
        if !mirrors.contains(&p.mirror()) {
            mirrors.push(p.mirror());
        }
        if !kinds.contains(&p.kind()) {
            kinds.push(p.kind());
        }
    }
    assert_eq!(
        (mirrors.len(), kinds.len()),
        (5, 7),
        "{mirrors:?} {kinds:?}"
    );
}

#[test]
fn fold_is_a_still() {
    assert_eq!(Tool::Fold.frames(), 1);
    let recipe = derive(7, FOLD);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, FOLD);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = FoldParams::default();
    assert_eq!(
        error(p.set_foldx(0.1)),
        "fold: foldx 0.1 is outside 0.2..=0.8"
    );
    assert_eq!(
        error(p.set_patches(17)),
        "fold: patches 17 is outside 1..=16"
    );
    assert_eq!(error(p.set_levels(1)), "fold: levels 1 is outside 2..=8");
    assert_eq!(error(p.set_px(1.01)), "fold: px 1.01 is outside 0..=1");
    assert_eq!(p, FoldParams::default());
    p.set_mirror(FoldMirror::None);
    p.set_kind(FoldKind::DotRows);
    assert_eq!(
        (p.mirror(), p.kind()),
        (FoldMirror::None, FoldKind::DotRows)
    );
}

#[test]
fn zebra_patches_draw_the_darkest_and_lightest_inks() {
    let palette = Palette::from_hex(&["#ff0000", "#808080", "#000000", "#ffffff"]).unwrap();
    let mut recipe = derive(7, FOLD);
    recipe.set_palette(palette).unwrap();
    let Params::Fold(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_kind(FoldKind::Zebra);
    params.set_patches(16).unwrap();
    params.set_size(1.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    for ink in [[0, 0, 0], [255, 255, 255]] {
        assert!(image.rgba().chunks(4).any(|px| px[..3] == ink), "{ink:?}");
    }
}

#[test]
fn every_fold_and_kind_renders_with_two_inks() {
    let palette = Palette::from_hex(&["#000000", "#ffffff"]).unwrap();
    for mirror in MIRRORS {
        for kind in KINDS {
            let mut recipe = derive(7, FOLD);
            recipe.set_palette(palette.clone()).unwrap();
            let Params::Fold(params) = recipe.params_mut() else {
                unreachable!()
            };
            params.set_mirror(mirror);
            params.set_kind(kind);
            let image = render(&recipe, &Frame::new(&recipe, 45, 80, 0).unwrap());
            assert_eq!(image.rgba().len(), 45 * 80 * 4, "{mirror:?} {kind:?}");
        }
    }
}
