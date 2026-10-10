use tirage::{
    FeteMotif, FeteParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const FETE: ToolPin = ToolPin::Tool(Tool::Fete);

fn fete(recipe: &Recipe) -> &FeteParams {
    let Params::Fete(params) = recipe.params() else {
        panic!("not a fete Recipe");
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
            r##"{"tirage":0,"tool":"fete","tool_seed":9611518,"palette":["#7b2cbf","#ff6d00","#00bbf9","#ffd60a"],"params":{"motifs":"Rings","scale":3.9,"res":104,"lw":1.7,"dots":52,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"fete","tool_seed":3724278653,"palette":["#7b2cbf","#ff6d00","#00bbf9","#ffd60a"],"params":{"motifs":"Atom","scale":5.3,"res":80,"lw":1.0,"dots":58,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"fete","tool_seed":3565986084,"palette":["#7b2cbf","#ff6d00","#00bbf9","#ffd60a"],"params":{"motifs":"Atom","scale":3.7,"res":36,"lw":1.6,"dots":2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, FETE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut motifs = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, FETE);
        let p = fete(&recipe);
        for (value, min, max, step) in [
            (p.scale(), 1.2, 6.0, 0.1),
            (f64::from(p.res()), 36.0, 120.0, 4.0),
            (p.lw(), 0.4, 2.2, 0.05),
            (f64::from(p.dots()), 0.0, 80.0, 2.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-9, "seed {seed}: {value}");
        }
        if !motifs.contains(&p.motif()) {
            motifs.push(p.motif());
        }
    }
    assert_eq!(motifs.len(), 7, "{motifs:?}");
}

#[test]
fn fete_is_a_still() {
    assert_eq!(Tool::Fete.frames(), 1);
    let recipe = derive(7, FETE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, FETE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = FeteParams::default();
    assert_eq!(
        error(p.set_scale(1.1)),
        "fete: scale 1.1 is outside 1.2..=6"
    );
    assert_eq!(error(p.set_res(124)), "fete: res 124 is outside 36..=120");
    assert_eq!(error(p.set_lw(2.25)), "fete: lw 2.25 is outside 0.4..=2.2");
    assert_eq!(error(p.set_dots(82)), "fete: dots 82 is outside 0..=80");
    assert_eq!(p, FeteParams::default());
    p.set_motif(FeteMotif::Wave);
    p.set_dots(0).unwrap();
    assert_eq!((p.motif(), p.dots()), (FeteMotif::Wave, 0));
}

#[test]
fn every_motif_draws_a_white_line_and_black_dots_over_the_palette() {
    let palette = Palette::from_hex(&["#ff0000", "#00ff00"]).unwrap();
    for motif in [
        FeteMotif::Auto,
        FeteMotif::Rings,
        FeteMotif::Spiral,
        FeteMotif::Burst,
        FeteMotif::Atom,
        FeteMotif::Wave,
        FeteMotif::Scribble,
    ] {
        let mut recipe = derive(7, FETE);
        recipe.set_palette(palette.clone()).unwrap();
        let Params::Fete(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_motif(motif);
        params.set_dots(80).unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        for ink in [[255, 0, 0], [0, 255, 0], [255, 255, 255], [10, 10, 10]] {
            assert!(
                image.rgba().chunks(4).any(|px| px[..3] == ink),
                "{motif:?} lacks {ink:?}"
            );
        }
    }
}
