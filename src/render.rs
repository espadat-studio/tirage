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
    use crate::{Tool, ToolPin, derive};

    const CLOSED: &[Tool] = &[Tool::Sonar, Tool::Husk];

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
}
