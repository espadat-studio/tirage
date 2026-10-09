use tirage::{
    AuraParams, AuraStyle, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const AURA: ToolPin = ToolPin::Tool(Tool::Aura);

fn aura(recipe: &Recipe) -> &AuraParams {
    let Params::Aura(params) = recipe.params() else {
        panic!("not an aura Recipe");
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
            r##"{"tirage":0,"tool":"aura","tool_seed":9611518,"palette":["#ff8a5b","#ffc15e","#f4a7d6","#8e7cff"],"params":{"styles":"Auto","scale":1.2,"churn":0.63,"punch":0.79,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"aura","tool_seed":3724278653,"palette":["#ff8a5b","#ffc15e","#f4a7d6","#8e7cff"],"params":{"styles":"Clouds","scale":1.6,"churn":0.93,"punch":0.36,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"aura","tool_seed":3565986084,"palette":["#ff8a5b","#ffc15e","#f4a7d6","#8e7cff"],"params":{"styles":"Mesh","scale":1.7,"churn":0.55,"punch":0.99,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, AURA).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut styles = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, AURA);
        let p = aura(&recipe);
        assert_eq!(
            recipe.palette(),
            &Palette::from_hex(&["#ff8a5b", "#ffc15e", "#f4a7d6", "#8e7cff"]).unwrap()
        );
        for (value, min, max, unit) in [
            (p.scale(), 0.9, 2.0, 20.0),
            (p.churn(), 0.25, 1.0, 100.0),
            (p.punch(), 0.25, 1.0, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        if !styles.contains(&p.style()) {
            styles.push(p.style());
        }
    }
    assert_eq!(styles.len(), 4, "{styles:?}");
}

#[test]
fn aura_is_a_still() {
    assert_eq!(Tool::Aura.frames(), 1);
    let recipe = derive(7, AURA);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, AURA);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = AuraParams::default();
    assert_eq!(
        error(p.set_scale(0.4)),
        "aura: scale 0.4 is outside 0.5..=2"
    );
    assert_eq!(error(p.set_churn(1.1)), "aura: churn 1.1 is outside 0..=1");
    assert_eq!(
        error(p.set_punch(f64::NAN)),
        "aura: punch NaN is outside 0..=1"
    );
    assert_eq!(p, AuraParams::default());
    p.set_style(AuraStyle::Mesh);
    p.set_scale(0.5).unwrap();
    assert_eq!((p.style(), p.scale()), (AuraStyle::Mesh, 0.5));
}

#[test]
fn recipe_json_errors_are_human() {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, AURA).to_json()).unwrap();
    value["params"]["styles"] = "Plasma".into();
    assert_eq!(
        error(Recipe::from_json(&value.to_string())),
        "Recipe JSON: params: unknown variant `Plasma`, expected one of `Auto`, `Clouds`, `Mesh`, `Sweep`"
    );
    value["params"]["styles"] = "Mesh".into();
    value["params"]["punch"] = 2.into();
    assert_eq!(
        error(Recipe::from_json(&value.to_string())),
        "aura: punch 2 is outside 0..=1"
    );
}

#[test]
fn a_frame_wider_than_the_buffer_is_scaled_up_smooth() {
    let recipe = derive(1, AURA);
    let image = render(&recipe, &Frame::new(&recipe, 1400, 2000, 0).unwrap());
    let rgba = image.rgba();
    assert_eq!(rgba.len(), 1400 * 2000 * 4);
    assert!(rgba.chunks(4).all(|px| px[3] == 255));
    let row = &rgba[..1400 * 4];
    let steps = row
        .chunks(4)
        .zip(row.chunks(4).skip(1))
        .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(x, y)| x.abs_diff(*y) > 4))
        .count();
    assert_eq!(steps, 0, "an upscaled row has hard steps");
}

#[test]
fn aura_is_appended_to_the_registry() {
    assert_eq!(Tool::ALL.last(), Some(&Tool::Aura));
    assert_eq!(Tool::from_slug("aura").unwrap(), Tool::Aura);
}
