# Canvas 2D API census vs Rust rasterizers

Resolves #2 (part of map #1). Researched 2026-10-08.

## Verdict

- All 52 Tools use only `getContext("2d")`. None uses WebGL.
- The feature surface is small. No Tool uses `shadowBlur`, `createPattern`, conic gradients, `bezierCurveTo`, `roundRect`, `strokeText`, `letterSpacing` or `isPointInPath`.
- The heavy parts are pixel-buffer loops (22 Tools write raw pixels; all 52 run a shared grain/dither pass) and plain vector fills and strokes.
- **vello_cpu 0.3**: best native fit. 49/52 Tools need no hand-written drawing code. Gaps: `arcTo` (frond, zig) and one SVG-stamp Tool (oddgrid).
- **tiny-skia 0.12**: 27/52 with no extra code. It reaches 52/52 with 4 small hand-written pieces: arc to cubic (16 Tools), `arcTo` (2), box/gaussian blur (1), save/restore stack (8). Text (2 Tools) needs a font crate on top. It is the most mature and best-known option.
- **raqote 0.8**: 37/52 with no extra code. Its text depends on `font-kit` with a FreeType loader, which is a poor fit for WASM. It has no advantage over tiny-skia.
- **femtovg**: out. It has only OpenGL and wgpu backends, no CPU rasterizer ([README](https://github.com/femtovg/femtovg)).

Recommendation: start with tiny-skia plus `kurbo` for arcs, or with vello_cpu if native text and blur matter more than maturity. Both read and write raw RGBA buffers, which covers the dominant pixel-loop Tools.

## Method

- Source: the 52 Tool pages from `https://www.playgrnd.tools/<slug>/`, saved locally. Every page has 3 inline scripts:
  - a tool-specific renderer (27-47 KB)
  - a small 1.6 KB block
  - a 46 KB UI and export shell, identical on all 52 pages
- The shell makes no drawing calls beyond one `drawImage` for a 64x64 thumbnail, so it is left out.
- Each renderer embeds the same 6 helper functions (`__grain`, `__dither` and their noise/Bayer helpers). They are counted once as a shared row, then stripped before counting per Tool.
- Counting means a regex search for each API name. "Tools" is the number of Tools whose renderer uses the feature at least once. "Calls" is the number of call sites in the code, not calls made at runtime.
- Rough split by Tool: 30 vector-only, 13 pixel-buffer-only, 9 mixed.

## Feature x Tool counts and support

Legend: **N** = native. **H** = needs a hand-written version (size noted). **X** = not supported. Tools are listed when 9 or fewer use a feature.

| Feature | Tools | Calls | tiny-skia | vello_cpu | raqote |
|---|---|---|---|---|---|
| Shared `__grain` + `__dither` pass (getImageData, per-pixel loop, putImageData) | 52 | 2 per Tool | N, `Pixmap::data_mut` | N, pixmap buffer | N, `get_data_mut` |
| `getImageData` inside the renderer (read back) | 4 (chaff, delta, frond, strand) | 5 | N | N | N |
| `putImageData` / `createImageData` (raw pixel field) | 22 / 21 | 24 / 21 | N, caveat 1 | N, caveat 1 | N, caveat 1 |
| `fillRect` / `strokeRect` | 35 | 93 | N | N | N |
| `clearRect` | 4 (delta, fete, quilt, static) | 4 | N | N | N |
| `beginPath` + `moveTo`/`lineTo`/`closePath` | 29 / 22 / 16 | 68 / 157 / 24 | N | N | N |
| `fill()` / `stroke()` | 25 / 17 | 52 / 26 | N | N | N |
| `quadraticCurveTo` | 1 (frond) | 2 | N | N | N |
| `bezierCurveTo` | 0 | 0 | N | N | N |
| `arc` | 16 | 27 | H, arc to cubics (~30 lines, or `kurbo::Arc`); `push_circle` covers full circles | N, `kurbo::Arc` | N, `PathBuilder::arc` |
| `arcTo` | 2 (frond, zig) | 5 | H, tangent math (~25 lines) | H | H |
| `ellipse` | 2 (specimen, splice) | 2 | N if axis-aligned and full (`push_oval`), else H | N, kurbo `Arc` with `x_rotation` | H, `arc` under a scale transform |
| `rect` | 6 | 8 | N | N | N |
| `Path2D` | 2 (specimen, vein) | 5 | N, `Path` is reusable | N, `BezPath` | N |
| `evenodd` fill rule | 3 (specimen, sprig, vein) | 8 | N, `FillRule::EvenOdd` | N, `Fill::EvenOdd` | N, `Winding::EvenOdd` |
| `lineWidth` / `lineCap` / `lineJoin` | 17 / 13 | 23 / 21 | N, `Stroke` | N | N, `StrokeStyle` |
| `setLineDash` | 2 (filament, frond) | 4 | N, `StrokeDash` | N, kurbo dashes | N |
| `createLinearGradient` | 2 (modular, pane) | 2 | N | N | N |
| `createRadialGradient` | 1 (frond) | 1 | N, two-point conical | N | N, two-circle |
| `createConicGradient` / `createPattern` | 0 / 0 | 0 | n/a | n/a | n/a |
| `globalAlpha` | 20 | 32 | N, paint/colour alpha | N, opacity layer or alpha | N, `DrawOptions.alpha` |
| `globalCompositeOperation` | 16 | 19 | N | N | N |
| - value `source-over` (mostly a reset) | 16 | 17 | N | N | N |
| - value `multiply` | 1 (parcel) | 1 | N, `BlendMode::Multiply` | N, `Mix::Multiply` | N |
| - value `lighter` | 1 (strand) | 1 | N, `BlendMode::Plus` | N, `Compose::Plus` | N, `Add` |
| `filter = "blur(...)"` | 1 (strand) | 2 | H, blur on a scratch pixmap | N, `GaussianBlur` (experimental, caveat 2) | H |
| `shadowBlur` / `shadowColor` / offsets | 0 | 0 | n/a | n/a | n/a |
| `fillText` + `measureText` + `font` + `textAlign`/`textBaseline` | 2 (atlas, kiosk) | 2 + 4 + 5 + 10 | X natively; H with a font crate (caveat 3) | N, `glyph_run` with the `text` feature | X for WASM (needs `font-kit` + FreeType) |
| `clip` | 6 (carve, optic, rise, riso, specimen, vee) | 8 | N, `Mask` | N, `push_clip_layer` | N, `push_clip` |
| `save` / `restore` | 8 | 24 | H, a small state stack in app code | N, `save_current_state` / `restore_state` | H |
| `translate` / `rotate` / `scale` | 4 / 3 / 2 | 8 / 5 / 2 | N, `Transform` | N, `Affine` | N, `Transform` |
| `setTransform` | 25 | 27 | N | N | N |
| `drawImage` (scale an offscreen buffer) | 9 | 9 | N, `draw_pixmap` | N, image paint | N, `draw_image_with_size_at` |
| `imageSmoothingEnabled` / `imageSmoothingQuality="high"` | 18 / 3 (aura, mist, pith) | 25 / 3 | N, `FilterQuality` Nearest/Bilinear/Bicubic | N, image quality | N, `FilterMode` Nearest/Bilinear |
| SVG built as markup and drawn via `new Image()` | 1 (oddgrid) | 1 | H, draw the marks as paths (or use resvg) | H | H |
| Offscreen `<canvas>` buffers | 52 | 116 | N, extra `Pixmap`s | N | N |
| `toDataURL` / `toBlob` (PNG export) | 52 | 52 | N, `encode_png` | encode with the `png` crate | N, `write_png` |
| `MediaRecorder` + `captureStream` (video export) | 52 (shell) | - | out of scope | out of scope | out of scope |

Caveats:

1. Canvas `ImageData` holds straight (non-premultiplied) RGBA. tiny-skia `Pixmap` stores premultiplied RGBA ([docs](https://docs.rs/tiny-skia/latest/tiny_skia/struct.Pixmap.html)). Opaque pixel fields copy over as-is. Any pixel loop that writes alpha < 255 must premultiply. The same applies to vello_cpu and raqote, which also composite premultiplied.
2. vello_cpu marks filters as "incomplete and experimental". `set_filter_effect` and `push_filter_layer` panic in multi-threaded mode. Only single-primitive filters exist ([RenderContext](https://docs.rs/vello_cpu/latest/vello_cpu/struct.RenderContext.html), [filter_effects](https://docs.rs/vello_cpu/latest/vello_cpu/filter_effects/index.html)). strand needs only one blur, so this is fine single-threaded.
3. The 2 text Tools draw single glyphs on a monospace grid. They use a system monospace font stack (`ui-monospace`, ..., `DejaVu Sans Mono`). Browser output therefore already differs by OS. A Rust port should embed one monospace font and turn glyph outlines into paths, for example with `ttf-parser` or `skrifa`. That is the approach resvg takes on top of tiny-skia. No shaping is needed for a fixed character grid.

## Rasterizer facts (primary sources)

- **tiny-skia 0.12.0** ([docs.rs](https://docs.rs/tiny-skia/latest/tiny_skia/), [repo](https://github.com/linebender/tiny-skia))
  - README lists text rendering, GPU, conic path segments and path effects other than dashing as out of scope.
  - `PathBuilder` has `move_to`, `line_to`, `quad_to`, `cubic_to`, `close`, `push_rect`, `push_oval` and `push_circle`. It has no arc or arcTo ([docs](https://docs.rs/tiny-skia-path/latest/tiny_skia_path/struct.PathBuilder.html)).
  - `BlendMode` has all 12 Porter-Duff modes plus 13 separable and 4 non-separable blend modes, 29 variants in total ([docs](https://docs.rs/tiny-skia/latest/tiny_skia/enum.BlendMode.html)).
  - Shaders: `LinearGradient`, `RadialGradient` (two-point conical, the same model as the Canvas 6-argument form), `SweepGradient`, `Pattern`.
  - It has no blur or filter API.
- **vello_cpu 0.3.0** ([docs.rs](https://docs.rs/vello_cpu/latest/vello_cpu/))
  - Supports paths, strokes, linear/radial/sweep gradients, images, blend and compose modes, opacity and mask layers, clip paths and glyph runs.
  - SIMD on x86_64, aarch64 and WebAssembly. `no_std` with `libm`.
  - Filters: `GaussianBlur`, `DropShadow`, `Flood`, `Offset`. Filter graphs and the CSS function filters are not implemented.
  - Compose modes (via peniko 0.6) include `Plus` and `PlusLighter` ([docs](https://docs.rs/peniko/latest/peniko/enum.Compose.html)).
  - Geometry comes from kurbo, whose `Arc` has `x_rotation` and can be appended to a `BezPath` ([docs](https://docs.rs/kurbo/latest/kurbo/struct.Arc.html)).
- **raqote 0.8.5** ([docs.rs](https://docs.rs/raqote/latest/raqote/))
  - `PathBuilder` has `arc` and `rect`.
  - Gradients: linear, radial, two-circle and sweep. Supports clipping, dashes, layers with blend modes, and image filter modes.
  - `draw_text` takes a `font_kit` FreeType font and is gated behind the optional `font-kit` feature ([DrawTarget](https://docs.rs/raqote/latest/raqote/struct.DrawTarget.html)). It has no blur.
- **femtovg** ([repo](https://github.com/femtovg/femtovg)): OpenGL (ES) and wgpu backends only. It has no CPU backend, so it cannot be a CPU/WASM rasterizer.

## Open points

- WASM build size and speed for tiny-skia vs vello_cpu were not measured. A prototype ticket could render one pixel Tool (e.g. mist) and one vector Tool (e.g. frond) with each.
- Anti-aliasing will not match the browser pixel for pixel in any of these rasterizers. Checks against a Reference export need a tolerance.
