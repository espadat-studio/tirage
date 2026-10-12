use crate::surface::Surface;
use crate::{Error, Image, Recipe};

pub const MAX_EDGE: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    t: Option<u32>,
}

impl Frame {
    pub fn new(recipe: &Recipe, width: u32, height: u32, t: u32) -> Result<Self, Error> {
        let frames = recipe.tool().frames();
        if t >= frames {
            return Err(Error::FrameTime { t, frames });
        }
        Ok(Self {
            t: Some(t),
            ..Self::still(width, height)?
        })
    }

    pub fn still(width: u32, height: u32) -> Result<Self, Error> {
        let edges = 1..=MAX_EDGE;
        if !edges.contains(&width) || !edges.contains(&height) {
            return Err(Error::FrameSize { width, height });
        }
        Ok(Self {
            width,
            height,
            t: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn t(&self) -> Option<u32> {
        self.t
    }
}

pub fn render(recipe: &Recipe, frame: &Frame) -> Image {
    let mut surface = Surface::new(frame.width, frame.height);
    recipe.params().render(
        &mut surface,
        recipe.palette(),
        recipe.tool_seed().get(),
        frame.t,
    );
    surface.into_image()
}

pub fn render_svg(recipe: &Recipe, width: u32, height: u32) -> Result<String, Error> {
    let tool = recipe.tool();
    if !tool.supports_svg() {
        return Err(Error::SvgUnsupported { tool });
    }
    let frame = Frame::still(width, height)?;
    let mut surface = Surface::svg(frame.width, frame.height);
    recipe.params().render(
        &mut surface,
        recipe.palette(),
        recipe.tool_seed().get(),
        frame.t,
    );
    Ok(surface.into_svg())
}

#[cfg(test)]
impl Frame {
    pub(crate) fn after_last(recipe: &Recipe, width: u32, height: u32) -> Self {
        Self {
            t: Some(recipe.tool().frames()),
            ..Self::still(width, height).expect("a test frame size is valid")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Frame, render};
    use crate::{Palette, Params, QuiltStyle, Tool, ToolPin, ZigStyle, derive};

    const CLOSED: &[Tool] = &[
        Tool::Sonar,
        Tool::Husk,
        Tool::Bloom,
        Tool::Weave,
        Tool::Warp,
        Tool::Dahlia,
        Tool::Tokens,
        Tool::Optic,
        Tool::Vee,
        Tool::Riso,
        Tool::Rise,
        Tool::Prism,
        Tool::Filament,
        Tool::Culture,
        Tool::Sprig,
        Tool::Fold,
        Tool::Splice,
        Tool::Crowd,
        Tool::Parcel,
        Tool::Chaff,
    ];

    #[test]
    fn the_frame_after_the_last_is_frame_0_on_continuous_loops() {
        for &tool in CLOSED {
            for seed in 1..=3 {
                let recipe = derive(seed, ToolPin::Tool(tool));
                let first = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
                let after = render(&recipe, &Frame::after_last(&recipe, 270, 480));
                assert!(
                    first.rgba() == after.rgba(),
                    "{} seed {seed}: frame {} is not frame 0",
                    tool.slug(),
                    tool.frames()
                );
            }
        }
    }

    #[test]
    fn whorl_rotates_its_stripe_inks_one_step_at_the_seam() {
        let inks = ["#000000", "#ff0000", "#00ff00", "#0000ff"];
        let rotated = ["#000000", "#00ff00", "#0000ff", "#ff0000"];
        for seed in 1..=3 {
            for (from, to) in [(&inks[..2], &inks[..2]), (&inks[..], &rotated[..])] {
                let mut recipe = derive(seed, ToolPin::Tool(Tool::Whorl));
                recipe
                    .set_palette(Palette::from_hex(from).unwrap())
                    .unwrap();
                let after = render(&recipe, &Frame::after_last(&recipe, 270, 480));
                recipe.set_palette(Palette::from_hex(to).unwrap()).unwrap();
                let first = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
                let off = first
                    .rgba()
                    .iter()
                    .zip(after.rgba())
                    .map(|(a, b)| a.abs_diff(*b));
                assert!(off.max() <= Some(1), "seed {seed}, {} inks", from.len());
            }
        }
    }

    fn zig_seam(seed: u64, style: ZigStyle, width: f64) -> bool {
        let mut recipe = derive(seed, ToolPin::Tool(Tool::Zig));
        let Params::Zig(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_style(style);
        params.set_width(width).unwrap();
        let first = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        let after = render(&recipe, &Frame::after_last(&recipe, 270, 480));
        first.rgba() != after.rgba()
    }

    #[test]
    fn zig_closes_on_every_style_but_wide_stairs() {
        use ZigStyle::*;
        for seed in 1..=3 {
            for (style, width) in [
                (Teeth, 1.0),
                (Chevron, 1.0),
                (Stairs, 0.7),
                (Ricrac, 1.0),
                (Waves, 1.0),
                (Scales, 1.0),
            ] {
                assert!(!zig_seam(seed, style, width), "seed {seed}: {style:?}");
            }
            for width in [1.0, 1.45, 1.8] {
                assert!(zig_seam(seed, Stairs, width), "seed {seed}: width {width}");
            }
        }
    }

    fn quilt_seam(seed: u64, style: QuiltStyle) -> bool {
        let mut recipe = derive(seed, ToolPin::Tool(Tool::Quilt));
        let Params::Quilt(params) = recipe.params_mut() else {
            unreachable!()
        };
        params.set_style(style);
        params.set_chunk(1.0).unwrap();
        let first = render(&recipe, &Frame::new(&recipe, 270, 480, 0).unwrap());
        let after = render(&recipe, &Frame::after_last(&recipe, 270, 480));
        first.rgba() != after.rgba()
    }

    #[test]
    fn quilt_closes_on_every_style_but_bands_tabs_and_cross() {
        use QuiltStyle::*;
        for seed in 1..=3 {
            for style in [
                Plaid, Dither, Steps, Zigzag, Diamond, Basket, Rings, Star, Waves, Gingham, Burst,
            ] {
                assert!(!quilt_seam(seed, style), "seed {seed}: {style:?}");
            }
            for style in [Bands, Tabs, Cross] {
                assert!(quilt_seam(seed, style), "seed {seed}: {style:?}");
            }
        }
    }
}
