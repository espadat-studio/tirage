use crate::surface::Surface;
use crate::{Error, Image, Recipe};

pub const MAX_EDGE: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    t: u32,
}

impl Frame {
    pub fn new(recipe: &Recipe, width: u32, height: u32, t: u32) -> Result<Self, Error> {
        let edges = 1..=MAX_EDGE;
        if !edges.contains(&width) || !edges.contains(&height) {
            return Err(Error::FrameSize { width, height });
        }
        let frames = recipe.tool().frames();
        if t >= frames {
            return Err(Error::FrameTime { t, frames });
        }
        Ok(Self { width, height, t })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn t(&self) -> u32 {
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
