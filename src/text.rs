use std::sync::OnceLock;

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{
    DrawSettings, Engine, GlyphStyles, HintingInstance, HintingOptions, OutlineGlyphCollection,
    OutlinePen, SmoothMode, Target,
};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, MetadataProvider};
use tiny_skia::PathBuilder;

use crate::surface::Surface;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Face {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "atlas is the first Tool to draw Book")
    )]
    Book,
    Bold,
}

impl Face {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Book => include_bytes!("../fonts/DejaVuSansMono.ttf"),
            Self::Bold => include_bytes!("../fonts/DejaVuSansMono-Bold.ttf"),
        }
    }
}

const MAX_HINTED_SIZE: f32 = 256.0;

pub(crate) struct Type {
    face: Face,
    font: FontRef<'static>,
    outlines: OutlineGlyphCollection<'static>,
    middle: (f32, f32),
    hinting: Vec<(f32, HintingInstance)>,
}

impl Type {
    pub(crate) fn new(face: Face) -> Self {
        let font = FontRef::new(face.bytes()).expect("the bundled font parses");
        let os2 = font.os2().expect("the bundled font has an OS/2 table");
        let middle = (
            f32::from(os2.s_typo_ascender()),
            -f32::from(os2.s_typo_descender()),
        );
        Self {
            face,
            outlines: font.outline_glyphs(),
            font,
            middle,
            hinting: Vec::new(),
        }
    }

    pub(crate) fn fill_centred(
        &mut self,
        surface: &mut Surface,
        ch: char,
        x: f32,
        y: f32,
        size: f32,
        ink: [u8; 3],
    ) {
        let gid = self
            .font
            .charmap()
            .map(ch)
            .expect("every character set is in the bundled font");
        let advance = self
            .font
            .glyph_metrics(Size::new(size), LocationRef::default())
            .advance_width(gid)
            .expect("the bundled font has hmtx");
        let mut left = x - advance / 2.0;
        let mut baseline = y + self.middle_offset(size);
        let hinted = size <= MAX_HINTED_SIZE;
        if hinted {
            left = (left * 4.0 + 0.5).floor() / 4.0;
            baseline = (baseline + 0.5).floor();
        }
        let outline = self
            .outlines
            .get(gid)
            .expect("a mapped glyph has an outline");
        let settings = if hinted {
            DrawSettings::hinted(self.hinting(size), false)
        } else {
            DrawSettings::unhinted(Size::new(size), LocationRef::default())
        };
        let mut pen = Pen {
            path: PathBuilder::new(),
            left,
            baseline,
        };
        outline
            .draw(settings, &mut pen)
            .expect("the bundled font draws");
        if let Some(path) = pen.path.finish() {
            surface.fill_path(&path, ink);
        }
    }

    fn middle_offset(&self, size: f32) -> f32 {
        let (ascender, descender) = self.middle;
        let ascent = sixty_fourths(ascender * size / (ascender + descender));
        let descent = sixty_fourths(size) - ascent;
        (ascent - descent) / 2.0
    }

    fn hinting(&mut self, size: f32) -> &HintingInstance {
        let slot = match self.hinting.iter().position(|(s, _)| *s == size) {
            Some(slot) => slot,
            None => {
                let instance = HintingInstance::new(
                    &self.outlines,
                    Size::new(size),
                    LocationRef::default(),
                    HintingOptions {
                        engine: Engine::Auto(Some(styles(self.face, &self.outlines))),
                        target: Target::Smooth {
                            mode: SmoothMode::Light,
                            symmetric_rendering: true,
                            preserve_linear_metrics: true,
                        },
                    },
                )
                .expect("the bundled font hints");
                self.hinting.push((size, instance));
                self.hinting.len() - 1
            }
        };
        &self.hinting[slot].1
    }
}

fn styles(face: Face, outlines: &OutlineGlyphCollection) -> GlyphStyles {
    static STYLES: [OnceLock<GlyphStyles>; 2] = [OnceLock::new(), OnceLock::new()];
    STYLES[face as usize]
        .get_or_init(|| GlyphStyles::new(outlines))
        .clone()
}

fn sixty_fourths(value: f32) -> f32 {
    (value * 64.0).round() / 64.0
}

struct Pen {
    path: PathBuilder,
    left: f32,
    baseline: f32,
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.left + x, self.baseline - y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.left + x, self.baseline - y);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path.quad_to(
            self.left + cx,
            self.baseline - cy,
            self.left + x,
            self.baseline - y,
        );
    }

    fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
        self.path.cubic_to(
            self.left + ax,
            self.baseline - ay,
            self.left + bx,
            self.baseline - by,
            self.left + x,
            self.baseline - y,
        );
    }

    fn close(&mut self) {
        self.path.close();
    }
}

#[cfg(test)]
mod tests {
    use super::{Face, Type};
    use crate::surface::Surface;

    fn inked(face: Face) -> usize {
        let mut surface = Surface::new(64, 64);
        surface.fill([255, 255, 255]);
        Type::new(face).fill_centred(&mut surface, '#', 32.0, 32.0, 48.0, [0, 0, 0]);
        surface
            .into_image()
            .rgba()
            .chunks(4)
            .filter(|px| px[0] < 128)
            .count()
    }

    #[test]
    fn book_inks_a_lighter_glyph_than_bold() {
        let (book, bold) = (inked(Face::Book), inked(Face::Bold));
        assert!(book > 0 && book < bold, "book {book}, bold {bold}");
    }
}
