use tiny_skia::{Color, ColorU8, Paint, Pixmap, Rect, Transform};

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

    pub(crate) fn edit_rgba(&mut self, edit: impl FnOnce(&mut [u8], u32, u32)) {
        let mut rgba = self.rgba();
        edit(&mut rgba, self.width(), self.height());
        for (pixel, &[r, g, b, a]) in self.0.pixels_mut().iter_mut().zip(rgba.as_chunks::<4>().0) {
            *pixel = ColorU8::from_rgba(r, g, b, a).premultiply();
        }
    }

    fn rgba(&self) -> Vec<u8> {
        self.0
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let c = pixel.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect()
    }

    pub(crate) fn into_image(self) -> Image {
        Image {
            width: self.width(),
            height: self.height(),
            rgba: self.rgba(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Surface;

    #[test]
    fn edit_rgba_hands_out_and_takes_back_straight_alpha() {
        let mut surface = Surface::new(1, 1);
        surface.edit_rgba(|rgba, _, _| rgba.copy_from_slice(&[200, 100, 50, 128]));
        let rgba = surface.into_image().rgba().to_vec();
        assert_eq!(rgba[3], 128);
        for (ours, straight) in rgba[..3].iter().zip([200, 100, 50]) {
            assert!(ours.abs_diff(straight) <= 1, "{rgba:?}");
        }
    }
}
