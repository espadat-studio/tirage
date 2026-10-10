use tirage::{
    Frame, Palette, Params, Recipe, SpliceCut, SpliceParams, Tool, ToolPin, derive, render,
};

const SPLICE: ToolPin = ToolPin::Tool(Tool::Splice);

fn splice(recipe: &Recipe) -> &SpliceParams {
    let Params::Splice(params) = recipe.params() else {
        panic!("not a splice Recipe");
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
            r##"{"tirage":0,"tool":"splice","tool_seed":9611518,"palette":["#1b1b1b","#ff6b35","#004e89","#f7c59f","#efefd0","#1a936f","#ff3cac"],"params":{"dirs":"Columns","slices":35,"shift":0.23,"scale":0.76,"tones":3,"key":0.69,"screen":0.25,"mix":0.15,"grain":0.18,"arcs":17,"weight":0.79,"bend":0.44,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"splice","tool_seed":3724278653,"palette":["#1b1b1b","#ff6b35","#004e89","#f7c59f","#efefd0","#1a936f","#ff3cac"],"params":{"dirs":"Rows","slices":9,"shift":0.1,"scale":0.37,"tones":7,"key":0.0,"screen":0.3,"mix":0.37,"grain":0.18,"arcs":16,"weight":0.68,"bend":0.29,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"splice","tool_seed":3565986084,"palette":["#1b1b1b","#ff6b35","#004e89","#f7c59f","#efefd0","#1a936f","#ff3cac"],"params":{"dirs":"Rows","slices":34,"shift":0.35,"scale":0.93,"tones":4,"key":0.43,"screen":0.27,"mix":0.8,"grain":0.18,"arcs":14,"weight":0.13,"bend":0.68,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, SPLICE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut cuts = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, SPLICE);
        let p = splice(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.slices()), 1.0, 40.0, 1.0),
            (p.shift(), 0.0, 1.0, 0.01),
            (p.scale(), 0.0, 1.0, 0.01),
            (f64::from(p.tones()), 2.0, 14.0, 1.0),
            (p.key(), 0.0, 1.0, 0.01),
            (p.screen(), 0.0, 1.0, 0.01),
            (p.mix(), 0.0, 1.0, 0.01),
            (f64::from(p.arcs()), 0.0, 24.0, 1.0),
            (p.weight(), 0.0, 1.0, 0.01),
            (p.bend(), 0.0, 1.0, 0.01),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-9, "seed {seed}: {value}");
        }
        assert_eq!(p.grain(), 0.18, "seed {seed}");
        if !cuts.contains(&p.cut()) {
            cuts.push(p.cut());
        }
    }
    assert_eq!(cuts.len(), 3, "{cuts:?}");
}

#[test]
fn splice_is_a_still() {
    assert_eq!(Tool::Splice.frames(), 1);
    let recipe = derive(7, SPLICE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, SPLICE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = SpliceParams::default();
    assert_eq!(
        error(p.set_slices(41)),
        "splice: slices 41 is outside 1..=40"
    );
    assert_eq!(error(p.set_tones(1)), "splice: tones 1 is outside 2..=14");
    assert_eq!(error(p.set_arcs(25)), "splice: arcs 25 is outside 0..=24");
    assert_eq!(
        error(p.set_bend(1.01)),
        "splice: bend 1.01 is outside 0..=1"
    );
    assert_eq!(p, SpliceParams::default());
    p.set_cut(SpliceCut::Blocks);
    p.set_arcs(0).unwrap();
    assert_eq!((p.cut(), p.arcs()), (SpliceCut::Blocks, 0));
}

#[test]
fn every_cut_prints_more_than_one_ink_and_arcs_blend_over_the_field() {
    let palette = Palette::from_hex(&["#000000", "#ff0000", "#0000ff"]).unwrap();
    for cut in [SpliceCut::Columns, SpliceCut::Rows, SpliceCut::Blocks] {
        let mut recipe = derive(7, SPLICE);
        recipe.set_palette(palette.clone()).unwrap();
        let Params::Splice(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_cut(cut);
        params.set_slices(6).unwrap();
        params.set_grain(0.0).unwrap();
        params.set_arcs(0).unwrap();
        let frame = Frame::new(&recipe, 270, 480, 0).unwrap();
        let flat = render(&recipe, &frame);
        let inks = [[0, 0, 0], [255, 0, 0], [0, 0, 255]];
        assert!(
            flat.rgba()
                .chunks(4)
                .all(|px| inks.contains(&[px[0], px[1], px[2]])),
            "{cut:?} paints only Palette inks"
        );
        let Params::Splice(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_arcs(24).unwrap();
        params.set_weight(1.0).unwrap();
        let arced = render(&recipe, &frame);
        assert!(
            arced
                .rgba()
                .chunks(4)
                .any(|px| !inks.contains(&[px[0], px[1], px[2]])),
            "{cut:?} arcs blend at alpha"
        );
    }
}
