#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl Image {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn to_png(&self) -> Vec<u8> {
        let mut png = Vec::new();
        let mut encoder = png::Encoder::new(&mut png, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("a Vec takes any header");
        writer
            .write_image_data(&self.rgba)
            .expect("rgba holds width * height pixels");
        writer.finish().expect("a Vec takes any image");
        png
    }
}
