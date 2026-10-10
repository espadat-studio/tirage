use tirage::{
    ChaffParams, ChaffShape, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const CHAFF: ToolPin = ToolPin::Tool(Tool::Chaff);

fn chaff(recipe: &Recipe) -> &ChaffParams {
    let Params::Chaff(params) = recipe.params() else {
        panic!("not a chaff Recipe");
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
            r##"{"tirage":0,"tool":"chaff","tool_seed":9611518,"palette":["#0e3b43","#f2c14e"],"params":{"count":91,"size":0.4,"vary":0.99,"apart":0.36,"shapes":"Leaf","curve":0.79,"slim":0.95,"taper":0.67,"mottle":0.66,"coarse":0.42,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"chaff","tool_seed":3724278653,"palette":["#0e3b43","#f2c14e"],"params":{"count":69,"size":0.17,"vary":0.27,"apart":0.96,"shapes":"Leaf","curve":0.12,"slim":0.81,"taper":0.13,"mottle":0.27,"coarse":0.53,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"chaff","tool_seed":3565986084,"palette":["#0e3b43","#f2c14e"],"params":{"count":86,"size":0.91,"vary":0.8,"apart":0.84,"shapes":"Crescent","curve":0.05,"slim":0.77,"taper":0.14,"mottle":0.62,"coarse":0.69,"grain":0.3,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CHAFF).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let on_grid = |value: f64| value == (value * 100.0).round() / 100.0;
    for seed in 0..2000 {
        let recipe = derive(seed, CHAFF);
        let p = chaff(&recipe);
        assert!((4..=250).contains(&p.count()), "seed {seed}");
        assert!((0.1..=1.0).contains(&p.size()), "seed {seed}");
        assert!((0.0..=0.75).contains(&p.mottle()), "seed {seed}");
        assert_eq!(p.grain(), 0.3, "seed {seed}");
        for value in [
            p.size(),
            p.vary(),
            p.apart(),
            p.curve(),
            p.slim(),
            p.taper(),
            p.mottle(),
            p.coarse(),
        ] {
            assert!((0.0..=1.0).contains(&value), "seed {seed}: {value}");
            assert!(on_grid(value), "seed {seed}: {value}");
        }
    }
}

#[test]
fn every_shape_is_dealt() {
    let shapes: Vec<ChaffShape> = (0..60)
        .map(|seed| chaff(&derive(seed, CHAFF)).shapes())
        .collect();
    for shape in [ChaffShape::Crescent, ChaffShape::Leaf, ChaffShape::Bar] {
        assert!(shapes.contains(&shape), "{shape:?} never dealt");
    }
}

#[test]
fn chaff_is_a_still() {
    assert_eq!(Tool::Chaff.frames(), 1);
    let recipe = derive(7, CHAFF);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CHAFF);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = ChaffParams::default();
    assert_eq!(error(p.set_count(3)), "chaff: count 3 is outside 4..=400");
    assert_eq!(error(p.set_slim(1.01)), "chaff: slim 1.01 is outside 0..=1");
    assert_eq!(p, ChaffParams::default());
    p.set_count(400).unwrap();
    p.set_shapes(ChaffShape::Bar);
    assert_eq!((p.count(), p.shapes()), (400, ChaffShape::Bar));
}

#[test]
fn a_palette_with_more_than_two_inks_is_rejected() {
    let mut recipe = derive(1, CHAFF);
    let three = Palette::from_hex(&["#000000", "#ff0000", "#ffffff"]).unwrap();
    assert!(recipe.set_palette(three).is_err());
}

#[test]
fn without_grain_every_pixel_is_the_ground_or_the_ink() {
    let mut recipe = derive(3, CHAFF);
    let Params::Chaff(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_grain(0.0).unwrap();
    recipe
        .set_palette(Palette::from_hex(&["#000000", "#ff0000"]).unwrap())
        .unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let (mut ground, mut ink) = (0, 0);
    for px in image.rgba().chunks(4) {
        match [px[0], px[1], px[2]] {
            [0, 0, 0] => ground += 1,
            [255, 0, 0] => ink += 1,
            other => panic!("{other:?} is neither ink"),
        }
    }
    assert!(ground > 0 && ink > 0);
}
