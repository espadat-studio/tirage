use std::fmt::Write;

use tiny_skia::{Path, PathBuilder, PathSegment, Rect, Transform};

use crate::Image;
use crate::surface::{Cap, Join, nearest_source};

pub(crate) struct SvgWriter {
    width: u32,
    height: u32,
    markup: String,
    group: Option<([u8; 3], bool)>,
    gradients: u32,
    clips: Vec<u32>,
    next_clip: u32,
    alpha: f32,
    multiply: bool,
    strokes: Vec<String>,
    pixels: Vec<(u32, u32, [u8; 3])>,
    xlink: bool,
}

impl SvgWriter {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            markup: format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
            ),
            group: None,
            gradients: 0,
            clips: Vec::new(),
            next_clip: 0,
            alpha: 1.0,
            multiply: false,
            strokes: Vec::new(),
            pixels: Vec::new(),
            xlink: false,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    fn opacity(&mut self, paint: &str, opacity: f32) {
        if opacity < 1.0 {
            write!(self.markup, r#" {paint}-opacity="{opacity:.4}""#)
                .expect("a String takes any write");
        }
    }

    pub(crate) fn set_multiply(&mut self, multiply: bool) {
        self.multiply = multiply;
    }

    fn open(&mut self, ink: [u8; 3], crisp: bool) {
        self.group(ink, crisp);
        self.open_wrappers();
    }

    fn open_smooth(&mut self, ink: [u8; 3]) {
        match self.group {
            Some((_, false)) if self.pixels.is_empty() => self.open_wrappers(),
            _ => self.open(ink, false),
        }
    }

    fn open_wrappers(&mut self) {
        if self.multiply {
            self.markup
                .push_str(r#"<g style="mix-blend-mode:multiply">"#);
        }
        if let Some(id) = self.clips.last() {
            write!(
                self.markup,
                r#"<g clip-path="url(#c{id})" filter="url(#frame)">"#
            )
            .expect("a String takes any write");
        }
    }

    fn end_group(&mut self) {
        if self.group.take().is_some() {
            self.markup.push_str("</g>");
        }
        self.flush_pixels();
    }

    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, ink: [u8; 3]) {
        assert!(self.clips.is_empty(), "pixel writes ignore a clip");
        self.pixels.push((x, y, ink));
    }

    fn flush_pixels(&mut self) {
        if self.pixels.is_empty() {
            return;
        }
        let pixels = std::mem::take(&mut self.pixels);
        let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
        for &(x, y, _) in &pixels {
            (left, top) = (left.min(x), top.min(y));
            (right, bottom) = (right.max(x), bottom.max(y));
        }
        let (width, height) = (right - left + 1, bottom - top + 1);
        let mut rgba = vec![0; (width * height * 4) as usize];
        for (x, y, [r, g, b]) in pixels {
            let at = (((y - top) * width + x - left) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[r, g, b, 255]);
        }
        let png = Image {
            width,
            height,
            rgba,
        }
        .to_png();
        write!(
            self.markup,
            r#"<image x="{left}" y="{top}" width="{width}" height="{height}" image-rendering="optimizeSpeed" xlink:href="data:image/png;base64,{}"/>"#,
            base64(&png)
        )
        .expect("a String takes any write");
        self.xlink = true;
    }

    pub(crate) fn push_clip(&mut self, path: &Path, transform: Transform) {
        self.end_group();
        let id = self.next_clip;
        self.next_clip += 1;
        if id == 0 {
            let (width, height) = (self.width, self.height);
            write!(
                self.markup,
                r#"<filter id="frame" filterUnits="userSpaceOnUse" x="0" y="0" width="{width}" height="{height}" color-interpolation-filters="sRGB"><feOffset/></filter>"#
            )
            .expect("a String takes any write");
        }
        write!(self.markup, r#"<clipPath id="c{id}""#).expect("a String takes any write");
        if let Some(outer) = self.clips.last() {
            write!(self.markup, r#" clip-path="url(#c{outer})""#)
                .expect("a String takes any write");
        }
        self.markup.push('>');
        if transform.is_identity() {
            self.open_path(path, true);
            self.markup.push('"');
        } else {
            self.open_path(path, false);
            self.matrix(transform);
        }
        self.markup.push_str("/></clipPath>");
        self.clips.push(id);
    }

    pub(crate) fn pop_clip(&mut self) {
        self.clips.pop().expect("a pop follows its push");
    }

    fn matrix(&mut self, transform: Transform) {
        let Transform {
            sx,
            ky,
            kx,
            sy,
            tx,
            ty,
        } = transform;
        write!(
            self.markup,
            r#"" transform="matrix({sx} {ky} {kx} {sy} {tx} {ty})""#
        )
        .expect("a String takes any write");
    }

    fn close(&mut self) {
        self.markup.push_str("/>");
        if !self.clips.is_empty() {
            self.markup.push_str("</g>");
        }
        if self.multiply {
            self.markup.push_str("</g>");
        }
    }

    fn alpha8(&self) -> i32 {
        (self.alpha * 255.0 + 0.5) as i32
    }

    fn faded(&self, ink: [u8; 3]) -> [u8; 3] {
        if self.alpha >= 1.0 {
            return ink;
        }
        shifted(self.alpha8() as f32 / 255.0, premultiplied(ink, self.alpha))
    }

    fn faded_stroke(&self, ink: [u8; 3], width: f32) -> ([u8; 3], f32) {
        if self.alpha >= 1.0 || width <= 0.0 || width >= 1.0 {
            return (self.faded(ink), self.alpha);
        }
        let hairline = ((255 * (width * 256.0) as i32) >> 8) as f32 / 255.0;
        let alpha = self.alpha * hairline;
        let target = (alpha * 255.0 + 0.5) as i32;
        let opacity = (0..=255)
            .filter(|&o| (o as f32 / 255.0 * hairline * 255.0 + 0.5) as i32 == target)
            .min_by_key(|o| (o - self.alpha8()).abs())
            .expect("some u8 opacity scales to any hairline alpha") as f32
            / 255.0;
        (
            shifted(opacity * hairline, premultiplied(ink, alpha)),
            opacity,
        )
    }

    fn group(&mut self, ink: [u8; 3], crisp: bool) {
        if !self.pixels.is_empty() {
            self.end_group();
        }
        if self.group == Some((ink, crisp)) {
            return;
        }
        if self.group.is_some() {
            self.markup.push_str("</g>");
        }
        self.group = Some((ink, crisp));
        let [r, g, b] = ink;
        write!(self.markup, r##"<g fill="#{r:02x}{g:02x}{b:02x}""##)
            .expect("a String takes any write");
        if crisp {
            self.markup.push_str(r#" shape-rendering="crispEdges""#);
        }
        self.markup.push('>');
    }

    pub(crate) fn fill(&mut self, ink: [u8; 3]) {
        self.open(self.faded(ink), false);
        let (width, height) = (self.width, self.height);
        write!(self.markup, r#"<rect width="{width}" height="{height}""#)
            .expect("a String takes any write");
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, ink: [u8; 3]) {
        self.open(self.faded(ink), true);
        write!(
            self.markup,
            r#"<rect x="{x}" y="{y}" width="{width}" height="{height}""#
        )
        .expect("a String takes any write");
        self.opacity("fill", self.alpha);
        self.close();
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
        assert!(self.clips.is_empty(), "a ramp ignores a clip");
        self.end_group();
        let id = self.gradients;
        self.gradients += 1;
        let [x0, y0, x1, y1] = [x0, y0, x1, y1].map(|v| v as f32);
        write!(
            self.markup,
            r#"<linearGradient id="r{id}" gradientUnits="userSpaceOnUse" x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}">"#
        )
        .expect("a String takes any write");
        for &(offset, [r, g, b]) in stops {
            write!(
                self.markup,
                r##"<stop offset="{}" stop-color="#{r:02x}{g:02x}{b:02x}"/>"##,
                offset as f32
            )
            .expect("a String takes any write");
        }
        write!(
            self.markup,
            r#"</linearGradient><rect x="{x}" y="{y}" width="{width}" height="{height}" fill="url(#r{id})" shape-rendering="crispEdges"/>"#
        )
        .expect("a String takes any write");
    }

    pub(crate) fn draw_nearest(&mut self, rgba: &[u8], width: u32, height: u32) {
        assert!(self.clips.is_empty(), "pixel writes ignore a clip");
        assert!(
            self.alpha >= 1.0 && !self.multiply,
            "pixel writes are opaque and source-over"
        );
        let edges = |edge: u32, scaled: u32| {
            (0..=edge)
                .map(|cell| {
                    (0..scaled)
                        .find(|&at| nearest_source(at, edge, scaled) >= cell)
                        .unwrap_or(scaled)
                })
                .collect::<Vec<_>>()
        };
        let (xs, ys) = (edges(width, self.width), edges(height, self.height));
        let pixels = rgba.as_chunks::<4>().0;
        for (row, cells) in pixels.chunks(width as usize).enumerate() {
            let (y, rows) = (ys[row], ys[row + 1] - ys[row]);
            let mut start = 0;
            for end in 1..=cells.len() {
                if end < cells.len() && cells[end] == cells[start] {
                    continue;
                }
                let [r, g, b, a] = cells[start];
                assert_eq!(a, 255, "a nearest buffer is opaque");
                let columns = xs[end] - xs[start];
                if columns > 0 && rows > 0 {
                    self.fill_rect(xs[start] as i32, y as i32, columns, rows, [r, g, b]);
                }
                start = end;
            }
        }
    }

    pub(crate) fn fill_box(&mut self, x: f32, y: f32, width: f32, height: f32, ink: [u8; 3]) {
        let dot8 = |v: f32| ((v * 65536.0) as i32 + 0x80) >> 8;
        let (l, t) = (dot8(x.max(0.0)), dot8(y.max(0.0)));
        let r = dot8((x + width).min(self.width as f32));
        let b = dot8((y + height).min(self.height as f32));
        if l >= r || t >= b {
            return;
        }
        let mut top = t >> 8;
        if top == (b - 1) >> 8 {
            self.coverage_row(l, top, r, b - t - 1, ink);
            return;
        }
        if t & 0xFF != 0 {
            self.coverage_row(l, top, r, 256 - (t & 0xFF), ink);
            top += 1;
        }
        let bottom = b >> 8;
        if bottom > top {
            let rows = bottom - top;
            let mut left = l >> 8;
            if left == (r - 1) >> 8 {
                self.coverage(left, top, 1, rows, r - l - 1, ink);
            } else {
                if l & 0xFF != 0 {
                    self.coverage(left, top, 1, rows, 256 - (l & 0xFF), ink);
                    left += 1;
                }
                let right = r >> 8;
                self.coverage(left, top, right - left, rows, 255, ink);
                self.coverage(right, top, 1, rows, r & 0xFF, ink);
            }
        }
        self.coverage_row(l, bottom, r, b & 0xFF, ink);
    }

    fn coverage_row(&mut self, l: i32, y: i32, r: i32, alpha: i32, ink: [u8; 3]) {
        let scaled = |share: i32| (alpha * share) >> 8;
        let mut left = l >> 8;
        if left == (r - 1) >> 8 {
            self.coverage(left, y, 1, 1, scaled(r - l), ink);
            return;
        }
        if l & 0xFF != 0 {
            self.coverage(left, y, 1, 1, scaled(256 - (l & 0xFF)), ink);
            left += 1;
        }
        let right = r >> 8;
        self.coverage(left, y, right - left, 1, alpha, ink);
        self.coverage(right, y, 1, 1, scaled(r & 0xFF), ink);
    }

    fn coverage(&mut self, x: i32, y: i32, width: i32, height: i32, alpha: i32, ink: [u8; 3]) {
        if width <= 0 || alpha <= 0 {
            return;
        }
        let (width, height) = (width as u32, height as u32);
        if alpha >= 255 {
            self.fill_rect(x, y, width, height, ink);
            return;
        }
        let (base, opacity) = if self.alpha < 1.0 {
            (
                premultiplied(ink, self.alpha),
                (self.alpha8() * alpha + 128) >> 8,
            )
        } else {
            (ink.map(i32::from), alpha)
        };
        if opacity <= 0 {
            return;
        }
        let shifted = shifted(opacity as f32 / 255.0, base.map(|c| (c * alpha + 128) >> 8));
        self.open(ink, true);
        write!(
            self.markup,
            r#"<rect x="{x}" y="{y}" width="{width}" height="{height}""#
        )
        .expect("a String takes any write");
        if shifted != ink {
            let [r, g, b] = shifted;
            write!(self.markup, r##" fill="#{r:02x}{g:02x}{b:02x}""##)
                .expect("a String takes any write");
        }
        let opacity = opacity as f32 / 255.0;
        write!(self.markup, r#" fill-opacity="{opacity:.4}""#).expect("a String takes any write");
        self.close();
    }

    fn misses_frame(&self, bounds: Rect, reach: f32) -> bool {
        bounds.right() + reach < 0.0
            || bounds.bottom() + reach < 0.0
            || bounds.left() - reach > self.width as f32
            || bounds.top() - reach > self.height as f32
    }

    fn trimmed(&self, path: &Path, reach: f32) -> Option<Path> {
        let mut runs: Vec<Vec<tiny_skia::Point>> = Vec::new();
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => runs.push(vec![p]),
                PathSegment::LineTo(p) => runs.last_mut().expect("a line follows a move").push(p),
                _ => return Some(path.clone()),
            }
        }
        let mut trimmed = PathBuilder::new();
        for points in runs {
            let seen: Vec<bool> = points
                .windows(2)
                .map(|ends| {
                    let bounds = Rect::from_points(ends).expect("two finite points bound a rect");
                    !self.misses_frame(bounds, reach)
                })
                .collect();
            let mut open = false;
            for (i, ends) in points.windows(2).enumerate() {
                let kept = seen[i] || (i > 0 && seen[i - 1]) || seen.get(i + 1) == Some(&true);
                if !kept {
                    open = false;
                    continue;
                }
                if !open {
                    trimmed.move_to(ends[0].x, ends[0].y);
                    open = true;
                }
                trimmed.line_to(ends[1].x, ends[1].y);
            }
        }
        trimmed.finish()
    }

    pub(crate) fn fill_path(&mut self, path: &Path, ink: [u8; 3]) {
        if self.misses_frame(path.bounds(), 0.0) {
            return;
        }
        self.open(self.faded(ink), false);
        self.open_path(path, true);
        self.markup.push('"');
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn fill_even_odd(&mut self, path: &Path, ink: [u8; 3]) {
        if self.misses_frame(path.bounds(), 0.0) {
            return;
        }
        self.open(self.faded(ink), false);
        self.open_path(path, true);
        self.markup.push_str(r#"" fill-rule="evenodd""#);
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn fill_transformed(&mut self, path: &Path, ink: [u8; 3], transform: Transform) {
        if transform.is_identity() {
            self.fill_path(path, ink);
            return;
        }
        if let Some(placed) = path.clone().transform(transform)
            && self.misses_frame(placed.bounds(), 0.0)
        {
            return;
        }
        self.open(self.faded(ink), false);
        self.open_path(path, false);
        self.matrix(transform);
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn stroke(&mut self, path: &Path, ink: [u8; 3], width: f32, cap: Cap, join: Join) {
        let reach = match join {
            Join::Round => width,
            Join::Miter => width * 5.0,
        };
        if self.misses_frame(path.bounds(), reach) {
            return;
        }
        let Some(path) = &self.trimmed(path, reach) else {
            return;
        };
        let (ink, opacity) = self.faded_stroke(ink, width);
        let parts = if self.multiply || (!self.clips.is_empty() && width <= 1.0) {
            subpaths(path)
        } else {
            vec![path.clone()]
        };
        let class = self.stroke_class(ink, width, opacity, cap, join);
        for part in &parts {
            self.open_smooth(ink);
            self.open_path(part, false);
            write!(self.markup, r#"" class="s{class}""#).expect("a String takes any write");
            self.close();
        }
    }

    fn stroke_class(
        &mut self,
        [r, g, b]: [u8; 3],
        width: f32,
        opacity: f32,
        cap: Cap,
        join: Join,
    ) -> usize {
        let mut style = format!("fill:none;stroke:#{r:02x}{g:02x}{b:02x};stroke-width:{width}");
        if opacity < 1.0 {
            write!(style, ";stroke-opacity:{opacity:.4}").expect("a String takes any write");
        }
        if let Cap::Round = cap {
            style.push_str(";stroke-linecap:round");
        }
        style.push_str(match join {
            Join::Round => ";stroke-linejoin:round",
            Join::Miter => ";stroke-miterlimit:10",
        });
        self.strokes
            .iter()
            .position(|known| *known == style)
            .unwrap_or_else(|| {
                self.strokes.push(style);
                self.strokes.len() - 1
            })
    }

    fn open_path(&mut self, path: &Path, snap: bool) {
        self.markup.push_str(r#"<path d=""#);
        let bounds = path.bounds();
        let unclipped = snap
            && bounds.left() >= 0.0
            && bounds.top() >= 0.0
            && bounds.right() <= self.width as f32
            && bounds.bottom() <= self.height as f32;
        let lands = if unclipped { on_edge } else { exact };
        let (mut pen, mut start) = ((0.0, 0.0), (0.0, 0.0));
        let mut run = ' ';
        let mut dotted = None;
        for segment in path.segments() {
            let (command, points) = match segment {
                PathSegment::MoveTo(p) => ('m', vec![p]),
                PathSegment::LineTo(p) => ('l', vec![p]),
                PathSegment::QuadTo(c, p) => ('q', vec![c, p]),
                PathSegment::CubicTo(a, b, p) => ('c', vec![a, b, p]),
                PathSegment::Close => {
                    self.markup.push('z');
                    (pen, run, dotted) = (start, 'z', None);
                    continue;
                }
            };
            let near: Option<Vec<_>> = points
                .iter()
                .flat_map(|p| [(pen.0, p.x), (pen.1, p.y)])
                .map(|(base, v)| relative(base, v, lands))
                .collect();
            let far: Vec<_> = points
                .iter()
                .flat_map(|p| [p.x, p.y])
                .map(|v| absolute(v, lands))
                .collect();
            let length =
                |texts: &[(String, f64)]| texts.iter().map(|(text, _)| text.len()).sum::<usize>();
            let (command, texts) = match near {
                Some(near) if length(&near) <= length(&far) => (command, near),
                _ => (command.to_ascii_uppercase(), far),
            };
            if command != run {
                self.markup.push(command);
                dotted = None;
            }
            for (text, _) in &texts {
                let joins =
                    text.starts_with('-') || (text.starts_with('.') && dotted == Some(true));
                if dotted.is_some() && !joins {
                    self.markup.push(' ');
                }
                dotted = Some(text.contains('.'));
                self.markup.push_str(text);
            }
            pen = (texts[texts.len() - 2].1, texts[texts.len() - 1].1);
            run = command;
            if command.eq_ignore_ascii_case(&'m') {
                start = pen;
                run = if command == 'm' { 'l' } else { 'L' };
            }
        }
    }

    pub(crate) fn finish(mut self) -> String {
        self.end_group();
        if !self.strokes.is_empty() {
            let root = self.markup.find('>').expect("the root tag closes") + 1;
            let css: String = self
                .strokes
                .iter()
                .enumerate()
                .map(|(class, style)| format!(".s{class}{{{style}}}"))
                .collect();
            self.markup
                .insert_str(root, &format!("<style>{css}</style>"));
        }
        if self.xlink {
            let svg = r#"xmlns="http://www.w3.org/2000/svg""#;
            let xlink = r#" xmlns:xlink="http://www.w3.org/1999/xlink""#;
            self.markup.insert_str(svg.len() + 5, xlink);
        }
        self.markup + "</svg>\n"
    }
}

fn subpaths(path: &Path) -> Vec<Path> {
    let mut parts = Vec::new();
    let mut part = PathBuilder::new();
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                parts.extend(std::mem::take(&mut part).finish());
                part.move_to(p.x, p.y);
            }
            PathSegment::LineTo(p) => part.line_to(p.x, p.y),
            PathSegment::QuadTo(c, p) => part.quad_to(c.x, c.y, p.x, p.y),
            PathSegment::CubicTo(a, b, p) => part.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y),
            PathSegment::Close => part.close(),
        }
    }
    parts.extend(part.finish());
    parts
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let word = chunk
            .iter()
            .enumerate()
            .fold(0, |word, (i, &byte)| word | u32::from(byte) << (16 - 8 * i));
        for i in 0..4 {
            text.push(if i <= chunk.len() {
                char::from(ALPHABET[(word >> (18 - 6 * i) & 63) as usize])
            } else {
                '='
            });
        }
    }
    text
}

fn premultiplied(ink: [u8; 3], alpha: f32) -> [i32; 3] {
    ink.map(|c| (f32::from(c) / 255.0 * alpha * 255.0 + 0.5) as i32)
}

fn shifted(opacity: f32, scaled: [i32; 3]) -> [u8; 3] {
    scaled.map(|scaled| {
        (0..=255u8)
            .find(|&v| (f32::from(v) / 255.0 * opacity * 255.0 + 0.5) as i32 == scaled)
            .expect("ink 255 at this opacity reaches past any scaled ink")
    })
}

const MAX_DELTA_DIGITS: i32 = 12;

fn on_edge(sum: f64, v: f32) -> bool {
    let edge = |v: f32| (v * 256.0) as i32;
    edge(sum as f32) == edge(v)
}

fn exact(sum: f64, v: f32) -> bool {
    sum as f32 == v
}

fn relative(base: f64, v: f32, lands: fn(f64, f32) -> bool) -> Option<(String, f64)> {
    shortest(base, v, lands, MAX_DELTA_DIGITS)
}

fn absolute(v: f32, lands: fn(f64, f32) -> bool) -> (String, f64) {
    shortest(0.0, v, lands, i32::from(u8::MAX)).expect("v's own digits land on v")
}

fn shortest(
    base: f64,
    v: f32,
    lands: fn(f64, f32) -> bool,
    max_digits: i32,
) -> Option<(String, f64)> {
    let delta = f64::from(v) - base;
    let (text, sum) = (0..=max_digits)
        .flat_map(|digits| {
            let scale = 10f64.powi(digits);
            let scaled = delta * scale;
            [scaled.floor(), scaled.ceil()].map(|d| format!("{:.*}", digits as usize, d / scale))
        })
        .map(|text| {
            let sum = base + text.parse::<f64>().expect("a formatted f64 parses");
            (text, sum)
        })
        .find(|&(_, sum)| lands(sum, v))?;
    let digits = text.trim_start_matches('-').trim_start_matches('0');
    let text = if digits.chars().all(|c| c == '0' || c == '.') {
        "0".to_owned()
    } else if text.starts_with('-') {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    Some((text, sum))
}

#[cfg(test)]
mod tests {
    use tiny_skia::{PathBuilder, Transform};

    use super::SvgWriter;
    use crate::surface::{Cap, Join};

    #[test]
    fn an_empty_writer_gives_a_root_svg_sized_by_width_height_and_view_box() {
        assert_eq!(
            SvgWriter::new(540, 960).finish(),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"540\" height=\"960\" viewBox=\"0 0 540 960\"></svg>\n"
        );
    }

    fn body(svg: String) -> String {
        let open = svg.find('>').unwrap() + 1;
        svg[open..].trim_end_matches("</svg>\n").to_owned()
    }

    #[test]
    fn fill_covers_the_frame_with_one_rect_in_rrggbb() {
        let mut svg = SvgWriter::new(3, 2);
        svg.fill([10, 255, 0]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#0aff00"><rect width="3" height="2"/></g>"##
        );
    }

    #[test]
    fn fill_rect_writes_a_crisp_edged_rect_at_its_pixel_corner() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(-1, 4, 2, 3, [255, 209, 102]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#ffd166" shape-rendering="crispEdges"><rect x="-1" y="4" width="2" height="3"/></g>"##
        );
    }

    #[test]
    fn a_run_of_one_ink_shares_a_group_and_a_new_ink_opens_the_next() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        svg.fill_rect(1, 0, 1, 1, [0, 0, 0]);
        svg.fill_rect(2, 0, 1, 1, [255, 255, 255]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#000000" shape-rendering="crispEdges">"##,
                r#"<rect x="0" y="0" width="1" height="1"/><rect x="1" y="0" width="1" height="1"/></g>"#,
                r##"<g fill="#ffffff" shape-rendering="crispEdges"><rect x="2" y="0" width="1" height="1"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn fill_box_writes_the_pixel_coverage_tiny_skia_gives_an_anti_aliased_rect() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_box(0.5, 1.0, 2.0, 1.25, [1, 2, 3]);
        let edge = |x, y, ink, alpha| {
            format!(
                r##"<rect x="{x}" y="{y}" width="1" height="1" fill="#{ink}" fill-opacity="{alpha}"/>"##
            )
        };
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#010203" shape-rendering="crispEdges">"##.to_owned(),
                edge(0, 1, "010103", "0.5020"),
                r#"<rect x="1" y="1" width="1" height="1"/>"#.to_owned(),
                edge(2, 1, "010103", "0.5020"),
                edge(0, 2, "000000", "0.1255"),
                edge(1, 2, "000202", "0.2510"),
                edge(2, 2, "000000", "0.1255"),
                "</g>".to_owned(),
            ]
            .concat()
        );
    }

    #[test]
    fn a_partial_pixel_shifts_its_ink_so_resvg_premultiplies_to_tiny_skias_scaled_ink() {
        let mut svg = SvgWriter::new(1, 1);
        svg.fill_box(0.0, 0.0, 1.0, 0.5, [255, 128, 0]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#ff8000" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1" fill="#fe8000" fill-opacity="0.4980"/></g>"##
        );
    }

    #[test]
    fn fill_box_clips_to_the_frame_before_it_snaps_edges() {
        let mut svg = SvgWriter::new(4, 4);
        svg.fill_box(-3.5, 3.5, 5.0, 2.0, [0, 0, 0]);
        let row = |x, width, alpha| {
            format!(r#"<rect x="{x}" y="3" width="{width}" height="1" fill-opacity="{alpha}"/>"#)
        };
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#000000" shape-rendering="crispEdges">"##.to_owned(),
                row(0, 1, "0.4980"),
                row(1, 1, "0.2471"),
                "</g>".to_owned(),
            ]
            .concat()
        );
    }

    #[test]
    fn fill_path_writes_each_segment_relative_or_absolute_whichever_is_shorter() {
        let mut path = PathBuilder::new();
        path.move_to(0.5, 1.0);
        path.line_to(2.0, 1.0 / 3.0);
        path.quad_to(3.0, 0.0, 4.0, 1.0);
        path.cubic_to(5.0, 2.0, 6.0, 3.0, 7.0, 4.0);
        path.close();
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_path(&path.finish().unwrap(), [0, 128, 255]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#0080ff"><path d="m.5 1L2 .333Q3 0 4 1c1 1 2 2 3 3z"/></g>"##
        );
    }

    #[test]
    fn a_path_past_the_frame_keeps_shortest_f32_coordinates_for_tiny_skias_clipper() {
        let mut path = PathBuilder::new();
        path.move_to(1.0 / 3.0, -1.0);
        path.line_to(2.0, 1.0);
        path.close();
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_path(&path.finish().unwrap(), [0, 0, 0]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#000000"><path d="m.33333334-1L2 1z"/></g>"##
        );
    }

    fn triangle() -> tiny_skia::Path {
        let mut path = PathBuilder::new();
        path.move_to(0.0, 0.0);
        path.line_to(4.0, 0.0);
        path.line_to(0.0, 3.5);
        path.close();
        path.finish().unwrap()
    }

    #[test]
    fn fill_even_odd_writes_the_path_with_an_evenodd_fill_rule() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_even_odd(&triangle(), [1, 2, 3]);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#010203"><path d="m0 0 4 0L0 3.5z" fill-rule="evenodd"/></g>"##
        );
    }

    #[test]
    fn fill_transformed_writes_its_matrix_and_keeps_shortest_f32_coordinates() {
        let mut path = PathBuilder::new();
        path.move_to(1.0 / 3.0, 0.0);
        path.line_to(4.0, 0.0);
        path.line_to(0.0, 3.5);
        path.close();
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_transformed(
            &path.finish().unwrap(),
            [1, 2, 3],
            Transform::from_row(0.0, 1.0, -1.5, 0.0, 5.0, 1.0 / 3.0),
        );
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#010203"><path d="m.33333334 0L4 0 0 3.5z" transform="matrix(0 1 -1.5 0 5 0.33333334)"/></g>"##
        );
    }

    #[test]
    fn an_identity_fill_transformed_writes_the_plain_snapped_path() {
        let mut path = PathBuilder::new();
        path.move_to(2.7612965, 0.0);
        path.line_to(4.0, 0.0);
        path.line_to(0.0, 3.5);
        path.close();
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_transformed(&path.finish().unwrap(), [1, 2, 3], Transform::identity());
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#010203"><path d="m2.76 0L4 0 0 3.5z"/></g>"##
        );
    }

    #[test]
    fn a_butt_capped_miter_joined_stroke_writes_tiny_skias_miter_limit() {
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(&triangle(), [255, 0, 16], 1.5, Cap::Butt, Join::Miter);
        assert_eq!(
            body(svg.finish()),
            r##"<style>.s0{fill:none;stroke:#ff0010;stroke-width:1.5;stroke-miterlimit:10}</style><g fill="#ff0010"><path d="m0 0 4 0L0 3.5z" class="s0"/></g>"##
        );
    }

    #[test]
    fn strokes_share_a_class_per_paint_and_ride_in_the_open_smooth_group() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_path(&triangle(), [1, 2, 3]);
        svg.stroke(&triangle(), [255, 0, 16], 1.5, Cap::Round, Join::Round);
        svg.stroke(&triangle(), [0, 0, 0], 1.5, Cap::Round, Join::Round);
        svg.stroke(&triangle(), [255, 0, 16], 1.5, Cap::Round, Join::Round);
        svg.fill_rect(0, 0, 1, 1, [1, 2, 3]);
        svg.stroke(&triangle(), [0, 0, 0], 1.5, Cap::Round, Join::Round);
        let round = "stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round";
        let path = |class| format!(r#"<path d="m0 0 4 0L0 3.5z" class="s{class}"/>"#);
        assert_eq!(
            body(svg.finish()),
            [
                format!("<style>.s0{{fill:none;stroke:#ff0010;{round}}}.s1{{fill:none;stroke:#000000;{round}}}</style>"),
                r##"<g fill="#010203"><path d="m0 0 4 0L0 3.5z"/>"##.to_owned(),
                path(0),
                path(1),
                path(0),
                r##"</g><g fill="#010203" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1"/></g>"##.to_owned(),
                r##"<g fill="#000000">"##.to_owned(),
                path(1),
                "</g>".to_owned(),
            ]
            .concat()
        );
    }

    #[test]
    fn a_round_capped_round_joined_stroke_names_both() {
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(&triangle(), [0, 0, 0], 0.25, Cap::Round, Join::Round);
        assert_eq!(
            body(svg.finish()),
            r##"<style>.s0{fill:none;stroke:#000000;stroke-width:0.25;stroke-linecap:round;stroke-linejoin:round}</style><g fill="#000000"><path d="m0 0 4 0L0 3.5z" class="s0"/></g>"##
        );
    }

    #[test]
    fn a_stroke_keeps_shortest_f32_coordinates_for_tiny_skias_stroker() {
        let mut path = PathBuilder::new();
        path.move_to(1.0 / 3.0, 1.0);
        path.line_to(2.0, 1.0);
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(
            &path.finish().unwrap(),
            [0, 0, 0],
            1.0,
            Cap::Butt,
            Join::Round,
        );
        assert!(body(svg.finish()).contains(r#"d="m.33333334 1L2 1""#));
    }

    #[test]
    fn fill_rect_linear_writes_a_user_space_gradient_and_a_crisp_rect_outside_any_ink_group() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        svg.fill_rect_linear(
            1,
            2,
            3,
            4,
            (1.0, 2.0),
            (4.0, 1.0 / 3.0),
            &[(0.0, [255, 0, 16]), (0.25, [0, 0, 0])],
        );
        svg.fill_rect_linear(0, 0, 1, 1, (0.0, 0.0), (1.0, 0.0), &[(1.0, [1, 2, 3])]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1"/></g>"##,
                r#"<linearGradient id="r0" gradientUnits="userSpaceOnUse" x1="1" y1="2" x2="4" y2="0.33333334">"#,
                r##"<stop offset="0" stop-color="#ff0010"/><stop offset="0.25" stop-color="#000000"/></linearGradient>"##,
                r#"<rect x="1" y="2" width="3" height="4" fill="url(#r0)" shape-rendering="crispEdges"/>"#,
                r#"<linearGradient id="r1" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="1" y2="0">"#,
                r##"<stop offset="1" stop-color="#010203"/></linearGradient>"##,
                r#"<rect x="0" y="0" width="1" height="1" fill="url(#r1)" shape-rendering="crispEdges"/>"#,
            ]
            .concat()
        );
    }

    #[test]
    fn under_an_alpha_each_fill_and_stroke_writes_its_own_paint_opacity() {
        let mut svg = SvgWriter::new(9, 9);
        svg.set_alpha(0.85);
        svg.fill([0, 0, 0]);
        svg.fill_rect(1, 2, 3, 4, [0, 0, 0]);
        svg.fill_path(&triangle(), [0, 0, 0]);
        svg.stroke(&triangle(), [0, 0, 0], 1.0, Cap::Butt, Join::Miter);
        svg.set_alpha(1.0);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<style>.s0{fill:none;stroke:#000000;stroke-width:1;stroke-opacity:0.8500;stroke-miterlimit:10}</style>"##,
                r##"<g fill="#000000"><rect width="9" height="9" fill-opacity="0.8500"/></g>"##,
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="1" y="2" width="3" height="4" fill-opacity="0.8500"/></g>"##,
                r##"<g fill="#000000"><path d="m0 0 4 0L0 3.5z" fill-opacity="0.8500"/>"##,
                r#"<path d="m0 0 4 0L0 3.5z" class="s0"/></g>"#,
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn under_an_alpha_an_ink_shifts_so_resvgs_u8_opacity_premultiplies_to_tiny_skias() {
        let mut svg = SvgWriter::new(9, 9);
        svg.set_alpha(0.39);
        svg.stroke(&triangle(), [27, 27, 27], 1.0, Cap::Butt, Join::Miter);
        svg.fill_rect(0, 0, 1, 1, [27, 27, 27]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<style>.s0{fill:none;stroke:#1c1c1c;stroke-width:1;stroke-opacity:0.3900;stroke-miterlimit:10}</style>"##,
                r##"<g fill="#1c1c1c"><path d="m0 0 4 0L0 3.5z" class="s0"/></g>"##,
                r##"<g fill="#1c1c1c" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1" fill-opacity="0.3900"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn a_faded_hairline_picks_the_opacity_whose_u8_tiny_skia_scales_by_its_width_to_its_alpha() {
        let mut svg = SvgWriter::new(9, 9);
        svg.set_alpha(0.81);
        svg.stroke(&triangle(), [217, 212, 199], 0.594, Cap::Butt, Join::Miter);
        assert_eq!(
            body(svg.finish()),
            r##"<style>.s0{fill:none;stroke:#d9d5c6;stroke-width:0.594;stroke-opacity:0.8078;stroke-miterlimit:10}</style><g fill="#d9d5c6"><path d="m0 0 4 0L0 3.5z" class="s0"/></g>"##
        );
    }

    #[test]
    fn under_an_alpha_a_box_scales_its_pixel_coverage_by_the_alpha() {
        let mut svg = SvgWriter::new(2, 1);
        svg.set_alpha(0.5);
        svg.fill_box(0.0, 0.0, 1.5, 1.0, [255, 255, 255]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#ffffff" shape-rendering="crispEdges">"##,
                r#"<rect x="0" y="0" width="1" height="1" fill-opacity="0.5000"/>"#,
                r##"<rect x="1" y="0" width="1" height="1" fill="#fefefe" fill-opacity="0.2510"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn under_multiply_each_element_and_each_stroked_subpath_sits_in_its_own_blend_group() {
        let mut svg = SvgWriter::new(9, 9);
        svg.set_multiply(true);
        svg.fill_rect(0, 0, 1, 1, [1, 2, 3]);
        svg.fill_rect(1, 0, 1, 1, [1, 2, 3]);
        svg.stroke(&triangle(), [1, 2, 3], 1.0, Cap::Butt, Join::Miter);
        svg.fill_box(0.0, 0.0, 1.0, 0.5, [255, 255, 255]);
        let mut cross = PathBuilder::new();
        cross.move_to(0.0, 1.5);
        cross.line_to(4.0, 1.5);
        cross.move_to(1.5, 0.0);
        cross.line_to(1.5, 4.0);
        svg.stroke(
            &cross.finish().unwrap(),
            [0, 0, 0],
            1.0,
            Cap::Butt,
            Join::Miter,
        );
        svg.set_multiply(false);
        svg.fill_rect(2, 0, 1, 1, [1, 2, 3]);
        let blend = r#"<g style="mix-blend-mode:multiply">"#;
        assert_eq!(
            body(svg.finish()),
            [
                r#"<style>.s0{fill:none;stroke:#010203;stroke-width:1;stroke-miterlimit:10}"#,
                r#".s1{fill:none;stroke:#000000;stroke-width:1;stroke-miterlimit:10}</style>"#,
                r##"<g fill="#010203" shape-rendering="crispEdges">"##,
                blend,
                r#"<rect x="0" y="0" width="1" height="1"/></g>"#,
                blend,
                r#"<rect x="1" y="0" width="1" height="1"/></g></g>"#,
                r##"<g fill="#010203">"##,
                blend,
                r#"<path d="m0 0 4 0L0 3.5z" class="s0"/></g></g>"#,
                r##"<g fill="#ffffff" shape-rendering="crispEdges">"##,
                blend,
                r##"<rect x="0" y="0" width="1" height="1" fill="#fefefe" fill-opacity="0.4980"/></g></g>"##,
                r##"<g fill="#000000">"##,
                blend,
                r#"<path d="m0 1.5 4 0" class="s1"/></g>"#,
                blend,
                r#"<path d="m1.5 0 0 4" class="s1"/></g></g>"#,
                r##"<g fill="#010203" shape-rendering="crispEdges"><rect x="2" y="0" width="1" height="1"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn each_clipped_element_sits_in_its_own_frame_sized_clip_group_as_tiny_skia_masks_each_draw() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        svg.push_clip(&triangle(), Transform::identity());
        svg.fill_rect(1, 0, 1, 1, [0, 0, 0]);
        svg.fill_path(&triangle(), [0, 0, 0]);
        svg.pop_clip();
        svg.fill_rect(2, 0, 1, 1, [0, 0, 0]);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="0" y="0" width="1" height="1"/></g>"##,
                r#"<filter id="frame" filterUnits="userSpaceOnUse" x="0" y="0" width="9" height="9" color-interpolation-filters="sRGB"><feOffset/></filter>"#,
                r#"<clipPath id="c0"><path d="m0 0 4 0L0 3.5z"/></clipPath>"#,
                r##"<g fill="#000000" shape-rendering="crispEdges"><g clip-path="url(#c0)" filter="url(#frame)"><rect x="1" y="0" width="1" height="1"/></g></g>"##,
                r##"<g fill="#000000"><g clip-path="url(#c0)" filter="url(#frame)"><path d="m0 0 4 0L0 3.5z"/></g></g>"##,
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="2" y="0" width="1" height="1"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn a_clipped_hairline_writes_each_subpath_in_its_own_clip_group_and_a_wider_stroke_one() {
        let mut cross = PathBuilder::new();
        cross.move_to(0.0, 1.5);
        cross.line_to(4.0, 1.5);
        cross.move_to(1.5, 0.0);
        cross.line_to(1.5, 4.0);
        let cross = cross.finish().unwrap();
        let mut svg = SvgWriter::new(9, 9);
        svg.push_clip(&triangle(), Transform::identity());
        svg.stroke(&cross, [0, 0, 0], 1.0, Cap::Butt, Join::Miter);
        svg.stroke(&cross, [0, 0, 0], 1.5, Cap::Butt, Join::Miter);
        svg.pop_clip();
        let clip = r#"<g clip-path="url(#c0)" filter="url(#frame)">"#;
        let stroke = |d: &str, class: u8| format!(r#"<path d="{d}" class="s{class}"/></g>"#);
        assert!(
            body(svg.finish()).ends_with(
                &[
                    r##"<g fill="#000000">"##.to_owned(),
                    clip.to_owned(),
                    stroke("m0 1.5 4 0", 0),
                    clip.to_owned(),
                    stroke("m1.5 0 0 4", 0),
                    clip.to_owned(),
                    stroke("m0 1.5 4 0M1.5 0l0 4", 1),
                    "</g>".to_owned(),
                ]
                .concat()
            )
        );
    }

    #[test]
    fn a_nested_transformed_clip_is_clipped_by_the_outer_and_writes_its_matrix_on_its_path() {
        let mut svg = SvgWriter::new(9, 9);
        svg.push_clip(&triangle(), Transform::identity());
        svg.push_clip(
            &triangle(),
            Transform::from_row(0.0, 1.0, -1.5, 0.0, 5.0, 1.0 / 3.0),
        );
        svg.fill([1, 2, 3]);
        svg.pop_clip();
        svg.fill([1, 2, 3]);
        svg.pop_clip();
        assert_eq!(
            body(svg.finish()),
            [
                r#"<filter id="frame" filterUnits="userSpaceOnUse" x="0" y="0" width="9" height="9" color-interpolation-filters="sRGB"><feOffset/></filter>"#,
                r#"<clipPath id="c0"><path d="m0 0 4 0L0 3.5z"/></clipPath>"#,
                r#"<clipPath id="c1" clip-path="url(#c0)"><path d="m0 0 4 0L0 3.5z" transform="matrix(0 1 -1.5 0 5 0.33333334)"/></clipPath>"#,
                r##"<g fill="#010203"><g clip-path="url(#c1)" filter="url(#frame)"><rect width="9" height="9"/></g>"##,
                r#"<g clip-path="url(#c0)" filter="url(#frame)"><rect width="9" height="9"/></g></g>"#,
            ]
            .concat()
        );
    }

    #[test]
    fn draw_nearest_writes_a_crisp_rect_per_cell_merges_ink_runs_and_gives_edge_ties_left() {
        let (red, blue) = ([255, 0, 0, 255], [0, 0, 255, 255]);
        let mut svg = SvgWriter::new(10, 2);
        svg.draw_nearest(&[red, blue, blue, red].concat(), 4, 1);
        assert_eq!(
            body(svg.finish()),
            [
                r##"<g fill="#ff0000" shape-rendering="crispEdges"><rect x="0" y="0" width="3" height="2"/></g>"##,
                r##"<g fill="#0000ff" shape-rendering="crispEdges"><rect x="3" y="0" width="5" height="2"/></g>"##,
                r##"<g fill="#ff0000" shape-rendering="crispEdges"><rect x="8" y="0" width="2" height="2"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    #[should_panic(expected = "a nearest buffer is opaque")]
    fn draw_nearest_refuses_a_translucent_buffer() {
        SvgWriter::new(1, 1).draw_nearest(&[0, 0, 0, 254], 1, 1);
    }

    #[test]
    #[should_panic(expected = "pixel writes are opaque and source-over")]
    fn draw_nearest_refuses_an_alpha() {
        let mut svg = SvgWriter::new(1, 1);
        svg.set_alpha(0.5);
        svg.draw_nearest(&[0, 0, 0, 255], 1, 1);
    }

    #[test]
    fn pixels_flush_as_one_png_image_over_their_bounds_before_the_next_element() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        svg.set_pixel(3, 2, [255, 0, 0]);
        svg.set_pixel(1, 4, [0, 255, 0]);
        svg.fill_rect(0, 0, 1, 1, [0, 0, 0]);
        let svg = svg.finish();
        assert!(svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="9""#
        ));
        let body = body(svg);
        let image = r#"</g><image x="1" y="2" width="3" height="3" image-rendering="optimizeSpeed" xlink:href="data:image/png;base64,iVBORw0KGgo"#;
        let at = body.find(image).expect(&body);
        assert!(body[at..].contains(r##"/><g fill="#000000""##), "{body}");
        assert_eq!(body.matches("<image").count(), 1);
    }

    #[test]
    fn pixels_last_flush_when_the_writer_finishes() {
        let mut svg = SvgWriter::new(2, 2);
        svg.set_pixel(1, 1, [1, 2, 3]);
        assert!(body(svg.finish()).starts_with(r#"<image x="1" y="1" width="1" height="1""#));
    }

    #[test]
    fn base64_pads_each_tail_to_four_characters() {
        let encoded =
            ["", "f", "fo", "foo", "foob", "fooba", "foobar"].map(|s| super::base64(s.as_bytes()));
        assert_eq!(
            encoded,
            [
                "", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"
            ]
        );
        assert_eq!(super::base64(&[0xfb, 0xff]), "+/8=");
    }

    #[test]
    fn a_fill_or_stroke_that_misses_the_frame_writes_nothing() {
        let mut far = PathBuilder::new();
        far.move_to(-4.0, 1.0);
        far.line_to(-2.0, 1.0);
        far.line_to(-2.0, 3.0);
        far.close();
        let far = far.finish().unwrap();
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_path(&far, [0, 0, 0]);
        svg.fill_transformed(&far, [0, 0, 0], Transform::from_translate(0.0, 20.0));
        svg.stroke(&far, [0, 0, 0], 1.0, Cap::Round, Join::Round);
        assert_eq!(body(svg.finish()), "");
    }

    #[test]
    fn a_stroke_whose_width_reaches_the_frame_is_kept() {
        let mut near = PathBuilder::new();
        near.move_to(-0.5, 1.0);
        near.line_to(-0.5, 3.0);
        let near = near.finish().unwrap();
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(&near, [0, 0, 0], 2.0, Cap::Butt, Join::Round);
        svg.stroke(&near, [0, 0, 0], 1.0, Cap::Butt, Join::Miter);
        assert!(body(svg.finish()).contains("<path"));
    }

    #[test]
    fn a_stroke_around_the_frame_whose_every_segment_misses_it_writes_nothing() {
        let mut corner = PathBuilder::new();
        corner.move_to(-10.0, -10.0);
        corner.line_to(20.0, -10.0);
        corner.line_to(20.0, 20.0);
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(
            &corner.finish().unwrap(),
            [0, 0, 0],
            1.0,
            Cap::Round,
            Join::Round,
        );
        assert_eq!(body(svg.finish()), "");
    }

    #[test]
    fn a_stroke_drops_runs_past_the_frame_but_keeps_one_segment_beyond_each_seen_one() {
        let mut path = PathBuilder::new();
        path.move_to(1.0, 1.0);
        for (x, y) in [
            (5.0, 1.0),
            (20.0, 1.0),
            (30.0, 1.0),
            (40.0, 1.0),
            (30.0, 2.0),
            (20.0, 2.0),
            (5.0, 2.0),
        ] {
            path.line_to(x, y);
        }
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(
            &path.finish().unwrap(),
            [0, 0, 0],
            1.0,
            Cap::Round,
            Join::Round,
        );
        assert!(body(svg.finish()).contains(r#"d="m1 1 4 0 15 0 10 0m0 1L20 2 5 2""#));
    }

    #[test]
    fn a_delta_takes_the_fewest_digits_whose_sum_keeps_its_256th_of_a_pixel() {
        let text = |base, v| super::relative(base, v, super::on_edge).unwrap().0;
        assert_eq!(text(0.0, 2.7612965), "2.76");
        assert_eq!(text(0.0, 0.95035976), ".95");
        assert_eq!(text(0.0, 0.0039), "0");
        assert_eq!(text(10.0, 12.0), "2");
        assert_eq!(text(12.5, 12.0), "-.5");
    }

    #[test]
    fn an_exact_delta_sums_in_f64_back_to_its_f32_as_svgtypes_does() {
        let (text, sum) = super::relative(1.0, 1.0 / 3.0, super::exact).unwrap();
        assert_eq!((1.0 + text.parse::<f64>().unwrap()) as f32, 1.0 / 3.0);
        assert_eq!(sum as f32, 1.0 / 3.0);
    }

    #[test]
    fn a_point_past_f64_reach_of_the_pen_is_written_absolute() {
        assert_eq!(
            super::relative(-11.394995999999988, 1.5347724e-14, super::exact),
            None
        );
        let mut path = PathBuilder::new();
        path.move_to(-11.394996, 1.0);
        path.line_to(1.5347724e-14, 1.0);
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(
            &path.finish().unwrap(),
            [0, 0, 0],
            1.0,
            Cap::Butt,
            Join::Round,
        );
        assert!(body(svg.finish()).contains(r#"d="m-11.394996 1L.000000000000015347723 1""#));
    }
}
