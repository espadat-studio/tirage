use tirage::{Frame, Palette, Params, Recipe, SpecimenParams, Tool, ToolPin, derive, render};

const SPECIMEN: ToolPin = ToolPin::Tool(Tool::Specimen);

fn specimen(recipe: &Recipe) -> &SpecimenParams {
    let Params::Specimen(params) = recipe.params() else {
        panic!("not a specimen Recipe");
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
            r##"{"tirage":0,"tool":"specimen","tool_seed":9611518,"palette":["#e63946","#457b9d","#f4a261","#2a9d8f","#e9c46a","#8338ec","#1b1b1b"],"params":{"cols":5,"fill":0.19,"span":0.37,"gut":0.0,"rules":0.21,"wMarble":56,"wTendril":1,"wLoop":62,"wFlat":5,"wRing":92,"wFan":62,"wArc":23,"wSpike":21,"mScale":2.7,"mBands":7,"density":0.6,"lineW":1.2,"inkMix":0.2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"specimen","tool_seed":3724278653,"palette":["#e63946","#457b9d","#f4a261","#2a9d8f","#e9c46a","#8338ec","#1b1b1b"],"params":{"cols":8,"fill":0.79,"span":0.02,"gut":0.045,"rules":0.42,"wMarble":100,"wTendril":92,"wLoop":0,"wFlat":0,"wRing":70,"wFan":17,"wArc":0,"wSpike":4,"mScale":3.4,"mBands":3,"density":0.56,"lineW":7.0,"inkMix":0.83,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"specimen","tool_seed":3565986084,"palette":["#e63946","#457b9d","#f4a261","#2a9d8f","#e9c46a","#8338ec","#1b1b1b"],"params":{"cols":11,"fill":0.79,"span":0.83,"gut":0.16,"rules":0.35,"wMarble":15,"wTendril":65,"wLoop":63,"wFlat":91,"wRing":29,"wFan":85,"wArc":46,"wSpike":70,"mScale":4.2,"mBands":5,"density":0.77,"lineW":10.0,"inkMix":0.62,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, SPECIMEN).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, SPECIMEN);
        let p = specimen(&recipe);
        assert!((2..=16).contains(&p.cols()), "seed {seed}");
        assert!((2..=7).contains(&p.m_bands()), "seed {seed}");
        for weight in [
            p.w_marble(),
            p.w_tendril(),
            p.w_loop(),
            p.w_flat(),
            p.w_ring(),
            p.w_fan(),
            p.w_arc(),
            p.w_spike(),
        ] {
            assert!(weight <= 100, "seed {seed}");
        }
        for (value, min, max, unit) in [
            (p.fill(), 0.1, 1.0, 100.0),
            (p.span(), 0.0, 1.0, 100.0),
            (p.gut(), 0.0, 0.3, 200.0),
            (p.rules(), 0.0, 1.0, 100.0),
            (p.m_scale(), 0.6, 6.0, 10.0),
            (p.density(), 0.1, 1.0, 100.0),
            (p.line_w(), 0.4, 10.0, 5.0),
            (p.ink_mix(), 0.0, 1.0, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        assert!(!p.grain().on(), "seed {seed}");
    }
}

#[test]
fn specimen_is_a_still() {
    assert_eq!(Tool::Specimen.frames(), 1);
    let recipe = derive(7, SPECIMEN);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, SPECIMEN);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = SpecimenParams::default();
    assert_eq!(error(p.set_cols(1)), "specimen: cols 1 is outside 2..=16");
    assert_eq!(
        error(p.set_gut(0.31)),
        "specimen: gut 0.31 is outside 0..=0.3"
    );
    assert_eq!(
        error(p.set_line_w(0.2)),
        "specimen: lineW 0.2 is outside 0.4..=10"
    );
    assert_eq!(p, SpecimenParams::default());
    p.set_cols(16).unwrap();
    p.set_w_spike(100).unwrap();
    assert_eq!((p.cols(), p.w_spike()), (16, 100));
}

#[test]
fn page_and_ink_stay_fixed_whatever_the_palette() {
    let mut recipe = derive(2, SPECIMEN);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let inks: Vec<[u8; 3]> = image
        .rgba()
        .chunks(4)
        .map(|px| [px[0], px[1], px[2]])
        .collect();
    for ink in [
        [0xf5, 0xf1, 0xe8],
        [0x1b, 0x1b, 0x1b],
        [255, 0, 0],
        [0, 255, 0],
    ] {
        assert!(inks.contains(&ink), "{ink:?} never drawn");
    }
}
