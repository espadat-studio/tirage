mod derive;
mod error;
mod image;
mod palette;
mod recipe;
mod render;
mod sonar;
mod surface;

pub use derive::{ToolPin, derive};
pub use error::Error;
pub use image::Image;
pub use palette::Palette;
pub use recipe::{Params, Recipe, Tool};
pub use render::{Frame, MAX_EDGE, render};
pub use sonar::SonarParams;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DERIVATION_MAJOR: u32 = 0;
