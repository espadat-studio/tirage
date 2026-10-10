use tirage::{Frame, Palette, Params, Recipe, RisoParams, Tool, ToolPin, derive, render};

const RISO: ToolPin = ToolPin::Tool(Tool::Riso);

fn riso(recipe: &Recipe) -> &RisoParams {
    let Params::Riso(params) = recipe.params() else {
        panic!("not a riso Recipe");
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
            r##"{"tirage":0,"tool":"riso","tool_seed":9611518,"palette":["#3a86ff","#ff006e","#fb5607","#1b1b2f","#ffbe0b","#f7f3ea"],"params":{"bands":2,"rough":0.85,"scrib":0.69,"dotp":0.81,"grain":0.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"riso","tool_seed":3724278653,"palette":["#3a86ff","#ff006e","#fb5607","#1b1b2f","#ffbe0b","#f7f3ea"],"params":{"bands":5,"rough":0.96,"scrib":0.21,"dotp":0.16,"grain":0.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"riso","tool_seed":3565986084,"palette":["#3a86ff","#ff006e","#fb5607","#1b1b2f","#ffbe0b","#f7f3ea"],"params":{"bands":4,"rough":0.35,"scrib":0.95,"dotp":0.53,"grain":0.5,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, RISO).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, RISO);
        let p = riso(&recipe);
        assert!((2..=5).contains(&p.bands()), "seed {seed}");
        for (value, min, max) in [
            (p.rough(), 0.1, 1.0),
            (p.scrib(), 0.0, 1.0),
            (p.dotp(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert_eq!(p.grain(), 0.5, "seed {seed}");
    }
}

#[test]
fn riso_is_a_still() {
    assert_eq!(Tool::Riso.frames(), 1);
    let recipe = derive(7, RISO);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, RISO);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = RisoParams::default();
    assert_eq!(error(p.set_bands(6)), "riso: bands 6 is outside 2..=5");
    assert_eq!(
        error(p.set_rough(0.05)),
        "riso: rough 0.05 is outside 0.1..=1"
    );
    assert_eq!(
        error(p.set_grain(1.01)),
        "riso: grain 1.01 is outside 0..=1"
    );
    assert_eq!(p, RisoParams::default());
    p.set_bands(5).unwrap();
    p.set_dotp(1.0).unwrap();
    assert_eq!((p.bands(), p.dotp()), (5, 1.0));
}

#[test]
fn every_palette_ink_can_show() {
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for seed in 0..12 {
        let mut recipe = derive(seed, RISO);
        recipe
            .set_palette(Palette::from_hex(&["#000000", "#ff0000", "#ffffff"]).unwrap())
            .unwrap();
        let Params::Riso(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_grain(0.0).unwrap();
        params.set_scrib(0.0).unwrap();
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
