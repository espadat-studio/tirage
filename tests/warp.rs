use tirage::{
    Frame, Palette, Params, Recipe, Tool, ToolPin, WarpParams, WarpStyle, derive, render,
};

const WARP: ToolPin = ToolPin::Tool(Tool::Warp);

fn warp(recipe: &Recipe) -> &WarpParams {
    let Params::Warp(params) = recipe.params() else {
        panic!("not a warp Recipe");
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
            r##"{"tirage":0,"tool":"warp","tool_seed":9611518,"palette":["#f7f3ea","#f72585","#4361ee","#f7b32b","#1b1b1b"],"params":{"styles":"Checker","scale":1.4,"warp":0.73,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"warp","tool_seed":3724278653,"palette":["#f7f3ea","#f72585","#4361ee","#f7b32b","#1b1b1b"],"params":{"styles":"Slash","scale":0.75,"warp":0.79,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"warp","tool_seed":3565986084,"palette":["#f7f3ea","#f72585","#4361ee","#f7b32b","#1b1b1b"],"params":{"styles":"Slash","scale":1.85,"warp":0.81,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, WARP).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut styles = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, WARP);
        let p = warp(&recipe);
        for (value, min, max, unit) in [(p.scale(), 0.5, 2.0, 20.0), (p.warp(), 0.0, 1.0, 100.0)] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !styles.contains(&p.style()) {
            styles.push(p.style());
        }
    }
    assert_eq!(styles.len(), 2, "{styles:?}");
}

#[test]
fn warp_is_a_still() {
    assert_eq!(Tool::Warp.frames(), 1);
    let recipe = derive(7, WARP);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, WARP);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = WarpParams::default();
    assert_eq!(
        error(p.set_scale(0.45)),
        "warp: scale 0.45 is outside 0.5..=2"
    );
    assert_eq!(error(p.set_warp(1.01)), "warp: warp 1.01 is outside 0..=1");
    assert_eq!(p, WarpParams::default());
    p.set_style(WarpStyle::Checker);
    p.set_scale(2.0).unwrap();
    assert_eq!((p.style(), p.scale()), (WarpStyle::Checker, 2.0));
}

#[test]
fn a_palette_over_five_inks_is_refused() {
    let six = Palette::from_hex(&[
        "#0a0a0c", "#f05a1e", "#f0d82c", "#2cc8f0", "#f03c8c", "#ffffff",
    ])
    .unwrap();
    let mut recipe = derive(7, WARP);
    assert_eq!(
        error(recipe.set_palette(six)),
        "warp: palette has 6 inks, warp draws at most 5"
    );
    assert_eq!(recipe, derive(7, WARP));
}

#[test]
fn each_style_paints_two_inks() {
    let mut recipe = derive(1, WARP);
    for style in [WarpStyle::Checker, WarpStyle::Slash] {
        let Params::Warp(params) = recipe.params_mut() else {
            panic!("not a warp Recipe");
        };
        params.set_style(style);
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        let mut inks: Vec<&[u8]> = image.rgba().chunks(4).collect();
        inks.sort_unstable();
        inks.dedup();
        assert!(inks.len() >= 2, "{style:?}");
    }
}
