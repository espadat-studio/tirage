use std::collections::BTreeMap;

use tirage::{Frame, FrondParams, Palette, Params, Plant, Recipe, Tool, ToolPin, derive, render};

const FROND: ToolPin = ToolPin::Tool(Tool::Frond);

const DEFAULT_PALETTE: [&str; 5] = ["#ede4d3", "#ff8c42", "#0b3d91", "#1a1a1a", "#a89f8c"];

fn frond(recipe: &Recipe) -> &FrondParams {
    let Params::Frond(params) = recipe.params() else {
        panic!("not a frond Recipe");
    };
    params
}

fn error<T: std::fmt::Debug>(result: Result<T, tirage::Error>) -> String {
    result.unwrap_err().to_string()
}

fn with_plant(seed: u64, plant: Plant) -> Recipe {
    let mut recipe = derive(seed, FROND);
    let Params::Frond(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_plant(plant);
    recipe
}

#[test]
fn derive_gives_golden_recipes() {
    let golden = [
        (
            1,
            r##"{"tirage":0,"tool":"frond","tool_seed":9611518,"palette":["#ede4d3","#ff8c42","#0b3d91","#1a1a1a","#a89f8c"],"params":{"dirs":"Bouquet","masses":1,"size":0.34,"round":0.76,"growth":0.32,"detail":0.66,"coarse":0.27,"breakup":0.43,"noise":0.77,"patch":0.03,"circles":0.18,"rules":0.01,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"frond","tool_seed":3724278653,"palette":["#ede4d3","#ff8c42","#0b3d91","#1a1a1a","#a89f8c"],"params":{"dirs":"Bouquet","masses":5,"size":0.37,"round":0.89,"growth":0.56,"detail":0.56,"coarse":0.59,"breakup":0.31,"noise":0.49,"patch":0.62,"circles":0.3,"rules":0.76,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"frond","tool_seed":3565986084,"palette":["#ede4d3","#ff8c42","#0b3d91","#1a1a1a","#a89f8c"],"params":{"dirs":"Bouquet","masses":1,"size":0.47,"round":0.66,"growth":0.75,"detail":0.83,"coarse":0.35,"breakup":0.48,"noise":0.82,"patch":0.31,"circles":0.67,"rules":0.03,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, FROND).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut plants = BTreeMap::new();
    for seed in 0..2000 {
        let recipe = derive(seed, FROND);
        let p = frond(&recipe);
        assert_eq!(
            recipe.palette().unwrap(),
            &Palette::from_hex(&DEFAULT_PALETTE).unwrap()
        );
        *plants.entry(format!("{:?}", p.plant())).or_insert(0) += 1;
        for (value, min, max) in [
            (p.size(), 0.0, 0.75),
            (p.round(), 0.25, 1.0),
            (p.growth(), 0.25, 0.75),
            (p.detail(), 0.0, 1.0),
            (p.coarse(), 0.25, 0.75),
            (p.breakup(), 0.25, 0.5),
            (p.noise(), 0.0, 1.0),
            (p.patch(), 0.0, 1.0),
            (p.circles(), 0.0, 1.0),
            (p.rules(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(p.masses() <= 6, "seed {seed}: masses {}", p.masses());
    }
    assert_eq!(
        plants.keys().collect::<Vec<_>>(),
        ["Bouquet", "Fronds", "Potted"]
    );
    assert!(plants.values().all(|&n| n > 500), "{plants:?}");
}

#[test]
fn derive_keeps_grain_and_dither_off() {
    let recipe = derive(7, FROND);
    assert!(!frond(&recipe).grain_pass().on());
    assert!(!frond(&recipe).dither().on());
}

#[test]
fn tool_seed_is_never_zero() {
    assert!((0..10_000).all(|seed| derive(seed, FROND).tool_seed().get() != 0));
}

#[test]
fn recipe_round_trips_through_json() {
    let recipe = with_plant(7, Plant::Fronds);
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = FrondParams::default();
    assert_eq!(error(p.set_size(1.4)), "frond: size 1.4 is outside 0..=1");
    assert_eq!(error(p.set_masses(7)), "frond: masses 7 is outside 0..=6");
    assert_eq!(
        error(p.set_breakup(f64::NAN)),
        "frond: breakup NaN is outside 0..=1"
    );
    assert_eq!(
        error(p.set_rules(-0.1)),
        "frond: rules -0.1 is outside 0..=1"
    );
    assert_eq!(p, FrondParams::default());
    p.set_masses(0).unwrap();
    assert_eq!(p.masses(), 0);
}

type Edit = fn(&mut serde_json::Value);

fn json(edit: impl FnOnce(&mut serde_json::Value)) -> Result<Recipe, tirage::Error> {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, FROND).to_json()).unwrap();
    edit(&mut value);
    Recipe::from_json(&value.to_string())
}

#[test]
fn recipe_json_errors_are_human() {
    let cases: [(&str, Edit); 4] = [
        (
            "Recipe JSON: params: unknown field `leaves`, expected one of `dirs`, `masses`, `size`, `round`, `growth`, `detail`, `coarse`, `breakup`, `noise`, `patch`, `circles`, `rules`, `ditherTog`, `dthKinds`, `dthSize`, `dthLevels`, `dthAmount`, `grainTog`, `grnBlends`, `grnAmount`, `grnSize`, `grnSpecks`, `grnVignette`",
            |v| v["params"]["leaves"] = 1.into(),
        ),
        ("frond: coarse 2 is outside 0..=1", |v| {
            v["params"]["coarse"] = 2.0.into()
        }),
        (
            "Recipe JSON: params: unknown variant `Vase`, expected one of `Potted`, `Bouquet`, `Fronds`",
            |v| v["params"]["dirs"] = "Vase".into(),
        ),
        ("Recipe JSON: params: missing field `rules`", |v| {
            v["params"].as_object_mut().unwrap().remove("rules");
        }),
    ];
    for (message, edit) in cases {
        assert_eq!(error(json(edit)), message);
    }
}

#[test]
fn frond_loops_at_its_site_default_motion() {
    assert_eq!((Tool::Frond.frames(), Tool::Frond.fps()), (36, 12));
    let recipe = derive(7, FROND);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 36)),
        "frame 36 is outside 0..36"
    );
}

#[test]
fn boil_redeals_the_marks_every_frame() {
    let recipe = derive(7, FROND);
    let frame = |t| render(&recipe, &Frame::new(&recipe, 90, 160, t).unwrap());
    assert_eq!(frame(0), frame(0));
    assert_ne!(frame(0), frame(1));
    assert_ne!(frame(1), frame(35));
}

#[test]
fn render_gives_an_opaque_image_of_the_frame_size_for_every_plant() {
    for plant in [Plant::Potted, Plant::Bouquet, Plant::Fronds] {
        let recipe = with_plant(7, plant);
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        assert_eq!(
            (image.width(), image.height(), image.rgba().len()),
            (90, 160, 90 * 160 * 4)
        );
        assert!(image.rgba().chunks(4).all(|px| px[3] == 255), "{plant:?}");
    }
}

#[test]
fn a_two_ink_palette_falls_back_by_role() {
    let mut recipe = with_plant(7, Plant::Fronds);
    recipe
        .set_palette(Palette::from_hex(&["#ffffff", "#000000"]).unwrap())
        .unwrap();
    let Params::Frond(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_masses(0).unwrap();
    params.set_noise(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 180, 320, 0).unwrap());
    assert!(
        image.rgba().chunks(4).any(|px| px[..3] == [0, 0, 0]),
        "marks fall back to the mass ink"
    );
}
