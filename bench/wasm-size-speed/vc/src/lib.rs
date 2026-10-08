#[path = "../../shared/scene.rs"]
pub mod scene;

use scene::{Cmd, H, W};
use vello_cpu::color::AlphaColor;
use vello_cpu::kurbo::{BezPath, Circle, Point, Rect, Shape};
use vello_cpu::peniko::{ColorStop, ColorStops, Gradient, LinearGradientPosition, RadialGradientPosition};
use vello_cpu::{Pixmap, RenderContext, Resources};
use wasm_bindgen::prelude::*;

type Color = AlphaColor<vello_cpu::color::Srgb>;

#[wasm_bindgen]
pub struct Renderer {
    ctx: RenderContext,
    res: Resources,
    pixmap: Pixmap,
}

fn stops(a: Color, b: Color) -> ColorStops {
    ColorStops::from([ColorStop::from((0.0, a)), ColorStop::from((1.0, b))].as_slice())
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Renderer {
        Renderer {
            ctx: RenderContext::new(W as u16, H as u16),
            res: Resources::new(),
            pixmap: Pixmap::new(W as u16, H as u16),
        }
    }

    pub fn level(&self) -> String {
        format!("{:?}", self.ctx.render_settings().level)
    }

    pub fn paths(&mut self, frame: u32) {
        let t = frame as f32 / 60.0;
        let ctx = &mut self.ctx;
        ctx.reset();
        ctx.set_paint(Gradient {
            kind: LinearGradientPosition { start: Point::new(0.0, 0.0), end: Point::new(W as f64, H as f64) }.into(),
            stops: stops(Color::from_rgba8(20, 10, 60, 255), Color::from_rgba8(240, 120, 40, 255)),
            ..Default::default()
        });
        ctx.fill_rect(&Rect::new(0.0, 0.0, W as f64, H as f64));

        ctx.set_paint(
            Gradient {
                kind: RadialGradientPosition {
                    start_center: Point::new(540.0, 900.0),
                    start_radius: 0.0,
                    end_center: Point::new(540.0, 900.0),
                    end_radius: 700.0,
                }
                .into(),
                stops: stops(Color::from_rgba8(255, 255, 220, 255), Color::from_rgba8(255, 0, 120, 0)),
                ..Default::default()
            }
            .multiply_alpha(0.7),
        );
        ctx.fill_path(&Circle::new((540.0, 900.0), 700.0).to_path(0.1));

        for s in scene::build(7, t) {
            let mut p = BezPath::new();
            for c in s.cmds {
                match c {
                    Cmd::M(x, y) => p.move_to((x as f64, y as f64)),
                    Cmd::L(x, y) => p.line_to((x as f64, y as f64)),
                    Cmd::C(a, b, c, d, e, f) => {
                        p.curve_to((a as f64, b as f64), (c as f64, d as f64), (e as f64, f as f64))
                    }
                    Cmd::Z => p.close_path(),
                }
            }
            ctx.set_paint(Color::from_rgba8(s.rgb[0], s.rgb[1], s.rgb[2], (s.alpha * 255.0) as u8));
            ctx.fill_path(&p);
        }
        ctx.flush();
        ctx.render(&mut self.pixmap, &mut self.res);
    }

    pub fn grain(&mut self, frame: u32) {
        scene::grain(self.pixmap.data_as_u8_slice_mut(), frame, 24);
    }

    pub fn frame(&mut self, frame: u32) -> u32 {
        self.paths(frame);
        self.grain(frame);
        scene::checksum(self.pixmap.data_as_u8_slice())
    }

    pub fn ptr(&self) -> *const u8 {
        self.pixmap.data_as_u8_slice().as_ptr()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
