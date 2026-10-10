use std::fmt;

use crate::{DERIVATION_MAJOR, MAX_EDGE, Tool};

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    OutOfRange {
        scope: &'static str,
        param: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },
    FrameSize {
        width: u32,
        height: u32,
    },
    FrameTime {
        t: u32,
        frames: u32,
    },
    Ink(String),
    FewInks(usize),
    TooManyInks {
        tool: Tool,
        inks: usize,
        max: usize,
    },
    NoPalette(Tool),
    OffStep {
        scope: &'static str,
        param: &'static str,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
    },
    Reversed {
        scope: &'static str,
        param: &'static str,
        low: f64,
        high: f64,
    },
    Major(u64),
    UnknownTool(String),
    UnknownParameter {
        tool: Tool,
        param: String,
    },
    Json(String),
    TasteJson(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange {
                scope,
                param,
                value,
                min,
                max,
            } => write!(f, "{scope}: {param} {value} is outside {min}..={max}"),
            Self::FrameSize { width, height } => {
                write!(
                    f,
                    "frame {width}x{height} is outside 1..={MAX_EDGE} per edge"
                )
            }
            Self::FrameTime { t, frames } => write!(f, "frame {t} is outside 0..{frames}"),
            Self::Ink(ink) => write!(f, "palette: {ink:?} is not a #rrggbb colour"),
            Self::FewInks(n) => write!(f, "palette: needs at least 2 inks, got {n}"),
            Self::TooManyInks { tool, inks, max } => {
                let slug = tool.slug();
                write!(
                    f,
                    "{slug}: palette has {inks} inks, {slug} draws at most {max}"
                )
            }
            Self::NoPalette(tool) => write!(f, "{}: takes no Palette", tool.slug()),
            Self::Major(found) => write!(
                f,
                "Recipe is tirage major {found}, this build reads major {DERIVATION_MAJOR}"
            ),
            Self::UnknownTool(slug) => {
                let known: Vec<_> = Tool::ALL.iter().map(|tool| tool.slug()).collect();
                write!(
                    f,
                    "unknown tool {slug:?}, expected one of {}",
                    known.join(", ")
                )
            }
            Self::OffStep {
                scope,
                param,
                value,
                min,
                max,
                step,
            } => write!(
                f,
                "{scope}: {param} {value} is not a slider value, {min}..={max} step {step}"
            ),
            Self::Reversed {
                scope,
                param,
                low,
                high,
            } => write!(
                f,
                "{scope}: {param} Taste bounds {low}..={high} have min above max"
            ),
            Self::UnknownParameter { tool, param } => {
                let known: Vec<_> = tool.params().iter().map(|p| p.id).collect();
                write!(
                    f,
                    "{}: unknown Parameter {param:?}, expected one of {}",
                    tool.slug(),
                    known.join(", ")
                )
            }
            Self::Json(message) => write!(f, "Recipe JSON: {message}"),
            Self::TasteJson(message) => write!(f, "Taste JSON: {message}"),
        }
    }
}

impl std::error::Error for Error {}
