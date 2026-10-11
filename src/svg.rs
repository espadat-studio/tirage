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
}
