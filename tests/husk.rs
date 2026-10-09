use tirage::{Bite, Frame, HuskParams, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const HUSK: ToolPin = ToolPin::Tool(Tool::Husk);

fn husk(recipe: &Recipe) -> &HuskParams {
    let Params::Husk(params) = recipe.params() else {
        panic!("not a husk Recipe");
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
            r##"{"tirage":0,"tool":"husk","tool_seed":9611518,"palette":["#e0c3fc","#1b1b1e","#f9f871"],"params":{"count":20,"size":0.92,"vary":0.96,"lump":0.92,"bites":"Crumble","eat":0.82,"tex":0.08,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"husk","tool_seed":3724278653,"palette":["#e0c3fc","#1b1b1e","#f9f871"],"params":{"count":29,"size":0.13,"vary":0.19,"lump":0.9,"bites":"Crumble","eat":0.97,"tex":0.69,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"husk","tool_seed":3565986084,"palette":["#e0c3fc","#1b1b1e","#f9f871"],"params":{"count":37,"size":0.45,"vary":0.3,"lump":0.43,"bites":"Crumble","eat":0.39,"tex":0.52,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, HUSK).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, HUSK);
        let p = husk(&recipe);
        assert_eq!(
            recipe.palette(),
            &Palette::from_hex(&["#e0c3fc", "#1b1b1e", "#f9f871"]).unwrap()
        );
        assert_eq!((p.bite(), p.grain()), (Bite::Crumble, 0.3), "seed {seed}");
        assert!(
            (18..=70).contains(&p.count()),
            "seed {seed}: count {}",
            p.count()
        );
        for (value, min, max) in [
            (p.size(), 0.1, 1.0),
            (p.vary(), 0.0, 1.0),
            (p.lump(), 0.0, 1.0),
            (p.eat(), 0.2, 1.0),
            (p.tex(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn husk_is_a_still() {
    assert_eq!(Tool::Husk.frames(), 1);
    let recipe = derive(7, HUSK);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, HUSK);
    let Params::Husk(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_bite(Bite::Dots);
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = HuskParams::default();
    assert_eq!(error(p.set_count(0)), "husk: count 0 is outside 1..=70");
    assert_eq!(error(p.set_count(71)), "husk: count 71 is outside 1..=70");
    assert_eq!(error(p.set_size(1.2)), "husk: size 1.2 is outside 0..=1");
    assert_eq!(error(p.set_eat(f64::NAN)), "husk: eat NaN is outside 0..=1");
    assert_eq!(
        error(p.set_grain(-0.1)),
        "husk: grain -0.1 is outside 0..=1"
    );
    assert_eq!(
        error(p.grain_pass_mut().set_amount(2.0)),
        "grain: grnAmount 2 is outside 0..=1"
    );
    assert_eq!(p, HuskParams::default());
}

#[test]
fn recipe_json_rejects_unknown_bites_and_keys() {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, HUSK).to_json()).unwrap();
    value["params"]["bites"] = "Chew".into();
    assert_eq!(
        error(Recipe::from_json(&value.to_string())),
        "Recipe JSON: params: unknown variant `Chew`, expected `Crumble` or `Dots`"
    );
    value["params"]["bites"] = "Dots".into();
    value["params"]["level"] = 0.5.into();
    assert!(
        error(Recipe::from_json(&value.to_string()))
            .starts_with("Recipe JSON: params: unknown field `level`")
    );
}

#[test]
fn a_two_ink_palette_fills_with_the_silhouette() {
    let mut recipe = derive(7, HUSK);
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    let Params::Husk(p) = recipe.params() else {
        unreachable!()
    };
    let spread = (p.grain() * 40.0).ceil() as u8;
    assert!(
        image
            .rgba()
            .chunks(4)
            .all(|px| px[1] <= spread && px[2] <= spread && px[3] == 255)
    );
}
