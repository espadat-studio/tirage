# Loops as pre-rendered video

A Loop reaches the Reel page as a server-rendered H.264 MP4, not as WASM drawing live in the browser. The core yields raw frames and stays wasm-clean. A separate non-core tirage crate encodes them with `openh264` and an MP4 muxer, so recordreel stays one binary with nothing to install. recordreel renders a Loop lazily on first request and caches it on disk like a Share card. The Still is frame 0 and the `<video poster>`, so reduced motion, blocked autoplay and Low Power Mode all fall back to the Still with no jump. Only the full-bleed backdrops loop (kiosk, frond, sonar). Each Slide holds its own `<video preload="none">`, and only the Slide in view gets a `src` and plays.

## Considered Options

- **Live WASM in the browser**: no encoder and no cache, but tiny-skia takes ~97 ms per 1080x1920 frame on desktop. That is ~2-3 fps on a mid-range phone (vello_cpu reaches ~12 fps). It would also bring back the frontend runtime recordreel rejected to keep first paint fast.
- **Animated WebP**: needs no `<video>`, but it came out 2-4x larger than H.264 on every Tool measured and gets no hardware decode.
- **VP9 WebM with an H.264 fallback**: smaller only on kiosk, for two encodes and two cache entries.
- **`ffmpeg` subprocess (x264)**: 10-30% smaller files, but it adds a runtime dependency to the deploy image and a GPL encoder.
- **CSS motion on the Still**: free, but it is not the Tool's own motion.

## Consequences

- A Loop is 720x1280 at the Tool's own frame count and fps. The Still keeps full resolution for the poster and the Share card.
- A Loop must stay under 1.5 MB, checked in tests on fixed Seeds per Tool. A Tool over the cap ships Still-only until it is tuned. Measured with x264 crf28: kiosk 0.71 MB, sonar 1.15 MB, frond 1.34 MB.
- The fidelity check covers Loop frames as well as the Still.
- The cache key carries the encoder settings as well as the Seed and Recipe version.
- ADR 0001's deferred `wasm-bindgen` packaging is not needed for recordreel.
