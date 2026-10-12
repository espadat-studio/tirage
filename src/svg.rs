use std::fmt::Write;

use tiny_skia::{Path, PathSegment};

pub(crate) struct SvgWriter {
    width: u32,
    height: u32,
    markup: String,
}

impl SvgWriter {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            markup: format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
            ),
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn fill(&mut self, ink: [u8; 3]) {
        let (width, height, [r, g, b]) = (self.width, self.height, ink);
        write!(
            self.markup,
            r##"<rect width="{width}" height="{height}" fill="#{r:02x}{g:02x}{b:02x}"/>"##
        )
        .expect("a String takes any write");
    }

    pub(crate) fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, ink: [u8; 3]) {
        let [r, g, b] = ink;
        write!(
            self.markup,
            r##"<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="#{r:02x}{g:02x}{b:02x}" shape-rendering="crispEdges"/>"##
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
        let opacity = alpha as f32 / 255.0;
        let [r, g, b] = ink.map(|c| {
            let scaled = (i32::from(c) * alpha + 128) >> 8;
            (0..=255u8)
                .find(|&v| (f32::from(v) / 255.0 * opacity * 255.0 + 0.5) as i32 == scaled)
                .expect("ink 255 at this opacity reaches past any scaled ink")
        });
        write!(
            self.markup,
            r##"<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="#{r:02x}{g:02x}{b:02x}" fill-opacity="{opacity}" shape-rendering="crispEdges"/>"##
        )
        .expect("a String takes any write");
    }

    pub(crate) fn fill_path(&mut self, path: &Path, ink: [u8; 3]) {
        self.markup.push_str(r#"<path d=""#);
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => write!(self.markup, "M{} {}", p.x, p.y),
                PathSegment::LineTo(p) => write!(self.markup, "L{} {}", p.x, p.y),
                PathSegment::QuadTo(c, p) => {
                    write!(self.markup, "Q{} {} {} {}", c.x, c.y, p.x, p.y)
                }
                PathSegment::CubicTo(a, b, p) => write!(
                    self.markup,
                    "C{} {} {} {} {} {}",
                    a.x, a.y, b.x, b.y, p.x, p.y
                ),
                PathSegment::Close => write!(self.markup, "Z"),
            }
            .expect("a String takes any write");
        }
        let [r, g, b] = ink;
        write!(self.markup, r##"" fill="#{r:02x}{g:02x}{b:02x}"/>"##)
            .expect("a String takes any write");
    }

    pub(crate) fn finish(self) -> String {
        self.markup + "</svg>\n"
    }
}

#[cfg(test)]
mod tests {
    use tiny_skia::PathBuilder;

    use super::SvgWriter;

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
            r##"<rect width="3" height="2" fill="#0aff00"/>"##
        );
    }

    #[test]
    fn fill_rect_writes_a_crisp_edged_rect_at_its_pixel_corner() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_rect(-1, 4, 2, 3, [255, 209, 102]);
        assert_eq!(
            body(svg.finish()),
            r##"<rect x="-1" y="4" width="2" height="3" fill="#ffd166" shape-rendering="crispEdges"/>"##
        );
    }

    #[test]
    fn fill_box_writes_the_pixel_coverage_tiny_skia_gives_an_anti_aliased_rect() {
        let mut svg = SvgWriter::new(9, 9);
        svg.fill_box(0.5, 1.0, 2.0, 1.25, [1, 2, 3]);
        let edge = |x, y, ink, alpha| {
            format!(
                r##"<rect x="{x}" y="{y}" width="1" height="1" fill="#{ink}" fill-opacity="{alpha}" shape-rendering="crispEdges"/>"##
            )
        };
        let solid = r##"<rect x="1" y="1" width="1" height="1" fill="#010203" shape-rendering="crispEdges"/>"##;
        assert_eq!(
            body(svg.finish()),
            [
                edge(0, 1, "010103", "0.5019608"),
                solid.to_owned(),
                edge(2, 1, "010103", "0.5019608"),
                edge(0, 2, "000000", "0.1254902"),
                edge(1, 2, "000202", "0.2509804"),
                edge(2, 2, "000000", "0.1254902"),
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
            r##"<rect x="0" y="0" width="1" height="1" fill="#fe8000" fill-opacity="0.49803922" shape-rendering="crispEdges"/>"##
        );
    }

    #[test]
    fn fill_box_clips_to_the_frame_before_it_snaps_edges() {
        let mut svg = SvgWriter::new(4, 4);
        svg.fill_box(-3.5, 3.5, 5.0, 2.0, [0, 0, 0]);
        let row = |x, width, alpha| {
            format!(
                r##"<rect x="{x}" y="3" width="{width}" height="1" fill="#000000" fill-opacity="{alpha}" shape-rendering="crispEdges"/>"##
            )
        };
        assert_eq!(
            body(svg.finish()),
            [row(0, 1, "0.49803922"), row(1, 1, "0.24705882")].concat()
        );
    }

    #[test]
    fn fill_path_writes_each_segment_as_path_data_at_shortest_f32_coordinates() {
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
            r##"<path d="M0.5 1L2 0.33333334Q3 0 4 1C5 2 6 3 7 4Z" fill="#0080ff"/>"##
        );
    }
}
