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
    Terrain: terrain::{TerrainParams},
    Stitch: stitch::{StitchParams},
    Pith: pith::{PithParams},
    Mosh: mosh::{MoshParams},
    Mist: mist::{MistParams},
    Coral: coral::{CoralParams},
    Whorl: whorl::{WhorlParams, WhorlWarp},
    Sear: sear::{SearParams},
    Culture: culture::{CultureParams, CultureTexture},
    Bloom: bloom::{BloomParams},
    Weave: weave::{WeaveParams},
    Warp: warp::{WarpParams, WarpStyle},
    Zig: zig::{ZigParams, ZigStyle},
    Relief: relief::{ReliefParams},
    Atlas: atlas::{AtlasParams},
    Sprig: sprig::{SprigParams, MotifSet},
    Stipple: stipple::{StippleParams, DotMode, Lattice, DotShape, Symmetry},
    Motley: motley::{MotleyParams},
    Oddgrid: oddgrid::{OddgridParams, Motif},
    Fete: fete::{FeteParams, FeteMotif},
    Fold: fold::{FoldParams, FoldMirror, FoldKind},
    Quilt: quilt::{QuiltParams, QuiltStyle},
    Static: r#static::{StaticParams},
    Splice: splice::{SpliceParams, SpliceCut},
    Dahlia: dahlia::{DahliaParams},
    Hiss: hiss::{HissParams},
    Crowd: crowd::{CrowdParams},
    Cipher: cipher::{CipherParams, CipherField},
    Riso: riso::{RisoParams},
    Rise: rise::{RiseParams, RiseAnchor},
    Carve: carve::{CarveParams},
    Specimen: specimen::{SpecimenParams},
    Pane: pane::{PaneParams},
    Modular: modular::{ModularParams},
    Prism: prism::{PrismParams},
    Parcel: parcel::{ParcelParams, LineBlend},
    Tokens: tokens::{TokensParams},
    Optic: optic::{OpticParams, OpticStyle},
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
