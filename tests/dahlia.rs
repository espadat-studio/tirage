use tirage::{DahliaParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const DAHLIA: ToolPin = ToolPin::Tool(Tool::Dahlia);

fn dahlia(recipe: &Recipe) -> &DahliaParams {
    let Params::Dahlia(params) = recipe.params() else {
        panic!("not a dahlia Recipe");
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
            r##"{"tirage":0,"tool":"dahlia","tool_seed":9611518,"palette":["#f6eedc","#3fd3f0","#ff6fd8","#fff23a","#f52a2a"],"params":{"rays":277,"size":0.96,"ragged":0.84,"cx":0.59,"cy":0.49,"width":0.75,"dash":0.37,"gap":0.57,"bend":0.24,"caps":0.08,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.22,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.15}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"dahlia","tool_seed":3724278653,"palette":["#f6eedc","#3fd3f0","#ff6fd8","#fff23a","#f52a2a"],"params":{"rays":110,"size":0.21,"ragged":0.53,"cx":0.27,"cy":0.23,"width":0.15,"dash":0.6,"gap":0.41,"bend":0.28,"caps":0.46,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.22,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.15}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"dahlia","tool_seed":3565986084,"palette":["#f6eedc","#3fd3f0","#ff6fd8","#fff23a","#f52a2a"],"params":{"rays":62,"size":0.11,"ragged":0.22,"cx":0.47,"cy":0.59,"width":0.34,"dash":0.02,"gap":0.12,"bend":0.85,"caps":0.64,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":true,"grnBlends":"Multiply","grnAmount":0.22,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.15}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, DAHLIA).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, DAHLIA);
        let p = dahlia(&recipe);
        assert!((8..=300).contains(&p.rays()), "seed {seed}");
        for (value, min, max) in [
            (p.size(), 0.0, 1.0),
            (p.ragged(), 0.0, 1.0),
            (p.cx(), 0.1, 0.9),
            (p.cy(), 0.1, 0.9),
            (p.width(), 0.0, 1.0),
            (p.dash(), 0.0, 1.0),
            (p.gap(), 0.0, 1.0),
            (p.bend(), 0.0, 1.0),
            (p.caps(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!(p.grain().on(), "seed {seed}");
    }
}

#[test]
fn dahlia_is_a_still() {
    assert_eq!(Tool::Dahlia.frames(), 1);
    let recipe = derive(7, DAHLIA);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, DAHLIA);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = DahliaParams::default();
    assert_eq!(error(p.set_rays(7)), "dahlia: rays 7 is outside 8..=300");
    assert_eq!(
        error(p.set_cx(0.05)),
        "dahlia: cx 0.05 is outside 0.1..=0.9"
    );
    assert_eq!(
        error(p.set_caps(1.01)),
        "dahlia: caps 1.01 is outside 0..=1"
    );
    assert_eq!(p, DahliaParams::default());
    p.set_rays(300).unwrap();
    p.set_bend(1.0).unwrap();
    assert_eq!((p.rays(), p.bend()), (300, 1.0));
}

fn inks_drawn(palette: &[&str]) -> Vec<[u8; 3]> {
    let mut recipe = derive(7, DAHLIA);
    recipe
        .set_palette(Palette::from_hex(palette).unwrap())
        .unwrap();
    let Params::Dahlia(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.grain_mut().set_on(false);
    params.set_caps(1.0).unwrap();
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

#[test]
fn paper_ray_inks_and_accent_beads_all_show() {
    let inks = inks_drawn(&["#000000", "#ff0000", "#00ff00", "#0000ff"]);
    for ink in [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(inks.contains(&ink), "{ink:?} never drawn");
    }
}

#[test]
fn two_inks_draw_the_rays_in_the_accent() {
    let inks = inks_drawn(&["#000000", "#ff0000"]);
    assert!(inks.contains(&[255, 0, 0]));
    assert!(inks.iter().all(|&[_, g, b]| g == 0 && b == 0));
}
