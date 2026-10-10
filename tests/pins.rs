use tirage::{Palette, Params, Recipe, Taste, Tool, ToolPin, derive};

const SONAR: ToolPin = ToolPin::Tool(Tool::Sonar);

fn level(recipe: &Recipe) -> f64 {
    let Params::Sonar(params) = recipe.params() else {
        panic!("not a sonar Recipe");
    };
    params.level()
}

fn error<T: std::fmt::Debug>(result: Result<T, tirage::Error>) -> String {
    result.unwrap_err().to_string()
}

#[test]
fn any_deals_a_tool_and_pinning_it_changes_nothing() {
    for seed in 0..500 {
        let dealt = derive(seed, ToolPin::Any);
        assert!(Tool::ALL.contains(&dealt.tool()));
        assert_eq!(
            derive(seed, ToolPin::Tool(dealt.tool())),
            dealt,
            "seed {seed}"
        );
    }
}

#[test]
fn pinning_the_palette_to_its_dealt_value_changes_nothing() {
    for seed in 0..500 {
        let dealt = derive(seed, SONAR);
        let mut pinned = derive(seed, SONAR);
        pinned
            .set_palette(dealt.palette().unwrap().clone())
            .unwrap();
        assert_eq!(pinned, dealt, "seed {seed}");
    }
}

#[test]
fn a_palette_pin_leaves_every_other_field_unchanged() {
    let dealt = derive(9, SONAR);
    let inks = Palette::from_hex(&["#000000", "#ffffff"]).unwrap();
    let mut pinned = derive(9, SONAR);
    pinned.set_palette(inks.clone()).unwrap();
    assert_eq!(pinned.palette().unwrap(), &inks);
    assert_eq!(
        (pinned.tool(), pinned.tool_seed(), pinned.params()),
        (dealt.tool(), dealt.tool_seed(), dealt.params())
    );
}

#[test]
fn shipped_taste_bounds_deal_like_a_tool_pin() {
    let none: &[(&str, [f64; 2])] = &[];
    let shipped = [("level", [0.25, 0.7]), ("grid", [140.0, 300.0])];
    for seed in 0..500 {
        for taste in [
            Taste::new(Tool::Sonar, none).unwrap(),
            Taste::new(Tool::Sonar, &shipped).unwrap(),
        ] {
            assert_eq!(
                derive(seed, ToolPin::Taste(taste)),
                derive(seed, SONAR),
                "seed {seed}"
            );
        }
    }
}

#[test]
fn a_taste_override_moves_only_its_own_parameter() {
    let taste = Taste::new(Tool::Sonar, &[("level", [0.9, 1.0])]).unwrap();
    assert_eq!(taste.tool(), Tool::Sonar);
    for seed in 0..500 {
        let dealt = derive(seed, SONAR);
        let mut tasted = derive(seed, ToolPin::Taste(taste.clone()));
        assert!((0.9..=1.0).contains(&level(&tasted)), "seed {seed}");
        let Params::Sonar(params) = tasted.params_mut() else {
            panic!("not a sonar Recipe");
        };
        params.set_level(level(&dealt)).unwrap();
        assert_eq!(tasted, dealt, "seed {seed}");
    }
}

#[test]
fn taste_rejects_bounds_the_slider_cannot_show() {
    let cases: [(&str, [f64; 2], &str); 6] = [
        ("level", [0.2, 1.4], "sonar: level 1.4 is outside 0..=1"),
        ("grid", [20.0, 300.0], "sonar: grid 20 is outside 40..=320"),
        (
            "spark",
            [f64::NAN, 1.0],
            "sonar: spark NaN is outside 0..=1",
        ),
        (
            "grid",
            [141.0, 300.0],
            "sonar: grid 141 is not a slider value, 40..=320 step 2",
        ),
        (
            "level",
            [0.255, 0.5],
            "sonar: level 0.255 is not a slider value, 0..=1 step 0.01",
        ),
        (
            "level",
            [0.7, 0.25],
            "sonar: level Taste bounds 0.7..=0.25 have min above max",
        ),
    ];
    for (id, range, message) in cases {
        assert_eq!(error(Taste::new(Tool::Sonar, &[(id, range)])), message);
    }
    assert_eq!(
        error(Taste::new(Tool::Sonar, &[("lvl", [0.0, 1.0])])),
        r#"sonar: unknown Parameter "lvl", expected one of level, scale, warp, grid, depth, fringe, spark"#
    );
}

#[test]
fn taste_json_overrides_named_parameters_only() {
    let taste = Taste::from_json(r#"{"tool":"sonar","level":[0.5,0.5],"grid":[40,40]}"#).unwrap();
    assert_eq!(
        taste,
        Taste::new(
            Tool::Sonar,
            &[("level", [0.5, 0.5]), ("grid", [40.0, 40.0])]
        )
        .unwrap()
    );
    let recipe = derive(3, ToolPin::Taste(taste));
    let Params::Sonar(params) = recipe.params() else {
        panic!("not a sonar Recipe");
    };
    assert_eq!((params.level(), params.grid()), (0.5, 40));
}

#[test]
fn taste_json_errors_are_human() {
    let cases = [
        (
            r#"{"level":[0,1]}"#,
            "Taste JSON: missing field `tool` at line 1 column 15",
        ),
        (
            r#"{"tool":"vien"}"#,
            r#"unknown tool "vien", expected one of sonar, husk, vein, aura, kiosk, frond, benday, terrain, stitch, pith, mosh, mist, coral, whorl, sear, culture, bloom, weave, warp, zig, relief, atlas, sprig, stipple, motley, oddgrid, fete, fold, quilt, static, splice, dahlia, hiss, crowd, cipher, riso, rise, carve, specimen, pane, modular, prism, parcel, tokens, optic, vee, sampler, totem"#,
        ),
        (
            r#"{"tool":"sonar","level":[0,1.4]}"#,
            "sonar: level 1.4 is outside 0..=1",
        ),
        (
            r#"{"tool":"sonar","lvl":[0,1]}"#,
            r#"sonar: unknown Parameter "lvl", expected one of level, scale, warp, grid, depth, fringe, spark"#,
        ),
        (
            r#"{"tool":"sonar","level":[0,1],"level":[0.5,1]}"#,
            "Taste JSON: duplicate field `level` at line 1 column 46",
        ),
        (
            r#"{"tool":"sonar","tool":"husk"}"#,
            "Taste JSON: duplicate field `tool` at line 1 column 22",
        ),
    ];
    for (json, message) in cases {
        assert_eq!(error(Taste::from_json(json)), message, "{json}");
    }
    assert!(error(Taste::from_json(r#"{"tool":"sonar","level":0.5}"#)).starts_with("Taste JSON: "));
}
