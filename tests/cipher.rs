use tirage::{
    CipherField, CipherParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const CIPHER: ToolPin = ToolPin::Tool(Tool::Cipher);

fn cipher(recipe: &Recipe) -> &CipherParams {
    let Params::Cipher(params) = recipe.params() else {
        panic!("not a cipher Recipe");
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
            r##"{"tirage":0,"tool":"cipher","tool_seed":9611518,"palette":["#2a0a14","#dce0f0","#f5d20a","#7cc4f0","#f58a1e","#e8412c","#2e6bc0"],"params":{"cells":128,"bands":5,"fields":"Relief","wave":0.74,"tilt":-0.89,"detail":0.24,"size":0.52,"edges":0.5,"flecks":0.13,"smears":0.48,"dots":0.4,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"cipher","tool_seed":3724278653,"palette":["#2a0a14","#dce0f0","#f5d20a","#7cc4f0","#f58a1e","#e8412c","#2e6bc0"],"params":{"cells":39,"bands":5,"fields":"Relief","wave":0.82,"tilt":0.11,"detail":0.24,"size":0.87,"edges":0.14,"flecks":0.16,"smears":0.38,"dots":0.66,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"cipher","tool_seed":3565986084,"palette":["#2a0a14","#dce0f0","#f5d20a","#7cc4f0","#f58a1e","#e8412c","#2e6bc0"],"params":{"cells":94,"bands":6,"fields":"Figure","wave":0.52,"tilt":-0.55,"detail":0.96,"size":0.89,"edges":0.71,"flecks":0.21,"smears":0.3,"dots":0.21,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CIPHER).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut fields = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, CIPHER);
        let p = cipher(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.cells()), 20.0, 140.0, 1.0),
            (f64::from(p.bands()), 2.0, 7.0, 1.0),
            (p.wave(), 0.0, 1.0, 0.01),
            (p.tilt(), -1.0, 1.0, 0.01),
            (p.detail(), 0.0, 1.0, 0.01),
            (p.size(), 0.48, 1.0, 0.01),
            (p.edges(), 0.0, 1.0, 0.01),
            (p.flecks(), 0.0, 0.3, 0.01),
            (p.smears(), 0.0, 0.6, 0.01),
            (p.dots(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-6, "seed {seed}: {value}");
        }
        if !fields.contains(&p.field()) {
            fields.push(p.field());
        }
    }
    assert_eq!(fields.len(), 3, "{fields:?}");
}

#[test]
fn cipher_is_a_still() {
    assert_eq!(Tool::Cipher.frames(), 1);
    let recipe = derive(7, CIPHER);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CIPHER);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = CipherParams::default();
    assert_eq!(
        error(p.set_cells(141)),
        "cipher: cells 141 is outside 20..=140"
    );
    assert_eq!(error(p.set_bands(1)), "cipher: bands 1 is outside 2..=7");
    assert_eq!(
        error(p.set_tilt(-1.01)),
        "cipher: tilt -1.01 is outside -1..=1"
    );
    assert_eq!(
        error(p.set_flecks(0.31)),
        "cipher: flecks 0.31 is outside 0..=0.3"
    );
    assert_eq!(p, CipherParams::default());
    p.set_field(CipherField::Slope);
    p.set_bands(2).unwrap();
    assert_eq!((p.field(), p.bands()), (CipherField::Slope, 2));
}

#[test]
fn every_field_draws_the_ground_and_marks_from_the_palette() {
    let palette = Palette::from_hex(&["#000000", "#ff0000", "#00ff00"]).unwrap();
    for field in [CipherField::Figure, CipherField::Relief, CipherField::Slope] {
        let mut recipe = derive(7, CIPHER);
        recipe.set_palette(palette.clone()).unwrap();
        let Params::Cipher(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_field(field);
        params.set_bands(4).unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        for ink in [[0, 0, 0], [255, 0, 0], [0, 255, 0]] {
            assert!(
                image.rgba().chunks(4).any(|px| px[..3] == ink),
                "{field:?} lacks {ink:?}"
            );
        }
    }
}
