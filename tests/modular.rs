use tirage::{Frame, ModularParams, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const MODULAR: ToolPin = ToolPin::Tool(Tool::Modular);

fn modular(recipe: &Recipe) -> &ModularParams {
    let Params::Modular(params) = recipe.params() else {
        panic!("not a modular Recipe");
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
            r##"{"tirage":0,"tool":"modular","tool_seed":9611518,"palette":["#e63946","#1d3557","#f1c40f","#1b1b1b"],"params":{"gcols":3,"unit":11,"merge":0.99,"wEmpty":59,"wSolid":82,"wBlocks":13,"wDots":87,"wLines":60,"wGrad":60,"blockFill":0.76,"dot":0.99,"rules":0.39,"ruleW":1.0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"modular","tool_seed":3724278653,"palette":["#e63946","#1d3557","#f1c40f","#1b1b1b"],"params":{"gcols":7,"unit":7,"merge":0.46,"wEmpty":82,"wSolid":48,"wBlocks":58,"wDots":87,"wLines":82,"wGrad":51,"blockFill":0.12,"dot":0.49,"rules":0.1,"ruleW":0.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"modular","tool_seed":3565986084,"palette":["#e63946","#1d3557","#f1c40f","#1b1b1b"],"params":{"gcols":2,"unit":3,"merge":0.66,"wEmpty":21,"wSolid":27,"wBlocks":54,"wDots":0,"wLines":38,"wGrad":33,"blockFill":0.1,"dot":0.9,"rules":0.68,"ruleW":2.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, MODULAR).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, MODULAR);
        let p = modular(&recipe);
        assert!((2..=12).contains(&p.gcols()), "seed {seed}");
        assert!((2..=12).contains(&p.unit()), "seed {seed}");
        for weight in [
            p.w_empty(),
            p.w_solid(),
            p.w_blocks(),
            p.w_dots(),
            p.w_lines(),
            p.w_grad(),
        ] {
            assert!(weight <= 100, "seed {seed}: {weight}");
        }
        for (value, min, max) in [
            (p.merge(), 0.0, 1.0),
            (p.block_fill(), 0.1, 0.9),
            (p.dot(), 0.2, 1.0),
            (p.rules(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!((0.5..=4.0).contains(&p.rule_w()), "seed {seed}");
        assert_eq!(p.rule_w(), (p.rule_w() * 2.0).round() / 2.0, "seed {seed}");
        assert!(!p.grain().on() && !p.dither().on(), "seed {seed}");
    }
}

#[test]
fn modular_is_a_still() {
    assert_eq!(Tool::Modular.frames(), 1);
    let recipe = derive(7, MODULAR);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, MODULAR);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = ModularParams::default();
    assert_eq!(
        error(p.set_gcols(13)),
        "modular: gcols 13 is outside 2..=12"
    );
    assert_eq!(
        error(p.set_w_grad(101)),
        "modular: wGrad 101 is outside 0..=100"
    );
    assert_eq!(
        error(p.set_block_fill(0.05)),
        "modular: blockFill 0.05 is outside 0.1..=0.9"
    );
    assert_eq!(
        error(p.set_rule_w(1.25)),
        "modular: ruleW 1.25 is not a slider value, 0.5..=4 step 0.5"
    );
    assert_eq!(p, ModularParams::default());
    p.set_gcols(12).unwrap();
    p.set_rule_w(4.0).unwrap();
    assert_eq!((p.gcols(), p.rule_w()), (12, 4.0));
}

fn inks_drawn(edit: impl FnOnce(&mut ModularParams)) -> Vec<[u8; 3]> {
    let mut recipe = derive(7, MODULAR);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let Params::Modular(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_gcols(8).unwrap();
    params.set_rules(0.0).unwrap();
    edit(params);
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let mut inks: Vec<[u8; 3]> = Vec::new();
    for px in image.rgba().chunks(4) {
        let ink = [px[0], px[1], px[2]];
        if !inks.contains(&ink) {
            inks.push(ink);
        }
    }
    inks
}

fn only(weight: fn(&mut ModularParams, u32) -> Result<(), tirage::Error>) -> Vec<[u8; 3]> {
    inks_drawn(|p| {
        for set in [
            ModularParams::set_w_empty,
            ModularParams::set_w_solid,
            ModularParams::set_w_blocks,
            ModularParams::set_w_dots,
            ModularParams::set_w_lines,
            ModularParams::set_w_grad,
        ] {
            set(p, 0).unwrap();
        }
        weight(p, 1).unwrap();
    })
}

#[test]
fn every_weight_zero_draws_only_the_ground() {
    let inks = only(|_, _| Ok(()));
    assert_eq!(inks, [[0xf4, 0xef, 0xe3]]);
}

#[test]
fn solid_modules_draw_every_ink_flat() {
    let inks = only(ModularParams::set_w_solid);
    for ink in [[255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(inks.contains(&ink), "{ink:?} never drawn");
    }
    assert!(inks.len() <= 4, "{inks:?}");
}

#[test]
fn gradient_modules_ramp_between_two_inks() {
    let inks = only(ModularParams::set_w_grad);
    assert!(inks.len() > 20, "{} tones", inks.len());
}

#[test]
fn rules_draw_over_the_grid_at_their_opacity() {
    let bare = inks_drawn(|_| {});
    let ruled = inks_drawn(|p| p.set_rules(1.0).unwrap());
    assert!(!bare.contains(&[0x1b, 0x1b, 0x1b]));
    assert!(ruled.contains(&[0x1b, 0x1b, 0x1b]));
}
