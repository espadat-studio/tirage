use tirage::{
    Frame, OpticParams, OpticStyle, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const OPTIC: ToolPin = ToolPin::Tool(Tool::Optic);

fn optic(recipe: &Recipe) -> &OpticParams {
    let Params::Optic(params) = recipe.params() else {
        panic!("not an optic Recipe");
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
            r##"{"tirage":0,"tool":"optic","tool_seed":9611518,"palette":["#f5f0e6","#7b2cbf"],"params":{"styles":"Peak","count":27,"weight":0.41,"sizeF":0.88,"levels":1,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"optic","tool_seed":3724278653,"palette":["#f5f0e6","#7b2cbf"],"params":{"styles":"Diamond","count":14,"weight":0.42,"sizeF":0.82,"levels":2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"optic","tool_seed":3565986084,"palette":["#f5f0e6","#7b2cbf"],"params":{"styles":"Square","count":16,"weight":0.62,"sizeF":0.74,"levels":2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, OPTIC).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, OPTIC);
        let p = optic(&recipe);
        assert!((6..=28).contains(&p.count()), "seed {seed}");
        assert!((1..=3).contains(&p.levels()), "seed {seed}");
        for (value, min, max) in [(p.weight(), 0.3, 0.7), (p.size_f(), 0.4, 1.0)] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(!p.grain().on(), "seed {seed}");
    }
}

#[test]
fn optic_is_a_still() {
    assert_eq!(Tool::Optic.frames(), 1);
    let recipe = derive(7, OPTIC);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, OPTIC);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = OpticParams::default();
    assert_eq!(error(p.set_count(5)), "optic: count 5 is outside 6..=28");
    assert_eq!(
        error(p.set_weight(0.71)),
        "optic: weight 0.71 is outside 0.3..=0.7"
    );
    assert_eq!(
        error(p.set_size_f(0.39)),
        "optic: sizeF 0.39 is outside 0.4..=1"
    );
    assert_eq!(error(p.set_levels(4)), "optic: levels 4 is outside 1..=3");
    assert_eq!(p, OpticParams::default());
    p.set_levels(3).unwrap();
    p.set_size_f(1.0).unwrap();
    assert_eq!((p.levels(), p.size_f()), (3, 1.0));
}

#[test]
fn a_palette_over_two_inks_is_refused() {
    let three = Palette::from_hex(&["#000000", "#ff0000", "#00ff00"]).unwrap();
    let mut recipe = derive(7, OPTIC);
    assert_eq!(
        error(recipe.set_palette(three)),
        "optic: palette has 3 inks, optic draws at most 2"
    );
    assert_eq!(recipe, derive(7, OPTIC));
}

#[test]
fn the_field_puts_a_bar_on_the_centre_column_outside_every_figure() {
    for styles in [
        OpticStyle::Auto,
        OpticStyle::Diamond,
        OpticStyle::Circle,
        OpticStyle::Peak,
        OpticStyle::Square,
    ] {
        let mut recipe = derive(7, OPTIC);
        recipe
            .set_palette(Palette::from_hex(&["#ff0000", "#0000ff"]).unwrap())
            .unwrap();
        let Params::Optic(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_styles(styles);
        params.set_levels(3).unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        let i = ((2 * 90 + 45) * 4) as usize;
        assert_eq!(&image.rgba()[i..i + 3], [0, 0, 255], "{styles:?}");
    }
}
