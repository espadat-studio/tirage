use tirage::{Frame, Palette, Params, Recipe, Tool, ToolPin, VeeParams, VeeStyle, derive, render};

const VEE: ToolPin = ToolPin::Tool(Tool::Vee);

fn vee(recipe: &Recipe) -> &VeeParams {
    let Params::Vee(params) = recipe.params() else {
        panic!("not a vee Recipe");
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
            r##"{"tirage":0,"tool":"vee","tool_seed":9611518,"palette":["#f4efe3","#d62828","#003049","#1b1b1b"],"params":{"styles":"Cross","angle":26,"width":0.95,"weight":0.39,"bands":4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"vee","tool_seed":3724278653,"palette":["#f4efe3","#d62828","#003049","#1b1b1b"],"params":{"styles":"Chevron","angle":24,"width":0.7,"weight":0.75,"bands":1,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"vee","tool_seed":3565986084,"palette":["#f4efe3","#d62828","#003049","#1b1b1b"],"params":{"styles":"Diagonal","angle":58,"width":1.6,"weight":0.54,"bands":2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, VEE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, VEE);
        let p = vee(&recipe);
        assert!((15..=75).contains(&p.angle()), "seed {seed}");
        assert!((1..=4).contains(&p.bands()), "seed {seed}");
        assert!((0.4..=2.2).contains(&p.width()), "seed {seed}");
        assert_eq!(p.width(), (p.width() * 20.0).round() / 20.0, "seed {seed}");
        assert!((0.15..=0.85).contains(&p.weight()), "seed {seed}");
        assert_eq!(
            p.weight(),
            (p.weight() * 100.0).round() / 100.0,
            "seed {seed}"
        );
        assert!(!p.grain().on(), "seed {seed}");
    }
}

#[test]
fn vee_is_a_still() {
    assert_eq!(Tool::Vee.frames(), 1);
    let recipe = derive(7, VEE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, VEE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = VeeParams::default();
    assert_eq!(error(p.set_angle(76)), "vee: angle 76 is outside 15..=75");
    assert_eq!(
        error(p.set_width(0.35)),
        "vee: width 0.35 is outside 0.4..=2.2"
    );
    assert_eq!(error(p.set_bands(0)), "vee: bands 0 is outside 1..=4");
    assert_eq!(p, VeeParams::default());
    p.set_style(VeeStyle::Quad);
    p.set_weight(0.85).unwrap();
    assert_eq!((p.style(), p.weight()), (VeeStyle::Quad, 0.85));
}

#[test]
fn a_palette_longer_than_four_inks_is_refused() {
    let mut recipe = derive(1, VEE);
    let five = Palette::from_hex(&["#000000", "#111111", "#222222", "#333333", "#444444"]);
    assert!(recipe.set_palette(five.unwrap()).is_err());
}

#[test]
fn the_cross_ink_shows_only_where_two_stripe_sets_meet() {
    let inks = |style: VeeStyle| {
        let mut recipe = derive(3, VEE);
        recipe
            .set_palette(Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap())
            .unwrap();
        let Params::Vee(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_style(style);
        params.set_bands(1).unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        let mut seen: Vec<[u8; 3]> = Vec::new();
        for px in image.rgba().chunks(4) {
            let ink = [px[0], px[1], px[2]];
            if !seen.contains(&ink) {
                seen.push(ink);
            }
        }
        seen
    };
    let cross = inks(VeeStyle::Cross);
    for ink in [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(cross.contains(&ink), "cross never drew {ink:?}");
    }
    let chevron = inks(VeeStyle::Chevron);
    assert!(chevron.contains(&[0, 0, 0]));
    assert_eq!(
        [[255, 0, 0], [0, 255, 0], [0, 0, 255]]
            .iter()
            .filter(|ink| chevron.contains(ink))
            .count(),
        1,
        "a chevron band prints one stripe ink"
    );
}
