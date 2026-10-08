use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

use crate::Image;

pub(crate) struct Surface(Pixmap);

impl Surface {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self(Pixmap::new(width, height).expect("frame edges are validated"))
    }

    pub(crate) fn width(&self) -> u32 {
        self.0.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.0.height()
    }

    pub(crate) fn fill(&mut self, [r, g, b]: [u8; 3]) {
        self.0.fill(Color::from_rgba8(r, g, b, 255));
    }

    pub(crate) fn fill_rect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        [r, g, b]: [u8; 3],
    ) {
        let rect = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32)
            .expect("rect sides are at least 1 px");
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, 255);
        paint.anti_alias = false;
        self.0.fill_rect(rect, &paint, Transform::identity(), None);
    }

    pub(crate) fn into_image(self) -> Image {
        let rgba = self
            .0
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let c = pixel.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        Image {
            width: self.0.width(),
            height: self.0.height(),
            rgba,
        }
    }
}
