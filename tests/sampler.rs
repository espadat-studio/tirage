use tirage::{Frame, Palette, Params, Recipe, SamplerParams, Tool, ToolPin, derive, render};

const SAMPLER: ToolPin = ToolPin::Tool(Tool::Sampler);

fn sampler(recipe: &Recipe) -> &SamplerParams {
    let Params::Sampler(params) = recipe.params() else {
        panic!("not a sampler Recipe");
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
            r##"{"tirage":0,"tool":"sampler","tool_seed":9611518,"palette":["#101010","#d6ff3c","#6f6a1c","#ff7a1f","#f4786e","#c9c4ca","#a2218e"],"params":{"grid":6,"bands":11,"mix":0.66,"turn":0.38,"density":0.94,"weight":0.17,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"sampler","tool_seed":3724278653,"palette":["#101010","#d6ff3c","#6f6a1c","#ff7a1f","#f4786e","#c9c4ca","#a2218e"],"params":{"grid":36,"bands":11,"mix":0.48,"turn":0.29,"density":0.85,"weight":0.73,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"sampler","tool_seed":3565986084,"palette":["#101010","#d6ff3c","#6f6a1c","#ff7a1f","#f4786e","#c9c4ca","#a2218e"],"params":{"grid":33,"bands":3,"mix":0.09,"turn":0.84,"density":0.75,"weight":0.44,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, SAMPLER).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, SAMPLER);
        let p = sampler(&recipe);
        assert!((4..=48).contains(&p.grid()), "seed {seed}");
        assert!((1..=14).contains(&p.bands()), "seed {seed}");
        for (value, min, max) in [
            (p.mix(), 0.0, 1.0),
            (p.turn(), 0.0, 1.0),
            (p.density(), 0.55, 1.0),
            (p.weight(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
    }
}

#[test]
fn sampler_is_a_still() {
    assert_eq!(Tool::Sampler.frames(), 1);
    let recipe = derive(7, SAMPLER);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, SAMPLER);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = SamplerParams::default();
    assert_eq!(error(p.set_grid(3)), "sampler: grid 3 is outside 4..=48");
    assert_eq!(
        error(p.set_bands(15)),
        "sampler: bands 15 is outside 1..=14"
    );
    assert_eq!(
        error(p.set_density(0.05)),
        "sampler: density 0.05 is outside 0.1..=1"
    );
    assert_eq!(p, SamplerParams::default());
    p.set_grid(48).unwrap();
    p.set_density(0.1).unwrap();
    assert_eq!((p.grid(), p.density()), (48, 0.1));
}

#[test]
fn every_palette_ink_can_show() {
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for seed in 0..12 {
        let mut recipe = derive(seed, SAMPLER);
        recipe
            .set_palette(Palette::from_hex(&["#ff0000", "#000000", "#ffffff"]).unwrap())
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
