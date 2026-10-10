use tirage::{
    Frame, Palette, Params, QuiltParams, QuiltStyle, Recipe, Tool, ToolPin, derive, render,
};

const QUILT: ToolPin = ToolPin::Tool(Tool::Quilt);

const STYLES: [QuiltStyle; 15] = [
    QuiltStyle::Auto,
    QuiltStyle::Bands,
    QuiltStyle::Tabs,
    QuiltStyle::Plaid,
    QuiltStyle::Dither,
    QuiltStyle::Steps,
    QuiltStyle::Zigzag,
    QuiltStyle::Diamond,
    QuiltStyle::Cross,
    QuiltStyle::Basket,
    QuiltStyle::Rings,
    QuiltStyle::Star,
    QuiltStyle::Waves,
    QuiltStyle::Gingham,
    QuiltStyle::Burst,
];

fn quilt(recipe: &Recipe) -> &QuiltParams {
    let Params::Quilt(params) = recipe.params() else {
        panic!("not a quilt Recipe");
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
            r##"{"tirage":0,"tool":"quilt","tool_seed":9611518,"palette":["#f3e9dc","#2a6f97","#c8553d","#f4d35e"],"params":{"styles":"Burst","cells":48,"chunk":1.55,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"quilt","tool_seed":3724278653,"palette":["#f3e9dc","#2a6f97","#c8553d","#f4d35e"],"params":{"styles":"Zigzag","cells":30,"chunk":1.2,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"quilt","tool_seed":3565986084,"palette":["#f3e9dc","#2a6f97","#c8553d","#f4d35e"],"params":{"styles":"Rings","cells":54,"chunk":1.55,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, QUILT).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let mut styles = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, QUILT);
        let p = quilt(&recipe);
        for (value, min, max, step) in [
            (f64::from(p.cells()), 28.0, 72.0, 2.0),
            (p.chunk(), 0.7, 1.8, 0.05),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            let steps = (value - min) / step;
            assert!((steps - steps.round()).abs() < 1e-6, "seed {seed}: {value}");
        }
        if !styles.contains(&p.style()) {
            styles.push(p.style());
        }
    }
    assert_eq!(styles.len(), 15, "{styles:?}");
}

#[test]
fn quilt_is_a_still() {
    assert_eq!(Tool::Quilt.frames(), 1);
    let recipe = derive(7, QUILT);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, QUILT);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = QuiltParams::default();
    assert_eq!(error(p.set_cells(74)), "quilt: cells 74 is outside 28..=72");
    assert_eq!(
        error(p.set_chunk(0.65)),
        "quilt: chunk 0.65 is outside 0.7..=1.8"
    );
    assert_eq!(p, QuiltParams::default());
    p.set_style(QuiltStyle::Gingham);
    p.set_cells(28).unwrap();
    assert_eq!((p.style(), p.cells()), (QuiltStyle::Gingham, 28));
}

#[test]
fn a_fifth_ink_is_refused() {
    let mut recipe = derive(7, QUILT);
    let five = Palette::from_hex(&["#000000", "#111111", "#222222", "#333333", "#444444"]).unwrap();
    assert!(recipe.set_palette(five).is_err());
}

#[test]
fn every_style_paints_only_palette_inks_and_the_base() {
    let inks = [[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]];
    let palette = Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap();
    for style in STYLES {
        let mut recipe = derive(7, QUILT);
        recipe.set_palette(palette.clone()).unwrap();
        let Params::Quilt(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_style(style);
        let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
        assert!(
            image
                .rgba()
                .chunks(4)
                .all(|px| inks.iter().any(|ink| px[..3] == ink[..])),
            "{style:?}"
        );
        assert!(
            image.rgba().chunks(4).any(|px| px[..3] == inks[0]),
            "{style:?}"
        );
    }
}
