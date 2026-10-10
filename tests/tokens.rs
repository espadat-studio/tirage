use tirage::{Frame, Palette, Params, Recipe, TokensParams, Tool, ToolPin, derive, render};

const TOKENS: ToolPin = ToolPin::Tool(Tool::Tokens);

fn tokens(recipe: &Recipe) -> &TokensParams {
    let Params::Tokens(params) = recipe.params() else {
        panic!("not a tokens Recipe");
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
            r##"{"tirage":0,"tool":"tokens","tool_seed":9611518,"palette":["#1b1b1b","#e63946","#3a86ff","#ffbe0b","#2a9d8f","#ff006e","#8338ec","#fb5607","#00b4d8"],"params":{"cols":48,"ruleEvery":6,"grid":0.19,"count":30,"wRun":59,"wBlock":40,"wPlus":99,"wOne":97,"clump":0.49,"align":0.22,"runLen":4,"upright":0.3,"sizeT":0.65,"svar":0.45,"hier":0.03,"square":0.59,"strokeW":4.0,"hollow":0.93,"kin":0.08,"break":0.23,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"tokens","tool_seed":3724278653,"palette":["#1b1b1b","#e63946","#3a86ff","#ffbe0b","#2a9d8f","#ff006e","#8338ec","#fb5607","#00b4d8"],"params":{"cols":43,"ruleEvery":2,"grid":0.53,"count":315,"wRun":82,"wBlock":31,"wPlus":83,"wOne":12,"clump":0.54,"align":0.94,"runLen":2,"upright":0.51,"sizeT":0.48,"svar":0.37,"hier":0.7,"square":0.79,"strokeW":1.5,"hollow":0.9,"kin":0.27,"break":0.29,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"tokens","tool_seed":3565986084,"palette":["#1b1b1b","#e63946","#3a86ff","#ffbe0b","#2a9d8f","#ff006e","#8338ec","#fb5607","#00b4d8"],"params":{"cols":4,"ruleEvery":9,"grid":0.58,"count":205,"wRun":82,"wBlock":53,"wPlus":78,"wOne":100,"clump":0.71,"align":0.62,"runLen":10,"upright":0.19,"sizeT":0.35,"svar":1.0,"hier":0.12,"square":0.08,"strokeW":5.5,"hollow":0.9,"kin":0.62,"break":0.18,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, TOKENS).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, TOKENS);
        let p = tokens(&recipe);
        for (value, min, max) in [
            (p.cols(), 4, 48),
            (p.rule_every(), 1, 10),
            (p.count(), 8, 400),
            (p.w_run(), 0, 100),
            (p.w_block(), 0, 100),
            (p.w_plus(), 0, 100),
            (p.w_one(), 0, 100),
            (p.run_len(), 2, 12),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
        }
        for (value, min, max) in [
            (p.grid(), 0.0, 1.0),
            (p.clump(), 0.0, 1.0),
            (p.align(), 0.0, 1.0),
            (p.upright(), 0.0, 1.0),
            (p.size_t(), 0.2, 1.0),
            (p.svar(), 0.0, 1.0),
            (p.hier(), 0.0, 1.0),
            (p.square(), 0.0, 1.0),
            (p.hollow(), 0.0, 1.0),
            (p.kin(), 0.0, 1.0),
            (p.brk(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        assert!((0.5..=8.0).contains(&p.stroke_w()), "seed {seed}");
        assert_eq!(p.stroke_w(), (p.stroke_w() * 2.0).round() / 2.0);
        assert!(!p.grain().on() && !p.dither().on(), "seed {seed}");
    }
}

#[test]
fn tokens_is_a_still() {
    assert_eq!(Tool::Tokens.frames(), 1);
    let recipe = derive(7, TOKENS);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 1)),
        "frame 1 is outside 0..1"
    );
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..20 {
        let recipe = derive(seed, TOKENS);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = TokensParams::default();
    assert_eq!(error(p.set_cols(3)), "tokens: cols 3 is outside 4..=120");
    assert_eq!(
        error(p.set_size_t(0.02)),
        "tokens: sizeT 0.02 is outside 0.03..=1"
    );
    assert_eq!(
        error(p.set_stroke_w(8.5)),
        "tokens: strokeW 8.5 is outside 0..=8"
    );
    assert_eq!(
        error(p.set_brk(1.01)),
        "tokens: break 1.01 is outside 0..=1"
    );
    assert_eq!(p, TokensParams::default());
    p.set_count(400).unwrap();
    p.set_brk(1.0).unwrap();
    assert_eq!((p.count(), p.brk()), (400, 1.0));
}

fn inks_drawn(width: u32, edit: impl FnOnce(&mut TokensParams)) -> Vec<[u8; 3]> {
    let mut recipe = derive(7, TOKENS);
    recipe
        .set_palette(Palette::from_hex(&["#ff0000", "#00ff00", "#0000ff"]).unwrap())
        .unwrap();
    let Params::Tokens(params) = recipe.params_mut() else {
        unreachable!()
    };
    params.set_cols(16).unwrap();
    params.set_count(60).unwrap();
    params.set_clump(0.0).unwrap();
    params.set_hollow(0.0).unwrap();
    params.set_stroke_w(1.5).unwrap();
    edit(params);
    let image = render(
        &recipe,
        &Frame::new(&recipe, width, width * 16 / 9, 0).unwrap(),
    );
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
fn counters_draw_every_ink_on_the_board() {
    let inks = inks_drawn(270, |_| {});
    for ink in [[0xf4, 0xf1, 0xea], [255, 0, 0], [0, 255, 0], [0, 0, 255]] {
        assert!(inks.contains(&ink), "{ink:?} never drawn");
    }
}

#[test]
fn the_grid_is_the_rule_ink_at_its_opacity() {
    let blended = |grid: f64| {
        inks_drawn(1000, |p| {
            p.set_hollow(1.0).unwrap();
            p.set_stroke_w(0.0).unwrap();
            p.set_grid(grid).unwrap();
        })
    };
    assert_eq!(blended(0.0), [[0xf4, 0xf1, 0xea]]);
    let full = blended(1.0);
    assert!(full.contains(&[0xd9, 0xd4, 0xc7]), "{full:?}");
}
