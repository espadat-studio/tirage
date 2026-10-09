use std::io::Cursor;

use mp4::{MediaType, Mp4Reader};
use tirage::{Params, Plant, Tool, ToolPin, derive};
use tirage_encode::{Error, HEIGHT, MAX_BYTES, WIDTH, encode};

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

#[test]
fn a_loop_over_the_cap_is_an_error() {
    let mut recipe = derive(1, ToolPin::Tool(Tool::Frond));
    let Params::Frond(p) = recipe.params_mut() else {
        unreachable!()
    };
    p.set_plant(Plant::Bouquet);
    p.set_masses(6).unwrap();
    p.set_growth(0.75).unwrap();
    p.set_detail(1.0).unwrap();
    p.set_noise(1.0).unwrap();
    p.set_patch(1.0).unwrap();
    p.set_coarse(0.25).unwrap();
    p.set_breakup(0.5).unwrap();
    p.set_circles(1.0).unwrap();
    p.set_rules(1.0).unwrap();
    let error = encode(&recipe).unwrap_err();
    assert!(
        matches!(error, Error::OverCap { bytes } if bytes >= MAX_BYTES),
        "{error}"
    );
    assert!(
        error.to_string().ends_with("over the 1500000 byte cap"),
        "{error}"
    );
}
