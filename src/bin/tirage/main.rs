use std::error::Error as _;
use std::fs;
use std::hash::{BuildHasher, RandomState};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::styling::{AnsiColor, Styles};
use clap::builder::{PossibleValue, TypedValueParser};
use clap::error::ErrorKind;
use clap::{ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::Shell;
use serde::Serialize;

use prompt::{Choice, Prompt};

mod hint;
mod prompt;

use tirage::{
    DERIVATION_MAJOR, Frame, MAX_EDGE, Palette, Parameter, ParameterKind, Recipe, Taste, Tool,
    ToolPin, VERSION, derive, render,
};

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

Commands prompt on a terminal for values you leave out.
Run 'tirage --help' for more.";

const LINKS: &str = "Docs: https://github.com/espadat-studio/tirage#readme
Issues: https://github.com/espadat-studio/tirage/issues";

#[derive(Parser)]
#[command(
    name = "tirage",
    about = "Seed to PNG: derive a Recipe, then render it",
    before_help = EXAMPLES,
    after_help = LINKS,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 'h', action = ArgAction::Help, global = true, help = "Print short help")]
    help: Option<bool>,
    #[arg(long = "help", action = ArgAction::HelpLong, global = true, help = "Print long help")]
    help_long: Option<bool>,
    #[arg(long, action = ArgAction::Version, help = "Print version and derivation major")]
    version: Option<bool>,
    #[arg(
        long,
        global = true,
        help = "Turn off colour, also NO_COLOR or TERM=dumb"
    )]
    no_color: bool,
    #[arg(long, global = true, help = "Never prompt, also TIRAGE_NO_INPUT=1")]
    no_input: bool,
}

#[derive(Subcommand)]
enum Command {
    #[command(
        visible_alias = "d",
        about = "Derive a Recipe from a Seed and print it as JSON",
        before_help = "Examples:\n  tirage derive --seed 42 > recipe.json\n  tirage derive --seed 42 --tool sonar --palette '#000000,#ffffff'\n  tirage derive --seed 42 --taste taste.json"
    )]
    Derive {
        #[arg(
            long,
            value_name = "N",
            help = "Seed to derive the Recipe from, prompted on a terminal",
            long_help = "Seed to derive the Recipe from, an integer in 0..=18446744073709551615. Prompted on a terminal with a random Seed as default"
        )]
        seed: Option<u64>,
        #[arg(long, value_name = "SLUG", value_parser = ToolParser, hide_possible_values = true, help = "Tool to pin, see 'tirage tools'. Dealt from the Seed when left out")]
        tool: Option<Tool>,
        #[arg(
            long,
            value_name = "FILE",
            conflicts_with = "tool",
            help = "Taste bounds JSON to deal Parameters from, pins its Tool"
        )]
        taste: Option<PathBuf>,
        #[arg(long, value_name = "INKS", value_parser = parse_palette, help = "Palette to pin, as comma-separated #rrggbb inks")]
        palette: Option<Palette>,
    },
    #[command(
        visible_alias = "r",
        about = "Render a Recipe to a PNG",
        before_help = "Examples:\n  tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png\n  tirage render recipe.json --size 540x960 -o - > still.png"
    )]
    Render {
        #[arg(
            value_name = "FILE",
            help = "Recipe JSON, '-' for stdin, derived by prompts on a terminal",
            long_help = "Recipe JSON, '-' for stdin, which is the default. On a terminal with nothing piped in, prompts for a Seed and a Tool and derives the Recipe"
        )]
        input: Option<PathBuf>,
        #[arg(
            long,
            value_name = "W>x<H",
            value_parser = parse_size,
            help = "Frame size in pixels, prompted on a terminal",
            long_help = "Frame size in pixels as <W>x<H>, each edge in 1..=8192. Prompted on a terminal with presets: 1080x1920 story, 1080x1350 portrait, 1080x1080 square, 1920x1080 landscape"
        )]
        size: Option<(u32, u32)>,
        #[arg(
            long,
            value_name = "T",
            default_value_t = 0,
            help = "Frame of a Loop, from 0"
        )]
        frame: u32,
        #[arg(
            short,
            value_name = "FILE",
            help = "PNG to write, '-' for stdout, prompted on a terminal",
            long_help = "PNG to write, '-' for stdout. Prompted on a terminal with <tool>-<seed>.png or out.png as default, asking before it overwrites a file"
        )]
        output: Option<PathBuf>,
    },
    #[command(
        visible_aliases = ["t", "ls"],
        about = "List Tools with their frame counts and Parameters",
        before_help = "Examples:\n  tirage tools\n  tirage tools --json"
    )]
    Tools {
        #[arg(long, help = "Print JSON instead of text")]
        json: bool,
    },
    #[command(
        about = "Print a shell completion script",
        before_help = "Examples:\n  tirage completions bash > ~/.local/share/bash-completion/completions/tirage\n  tirage completions fish > ~/.config/fish/completions/tirage.fish"
    )]
    Completions {
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
}

#[derive(Clone)]
struct ToolParser;

impl TypedValueParser for ToolParser {
    type Value = Tool;

    fn parse_ref(
        &self,
        command: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Tool, clap::Error> {
        let from_slug: fn(&str) -> Result<Tool, tirage::Error> = Tool::from_slug;
        from_slug.parse_ref(command, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            Tool::ALL.iter().map(|tool| PossibleValue::new(tool.slug())),
        ))
    }
}

#[derive(Serialize)]
struct ToolListing {
    slug: &'static str,
    frames: u32,
    params: Vec<Parameter>,
}

enum Failure {
    Usage(&'static str, String, Option<String>),
    Runtime(String, Option<String>),
    Aborted(String),
}

const SEED: Choice = Choice {
    noun: "Seed",
    prompt: "Seed",
    resolved_by: "--seed <N>, like --seed 42",
};

const TOOL: Choice = Choice {
    noun: "Tool",
    prompt: "Tool",
    resolved_by: "--tool <SLUG>",
};

const SIZE: Choice = Choice {
    noun: "frame size",
    prompt: "Frame size",
    resolved_by: "--size <W>x<H>, like 1080x1920",
};

const CUSTOM_SIZE: Choice = Choice {
    noun: "frame size",
    prompt: "Frame size as <W>x<H>",
    resolved_by: "--size <W>x<H>, like 1080x1920",
};

const OUTPUT: Choice = Choice {
    noun: "output path",
    prompt: "PNG to write",
    resolved_by: "-o FILE, or -o - for stdout",
};

const SIZES: [(Option<(u32, u32)>, &str); 5] = [
    (Some((1080, 1920)), "story"),
    (Some((1080, 1350)), "portrait 4:5"),
    (Some((1080, 1080)), "square"),
    (Some((1920, 1080)), "landscape"),
    (None, "custom…"),
];

impl Failure {
    fn usage(subcommand: &'static str, error: &tirage::Error) -> Self {
        Self::Usage(subcommand, error.to_string(), hint::for_error(error))
    }

    fn runtime(error: &tirage::Error) -> Self {
        Self::Runtime(error.to_string(), hint::for_error(error))
    }

    fn unreadable(path: &Path, error: &io::Error) -> Self {
        let cause = match error.kind() {
            io::ErrorKind::NotFound => "no such file".to_owned(),
            io::ErrorKind::PermissionDenied => "permission denied".to_owned(),
            _ => error.to_string(),
        };
        Self::Runtime(format!("cannot read {}: {cause}", path.display()), None)
    }
}

fn main() -> ExitCode {
    if std::env::args_os().len() == 1 {
        eprintln!("{CONCISE}");
        return ExitCode::from(2);
    }
    let matches = command().try_get_matches().unwrap_or_else(|error| {
        let Some(hint) = error
            .source()
            .and_then(|source| source.downcast_ref::<tirage::Error>())
            .and_then(hint::for_error)
        else {
            error.exit()
        };
        let _ = error.print();
        print_hint(&hint);
        std::process::exit(2)
    });
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    match run(cli.command, cli.no_input) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(subcommand, message, hint)) => {
            let mut command = command();
            command.build();
            let _ = command
                .find_subcommand_mut(subcommand)
                .expect("usage names a subcommand")
                .error(ErrorKind::InvalidValue, message)
                .print();
            hint.inspect(|hint| print_hint(hint));
            ExitCode::from(2)
        }
        Err(Failure::Runtime(message, hint)) => {
            eprintln!("{} {message}", paint("1;31", "error:"));
            hint.inspect(|hint| print_hint(hint));
            ExitCode::FAILURE
        }
        Err(Failure::Aborted(message)) => {
            eprintln!("{} {message}", paint("1;31", "error:"));
            ExitCode::from(130)
        }
    }
}

fn print_hint(hint: &str) {
    eprintln!("{}", paint("2", &format!("hint: {hint}")));
}

fn colour_allowed() -> bool {
    !std::env::args_os().any(|arg| arg == "--no-color")
        && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
        && std::env::var_os("TERM").is_none_or(|term| term != "dumb")
}

fn stderr_colour() -> bool {
    colour_allowed() && io::stderr().is_terminal()
}

fn paint(style: &str, text: &str) -> String {
    if stderr_colour() {
        format!("\x1b[{style}m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn run(command: Command, no_input: bool) -> Result<(), Failure> {
    match command {
        Command::Derive {
            seed,
            tool,
            taste,
            palette,
        } => {
            let prompt = Prompt::new("derive", no_input, stderr_colour());
            let seed = match seed {
                Some(seed) => seed,
                None => {
                    let seed = prompt.ask(&SEED, Some(random_seed()), |_| Ok(()))?;
                    echo(&format!("{} --seed {seed}", rerun()));
                    seed
                }
            };
            let pin = match (tool, taste) {
                (Some(tool), _) => ToolPin::Tool(tool),
                (_, Some(path)) => {
                    let json =
                        fs::read_to_string(&path).map_err(|e| Failure::unreadable(&path, &e))?;
                    ToolPin::Taste(
                        Taste::from_json(&json).map_err(|e| Failure::usage("derive", &e))?,
                    )
                }
                (None, None) => ToolPin::Any,
            };
            let mut recipe = derive(seed, pin);
            if let Some(palette) = palette {
                recipe
                    .set_palette(palette)
                    .map_err(|e| Failure::usage("derive", &e))?;
            }
            emit(format!("{}\n", recipe.to_json()).as_bytes())
        }
        Command::Render {
            input,
            size,
            frame,
            output,
        } => {
            let prompt = Prompt::new("render", no_input, stderr_colour());
            let stdio = Path::new("-");
            if output.as_deref() == Some(stdio) && io::stdout().is_terminal() {
                return Err(refuse_terminal());
            }
            if size.is_none() {
                prompt.check(&SIZE)?;
            }
            if output.is_none() {
                prompt.check(&OUTPUT)?;
            }
            let mut pipeline = String::new();
            let mut flags = String::new();
            let (recipe, seed) = match input {
                Some(input) if input != stdio => {
                    let json =
                        fs::read_to_string(&input).map_err(|e| Failure::unreadable(&input, &e))?;
                    (
                        Recipe::from_json(&json).map_err(|e| Failure::runtime(&e))?,
                        None,
                    )
                }
                _ if !io::stdin().is_terminal() => {
                    let json = io::read_to_string(io::stdin())
                        .map_err(|e| Failure::unreadable(stdio, &e))?;
                    (
                        Recipe::from_json(&json).map_err(|e| Failure::runtime(&e))?,
                        None,
                    )
                }
                _ if prompt.is_interactive() => {
                    let seed = prompt.ask(&SEED, Some(random_seed()), |_| Ok(()))?;
                    let mut tools = vec![None];
                    tools.extend(Tool::ALL.iter().copied().map(Some));
                    let tool = prompt.select(
                        &tools,
                        |tool| tool.map_or("deal from Seed".to_owned(), frames_label),
                        &TOOL,
                    )?;
                    pipeline = format!("tirage derive --seed {seed}");
                    if let Some(tool) = tool {
                        pipeline += &format!(" --tool {}", tool.slug());
                    }
                    pipeline += " | ";
                    (
                        derive(seed, tool.map_or(ToolPin::Any, ToolPin::Tool)),
                        Some(seed),
                    )
                }
                _ => {
                    return Err(usage(
                        "render",
                        "no Recipe on stdin, pass a FILE or pipe 'tirage derive' into it",
                    ));
                }
            };
            let (width, height) = match size {
                Some(size) => size,
                None => {
                    let size = match prompt.select(
                        &SIZES,
                        |(size, name)| match size {
                            Some((width, height)) => format!("{width}x{height}  {name}"),
                            None => (*name).to_owned(),
                        },
                        &SIZE,
                    )? {
                        (Some(size), _) => size,
                        (None, _) => {
                            let size: String =
                                prompt.ask(&CUSTOM_SIZE, None, |size: &String| {
                                    checked_size(size).map(|_| ())
                                })?;
                            checked_size(&size).expect("the prompt checked the size")
                        }
                    };
                    flags += &format!(" --size {}x{}", size.0, size.1);
                    size
                }
            };
            let output = match output {
                Some(output) => output,
                None => {
                    let default = match seed {
                        Some(seed) => format!("{}-{seed}.png", recipe.tool().slug()),
                        None => "out.png".to_owned(),
                    };
                    let output = loop {
                        let path = PathBuf::from(prompt.ask(
                            &OUTPUT,
                            Some(default.clone()),
                            |_| Ok(()),
                        )?);
                        if !path.exists()
                            || prompt.confirm(
                                &format!("{} exists, overwrite it?", path.display()),
                                &OUTPUT,
                            )?
                        {
                            break path;
                        }
                    };
                    flags += &format!(" -o {}", quote(&output.to_string_lossy()));
                    output
                }
            };
            if !pipeline.is_empty() || !flags.is_empty() {
                echo(&format!("{pipeline}{}{flags}", rerun()));
            }
            if output == stdio && io::stdout().is_terminal() {
                return Err(refuse_terminal());
            }
            let frame =
                Frame::new(&recipe, width, height, frame).map_err(|e| Failure::runtime(&e))?;
            let png = render(&recipe, &frame).to_png();
            if output == stdio {
                emit(&png)
            } else {
                fs::write(&output, png).map_err(|e| {
                    Failure::Runtime(format!("cannot write {}: {e}", output.display()), None)
                })
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
            let width = Tool::ALL
                .iter()
                .flat_map(|tool| tool.parameters())
                .map(|p| p.id.len())
                .max()
                .expect("a Tool has Parameters");
            let mut text = String::new();
            for tool in Tool::ALL {
                text += &format!("{}\n", frames_label(*tool));
                for p in tool.parameters() {
                    let values = match p.kind {
                        ParameterKind::Range { min, max, step } => {
                            format!("{:<9} step {step}", format!("{min}..={max}"))
                        }
                        ParameterKind::Toggle => "on/off".to_owned(),
                        ParameterKind::Choice { choices } => choices.join(", "),
                    };
                    text += &format!("  {:<width$} {values}\n", p.id);
                }
            }
            emit(text.as_bytes())
        }
        Command::Completions { shell } => {
            let mut script = Vec::new();
            clap_complete::generate(shell, &mut self::command(), "tirage", &mut script);
            emit(&script)
        }
    }
}

fn command() -> clap::Command {
    let color = if colour_allowed() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    Cli::command()
        .color(color)
        .version(format!("{VERSION} (derivation major {DERIVATION_MAJOR})"))
        .styles(Styles::plain().error(AnsiColor::Red.on_default().bold()))
}

fn usage(subcommand: &'static str, message: &str) -> Failure {
    Failure::Usage(subcommand, message.to_owned(), None)
}

fn refuse_terminal() -> Failure {
    usage(
        "render",
        "refusing to write a PNG to a terminal, pass -o FILE or pipe stdout",
    )
}

fn frames_label(tool: Tool) -> String {
    let frames = tool.frames();
    let unit = if frames == 1 { "frame" } else { "frames" };
    format!("{}  {frames} {unit}", tool.slug())
}

fn random_seed() -> u64 {
    RandomState::new().hash_one(std::time::SystemTime::now())
}

fn rerun() -> String {
    std::iter::once("tirage".to_owned())
        .chain(std::env::args().skip(1).map(|arg| quote(&arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote(arg: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c);
    if !arg.is_empty() && arg.chars().all(plain) {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

fn echo(command: &str) {
    eprintln!("{}", paint("2", &format!("→ {command}")));
}

fn emit(bytes: &[u8]) -> Result<(), Failure> {
    match io::stdout().lock().write_all(bytes) {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
            Err(Failure::Runtime(format!("cannot write stdout: {e}"), None))
        }
        _ => Ok(()),
    }
}

fn parse_size(size: &str) -> Result<(u32, u32), String> {
    size.split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| "expected <W>x<H> in pixels, like 1080x1920".to_owned())
}

fn checked_size(size: &str) -> Result<(u32, u32), String> {
    let (width, height) = parse_size(size)?;
    let edges = 1..=MAX_EDGE;
    if edges.contains(&width) && edges.contains(&height) {
        Ok((width, height))
    } else {
        Err(format!("each edge must be in 1..={MAX_EDGE}"))
    }
}

fn parse_palette(inks: &str) -> Result<Palette, tirage::Error> {
    Palette::from_hex(&inks.split(',').collect::<Vec<_>>())
}
