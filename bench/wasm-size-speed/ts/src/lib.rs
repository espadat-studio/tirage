#[path = "../../shared/scene.rs"]
pub mod scene;

use scene::{Cmd, H, W};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, RadialGradient, Rect, SpreadMode,
    Transform,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Renderer {
    pixmap: Pixmap,
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Renderer {
        Renderer { pixmap: Pixmap::new(W, H).unwrap() }
    }

    pub fn paths(&mut self, frame: u32) {
        let t = frame as f32 / 60.0;
        let px = &mut self.pixmap;
        let mut paint = Paint::default();
        paint.shader = LinearGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(W as f32, H as f32),
            vec![
                GradientStop::new(0.0, Color::from_rgba8(20, 10, 60, 255)),
                GradientStop::new(1.0, Color::from_rgba8(240, 120, 40, 255)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();
        px.fill_rect(Rect::from_xywh(0.0, 0.0, W as f32, H as f32).unwrap(), &paint, Transform::identity(), None);

        let mut radial = RadialGradient::new(
            Point::from_xy(540.0, 900.0),
            0.0,
            Point::from_xy(540.0, 900.0),
            700.0,
            vec![
                GradientStop::new(0.0, Color::from_rgba8(255, 255, 220, 255)),
                GradientStop::new(1.0, Color::from_rgba8(255, 0, 120, 0)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap();
        radial.apply_opacity(0.7);
        paint.shader = radial;
        let circle = PathBuilder::from_circle(540.0, 900.0, 700.0).unwrap();
        px.fill_path(&circle, &paint, FillRule::Winding, Transform::identity(), None);

        for s in scene::build(7, t) {
            let mut pb = PathBuilder::new();
            for c in s.cmds {
                match c {
                    Cmd::M(x, y) => pb.move_to(x, y),
                    Cmd::L(x, y) => pb.line_to(x, y),
                    Cmd::C(a, b, c, d, e, f) => pb.cubic_to(a, b, c, d, e, f),
                    Cmd::Z => pb.close(),
                }
            }
            let Some(path) = pb.finish() else { continue };
            paint.set_color_rgba8(s.rgb[0], s.rgb[1], s.rgb[2], (s.alpha * 255.0) as u8);
            px.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
        }
    }

    pub fn grain(&mut self, frame: u32) {
        scene::grain(self.pixmap.data_mut(), frame, 24);
    }

    pub fn frame(&mut self, frame: u32) -> u32 {
        self.paths(frame);
        self.grain(frame);
        scene::checksum(self.pixmap.data())
    }

    pub fn ptr(&self) -> *const u8 {
        self.pixmap.data().as_ptr()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
