use std::f64::consts::{PI, TAU};

use kurbo::{Arc, PathEl, Shape, Vec2};

use tiny_skia::{
    BlendMode, Color, ColorU8, FillRule, FilterQuality, GradientStop, IntSize, LineCap, LineJoin,
    LinearGradient, Mask, Paint, Path, PathBuilder, Pixmap, PixmapPaint, Point, RadialGradient,
    Rect, SpreadMode, Stroke,
};

use crate::Image;

pub(crate) struct Surface(Pixmap, f32, Option<Mask>, BlendMode);

#[derive(Clone, Copy)]
pub(crate) enum Cap {
    Butt,
    Round,
}

#[derive(Clone, Copy)]
pub(crate) enum Join {
    Round,
    Miter,
}

#[derive(Clone, Copy)]
pub(crate) enum Smoothing {
    Bicubic,
    Nearest,
}

#[derive(Clone, Copy)]
pub(crate) struct Transform(tiny_skia::Transform);

impl Transform {
    pub(crate) const IDENTITY: Self = Self(tiny_skia::Transform {
        sx: 1.0,
        kx: 0.0,
        ky: 0.0,
        sy: 1.0,
        tx: 0.0,
        ty: 0.0,
    });

    pub(crate) fn translate(self, x: f64, y: f64) -> Self {
        Self(self.0.pre_translate(x as f32, y as f32))
    }

    pub(crate) fn rotate(self, radians: f64) -> Self {
        Self(self.0.pre_rotate(radians.to_degrees() as f32))
    }

    pub(crate) fn scale(self, x: f64, y: f64) -> Self {
        Self(self.0.pre_scale(x as f32, y as f32))
    }
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

    #[expect(clippy::too_many_arguments, reason = "canvas ellipse() takes seven")]
    pub(crate) fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        start: f64,
        end: f64,
    ) {
        let sweep = end - start;
        let arc = Arc::new(
            (x, y),
            Vec2::new(rx, ry),
            start,
            if sweep >= TAU {
                TAU
            } else {
                sweep.rem_euclid(TAU)
            },
            rotation,
        );
        for element in arc.path_elements(0.01) {
            match element {
                PathEl::MoveTo(p) if self.is_empty() => self.move_to(p.x, p.y),
                PathEl::MoveTo(p) => self.line_to(p.x, p.y),
                PathEl::CurveTo(a, b, p) => self.0.cubic_to(
                    a.x as f32, a.y as f32, b.x as f32, b.y as f32, p.x as f32, p.y as f32,
                ),
                _ => unreachable!("an arc is one move and its cubics"),
            }
        }
    }

    pub(crate) fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            self.0.push_rect(rect);
        }
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

impl Surface {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self(
            Pixmap::new(width, height).expect("frame edges are validated"),
            1.0,
            None,
            BlendMode::SourceOver,
        )
    }

    pub(crate) fn with_alpha(&mut self, alpha: f32, draw: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.1, alpha);
        draw(self);
        self.1 = outer;
    }

    pub(crate) fn with_clip(
        &mut self,
        clip: &Path2D,
        transform: Transform,
        draw: impl FnOnce(&mut Self),
    ) {
        let path = clip.0.clone().finish().expect("a clip has a closed loop");
        let mask = if let Some(outer) = &self.2 {
            let mut mask = outer.clone();
            mask.intersect_path(&path, FillRule::Winding, true, transform.0);
            mask
        } else {
            let mut mask =
                Mask::new(self.width(), self.height()).expect("frame edges are validated");
            mask.fill_path(&path, FillRule::Winding, true, transform.0);
            mask
        };
        let outer = self.2.replace(mask);
        draw(self);
        self.2 = outer;
    }

    pub(crate) fn with_multiply(&mut self, draw: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.3, BlendMode::Multiply);
        draw(self);
        self.3 = outer;
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "strand adds its eroded rods"))]
    pub(crate) fn with_plus(&mut self, draw: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.3, BlendMode::Plus);
        draw(self);
        self.3 = outer;
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "strand blurs its eroded rods"))]
    pub(crate) fn with_blur(&mut self, sigma: f64, draw: impl FnOnce(&mut Self)) {
        let mut layer = Self::new(self.width(), self.height());
        draw(&mut layer);
        let (width, height) = (self.width() as usize, self.height() as usize);
        let mut channels = layer
            .0
            .data()
            .iter()
            .map(|&c| f64::from(c))
            .collect::<Vec<_>>();
        for window in box_windows(sigma) {
            box_pass(&mut channels, (width, height), (4, width * 4), window);
            box_pass(&mut channels, (height, width), (width * 4, 4), window);
        }
        for (byte, value) in layer.0.data_mut().iter_mut().zip(channels) {
            *byte = value.round().clamp(0.0, 255.0) as u8;
        }
        let paint = PixmapPaint {
            opacity: self.1,
            blend_mode: self.3,
            ..PixmapPaint::default()
        };
        self.0.draw_pixmap(
            0,
            0,
            layer.0.as_ref(),
            &paint,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    fn solid(&self, [r, g, b]: [u8; 3]) -> Paint<'static> {
        let [r, g, b] = [r, g, b].map(|c| f32::from(c) / 255.0);
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba(r, g, b, self.1).expect("alpha is 0..=1"));
        paint.blend_mode = self.3;
        paint
    }

    pub(crate) fn width(&self) -> u32 {
        self.0.width()
    }

    pub(crate) fn height(&self) -> u32 {
        self.0.height()
    }

    pub(crate) fn fill(&mut self, ink: [u8; 3]) {
        let (w, h) = (f64::from(self.width()), f64::from(self.height()));
        self.fill_box(0.0, 0.0, w, h, ink);
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
        let mut paint = self.solid([r, g, b]);
        paint.anti_alias = false;
        self.0.fill_rect(
            rect,
            &paint,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "a rect, a gradient line and its stops"
    )]
    pub(crate) fn fill_rect_linear(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        (x0, y0): (f64, f64),
        (x1, y1): (f64, f64),
        stops: &[(f64, [u8; 3])],
    ) {
        let rect = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32)
            .expect("rect sides are at least 1 px");
        let stops = stops
            .iter()
            .map(|&(offset, [r, g, b])| {
                GradientStop::new(offset as f32, Color::from_rgba8(r, g, b, 255))
            })
            .collect();
        let shader = LinearGradient::new(
            Point::from_xy(x0 as f32, y0 as f32),
            Point::from_xy(x1 as f32, y1 as f32),
            stops,
            SpreadMode::Pad,
            tiny_skia::Transform::identity(),
        )
        .expect("a ramp has two stops and a line of positive length");
        let paint = Paint {
            shader,
            anti_alias: false,
            ..Paint::default()
        };
        self.0
            .fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
    }

    pub(crate) fn fill_even_odd(&mut self, path: &Path2D, ink: [u8; 3]) {
        let path = path
            .0
            .clone()
            .finish()
            .expect("a filled path has a closed loop");
        let paint = self.solid(ink);
        self.0.fill_path(
            &path,
            &paint,
            FillRule::EvenOdd,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    pub(crate) fn stroke(&mut self, path: &Path2D, ink: [u8; 3], width: f64, cap: Cap, join: Join) {
        let stroke = Stroke {
            width: width as f32,
            line_cap: match cap {
                Cap::Butt => LineCap::Butt,
                Cap::Round => LineCap::Round,
            },
            line_join: match join {
                Join::Round => LineJoin::Round,
                Join::Miter => LineJoin::Miter,
            },
            miter_limit: 10.0,
            ..Stroke::default()
        };
        let path = path
            .0
            .clone()
            .finish()
            .expect("a stroked path has a closed loop");
        let paint = self.solid(ink);
        self.0.stroke_path(
            &path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    pub(crate) fn fill_box(&mut self, x: f64, y: f64, width: f64, height: f64, ink: [u8; 3]) {
        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) {
            let paint = self.solid(ink);
            self.0.fill_rect(
                rect,
                &paint,
                tiny_skia::Transform::identity(),
                self.2.as_ref(),
            );
        }
    }

    pub(crate) fn fill_path(&mut self, path: &Path, ink: [u8; 3]) {
        let paint = self.solid(ink);
        self.0.fill_path(
            path,
            &paint,
            FillRule::Winding,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    pub(crate) fn fill_transformed(&mut self, path: &Path2D, ink: [u8; 3], transform: Transform) {
        let path = path
            .0
            .clone()
            .finish()
            .expect("a filled path has a closed loop");
        let paint = self.solid(ink);
        self.0.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            transform.0,
            self.2.as_ref(),
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
            tiny_skia::Transform::identity(),
        )
        .expect("a fade has a positive radius");
        let paint = Paint {
            shader,
            ..Paint::default()
        };
        let side = (radius * 2.0) as f32;
        let rect = Rect::from_xywh((x - radius) as f32, (y - radius) as f32, side, side)
            .expect("a fade has a positive radius");
        self.0.fill_rect(
            rect,
            &paint,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, [r, g, b]: [u8; 3]) {
        assert!(self.2.is_none(), "pixel writes ignore a clip");
        let width = self.width();
        self.0.pixels_mut()[(y * width + x) as usize] =
            ColorU8::from_rgba(r, g, b, 255).premultiply();
    }

    pub(crate) fn fill_circle(&mut self, x: f64, y: f64, radius: f64, ink: [u8; 3]) {
        let path = PathBuilder::from_circle(x as f32, y as f32, radius as f32)
            .expect("a circle has a positive radius");
        let paint = self.solid(ink);
        self.0.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            tiny_skia::Transform::identity(),
            self.2.as_ref(),
        );
    }

    pub(crate) fn draw_smooth(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        smoothing: Smoothing,
    ) {
        if let Smoothing::Nearest = smoothing {
            self.draw_nearest(rgba, width, height);
            return;
        }
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
        let scale = tiny_skia::Transform::from_scale(
            self.width() as f32 / width as f32,
            self.height() as f32 / height as f32,
        );
        let paint = PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };
        self.0
            .draw_pixmap(0, 0, image.as_ref(), &paint, scale, self.2.as_ref());
    }

    fn draw_nearest(&mut self, rgba: &[u8], width: u32, height: u32) {
        assert!(self.2.is_none(), "pixel writes ignore a clip");
        let (columns, rows) = (self.width(), self.height());
        let source = |at: u32, edge: u32, scaled: u32| {
            ((f64::from(at) + 0.5) * f64::from(edge) / f64::from(scaled)).ceil() as u32 - 1
        };
        let pixels = self.0.pixels_mut();
        for y in 0..rows {
            let sy = source(y, height, rows);
            for x in 0..columns {
                let sx = source(x, width, columns);
                let i = ((sy * width + sx) * 4) as usize;
                let [r, g, b, a] = [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]];
                pixels[(y * columns + x) as usize] = ColorU8::from_rgba(r, g, b, a).premultiply();
            }
        }
    }

    pub(crate) fn edit_rgba(&mut self, edit: impl FnOnce(&mut [u8], u32, u32)) {
        assert!(self.2.is_none(), "pixel writes ignore a clip");
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

fn box_windows(sigma: f64) -> [(usize, usize); 3] {
    let d = (sigma * 3.0 * TAU.sqrt() / 4.0 + 0.5).floor().max(1.0) as usize;
    let half = d / 2;
    if d % 2 == 1 {
        [(half, half); 3]
    } else {
        [(half, half - 1), (half - 1, half), (half, half)]
    }
}

fn box_pass(
    channels: &mut [f64],
    (length, lines): (usize, usize),
    (step, line_step): (usize, usize),
    (left, right): (usize, usize),
) {
    let size = (left + right + 1) as f64;
    let mut line = vec![0.0; length];
    for l in 0..lines {
        for c in 0..4 {
            let at = |i: usize| l * line_step + i * step + c;
            for (i, value) in line.iter_mut().enumerate() {
                *value = channels[at(i)];
            }
            let mut sum: f64 = line[..=right.min(length - 1)].iter().sum();
            for i in 0..length {
                channels[at(i)] = sum / size;
                if i + right + 1 < length {
                    sum += line[i + right + 1];
                }
                if i >= left {
                    sum -= line[i - left];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{PI, TAU};

    use super::{Cap, Join, Path2D, Smoothing, Surface, Transform};

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
    fn with_alpha_composites_straight_ink_source_over_then_restores_opaque_ink() {
        let mut surface = Surface::new(2, 1);
        surface.fill([0, 0, 0]);
        surface.with_alpha(0.85, |surface| {
            surface.fill_box(0.0, 0.0, 1.0, 1.0, [200, 100, 40]);
        });
        surface.fill_box(1.0, 0.0, 1.0, 1.0, [200, 100, 40]);
        let rgba = surface.into_image().rgba().to_vec();
        for (ours, blended) in rgba[..4].iter().zip([170, 85, 34, 255]) {
            assert!(ours.abs_diff(blended) <= 1, "{rgba:?}");
        }
        assert_eq!(&rgba[4..], [200, 100, 40, 255]);
    }

    #[test]
    fn with_clip_draws_only_inside_the_clip_then_restores_the_whole_frame() {
        let mut surface = Surface::new(3, 1);
        let mut band = Path2D::default();
        band.move_to(1.0, 0.0);
        band.line_to(2.0, 0.0);
        band.line_to(2.0, 1.0);
        band.line_to(1.0, 1.0);
        band.close();
        surface.with_clip(&band, Transform::IDENTITY, |surface| {
            surface.fill([255, 0, 0]);
        });
        let clipped = surface
            .0
            .pixels()
            .iter()
            .map(|p| p.red())
            .collect::<Vec<_>>();
        assert_eq!(clipped, [0, 255, 0]);
        surface.fill_box(0.0, 0.0, 3.0, 1.0, [0, 0, 255]);
        assert!(surface.0.pixels().iter().all(|p| p.blue() == 255));
    }

    #[test]
    fn a_nested_clip_draws_only_where_both_clips_cover_then_restores_the_outer() {
        let mut surface = Surface::new(3, 1);
        let mut left = Path2D::default();
        left.rect(0.0, 0.0, 2.0, 1.0);
        let mut right = Path2D::default();
        right.rect(1.0, 0.0, 2.0, 1.0);
        surface.with_clip(&left, Transform::IDENTITY, |surface| {
            surface.with_clip(&right, Transform::IDENTITY, |surface| {
                surface.fill([255, 0, 0]);
            });
            surface.fill_box(0.0, 0.0, 3.0, 1.0, [0, 0, 255]);
        });
        let image = surface.into_image();
        let pixels = image.rgba().as_chunks::<4>().0;
        assert_eq!(pixels, [[0, 0, 255, 255], [0, 0, 255, 255], [0, 0, 0, 0]]);
    }

    #[test]
    fn with_clip_places_the_clip_path_through_its_transform() {
        let mut surface = Surface::new(3, 1);
        let mut cell = Path2D::default();
        cell.rect(0.0, 0.0, 1.0, 1.0);
        let shift = Transform::IDENTITY.translate(2.0, 0.0);
        surface.with_clip(&cell, shift, |surface| surface.fill([255, 0, 0]));
        let red = |x: usize| surface.0.pixels()[x].red();
        assert_eq!([red(0), red(1), red(2)], [0, 0, 255]);
    }

    fn lit_cells(transform: Transform, rect: (f64, f64, f64, f64)) -> Vec<(u32, u32)> {
        let mut surface = Surface::new(10, 10);
        let mut path = Path2D::default();
        path.rect(rect.0, rect.1, rect.2, rect.3);
        surface.fill_transformed(&path, [255, 255, 255], transform);
        let image = surface.into_image();
        (0..100)
            .filter(|i| image.rgba()[*i as usize * 4] > 128)
            .map(|i| (i % 10, i / 10))
            .collect()
    }

    #[test]
    fn fill_transformed_turns_a_bar_about_the_translated_origin_as_canvas_does() {
        let turned = Transform::IDENTITY.translate(5.0, 5.0).rotate(PI / 2.0);
        let cells = lit_cells(turned, (-5.0, 0.0, 10.0, 1.0));
        assert_eq!(cells, (0..10).map(|y| (4, y)).collect::<Vec<_>>());
    }

    #[test]
    fn fill_transformed_mirrors_with_a_negative_scale() {
        let mirror = Transform::IDENTITY.translate(10.0, 0.0).scale(-1.0, 1.0);
        let cells = lit_cells(mirror, (0.0, 0.0, 3.0, 1.0));
        assert_eq!(cells, [(7, 0), (8, 0), (9, 0)]);
    }

    #[test]
    fn fill_transformed_scales_unit_space_into_a_cell() {
        let cell = Transform::IDENTITY.translate(2.0, 4.0).scale(3.0, 2.0);
        let cells = lit_cells(cell, (0.0, 0.0, 1.0, 1.0));
        assert_eq!(cells, [(2, 4), (3, 4), (4, 4), (2, 5), (3, 5), (4, 5)]);
    }

    fn stroked_arc(start: f64, end: f64) -> Vec<u8> {
        let mut surface = Surface::new(40, 40);
        let mut path = Path2D::default();
        path.ellipse(20.0, 20.0, 16.0, 8.0, PI / 2.0, start, end);
        surface.stroke(&path, [255, 255, 255], 2.0, Cap::Butt, Join::Round);
        surface.into_image().rgba().to_vec()
    }

    fn lit(rgba: &[u8], x: u32, y: u32) -> bool {
        rgba[((y * 40 + x) * 4) as usize] > 128
    }

    #[test]
    fn ellipse_strokes_a_rotated_partial_arc_from_start_to_end_angle() {
        let half = stroked_arc(0.0, PI);
        assert!(lit(&half, 19, 36), "start, rx rotated onto +y");
        assert!(lit(&half, 12, 20), "middle, ry rotated onto -x");
        assert!(lit(&half, 19, 4), "end");
        assert!(!lit(&half, 28, 20), "the other half is not drawn");
        assert!(
            lit(&stroked_arc(1.0, 1.0 + 3.0 * TAU), 28, 20),
            "a sweep past TAU is whole"
        );
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
    fn draw_smooth_nearest_gives_a_centre_on_a_cell_edge_to_the_left_cell() {
        let mut surface = Surface::new(5, 1);
        surface.draw_smooth(&[255, 0, 0, 255, 0, 0, 255, 255], 2, 1, Smoothing::Nearest);
        let rgba = surface.into_image().rgba().to_vec();
        assert_eq!(&rgba[8..12], [255, 0, 0, 255]);
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

    fn inked(cap: Cap, join: Join, x: u32, y: u32) -> bool {
        let mut surface = Surface::new(40, 40);
        let mut path = Path2D::default();
        path.move_to(10.0, 30.0);
        path.line_to(10.0, 10.0);
        path.line_to(30.0, 10.0);
        surface.stroke(&path, [255, 255, 255], 10.0, cap, join);
        let image = surface.into_image();
        image.rgba()[((y * 40 + x) * 4) as usize] > 128
    }

    #[test]
    fn stroke_draws_a_round_cap_past_the_end() {
        assert!(inked(Cap::Round, Join::Round, 32, 10));
        assert!(!inked(Cap::Butt, Join::Round, 32, 10));
    }

    #[test]
    fn stroke_draws_a_miter_join_into_the_corner() {
        assert!(inked(Cap::Butt, Join::Miter, 5, 5));
        assert!(!inked(Cap::Butt, Join::Round, 5, 5));
    }

    #[test]
    fn fill_rect_linear_ramps_between_stops_and_pads_past_the_ends() {
        let mut surface = Surface::new(10, 1);
        surface.fill_rect_linear(
            0,
            0,
            10,
            1,
            (2.0, 0.0),
            (8.0, 0.0),
            &[(0.0, [0, 0, 0]), (1.0, [240, 120, 0])],
        );
        let rgba = surface.into_image().rgba().to_vec();
        let red = |x: usize| rgba[x * 4];
        assert_eq!((red(0), red(1)), (0, 0), "padded before the start");
        assert_eq!((red(8), red(9)), (240, 240), "padded past the end");
        assert!(red(4).abs_diff(100) <= 2, "x 4.5 is 2.5/6 along: {rgba:?}");
        assert!(rgba.chunks(4).all(|px| px[2] == 0 && px[3] == 255));
    }

    #[test]
    fn with_multiply_darkens_by_the_ink_then_restores_source_over() {
        let mut surface = Surface::new(2, 1);
        surface.fill([200, 100, 50]);
        surface.with_multiply(|surface| {
            surface.fill_box(0.0, 0.0, 1.0, 1.0, [128, 255, 0]);
        });
        surface.fill_box(1.0, 0.0, 1.0, 1.0, [128, 255, 0]);
        let rgba = surface.into_image().rgba().to_vec();
        for (ours, multiplied) in rgba[..4].iter().zip([100, 100, 0, 255]) {
            assert!(ours.abs_diff(multiplied) <= 1, "{rgba:?}");
        }
        assert_eq!(&rgba[4..], [128, 255, 0, 255]);
    }

    #[test]
    fn with_plus_adds_the_ink_and_clamps_then_restores_source_over() {
        let mut surface = Surface::new(2, 1);
        surface.fill([200, 100, 0]);
        surface.with_plus(|surface| {
            surface.fill_box(0.0, 0.0, 1.0, 1.0, [100, 50, 7]);
        });
        surface.fill_box(1.0, 0.0, 1.0, 1.0, [100, 50, 7]);
        let rgba = surface.into_image().rgba().to_vec();
        assert_eq!(rgba, [255, 150, 7, 255, 100, 50, 7, 255]);
    }

    #[test]
    fn with_blur_spreads_a_dot_evenly_and_keeps_its_ink() {
        let mut surface = Surface::new(21, 21);
        surface.with_blur(2.0, |surface| {
            surface.fill_box(10.0, 10.0, 1.0, 1.0, [0, 0, 255]);
        });
        let image = surface.into_image();
        let alpha = |x: usize, y: usize| u32::from(image.rgba()[(y * 21 + x) * 4 + 3]);
        let total: u32 = (0..21)
            .flat_map(|y| (0..21).map(move |x| (x, y)))
            .map(|(x, y)| alpha(x, y))
            .sum();
        assert!(total.abs_diff(255) <= 30, "{total}");
        assert!(alpha(10, 10) > alpha(12, 10) && alpha(12, 10) > alpha(14, 10));
        assert_eq!(alpha(12, 10), alpha(8, 10));
        assert_eq!(alpha(10, 12), alpha(10, 8));
        assert_eq!(alpha(0, 0), 0);
    }
}
