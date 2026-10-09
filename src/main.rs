use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::error::ErrorKind;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};
use serde::Serialize;

use tirage::{DERIVATION_MAJOR, Frame, Parameter, Recipe, Tool, ToolPin, VERSION, derive, render};

const EXAMPLES: &str = "Examples:
  tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
  tirage derive --seed 42 --tool sonar > recipe.json
  tirage render recipe.json --size 540x960 -o - > still.png
  tirage tools";

const CONCISE: &str = "tirage: Seed to PNG, rendered from code

Usage: tirage <COMMAND>

Examples:
  tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
  tirage tools

Run 'tirage --help' for more.";

#[derive(Parser)]
#[command(
    name = "tirage",
    about = "Seed to PNG: derive a Recipe, then render it",
    before_help = EXAMPLES,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long, action = ArgAction::HelpLong, global = true, help = "Print help")]
    help: Option<bool>,
    #[arg(long, action = ArgAction::Version, help = "Print version and derivation major")]
    version: Option<bool>,
}

#[derive(Subcommand)]
enum Command {
    #[command(
        about = "Derive a Recipe from a Seed and print it as JSON",
        before_help = "Examples:\n  tirage derive --seed 42 --tool sonar > recipe.json"
    )]
    Derive {
        #[arg(long, help = "Seed to derive the Recipe from")]
        seed: u64,
        #[arg(long, value_name = "SLUG", value_parser = Tool::from_slug, help = "Tool to pin, see 'tirage tools'")]
        tool: Tool,
    },
    #[command(
        about = "Render a Recipe to a PNG",
        before_help = "Examples:\n  tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png\n  tirage render recipe.json --size 540x960 -o - > still.png"
    )]
    Render {
        #[arg(
            value_name = "FILE",
            default_value = "-",
            help = "Recipe JSON, '-' for stdin"
        )]
        input: PathBuf,
        #[arg(long, value_name = "W>x<H", value_parser = parse_size, help = "Frame size in pixels")]
        size: (u32, u32),
        #[arg(
            long,
            value_name = "T",
            default_value_t = 0,
            help = "Frame of a Loop, from 0"
        )]
        frame: u32,
        #[arg(short, value_name = "FILE", help = "PNG to write, '-' for stdout")]
        output: PathBuf,
    },
    #[command(
        about = "List Tools with their frame counts and Parameter ranges",
        before_help = "Examples:\n  tirage tools\n  tirage tools --json"
    )]
    Tools {
        #[arg(long, help = "Print JSON instead of text")]
        json: bool,
    },
}

#[derive(Serialize)]
struct ToolListing {
    slug: &'static str,
    frames: u32,
    params: Vec<Parameter>,
}

fn main() -> ExitCode {
    if std::env::args_os().len() == 1 {
        eprintln!("{CONCISE}");
        return ExitCode::from(2);
    }
    let matches = command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    match run(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Derive { seed, tool } => {
            emit(format!("{}\n", derive(seed, ToolPin::Tool(tool)).to_json()).as_bytes())
        }
        Command::Render {
            input,
            size: (width, height),
            frame,
            output,
        } => {
            let stdio = Path::new("-");
            if output == stdio && io::stdout().is_terminal() {
                render_usage("refusing to write a PNG to a terminal, pass -o FILE or pipe stdout");
            }
            if input == stdio && io::stdin().is_terminal() {
                render_usage("no Recipe on stdin, pass a FILE or pipe 'tirage derive' into it");
            }
            let json = if input == stdio {
                io::read_to_string(io::stdin()).map_err(|e| format!("cannot read stdin: {e}"))?
            } else {
                fs::read_to_string(&input)
                    .map_err(|e| format!("cannot read {}: {e}", input.display()))?
            };
            let recipe = Recipe::from_json(&json).map_err(|e| e.to_string())?;
            let frame = Frame::new(&recipe, width, height, frame).map_err(|e| e.to_string())?;
            let png = render(&recipe, &frame).to_png();
            if output == stdio {
                emit(&png)
            } else {
                fs::write(&output, png)
                    .map_err(|e| format!("cannot write {}: {e}", output.display()))
            }
        }
        Command::Tools { json } => {
            if json {
                let listings: Vec<_> = Tool::ALL
                    .iter()
                    .map(|tool| ToolListing {
                        slug: tool.slug(),
                        frames: tool.frames(),
                        params: tool.parameters(),
                    })
                    .collect();
                let json = serde_json::to_string(&listings).expect("a listing serializes");
                return emit(format!("{json}\n").as_bytes());
            }
            let mut text = String::new();
            for tool in Tool::ALL {
                let frames = tool.frames();
                let unit = if frames == 1 { "frame" } else { "frames" };
                text += &format!("{}  {frames} {unit}\n", tool.slug());
                for p in tool.parameters() {
                    let range = format!("{}..={}", p.min, p.max);
                    text += &format!("  {:<7} {range:<9} step {}\n", p.id, p.step);
                }
            }
            emit(text.as_bytes())
        }
    }
}

fn command() -> clap::Command {
    Cli::command().version(format!("{VERSION} (derivation major {DERIVATION_MAJOR})"))
}

fn render_usage(message: &str) -> ! {
    let mut command = command();
    command.build();
    command
        .find_subcommand_mut("render")
        .expect("render is a subcommand")
        .error(ErrorKind::InvalidValue, message)
        .exit()
}

fn emit(bytes: &[u8]) -> Result<(), String> {
    match io::stdout().lock().write_all(bytes) {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => Err(format!("cannot write stdout: {e}")),
        _ => Ok(()),
    }
}

fn parse_size(size: &str) -> Result<(u32, u32), String> {
    size.split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| "expected <W>x<H> in pixels, like 1080x1920".to_owned())
}
