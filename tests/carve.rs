use tirage::{CarveParams, Frame, Palette, Params, Recipe, Tool, ToolPin, derive, render};

const CARVE: ToolPin = ToolPin::Tool(Tool::Carve);

fn carve(recipe: &Recipe) -> &CarveParams {
    let Params::Carve(params) = recipe.params() else {
        panic!("not a carve Recipe");
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
            r##"{"tirage":0,"tool":"carve","tool_seed":9611518,"palette":["#1a1a1a","#f5f2ec","#ff5c00","#00a3ff","#ff00a8","#a8ff00"],"params":{"cuts":14,"uneven":0.86,"gap":0.07,"mix":0.04,"pitch":0.52,"grain":0.5,"nodes":0.79,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"carve","tool_seed":3724278653,"palette":["#1a1a1a","#f5f2ec","#ff5c00","#00a3ff","#ff00a8","#a8ff00"],"params":{"cuts":8,"uneven":0.87,"gap":0.4,"mix":0.64,"pitch":0.5,"grain":0.5,"nodes":0.54,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"carve","tool_seed":3565986084,"palette":["#1a1a1a","#f5f2ec","#ff5c00","#00a3ff","#ff00a8","#a8ff00"],"params":{"cuts":4,"uneven":0.42,"gap":0.54,"mix":0.58,"pitch":0.14,"grain":0.5,"nodes":0.16,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, CARVE).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, CARVE);
        let p = carve(&recipe);
        assert!((1..=16).contains(&p.cuts()), "seed {seed}");
        for value in [p.uneven(), p.gap(), p.mix(), p.pitch(), p.nodes()] {
            assert!((0.0..=1.0).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert_eq!(p.grain(), 0.5, "seed {seed}");
        assert!(!p.grain_pass().on(), "seed {seed}");
    }
}

#[test]
fn carve_is_a_still() {
    assert_eq!(Tool::Carve.frames(), 1);
    let recipe = derive(7, CARVE);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, CARVE);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = CarveParams::default();
    assert_eq!(error(p.set_cuts(0)), "carve: cuts 0 is outside 1..=16");
    assert_eq!(error(p.set_cuts(17)), "carve: cuts 17 is outside 1..=16");
    assert_eq!(
        error(p.set_grain(1.01)),
        "carve: grain 1.01 is outside 0..=1"
    );
    assert_eq!(p, CarveParams::default());
    p.set_cuts(16).unwrap();
    p.set_nodes(1.0).unwrap();
    assert_eq!((p.cuts(), p.nodes()), (16, 1.0));
}

#[test]
fn unpatterned_panels_are_flat_palette_inks() {
    let mut recipe = derive(7, CARVE);
    let hex = ["#000000", "#ff0000", "#00ff00", "#0000ff"];
    recipe
        .set_palette(Palette::from_hex(&hex).unwrap())
        .unwrap();
    let Params::Carve(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_mix(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let inks = [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]];
    let pixels = image.rgba().chunks(4).collect::<Vec<_>>();
    let flat = pixels
        .iter()
        .filter(|px| inks.contains(&[px[0], px[1], px[2]]))
        .count();
    assert!(
        flat * 100 >= pixels.len() * 95,
        "{flat} of {}",
        pixels.len()
    );
}
