mod chassis;
mod derive;
mod error;
mod image;
mod palette;
mod param;
mod recipe;
mod registry;
mod render;
mod surface;
mod taste;
mod text;
mod unique_keys;

registry::tools! {
    Sonar: sonar::{SonarParams},
    Husk: husk::{HuskParams, Bite},
    Vein: vein::{VeinParams, Flow},
    Aura: aura::{AuraParams, AuraStyle},
    Kiosk: kiosk::{KioskParams, CharacterSet},
    Frond: frond::{FrondParams, Plant},
    Benday: benday::{BendayParams},
}

pub use chassis::{Blend, Dither, DitherKind, Grain};
pub use derive::{ToolPin, derive};
pub use error::Error;
pub use image::Image;
pub use palette::Palette;
pub use recipe::{Parameter, ParameterKind, Recipe};
pub use render::{Frame, MAX_EDGE, render};
pub use taste::Taste;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DERIVATION_MAJOR: u32 = 0;
