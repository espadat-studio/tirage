use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
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
    if let Err(e) = child.stdin.take().unwrap().write_all(stdin) {
        assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe);
    }
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

fn on_tty_typing(script: &str, keys: &str, dir: &Path) -> Output {
    let mut child = Command::new("script")
        .args([
            "-qec",
            &script.replace("tirage", &format!("'{BIN}'")),
            "/dev/null",
        ])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(keys.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    drop(stdin);
    out
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tirage-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
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
    let repeated = text(&recipe()).replacen(r#""level":"#, r#""level":0.5,"level":"#, 1);
    let cases: [(&[&str], &[u8], &str); 5] = [
        (
            &["render", "--size", "9x16", "-o", "-"],
            b"{",
            "error: Recipe JSON: EOF while parsing an object at line 1 column 1\nhint: pass a Recipe printed by 'tirage derive'\n",
        ),
        (
            &["render", "--size", "9x16", "-o", "-"],
            repeated.as_bytes(),
            "error: Recipe JSON: params: duplicate field `level` at line 1 column 156\nhint: pass a Recipe printed by 'tirage derive'\n",
        ),
        (
            &["render", "--size", "0x16", "-o", "-"],
            &recipe(),
            "error: frame 0x16 is outside 1..=8192 per edge\nhint: pass --size <W>x<H> with each edge in 1..=8192, like 1080x1920\n",
        ),
        (
            &["render", "--size", "9x16", "--frame", "24", "-o", "-"],
            &recipe(),
            "error: frame 24 is outside 0..24\nhint: pass --frame from 0 to 23, or leave it out for the Still\n",
        ),
        (
            &["render", "missing.json", "--size", "9x16", "-o", "-"],
            b"",
            "error: cannot read missing.json: no such file\n",
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
    assert!(text(&out.stderr).contains(
        r#"unknown tool "vien", expected one of sonar, husk, vein, aura, kiosk, frond, benday, terrain, stitch, pith, mosh, mist, coral, whorl, sear, culture, bloom, weave, warp, zig, relief, atlas, sprig, stipple, motley, oddgrid, fete, fold, quilt, static, splice, dahlia, hiss, crowd, cipher, riso, rise, carve, specimen, pane, modular, prism, parcel, tokens, optic, vee, sampler, totem, filament, chaff, strand"#
    ));
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
    let out = on_tty("tirage render --size 9x16 -o out.png --no-input");
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
        format!("tirage {} (derivation major 0)\n", tirage::VERSION)
    );
    assert_eq!(tirage(&["-V"], b"").status.code(), Some(2));
}

const CHASSIS_LISTING: &str = concat!(
    r#"{"id":"ditherTog","kind":"toggle","taste":false},"#,
    r#"{"id":"dthKinds","kind":"choice","choices":["Bayer 8","Bayer 4","Noise"],"taste":false},"#,
    r#"{"id":"dthSize","kind":"range","min":1.0,"max":10.0,"step":1.0,"taste":false},"#,
    r#"{"id":"dthLevels","kind":"range","min":2.0,"max":8.0,"step":1.0,"taste":false},"#,
    r#"{"id":"dthAmount","kind":"range","min":0.0,"max":1.0,"step":0.05,"taste":false},"#,
    r#"{"id":"grainTog","kind":"toggle","taste":false},"#,
    r#"{"id":"grnBlends","kind":"choice","choices":["Add","Overlay","Soft light","Multiply","Screen"],"taste":false},"#,
    r#"{"id":"grnAmount","kind":"range","min":0.0,"max":1.0,"step":0.05,"taste":false},"#,
    r#"{"id":"grnSize","kind":"range","min":0.5,"max":4.0,"step":0.1,"taste":false},"#,
    r#"{"id":"grnSpecks","kind":"range","min":0.0,"max":1.0,"step":0.05,"taste":false},"#,
    r#"{"id":"grnVignette","kind":"range","min":0.0,"max":1.0,"step":0.05,"taste":false}"#,
);

#[test]
fn tools_json_has_a_stable_shape() {
    let out = tirage(&["tools", "--json"], b"");
    assert_eq!(out.status.code(), Some(0));
    let sonar = concat!(
        r#"{"slug":"sonar","frames":24,"svg":true,"params":["#,
        r#"{"id":"level","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":1.0,"max":10.0,"step":0.1,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grid","kind":"range","min":40.0,"max":320.0,"step":2.0,"taste":true},"#,
        r#"{"id":"depth","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"fringe","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"spark","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let husk = concat!(
        r#"{"slug":"husk","frames":36,"svg":false,"params":["#,
        r#"{"id":"count","kind":"range","min":1.0,"max":70.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"lump","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bites","kind":"choice","choices":["Crumble","Dots"],"taste":false},"#,
        r#"{"id":"eat","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tex","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let vein = concat!(
        r#"{"slug":"vein","frames":36,"svg":true,"params":["#,
        r#"{"id":"flows","kind":"choice","choices":["Marble","Swirl","Ripple"],"taste":false},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"curve","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"levels","kind":"range","min":3.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"tiger","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"edges","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"stars","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tints","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"runs","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let aura = concat!(
        r#"{"slug":"aura","frames":72,"svg":false,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Auto","Clouds","Mesh","Sweep"],"taste":false},"#,
        r#"{"id":"scale","kind":"range","min":0.5,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"churn","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"punch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let kiosk = concat!(
        r#"{"slug":"kiosk","frames":24,"svg":true,"params":["#,
        r#"{"id":"split","kind":"range","min":0.15,"max":0.95,"step":0.01,"taste":true},"#,
        r#"{"id":"rings","kind":"range","min":4.0,"max":40.0,"step":1.0,"taste":true},"#,
        r#"{"id":"stripes","kind":"range","min":2.0,"max":24.0,"step":1.0,"taste":true},"#,
        r#"{"id":"sets","kind":"choice","choices":["DOS","Stipple","Blocks","Code","Digits","Runes"],"taste":false},"#,
        r#"{"id":"grid","kind":"range","min":4.0,"max":30.0,"step":1.0,"taste":true},"#,
        r#"{"id":"bigSize","kind":"range","min":0.3,"max":1.6,"step":0.01,"taste":true},"#,
        r#"{"id":"density","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"smallSize","kind":"range","min":0.1,"max":0.9,"step":0.01,"taste":true},"#,
        r#"{"id":"small","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"blocks","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let frond = concat!(
        r#"{"slug":"frond","frames":36,"svg":false,"params":["#,
        r#"{"id":"dirs","kind":"choice","choices":["Potted","Bouquet","Fronds"],"taste":false},"#,
        r#"{"id":"masses","kind":"range","min":0.0,"max":6.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"round","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"growth","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"detail","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"coarse","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"breakup","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"noise","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"patch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"circles","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"rules","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let benday = concat!(
        r#"{"slug":"benday","frames":36,"svg":false,"params":["#,
        r#"{"id":"turb","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"streak","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dir","kind":"range","min":-1.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"black","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":2.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"rims","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dot","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"ring","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"angle","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bite","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let terrain = concat!(
        r#"{"slug":"terrain","frames":30,"svg":false,"params":["#,
        r#"{"id":"scale","kind":"range","min":0.6,"max":9.0,"step":0.1,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":3.0,"step":0.02,"taste":true},"#,
        r#"{"id":"oct","kind":"range","min":1.0,"max":7.0,"step":1.0,"taste":true},"#,
        r#"{"id":"contrast","kind":"range","min":0.5,"max":4.0,"step":0.05,"taste":true},"#,
        r#"{"id":"balance","kind":"range","min":-1.2,"max":1.2,"step":0.05,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.2,"step":0.01,"taste":false},"#,
        r#"{"id":"block","kind":"range","min":0.0,"max":14.0,"step":1.0,"taste":true},"#,
    );
    let stitch = concat!(
        r#"{"slug":"stitch","frames":36,"svg":false,"params":["#,
        r#"{"id":"cols","kind":"range","min":16.0,"max":96.0,"step":1.0,"taste":true},"#,
        r#"{"id":"gutter","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"streak","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"slip","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"band","kind":"range","min":1.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"blobs","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"blobsize","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"steps","kind":"range","min":2.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"ground","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"stray","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let pith = concat!(
        r#"{"slug":"pith","frames":36,"svg":false,"params":["#,
        r#"{"id":"count","kind":"range","min":1.0,"max":60.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"zoom","kind":"range","min":1.0,"max":8.0,"step":0.1,"taste":true},"#,
        r#"{"id":"round","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wobble","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"band","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dither","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
        r#"{"id":"veins","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"thick","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let mosh = concat!(
        r#"{"slug":"mosh","frames":24,"svg":true,"params":["#,
        r#"{"id":"bands","kind":"range","min":1.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"cols","kind":"range","min":24.0,"max":420.0,"step":2.0,"taste":true},"#,
        r#"{"id":"mix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tears","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"runs","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bright","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let mist = concat!(
        r#"{"slug":"mist","frames":72,"svg":false,"params":["#,
        r#"{"id":"streaks","kind":"range","min":0.4,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"cover","kind":"range","min":0.2,"max":0.85,"step":0.01,"taste":true},"#,
        r#"{"id":"soft","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"blot","kind":"range","min":0.2,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let coral = concat!(
        r#"{"slug":"coral","frames":36,"svg":false,"params":["#,
        r#"{"id":"branches","kind":"range","min":2.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"spacing","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"spread","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"width","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wobble","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"cover","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"beneath","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"ribs","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dots","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"shadow","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let whorl = concat!(
        r#"{"slug":"whorl","frames":36,"svg":false,"params":["#,
        r#"{"id":"centres","kind":"range","min":1.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"pull","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"push","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"stripes","kind":"range","min":2.0,"max":90.0,"step":1.0,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dirs","kind":"choice","choices":["Smooth","Turbulent","Ripple"],"taste":false},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"detail","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let sear = concat!(
        r#"{"slug":"sear","frames":36,"svg":false,"params":["#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"detail","kind":"range","min":1.0,"max":5.0,"step":1.0,"taste":true},"#,
        r#"{"id":"smear","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"drag","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tear","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"steps","kind":"range","min":2.0,"max":24.0,"step":1.0,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let culture = concat!(
        r#"{"slug":"culture","frames":36,"svg":false,"params":["#,
        r#"{"id":"count","kind":"range","min":1.0,"max":90.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"fuse","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"cover","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"spread","kind":"range","min":0.05,"max":2.0,"step":0.01,"taste":true},"#,
        r#"{"id":"soft","kind":"range","min":0.02,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"textures","kind":"choice","choices":["Fine","Stipple"],"taste":false},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
        r#"{"id":"dot","kind":"range","min":1.0,"max":10.0,"step":1.0,"taste":true},"#,
    );
    let bloom = concat!(
        r#"{"slug":"bloom","frames":24,"svg":true,"params":["#,
        r#"{"id":"cols","kind":"range","min":8.0,"max":96.0,"step":1.0,"taste":true},"#,
        r#"{"id":"rings","kind":"range","min":1.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
        r#"{"id":"steps","kind":"range","min":2.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"calm","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let weave = concat!(
        r#"{"slug":"weave","frames":120,"svg":true,"params":["#,
        r#"{"id":"bands","kind":"range","min":3.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"stripe","kind":"range","min":0.5,"max":2.2,"step":0.05,"taste":true},"#,
    );
    let warp = concat!(
        r#"{"slug":"warp","frames":72,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Checker","Slash"],"taste":false},"#,
        r#"{"id":"scale","kind":"range","min":0.5,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let zig = concat!(
        r#"{"slug":"zig","frames":120,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Teeth","Chevron","Stairs","Ricrac","Waves","Scales"],"taste":false},"#,
        r#"{"id":"width","kind":"range","min":0.5,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"depth","kind":"range","min":0.2,"max":1.4,"step":0.02,"taste":true},"#,
        r#"{"id":"tooth","kind":"range","min":0.5,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"round","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let relief = concat!(
        r#"{"slug":"relief","frames":24,"svg":true,"params":["#,
        r#"{"id":"scale","kind":"range","min":12.0,"max":180.0,"step":1.0,"taste":true},"#,
        r#"{"id":"repeat","kind":"range","min":1.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"variety","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"solids","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"relief","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"accent","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let atlas = concat!(
        r#"{"slug":"atlas","frames":24,"svg":true,"params":["#,
        r#"{"id":"scale","kind":"range","min":1.0,"max":9.0,"step":0.1,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":1.6,"step":0.01,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":2.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"mix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"sets","kind":"choice","choices":["DOS","Stipple","Blocks","Code","Digits","Runes"],"taste":false},"#,
        r#"{"id":"cols","kind":"range","min":24.0,"max":260.0,"step":1.0,"taste":true},"#,
        r#"{"id":"density","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.0,"max":1.5,"step":0.01,"taste":true},"#,
    );
    let sprig = concat!(
        r#"{"slug":"sprig","frames":36,"svg":false,"params":["#,
        r#"{"id":"dirs","kind":"choice","choices":["Garden","Blooms","Leaves"],"taste":false},"#,
        r#"{"id":"count","kind":"range","min":2.0,"max":120.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"solids","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"rough","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wobble","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"detail","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let stipple = concat!(
        r#"{"slug":"stipple","frames":30,"svg":true,"params":["#,
        r#"{"id":"modesDot","kind":"choice","choices":["Lattice","Contour"],"taste":false},"#,
        r#"{"id":"lattices","kind":"choice","choices":["Square","Hex"],"taste":false},"#,
        r#"{"id":"shapes","kind":"choice","choices":["Circle","Square"],"taste":false},"#,
        r#"{"id":"res","kind":"range","min":12.0,"max":200.0,"step":1.0,"taste":true},"#,
        r#"{"id":"dot","kind":"range","min":0.05,"max":1.4,"step":0.01,"taste":true},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"cut","kind":"range","min":0.0,"max":0.95,"step":0.01,"taste":true},"#,
        r#"{"id":"jit","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"edge","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"loose","kind":"range","min":0.0,"max":0.6,"step":0.01,"taste":true},"#,
        r#"{"id":"zoom","kind":"range","min":0.5,"max":9.0,"step":0.1,"taste":true},"#,
        r#"{"id":"warp","kind":"range","min":0.0,"max":3.0,"step":0.02,"taste":true},"#,
        r#"{"id":"oct","kind":"range","min":1.0,"max":7.0,"step":1.0,"taste":true},"#,
        r#"{"id":"contrast","kind":"range","min":0.4,"max":4.0,"step":0.05,"taste":true},"#,
        r#"{"id":"syms","kind":"choice","choices":["None","Mirror","Quad","Radial"],"taste":false},"#,
        r#"{"id":"folds","kind":"range","min":2.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"accRate","kind":"range","min":0.0,"max":0.15,"step":0.001,"taste":true},"#,
    );
    let motley = concat!(
        r#"{"slug":"motley","frames":36,"svg":false,"params":["#,
        r#"{"id":"cells","kind":"range","min":16.0,"max":120.0,"step":1.0,"taste":true},"#,
        r#"{"id":"mark","kind":"range","min":0.4,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"patches","kind":"range","min":1.0,"max":60.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"ragged","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"blocks","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"sparse","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"stitch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"gaps","kind":"range","min":0.0,"max":0.5,"step":0.01,"taste":true},"#,
    );
    let oddgrid = concat!(
        r#"{"slug":"oddgrid","frames":32,"svg":true,"params":["#,
        r#"{"id":"cols","kind":"range","min":12.0,"max":110.0,"step":1.0,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":2.0,"max":60.0,"step":0.5,"taste":true},"#,
        r#"{"id":"density","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.4,"step":0.01,"taste":true},"#,
        r#"{"id":"variety","kind":"range","min":0.0,"max":1.6,"step":0.01,"taste":true},"#,
        r#"{"id":"block","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bsize","kind":"range","min":2.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"speck","kind":"range","min":0.0,"max":0.6,"step":0.01,"taste":true},"#,
        r#"{"id":"motifs","kind":"choice","choices":["None","Dot","Ring","Square","Wedge","Mixed"],"taste":false},"#,
        r#"{"id":"motifAmt","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"markSize","kind":"range","min":0.15,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"balance","kind":"range","min":-1.2,"max":1.2,"step":0.05,"taste":true},"#,
    );
    let fete = concat!(
        r#"{"slug":"fete","frames":100,"svg":true,"params":["#,
        r#"{"id":"motifs","kind":"choice","choices":["Auto","Rings","Spiral","Burst","Atom","Wave","Scribble"],"taste":false},"#,
        r#"{"id":"scale","kind":"range","min":1.2,"max":6.0,"step":0.1,"taste":true},"#,
        r#"{"id":"res","kind":"range","min":36.0,"max":120.0,"step":4.0,"taste":true},"#,
        r#"{"id":"lw","kind":"range","min":0.4,"max":2.2,"step":0.05,"taste":true},"#,
        r#"{"id":"dots","kind":"range","min":0.0,"max":80.0,"step":2.0,"taste":true},"#,
    );
    let fold = concat!(
        r#"{"slug":"fold","frames":36,"svg":false,"params":["#,
        r#"{"id":"folds","kind":"choice","choices":["Four ways","Kaleidoscope","Across","Down","None"],"taste":false},"#,
        r#"{"id":"kinds","kind":"choice","choices":["Mixed","Zebra","Staircases","Dot rows","Bursts","Chevrons","Checks"],"taste":false},"#,
        r#"{"id":"foldx","kind":"range","min":0.2,"max":0.8,"step":0.01,"taste":true},"#,
        r#"{"id":"foldy","kind":"range","min":0.2,"max":0.8,"step":0.01,"taste":true},"#,
        r#"{"id":"patches","kind":"range","min":1.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wave","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"levels","kind":"range","min":2.0,"max":8.0,"step":1.0,"taste":true},"#,
        r#"{"id":"tilt","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"px","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let quilt = concat!(
        r#"{"slug":"quilt","frames":60,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Auto","Bands","Tabs","Plaid","Dither","Steps","Zigzag","Diamond","Cross","Basket","Rings","Star","Waves","Gingham","Burst"],"taste":false},"#,
        r#"{"id":"cells","kind":"range","min":28.0,"max":72.0,"step":2.0,"taste":true},"#,
        r#"{"id":"chunk","kind":"range","min":0.7,"max":1.8,"step":0.05,"taste":true},"#,
    );
    let static_tool = concat!(
        r#"{"slug":"static","frames":60,"svg":true,"params":["#,
        r#"{"id":"regions","kind":"range","min":2.0,"max":5.0,"step":1.0,"taste":true},"#,
        r#"{"id":"res","kind":"range","min":48.0,"max":160.0,"step":4.0,"taste":true},"#,
        r#"{"id":"glitch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let splice = concat!(
        r#"{"slug":"splice","frames":36,"svg":false,"params":["#,
        r#"{"id":"dirs","kind":"choice","choices":["Columns","Rows","Blocks"],"taste":false},"#,
        r#"{"id":"slices","kind":"range","min":1.0,"max":40.0,"step":1.0,"taste":true},"#,
        r#"{"id":"shift","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tones","kind":"range","min":2.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"key","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"screen","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"mix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
        r#"{"id":"arcs","kind":"range","min":0.0,"max":24.0,"step":1.0,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bend","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let dahlia = concat!(
        r#"{"slug":"dahlia","frames":36,"svg":false,"params":["#,
        r#"{"id":"rays","kind":"range","min":8.0,"max":300.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"ragged","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"cx","kind":"range","min":0.1,"max":0.9,"step":0.01,"taste":true},"#,
        r#"{"id":"cy","kind":"range","min":0.1,"max":0.9,"step":0.01,"taste":true},"#,
        r#"{"id":"width","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dash","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"gap","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bend","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"caps","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let hiss = concat!(
        r#"{"slug":"hiss","frames":36,"svg":false,"params":["#,
        r#"{"id":"leaves","kind":"range","min":1.0,"max":5.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.4,"max":1.4,"step":0.01,"taste":true},"#,
        r#"{"id":"overlap","kind":"range","min":0.0,"max":0.5,"step":0.01,"taste":true},"#,
        r#"{"id":"tilt","kind":"range","min":-1.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"shift","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"swirl","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"black","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"checkers","kind":"range","min":0.0,"max":4.0,"step":1.0,"taste":true},"#,
        r#"{"id":"steps","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"comb","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"marks","kind":"range","min":0.0,"max":24.0,"step":1.0,"taste":true},"#,
        r#"{"id":"flecks","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"coarse","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let crowd = concat!(
        r#"{"slug":"crowd","frames":36,"svg":false,"params":["#,
        r#"{"id":"crowd","kind":"range","min":1.0,"max":48.0,"step":1.0,"taste":true},"#,
        r#"{"id":"scale","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wobble","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"blur","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"halo","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"edge","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"arrows","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"arrowSize","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let cipher = concat!(
        r#"{"slug":"cipher","frames":36,"svg":false,"params":["#,
        r#"{"id":"cells","kind":"range","min":20.0,"max":140.0,"step":1.0,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":2.0,"max":7.0,"step":1.0,"taste":true},"#,
        r#"{"id":"fields","kind":"choice","choices":["Figure","Relief","Slope"],"taste":false},"#,
        r#"{"id":"wave","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tilt","kind":"range","min":-1.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"detail","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.3,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"edges","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"flecks","kind":"range","min":0.0,"max":0.3,"step":0.01,"taste":true},"#,
        r#"{"id":"smears","kind":"range","min":0.0,"max":0.6,"step":0.01,"taste":true},"#,
        r#"{"id":"dots","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let riso = concat!(
        r#"{"slug":"riso","frames":16,"svg":true,"params":["#,
        r#"{"id":"bands","kind":"range","min":2.0,"max":5.0,"step":1.0,"taste":true},"#,
        r#"{"id":"rough","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"scrib","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"dotp","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let rise = concat!(
        r#"{"slug":"rise","frames":72,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Bottom","Top","Left","Right","Centre"],"taste":false},"#,
        r#"{"id":"count","kind":"range","min":6.0,"max":28.0,"step":1.0,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.3,"max":0.7,"step":0.01,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":1.0,"max":8.0,"step":1.0,"taste":true},"#,
        r#"{"id":"depth","kind":"range","min":0.1,"max":0.4,"step":0.01,"taste":true},"#,
    );
    let carve = concat!(
        r#"{"slug":"carve","frames":36,"svg":true,"params":["#,
        r#"{"id":"cuts","kind":"range","min":1.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"uneven","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"gap","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"mix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"pitch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
        r#"{"id":"nodes","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let specimen = concat!(
        r#"{"slug":"specimen","frames":24,"svg":true,"params":["#,
        r#"{"id":"cols","kind":"range","min":2.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"fill","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"span","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"gut","kind":"range","min":0.0,"max":0.3,"step":0.005,"taste":true},"#,
        r#"{"id":"rules","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wMarble","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wTendril","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wLoop","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wFlat","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wRing","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wFan","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wArc","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wSpike","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"mScale","kind":"range","min":0.6,"max":6.0,"step":0.1,"taste":true},"#,
        r#"{"id":"mBands","kind":"range","min":2.0,"max":7.0,"step":1.0,"taste":true},"#,
        r#"{"id":"density","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"lineW","kind":"range","min":0.4,"max":10.0,"step":0.2,"taste":true},"#,
        r#"{"id":"inkMix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let pane = concat!(
        r#"{"slug":"pane","frames":24,"svg":true,"params":["#,
        r#"{"id":"rows","kind":"range","min":1.0,"max":10.0,"step":1.0,"taste":true},"#,
        r#"{"id":"cells","kind":"range","min":1.0,"max":16.0,"step":1.0,"taste":true},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"diag","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"soft","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"spread","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let modular = concat!(
        r#"{"slug":"modular","frames":24,"svg":true,"params":["#,
        r#"{"id":"gcols","kind":"range","min":2.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"unit","kind":"range","min":2.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"merge","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"wEmpty","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wSolid","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wBlocks","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wDots","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wLines","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wGrad","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"blockFill","kind":"range","min":0.1,"max":0.9,"step":0.01,"taste":true},"#,
        r#"{"id":"dot","kind":"range","min":0.2,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"rules","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"ruleW","kind":"range","min":0.5,"max":4.0,"step":0.5,"taste":true},"#,
    );
    let prism = concat!(
        r#"{"slug":"prism","frames":12,"svg":true,"params":["#,
        r#"{"id":"cols","kind":"range","min":24.0,"max":72.0,"step":2.0,"taste":true},"#,
        r#"{"id":"streams","kind":"range","min":1.0,"max":5.0,"step":1.0,"taste":true},"#,
        r#"{"id":"reach","kind":"range","min":0.3,"max":0.85,"step":0.01,"taste":true},"#,
        r#"{"id":"fringe","kind":"range","min":0.5,"max":1.6,"step":0.01,"taste":true},"#,
        r#"{"id":"dots","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let parcel = concat!(
        r#"{"slug":"parcel","frames":72,"svg":true,"params":["#,
        r#"{"id":"cells","kind":"range","min":8.0,"max":28.0,"step":1.0,"taste":true},"#,
        r#"{"id":"cover","kind":"range","min":0.2,"max":0.8,"step":0.01,"taste":true},"#,
        r#"{"id":"chunk","kind":"range","min":0.5,"max":2.0,"step":0.05,"taste":true},"#,
        r#"{"id":"grids","kind":"range","min":0.0,"max":8.0,"step":1.0,"taste":true},"#,
        r#"{"id":"blends","kind":"choice","choices":["Multiply","Normal"],"taste":false},"#,
    );
    let tokens = concat!(
        r#"{"slug":"tokens","frames":36,"svg":true,"params":["#,
        r#"{"id":"cols","kind":"range","min":4.0,"max":120.0,"step":1.0,"taste":true},"#,
        r#"{"id":"ruleEvery","kind":"range","min":1.0,"max":10.0,"step":1.0,"taste":true},"#,
        r#"{"id":"grid","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"count","kind":"range","min":1.0,"max":400.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wRun","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wBlock","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wPlus","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wOne","kind":"range","min":0.0,"max":100.0,"step":1.0,"taste":true},"#,
        r#"{"id":"clump","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"align","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"runLen","kind":"range","min":2.0,"max":12.0,"step":1.0,"taste":true},"#,
        r#"{"id":"upright","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"sizeT","kind":"range","min":0.03,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"svar","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"hier","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"square","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"strokeW","kind":"range","min":0.0,"max":8.0,"step":0.5,"taste":true},"#,
        r#"{"id":"hollow","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"kin","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"break","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let optic = concat!(
        r#"{"slug":"optic","frames":72,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Auto","Diamond","Circle","Peak","Square"],"taste":false},"#,
        r#"{"id":"count","kind":"range","min":6.0,"max":28.0,"step":1.0,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.3,"max":0.7,"step":0.01,"taste":true},"#,
        r#"{"id":"sizeF","kind":"range","min":0.4,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"levels","kind":"range","min":1.0,"max":3.0,"step":1.0,"taste":true},"#,
    );
    let vee = concat!(
        r#"{"slug":"vee","frames":72,"svg":true,"params":["#,
        r#"{"id":"styles","kind":"choice","choices":["Auto","Chevron","Quad","Diagonal","Cross"],"taste":false},"#,
        r#"{"id":"angle","kind":"range","min":15.0,"max":75.0,"step":1.0,"taste":true},"#,
        r#"{"id":"width","kind":"range","min":0.4,"max":2.2,"step":0.05,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.15,"max":0.85,"step":0.01,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":1.0,"max":4.0,"step":1.0,"taste":true},"#,
    );
    let sampler = concat!(
        r#"{"slug":"sampler","frames":24,"svg":true,"params":["#,
        r#"{"id":"grid","kind":"range","min":4.0,"max":48.0,"step":1.0,"taste":true},"#,
        r#"{"id":"bands","kind":"range","min":1.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"mix","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"turn","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"density","kind":"range","min":0.1,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"weight","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
    );
    let totem = concat!(
        r#"{"slug":"totem","frames":24,"svg":true,"params":["#,
        r#"{"id":"border","kind":"range","min":0.0,"max":0.4,"step":0.005,"taste":true},"#,
        r#"{"id":"mat","kind":"range","min":0.0,"max":0.7,"step":0.01,"taste":true},"#,
        r#"{"id":"matGrain","kind":"range","min":1.0,"max":6.0,"step":1.0,"taste":true},"#,
        r#"{"id":"keyline","kind":"range","min":0.0,"max":10.0,"step":1.0,"taste":true},"#,
        r#"{"id":"regions","kind":"range","min":1.0,"max":30.0,"step":1.0,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":16.0,"max":220.0,"step":1.0,"taste":true},"#,
        r#"{"id":"mirror","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"variety","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"core","kind":"range","min":0.0,"max":0.6,"step":0.01,"taste":true},"#,
        r#"{"id":"coreRings","kind":"range","min":0.0,"max":8.0,"step":1.0,"taste":true},"#,
    );
    let filament = concat!(
        r#"{"slug":"filament","frames":30,"svg":false,"params":["#,
        r#"{"id":"zoom","kind":"range","min":0.3,"max":6.0,"step":0.05,"taste":true},"#,
        r#"{"id":"turn","kind":"range","min":0.2,"max":5.0,"step":0.05,"taste":true},"#,
        r#"{"id":"oct","kind":"range","min":1.0,"max":6.0,"step":1.0,"taste":true},"#,
        r#"{"id":"curl","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tangle","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bundles","kind":"range","min":1.0,"max":90.0,"step":1.0,"taste":true},"#,
        r#"{"id":"per","kind":"range","min":1.0,"max":40.0,"step":1.0,"taste":true},"#,
        r#"{"id":"clump","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tight","kind":"range","min":0.005,"max":0.2,"step":0.005,"taste":true},"#,
        r#"{"id":"len","kind":"range","min":20.0,"max":600.0,"step":10.0,"taste":true},"#,
        r#"{"id":"step","kind":"range","min":1.0,"max":10.0,"step":0.2,"taste":true},"#,
        r#"{"id":"wgt","kind":"range","min":0.2,"max":8.0,"step":0.1,"taste":true},"#,
        r#"{"id":"wvar","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"hier","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"hair","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"mstyles","kind":"choice","choices":["Symbols","Beads","Both"],"taste":false},"#,
        r#"{"id":"mark","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bead","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"bgap","kind":"range","min":2.0,"max":14.0,"step":1.0,"taste":true},"#,
        r#"{"id":"msize","kind":"range","min":2.0,"max":26.0,"step":0.5,"taste":true},"#,
    );
    let chaff = concat!(
        r#"{"slug":"chaff","frames":36,"svg":false,"params":["#,
        r#"{"id":"count","kind":"range","min":4.0,"max":400.0,"step":1.0,"taste":true},"#,
        r#"{"id":"size","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"vary","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"apart","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"shapes","kind":"choice","choices":["Crescent","Leaf","Bar"],"taste":false},"#,
        r#"{"id":"curve","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"slim","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"taper","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"mottle","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"coarse","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    let strand = concat!(
        r#"{"slug":"strand","frames":36,"svg":false,"params":["#,
        r#"{"id":"count","kind":"range","min":1.0,"max":40.0,"step":1.0,"taste":true},"#,
        r#"{"id":"len","kind":"range","min":3.0,"max":90.0,"step":1.0,"taste":true},"#,
        r#"{"id":"wander","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"branch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"thick","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"rod","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"notch","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"rough","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"offset","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"edge","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"tex","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":true},"#,
        r#"{"id":"texKinds","kind":"choice","choices":["Stipple","Drag","Screen"],"taste":false},"#,
        r#"{"id":"grain","kind":"range","min":0.0,"max":1.0,"step":0.01,"taste":false},"#,
    );
    assert_eq!(
        text(&out.stdout),
        format!(
            "[{sonar}{CHASSIS_LISTING}]}},{husk}{CHASSIS_LISTING}]}},{vein}{CHASSIS_LISTING}]}},{aura}{CHASSIS_LISTING}]}},{kiosk}{CHASSIS_LISTING}]}},{frond}{CHASSIS_LISTING}]}},{benday}{CHASSIS_LISTING}]}},{terrain}{CHASSIS_LISTING}]}},{stitch}{CHASSIS_LISTING}]}},{pith}{CHASSIS_LISTING}]}},{mosh}{CHASSIS_LISTING}]}},{mist}{CHASSIS_LISTING}]}},{coral}{CHASSIS_LISTING}]}},{whorl}{CHASSIS_LISTING}]}},{sear}{CHASSIS_LISTING}]}},{culture}{CHASSIS_LISTING}]}},{bloom}{CHASSIS_LISTING}]}},{weave}{CHASSIS_LISTING}]}},{warp}{CHASSIS_LISTING}]}},{zig}{CHASSIS_LISTING}]}},{relief}{CHASSIS_LISTING}]}},{atlas}{CHASSIS_LISTING}]}},{sprig}{CHASSIS_LISTING}]}},{stipple}{CHASSIS_LISTING}]}},{motley}{CHASSIS_LISTING}]}},{oddgrid}{CHASSIS_LISTING}]}},{fete}{CHASSIS_LISTING}]}},{fold}{CHASSIS_LISTING}]}},{quilt}{CHASSIS_LISTING}]}},{static_tool}{CHASSIS_LISTING}]}},{splice}{CHASSIS_LISTING}]}},{dahlia}{CHASSIS_LISTING}]}},{hiss}{CHASSIS_LISTING}]}},{crowd}{CHASSIS_LISTING}]}},{cipher}{CHASSIS_LISTING}]}},{riso}{CHASSIS_LISTING}]}},{rise}{CHASSIS_LISTING}]}},{carve}{CHASSIS_LISTING}]}},{specimen}{CHASSIS_LISTING}]}},{pane}{CHASSIS_LISTING}]}},{modular}{CHASSIS_LISTING}]}},{prism}{CHASSIS_LISTING}]}},{parcel}{CHASSIS_LISTING}]}},{tokens}{CHASSIS_LISTING}]}},{optic}{CHASSIS_LISTING}]}},{vee}{CHASSIS_LISTING}]}},{sampler}{CHASSIS_LISTING}]}},{totem}{CHASSIS_LISTING}]}},{filament}{CHASSIS_LISTING}]}},{chaff}{CHASSIS_LISTING}]}},{strand}{CHASSIS_LISTING}]}}]\n"
        )
    );
}

#[test]
fn tools_lists_slugs_frames_and_ranges_for_humans() {
    let out = tirage(&["tools"], b"");
    assert_eq!(out.status.code(), Some(0));
    let listing = text(&out.stdout);
    assert!(listing.starts_with("sonar  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nhusk  36 frames\n"), "{listing}");
    assert!(listing.contains("\nvein  36 frames, svg\n"), "{listing}");
    assert!(listing.contains("\naura  72 frames\n"), "{listing}");
    assert!(listing.contains("\nfrond  36 frames\n"), "{listing}");
    assert!(listing.contains("\natlas  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\noddgrid  32 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nfete  100 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nfold  36 frames\n"), "{listing}");
    assert!(listing.contains("\nquilt  60 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nstatic  60 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nsplice  36 frames\n"), "{listing}");
    assert!(listing.contains("\ndahlia  36 frames\n"), "{listing}");
    assert!(listing.contains("\nhiss  36 frames\n"), "{listing}");
    assert!(listing.contains("\ncrowd  36 frames\n"), "{listing}");
    assert!(listing.contains("\ncipher  36 frames\n"), "{listing}");
    assert!(listing.contains("\nriso  16 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nrise  72 frames, svg\n"), "{listing}");
    assert!(listing.contains("\ncarve  36 frames, svg\n"), "{listing}");
    assert!(
        listing.contains("\nspecimen  24 frames, svg\n"),
        "{listing}"
    );
    assert!(listing.contains("\npane  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nmodular  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nprism  12 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nparcel  72 frames, svg\n"), "{listing}");
    assert!(listing.contains("\ntokens  36 frames, svg\n"), "{listing}");
    assert!(listing.contains("\noptic  72 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nvee  72 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nsampler  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\ntotem  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nfilament  30 frames\n"), "{listing}");
    assert!(listing.contains("\nchaff  36 frames\n"), "{listing}");
    assert!(listing.contains("\nstrand  36 frames\n"), "{listing}");
    assert!(listing.contains("\nmotley  36 frames\n"), "{listing}");
    assert!(listing.contains("\nstipple  30 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nsprig  36 frames\n"), "{listing}");
    assert!(listing.contains("\nmist  72 frames\n"), "{listing}");
    assert!(listing.contains("\ncoral  36 frames\n"), "{listing}");
    assert!(listing.contains("\nwhorl  36 frames\n"), "{listing}");
    assert!(listing.contains("\nsear  36 frames\n"), "{listing}");
    assert!(listing.contains("\nculture  36 frames\n"), "{listing}");
    assert!(listing.contains("\nmosh  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nbloom  24 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nweave  120 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nwarp  72 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nzig  120 frames, svg\n"), "{listing}");
    assert!(listing.contains("\nrelief  24 frames, svg\n"), "{listing}");
    for line in [
        "  grid        40..=320  step 2\n",
        "  grnVignette 0..=1     step 0.05 [no Taste]\n",
        "  grainTog    on/off [no Taste]\n",
        "  grnBlends   Add, Overlay, Soft light, Multiply, Screen [no Taste]\n",
        "  dthKinds    Bayer 8, Bayer 4, Noise [no Taste]\n",
        "  levels      3..=14    step 1\n",
        "  flows       Marble, Swirl, Ripple [no Taste]\n",
        "  styles      Auto, Clouds, Mesh, Sweep [no Taste]\n",
        "  dirs        Potted, Bouquet, Fronds [no Taste]\n",
    ] {
        assert!(listing.contains(line), "{line:?} in\n{listing}");
    }
}

#[test]
fn tools_marks_the_parameters_taste_cannot_bound() {
    let listing = text(&tirage(&["tools"], b"").stdout);
    let section = |slug: &str| {
        let header = format!("{slug}  ");
        listing
            .lines()
            .skip_while(|line| !line.starts_with(&header))
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .collect::<Vec<_>>()
    };
    let terrain = section("terrain");
    for id in ["grain ", "ditherTog ", "grnSize "] {
        let line = terrain
            .iter()
            .find(|l| l.trim_start().starts_with(id))
            .unwrap();
        assert!(line.ends_with(" [no Taste]"), "{line}");
    }
    let scale = terrain
        .iter()
        .find(|l| l.trim_start().starts_with("scale "))
        .unwrap();
    assert!(!scale.contains("[no Taste]"), "{scale}");
    let aura = section("aura");
    let styles = aura
        .iter()
        .find(|l| l.trim_start().starts_with("styles "))
        .unwrap();
    assert!(styles.ends_with(" [no Taste]"), "{styles}");

    let listings = json(&tirage(&["tools", "--json"], b"").stdout);
    for listing in listings.as_array().unwrap() {
        let tool = tirage::Tool::from_slug(listing["slug"].as_str().unwrap()).unwrap();
        for param in listing["params"].as_array().unwrap() {
            let id = param["id"].as_str().unwrap();
            assert_eq!(param["taste"], tool.takes_taste(id), "{} {id}", tool.slug());
        }
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
    let repeated = dir.join("repeated.json");
    std::fs::write(&repeated, r#"{"tool":"sonar","level":[0,1],"level":[0,1]}"#).unwrap();
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
            "42",
            "--taste",
            repeated.to_str().unwrap(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("Taste JSON: duplicate field `level` at line 1 column 44"),
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
        (Some(1), "error: cannot read missing.json: no such file\n")
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
fn derive_rejects_a_palette_for_prism_and_hints_to_pin_a_tool() {
    assert_eq!(
        tirage::derive(24, tirage::ToolPin::Any).tool(),
        tirage::Tool::Prism
    );
    for args in [
        &["derive", "--seed", "24", "--palette", "#000000,#ffffff"][..],
        &[
            "derive",
            "--seed",
            "24",
            "--tool",
            "prism",
            "--palette",
            "#000000,#ffffff",
        ],
    ] {
        let out = tirage(args, b"");
        assert_eq!(out.status.code(), Some(2));
        let stderr = text(&out.stderr);
        assert!(
            stderr.starts_with("error: prism: takes no Palette\n"),
            "{stderr}"
        );
        assert!(
            stderr.contains("pin a Tool that takes a Palette with --tool <slug>"),
            "{stderr}"
        );
        assert!(out.stdout.is_empty());
    }
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

#[test]
fn aliases_run_the_same_command_and_show_in_help() {
    let tools = tirage(&["tools"], b"").stdout;
    for alias in ["t", "ls"] {
        assert_eq!(tirage(&[alias], b"").stdout, tools, "{alias}");
    }
    assert_eq!(
        tirage(&["d", "--seed", "42"], b"").stdout,
        tirage(&["derive", "--seed", "42"], b"").stdout
    );
    let out = tirage(&["r", "--size", "9x16", "-o", "-"], &recipe());
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(out.stdout.starts_with(b"\x89PNG"));
    let help = text(&tirage(&["--help"], b"").stdout);
    for alias in ["[alias: d]", "[alias: r]", "[aliases: t, ls]"] {
        assert!(help.contains(alias), "{alias} in\n{help}");
    }
}

#[test]
fn a_mistyped_command_suggests_the_closest() {
    let out = tirage(&["rendr"], b"");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("'render'"),
        "{}",
        text(&out.stderr)
    );
}

#[test]
fn completions_name_every_subcommand_and_alias() {
    let out = tirage(&["completions", "bash"], b"");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let script = text(&out.stdout);
    for word in [
        "derive",
        "render",
        "tools",
        "completions",
        "ls",
        "sonar",
        "kiosk",
    ] {
        assert!(script.contains(word), "{word}");
    }
}

#[test]
fn help_links_to_the_readme_and_issues() {
    let help = text(&tirage(&["--help"], b"").stdout);
    assert!(
        help.contains("https://github.com/espadat-studio/tirage#readme"),
        "{help}"
    );
    assert!(
        help.trim_end()
            .ends_with("https://github.com/espadat-studio/tirage/issues"),
        "{help}"
    );
}

fn last_line(bytes: &[u8]) -> String {
    text(bytes)
        .trim_end()
        .lines()
        .last()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn errors_end_with_a_hint() {
    let major = text(&recipe()).replacen(r#""tirage":0"#, r#""tirage":9"#, 1);
    let out_of_range = text(&recipe()).replacen(r#""level":0.64"#, r#""level":1.4"#, 1);
    let cases: [(&[&str], &[u8], &str); 6] = [
        (
            &["render", "--size", "9x16", "-o", "-"],
            major.as_bytes(),
            "hint: derive it again from its Seed with this build: tirage derive --seed <N>",
        ),
        (
            &["render", "--size", "9x16", "-o", "-"],
            out_of_range.as_bytes(),
            "hint: run 'tirage tools' to see each Parameter's range and step",
        ),
        (
            &["derive", "--seed", "1", "--tool", "sonr"],
            b"",
            "hint: did you mean 'sonar'?",
        ),
        (
            &["derive", "--seed", "1", "--tool", "zzzzzz"],
            b"",
            "hint: run 'tirage tools' to list Tools",
        ),
        (
            &["derive", "--seed", "1", "--palette", "#000000,#fff"],
            b"",
            "hint: pass at least 2 comma-separated #rrggbb inks, like '#000000,#ffffff'",
        ),
        (
            &[
                "derive",
                "--seed",
                "1",
                "--tool",
                "aura",
                "--palette",
                "#000000,#111111,#222222,#333333,#444444",
            ],
            b"",
            "hint: pass at most 4 inks for aura",
        ),
    ];
    for (args, stdin, hint) in cases {
        let out = tirage(args, stdin);
        assert_eq!(
            last_line(&out.stderr),
            hint,
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
}

#[test]
fn taste_errors_hint_at_parameters() {
    let dir = std::env::temp_dir().join(format!("tirage-hint-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cases = [
        (
            r#"{"tool":"sonar","levle":[0,1]}"#,
            "hint: did you mean 'level'?",
        ),
        (
            r#"{"tool":"sonar","level":[0,1.4]}"#,
            "hint: run 'tirage tools' to see each Parameter's range and step",
        ),
        (
            r#"{"tool":"terrain","grain":[0,1]}"#,
            "hint: Taste bounds only the Parameters 'tirage tools' does not mark [no Taste]",
        ),
        (
            r#"{"tool":"strand","grain":[0,1]}"#,
            "hint: Taste bounds only the Parameters 'tirage tools' does not mark [no Taste]",
        ),
        (
            r#"{"tool":"terrain","scal":[0,1]}"#,
            "hint: did you mean 'scale'?",
        ),
        (
            r#"{"tool":"terrain","grainn":[0,1]}"#,
            "hint: run 'tirage tools' to list terrain's Parameters",
        ),
        (
            r#"{"tool":"sonar","#,
            "hint: Taste bounds are a JSON object of a tool and [min, max] per Parameter id",
        ),
    ];
    for (json, hint) in cases {
        let path = dir.join("taste.json");
        std::fs::write(&path, json).unwrap();
        let out = tirage(
            &["derive", "--seed", "1", "--taste", path.to_str().unwrap()],
            b"",
        );
        assert_eq!(out.status.code(), Some(2), "{json}");
        assert_eq!(
            last_line(&out.stderr),
            hint,
            "{json}: {}",
            text(&out.stderr)
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn errors_are_red_on_a_colour_terminal_only() {
    let failing = "tirage render missing.json --size 9x16 -o x.png";
    let out = on_tty(failing);
    assert!(
        text(&out.stdout).contains("\x1b[1;31merror:\x1b[0m"),
        "{:?}",
        text(&out.stdout)
    );
    for script in [
        format!("NO_COLOR=1 {failing}"),
        format!("TERM=dumb {failing}"),
        format!("{failing} --no-color"),
        "tirage --no-color derive --seed 1 --tool sonr".to_owned(),
    ] {
        let out = on_tty(&script);
        assert!(text(&out.stdout).contains("error:"), "{script}");
        assert!(
            !text(&out.stdout).contains('\x1b'),
            "{script}: {:?}",
            text(&out.stdout)
        );
    }
    assert!(
        !text(
            &tirage(
                &["render", "missing.json", "--size", "9x16", "-o", "x.png"],
                b""
            )
            .stderr
        )
        .contains('\x1b')
    );
}

#[test]
fn derive_prompts_for_a_missing_seed_on_a_terminal() {
    let dir = scratch("seed");
    let out = on_tty_typing(
        "tirage derive --tool sonar > recipe.json",
        "abc\r42\r",
        &dir,
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    assert_eq!(std::fs::read(dir.join("recipe.json")).unwrap(), recipe());
    assert!(
        text(&out.stdout).contains("→ tirage derive --tool sonar --seed 42"),
        "{}",
        text(&out.stdout)
    );
    let out = on_tty_typing("tirage derive > random.json", "\r", &dir);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    json(&std::fs::read(dir.join("random.json")).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn render_derives_a_recipe_inline_like_derive_piped_into_render() {
    let dir = scratch("inline");
    let out = on_tty_typing("tirage render --size 9x16", "42\rso\r\r", &dir);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let piped = tirage(&["render", "--size", "9x16", "-o", "-"], &recipe()).stdout;
    assert_eq!(std::fs::read(dir.join("sonar-42.png")).unwrap(), piped);
    assert!(
        text(&out.stdout).contains(
            "→ tirage derive --seed 42 --tool sonar | tirage render --size 9x16 -o sonar-42.png"
        ),
        "{}",
        text(&out.stdout)
    );
    let out = on_tty_typing("tirage render --size 9x16 -o dealt.png", "42\r\r", &dir);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let dealt = tirage(&["derive", "--seed", "42"], b"").stdout;
    let piped = tirage(&["render", "--size", "9x16", "-o", "-"], &dealt).stdout;
    assert_eq!(std::fs::read(dir.join("dealt.png")).unwrap(), piped);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn render_prompts_for_a_preset_or_custom_size() {
    let dir = scratch("size");
    std::fs::write(dir.join("recipe.json"), recipe()).unwrap();
    for (keys, size) in [("\r", (1080, 1920)), ("cus\r0x5\r90x160\r", (90, 160))] {
        let out = on_tty_typing("tirage render recipe.json -o out.png", keys, &dir);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{keys:?}: {}",
            text(&out.stdout)
        );
        let png = image::open(dir.join("out.png")).unwrap();
        assert_eq!((png.width(), png.height()), size, "{keys:?}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_output_prompt_asks_before_overwriting() {
    let dir = scratch("overwrite");
    std::fs::write(dir.join("recipe.json"), recipe()).unwrap();
    std::fs::write(dir.join("out.png"), "keep").unwrap();
    let out = on_tty_typing(
        "tirage render recipe.json --size 9x16",
        "\r\rother.png\r",
        &dir,
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    assert_eq!(std::fs::read(dir.join("out.png")).unwrap(), b"keep");
    image::open(dir.join("other.png")).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn esc_at_a_prompt_exits_130_and_writes_nothing() {
    let dir = scratch("esc");
    std::fs::write(dir.join("recipe.json"), recipe()).unwrap();
    std::fs::write(dir.join("out.png"), "keep").unwrap();
    for (script, keys, message) in [
        ("tirage render --size 9x16", "42\r\x1b", "no Tool selected"),
        (
            "tirage render recipe.json -o new.png",
            "\x1b",
            "no frame size selected",
        ),
        (
            "tirage render recipe.json --size 9x16",
            "\r\x1b",
            "no output path selected",
        ),
    ] {
        let out = on_tty_typing(script, keys, &dir);
        assert_eq!(
            out.status.code(),
            Some(130),
            "{script}: {}",
            text(&out.stdout)
        );
        assert!(
            text(&out.stdout).contains(message),
            "{script}: {}",
            text(&out.stdout)
        );
    }
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    files.sort();
    assert_eq!(files, ["out.png", "recipe.json"]);
    assert_eq!(std::fs::read(dir.join("out.png")).unwrap(), b"keep");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn prompts_are_off_without_a_full_terminal() {
    let dir = scratch("no-input");
    let seed = "no Seed given, pass --seed <N>, like --seed 42";
    for script in [
        "tirage derive --no-input",
        "tirage --no-input derive",
        "TIRAGE_NO_INPUT=1 tirage derive",
        "TERM=dumb tirage derive",
    ] {
        let out = on_tty_typing(script, "", &dir);
        assert_eq!(out.status.code(), Some(2), "{script}");
        assert!(
            text(&out.stdout).contains(seed),
            "{script}: {}",
            text(&out.stdout)
        );
    }
    let out = on_tty_typing("tirage derive 2>err.log", "", &dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        std::fs::read_to_string(dir.join("err.log"))
            .unwrap()
            .contains(seed)
    );
    let cases: [(&[&str], &[u8], &str); 3] = [
        (&["derive"], b"", seed),
        (
            &["render", "-o", "-"],
            &recipe(),
            "no frame size given, pass --size <W>x<H>, like 1080x1920",
        ),
        (
            &["render", "--size", "9x16"],
            &recipe(),
            "no output path given, pass -o FILE, or -o - for stdout",
        ),
    ];
    for (args, stdin, message) in cases {
        let out = tirage(args, stdin);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(
            text(&out.stderr).contains(message),
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn short_help_stays_short_and_long_help_shows_shapes() {
    let short = text(&tirage(&["render", "-h"], b"").stdout);
    let long = text(&tirage(&["render", "--help"], b"").stdout);
    assert!(short.contains("prompted on a terminal"), "{short}");
    assert!(!short.contains("1080x1350"), "{short}");
    assert!(long.contains("1080x1350 portrait"), "{long}");
    let concise = text(&on_tty("tirage").stdout);
    assert!(concise.contains("prompt on a terminal"), "{concise}");
}

fn flags_in_help(help: &str) -> BTreeSet<String> {
    help.lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with('-'))
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

fn flags_on_page(page: &str) -> BTreeSet<String> {
    page.split("```")
        .step_by(2)
        .flat_map(|prose| prose.split('`').skip(1).step_by(2))
        .filter_map(|span| span.split_whitespace().next())
        .filter(|word| word.starts_with('-') && word.len() > 1)
        .map(str::to_owned)
        .collect()
}

#[test]
fn docs_pages_list_every_flag_in_help_and_no_other() {
    let pages =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/src/content/docs/cli-reference");
    let commands: [(&str, &[&str]); 6] = [
        ("tirage", &["--help"]),
        ("derive", &["derive", "--help"]),
        ("render", &["render", "--help"]),
        ("roll", &["roll", "--help"]),
        ("tools", &["tools", "--help"]),
        ("completions", &["completions", "--help"]),
    ];
    let encode: &[(&str, &[&str])] = if cfg!(feature = "encode") {
        &[("encode", &["encode", "--help"])]
    } else {
        &[]
    };
    for (name, args) in commands.iter().chain(encode) {
        let help = text(&tirage(args, b"").stdout);
        let path = pages.join(format!("{name}.md"));
        let page =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(flags_in_help(&help), flags_on_page(&page), "{name}");
    }
}

#[test]
fn every_shell_example_on_the_concepts_pages_runs() {
    let concepts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/src/content/docs/concepts");
    let path = format!(
        "{}:{}",
        Path::new(BIN).parent().unwrap().display(),
        std::env::var("PATH").unwrap()
    );
    let dir = scratch("concepts");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&concepts)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|page| page.extension().is_some_and(|ext| ext == "md"))
        .collect();
    pages.sort();
    assert!(!pages.is_empty());
    for page in pages {
        let markdown = std::fs::read_to_string(&page).unwrap();
        let script = shell_blocks(&markdown).join("\n");
        let cwd = dir.join(page.file_stem().unwrap());
        std::fs::create_dir_all(&cwd).unwrap();
        let out = Command::new("sh")
            .args(["-ec", &script])
            .current_dir(&cwd)
            .env("PATH", &path)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}:\n{script}\n{}",
            page.display(),
            text(&out.stderr)
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

fn shell_blocks(markdown: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in markdown.lines() {
        match (in_block, line.trim_end()) {
            (false, "```sh") => in_block = true,
            (true, "```") => in_block = false,
            (true, line) => lines.push(line),
            (false, _) => {}
        }
    }
    lines
}

fn loop_seed() -> u64 {
    (0..)
        .find(|&seed| tirage::derive(seed, tirage::ToolPin::Any).tool().frames() > 1)
        .unwrap()
}

fn roll_in(dir: &Path, args: &[&str], seed: u64) -> Output {
    Command::new(BIN)
        .arg("roll")
        .args(args)
        .env("TIRAGE_ROLL_SEED", seed.to_string())
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn reproduce(stderr: &[u8], dir: &Path) {
    let line = text(stderr);
    let command = line
        .lines()
        .find_map(|line| line.strip_prefix("→ "))
        .unwrap_or_else(|| panic!("no reproduce line in\n{line}"));
    let status = Command::new("sh")
        .args(["-ec", &command.replace("tirage ", &format!("'{BIN}' "))])
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "{command}");
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[cfg(not(feature = "encode"))]
#[test]
fn roll_without_encode_writes_the_still_of_a_loop_as_a_story_png_its_reproduce_line_remakes() {
    let seed = loop_seed();
    let tool = tirage::derive(seed, tirage::ToolPin::Any).tool().slug();
    let name = format!("{tool}-{seed}.png");
    let dir = scratch("roll-still");
    let out = roll_in(&dir, &[], seed);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(out.stdout.is_empty());
    assert_eq!(
        text(&out.stderr),
        format!("→ tirage derive --seed {seed} | tirage render --size 1080x1920 -o {name}\n")
    );
    assert_eq!(files_in(&dir), std::slice::from_ref(&name));
    let rolled = std::fs::read(dir.join(&name)).unwrap();
    let png = image::load_from_memory(&rolled).unwrap();
    assert_eq!((png.width(), png.height()), (1080, 1920));
    let again = scratch("roll-still-again");
    reproduce(&out.stderr, &again);
    assert_eq!(std::fs::read(again.join(&name)).unwrap(), rolled);
    std::fs::remove_dir_all(dir).unwrap();
    std::fs::remove_dir_all(again).unwrap();
}

#[test]
fn roll_refuses_an_existing_target_and_writes_nothing() {
    let seed = loop_seed();
    let tool = tirage::derive(seed, tirage::ToolPin::Any).tool().slug();
    let extension = if cfg!(feature = "encode") {
        "mp4"
    } else {
        "png"
    };
    let name = format!("{tool}-{seed}.{extension}");
    let dir = scratch("roll-exists");
    std::fs::write(dir.join(&name), b"mine").unwrap();
    let out = roll_in(&dir, &[], seed);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains(&format!("error: {name} exists\nhint: ")),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(files_in(&dir), std::slice::from_ref(&name));
    assert_eq!(std::fs::read(dir.join(&name)).unwrap(), b"mine");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn roll_never_prompts() {
    let seed = loop_seed();
    let dir = scratch("roll-no-input");
    let quiet = roll_in(&dir, &["--no-input"], seed);
    assert_eq!(quiet.status.code(), Some(0), "{}", text(&quiet.stderr));
    let piped = scratch("roll-piped");
    let out = roll_in(&piped, &[], seed);
    let last = |stderr: &[u8]| text(stderr).lines().last().map(str::to_owned);
    assert_eq!(last(&out.stderr), last(&quiet.stderr));
    assert_eq!(files_in(&piped), files_in(&dir));
    let tty = scratch("roll-tty");
    let out = on_tty_typing("tirage roll", "", &tty);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    assert_eq!(files_in(&tty).len(), 1);
    for dir in [dir, piped, tty] {
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn roll_shows_in_help_with_examples() {
    let help = text(&tirage(&["--help"], b"").stdout);
    assert!(help.contains("  roll "), "{help}");
    let help = text(&tirage(&["roll", "--help"], b"").stdout);
    assert!(help.starts_with("Examples:\n  tirage roll\n"), "{help}");
    let script = text(&tirage(&["completions", "bash"], b"").stdout);
    assert!(script.contains("roll"), "{script}");
}

#[cfg(feature = "encode")]
mod encode {
    use super::*;

    fn sonar_mp4() -> Vec<u8> {
        tirage_encode::encode(
            &tirage::derive(42, tirage::ToolPin::Tool(tirage::Tool::Sonar)),
            tirage_encode::WIDTH,
            tirage_encode::HEIGHT,
        )
        .unwrap()
    }

    #[test]
    fn derive_pipes_into_encode_as_the_mp4_the_library_encodes() {
        let dir = scratch("encode-pipe");
        let mut derive = Command::new(BIN)
            .args(["derive", "--seed", "42", "--tool", "sonar"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let out = Command::new(BIN)
            .args(["encode", "-o", "out.mp4"])
            .current_dir(&dir)
            .stdin(derive.stdout.take().unwrap())
            .output()
            .unwrap();
        assert_eq!(derive.wait().unwrap().code(), Some(0));
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert!(out.stdout.is_empty());
        assert_eq!(std::fs::read(dir.join("out.mp4")).unwrap(), sonar_mp4());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn encode_reads_a_recipe_file_and_streams_the_mp4_to_stdout() {
        let dir = scratch("encode-file");
        let input = dir.join("recipe.json");
        std::fs::write(&input, recipe()).unwrap();
        let out = tirage(&["encode", input.to_str().unwrap(), "-o", "-"], b"");
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert_eq!(out.stdout, sonar_mp4());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn encode_size_writes_the_mp4_the_library_encodes_at_that_size() {
        let out = tirage(&["encode", "--size", "360x640", "-o", "-"], &recipe());
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        let recipe = tirage::derive(42, tirage::ToolPin::Tool(tirage::Tool::Sonar));
        assert_eq!(
            out.stdout,
            tirage_encode::encode(&recipe, 360, 640).unwrap()
        );
    }

    #[test]
    fn encode_refuses_an_odd_or_out_of_range_size() {
        for size in ["361x640", "360x641", "14x640", "360x2050"] {
            let out = tirage(&["encode", "--size", size, "-o", "-"], &recipe());
            assert_eq!(out.status.code(), Some(2), "{size}");
            assert!(
                text(&out.stderr).contains("each edge must be even and in 16..=2048, like 360x640"),
                "{size}: {}",
                text(&out.stderr)
            );
        }
    }

    #[test]
    fn encode_refuses_mp4_bytes_on_a_terminal() {
        let out = on_tty("tirage derive --seed 42 --tool sonar | tirage encode -o -");
        assert_eq!(out.status.code(), Some(2));
        assert!(
            text(&out.stdout).contains("refusing to write an MP4 to a terminal"),
            "{}",
            text(&out.stdout)
        );
        assert!(
            text(&out.stdout).contains("Usage: tirage encode"),
            "{}",
            text(&out.stdout)
        );
    }

    #[test]
    fn encode_prompts_for_a_seed_a_loop_tool_and_an_output_path() {
        let dir = scratch("encode-prompts");
        let out = on_tty_typing("tirage encode", "42\rso\r\r", &dir);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
        assert_eq!(
            std::fs::read(dir.join("sonar-42.mp4")).unwrap(),
            sonar_mp4()
        );
        let screen = text(&out.stdout);
        assert!(
            screen
                .contains("→ tirage derive --seed 42 --tool sonar | tirage encode -o sonar-42.mp4"),
            "{screen}"
        );
        assert!(!screen.contains("deal from Seed"), "{screen}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_loop_over_the_cap_fails_with_the_cap_in_the_message() {
        let dir = scratch("encode-cap");
        let mut recipe = tirage::derive(1, tirage::ToolPin::Tool(tirage::Tool::Frond));
        let tirage::Params::Frond(p) = recipe.params_mut() else {
            unreachable!()
        };
        p.set_plant(tirage::Plant::Bouquet);
        p.set_masses(6).unwrap();
        p.set_growth(0.75).unwrap();
        p.set_detail(1.0).unwrap();
        p.set_noise(1.0).unwrap();
        p.set_patch(1.0).unwrap();
        p.set_coarse(0.25).unwrap();
        p.set_breakup(0.5).unwrap();
        p.set_circles(1.0).unwrap();
        p.set_rules(1.0).unwrap();
        let output = dir.join("out.mp4");
        let out = tirage(
            &["encode", "-o", output.to_str().unwrap()],
            recipe.to_json().as_bytes(),
        );
        assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
        assert!(
            text(&out.stderr).contains("over the 1500000 byte cap"),
            "{}",
            text(&out.stderr)
        );
        assert!(!output.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn encode_prompts_are_off_without_a_full_terminal() {
        let out = tirage(&["encode"], &recipe());
        assert_eq!(out.status.code(), Some(2));
        assert!(
            text(&out.stderr).contains("no output path given, pass -o FILE, or -o - for stdout"),
            "{}",
            text(&out.stderr)
        );
        let out = on_tty("tirage encode -o out.mp4 --no-input");
        assert_eq!(out.status.code(), Some(2));
        assert!(
            text(&out.stdout).contains("no Recipe on stdin"),
            "{}",
            text(&out.stdout)
        );
    }

    #[test]
    fn roll_encodes_a_loop_its_reproduce_line_remakes() {
        let seed = loop_seed();
        let recipe = tirage::derive(seed, tirage::ToolPin::Any);
        let name = format!("{}-{seed}.mp4", recipe.tool().slug());
        let dir = scratch("roll-loop");
        let out = roll_in(&dir, &[], seed);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert!(
            text(&out.stderr).contains(&format!(
                "→ tirage derive --seed {seed} | tirage encode -o {name}\n"
            )),
            "{}",
            text(&out.stderr)
        );
        assert_eq!(files_in(&dir), std::slice::from_ref(&name));
        let rolled = std::fs::read(dir.join(&name)).unwrap();
        assert_eq!(
            rolled,
            tirage_encode::encode(&recipe, tirage_encode::WIDTH, tirage_encode::HEIGHT).unwrap()
        );
        let again = scratch("roll-loop-again");
        reproduce(&out.stderr, &again);
        assert_eq!(std::fs::read(again.join(&name)).unwrap(), rolled);
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::remove_dir_all(again).unwrap();
    }

    #[test]
    fn encode_shows_in_help_and_completions() {
        let help = text(&tirage(&["--help"], b"").stdout);
        assert!(
            help.contains("encode") && help.contains("[alias: e]"),
            "{help}"
        );
        let script = text(&tirage(&["completions", "bash"], b"").stdout);
        assert!(script.contains("encode"), "{script}");
        let out = tirage(&["e", "-o", "-"], &recipe());
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert_eq!(out.stdout, sonar_mp4());
    }
}

#[cfg(not(feature = "encode"))]
#[test]
fn encode_is_absent_without_the_feature() {
    let out = tirage(&["encode", "-o", "out.mp4"], &recipe());
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("unrecognized subcommand 'encode'"),
        "{}",
        text(&out.stderr)
    );
    let help = text(&tirage(&["--help"], b"").stdout);
    assert!(!help.contains("encode"), "{help}");
}

#[test]
fn render_refuses_svg_for_a_tool_without_it_whether_flagged_or_inferred() {
    let husk = tirage(&["derive", "--seed", "1", "--tool", "husk"], b"").stdout;
    let dir = scratch("svg-refused");
    let svg = dir.join("out.svg");
    for args in [
        vec!["render", "--size", "9x16", "--format", "svg", "-o", "-"],
        vec!["render", "--size", "9x16", "-o", svg.to_str().unwrap()],
    ] {
        let out = tirage(&args, &husk);
        assert_eq!(
            (out.status.code(), text(&out.stderr).as_str()),
            (
                Some(1),
                "error: husk: renders PNG only, not SVG\nhint: run 'tirage tools' to see which Tools render SVG, or write a .png\n"
            ),
            "{args:?}"
        );
        assert!(out.stdout.is_empty());
    }
    assert!(!svg.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn render_format_png_wins_over_an_svg_extension() {
    let dir = scratch("format-wins");
    let output = dir.join("out.svg");
    let out = tirage(
        &[
            "render",
            "--size",
            "9x16",
            "--format",
            "png",
            "-o",
            output.to_str().unwrap(),
        ],
        &recipe(),
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(std::fs::read(&output).unwrap().starts_with(b"\x89PNG"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn render_refuses_an_svg_of_any_frame_even_0() {
    let out = tirage(
        &[
            "render", "--size", "9x16", "--frame", "0", "--format", "svg", "-o", "-",
        ],
        &recipe(),
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out.stderr).contains("an SVG is the Still, drop --frame or pass --format png"),
        "{}",
        text(&out.stderr)
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn render_writes_an_svg_from_its_extension_or_to_stdout_with_format_svg() {
    let dir = scratch("svg");
    let output = dir.join("still.svg");
    let file = tirage(
        &["render", "--size", "9x16", "-o", output.to_str().unwrap()],
        &recipe(),
    );
    assert_eq!(file.status.code(), Some(0), "{}", text(&file.stderr));
    let svg = std::fs::read_to_string(&output).unwrap();
    assert!(
        svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="9" height="16" viewBox="0 0 9 16">"#
        ),
        "{svg}"
    );
    let piped = tirage(
        &["render", "--size", "9x16", "--format", "svg", "-o", "-"],
        &recipe(),
    );
    assert_eq!(piped.status.code(), Some(0), "{}", text(&piped.stderr));
    assert_eq!(text(&piped.stdout), svg);
    std::fs::remove_dir_all(dir).unwrap();
}
