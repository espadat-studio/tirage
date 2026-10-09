use std::fmt;
use std::io::Cursor;

use bytes::Bytes;
use mp4::{AvcConfig, FourCC, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer, TrackConfig};
use openh264::OpenH264API;
use openh264::encoder::{
    BitRate, Encoder, EncoderConfig, FrameRate, FrameType, QpRange, RateControlMode,
};
use openh264::formats::{RgbaSliceU8, YUVBuffer};
use tirage::{Frame, Recipe, render};

pub const WIDTH: u32 = 720;
pub const HEIGHT: u32 = 1280;

pub const OPENH264: &str = "0.9.8";

pub const MAX_BYTES: usize = 1_500_000;

const BUDGET: u32 = 1_200_000;
const QP: (u8, u8) = (12, 51);

const SPS: u8 = 7;
const PPS: u8 = 8;

pub fn settings() -> String {
    format!(
        "tirage-encode {} openh264 {OPENH264} h264 {WIDTH}x{HEIGHT} rc budget {BUDGET} bytes per Loop qp {}..={} no-skip",
        env!("CARGO_PKG_VERSION"),
        QP.0,
        QP.1
    )
}

#[derive(Debug)]
pub enum Error {
    H264(openh264::Error),
    Mp4(mp4::Error),
    OverCap { bytes: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::H264(error) => write!(f, "h264: {error}"),
            Self::Mp4(error) => write!(f, "mp4: {error}"),
            Self::OverCap { bytes } => {
                write!(f, "loop is {bytes} bytes, over the {MAX_BYTES} byte cap")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<openh264::Error> for Error {
    fn from(error: openh264::Error) -> Self {
        Self::H264(error)
    }
}

impl From<mp4::Error> for Error {
    fn from(error: mp4::Error) -> Self {
        Self::Mp4(error)
    }
}

pub fn encode(recipe: &Recipe) -> Result<Vec<u8>, Error> {
    let tool = recipe.tool();
    let fps = tool.fps();
    let bitrate = u64::from(BUDGET) * 8 * u64::from(fps) / u64::from(tool.frames());
    let config = EncoderConfig::new()
        .bitrate(BitRate::from_bps(bitrate as u32))
        .qp(QpRange::new(QP.0, QP.1))
        .max_frame_rate(FrameRate::from_hz(fps as f32))
        .rate_control_mode(RateControlMode::Bitrate)
        .skip_frames(false);
    let mut encoder = Encoder::with_api_config(OpenH264API::from_source(), config)?;

    let mut sps = Vec::new();
    let mut pps = Vec::new();
    let mut samples = Vec::new();
    for t in 0..tool.frames() {
        let frame =
            Frame::new(recipe, WIDTH, HEIGHT, t).expect("720x1280 inside the Loop is valid");
        let image = render(recipe, &frame);
        let rgba = RgbaSliceU8::new(image.rgba(), (WIDTH as usize, HEIGHT as usize));
        let stream = encoder.encode(&YUVBuffer::from_rgb_source(rgba))?;
        let is_sync = matches!(stream.frame_type(), FrameType::IDR);
        let mut sample = Vec::new();
        for layer in (0..stream.num_layers()).filter_map(|i| stream.layer(i)) {
            for nal in (0..layer.nal_count()).filter_map(|j| layer.nal_unit(j)) {
                let nal = strip_start_code(nal);
                match nal[0] & 0x1f {
                    SPS => sps = nal.to_vec(),
                    PPS => pps = nal.to_vec(),
                    _ => {
                        sample.extend((nal.len() as u32).to_be_bytes());
                        sample.extend(nal);
                    }
                }
            }
        }
        samples.push(Mp4Sample {
            start_time: u64::from(t),
            duration: 1,
            rendering_offset: 0,
            is_sync,
            bytes: Bytes::from(sample),
        });
    }

    let mut writer = Mp4Writer::write_start(
        Cursor::new(Vec::new()),
        &Mp4Config {
            major_brand: FourCC::from(*b"isom"),
            minor_version: 512,
            compatible_brands: [*b"isom", *b"iso2", *b"avc1", *b"mp41"]
                .map(FourCC::from)
                .to_vec(),
            timescale: 1000,
        },
    )?;
    writer.add_track(&TrackConfig {
        timescale: fps,
        ..TrackConfig::from(MediaConfig::AvcConfig(AvcConfig {
            width: WIDTH as u16,
            height: HEIGHT as u16,
            seq_param_set: sps,
            pic_param_set: pps,
        }))
    })?;
    for sample in &samples {
        writer.write_sample(1, sample)?;
    }
    writer.write_end()?;
    let mp4 = writer.into_writer().into_inner();
    if mp4.len() >= MAX_BYTES {
        return Err(Error::OverCap { bytes: mp4.len() });
    }
    Ok(mp4)
}

fn strip_start_code(nal: &[u8]) -> &[u8] {
    let start = nal
        .windows(3)
        .position(|window| window == [0, 0, 1])
        .expect("openh264 prefixes every NAL with a start code");
    &nal[start + 3..]
}
