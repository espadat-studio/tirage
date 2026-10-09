use tirage::{
    Blend, Flow, Frame, Palette, Params, Recipe, Tool, ToolPin, VeinParams, derive, render,
};

const VEIN: ToolPin = ToolPin::Tool(Tool::Vein);

const DEFAULT_PALETTE: [&str; 8] = [
    "#0e1a2b", "#2e63b8", "#7fa8e0", "#e9dcc3", "#f2892b", "#e0362f", "#2fa39a", "#8b5a2b",
];

fn vein(recipe: &Recipe) -> &VeinParams {
    let Params::Vein(params) = recipe.params() else {
        panic!("not a vein Recipe");
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
            r##"{"tirage":0,"tool":"vein","tool_seed":9611518,"palette":["#0e1a2b","#2e63b8","#7fa8e0","#e9dcc3","#f2892b","#e0362f","#2fa39a","#8b5a2b"],"params":{"flows":"Marble","scale":0.37,"curve":0.62,"levels":9,"tiger":0.87,"edges":0.67,"stars":0.96,"tints":0.23,"runs":0.54,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.4,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.25}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"vein","tool_seed":3724278653,"palette":["#0e1a2b","#2e63b8","#7fa8e0","#e9dcc3","#f2892b","#e0362f","#2fa39a","#8b5a2b"],"params":{"flows":"Swirl","scale":0.33,"curve":0.09,"levels":9,"tiger":0.49,"edges":0.71,"stars":0.69,"tints":0.11,"runs":0.66,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.4,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.25}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"vein","tool_seed":3565986084,"palette":["#0e1a2b","#2e63b8","#7fa8e0","#e9dcc3","#f2892b","#e0362f","#2fa39a","#8b5a2b"],"params":{"flows":"Ripple","scale":0.58,"curve":0.31,"levels":9,"tiger":0.06,"edges":0.69,"stars":0.85,"tints":0.63,"runs":0.6,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Overlay","grnAmount":0.4,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.25}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, VEIN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut flows = std::collections::BTreeSet::new();
    for seed in 0..2000 {
        let recipe = derive(seed, VEIN);
        let p = vein(&recipe);
        assert_eq!(
            recipe.palette(),
            &Palette::from_hex(&DEFAULT_PALETTE).unwrap()
        );
        flows.insert(format!("{:?}", p.flow()));
        for (value, min, max) in [
            (p.scale(), 0.25, 0.75),
            (p.curve(), 0.0, 1.0),
            (p.tiger(), 0.0, 1.0),
            (p.edges(), 0.37, 0.75),
            (p.stars(), 0.0, 1.0),
            (p.tints(), 0.0, 1.0),
            (p.runs(), 0.5, 0.75),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(
            (9..=11).contains(&p.levels()),
            "seed {seed}: levels {}",
            p.levels()
        );
    }
    assert_eq!(
        flows.into_iter().collect::<Vec<_>>(),
        ["Marble", "Ripple", "Swirl"]
    );
}

#[test]
fn derive_opens_with_the_sites_printed_grain() {
    let recipe = derive(7, VEIN);
    let grain = vein(&recipe).grain_pass();
    assert!(grain.on());
    assert_eq!(
        (
            grain.blend(),
            grain.amount(),
            grain.size(),
            grain.specks(),
            grain.vignette()
        ),
        (Blend::Overlay, 0.4, 1.0, 0.4, 0.25)
    );
    assert!(!vein(&recipe).dither().on());
}

#[test]
fn tool_seed_is_never_zero() {
    assert!((0..10_000).all(|seed| derive(seed, VEIN).tool_seed().get() != 0));
}

#[test]
fn recipe_round_trips_through_json() {
    let mut recipe = derive(7, VEIN);
    let Params::Vein(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_flow(Flow::Ripple);
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = VeinParams::default();
    assert_eq!(error(p.set_scale(1.4)), "vein: scale 1.4 is outside 0..=1");
    assert_eq!(error(p.set_levels(2)), "vein: levels 2 is outside 3..=14");
    assert_eq!(error(p.set_levels(15)), "vein: levels 15 is outside 3..=14");
    assert_eq!(
        error(p.set_runs(f64::NAN)),
        "vein: runs NaN is outside 0..=1"
    );
    assert_eq!(
        error(p.set_edges(-0.1)),
        "vein: edges -0.1 is outside 0..=1"
    );
    assert_eq!(p, VeinParams::default());
    p.set_levels(14).unwrap();
    assert_eq!(p.levels(), 14);
}

type Edit = fn(&mut serde_json::Value);

fn json(edit: impl FnOnce(&mut serde_json::Value)) -> Result<Recipe, tirage::Error> {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, VEIN).to_json()).unwrap();
    edit(&mut value);
    Recipe::from_json(&value.to_string())
}

#[test]
fn recipe_json_errors_are_human() {
    let cases: [(&str, Edit); 4] = [
        (
            "Recipe JSON: params: unknown field `stripes`, expected one of `flows`, `scale`, `curve`, `levels`, `tiger`, `edges`, `stars`, `tints`, `runs`, `ditherTog`, `dthKinds`, `dthSize`, `dthLevels`, `dthAmount`, `grainTog`, `grnBlends`, `grnAmount`, `grnSize`, `grnSpecks`, `grnVignette`",
            |v| v["params"]["stripes"] = 1.into(),
        ),
        ("vein: curve 2 is outside 0..=1", |v| {
            v["params"]["curve"] = 2.0.into()
        }),
        (
            "Recipe JSON: params: unknown variant `Pour`, expected one of `Marble`, `Swirl`, `Ripple`",
            |v| v["params"]["flows"] = "Pour".into(),
        ),
        ("Recipe JSON: params: missing field `runs`", |v| {
            v["params"].as_object_mut().unwrap().remove("runs");
        }),
    ];
    for (message, edit) in cases {
        assert_eq!(error(json(edit)), message);
    }
}

#[test]
fn vein_is_a_still() {
    assert_eq!(Tool::Vein.frames(), 1);
    let recipe = derive(7, VEIN);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn render_gives_an_opaque_image_of_the_frame_size_for_every_flow() {
    for flow in [Flow::Marble, Flow::Swirl, Flow::Ripple] {
        let mut recipe = derive(7, VEIN);
        let Params::Vein(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_flow(flow);
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        assert_eq!(
            (image.width(), image.height(), image.rgba().len()),
            (90, 160, 90 * 160 * 4)
        );
        assert!(image.rgba().chunks(4).all(|px| px[3] == 255), "{flow:?}");
    }
}
