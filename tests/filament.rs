use tirage::{
    FilamentMarks, FilamentParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const FILAMENT: ToolPin = ToolPin::Tool(Tool::Filament);

fn filament(recipe: &Recipe) -> &FilamentParams {
    let Params::Filament(params) = recipe.params() else {
        panic!("not a filament Recipe");
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
            r##"{"tirage":0,"tool":"filament","tool_seed":9611518,"palette":["#ff5da2","#ff9acb","#ffe6f1"],"params":{"zoom":3.75,"turn":2.4,"oct":1,"curl":0.93,"tangle":0.61,"bundles":58,"per":21,"clump":0.29,"tight":0.005,"len":540,"step":3.4,"wgt":7.7,"wvar":0.45,"hier":0.48,"hair":0.11,"mstyles":"Both","mark":0.92,"bead":0.79,"bgap":8,"msize":15.0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"filament","tool_seed":3724278653,"palette":["#ff5da2","#ff9acb","#ffe6f1"],"params":{"zoom":3.1,"turn":2.0,"oct":2,"curl":0.44,"tangle":0.29,"bundles":29,"per":6,"clump":0.69,"tight":0.18,"len":360,"step":7.6,"wgt":4.7,"wvar":0.17,"hier":0.37,"hair":0.63,"mstyles":"Both","mark":0.94,"bead":0.84,"bgap":10,"msize":11.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"filament","tool_seed":3565986084,"palette":["#ff5da2","#ff9acb","#ffe6f1"],"params":{"zoom":4.6,"turn":4.85,"oct":5,"curl":0.12,"tangle":0.75,"bundles":31,"per":30,"clump":0.12,"tight":0.155,"len":370,"step":7.4,"wgt":3.7,"wvar":0.56,"hier":0.36,"hair":0.91,"mstyles":"Symbols","mark":0.47,"bead":0.92,"bgap":8,"msize":7.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, FILAMENT).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let on_grid = |value: f64, low: f64, step: f64| {
        let ticks = (value - low) / step;
        (ticks - ticks.round()).abs() < 1e-9
    };
    for seed in 0..2000 {
        let recipe = derive(seed, FILAMENT);
        let p = filament(&recipe);
        assert!((23..=90).contains(&p.bundles()), "seed {seed}");
        assert!((160..=600).contains(&p.len()), "seed {seed}");
        assert!(p.len().is_multiple_of(10), "seed {seed}: {}", p.len());
        assert!((0.3..=6.0).contains(&p.zoom()), "seed {seed}");
        assert!(on_grid(p.zoom(), 0.3, 0.05), "seed {seed}: {}", p.zoom());
        assert!(
            on_grid(p.tight(), 0.005, 0.005),
            "seed {seed}: {}",
            p.tight()
        );
        assert!(on_grid(p.step(), 1.0, 0.2), "seed {seed}: {}", p.step());
        assert!(on_grid(p.msize(), 2.0, 0.5), "seed {seed}: {}", p.msize());
    }
}

#[test]
fn derive_deals_every_mark_style() {
    let styles: Vec<FilamentMarks> = (0..60)
        .map(|seed| filament(&derive(seed, FILAMENT)).mstyles())
        .collect();
    for style in [
        FilamentMarks::Symbols,
        FilamentMarks::Beads,
        FilamentMarks::Both,
    ] {
        assert!(styles.contains(&style), "{style:?} never dealt");
    }
}

#[test]
fn filament_is_a_still() {
    assert_eq!(Tool::Filament.frames(), 1);
    let recipe = derive(7, FILAMENT);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, FILAMENT);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = FilamentParams::default();
    assert_eq!(
        error(p.set_bundles(91)),
        "filament: bundles 91 is outside 1..=90"
    );
    assert_eq!(
        error(p.set_tight(0.001)),
        "filament: tight 0.001 is outside 0.005..=0.2"
    );
    assert_eq!(p, FilamentParams::default());
    p.set_len(20).unwrap();
    p.set_wgt(8.0).unwrap();
    assert_eq!((p.len(), p.wgt()), (20, 8.0));
}

fn inks(recipe: &Recipe) -> Vec<[u8; 3]> {
    let image = render(recipe, &Frame::new(recipe, 270, 480, 0).unwrap());
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for px in image.rgba().chunks(4) {
        let ink = [px[0], px[1], px[2]];
        if !seen.contains(&ink) {
            seen.push(ink);
        }
    }
    seen
}

#[test]
fn strands_take_palette_inks_and_symbols_take_the_fixed_accents() {
    let mut recipe = derive(1, FILAMENT);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let seen = inks(&recipe);
    for ink in [
        [0x12, 0x0a, 0x1e],
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [0xf9, 0xf8, 0x71],
        [0x4c, 0xe0, 0xd2],
        [0xb0, 0x9c, 0xff],
    ] {
        assert!(seen.contains(&ink), "{ink:?} never drawn");
    }
}

#[test]
fn beads_alone_draw_no_accent() {
    let mut recipe = derive(1, FILAMENT);
    let Params::Filament(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_mstyles(FilamentMarks::Beads);
    let seen = inks(&recipe);
    assert!(!seen.contains(&[0xf9, 0xf8, 0x71]));
    assert!(!seen.contains(&[0x4c, 0xe0, 0xd2]));
}
