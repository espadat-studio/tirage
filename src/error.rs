use std::fmt;

use crate::{DERIVATION_MAJOR, MAX_EDGE, Tool};

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    OutOfRange {
        tool: &'static str,
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
    Major(u64),
    UnknownTool(String),
    Json(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange {
                tool,
                param,
                value,
                min,
                max,
            } => write!(f, "{tool}: {param} {value} is outside {min}..={max}"),
            Self::FrameSize { width, height } => {
                write!(
                    f,
                    "frame {width}x{height} is outside 1..={MAX_EDGE} per edge"
                )
            }
            Self::FrameTime { t, frames } => write!(f, "frame {t} is outside 0..{frames}"),
            Self::Ink(ink) => write!(f, "palette: {ink:?} is not a #rrggbb colour"),
            Self::FewInks(n) => write!(f, "palette: needs at least 2 inks, got {n}"),
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
            Self::Json(message) => write!(f, "Recipe JSON: {message}"),
        }
    }
}

impl std::error::Error for Error {}
