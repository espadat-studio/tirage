use std::f64::consts::PI;

use tiny_skia::{
    Color, ColorU8, FillRule, FilterQuality, GradientStop, IntSize, LineJoin, Paint, Path,
    PathBuilder, Pixmap, PixmapPaint, Point, RadialGradient, Rect, SpreadMode, Stroke, Transform,
};

use crate::Image;

pub(crate) struct Surface(Pixmap);

#[derive(Clone, Copy)]
pub(crate) enum Smoothing {
    Bicubic,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "fete is the first Tool to draw it")
    )]
    Nearest,
}

#[derive(Default)]
pub(crate) struct Path2D(PathBuilder);

impl Path2D {
    pub(crate) fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to(x as f32, y as f32);
    }

    pub(crate) fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(x as f32, y as f32);
    }

    pub(crate) fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) {
        let start = self.0.last_point().expect("arcTo follows a point");
        let (ux, uy) = (f64::from(start.x) - x1, f64::from(start.y) - y1);
        let (vx, vy) = (x2 - x1, y2 - y1);
        let (lu, lv) = (ux.hypot(uy), vx.hypot(vy));
        if radius == 0.0 || lu == 0.0 || lv == 0.0 {
            self.line_to(x1, y1);
            return;
        }
        let (ux, uy, vx, vy) = (ux / lu, uy / lu, vx / lv, vy / lv);
        if (ux * vy - uy * vx).abs() <= 1.0 / 4096.0 {
            self.line_to(x1, y1);
            return;
        }
        let between = (ux * vx + uy * vy).clamp(-1.0, 1.0).acos();
        let reach = radius / (between / 2.0).tan();
        let handle = radius * 4.0 / 3.0 * ((PI - between) / 4.0).tan();
        let (sx, sy) = (x1 + ux * reach, y1 + uy * reach);
        let (ex, ey) = (x1 + vx * reach, y1 + vy * reach);
        self.line_to(sx, sy);
        self.0.cubic_to(
            (sx - ux * handle) as f32,
            (sy - uy * handle) as f32,
            (ex - vx * handle) as f32,
            (ey - vy * handle) as f32,
            ex as f32,
            ey as f32,
        );
    }

    pub(crate) fn push_circle(&mut self, x: f64, y: f64, radius: f64) {
        self.0.push_circle(x as f32, y as f32, radius as f32);
    }

    pub(crate) fn into_path(self) -> Path {
        self.0.finish().expect("a filled path is not empty")
    }

    pub(crate) fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) {
        self.0.quad_to(cx as f32, cy as f32, x as f32, y as f32);
    }

    pub(crate) fn close(&mut self) {
        self.0.close();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn solid([r, g, b]: [u8; 3]) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint
}

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
        let mut paint = solid([r, g, b]);
        paint.anti_alias = false;
        self.0.fill_rect(rect, &paint, Transform::identity(), None);
    }

    pub(crate) fn fill_even_odd(&mut self, path: &Path2D, ink: [u8; 3]) {
        let path = path
            .0
            .clone()
            .finish()
            .expect("a filled path has a closed loop");
        self.0.fill_path(
            &path,
            &solid(ink),
            FillRule::EvenOdd,
            Transform::identity(),
            None,
        );
    }

    pub(crate) fn stroke(&mut self, path: &Path2D, ink: [u8; 3], width: f64) {
        let stroke = Stroke {
            width: width as f32,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        let path = path
            .0
            .clone()
            .finish()
            .expect("a stroked path has a closed loop");
        self.0
            .stroke_path(&path, &solid(ink), &stroke, Transform::identity(), None);
    }

    pub(crate) fn fill_box(&mut self, x: f64, y: f64, width: f64, height: f64, ink: [u8; 3]) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            self.0
                .fill_rect(rect, &solid(ink), Transform::identity(), None);
        }
    }

    pub(crate) fn fill_path(&mut self, path: &Path, ink: [u8; 3]) {
        self.0.fill_path(
            path,
            &solid(ink),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    pub(crate) fn fill_radial_fade(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        [r, g, b]: [u8; 3],
        alpha: f32,
    ) {
        let centre = Point::from_xy(x as f32, y as f32);
        let [r, g, b] = [r, g, b].map(|c| f32::from(c) / 255.0);
        let stop = |offset, alpha| {
            GradientStop::new(
                offset,
                Color::from_rgba(r, g, b, alpha).expect("alpha is 0..=1"),
            )
        };
        let shader = RadialGradient::new(
            centre,
            0.0,
            centre,
            radius as f32,
            vec![stop(0.0, alpha), stop(1.0, 0.0)],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .expect("a fade has a positive radius");
        let paint = Paint {
            shader,
            ..Paint::default()
        };
        let side = (radius * 2.0) as f32;
        let rect = Rect::from_xywh((x - radius) as f32, (y - radius) as f32, side, side)
            .expect("a fade has a positive radius");
        self.0.fill_rect(rect, &paint, Transform::identity(), None);
    }

    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, [r, g, b]: [u8; 3]) {
        let width = self.width();
        self.0.pixels_mut()[(y * width + x) as usize] =
            ColorU8::from_rgba(r, g, b, 255).premultiply();
    }

    pub(crate) fn fill_circle(&mut self, x: f64, y: f64, radius: f64, ink: [u8; 3]) {
        let path = PathBuilder::from_circle(x as f32, y as f32, radius as f32)
            .expect("a circle has a positive radius");
        self.0.fill_path(
            &path,
            &solid(ink),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    pub(crate) fn draw_smooth(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        smoothing: Smoothing,
    ) {
        let premultiplied = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[r, g, b, a]| {
                let c = ColorU8::from_rgba(r, g, b, a).premultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        let size = IntSize::from_wh(width, height).expect("buffer edges are at least 1 px");
        let image = Pixmap::from_vec(premultiplied, size).expect("buffer holds width x height");
        let scale = Transform::from_scale(
            self.width() as f32 / width as f32,
            self.height() as f32 / height as f32,
        );
        let paint = PixmapPaint {
            quality: match smoothing {
                Smoothing::Bicubic => FilterQuality::Bicubic,
                Smoothing::Nearest => FilterQuality::Nearest,
            },
            ..PixmapPaint::default()
        };
        self.0
            .draw_pixmap(0, 0, image.as_ref(), &paint, scale, None);
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
    use super::{Path2D, Smoothing, Surface};

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

    #[test]
    fn arc_to_draws_a_near_straight_corner_as_a_line() {
        let mut path = Path2D::default();
        path.move_to(0.0, 0.0);
        path.arc_to(10.0, 0.001, 20.0, 0.0, 5.0);
        let end = path.0.last_point().unwrap();
        assert_eq!((end.x, end.y), (10.0, 0.001));
    }

    #[test]
    fn draw_smooth_nearest_keeps_buffer_pixels_sharp() {
        let mut surface = Surface::new(4, 1);
        surface.draw_smooth(&[255, 0, 0, 255, 0, 0, 255, 255], 2, 1, Smoothing::Nearest);
        let rgba = surface.into_image().rgba().to_vec();
        let red = [255, 0, 0, 255];
        let blue = [0, 0, 255, 255];
        assert_eq!(rgba, [red, red, blue, blue].concat());
    }
}
