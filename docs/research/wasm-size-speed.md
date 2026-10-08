# WASM size and render speed: tiny-skia vs vello_cpu

Resolves #11 (part of map #1). Measured 2026-10-08. Context: [Canvas 2D census](https://github.com/espadat-studio/tirage/blob/research/canvas2d-census/docs/research/canvas2d-census.md).

## Verdict

- **vello_cpu 0.3 is about 5x faster than tiny-skia 0.12** on the same scene, natively and in WASM.
- **Size cost of vello_cpu: about 80 KB more gzipped** (147-155 KB vs 69-73 KB). The wasm-bindgen JS glue is under 2 KB gzipped for both.
- **Do not build the renderer with `opt-level = "z"` or `"s"`.** `"z"` makes rendering 5-8x slower and `"s"` 3-6x slower, and both save little or nothing once gzipped. Use `opt-level = 3` (or 2) with `lto`, and build with `-C target-feature=+simd128`.
- **Best desktop WASM frame: vello_cpu, simd128, opt-level 3: 18-20 ms** (12-14 ms paths + 5.5-6 ms grain). tiny-skia: 93-97 ms.
- **30 fps on a mid-range phone is not plausible for this scene at 1080x1920.** With the 4x desktop-to-mid-tier-mobile factor from Lighthouse, vello_cpu lands at about 80 ms/frame (about 12 fps). The plausible range is 40-200 ms (5-25 fps), from Lighthouse's 2-10x band. tiny-skia lands at about 390 ms (about 2.5 fps).
- A 30 fps Loop needs at most 33 ms on the phone, so at most 8 ms on this desktop in WASM. The grain pass alone takes 5.5-6 ms. Ways to get there, not measured here: render the Loop at a lower resolution and upscale, use fewer or smaller shapes, make the grain cheaper (precomputed noise tile, or `u32`/SIMD per-pixel work), or pre-render the Loop's frames instead of rendering them live.

## Numbers

Host: AMD Ryzen 7 PRO 6850U, single thread. rustc 1.99.0, wasm-bindgen 0.2.129, binaryen (wasm-opt) 132, node 26.10.0, bun 1.4.2. Each time is the median of 120 frames after 10 warm-up frames. "paths" means building the 200 paths, filling them and rasterising. "grain" is the full-frame pixel pass.

### Size (bytes, `gzip -9`)

All builds use `lto = true`, `codegen-units = 1`, `panic = "abort"` and `strip = true`. "bindgen" is the `_bg.wasm` that `wasm-bindgen --target web` emits. "+wasm-opt" means that file after `wasm-opt` run at the matching level (`-Oz`, `-Os`, `-O2` or `-O3`).

| Build | tiny-skia bindgen | tiny-skia +wasm-opt | vello_cpu bindgen | vello_cpu +wasm-opt |
|---|---|---|---|---|
| opt-level "z", simd128 | 69,924 (raw 218,488) | 69,319 (raw 201,363) | 152,726 (raw 511,622) | 155,128 (raw 454,840) |
| opt-level 3, simd128 | 75,288 | 72,704 | 149,741 | 147,371 |
| opt-level 2, simd128 | - | 88,703 | - | 146,355 |
| opt-level "s", simd128 | - | 90,478 | - | 156,834 |
| opt-level "z", no SIMD | 72,478 | 71,021 | 364,493 | 399,099 |
| opt-level 3, no SIMD | 113,976 | 114,152 | 174,352 | 172,567 |

JS glue (`--target web`): tiny-skia 5,840 B raw, 1,709 B gzipped. vello_cpu 6,909 B raw, 1,941 B gzipped. The vello build exports one extra `level() -> String`, which adds the string glue.

wasm-opt `-Oz` shrinks the raw file by 8-11% but can grow the gzipped vello_cpu file by about 1.5%.

### Frame time (ms, paths + grain = total)

| Build | Runtime | tiny-skia | vello_cpu |
|---|---|---|---|
| native, opt-level 3 (x86-64 baseline) | native | 42.9 + 5.5 = **48.4** | 6.4 + 5.3 = **11.7** |
| native, opt-level 3, `target-cpu=native` | native | 44.1 + 4.6 = 48.7 | 7.8 + 5.1 = 12.9 |
| native, opt-level "z" | native | 390.4 + 5.8 = 396.1 | 50.6 + 5.6 = 56.2 |
| wasm, opt-level 3, simd128 | node | 91.5 + 5.8 = **97.3** | 14.1 + 5.8 = **19.9** |
| wasm, opt-level 3, simd128 | bun | 92.3 + 6.5 = 98.8 | 13.7 + 6.3 = 20.0 |
| wasm, opt-level 2, simd128 | node | 129.3 + 5.7 = 135.0 | 13.0 + 5.6 = 18.6 |
| wasm, opt-level "z", simd128 | node | 471.0 + 5.7 = 476.7 | 110.6 + 5.5 = 116.1 |
| wasm, opt-level "z", simd128 | bun | 469.3 + 5.6 = 474.9 | 108.9 + 5.5 = 114.5 |
| wasm, opt-level "s", simd128 | node | 311.8 + 5.7 = 317.6 | 115.3 + 5.6 = 120.8 |
| wasm, opt-level 3, no SIMD | node | 162.3 + 6.4 = 168.7 | 54.2 + 6.3 = 60.4 |
| wasm, opt-level 3, no SIMD | bun | 164.5 + 6.9 = 171.4 | 50.4 + 6.2 = 56.7 |
| wasm, opt-level "z", no SIMD | node | 545.5 + 5.6 = 551.1 | 1830.2 + 5.8 = 1835.9 |

A second run of `run.sh` (simd, opt "z" and 3) reproduced every number within 10%.

### Phone estimate

| | Desktop WASM (node) | x4 mid-tier phone | fps | x2-x10 band |
|---|---|---|---|---|
| vello_cpu, opt 3, simd128 | 19.9 ms | 80 ms | 12.6 | 40-199 ms (25-5 fps) |
| tiny-skia, opt 3, simd128 | 97.3 ms | 389 ms | 2.6 | 195-973 ms |

## Slowdown factor and its source

- Lighthouse throttles the CPU by "a constant 4x CPU multiplier which moves a typical run in the high-end desktop bracket somewhere into the mid-tier mobile bracket". Its table recommends 4x (range 2-10) for high-end desktop to mid-tier mobile ([Lighthouse throttling.md](https://github.com/GoogleChrome/lighthouse/blob/main/docs/throttling.md)).
- That table places devices by Lighthouse's *BenchmarkIndex*: high-end desktop 1500-2000, mid-tier mobile 125-800 (Chrome m86 calibration, same doc).
- I ran Lighthouse's own `computeBenchmarkIndex` ([page-functions.js](https://github.com/GoogleChrome/lighthouse/blob/main/core/lib/page-functions.js)) on this host. Node (V8) scored 2436-2560, and bun (JavaScriptCore) scored 1128-1218. So this laptop sits at or above the high-end desktop bracket under V8. The 4x factor is therefore, if anything, optimistic for a mid-tier phone.
- Caveat: BenchmarkIndex measures JS (string building and array copies), not WASM SIMD throughput. ARM phone cores may scale differently on SIMD-heavy rasterising. Only a run on a real device settles this.

## Scene

Shared by both renderers through `bench/wasm-size-speed/shared/scene.rs`, so the geometry and colours are the same in both.

- 1080x1920 premultiplied RGBA8 pixmap, redrawn from scratch each frame.
- Full-frame linear gradient (2 stops, opaque).
- Radial gradient circle, r = 700, with global alpha 0.7. tiny-skia uses `Shader::apply_opacity`. vello_cpu uses `Gradient::multiply_alpha`, not an opacity layer.
- 200 solid-colour paths with alpha 0.25-0.85 (Canvas `globalAlpha` folded into the paint colour), radius 20-200 px, animated per frame:
  - 67 full circles
  - 67 pie-slice arcs
  - 66 four-cubic blobs
  - Arcs are converted to cubics with one helper shared by both backends. This mirrors the hand-written arc piece that the census lists for tiny-skia.
- Grain: one xorshift value per pixel, added to R, G and B and clamped to `[0, alpha]`. This keeps premultiplied pixels valid, as census caveat 1 requires.
- The two renderers' outputs match visually (checked side by side). Their checksums differ because the anti-aliasing differs.
- Coverage is heavy: 2 full-frame gradient fills plus about 3 frame-areas of shapes. Real Tools may be lighter. Scale the estimate by your actual coverage.

## Notes from the sources

- vello_cpu picks its SIMD level when the code is compiled, through `fearless_simd`. "WASM SIMD doesn't have feature detection", so SIMD needs `-Ctarget-feature=+simd128` at build time ([fearless_simd 0.7 lib.rs docs](https://docs.rs/fearless_simd/0.7.0/fearless_simd/)). Without it, vello_cpu falls back to scalar code that is 3x (opt 3) to 16x (opt "z") slower.
- tiny-skia 0.12 has `wasm32` simd128 code paths in `src/wide/*` ([source](https://github.com/linebender/tiny-skia/tree/main/src/wide)). simd128 gives it about 1.7x at opt-level 3.
- vello_cpu features used: `std` and `u8_pipeline` only. The `u8_pipeline` is the speed path for `RenderMode::OptimizeSpeed`, which is the `RasterizerSettings` default. Dropping `f32_pipeline` reduces binary size ([vello_cpu 0.3 crate docs](https://docs.rs/vello_cpu/0.3.0/vello_cpu/)). `multithreading` is off. Using it in WASM would need WASM threads plus a cross-origin-isolated page. That was not tested.
- Fixed-width SIMD ships in Chrome 91, Firefox 89, Safari 16.4 and Node 16.4 ([WebAssembly features.json](https://github.com/WebAssembly/website/blob/main/features.json)). A simd128-only build is fine for current mobile browsers.
- Not measured: the browser `putImageData` copy of the 8.3 MB buffer, and browser WASM engines other than V8 (node) and JSC (bun). Chrome Android runs V8, and iOS Safari runs JSC.

## Reproduce

```sh
rustup target add wasm32-unknown-unknown
cargo binstall wasm-bindgen-cli@0.2.129
cd bench/wasm-size-speed && mkdir -p out && (cd out && npm i binaryen)
./run.sh
```

`run.sh` prints the native times, then every variant x profile x runtime, with sizes. Set `VARIANTS`/`PROFILES` to run a subset.
