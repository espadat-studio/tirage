use tirage::{
    Frame, Palette, Params, Recipe, StrandParams, StrandTexture, Tool, ToolPin, derive, render,
};

const STRAND: ToolPin = ToolPin::Tool(Tool::Strand);

fn strand(recipe: &Recipe) -> &StrandParams {
    let Params::Strand(params) = recipe.params() else {
        panic!("not a strand Recipe");
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
            r##"{"tirage":0,"tool":"strand","tool_seed":9611518,"palette":["#ffd166","#1b1b1b","#ef476f"],"params":{"count":4,"len":9,"wander":0.26,"branch":0.9,"thick":0.68,"rod":0.65,"notch":0.98,"rough":0.03,"offset":0.68,"edge":0.92,"tex":0.43,"texKinds":"Screen","grain":0.42,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"strand","tool_seed":3724278653,"palette":["#ffd166","#1b1b1b","#ef476f"],"params":{"count":14,"len":32,"wander":0.02,"branch":0.3,"thick":0.31,"rod":0.75,"notch":0.22,"rough":0.83,"offset":0.98,"edge":0.59,"tex":0.38,"texKinds":"Drag","grain":0.42,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"strand","tool_seed":3565986084,"palette":["#ffd166","#1b1b1b","#ef476f"],"params":{"count":17,"len":16,"wander":0.01,"branch":0.52,"thick":0.06,"rod":0.51,"notch":0.47,"rough":0.59,"offset":0.77,"edge":0.53,"tex":0.91,"texKinds":"Drag","grain":0.42,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, STRAND).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, STRAND);
        let p = strand(&recipe);
        assert!((4..=20).contains(&p.count()), "seed {seed}");
        assert!((6..=32).contains(&p.len()), "seed {seed}");
        assert!((0.0..=0.8).contains(&p.thick()), "seed {seed}");
        for value in [
            p.wander(),
            p.branch(),
            p.thick(),
            p.rod(),
            p.notch(),
            p.rough(),
            p.offset(),
            p.edge(),
            p.tex(),
        ] {
            assert!((0.0..=1.0).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert_eq!(p.grain(), 0.42, "seed {seed}");
        assert!(!p.grain_pass().on(), "seed {seed}");
    }
}

#[test]
fn strand_is_a_still() {
    assert_eq!(Tool::Strand.frames(), 1);
    let recipe = derive(7, STRAND);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, STRAND);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = StrandParams::default();
    assert_eq!(error(p.set_count(0)), "strand: count 0 is outside 1..=40");
    assert_eq!(error(p.set_len(91)), "strand: len 91 is outside 3..=90");
    assert_eq!(error(p.set_tex(1.01)), "strand: tex 1.01 is outside 0..=1");
    assert_eq!(p, StrandParams::default());
    p.set_count(40).unwrap();
    p.set_texture(StrandTexture::Screen);
    assert_eq!((p.count(), p.texture()), (40, StrandTexture::Screen));
}

#[test]
fn a_four_ink_palette_is_refused() {
    let mut recipe = derive(7, STRAND);
    let four = Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap();
    assert!(recipe.set_palette(four).is_err());
}

#[test]
fn without_grain_every_pixel_is_a_palette_ink() {
    let mut recipe = derive(7, STRAND);
    let hex = ["#000000", "#ff0000", "#00ff00"];
    recipe
        .set_palette(Palette::from_hex(&hex).unwrap())
        .unwrap();
    let Params::Strand(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_grain(0.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let inks = [[0, 0, 0], [255, 0, 0], [0, 255, 0]];
    for ink in inks {
        assert!(
            image.rgba().chunks(4).any(|px| px[..3] == ink),
            "{ink:?} is missing"
        );
    }
    assert!(
        image
            .rgba()
            .chunks(4)
            .all(|px| inks.contains(&[px[0], px[1], px[2]]))
    );
}
