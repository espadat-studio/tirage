mod aura;
mod chassis;
mod derive;
mod error;
mod husk;
mod image;
mod kiosk;
mod palette;
mod param;
mod recipe;
mod render;
mod sonar;
mod surface;
mod taste;
mod text;
mod unique_keys;
mod vein;

pub use aura::{AuraParams, AuraStyle};
pub use chassis::{Blend, Dither, DitherKind, Grain};
pub use derive::{ToolPin, derive};
pub use error::Error;
pub use husk::{Bite, HuskParams};
pub use image::Image;
pub use kiosk::{CharacterSet, KioskParams};
pub use palette::Palette;
pub use recipe::{Parameter, ParameterKind, Params, Recipe, Tool};
pub use render::{Frame, MAX_EDGE, render};
pub use sonar::SonarParams;
pub use taste::Taste;
pub use vein::{Flow, VeinParams};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DERIVATION_MAJOR: u32 = 0;
