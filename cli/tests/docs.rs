use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tirage::Tool;

const BIN: &str = env!("CARGO_BIN_EXE_tirage");
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read_page(tool: Tool) -> String {
    let path = Path::new(ROOT).join(format!("../docs/src/content/docs/tools/{}.md", tool.slug()));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
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
            .find(|line| line.starts_with("tirage derive "))
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
fn each_page_lists_parameters_as_tirage_tools_prints_them() {
    let tools = String::from_utf8(Command::new(BIN).arg("tools").output().unwrap().stdout).unwrap();
    for &tool in Tool::ALL {
        let page = read_page(tool);
        let header = format!("{}  ", tool.slug());
        let block = page
            .split("```")
            .skip(1)
            .step_by(2)
            .map(|block| block.trim_start_matches('\n'))
            .find(|block| block.starts_with(&header))
            .unwrap_or_else(|| panic!("{}: no 'tirage tools' block on the page", tool.slug()));
        let section = tools
            .split_inclusive('\n')
            .skip_while(|line| !line.starts_with(&header))
            .take_while(|line| line.starts_with(&header) || line.starts_with("  "))
            .collect::<String>();
        assert_eq!(
            block,
            section,
            "{}: the Parameters block differs from 'tirage tools'",
            tool.slug()
        );
    }
}
