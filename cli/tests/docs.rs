use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tirage::Tool;

const BIN: &str = env!("CARGO_BIN_EXE_tirage");
const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const SHARED: [&str; 11] = [
    "ditherTog",
    "dthKinds",
    "dthSize",
    "dthLevels",
    "dthAmount",
    "grainTog",
    "grnBlends",
    "grnAmount",
    "grnSize",
    "grnSpecks",
    "grnVignette",
];

fn is_shared(line: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|id| SHARED.contains(&id))
}

fn tools_listing() -> String {
    String::from_utf8(Command::new(BIN).arg("tools").output().unwrap().stdout).unwrap()
}

fn tools_section(tools: &str, tool: Tool) -> impl Iterator<Item = &str> {
    let header = format!("{}  ", tool.slug());
    tools
        .split_inclusive('\n')
        .skip_while(move |line| !line.starts_with(&header))
        .enumerate()
        .take_while(|(i, line)| *i == 0 || line.starts_with("  "))
        .map(|(_, line)| line)
}

fn code_block<'a>(page: &'a str, prefix: &str) -> Option<&'a str> {
    page.split("```")
        .skip(1)
        .step_by(2)
        .map(|block| block.trim_start_matches('\n'))
        .find(|block| block.starts_with(prefix))
}

fn read_doc(page: &str) -> String {
    let path = Path::new(ROOT).join(format!("../docs/src/content/docs/{page}.md"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_page(tool: Tool) -> String {
    read_doc(&format!("tools/{}", tool.slug()))
}

fn scratch(tool: Tool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tirage-docs-{}-{}",
        tool.slug(),
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn each_still_re_renders_byte_for_byte_from_the_command_on_its_page() {
    for &tool in Tool::ALL {
        let page = read_page(tool);
        let command = page
            .lines()
            .find(|line| line.starts_with("tirage derive ") && line.contains("| tirage render "))
            .unwrap_or_else(|| panic!("{}: no render command on the page", tool.slug()));
        let dir = scratch(tool);
        let status = Command::new("sh")
            .args(["-c", &command.replace("tirage ", &format!("'{BIN}' "))])
            .current_dir(&dir)
            .status()
            .unwrap();
        assert!(status.success(), "{}: {command}", tool.slug());
        let rendered = fs::read(dir.join(format!("{}.png", tool.slug()))).unwrap_or_else(|e| {
            panic!("{}: the command did not write <slug>.png: {e}", tool.slug())
        });
        let committed =
            fs::read(Path::new(ROOT).join(format!("../docs/src/assets/tools/{}.png", tool.slug())))
                .unwrap_or_else(|e| panic!("{}: no committed Still: {e}", tool.slug()));
        assert!(
            rendered == committed,
            "{}: the committed Still is not what its command renders",
            tool.slug()
        );
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn each_page_lists_its_own_parameters_as_tirage_tools_prints_them() {
    let tools = tools_listing();
    for &tool in Tool::ALL {
        let page = read_page(tool);
        let block = code_block(&page, &format!("{}  ", tool.slug()))
            .unwrap_or_else(|| panic!("{}: no 'tirage tools' block on the page", tool.slug()));
        let own = tools_section(&tools, tool)
            .filter(|line| !is_shared(line))
            .collect::<String>();
        assert_eq!(
            block,
            own,
            "{}: the Parameters block differs from 'tirage tools' without the shared rows",
            tool.slug()
        );
    }
}

#[test]
fn the_dither_and_grain_page_lists_the_shared_rows_every_tool_prints() {
    let page = read_doc("concepts/dither-and-grain");
    let block = code_block(&page, "ditherTog").expect("no shared Parameters block on the page");
    let tools = tools_listing();
    for &tool in Tool::ALL {
        let shared = tools_section(&tools, tool)
            .filter(|line| is_shared(line))
            .map(|line| line.trim_start())
            .collect::<String>();
        assert_eq!(block, shared, "{}: the shared rows differ", tool.slug());
    }
}

#[test]
fn each_page_plays_its_loop_encoded_from_its_own_seed() {
    for &tool in Tool::ALL {
        let slug = tool.slug();
        let page = read_page(tool);
        let render = page
            .lines()
            .find(|line| line.starts_with("tirage derive ") && line.contains("| tirage render "))
            .unwrap_or_else(|| panic!("{slug}: no render command on the page"));
        let derive = render.split(" | ").next().unwrap();
        let encode = format!("{derive} | tirage encode --size 360x640 -o {slug}.mp4");
        assert!(
            page.lines().any(|line| line == encode),
            "{slug}: no '{encode}' line on the page"
        );
        let video = format!(
            "<video src=\"/tools/{slug}.mp4\" width=\"360\" height=\"640\" autoplay loop muted playsinline></video>"
        );
        assert!(page.contains(&video), "{slug}: no '{video}' on the page");
        let path = Path::new(ROOT).join(format!("../docs/public/tools/{slug}.mp4"));
        assert!(
            path.is_file(),
            "{slug}: no committed Loop at {}",
            path.display()
        );
    }
}
