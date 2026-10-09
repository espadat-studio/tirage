use tirage::{Frame, Palette, Params, Recipe, SearParams, Tool, ToolPin, derive, render};

const SEAR: ToolPin = ToolPin::Tool(Tool::Sear);

fn sear(recipe: &Recipe) -> &SearParams {
    let Params::Sear(params) = recipe.params() else {
        panic!("not a sear Recipe");
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
            r##"{"tirage":0,"tool":"sear","tool_seed":9611518,"palette":["#03071e","#370617","#9d0208","#e85d04","#faa307","#ffe066"],"params":{"scale":0.26,"warp":0.76,"detail":1,"smear":0.36,"drag":0.85,"tear":0.15,"steps":12,"grain":0.41,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"sear","tool_seed":3724278653,"palette":["#03071e","#370617","#9d0208","#e85d04","#faa307","#ffe066"],"params":{"scale":0.83,"warp":0.14,"detail":1,"smear":0.59,"drag":0.71,"tear":0.58,"steps":23,"grain":0.48,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"sear","tool_seed":3565986084,"palette":["#03071e","#370617","#9d0208","#e85d04","#faa307","#ffe066"],"params":{"scale":0.41,"warp":0.43,"detail":3,"smear":0.82,"drag":0.89,"tear":0.19,"steps":6,"grain":0.57,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, SEAR).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, SEAR);
        let p = sear(&recipe);
        assert!((1..=5).contains(&p.detail()), "seed {seed}");
        assert!((2..=24).contains(&p.steps()), "seed {seed}");
        for (value, max) in [
            (p.scale(), 1.0),
            (p.warp(), 1.0),
            (p.smear(), 1.0),
            (p.drag(), 1.0),
            (p.tear(), 0.6),
            (p.grain(), 0.7),
        ] {
            assert!((0.0..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn sear_is_a_still() {
    assert_eq!(Tool::Sear.frames(), 1);
    let recipe = derive(7, SEAR);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, SEAR);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = SearParams::default();
    assert_eq!(error(p.set_steps(1)), "sear: steps 1 is outside 2..=24");
    assert_eq!(error(p.set_detail(6)), "sear: detail 6 is outside 1..=5");
    assert_eq!(p, SearParams::default());
    p.set_tear(1.0).unwrap();
    assert_eq!(p.tear(), 1.0);
}

#[test]
fn two_steps_print_only_the_ends_of_the_ramp() {
    let mut recipe = derive(7, SEAR);
    let mut value: serde_json::Value = serde_json::from_str(&recipe.to_json()).unwrap();
    value["params"]["steps"] = 2.into();
    value["params"]["grain"] = 0.into();
    recipe = Recipe::from_json(&value.to_string()).unwrap();
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#808080", "#ffffff"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    assert!(
        image
            .rgba()
            .chunks(4)
            .all(|px| px[..3] == [0, 0, 0] || px[..3] == [255, 255, 255])
    );
}
