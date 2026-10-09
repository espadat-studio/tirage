use tirage::{CoralParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const CORAL: ToolPin = ToolPin::Tool(Tool::Coral);

fn coral(recipe: &Recipe) -> &CoralParams {
    let Params::Coral(params) = recipe.params() else {
        panic!("not a coral Recipe");
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
            r##"{"tirage":0,"tool":"coral","tool_seed":9611518,"palette":["#33081c","#e8449e","#f7b8dc","#45be6e","#c9f24e"],"params":{"branches":12,"spacing":0.51,"spread":0.69,"width":0.31,"wobble":0.4,"cover":0.63,"size":0.56,"beneath":0.47,"ribs":0.96,"dots":0.55,"shadow":0.52,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.45,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.3}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"coral","tool_seed":3724278653,"palette":["#33081c","#e8449e","#f7b8dc","#45be6e","#c9f24e"],"params":{"branches":8,"spacing":0.37,"spread":0.34,"width":0.87,"wobble":0.93,"cover":0.02,"size":0.72,"beneath":0.77,"ribs":0.66,"dots":0.19,"shadow":0.97,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.45,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.3}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"coral","tool_seed":3565986084,"palette":["#33081c","#e8449e","#f7b8dc","#45be6e","#c9f24e"],"params":{"branches":2,"spacing":0.61,"spread":0.8,"width":0.2,"wobble":0.48,"cover":0.65,"size":0.19,"beneath":0.97,"ribs":0.56,"dots":0.95,"shadow":0.93,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.45,"grnSize":1.0,"grnSpecks":0.4,"grnVignette":0.3}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CORAL).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, CORAL);
        let p = coral(&recipe);
        assert!((2..=12).contains(&p.branches()), "seed {seed}");
        for (value, min) in [
            (p.spacing(), 0.0),
            (p.spread(), 0.0),
            (p.width(), 0.1),
            (p.wobble(), 0.0),
            (p.cover(), 0.0),
            (p.size(), 0.0),
            (p.beneath(), 0.0),
            (p.ribs(), 0.0),
            (p.dots(), 0.0),
            (p.shadow(), 0.0),
        ] {
            assert!((min..=1.0).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn the_grain_post_pass_is_on_as_the_site_opens() {
    let grain = CoralParams::default().grain_pass().clone();
    assert!(grain.on());
    assert_eq!(
        (
            grain.blend(),
            grain.amount(),
            grain.specks(),
            grain.vignette()
        ),
        (tirage::Blend::Multiply, 0.45, 0.4, 0.3)
    );
    assert_eq!(coral(&derive(7, CORAL)).grain_pass(), &grain);
}

#[test]
fn coral_is_a_still() {
    assert_eq!(Tool::Coral.frames(), 1);
    let recipe = derive(7, CORAL);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CORAL);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = CoralParams::default();
    assert_eq!(
        error(p.set_branches(13)),
        "coral: branches 13 is outside 2..=12"
    );
    assert_eq!(
        error(p.set_width(0.05)),
        "coral: width 0.05 is outside 0.1..=1"
    );
    assert_eq!(p, CoralParams::default());
    p.set_branches(2).unwrap();
    assert_eq!(p.branches(), 2);
}

#[test]
fn a_two_ink_palette_derives_the_other_roles() {
    let mut recipe = derive(7, CORAL);
    recipe
        .set_palette(Palette::from_hex(&["#101010", "#f0f0f0"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    assert!(image.rgba().chunks(4).all(|px| px[3] == 255));
}
