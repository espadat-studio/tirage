use std::io::Cursor;

use mp4::{MediaType, Mp4Reader};
use tirage::{Tool, ToolPin, derive};
use tirage_encode::{HEIGHT, MAX_BYTES, WIDTH, encode};

const SEEDS: [u64; 6] = [1, 2, 3, 5, 13, 26];

#[test]
fn kiosk_loop_is_h264_at_720x1280_with_the_tools_frames_and_fps() {
    let mp4 = encode(&derive(5, ToolPin::Tool(Tool::Kiosk))).unwrap();
    let reader = Mp4Reader::read_header(Cursor::new(&mp4[..]), mp4.len() as u64).unwrap();
    let tracks: Vec<_> = reader.tracks().values().collect();
    assert_eq!(tracks.len(), 1);
    let track = tracks[0];
    assert_eq!(track.media_type().unwrap(), MediaType::H264);
    assert_eq!(
        (u32::from(track.width()), u32::from(track.height())),
        (WIDTH, HEIGHT)
    );
    assert_eq!(track.sample_count(), Tool::Kiosk.frames());
    assert_eq!(track.frame_rate(), f64::from(Tool::Kiosk.fps()));
}

#[test]
fn every_fixed_kiosk_seed_stays_under_the_cap() {
    std::thread::scope(|scope| {
        for seed in SEEDS {
            scope.spawn(move || {
                let bytes = encode(&derive(seed, ToolPin::Tool(Tool::Kiosk)))
                    .unwrap()
                    .len();
                eprintln!("kiosk seed {seed}: {bytes} bytes");
                assert!(bytes < MAX_BYTES, "seed {seed}: {bytes} bytes");
            });
        }
    });
}
