use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{
    DrawSettings, Engine, HintingInstance, HintingOptions, OutlinePen, SmoothMode, Target,
};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, MetadataProvider};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

struct Glyph {
    ch: char,
    x: f32,
    y: f32,
    size: f32,
    rgb: [u8; 3],
}

struct Pen<'a> {
    pb: &'a mut PathBuilder,
    ox: f32,
    oy: f32,
}

impl OutlinePen for Pen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pb.move_to(self.ox + x, self.oy - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.pb.line_to(self.ox + x, self.oy - y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.pb.quad_to(self.ox + cx, self.oy - cy, self.ox + x, self.oy - y);
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.pb.cubic_to(
            self.ox + a,
            self.oy - b,
            self.ox + c,
            self.oy - d,
            self.ox + x,
            self.oy - y,
        );
    }
    fn close(&mut self) {
        self.pb.close();
    }
}

fn hex(s: &str) -> [u8; 3] {
    let n = u32::from_str_radix(s.trim_start_matches('#'), 16).expect("bad hex colour");
    [(n >> 16) as u8, (n >> 8) as u8, n as u8]
}

fn round64(v: f32) -> f32 {
    (v * 64.0).round() / 64.0
}

fn middle_offset(font: &FontRef, size: f32) -> f32 {
    let os2 = font.os2().expect("font has no OS/2 table");
    let asc = os2.s_typo_ascender() as f32;
    let desc = -(os2.s_typo_descender() as f32);
    let asc_n = round64(asc * size / (asc + desc));
    let desc_n = round64(size) - asc_n;
    (asc_n - desc_n) / 2.0
}

struct Variant {
    name: &'static str,
    engine: Option<Engine>,
    snap: bool,
}

fn render(font: &FontRef, w: u32, h: u32, paper: [u8; 3], glyphs: &[Glyph], v: &Variant) -> Pixmap {
    let mut pm = Pixmap::new(w, h).expect("pixmap");
    pm.fill(Color::from_rgba8(paper[0], paper[1], paper[2], 255));
    let outlines = font.outline_glyphs();
    let charmap = font.charmap();
    for g in glyphs {
        let gid = charmap.map(g.ch).unwrap_or_else(|| panic!("font lacks U+{:04X}", g.ch as u32));
        let size = Size::new(g.size);
        let adv = font
            .glyph_metrics(size, LocationRef::default())
            .advance_width(gid)
            .expect("advance");
        let mut ox = g.x - adv / 2.0;
        let mut oy = g.y + middle_offset(font, g.size);
        if v.snap {
            ox = (ox * 4.0).round() / 4.0;
            oy = oy.round();
        }
        let outline = outlines.get(gid).expect("outline");
        let mut pb = PathBuilder::new();
        let mut pen = Pen { pb: &mut pb, ox, oy };
        let inst;
        let settings = if let Some(engine) = v.engine.clone() {
            let target = Target::Smooth {
                mode: SmoothMode::Light,
                symmetric_rendering: true,
                preserve_linear_metrics: true,
            };
            inst = HintingInstance::new(
                &outlines,
                size,
                LocationRef::default(),
                HintingOptions { engine, target },
            )
            .expect("hinting instance");
            DrawSettings::hinted(&inst, false)
        } else {
            DrawSettings::unhinted(size, LocationRef::default())
        };
        outline.draw(settings, &mut pen).expect("draw");
        let Some(path) = pb.finish() else { continue };
        let mut paint = Paint::default();
        paint.set_color_rgba8(g.rgb[0], g.rgb[1], g.rgb[2], 255);
        paint.anti_alias = true;
        pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
    pm
}

fn luma48_share(a: &Pixmap, b: &Pixmap) -> f64 {
    assert_eq!((a.width(), a.height()), (b.width(), b.height()), "size mismatch");
    let over = a
        .data()
        .chunks_exact(4)
        .zip(b.data().chunks_exact(4))
        .filter(|(p, q)| {
            let d = |i: usize| (p[i] as i32 - q[i] as i32).unsigned_abs();
            (d(0) * 299 + d(1) * 587 + d(2) * 114) / 1000 > 48
        })
        .count();
    over as f64 / (a.width() * a.height()) as f64
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, glyphs_path, chrome_png, out_prefix] = args.as_slice() else {
        panic!("usage: kiosk-text-bench <font.ttf> <glyphs.tsv> <chrome.png> <out-prefix>");
    };
    let bytes = std::fs::read(font_path).expect("read font");
    let font = FontRef::new(&bytes).expect("parse font");
    let text = std::fs::read_to_string(glyphs_path).expect("read glyphs");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().expect("header").split('\t').collect();
    let (w, h, paper) = (head[0].parse().unwrap(), head[1].parse().unwrap(), hex(head[2]));
    let glyphs: Vec<Glyph> = lines
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Glyph {
                ch: char::from_u32(f[0].parse().unwrap()).unwrap(),
                x: f[1].parse().unwrap(),
                y: f[2].parse().unwrap(),
                size: f[3].parse().unwrap(),
                rgb: hex(f[4]),
            }
        })
        .collect();
    let chrome = Pixmap::load_png(chrome_png).expect("load chrome png");
    let variants = [
        Variant { name: "blank", engine: None, snap: false },
        Variant { name: "unhinted", engine: None, snap: false },
        Variant { name: "unhinted-snap", engine: None, snap: true },
        Variant { name: "interp-light", engine: Some(Engine::Interpreter), snap: false },
        Variant { name: "interp-light-snap", engine: Some(Engine::Interpreter), snap: true },
        Variant { name: "auto-light", engine: Some(Engine::Auto(None)), snap: false },
        Variant { name: "auto-light-snap", engine: Some(Engine::Auto(None)), snap: true },
    ];
    for v in &variants {
        let pm = render(&font, w, h, paper, if v.name == "blank" { &[] } else { &glyphs }, v);
        pm.save_png(format!("{out_prefix}{}.png", v.name)).expect("save png");
        println!("{}\t{:.3}%", v.name, luma48_share(&pm, &chrome) * 100.0);
    }
}
