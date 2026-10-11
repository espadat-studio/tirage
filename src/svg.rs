use std::fmt::Write;

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

    pub(crate) fn finish(self) -> String {
        self.markup + "</svg>\n"
    }
}

#[cfg(test)]
mod tests {
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
}
