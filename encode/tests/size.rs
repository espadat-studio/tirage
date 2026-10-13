use std::io::Cursor;

use mp4::Mp4Reader;
use tirage::{Tool, ToolPin, derive};
use tirage_encode::{Error, cap, encode, settings};

#[test]
fn a_sonar_loop_at_360x640_is_that_size_in_a_quarter_of_the_bytes() {
    let mp4 = encode(&derive(1, ToolPin::Tool(Tool::Sonar)), 360, 640).unwrap();
    let reader = Mp4Reader::read_header(Cursor::new(&mp4[..]), mp4.len() as u64).unwrap();
    let track = reader.tracks().values().next().unwrap();
    assert_eq!((track.width(), track.height()), (360, 640));
    assert_eq!(track.sample_count(), Tool::Sonar.frames());
    assert_eq!(track.frame_rate(), f64::from(Tool::Sonar.fps()));
    assert!(mp4.len() < 375_000, "{} bytes", mp4.len());
}

#[test]
fn the_cap_scales_up_with_pixel_area_and_never_below_1_5_mb() {
    assert_eq!(cap(720, 1280), 1_500_000);
    assert_eq!(cap(1280, 720), 1_500_000);
    assert_eq!(cap(1440, 2560), 6_000_000);
    assert_eq!(cap(360, 640), 1_500_000);
}

#[test]
fn an_odd_or_out_of_range_size_is_an_error() {
    let recipe = derive(1, ToolPin::Tool(Tool::Sonar));
    for (width, height) in [(361, 640), (360, 641), (14, 640), (360, 2050)] {
        assert!(
            matches!(
                encode(&recipe, width, height),
                Err(Error::Size { width: w, height: h }) if (w, h) == (width, height)
            ),
            "{width}x{height}"
        );
    }
}

#[test]
fn settings_name_the_size_and_its_budget() {
    let settings = settings(360, 640);
    assert!(settings.contains("h264 360x640"), "{settings}");
    assert!(settings.contains("budget 300000 bytes"), "{settings}");
}
