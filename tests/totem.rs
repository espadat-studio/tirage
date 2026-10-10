use tirage::{Frame, Palette, Params, Recipe, Tool, ToolPin, TotemParams, derive, render};

const TOTEM: ToolPin = ToolPin::Tool(Tool::Totem);

fn totem(recipe: &Recipe) -> &TotemParams {
    let Params::Totem(params) = recipe.params() else {
        panic!("not a totem Recipe");
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
            r##"{"tirage":0,"tool":"totem","tool_seed":9611518,"palette":["#1b1b1b","#ff6b35","#f7b32b","#2e86ab","#f72585"],"params":{"border":0.055,"mat":0.5,"matGrain":1,"keyline":5,"regions":16,"grain":69,"mirror":0.77,"variety":0.52,"core":0.5,"coreRings":0,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"totem","tool_seed":3724278653,"palette":["#1b1b1b","#ff6b35","#f7b32b","#2e86ab","#f72585"],"params":{"border":0.1,"mat":0.54,"matGrain":2,"keyline":3,"regions":26,"grain":81,"mirror":0.08,"variety":0.25,"core":0.4,"coreRings":7,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"totem","tool_seed":3565986084,"palette":["#1b1b1b","#ff6b35","#f7b32b","#2e86ab","#f72585"],"params":{"border":0.24,"mat":0.05,"matGrain":3,"keyline":4,"regions":18,"grain":76,"mirror":0.8,"variety":0.14,"core":0.35,"coreRings":5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, TOTEM).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let on_grid = |value: f64, unit: f64| value == (value * unit).round() / unit;
    for seed in 0..2000 {
        let recipe = derive(seed, TOTEM);
        let p = totem(&recipe);
        assert!((60..=220).contains(&p.grain()), "seed {seed}");
        assert!((3..=30).contains(&p.regions()), "seed {seed}");
        assert!((1..=6).contains(&p.mat_grain()), "seed {seed}");
        assert!(p.keyline() <= 6 && p.core_rings() <= 8, "seed {seed}");
        assert!((0.0..=0.3).contains(&p.border()), "seed {seed}");
        assert!(on_grid(p.border(), 200.0), "seed {seed}: {}", p.border());
        for (value, max) in [
            (p.mat(), 0.7),
            (p.mirror(), 1.0),
            (p.variety(), 1.0),
            (p.core(), 0.6),
        ] {
            assert!((0.0..=max).contains(&value), "seed {seed}: {value}");
            assert!(on_grid(value, 100.0), "seed {seed}: {value}");
        }
    }
}

#[test]
fn totem_is_a_still() {
    assert_eq!(Tool::Totem.frames(), 1);
    let recipe = derive(7, TOTEM);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, TOTEM);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = TotemParams::default();
    assert_eq!(
        error(p.set_grain(15)),
        "totem: grain 15 is outside 16..=220"
    );
    assert_eq!(
        error(p.set_border(0.41)),
        "totem: border 0.41 is outside 0..=0.4"
    );
    assert_eq!(
        error(p.set_core_rings(9)),
        "totem: coreRings 9 is outside 0..=8"
    );
    assert_eq!(p, TotemParams::default());
    p.set_grain(220).unwrap();
    p.set_mirror(0.0).unwrap();
    assert_eq!((p.grain(), p.mirror()), (220, 0.0));
}

fn mirrored_columns(image: &tirage::Image) -> bool {
    let (w, rgba) = (image.width() as usize, image.rgba());
    (0..image.height() as usize).all(|y| {
        (0..w / 2).all(|x| {
            let at = |x: usize| &rgba[(y * w + x) * 4..(y * w + x) * 4 + 3];
            at(x) == at(w - 1 - x)
        })
    })
}

#[test]
fn a_full_mirror_with_no_mat_or_core_reflects_the_frame_across_its_centre() {
    let mut recipe = derive(3, TOTEM);
    let Params::Totem(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_mat(0.0).unwrap();
    params.set_mirror(1.0).unwrap();
    params.set_grain(60).unwrap();
    params.set_variety(0.0).unwrap();
    params.set_core(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 540, 960, 0).unwrap());
    assert!(mirrored_columns(&image));
}

#[test]
fn every_palette_ink_can_show() {
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for seed in 0..12 {
        let mut recipe = derive(seed, TOTEM);
        recipe
            .set_palette(Palette::from_hex(&["#000000", "#ff0000", "#ffffff"]).unwrap())
            .unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        for px in image.rgba().chunks(4) {
            let ink = [px[0], px[1], px[2]];
            if !seen.contains(&ink) {
                seen.push(ink);
            }
        }
    }
    for ink in [[0, 0, 0], [255, 0, 0], [255, 255, 255]] {
        assert!(seen.contains(&ink), "{ink:?} never drawn");
    }
}
