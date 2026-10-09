use std::io::Cursor;

use mp4::{MediaType, Mp4Reader};
use tirage::{Tool, ToolPin, derive};
use tirage_encode::{HEIGHT, WIDTH, encode};

const MAX_BYTES: usize = 1_500_000;
const SEEDS: [u64; 3] = [1, 2, 3];

#[test]
fn frond_loop_is_h264_at_720x1280_with_the_tools_frames_and_fps() {
    let mp4 = encode(&derive(1, ToolPin::Tool(Tool::Frond))).unwrap();
    let reader = Mp4Reader::read_header(Cursor::new(&mp4[..]), mp4.len() as u64).unwrap();
    let tracks: Vec<_> = reader.tracks().values().collect();
    assert_eq!(tracks.len(), 1);
    let track = tracks[0];
    assert_eq!(track.media_type().unwrap(), MediaType::H264);
    assert_eq!(
        (u32::from(track.width()), u32::from(track.height())),
        (WIDTH, HEIGHT)
    );
    assert_eq!(track.sample_count(), Tool::Frond.frames());
    assert_eq!(track.frame_rate(), f64::from(Tool::Frond.fps()));
}

#[test]
fn every_fixed_frond_seed_stays_under_the_cap() {
    std::thread::scope(|scope| {
        for seed in SEEDS {
            scope.spawn(move || {
                let bytes = encode(&derive(seed, ToolPin::Tool(Tool::Frond)))
                    .unwrap()
                    .len();
                assert!(bytes < MAX_BYTES, "seed {seed}: {bytes} bytes");
            });
        }
    });
}
