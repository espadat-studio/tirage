# Reference export determinism

Question (#4): are Reference exports from `pg.ts` deterministic, what varies, and what must be pinned for a reproducible oracle? How does `~/code/recordreel` measure diff percentages, and can we reuse it?

## Answer

- Stills are deterministic. Every PNG and SVG run was byte-identical (15 PNG + 9 SVG exports, 3 runs each).
- Loops are deterministic in content only. Decoded frames match bit for bit across runs. The MP4 bytes differ in frame timestamps and `creation_time`.
- A reproducible oracle needs a pinned `--seed`, motion state, frame (`scrub`), the page version, and the Chromium build. Compare videos by decoded frames, never by file hash.

## Setup

- Driver: `~/.claude/skills/playgrnd/scripts/pg.ts` (playwright-core 1.4.2 driving `/usr/bin/chromium` 152.0.7977.82, headless, viewport 1600x1000).
- Recipe: `--seed 4242 --ratio 9:16 --size 1200 --colors "#1b1f3b,#e94f37,#f6f7eb,#3f88c5"`, all other Parameters default.
- Output: 1200x2133 stills, 1120x1990 H.264 MP4 (`--set frames=24`, default 12 fps).
- Compared with `sha256sum`, `magick compare -metric AE|RMSE`, and `ffmpeg -f framemd5 -fps_mode passthrough`.
- Date: 2026-10-08, live `www.playgrnd.tools` pages.

## Results: same Recipe, 3 runs

| Tool | PNG | SVG | Video |
|---|---|---|---|
| aura | identical | not offered | frames identical (24/24); bytes differ |
| riso | identical | identical | frames identical (24/24); bytes differ |
| mist | identical | not offered | not run |
| optic | identical | identical | not run |
| stipple | identical | identical | not run |

Identical means equal SHA-256, so `AE=0` and `RMSE=0` follow.

## Results: what breaks determinism

Two runs per row, aura unless noted.

| Variant | Same bytes? | AE (pixels differ, of 2,559,600) |
|---|---|---|
| no `--seed` | no | 457,588 (17.9%) |
| no `--colors` (seed pinned) | yes | 0 |
| motion on, no `scrub` (aura) | yes | 0 |
| motion on, no `scrub` (stipple) | no | 15,668 (0.6%) |
| motion on, `scrub=10` (aura, stipple) | yes | 0 |
| motion on, `scrub=10`, SVG (riso) | yes | n/a |
| motion on `scrub=0` vs motion off (aura) | no | 314,193 (12.3%) |

## What varies, by source

Sources: the Tool page HTML (`curl -sL https://www.playgrnd.tools/<slug>/`, read 2026-10-08) and `pg.ts`.

- `Math.random`
  - Used only for a fresh seed (`newSeed`: `(Math.random()*99999|0)+1`) and for the shuffle-palette button.
  - Art comes from a seeded xorshift `rng(s)` and `hashi(...)`. With `?seed=N`, no randomness reaches the pixels.
  - So `--seed` is mandatory. Without it, every load is a new artwork.
- Time-based motion
  - PNG export paints `paint(ctx, W, H, curFrame, 700)`.
  - With motion off, `curFrame` stays 0, so the Still is fixed.
  - With motion on, the preview plays on `requestAnimationFrame` and `curFrame` follows the wall clock. The export takes whatever frame is current at click time. Stipple drifted by 0.6% between runs. Aura did not, but that was luck of timing.
  - Setting `scrub` stops playback (`setPlaying(false)`) and fixes `curFrame`. `--set scrub=N` made motion stills byte-stable.
  - The motion toggle changes the image even at frame 0 (12.3% of pixels for aura). Motion on/off belongs in the Recipe.
- Video timing
  - Video uses `MediaRecorder(canvas.captureStream(60))` at 16 Mbit/s. A `setTimeout` + `requestAnimationFrame` loop paints frames in real time.
  - Frame timestamps are wall-clock. Aura spacing ranged 0.15 to 0.22 s against a 0.083 s target. Container `r_frame_rate` came out as 20/3, 30000/1 and 79/12 on 3 runs.
  - The MP4 also carries `creation_time` (export wall clock). Only 33 to 47 bytes differ between runs, all in headers and timing boxes.
  - Decoded frame content was identical on all 3 runs for both Tools. Default ffmpeg frame-rate conversion hid this at first by duplicating and dropping frames. Use `-fps_mode passthrough`.
- Internal render resolution
  - aura and mist paint into a buffer of width `budget` and upscale it. PNG uses 700 and video uses 300.
  - A video frame is therefore not the same image as the PNG of that frame. Do not mix them in one oracle.
- Fonts
  - No canvas `fillText` in the 5 pages, and no `<text>` in the 3 SVGs. The DM Mono web font only styles the page UI. Fonts do not affect exports.
- Embedded timestamps
  - PNG: none (byte-identical across minutes).
  - SVG: none.
  - MP4: `creation_time` and packet timestamps.
  - The page shows a `new Date()` clock in the UI only.
- Anti-aliasing and rasterization
  - Stable on one machine and one Chromium build. Not tested across builds, GPU vs software raster, or OS.
  - Canvas 2D anti-aliasing and the image smoothing used for upscaling belong to Skia. Expect small edge differences after a Chromium upgrade.
- Page drift
  - `pg.ts` loads the live site on every export. The response has `cache-control: max-age=0, must-revalidate`, so a site deploy can change any Tool without notice.
- Device pixel ratio and viewport
  - Exports draw to a new offscreen canvas at `S.size`. `devicePixelRatio` only sizes the preview. No effect seen.

## Pin list for a reproducible oracle

1. `--seed N` (always).
2. Every Parameter and toggle, including motion on/off. `--colors` when the Palette is part of the Recipe.
3. For motion stills: `--toggle motionTog --set scrub=N`. Never export a motion still without `scrub`.
4. Page version. Save the Tool page HTML with each Reference export and record its SHA-256 (2026-10-08: aura `2c3727b12a1e`, riso `57fca60b3b13`, mist `f2415f92ec1c`, optic `7a93219faad2`, stipple `cf8309ab86a7`). A changed hash means re-export.
5. Chromium build (`chromium --version`) and the playwright-core version.
6. Video: compare decoded frames by index (`ffmpeg -fps_mode passthrough -f framemd5`), or drop video and export PNG per frame with `scrub=0..frames-1` at PNG resolution. Ignore container timing and `creation_time`.

## recordreel diff tooling

Source: `~/code/recordreel/crates/app/tests/compare.py` (49 lines, Python 3 + Pillow), documented in `~/code/recordreel/DESIGN.md` ("Check" bullet).

- Purpose: compare a takumi raster of an HTML card with a headless Chromium screenshot of the same page.
- Metric (lines 38-41):
  ```python
  diff = ImageChops.difference(browser, takumi).convert("L").point(lambda v: 255 if v > 48 else 0)
  share = sum(diff.histogram()[255:]) / (size[0] * size[1])
  ```
  - Absolute RGB difference per pixel, folded to luma (Pillow `L`, ITU-R 601 weights).
  - A pixel counts when that luma is above 48 of 255.
  - The denominator is the full frame (1080x1920, or 1200x630 for `link`).
  - Printed as `{share:.2%} differ`.
- The docstring says "more than 48 on any channel". The code does not do that. A pure blue difference of 255 has luma of about 29, so it does not count.
- No anti-aliasing handling, no SSIM or PSNR, no pass/fail. It always exits 0.
- Manual only: `DESIGNS_OUT=<dir> cargo nextest run -p app --test designs`, then `python3 -I crates/app/tests/compare.py <dir> <slug>`. Not in CI.
- Coupling: the `<slug>/<fixture>/<kind>.html` layout, hardcoded card sizes, a built-in HTTP server and the Chromium screenshot step.

Reusable? Only the idea, not the file. The useful part is those 2 lines: two image paths in, one ratio out. For visuals:

- Byte equality (`sha256sum`) is the right oracle for repeat Reference exports. This study found zero drift.
- For our Visual vs a Reference export, use a thresholded share of differing pixels like recordreel. Either:
  - `magick compare -metric AE -fuzz <N>% a.png b.png null:`, divided by `w*h`. `-fuzz` uses RGB distance, so it does not have the luma blind spot.
  - Or a per-channel max, if we port the Python. That fixes the docstring bug.
- Pick the threshold from measured anti-aliasing noise between our renderer and Chromium, not 48 by default.

## Not tested

- Cross-machine and cross-Chromium-version pixel drift.
- `svg-anim`, `svg-loop` and `txt` formats.
- Video for mist, optic and stipple.
- WebM fallback, used when MP4 recording is not supported.
