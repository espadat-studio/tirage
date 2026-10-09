use std::io::Cursor;

use mp4::{MediaType, Mp4Reader};
use tirage::{Tool, ToolPin, derive};
use tirage_encode::{HEIGHT, MAX_BYTES, OPENH264, WIDTH, encode, settings};

const SEEDS: [u64; 3] = [1, 2, 3];

fn read(mp4: &[u8]) -> Mp4Reader<Cursor<&[u8]>> {
    Mp4Reader::read_header(Cursor::new(mp4), mp4.len() as u64).unwrap()
}

#[test]
fn sonar_loop_is_h264_at_720x1280_with_the_tools_frames_and_fps() {
    let mp4 = encode(&derive(1, ToolPin::Tool(Tool::Sonar))).unwrap();
    let reader = read(&mp4);
    let tracks: Vec<_> = reader.tracks().values().collect();
    assert_eq!(tracks.len(), 1);
    let track = tracks[0];
    assert_eq!(track.media_type().unwrap(), MediaType::H264);
    assert_eq!(
        (u32::from(track.width()), u32::from(track.height())),
        (WIDTH, HEIGHT)
    );
    assert_eq!((WIDTH, HEIGHT), (720, 1280));
    assert_eq!(track.sample_count(), Tool::Sonar.frames());
    assert_eq!(track.frame_rate(), f64::from(Tool::Sonar.fps()));
}

#[test]
fn every_fixed_sonar_seed_stays_under_the_cap() {
    std::thread::scope(|scope| {
        for seed in SEEDS {
            scope.spawn(move || {
                let bytes = encode(&derive(seed, ToolPin::Tool(Tool::Sonar)))
                    .unwrap()
                    .len();
                assert!(bytes < MAX_BYTES, "seed {seed}: {bytes} bytes");
            });
        }
    });
}

#[test]
fn settings_name_the_encoder_and_frame_size() {
    let settings = settings();
    assert!(settings.contains("openh264"), "{settings}");
    assert!(settings.contains("720x1280"), "{settings}");
}

#[test]
fn openh264_version_matches_the_lockfile() {
    let lock = include_str!("../../Cargo.lock");
    let entry = format!("name = \"openh264\"\nversion = \"{OPENH264}\"");
    assert!(
        lock.contains(&entry),
        "Cargo.lock has no openh264 {OPENH264}"
    );
    assert!(settings().contains(OPENH264));
}
