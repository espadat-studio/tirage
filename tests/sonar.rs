use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

use tirage::{Frame, Palette, Params, Recipe, SonarParams, Tool, ToolPin, derive, render};

const SONAR: ToolPin = ToolPin::Tool(Tool::Sonar);

fn sonar(recipe: &Recipe) -> &SonarParams {
    let Params::Sonar(params) = recipe.params() else {
        panic!("not a sonar Recipe");
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
            r##"{"tirage":0,"tool":"sonar","tool_seed":9611518,"palette":["#0a0f1c","#3ddc97","#4361ee","#ffd166","#ef476f","#f1faee"],"params":{"level":0.46,"scale":7.3,"warp":0.61,"grid":238,"depth":0.16,"fringe":0.04,"spark":0.1,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"sonar","tool_seed":3724278653,"palette":["#0a0f1c","#3ddc97","#4361ee","#ffd166","#ef476f","#f1faee"],"params":{"level":0.46,"scale":4.3,"warp":0.2,"grid":270,"depth":0.98,"fringe":0.58,"spark":0.85,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"sonar","tool_seed":3565986084,"palette":["#0a0f1c","#3ddc97","#4361ee","#ffd166","#ef476f","#f1faee"],"params":{"level":0.63,"scale":2.6,"warp":0.99,"grid":218,"depth":0.2,"fringe":0.4,"spark":0.02,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, SONAR).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    for seed in 0..2000 {
        let recipe = derive(seed, SONAR);
        let p = sonar(&recipe);
        assert_eq!(
            recipe.palette().unwrap(),
            &Palette::from_hex(&[
                "#0a0f1c", "#3ddc97", "#4361ee", "#ffd166", "#ef476f", "#f1faee"
            ])
            .unwrap()
        );
        for (value, min, max, unit) in [
            (p.level(), 0.25, 0.7, 100.0),
            (p.scale(), 1.0, 10.0, 10.0),
            (p.warp(), 0.0, 1.0, 100.0),
            (p.depth(), 0.0, 1.0, 100.0),
            (p.fringe(), 0.0, 1.0, 100.0),
            (p.spark(), 0.0, 1.0, 100.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(value, (value * unit).round() / unit, "seed {seed}: {value}");
        }
        assert!(
            (140..=300).contains(&p.grid()) && p.grid().is_multiple_of(2),
            "seed {seed}: grid {}",
            p.grid()
        );
    }
}

#[test]
fn tool_seed_is_never_zero() {
    assert!((0..10_000).all(|seed| derive(seed, SONAR).tool_seed().get() != 0));
}

#[test]
fn recipe_round_trips_through_json() {
    let recipe = derive(7, SONAR);
    assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = SonarParams::default();
    assert_eq!(error(p.set_level(1.4)), "sonar: level 1.4 is outside 0..=1");
    assert_eq!(
        error(p.set_scale(0.5)),
        "sonar: scale 0.5 is outside 1..=10"
    );
    assert_eq!(
        error(p.set_grid(400)),
        "sonar: grid 400 is outside 40..=320"
    );
    assert_eq!(
        error(p.set_spark(f64::NAN)),
        "sonar: spark NaN is outside 0..=1"
    );
    assert_eq!(
        error(p.grain_pass_mut().set_amount(-0.1)),
        "grain: grnAmount -0.1 is outside 0..=1"
    );
    assert_eq!(
        error(p.dither_mut().set_levels(9)),
        "dither: dthLevels 9 is outside 2..=8"
    );
    assert_eq!(
        error(p.dither_mut().set_size(0)),
        "dither: dthSize 0 is outside 1..=10"
    );
    assert_eq!(p, SonarParams::default());
    p.set_level(1.0).unwrap();
    assert_eq!(p.level(), 1.0);
}

#[test]
fn palette_rejects_malformed_inks() {
    assert_eq!(
        error(Palette::from_hex(&["#000000", "#fff"])),
        r##"palette: "#fff" is not a #rrggbb colour"##
    );
    assert_eq!(
        error(Palette::from_hex(&["#000000", "00ff00"])),
        r#"palette: "00ff00" is not a #rrggbb colour"#
    );
    assert_eq!(
        error(Palette::from_hex(&["#000000"])),
        "palette: needs at least 2 inks, got 1"
    );
    assert_eq!(
        Palette::from_hex(&["#ABCDEF", "#000000"]).unwrap().to_hex(),
        ["#abcdef", "#000000"]
    );
}

type Edit = fn(&mut serde_json::Value);

fn json(edit: impl FnOnce(&mut serde_json::Value)) -> Result<Recipe, tirage::Error> {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, SONAR).to_json()).unwrap();
    edit(&mut value);
    Recipe::from_json(&value.to_string())
}

#[test]
fn recipe_json_errors_are_human() {
    let cases: [(&str, Edit); 10] = [
        ("Recipe is tirage major 1, this build reads major 0", |v| {
            v["tirage"] = 1.into()
        }),
        (
            "Recipe JSON: unknown field `colour`, expected one of `tirage`, `tool`, `tool_seed`, `palette`, `params` at line 1 column 9",
            |v| v["colour"] = 1.into(),
        ),
        (
            "Recipe JSON: params: unknown field `lvl`, expected one of `level`, `scale`, `warp`, `grid`, `depth`, `fringe`, `spark`, `ditherTog`, `dthKinds`, `dthSize`, `dthLevels`, `dthAmount`, `grainTog`, `grnBlends`, `grnAmount`, `grnSize`, `grnSpecks`, `grnVignette`",
            |v| v["params"]["lvl"] = 1.into(),
        ),
        ("sonar: level 1.4 is outside 0..=1", |v| {
            v["params"]["level"] = 1.4.into()
        }),
        ("grain: grnSize 0.4 is outside 0.5..=4", |v| {
            v["params"]["grnSize"] = 0.4.into()
        }),
        (
            "Recipe JSON: params: unknown variant `Bayer 16`, expected one of `Bayer 8`, `Bayer 4`, `Noise`",
            |v| v["params"]["dthKinds"] = "Bayer 16".into(),
        ),
        (r##"palette: "#ggg000" is not a #rrggbb colour"##, |v| {
            v["palette"][0] = "#ggg000".into()
        }),
        ("palette: needs at least 2 inks, got 0", |v| {
            v.as_object_mut().unwrap().remove("palette");
        }),
        (
            r#"unknown tool "vien", expected one of sonar, husk, vein, aura, kiosk, frond, benday, terrain, stitch, pith, mosh, mist, coral, whorl, sear, culture, bloom, weave, warp, zig, relief, atlas, sprig, stipple, motley, oddgrid, fete, fold, quilt, static, splice, dahlia, hiss, crowd, cipher, riso, rise, carve, specimen, pane, modular, prism, parcel, tokens, optic, vee, sampler, totem, filament"#,
            |v| v["tool"] = "vien".into(),
        ),
        (
            "Recipe JSON: invalid value: integer `0`, expected a nonzero u32 at line 1 column 392",
            |v| v["tool_seed"] = 0.into(),
        ),
    ];
    for (message, edit) in cases {
        assert_eq!(error(json(edit)), message);
    }
}

#[test]
fn recipe_params_reject_a_repeated_parameter() {
    let json = derive(7, SONAR)
        .to_json()
        .replacen(r#""level":"#, r#""level":0.5,"level":"#, 1);
    assert_eq!(
        error(Recipe::from_json(&json)),
        "Recipe JSON: params: duplicate field `level` at line 1 column 154"
    );
}

#[test]
fn frame_checks_edges_and_time() {
    let recipe = derive(7, SONAR);
    assert_eq!(
        error(Frame::new(&recipe, 0, 960, 0)),
        "frame 0x960 is outside 1..=8192 per edge"
    );
    assert_eq!(
        error(Frame::new(&recipe, 540, 8193, 0)),
        "frame 540x8193 is outside 1..=8192 per edge"
    );
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 24)),
        "frame 24 is outside 0..24"
    );
    assert!(Frame::new(&recipe, 8192, 1, 23).is_ok());
}

#[test]
fn sonar_loops_at_its_site_default_motion() {
    assert_eq!((Tool::Sonar.frames(), Tool::Sonar.fps()), (24, 10));
}

#[test]
fn frame_zero_is_byte_identical_to_the_still_reference_export() {
    let refs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/refs");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(refs.join("manifest.json")).unwrap()).unwrap();
    let stills: Vec<_> = manifest["tools"]["sonar"]["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|fixture| fixture.get("frame").is_none())
        .collect();
    assert_eq!(stills.len(), 7);
    for fixture in stills {
        let name = fixture["name"].as_str().unwrap();
        let recipe = Recipe::from_json(&fixture["recipe"].to_string()).unwrap();
        let image = render(&recipe, &Frame::new(&recipe, 540, 960, 0).unwrap());
        let still = image::open(refs.join(format!("sonar/{name}.png")))
            .unwrap()
            .to_rgba8();
        assert!(
            image.rgba() == still.as_raw(),
            "{name}: frame 0 is not the Still"
        );
    }
}

#[test]
fn render_gives_a_straight_alpha_png_of_the_frame_size() {
    let mut recipe = derive(7, SONAR);
    recipe.set_tool_seed(NonZeroU32::new(42).unwrap());
    let image = render(&recipe, &Frame::new(&recipe, 90, 160, 0).unwrap());
    assert_eq!(
        (image.width(), image.height(), image.rgba().len()),
        (90, 160, 90 * 160 * 4)
    );
    assert!(image.rgba().chunks(4).all(|px| px[3] == 255));
    let decoded = image::load_from_memory(&image.to_png()).unwrap().to_rgba8();
    assert_eq!(decoded.as_raw(), image.rgba());
}
