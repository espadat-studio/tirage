use skrifa::{FontRef, MetadataProvider};
use tirage::{
    CharacterSet, Frame, KioskParams, Palette, Params, Recipe, Tool, ToolPin, derive, render,
};

const KIOSK: ToolPin = ToolPin::Tool(Tool::Kiosk);

fn kiosk(recipe: &Recipe) -> &KioskParams {
    let Params::Kiosk(params) = recipe.params() else {
        unreachable!("a kiosk Recipe")
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
            r##"{"tirage":0,"tool":"kiosk","tool_seed":9611518,"palette":["#f3efe0","#141414","#d7263d","#1b4079","#f4b942","#3c887e","#f26430","#8e44ad"],"params":{"split":0.35,"rings":35,"stripes":2,"sets":"Stipple","grid":7,"bigSize":0.39,"density":0.07,"smallSize":0.37,"small":0.11,"blocks":0.03,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            2,
            r##"{"tirage":0,"tool":"kiosk","tool_seed":3724278653,"palette":["#f3efe0","#141414","#d7263d","#1b4079","#f4b942","#3c887e","#f26430","#8e44ad"],"params":{"split":0.3,"rings":32,"stripes":18,"sets":"Stipple","grid":16,"bigSize":0.53,"density":0.03,"smallSize":0.41,"small":0.34,"blocks":0.03,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
        (
            u64::MAX,
            r##"{"tirage":0,"tool":"kiosk","tool_seed":3565986084,"palette":["#f3efe0","#141414","#d7263d","#1b4079","#f4b942","#3c887e","#f26430","#8e44ad"],"params":{"split":0.58,"rings":34,"stripes":23,"sets":"Runes","grid":13,"bigSize":0.52,"density":0.12,"smallSize":0.2,"small":0.41,"blocks":0.91,"ditherTog":false,"dthKinds":"Bayer 8","dthSize":2,"dthLevels":3,"dthAmount":1.0,"grainTog":false,"grnBlends":"Add","grnAmount":0.55,"grnSize":1.0,"grnSpecks":0.5,"grnVignette":0.5}}"##,
        ),
    ];
    for (seed, json) in golden {
        assert_eq!(derive(seed, KIOSK).to_json(), json, "seed {seed}");
    }
}

#[test]
fn every_draw_is_inside_taste_bounds_and_on_the_slider_grid() {
    let palette = Palette::from_hex(&[
        "#f3efe0", "#141414", "#d7263d", "#1b4079", "#f4b942", "#3c887e", "#f26430", "#8e44ad",
    ])
    .unwrap();
    let mut sets = Vec::new();
    for seed in 0..2000 {
        let recipe = derive(seed, KIOSK);
        assert_eq!(recipe.palette(), &palette);
        let p = kiosk(&recipe);
        for (value, min, max) in [
            (p.split(), 0.15, 0.95),
            (p.big_size(), 0.3, 0.63),
            (p.density(), 0.0, 0.5),
            (p.small_size(), 0.1, 0.5),
            (p.small(), 0.0, 0.5),
            (p.blocks(), 0.0, 1.0),
        ] {
            assert!((min..=max).contains(&value), "seed {seed}: {value}");
            assert_eq!(
                value,
                (value * 100.0).round() / 100.0,
                "seed {seed}: {value}"
            );
        }
        for (value, range) in [
            (p.rings(), 4..=40),
            (p.stripes(), 2..=24),
            (p.grid(), 4..=30),
        ] {
            assert!(range.contains(&value), "seed {seed}: {value}");
        }
        if !sets.contains(&p.sets()) {
            sets.push(p.sets());
        }
    }
    assert_eq!(sets.len(), CharacterSet::ALL.len());
}

#[test]
fn tool_seed_is_never_zero() {
    assert!((0..10_000).all(|seed| derive(seed, KIOSK).tool_seed().get() != 0));
}

#[test]
fn recipe_round_trips_through_json() {
    for seed in 0..50 {
        let recipe = derive(seed, KIOSK);
        assert_eq!(Recipe::from_json(&recipe.to_json()).unwrap(), recipe);
    }
}

#[test]
fn every_character_of_every_set_is_in_the_bundled_font() {
    let font = FontRef::new(include_bytes!("../fonts/DejaVuSansMono-Bold.ttf")).unwrap();
    let charmap = font.charmap();
    for set in CharacterSet::ALL {
        for ch in set.chars().chars() {
            assert!(charmap.map(ch).is_some(), "{set:?}: U+{:04X}", ch as u32);
        }
    }
    assert_eq!(
        CharacterSet::ALL
            .iter()
            .map(|set| set.chars().chars().count())
            .collect::<Vec<_>>(),
        [14, 9, 7, 13, 10, 9]
    );
}

#[test]
fn setters_reject_values_outside_the_site_range() {
    let mut p = KioskParams::default();
    assert_eq!(
        error(p.set_split(0.1)),
        "kiosk: split 0.1 is outside 0.15..=0.95"
    );
    assert_eq!(error(p.set_rings(41)), "kiosk: rings 41 is outside 4..=40");
    assert_eq!(
        error(p.set_stripes(1)),
        "kiosk: stripes 1 is outside 2..=24"
    );
    assert_eq!(error(p.set_grid(31)), "kiosk: grid 31 is outside 4..=30");
    assert_eq!(
        error(p.set_big_size(1.7)),
        "kiosk: bigSize 1.7 is outside 0.3..=1.6"
    );
    assert_eq!(
        error(p.set_density(-0.1)),
        "kiosk: density -0.1 is outside 0..=1"
    );
    assert_eq!(
        error(p.set_small_size(0.95)),
        "kiosk: smallSize 0.95 is outside 0.1..=0.9"
    );
    assert_eq!(
        error(p.set_small(f64::NAN)),
        "kiosk: small NaN is outside 0..=1"
    );
    assert_eq!(error(p.set_blocks(2.0)), "kiosk: blocks 2 is outside 0..=1");
    assert_eq!(p, KioskParams::default());
    p.set_sets(CharacterSet::Runes);
    assert_eq!(p.sets(), CharacterSet::Runes);
}

#[test]
fn recipe_json_names_the_character_set_by_its_site_label() {
    let mut value: serde_json::Value = serde_json::from_str(&derive(7, KIOSK).to_json()).unwrap();
    value["params"]["sets"] = "Custom".into();
    assert_eq!(
        error(Recipe::from_json(&value.to_string())),
        "Recipe JSON: params: unknown variant `Custom`, expected one of `DOS`, `Stipple`, `Blocks`, `Code`, `Digits`, `Runes`"
    );
    value["params"]["sets"] = "DOS".into();
    assert_eq!(
        kiosk(&Recipe::from_json(&value.to_string()).unwrap()).sets(),
        CharacterSet::Dos
    );
}

#[test]
fn kiosk_is_appended_to_the_registry() {
    assert_eq!(Tool::ALL.last(), Some(&Tool::Kiosk));
    assert_eq!(Tool::from_slug("kiosk").unwrap(), Tool::Kiosk);
}

#[test]
fn kiosk_loops_at_its_site_default_motion() {
    assert_eq!((Tool::Kiosk.frames(), Tool::Kiosk.fps()), (24, 6));
    let recipe = derive(7, KIOSK);
    assert_eq!(
        error(Frame::new(&recipe, 540, 960, 24)),
        "frame 24 is outside 0..24"
    );
}

#[test]
fn shuffle_redeals_the_type_every_frame() {
    let recipe = derive(7, KIOSK);
    let frame = |t| render(&recipe, &Frame::new(&recipe, 90, 160, t).unwrap());
    let still = frame(0);
    assert_eq!(frame(0), still);
    assert_ne!(frame(1), still);
}

#[test]
fn glyphs_above_the_hinting_limit_render() {
    let mut recipe = derive(7, KIOSK);
    let Params::Kiosk(p) = recipe.params_mut() else {
        unreachable!("a kiosk Recipe")
    };
    p.set_grid(4).unwrap();
    p.set_big_size(1.6).unwrap();
    p.set_density(1.0).unwrap();
    let image = render(&recipe, &Frame::new(&recipe, 2400, 4266, 0).unwrap());
    assert_eq!((image.width(), image.height()), (2400, 4266));
}
