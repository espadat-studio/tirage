use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

struct Pen(PathBuilder, f32, f32);

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(self.1 + x, self.2 - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(self.1 + x, self.2 - y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0.quad_to(self.1 + cx, self.2 - cy, self.1 + x, self.2 - y);
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.0.cubic_to(self.1 + a, self.2 - b, self.1 + c, self.2 - d, self.1 + x, self.2 - y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    Box::leak(vec![0u8; len].into_boxed_slice()).as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn glyph_ink(font: *const u8, len: usize, cp: u32, px: f32) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(font, len) };
    let Ok(font) = FontRef::new(bytes) else { return 0 };
    let Some(gid) = char::from_u32(cp).and_then(|c| font.charmap().map(c)) else { return 0 };
    let outlines = font.outline_glyphs();
    let Some(outline) = outlines.get(gid) else { return 0 };
    let size = Size::new(px);
    let mut pen = Pen(PathBuilder::new(), 8.0, px);
    #[cfg(feature = "autohint")]
    let inst = skrifa::outline::HintingInstance::new(
        &outlines,
        size,
        LocationRef::default(),
        skrifa::outline::HintingOptions {
            engine: skrifa::outline::Engine::Auto(None),
            target: skrifa::outline::Target::Smooth {
                mode: skrifa::outline::SmoothMode::Light,
                symmetric_rendering: true,
                preserve_linear_metrics: true,
            },
        },
    )
    .expect("hinting");
    #[cfg(feature = "autohint")]
    let settings = DrawSettings::hinted(&inst, false);
    #[cfg(not(feature = "autohint"))]
    let settings = DrawSettings::unhinted(size, LocationRef::default());
    if outline.draw(settings, &mut pen).is_err() {
        return 0;
    }
    let Some(path) = pen.0.finish() else { return 0 };
    let side = (px * 2.0) as u32;
    let Some(mut pm) = Pixmap::new(side, side) else { return 0 };
    let mut paint = Paint::default();
    paint.anti_alias = true;
    pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    pm.data().chunks_exact(4).map(|p| p[3] as u32).sum()
}
