# SVG census: Surface ops per site-SVG Tool

Answers [#129](https://github.com/espadat-studio/tirage/issues/129) for the [SVG export map](https://github.com/espadat-studio/tirage/issues/128). Base: `2188f19`.

## Answer

- 29 Tools export a Still SVG on the site. All 29 are ported.
- 25 draw only ops a recording `Surface` can write as SVG elements as they are.
- 3 (fete, quilt, static) also paint a low-res buffer with `draw_smooth(.., Nearest)`. A recorder can write that buffer as a grid of whole-pixel rects. No `<image>` is needed.
- 1 (carve) can't be recorded. Its ramp panels write pixels one by one with `set_pixel`: a curved gradient plus hashed noise per pixel.
- No site-SVG Tool uses `with_blur`, `with_plus`, `fill_radial_fade` or Bicubic `draw_smooth`.
- `edit_rgba` runs only in `chassis::finish` (grain, dither) on all 29. The site drops both passes in SVG, so the recorder skips it.
- 2 Tools draw type: atlas (Book) and kiosk (Bold). They use `text::Type::fill_centred`, which fills hinted glyph outlines from the bundled DejaVu Sans Mono with `fill_path`. These outlines record fine as `<path>`. A real `<text>` needs a per-Tool emitter, or a recorder hook on `Type`.

## Method

- Op inventory: every `.op(` call on the `Surface` in each `src/<tool>.rs`, plus `src/text.rs` and `src/chassis.rs`. `Surface` API from `src/surface.rs`.
- Site export path: `#expSvg` button and emitter names (`SvgSurface*`, `buildSVG`, `svgScene`, `svgDoc`, `markSVG`) found by grep in the live pages from `https://www.playgrnd.tools/<slug>/` (fetched 2026-10-11). 29 of our 51 slugs have `#expSvg`. This matches shell anatomy §7 (`research/shell-anatomy` branch, `docs/research/shell-anatomy.md`).
- No site code was read beyond names and tag names.

## Op recordability

Recordable as plain SVG:

| `Surface` op | SVG |
|---|---|
| `fill`, `fill_box`, `fill_circle`, `fill_path` | `<rect>`, `<circle>`, `<path>` with nonzero fill |
| `fill_rect` | `<rect shape-rendering="crispEdges">`; tiny-skia draws it with `anti_alias = false` |
| `fill_even_odd` | `<path fill-rule="evenodd">` |
| `stroke` | `<path>` with `stroke-linecap` butt/round, `stroke-linejoin` round/miter, `stroke-miterlimit="10"` |
| `fill_transformed` | `<path transform="matrix(..)">` |
| `with_alpha` | `fill-opacity` / `stroke-opacity` on each element, not group opacity: the alpha is per paint (`Surface::solid`) |
| `with_clip` (nested, with transform) | `<clipPath>` with a transformed child; nested clips become nested `<g clip-path>` |
| `with_multiply` | `mix-blend-mode: multiply` on the element |
| `fill_rect_linear` | `<linearGradient gradientUnits="userSpaceOnUse" spreadMethod="pad">`; the op ignores the current alpha, clip and blend, so the recorder must too |
| `draw_smooth(.., Nearest)` | one `<rect>` per cell (merge runs of one colour). Cell edges come from the same rule as `draw_nearest`, which gives exact cell-edge ties to the left cell |

Not recordable:

| `Surface` op | Why |
|---|---|
| `set_pixel` | Each pixel is its own colour. Vector output would need one rect per pixel, or a raster `<image>` |
| `edit_rgba` | Reads and rewrites the whole frame (grain, dither). Out of scope: the site drops both in SVG |
| `with_blur` | Our box blur is three box passes. `feGaussianBlur` is close but not equal. No site-SVG Tool uses it |
| `draw_smooth(.., Bicubic)` | A resampled image. No site-SVG Tool uses it |
| `with_plus` | `mix-blend-mode: plus-lighter` exists in CSS but SVG renderers don't agree on it. No site-SVG Tool uses it |

## Census

`site` is the site's SVG path: *rec* = recording surface (`SvgSurface*`), *emit* = dedicated emitter. Counts are call sites in the Tool module.

| Tool | site | `Surface` ops used | recordable | why not |
|---|---|---|---|---|
| atlas | emit | `fill_rect` 1, `Type::fill_centred` (`fill_path`) | yes, as glyph outlines | the site writes `<text>`; see below |
| bloom | rec | `fill_rect` 1 | yes | |
| carve | emit | `fill`, `fill_box` 6, `fill_circle`, `fill_path`, `stroke`, `with_clip` 2, `set_pixel` 1 | no | ramp panels are per-pixel (`set_pixel`). The site swaps in a `<linearGradient>` plus a `<filter>`, so its SVG doesn't match its own canvas there |
| fete | emit | `draw_smooth` Nearest, `fill_circle`, `stroke` | yes, with grid expansion | patch field is a Nearest buffer |
| filament | emit | `fill`, `fill_circle`, `fill_transformed`, `stroke` 2 | yes | |
| kiosk | rec | `fill_box` 4, `fill_circle` 2, `Type::fill_centred` (`fill_path`) | yes, as glyph outlines | the site's recorder turns `fillText` into `<text>` |
| modular | emit | `fill`, `fill_circle`, `fill_rect` 3, `fill_rect_linear`, `stroke` 2, `with_alpha` 2 | yes | |
| mosh | rec | `fill_rect` 1 | yes | |
| oddgrid | emit (`buildStaticSVG`) | `fill`, `fill_box`, `fill_circle`, `fill_path`, `fill_rect`, `stroke` | yes | |
| optic | emit | `fill` 2, `fill_box`, `fill_transformed`, `with_clip` | yes | |
| pane | rec | `fill_rect_linear` 1 | yes | |
| parcel | emit | `fill`, `fill_rect`, `stroke` 2, `with_multiply` | yes | the site uses `mix-blend-mode` too |
| prism | emit | `fill`, `fill_box`, `fill_rect`, `with_alpha` | yes | |
| quilt | emit | `draw_smooth` Nearest | yes, with grid expansion | the whole picture is a Nearest buffer |
| relief | rec | `fill`, `fill_even_odd`, `stroke` | yes | |
| rise | emit | `fill` 2, `fill_box`, `with_clip` | yes | |
| riso | emit | `fill_box` 2, `fill_path`, `stroke`, `with_alpha`, `with_clip` | yes | |
| sampler | rec | `fill`, `fill_transformed` | yes | |
| sonar | rec | `fill`, `fill_rect` | yes | |
| specimen | emit | `fill`, `fill_box`, `fill_even_odd`, `fill_path` 2, `stroke` 5, `with_alpha`, `with_clip` | yes | |
| static | emit | `fill`, `draw_smooth` Nearest | yes, with grid expansion | the noise field is a Nearest buffer |
| stipple | emit | `fill`, `fill_box`, `fill_circle` | yes | |
| tokens | emit | `fill`, `fill_box`, `fill_circle`, `stroke`, `with_alpha` | yes | |
| totem | rec | `fill`, `fill_rect` 4 | yes | |
| vee | emit | `fill`, `fill_transformed`, `with_clip` 3 | yes | |
| vein | emit | `fill`, `fill_circle`, `fill_even_odd`, `stroke` | yes | |
| warp | emit | `fill`, `fill_path` | yes | |
| weave | emit | `fill_rect` 2 | yes | |
| zig | emit | `fill`, `fill_path` | yes | |

All 29 also call `chassis::finish`, which uses `edit_rgba` for grain and dither. It is skipped in SVG.

## Summary counts

| | Tools |
|---|---|
| Site SVG Tools | 29 |
| Site path: recording surface | 8 (bloom, kiosk, mosh, pane, relief, sampler, sonar, totem) |
| Site path: dedicated emitter | 21 |
| Recordable, ops as they are | 25 |
| Recordable with Nearest-grid expansion | 3 (fete, quilt, static) |
| Not recordable | 1 (carve) |
| Draw type (`Type`) | 2 (atlas, kiosk) |
| Use `with_clip` | 6 (carve, optic, rise, riso, specimen, vee) |
| Use `with_alpha` | 5 (modular, prism, riso, specimen, tokens) |
| Use `with_multiply` | 1 (parcel) |
| Use `fill_rect_linear` | 2 (modular, pane) |

All 8 Tools the site records are recordable with our ops too.

## Glyphs and real `<text>`

Source today:

- `src/text.rs` loads `fonts/DejaVuSansMono.ttf` (Book, atlas) and `fonts/DejaVuSansMono-Bold.ttf` (Bold, kiosk) via `include_bytes!`. It reads them with skrifa.
- Each glyph is an outline from the font, not a hand-drawn path. Up to 256 px it is auto-hinted (light, smooth target). The left edge snaps to quarter pixels and the baseline to whole pixels. Then `Surface::fill_path` fills it.
- The site's `<text>` names a font stack only: `ui-monospace, SFMono-Regular, Menlo, Consolas, "DejaVu Sans Mono", monospace`. atlas sets `font-size = 0.98 ch` and uses `textLength`, kiosk uses `font-weight="700"`. Neither page embeds a font (no `@font-face`, base64 or woff in either page).

What real `<text>` would need:

- A hook: `Type::fill_centred` knows the character, centre, size and ink. A recorder could take those and write `<text>`, instead of the hinted outline it gets now through `fill_path`.
- Font: name `DejaVu Sans Mono` (with weight 700 for kiosk) so the file opens as editable type. Without an embedded font, a viewer may fall back to another monospace, and both glyph shapes and advances move.
- Embedding: an `@font-face` with a base64 font is about 330 KB per face, before subsetting. Subsetting to the used characters makes it small, but it is a font change (see licence).
- Licence: `fonts/LICENSE` (Bitstream Vera + Arev, DejaVu changes public domain). Copying and distributing is allowed, also inside a larger package. A modified font must be renamed so the name contains neither "Bitstream" nor "Vera". "DejaVu Sans Mono" already meets that. A font may not be sold on its own. Embedding it in an SVG we ship is allowed, and the notice must go with it.
- Fidelity: our PNG uses hinting and pixel snapping. An SVG `<text>` rendered elsewhere won't hint the same way. At 8 px glyphs (atlas at 540 px and 110 columns), hinting decides most pixels (see `meta/tools/atlas.md`). A `<text>` SVG will not pixel-match our PNG. Outline `<path>`s will match it much more closely.

## Open questions

- Fidelity vs editability for atlas and kiosk: outline paths match our PNG; `<text>` is editable but won't match. Is the target "our PNG", or the site's `<text>`, for these two? kiosk joins atlas here: the site writes `<text>` for both.
- Embed the font, or only name it? Embedding keeps glyphs stable across viewers. It adds a licence notice and up to about 330 KB per face.
- Does a test renderer (resvg, still to be approved) load an `@font-face` from a data URL? Not checked here. If not, `<text>` tests need the font in resvg's font database.
- carve: write the ramp panels as an embedded raster `<image>`, approximate them with a gradient as the site does (and so not match our PNG), or leave carve unsupported?
- Element counts for the Nearest-grid Tools (fete, quilt, static) depend on `res` and need run merging. They join stipple in the size-limit question. The map lists crowd and hiss as dense, but neither exports SVG on the site, so neither is in scope.
