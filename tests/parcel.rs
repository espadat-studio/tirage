use tirage::{
    Frame, LineBlend, Palette, Params, ParcelParams, Recipe, Tool, ToolPin, derive, render,
};

const PARCEL: ToolPin = ToolPin::Tool(Tool::Parcel);

fn parcel(recipe: &Recipe) -> &ParcelParams {
    let Params::Parcel(params) = recipe.params() else {
        panic!("not a parcel Recipe");
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
            r##"{"tirage":0,"tool":"parcel","tool_seed":9611518,"palette":["#e9f5db","#e63946","#2b2d42"],"params":{"cells":15,"cover":0.48,"chunk":1.0,"grids":0,"blends":"Multiply","ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"parcel","tool_seed":3724278653,"palette":["#e9f5db","#e63946","#2b2d42"],"params":{"cells":28,"cover":0.32,"chunk":1.4,"grids":0,"blends":"Multiply","ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"parcel","tool_seed":3565986084,"palette":["#e9f5db","#e63946","#2b2d42"],"params":{"cells":23,"cover":0.53,"chunk":1.4,"grids":4,"blends":"Multiply","ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, PARCEL).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, PARCEL);
        let p = parcel(&recipe);
        assert!((8..=28).contains(&p.cells()), "seed {seed}");
        assert!(p.grids() <= 8, "seed {seed}");
        assert!((0.2..=0.8).contains(&p.cover()), "seed {seed}");
        assert_eq!(
            p.cover(),
            (p.cover() * 100.0).round() / 100.0,
            "seed {seed}"
        );
        assert!((0.5..=2.0).contains(&p.chunk()), "seed {seed}");
        assert_eq!(p.chunk(), (p.chunk() * 20.0).round() / 20.0, "seed {seed}");
        assert_eq!(p.blend(), LineBlend::Multiply, "seed {seed}");
    }
}

#[test]
fn parcel_is_a_still() {
    assert_eq!(Tool::Parcel.frames(), 1);
    let recipe = derive(7, PARCEL);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, PARCEL);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = ParcelParams::default();
    assert_eq!(error(p.set_cells(7)), "parcel: cells 7 is outside 8..=28");
    assert_eq!(
        error(p.set_cover(0.1)),
        "parcel: cover 0.1 is outside 0.2..=0.8"
    );
    assert_eq!(
        error(p.set_chunk(2.05)),
        "parcel: chunk 2.05 is outside 0.5..=2"
    );
    assert_eq!(error(p.set_grids(9)), "parcel: grids 9 is outside 0..=8");
    assert_eq!(p, ParcelParams::default());
    p.set_cells(28).unwrap();
    p.set_blend(LineBlend::Normal);
    assert_eq!((p.cells(), p.blend()), (28, LineBlend::Normal));
}

#[test]
fn the_palette_is_capped_at_its_three_roles() {
    let mut recipe = derive(7, PARCEL);
    let four = Palette::from_hex(&["#000000", "#ff0000", "#00ff00", "#0000ff"]).unwrap();
    assert_eq!(
        error(recipe.set_palette(four)),
        "parcel: palette has 4 inks, parcel draws at most 3"
    );
}

fn inks_drawn(blend: LineBlend) -> Vec<[u8; 3]> {
    let mut recipe = derive(7, PARCEL);
    recipe
        .set_palette(Palette::from_hex(&["#ffffff", "#00ffff", "#ff0000"]).unwrap())
        .unwrap();
    let Params::Parcel(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_grids(8).unwrap();
    params.set_blend(blend);
    let image = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
    let mut inks: Vec<[u8; 3]> = Vec::new();
    for px in image.rgba().chunks(4) {
        let ink = [px[0], px[1], px[2]];
        if !inks.contains(&ink) {
            inks.push(ink);
        }
    }
    inks
}

#[test]
fn multiplied_lines_darken_the_blocks_they_cross() {
    let inks = inks_drawn(LineBlend::Multiply);
    for ink in [[255, 255, 255], [0, 255, 255], [255, 0, 0], [0, 0, 0]] {
        assert!(inks.contains(&ink), "{ink:?} never drawn");
    }
}

#[test]
fn normal_lines_cover_the_blocks_they_cross() {
    let inks = inks_drawn(LineBlend::Normal);
    assert!(inks.contains(&[255, 0, 0]));
    assert!(!inks.contains(&[0, 0, 0]));
}
