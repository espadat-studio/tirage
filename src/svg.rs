use std::fmt::Write;

use tiny_skia::{Path, PathBuilder, PathSegment, Transform};

use crate::surface::{Cap, Join};

pub(crate) struct SvgWriter {
    width: u32,
    height: u32,
    markup: String,
    group: Option<([u8; 3], bool)>,
    gradients: u32,
    alpha: f32,
    multiply: bool,
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
            alpha: 1.0,
            multiply: false,
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
        if self.multiply {
            self.markup
                .push_str(r#"<g style="mix-blend-mode:multiply">"#);
        }
    }

    fn close(&mut self) {
        self.markup.push_str("/>");
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
        if self.group.take().is_some() {
            self.markup.push_str("</g>");
        }
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

    pub(crate) fn fill_path(&mut self, path: &Path, ink: [u8; 3]) {
        self.open(self.faded(ink), false);
        self.open_path(path, true);
        self.markup.push('"');
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn fill_even_odd(&mut self, path: &Path, ink: [u8; 3]) {
        self.open(self.faded(ink), false);
        self.open_path(path, true);
        self.markup.push_str(r#"" fill-rule="evenodd""#);
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn fill_transformed(&mut self, path: &Path, ink: [u8; 3], transform: Transform) {
        self.open(self.faded(ink), false);
        self.open_path(path, false);
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
        self.opacity("fill", self.alpha);
        self.close();
    }

    pub(crate) fn stroke(&mut self, path: &Path, ink: [u8; 3], width: f32, cap: Cap, join: Join) {
        let (ink, opacity) = self.faded_stroke(ink, width);
        let parts = if self.multiply {
            subpaths(path)
        } else {
            vec![path.clone()]
        };
        for part in &parts {
            self.open(ink, false);
            self.open_path(part, false);
            let [r, g, b] = ink;
            write!(
                self.markup,
                r##"" fill="none" stroke="#{r:02x}{g:02x}{b:02x}" stroke-width="{width}""##
            )
            .expect("a String takes any write");
            self.opacity("stroke", opacity);
            if let Cap::Round = cap {
                self.markup.push_str(r#" stroke-linecap="round""#);
            }
            self.markup.push_str(match join {
                Join::Round => r#" stroke-linejoin="round""#,
                Join::Miter => r#" stroke-miterlimit="10""#,
            });
            self.close();
        }
    }

    fn open_path(&mut self, path: &Path, snap: bool) {
        self.markup.push_str(r#"<path d=""#);
        let bounds = path.bounds();
        let unclipped = snap
            && bounds.left() >= 0.0
            && bounds.top() >= 0.0
            && bounds.right() <= self.width as f32
            && bounds.bottom() <= self.height as f32;
        let number: fn(f32) -> String = if unclipped {
            coordinate
        } else {
            |v| v.to_string()
        };
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => {
                    write!(self.markup, "M{} {}", number(p.x), number(p.y))
                }
                PathSegment::LineTo(p) => {
                    write!(self.markup, "L{} {}", number(p.x), number(p.y))
                }
                PathSegment::QuadTo(c, p) => {
                    write!(
                        self.markup,
                        "Q{} {} {} {}",
                        number(c.x),
                        number(c.y),
                        number(p.x),
                        number(p.y)
                    )
                }
                PathSegment::CubicTo(a, b, p) => write!(
                    self.markup,
                    "C{} {} {} {} {} {}",
                    number(a.x),
                    number(a.y),
                    number(b.x),
                    number(b.y),
                    number(p.x),
                    number(p.y)
                ),
                PathSegment::Close => write!(self.markup, "Z"),
            }
            .expect("a String takes any write");
        }
    }

    pub(crate) fn finish(mut self) -> String {
        if self.group.is_some() {
            self.markup.push_str("</g>");
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

fn coordinate(v: f32) -> String {
    let edge = |v: f32| (v * 256.0) as i32;
    (0..)
        .flat_map(|digits| {
            let scale = 10f64.powi(digits);
            let scaled = f64::from(v) * scale;
            [scaled.floor(), scaled.ceil()].map(|d| format!("{:.*}", digits as usize, d / scale))
        })
        .find(|text| edge(text.parse::<f64>().expect("a formatted f64 parses") as f32) == edge(v))
        .expect("v's own digits land on v's edge")
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
    fn fill_path_writes_each_segment_at_the_shortest_decimal_on_tiny_skias_edge() {
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
            r##"<g fill="#0080ff"><path d="M0.5 1L2 0.333Q3 0 4 1C5 2 6 3 7 4Z"/></g>"##
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
            r##"<g fill="#000000"><path d="M0.33333334 -1L2 1Z"/></g>"##
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
            r##"<g fill="#010203"><path d="M0 0L4 0L0 3.5Z" fill-rule="evenodd"/></g>"##
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
            r##"<g fill="#010203"><path d="M0.33333334 0L4 0L0 3.5Z" transform="matrix(0 1 -1.5 0 5 0.33333334)"/></g>"##
        );
    }

    #[test]
    fn a_butt_capped_miter_joined_stroke_writes_tiny_skias_miter_limit() {
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(&triangle(), [255, 0, 16], 1.5, Cap::Butt, Join::Miter);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#ff0010"><path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#ff0010" stroke-width="1.5" stroke-miterlimit="10"/></g>"##
        );
    }

    #[test]
    fn a_round_capped_round_joined_stroke_names_both() {
        let mut svg = SvgWriter::new(9, 9);
        svg.stroke(&triangle(), [0, 0, 0], 0.25, Cap::Round, Join::Round);
        assert_eq!(
            body(svg.finish()),
            r##"<g fill="#000000"><path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#000000" stroke-width="0.25" stroke-linecap="round" stroke-linejoin="round"/></g>"##
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
        assert!(body(svg.finish()).contains(r#"d="M0.33333334 1L2 1""#));
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
                r##"<g fill="#000000"><rect width="9" height="9" fill-opacity="0.8500"/></g>"##,
                r##"<g fill="#000000" shape-rendering="crispEdges"><rect x="1" y="2" width="3" height="4" fill-opacity="0.8500"/></g>"##,
                r##"<g fill="#000000"><path d="M0 0L4 0L0 3.5Z" fill-opacity="0.8500"/>"##,
                r##"<path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#000000" stroke-width="1" stroke-opacity="0.8500" stroke-miterlimit="10"/></g>"##,
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
                r##"<g fill="#1c1c1c"><path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#1c1c1c" stroke-width="1" stroke-opacity="0.3900" stroke-miterlimit="10"/></g>"##,
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
            r##"<g fill="#d9d5c6"><path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#d9d5c6" stroke-width="0.594" stroke-opacity="0.8078" stroke-miterlimit="10"/></g>"##
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
                r##"<g fill="#010203" shape-rendering="crispEdges">"##,
                blend,
                r#"<rect x="0" y="0" width="1" height="1"/></g>"#,
                blend,
                r#"<rect x="1" y="0" width="1" height="1"/></g></g>"#,
                r##"<g fill="#010203">"##,
                blend,
                r##"<path d="M0 0L4 0L0 3.5Z" fill="none" stroke="#010203" stroke-width="1" stroke-miterlimit="10"/></g></g>"##,
                r##"<g fill="#ffffff" shape-rendering="crispEdges">"##,
                blend,
                r##"<rect x="0" y="0" width="1" height="1" fill="#fefefe" fill-opacity="0.4980"/></g></g>"##,
                r##"<g fill="#000000">"##,
                blend,
                r##"<path d="M0 1.5L4 1.5" fill="none" stroke="#000000" stroke-width="1" stroke-miterlimit="10"/></g>"##,
                blend,
                r##"<path d="M1.5 0L1.5 4" fill="none" stroke="#000000" stroke-width="1" stroke-miterlimit="10"/></g></g>"##,
                r##"<g fill="#010203" shape-rendering="crispEdges"><rect x="2" y="0" width="1" height="1"/></g>"##,
            ]
            .concat()
        );
    }

    #[test]
    fn a_coordinate_takes_the_fewest_digits_that_keep_its_256th_of_a_pixel() {
        assert_eq!(super::coordinate(2.7612965), "2.76");
        assert_eq!(super::coordinate(0.95035976), "0.95");
        assert_eq!(super::coordinate(12.0), "12");
        assert_eq!(super::coordinate(0.0039), "0");
        assert_eq!(super::coordinate(0.004), "0.004");
    }
}
