use tirage::{Frame, Palette, Params, Recipe, Tool, ToolPin, ZigParams, ZigStyle, derive, render};

const ZIG: ToolPin = ToolPin::Tool(Tool::Zig);

fn zig(recipe: &Recipe) -> &ZigParams {
    let Params::Zig(params) = recipe.params() else {
        panic!("not a zig Recipe");
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
            r##"{"tirage":0,"tool":"zig","tool_seed":9611518,"palette":["#1b1b1b","#f94144","#f8961e","#f9c74f","#43aa8b","#577590"],"params":{"styles":"Stairs","width":1.8,"depth":0.48,"tooth":0.65,"round":0.12,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"zig","tool_seed":3724278653,"palette":["#1b1b1b","#f94144","#f8961e","#f9c74f","#43aa8b","#577590"],"params":{"styles":"Stairs","width":1.45,"depth":0.62,"tooth":0.7,"round":0.33,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"zig","tool_seed":3565986084,"palette":["#1b1b1b","#f94144","#f8961e","#f9c74f","#43aa8b","#577590"],"params":{"styles":"Waves","width":1.4,"depth":1.1,"tooth":0.85,"round":0.63,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, ZIG).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_on_the_slider_grid_and_every_style_is_dealt() {
    let mut styles = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, ZIG);
        let p = zig(&recipe);
        for (value, min, max, step) in [
            (p.width(), 0.5, 2.0, 0.05),
            (p.depth(), 0.2, 1.4, 0.02),
            (p.tooth(), 0.5, 2.0, 0.05),
            (p.round(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-9, "seed {seed}: {value}");
        }
        if !styles.contains(&p.style()) {
            styles.push(p.style());
        }
    }
    assert_eq!(styles.len(), 6, "{styles:?}");
}

#[test]
fn zig_is_a_still() {
    assert_eq!(Tool::Zig.frames(), 1);
    let recipe = derive(7, ZIG);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, ZIG);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = ZigParams::default();
    assert_eq!(
        error(p.set_width(2.05)),
        "zig: width 2.05 is outside 0.5..=2"
    );
    assert_eq!(
        error(p.set_depth(0.1)),
        "zig: depth 0.1 is outside 0.2..=1.4"
    );
    assert_eq!(p, ZigParams::default());
    p.set_style(ZigStyle::Scales);
    p.set_round(0.0).unwrap();
    assert_eq!((p.style(), p.round()), (ZigStyle::Scales, 0.0));
}

#[test]
fn every_style_paints_exactly_two_inks_of_the_palette() {
    let inks = [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]];
    let palette = Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap();
    for style in [
        ZigStyle::Teeth,
        ZigStyle::Chevron,
        ZigStyle::Stairs,
        ZigStyle::Ricrac,
        ZigStyle::Waves,
        ZigStyle::Scales,
    ] {
        let mut recipe = derive(7, ZIG);
        recipe.set_palette(palette.clone()).unwrap();
        let Params::Zig(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_style(style);
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        let drawn = inks
            .iter()
            .filter(|ink| image.rgba().chunks(4).any(|px| px[..3] == ink[..]))
            .count();
        assert_eq!(drawn, 2, "{style:?}");
    }
}
