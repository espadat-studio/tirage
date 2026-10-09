use std::io::Write;
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_tirage");

fn tirage(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn on_tty(script: &str) -> Output {
    Command::new("script")
        .args([
            "-qec",
            &script.replace("tirage", &format!("'{BIN}'")),
            "/dev/null",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn recipe() -> Vec<u8> {
    tirage(&["derive", "--seed", "42", "--tool", "sonar"], b"").stdout
}

#[test]
fn derive_pipes_into_render_as_a_png() {
    let dir = std::env::temp_dir().join(format!("tirage-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut derive = Command::new(BIN)
        .args(["derive", "--seed", "42", "--tool", "sonar"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let out = Command::new(BIN)
        .args(["render", "--size", "1080x1920", "-o", "out.png"])
        .current_dir(&dir)
        .stdin(derive.stdout.take().unwrap())
        .output()
        .unwrap();
    assert_eq!(derive.wait().unwrap().code(), Some(0));
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let png = image::open(dir.join("out.png")).unwrap();
    assert_eq!((png.width(), png.height()), (1080, 1920));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_closed_pipe_is_not_an_error() {
    let mut child = Command::new(BIN)
        .args(["tools"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(out.stderr.is_empty(), "{}", text(&out.stderr));
}

#[test]
fn derive_prints_one_line_of_recipe_json() {
    let out = tirage(&["derive", "--seed", "42", "--tool", "sonar"], b"");
    assert_eq!(out.status.code(), Some(0));
    let json = text(&out.stdout);
    assert_eq!(
        json,
        format!(
            "{}\n",
            tirage::derive(42, tirage::ToolPin::Tool(tirage::Tool::Sonar)).to_json()
        )
    );
}

#[test]
fn render_reads_a_recipe_file_and_writes_a_png_file() {
    let dir = std::env::temp_dir().join(format!("tirage-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (input, output) = (dir.join("recipe.json"), dir.join("out.png"));
    std::fs::write(&input, recipe()).unwrap();
    let out = tirage(
        &[
            "render",
            input.to_str().unwrap(),
            "--size",
            "90x160",
            "-o",
            output.to_str().unwrap(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(out.stdout.is_empty());
    let png = image::open(&output).unwrap();
    assert_eq!((png.width(), png.height()), (90, 160));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn runtime_errors_exit_1_with_a_human_message() {
    let cases: [(&[&str], &[u8], &str); 4] = [
        (
            &["render", "--size", "9x16", "-o", "-"],
            b"{",
            "error: Recipe JSON: EOF while parsing an object at line 1 column 1\n",
        ),
        (
            &["render", "--size", "0x16", "-o", "-"],
            &recipe(),
            "error: frame 0x16 is outside 1..=8192 per edge\n",
        ),
        (
            &["render", "--size", "9x16", "--frame", "24", "-o", "-"],
            &recipe(),
            "error: frame 24 is outside 0..24\n",
        ),
        (
            &["render", "missing.json", "--size", "9x16", "-o", "-"],
            b"",
            "error: cannot read missing.json: No such file or directory (os error 2)\n",
        ),
    ];
    for (args, stdin, message) in cases {
        let out = tirage(args, stdin);
        assert_eq!(
            (out.status.code(), text(&out.stderr).as_str()),
            (Some(1), message),
            "{args:?}"
        );
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn usage_errors_exit_2() {
    let cases: [&[&str]; 5] = [
        &["derive", "--seed", "42", "--tool", "vien"],
        &["derive", "--seed", "-1", "--tool", "sonar"],
        &["render", "--size", "big", "-o", "-"],
        &["render", "--size", "9x16"],
        &["paint"],
    ];
    for args in cases {
        let out = tirage(args, b"");
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(
            text(&out.stderr).starts_with("error: "),
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
    let out = tirage(&["derive", "--seed", "42", "--tool", "vien"], b"");
    assert!(
        text(&out.stderr)
            .contains(r#"unknown tool "vien", expected one of sonar, husk, vein, aura"#)
    );
}

#[test]
fn render_refuses_png_bytes_on_a_terminal() {
    let out = on_tty("tirage derive --seed 42 --tool sonar | tirage render --size 9x16 -o -");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stdout).contains("refusing to write a PNG to a terminal"),
        "{}",
        text(&out.stdout)
    );
    assert!(
        text(&out.stdout).contains("Usage: tirage render"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn render_refuses_to_wait_on_a_terminal_for_a_recipe() {
    let out = on_tty("tirage render --size 9x16 -o out.png");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stdout).contains("no Recipe on stdin"),
        "{}",
        text(&out.stdout)
    );
}

#[test]
fn version_names_the_derivation_major() {
    let out = tirage(&["--version"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        text(&out.stdout),
        format!(
            "tirage {} (derivation major 0)\n",
            env!("CARGO_PKG_VERSION")
        )
    );
    assert_eq!(tirage(&["-V"], b"").status.code(), Some(2));
}

const CHASSIS_LISTING: &str = concat!(
    r#"{"id":"ditherTog","kind":"toggle"},"#,
    r#"{"id":"dthKinds","kind":"choice","choices":["Bayer 8","Bayer 4","Noise"]},"#,
    r#"{"id":"dthSize","kind":"range","min":1.0,"max":10.0,"step":1.0},"#,
    r#"{"id":"dthLevels","kind":"range","min":2.0,"max":8.0,"step":1.0},"#,
    r#"{"id":"dthAmount","kind":"range","min":0.0,"max":1.0,"step":0.05},"#,
    r#"{"id":"grainTog","kind":"toggle"},"#,
    r#"{"id":"grnBlends","kind":"choice","choices":["Add","Overlay","Soft light","Multiply","Screen"]},"#,
    r#"{"id":"grnAmount","kind":"range","min":0.0,"max":1.0,"step":0.05},"#,
    r#"{"id":"grnSize","kind":"range","min":0.5,"max":4.0,"step":0.1},"#,
    r#"{"id":"grnSpecks","kind":"range","min":0.0,"max":1.0,"step":0.05},"#,
    r#"{"id":"grnVignette","kind":"range","min":0.0,"max":1.0,"step":0.05}"#,
);

#[test]
fn tools_json_has_a_stable_shape() {
    let out = tirage(&["tools", "--json"], b"");
    assert_eq!(out.status.code(), Some(0));
    let sonar = concat!(
        r#"{"slug":"sonar","frames":24,"params":["#,
        r#"{"id":"level","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"scale","kind":"range","min":1.0,"max":10.0,"step":0.1},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"grid","kind":"range","min":40.0,"max":320.0,"step":2.0},"#,
        r#"{"id":"depth","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"fringe","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"spark","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
    );
    let husk = concat!(
        r#"{"slug":"husk","frames":1,"params":["#,
        r#"{"id":"count","kind":"range","min":1.0,"max":70.0,"step":1.0},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"lump","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"bites","kind":"choice","choices":["Crumble","Dots"]},"#,
        r#"{"id":"eat","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"tex","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
    );
    let vein = concat!(
        r#"{"slug":"vein","frames":1,"params":["#,
        r#"{"id":"flows","kind":"choice","choices":["Marble","Swirl","Ripple"]},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"curve","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"levels","kind":"range","min":3.0,"max":14.0,"step":1.0},"#,
        r#"{"id":"tiger","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"edges","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"stars","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"tints","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"runs","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
    );
    let aura = concat!(
        r#"{"slug":"aura","frames":1,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Auto","Clouds","Mesh","Sweep"]},"#,
        r#"{"id":"scale","kind":"range","min":0.5,"max":2.0,"step":0.05},"#,
        r#"{"id":"churn","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
        r#"{"id":"punch","kind":"range","min":0.0,"max":1.0,"step":0.01},"#,
    );
    assert_eq!(
        text(&out.stdout),
        format!(
            "[{sonar}{CHASSIS_LISTING}]}},{husk}{CHASSIS_LISTING}]}},{vein}{CHASSIS_LISTING}]}},{aura}{CHASSIS_LISTING}]}}]\n"
        )
    );
}

#[test]
fn tools_lists_slugs_frames_and_ranges_for_humans() {
    let out = tirage(&["tools"], b"");
    assert_eq!(out.status.code(), Some(0));
    let listing = text(&out.stdout);
    assert!(listing.starts_with("sonar  24 frames\n"), "{listing}");
    assert!(listing.contains("\nhusk  1 frame\n"), "{listing}");
    assert!(listing.contains("\nvein  1 frame\n"), "{listing}");
    assert!(listing.contains("\naura  1 frame\n"), "{listing}");
    for line in [
        "  grid        40..=320  step 2\n",
        "  grnVignette 0..=1     step 0.05\n",
        "  grainTog    on/off\n",
        "  grnBlends   Add, Overlay, Soft light, Multiply, Screen\n",
        "  dthKinds    Bayer 8, Bayer 4, Noise\n",
        "  levels      3..=14    step 1\n",
        "  flows       Marble, Swirl, Ripple\n",
        "  styles      Auto, Clouds, Mesh, Sweep\n",
    ] {
        assert!(listing.contains(line), "{line:?} in\n{listing}");
    }
}

#[test]
fn recipe_json_accepts_every_listed_parameter_value() {
    let listings = json(&tirage(&["tools", "--json"], b"").stdout);
    for listing in listings.as_array().unwrap() {
        let slug = listing["slug"].as_str().unwrap();
        let recipe = json(&tirage(&["derive", "--seed", "42", "--tool", slug], b"").stdout);
        let params = listing["params"].as_array().unwrap();
        let mut ids: Vec<_> = params.iter().map(|p| p["id"].as_str().unwrap()).collect();
        ids.sort_unstable();
        let keys: Vec<_> = recipe["params"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(ids, keys, "{slug}");
        for param in params {
            let values = match param["kind"].as_str().unwrap() {
                "range" => [&param["min"], &param["max"]]
                    .map(|bound| {
                        let bound = bound.as_f64().unwrap();
                        if bound.fract() == 0.0 {
                            serde_json::json!(bound as u64)
                        } else {
                            serde_json::json!(bound)
                        }
                    })
                    .to_vec(),
                "toggle" => vec![true.into(), false.into()],
                "choice" => param["choices"].as_array().unwrap().clone(),
                kind => panic!("{slug}: unknown kind {kind}"),
            };
            for value in values {
                let mut recipe = recipe.clone();
                recipe["params"][param["id"].as_str().unwrap()] = value.clone();
                if let Err(e) = tirage::Recipe::from_json(&recipe.to_string()) {
                    panic!("{slug} {} = {value}: {e}", param["id"]);
                }
            }
        }
    }
}

#[test]
fn no_args_on_a_terminal_prints_concise_help() {
    let out = on_tty("tirage");
    assert_eq!(out.status.code(), Some(2));
    let help = text(&out.stdout);
    assert!(
        help.contains("tirage derive --seed 42 --tool sonar | tirage render"),
        "{help}"
    );
    assert!(help.contains("tirage --help"), "{help}");
    assert!(help.lines().count() < 15, "{help}");
}

#[test]
fn help_leads_with_examples() {
    for args in [["-h"], ["--help"]] {
        let out = tirage(&args, b"");
        assert_eq!(out.status.code(), Some(0));
        let help = text(&out.stdout);
        assert!(help.starts_with("Examples:\n"), "{help}");
        assert!(
            help.contains("Commands:") && help.contains("tools"),
            "{help}"
        );
    }
    let help = text(&tirage(&["render", "--help"], b"").stdout);
    assert!(help.starts_with("Examples:\n"), "{help}");
}

fn json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).unwrap()
}

#[test]
fn derive_without_a_tool_deals_one() {
    let out = tirage(&["derive", "--seed", "42"], b"");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        format!("{}\n", tirage::derive(42, tirage::ToolPin::Any).to_json())
    );
}

#[test]
fn derive_pins_a_palette_and_keeps_the_rest() {
    let out = tirage(
        &["derive", "--seed", "42", "--palette", "#000000,#FFFFFF"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let dealt = json(&tirage(&["derive", "--seed", "42"], b"").stdout);
    let pinned = json(&out.stdout);
    assert_eq!(pinned["palette"], serde_json::json!(["#000000", "#ffffff"]));
    assert_eq!(
        (&pinned["tool"], &pinned["params"], &pinned["tool_seed"]),
        (&dealt["tool"], &dealt["params"], &dealt["tool_seed"])
    );
}

#[test]
fn derive_reads_taste_bounds_from_a_file() {
    let dir = std::env::temp_dir().join(format!("tirage-taste-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (good, bad) = (dir.join("good.json"), dir.join("bad.json"));
    std::fs::write(&good, r#"{"tool":"sonar","level":[0.5,0.5]}"#).unwrap();
    std::fs::write(&bad, r#"{"tool":"sonar","level":[0,1.4]}"#).unwrap();
    let out = tirage(
        &["derive", "--seed", "42", "--taste", good.to_str().unwrap()],
        b"",
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(json(&out.stdout)["params"]["level"], 0.5);
    let out = tirage(
        &["derive", "--seed", "42", "--taste", bad.to_str().unwrap()],
        b"",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("sonar: level 1.4 is outside 0..=1"),
        "{}",
        text(&out.stderr)
    );
    let out = tirage(
        &[
            "derive",
            "--seed",
            "1",
            "--tool",
            "sonar",
            "--taste",
            good.to_str().unwrap(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
    let out = tirage(&["derive", "--seed", "1", "--taste", "missing.json"], b"");
    assert_eq!(
        (out.status.code(), text(&out.stderr).as_str()),
        (
            Some(1),
            "error: cannot read missing.json: No such file or directory (os error 2)\n"
        )
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn derive_rejects_more_inks_than_the_tool_draws_as_a_usage_error() {
    let out = tirage(
        &[
            "derive",
            "--seed",
            "42",
            "--tool",
            "aura",
            "--palette",
            "#000000,#111111,#222222,#333333,#444444",
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("aura: palette has 5 inks, aura draws at most 4"),
        "{}",
        text(&out.stderr)
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn derive_rejects_malformed_hex_as_a_usage_error() {
    let out = tirage(
        &["derive", "--seed", "42", "--palette", "#000000,#fff"],
        b"",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains(r##"palette: "#fff" is not a #rrggbb colour"##),
        "{}",
        text(&out.stderr)
    );
    assert!(out.stdout.is_empty());
}
