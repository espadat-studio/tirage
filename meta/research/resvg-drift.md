# resvg drift against tiny-skia PNGs

Answers [#130](https://github.com/espadat-studio/tirage/issues/130) for the SVG export map [#128](https://github.com/espadat-studio/tirage/issues/128).
Question: if a scene drawn through `Surface` is written as SVG and rasterized by resvg, how far does it drift from our tiny-skia PNG, which ops cause it, and which resvg pairs with tiny-skia 0.12?

## Answer

- Pair: resvg 0.47.0 or later. Current is 0.48.1 (2026-08-02), which needs tiny-skia `^0.12.0`, kurbo `0.13`, MSRV 1.85.
- Geometry drift is zero when the recorder writes the `Path` segments it already has, as `M/L/Q/C/Z`, with shortest round-trip `f32` numbers.
- Drift comes from SVG primitives that re-derive geometry (`<circle>`, `<ellipse>`), from rounded numbers, and from the SVG default miter limit.
- Paint drift is small but nonzero: SVG opacity is quantized to 8 bits, and clips and blend modes go through an offscreen layer.
- Two ops have no faithful SVG form: `with_plus` (no `plus-lighter` in usvg) and `with_blur` (resvg blurs with a different kernel).
- Fidelity ceiling for a recording `Surface`: max luma diff about 1 per pixel on plain fills, strokes, gradients and alpha. Up to 8 at clip edges where translucent draws overlap. Blur and plus need per-Tool emitters, or the Tool is not supported.
- For the 29 site-SVG Tools this is enough. The [SVG census](https://github.com/espadat-studio/tirage/blob/research/svg-census/meta/research/svg-census.md) finds none of them use `with_blur` or `with_plus`. 6 use `with_clip` and 5 use `with_alpha`, so clip edges are the worst case: ≤ 8 luma, ≤ 13 with nested clips.
- Correction to the census: `fill_circle` must not become `<circle>`. That costs up to 45.8 luma at the rim (below).
- usvg 0.48.1 has no `@font-face` support (no match in its source). A `<text>` test must load DejaVu Sans Mono into `usvg::Options::fontdb`, which needs the `text` feature.

## Version pairing

| resvg | tiny-skia | usvg kurbo | released |
|---|---|---|---|
| 0.45.1 | ^0.11.4 | 0.11 | 2025-04-17 |
| 0.46.0 | ^0.11.4 | 0.13 | 2026-01-11 |
| 0.47.0 | ^0.12.0 | 0.13 | 2026-02-09 |
| 0.48.0 | ^0.12.0 | 0.13 | 2026-08-02 |
| 0.48.1 | ^0.12.0 | 0.13 | 2026-08-02 |

Sources: crates.io `api/v1/crates/resvg/<v>/dependencies`; resvg [CHANGELOG](https://github.com/linebender/resvg/blob/main/CHANGELOG.md) 0.47.0 says "`tiny-skia` has a major version bump from 0.11 to 0.12". The 0.46.0 entry upgraded kurbo to 0.13 and svgtypes to 0.16.1.

- Use `resvg = { version = "0.48.1", default-features = false }`. That drops text, system fonts, raster images and svgz. Nothing we emit needs them if glyphs are written as paths.
- 0.48.0 moved text to skrifa and harfrust. With `text` off, it does not pull skrifa, so our skrifa 0.44 does not clash.
- The pair builds on our toolchain (1.99) in the scratch crate below.

## How resvg draws, and where it departs from `Surface`

resvg parses with usvg, then calls the same tiny-skia `fill_path` and `stroke_path` we call. Drift can only enter at parse time (geometry, numbers, paint) or in resvg's compositing (layers).

| Step | resvg 0.48.1 behaviour | Effect on us | Source |
|---|---|---|---|
| Path numbers | svgtypes parses `f64`, usvg casts to `f32` | Shortest `f32` text round-trips exactly | [usvg `shapes.rs` `convert_path`](https://docs.rs/crate/usvg/0.48.1/source/src/parser/shapes.rs) |
| Path commands | `SimplifyingPathParser` keeps `Q` as quads, `C` as cubics; converts `A`, `S`, `T`, `H`, `V`, relatives | Writing tiny-skia segments 1:1 is lossless | same |
| `<circle>`, `<ellipse>`, `<rect rx>` | 4 quarter arcs through `kurbo::Arc::to_cubic_beziers(0.1, ..)` | Differs from tiny-skia `push_circle` (conics to quads) and from our kurbo arcs at tolerance 0.01 | same, `ellipse_to_path`, `arc_to` |
| Stroke miter limit | default 4 per SVG spec | We stroke with 10, so must write `stroke-miterlimit="10"` | [usvg `style.rs`](https://docs.rs/crate/usvg/0.48.1/source/src/parser/style.rs) line 123 |
| Solid colour + opacity | `set_color_rgba8(r, g, b, opacity.to_u8())` | Our `f32` alpha becomes `round(a*255)/255` | [resvg `path.rs`](https://docs.rs/crate/resvg/0.48.1/source/src/path.rs) `fill_path` |
| Gradient stops | `Color::from_rgba8(.., (stop_opacity * opacity).to_u8())`, then tiny-skia `LinearGradient` / `RadialGradient` | Same shader; alpha quantized to 8 bits | same, `convert_base_gradient` |
| Anti-aliasing | `shape-rendering` → `paint.anti_alias` | `fill_rect` and `fill_rect_linear` need `shape-rendering="crispEdges"` | same |
| Group opacity, clip, mask, filter, blend | `should_isolate()` → offscreen `Pixmap`, then `draw_pixmap` with `Nearest` | Clip is applied to the layer, not to each draw; blend acts between layer and backdrop | [resvg `render.rs`](https://docs.rs/crate/resvg/0.48.1/source/src/render.rs) `render_group`; [usvg `tree/mod.rs`](https://docs.rs/crate/usvg/0.48.1/source/src/tree/mod.rs) `should_isolate` |
| Clip path | Fill clip into a black pixmap with `BlendMode::Clear`, alpha `Mask::from_pixmap`, invert, `apply_mask` | Our clip is `Mask::fill_path(.., anti_alias=true)` applied per draw | [resvg `clip.rs`](https://docs.rs/crate/resvg/0.48.1/source/src/clip.rs) |
| `mix-blend-mode` | 16 CSS modes; no `plus-lighter` | `with_multiply` maps; `with_plus` has no form | [usvg `svgtree/mod.rs`](https://docs.rs/crate/usvg/0.48.1/source/src/parser/svgtree/mod.rs) `BlendMode::parse` |
| Path blend | Every path draws with `SourceOver`; blend only on groups | Multiply must wrap each path in its own `<g>` | `render.rs` `render_node` |
| `feGaussianBlur` | box blur if σ ≥ 2, IIR otherwise; default `color-interpolation-filters` is `linearRGB` | Different kernel from our `box_windows`; write `sRGB` | [resvg `filter/mod.rs`](https://docs.rs/crate/resvg/0.48.1/source/src/filter/mod.rs) `apply_blur`, `resolve_std_dev` |
| `<image>` | `image-rendering` → `FilterQuality` (`optimizeQuality` → Bicubic, `pixelated` → Nearest) | `draw_smooth` Bicubic maps; `draw_nearest` is our own sampler with Chrome's edge ties, so it may differ | [resvg `image.rs`](https://docs.rs/crate/resvg/0.48.1/source/src/image.rs) |

Not in scope for drift: stroke dashes. `Surface` has no dash op; riso and totem draw dashes as separate segments.

## Measurement

Throwaway crate outside the repo: tiny-skia 0.12.0, kurbo 0.13, resvg 0.48.1 without default features.
Each case draws a 256×256 scene with tiny-skia the way `Surface` does. It writes equivalent SVG by hand, rasterizes with `resvg::render`, and compares.
Luma is Rec. 601 on demultiplied 8-bit RGB. The background is opaque, so every pixel counts. "bytes" counts differing RGBA bytes out of 262,144.

| Case | SVG written | max luma | mean luma | px ≥ 0.5 | bytes |
|---|---|---|---|---|---|
| circle (`from_circle`) | `<circle>` | 45.77 | 0.1028 | 403 | 1209 |
| circle | exact `d` | 0 | 0 | 0 | 0 |
| rotated ellipse (kurbo, tol 0.01) | `<ellipse>` + `rotate` | 22.94 | 0.0244 | 123 | 369 |
| rotated ellipse | exact `d` | 0 | 0 | 0 | 0 |
| 40-quad blob | shortest `f32` | 0 | 0 | 0 | 0 |
| 40-quad blob | 3 decimals | 11.03 | 0.0007 | 4 | 12 |
| 40-quad blob | 2 decimals | 11.91 | 0.0076 | 44 | 132 |
| 40-quad blob | 1 decimal | 45.18 | 0.0761 | 386 | 1158 |
| 40-quad blob | integers | 101.98 | 0.4876 | 1150 | 3449 |
| zigzag stroke, miter, w 7.5 | no `stroke-miterlimit` | 127.42 | 0.5809 | 414 | 1242 |
| zigzag stroke, miter | `stroke-miterlimit="10"` | 0 | 0 | 0 | 0 |
| zigzag stroke, round join + cap | exact | 0 | 0 | 0 | 0 |
| 3-stop linear gradient rect, AA off | `crispEdges` | 0 | 0 | 0 | 0 |
| radial fade, alpha 0.63 → 0 | `stop-opacity="0.63"` | 1.00 | 0.0348 | 2692 | 7344 |
| 3 overlapping fills, alpha 0.37 | `fill-opacity` per path | 0.30 | 0.0516 | 0 | 11311 |
| same, alpha 94/255 | `fill-opacity` per path | 0 | 0 | 0 | 0 |
| 3 overlapping fills, alpha 0.37 | one `<g opacity>` | 60.65 | 6.1432 | 13213 | 40094 |
| rotated AA clip, 1 opaque fill | `<g clip-path>` | 0.89 | 0.0010 | 24 | 280 |
| rotated AA clip, 3 overlapping alpha 0.6 fills | `<g clip-path>` | 7.84 | 0.0389 | 300 | 11221 |
| same | `clip-path` on each path | 7.84 | 0.0783 | 5419 | 16324 |
| same, inside a second (circle) clip | nested `<g clip-path>` | 12.64 | 0.0407 | 305 | 11210 |
| 3 overlapping multiply fills | one `<g mix-blend-mode>` per path | 1.59 | 0.1809 | 11293 | 37061 |
| 3 overlapping multiply fills | one `<g>` around all | 165.33 | 16.4217 | 13256 | 48068 |
| 3 plus fills, alpha 0.7, dark bg | `screen` per path (nearest SVG mode) | 49.71 | 5.7945 | 21505 | 64479 |
| blurred disc, σ 1.5 | `feGaussianBlur`, `sRGB` | 6.96 | 0.1476 | 5088 | 14346 |
| blurred disc, σ 1.5 | default `linearRGB` | 7.74 | 1.2538 | 33470 | 100310 |
| blurred disc, σ 6 | `feGaussianBlur`, `sRGB` | 5.96 | 0.7776 | 19676 | 56498 |
| blurred disc, σ 6 | default `linearRGB` | 8.03 | 1.0949 | 20357 | 100082 |

The blur case reuses our `box_windows` and `box_pass` from `src/surface.rs`. The filter region is the full canvas in `userSpaceOnUse`, matching our full-canvas layer.

Why each nonzero row drifts:

- Alpha 0.37: we pass `f32` alpha. resvg rounds to 94/255. The diff is under 1 luma and changes at most one byte step. Alpha already on a k/255 grid gives zero drift.
- Group opacity: `with_alpha` sets alpha per draw, so overlaps stack. `<g opacity>` flattens the group first. That is a different picture, not drift.
- Clip edges: one opaque draw stays under 1 luma. Overlapping translucent draws reach 7.84 at AA edge pixels. We mask each draw. resvg masks a layer it renders through its own clip pixmap. Moving `clip-path` onto each path does not lower the max. A nested clip adds a layer and raises it to 12.64.
- Multiply per path: the isolated layer is 8-bit premultiplied before the blend, so we lose about 1.6 luma. One group around all paths multiplies the group as a whole, not path against path.
- Plus: usvg cannot express it. `screen` is the closest mode and is far off.
- Blur: kernels differ even in sRGB. resvg's IIR (σ < 2) and box blur do not match our three-pass box. Leaving out `color-interpolation-filters` adds a linearRGB round trip.

## Rules for the recording `Surface`

1. Write every `Path2D` and `Path` as `<path d>` from tiny-skia `PathSegment`s. Never as `<circle>`, `<ellipse>` or `A`.
2. Format numbers with Rust `{}` on `f32` (shortest round-trip). Two decimals would save 28% of bytes on the blob, but cost up to 12 luma.
3. Always write `stroke-miterlimit="10"` with `stroke-linejoin="miter"`.
4. Write `with_alpha` as `fill-opacity` / `stroke-opacity` on each element, never as group `opacity`.
5. Write `fill_rect`, `fill_rect_linear` and nearest pixel writes with `shape-rendering="crispEdges"`.
6. Write gradients with `gradientUnits="userSpaceOnUse"` and the same stop offsets and colours. Expect up to 1 luma from stop alpha.
7. Wrap each multiply draw in its own `<g style="mix-blend-mode:multiply">`. Do not group them.
8. Write `with_clip` as one `<clipPath>` per clip, with the transform on its path. Expect up to 8 luma at AA edges where translucent draws overlap inside a clip, and up to 13 with two nested clips.
9. `with_plus` and `with_blur` cannot be recorded faithfully. They need a per-Tool emitter, or the Tool is unsupported. This matches the map's backend note.
10. A test tolerance of max luma ≤ 1 covers rules 1–7. Clipped scenes need ≤ 8 (≤ 13 nested), or a mean bound: every clip case stays under 0.08 mean.

## Open questions

- Should Tools that use `with_alpha` quantize alpha to k/255 at the source? SVG drift would drop to 0, but the PNG changes by up to 1 luma and every Tool golden must be regenerated.
- Is ≤ 8 luma (≤ 13 nested) at clip edges acceptable for the test bar, or should clipped Tools compare by mean luma?
