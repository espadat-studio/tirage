# SVG Stills by a recording Surface

A Still renders as SVG through a second backend inside the same `Surface` struct, not through per-Tool emitters. `Surface` holds either a tiny-skia raster or an SVG writer, and every op matches on it, so Tool code is shared by PNG and SVG and never knows which one it draws to. The writer streams elements as ops arrive. Clips become inline `<clipPath>` plus a `<g>` scoped to the `with_clip` closure; alpha and multiply are written on each element. All 29 site-SVG Tools record. atlas and kiosk write their hinted glyph outlines as `<path>`, not `<text>`. carve's per-pixel ramps flush as a PNG `<image>`. Nearest buffers (fete, quilt, static) become row-merged `<rect>` grids. Grain and dither are dropped. Ops with no faithful SVG form (blur, plus, Bicubic resampling, radial fade) panic in SVG mode. No site-SVG Tool uses them.

## Considered Options

- **Per-Tool emitters, as the site does for 21 Tools**: richer markup (`<text>`, gradients), but each emitter is a second copy of the Tool's drawing code. Each one can drift from the PNG that the fidelity check proves.
- **`Surface` as a trait, Tools generic over it**: the same effect as the enum, but it means signature churn in 56 files and two compiled copies of every Tool.
- **Always record a display list, replay to raster or SVG**: one code path, but it rewires the proven PNG path and puts every golden at risk for no PNG gain.
- **Real `<text>` for atlas and kiosk**: editable type, but it can't pixel-match hinted outlines, and it needs a font choice (embed ~330 KB per face, or name a stack).
- **carve as a gradient approximation, or unsupported**: the site's approximation fails our own fidelity check. Leaving carve out breaks the 29-Tool parity.

## Consequences

- An SVG is held to the PNG of the same Recipe (max luma ≤ 1, clipped ≤ 8/13). It is not checked against the site's SVG.
- The writer's output rules are fixed by measured resvg drift: `<path>` not `<circle>`, per-element opacity, `stroke-miterlimit="10"`. A path inside the frame writes each coordinate at the fewest digits that keep its 1/256 px edge, since tiny-skia truncates points to that grid. A path the frame clips keeps shortest `f32` coordinates, because the clipper cuts curves before snapping. A transformed path writes its `matrix` and keeps shortest `f32` coordinates too, because tiny-skia maps the path before snapping. Runs of one ink and edge mode share a `<g fill>`, with `shape-rendering="crispEdges"` for rects. A partial-pixel rect overrides that `fill` when its shifted ink differs, and writes `fill-opacity` at 4 digits, since resvg rounds opacity to a u8.
- carve's SVG is partly raster. The docs say so.
- A new op added to `Surface` needs an SVG arm, or it panics in SVG mode.
- Adding a per-Tool emitter later is a new decision, not a hook that already exists.
