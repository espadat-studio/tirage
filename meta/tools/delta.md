# delta

A satellite photo of a river mouth, cut up. A muddy marbled field covers the frame, and on top of it sit pixelated chunks of that field, loose mosaic patches and solid blocks in loud inks, full-width bands, and finally strips of the finished frame shoved sideways.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range   | step | default | role                                               |
| ---------- | ------- | ---- | ------- | -------------------------------------------------- |
| `scale`    | 1..9    | 0.1  | 3.2     | field noise frequency                              |
| `warp`     | 0..1.6  | 0.01 | 0.55    | how far the field bends its own lookup             |
| `contrast` | 0..1    | 0.01 | 0.6     | field contrast around mid grey                     |
| `bleach`   | 0..1    | 0.01 | 0.35    | how far the field ramp is pulled toward grey       |
| `patches`  | 0..18   | 1    | 6       | mosaic patches                                     |
| `cell`     | 8..90   | 1    | 28      | grid unit, per 1200 px of width                    |
| `fill`     | 0.05..1 | 0.01 | 0.55    | share of a patch's cells that get an ink           |
| `quant`    | 0..1    | 0.01 | 0.35    | pixelated field regions; off below 0.01            |
| `blocks`   | 0..10   | 1    | 3       | solid blocks                                       |
| `bands`    | 0..6    | 1    | 1       | full-width bands                                   |
| `slice`    | 0..1    | 0.01 | 0.25    | how many strips shift, and how far; off below 0.01 |

The defaults are the slider values the page shows. The script opens at its own values (`scale` 4.6, `warp` 0.85, `contrast` 0.55, `bleach` 0.3, `patches` 11, `cell` 18, `fill` 0.62, `quant` 0.45, `slice` 0.22) until a slider moves; a Reference export sets every slider, so only the shown values matter.

Motion (`modes` Shuffle or Slice, `amt`, `fps`, `frames`) is off by default and not part of a Still. delta is a Still Tool: only frame 0 is rendered, and with motion off the frame seed is the Tool seed and the slice jitter is 0. Motion stays at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The seed does not move any slider: a new seed only redraws.

delta is a palette-family Tool. Its Palette is the swatch row, any number of inks from 2, read by position for the marks and by brightness for the field. Default Palette, 6 inks: `#FF6F00 #0057FF #FFEA00 #00D68F #FF2E88 #2A2A2A`.

## Random numbers

`s` is the Tool seed.

The marks draw from one xorshift stream with the arithmetic `>> 17` ([warp](warp.md#draws)), started straight from `s` (0 becoming 1), with no multiplier. The slice draws from a second such stream started from `s + 555`, mod 2^32.

The field uses a value noise of its own. Its hash is not integer math: `n = x · 374761393 + y · 668265263 + k · 1442695040888963407`, summed left to right in doubles (the last constant rounds to 1442695040888963456). Then `a = int32(n)`, `m = int32((a ^ (a >> 13)) · 1274126177)` with the product again a double, and the hash is `uint32(m ^ (m >> 16)) / 2^32`. `int32` is JavaScript's ToInt32, the double taken mod 2^32. The noise is bilinear over the hash at the four corners with smoothstep weights. `fbm(x, y, k, oct)` sums `oct` octaves at weight `0.5^i`, frequency `2^i` and seed `k + 57 i`, divided by the weight total.

## Field

For a `W x H` frame with long edge `L = max(W, H)`: `res = clamp(round(L / 3), 120, 420)`, and the field is `fw = max(2, round(res W / L))` by `fh = max(2, round(res H / L))`.

The ramp: sort the Palette by luma `0.299 r + 0.587 g + 0.114 b`, stable, darkest first. `mix = 0.1 + 0.55 bleach`. The stops are `(38, 36, 46)`, then each sorted ink pulled toward the grey `70 + 150 luma / 255` by `mix` and rounded per channel, then `(236, 231, 220)`. `n` is the stop count minus 1.

For each field pixel `(x, y)`, with `d = 13 s`, `u = x / fh · scale`, `v = y / fh · scale` (both over `fh`):

1. `wx = u + warp · (fbm(1.7 u + 11, 1.7 v, d + 91, 3) - 0.5) · 2.4`
2. `wy = v + warp · (fbm(1.7 u, 1.7 v + 7, d + 37, 3) - 0.5) · 2.4`
3. `t = fbm(wx, wy, d, 6)`, ridge `g = 1 - |2 fbm(2.1 wx + 5, 2.1 wy - 3, d + 211, 4) - 1|`, `t = 0.74 t + 0.26 g²`
4. `t = clamp((t - 0.5)(0.5 + 1.6 contrast) + 0.5, 0, 1)`
5. `p = t n`, `i = min(n - 1, floor p)`, mix stops `i` and `i + 1` by `p - i`. Each channel is stored as a byte, rounding half to even as a canvas pixel buffer does.

## Painting

`unit = max(4, round(cell W / 1200))`, `cols = ceil(W / unit)`, `rows = ceil(H / unit)`. `round` is half up, as JavaScript's `Math.round`. A pick is `pal[floor(r · len)]` from the Palette. Every rect below has whole-pixel corners and is opaque unless stated.

1. The field drawn over the frame with bilinear smoothing.
2. Quant, when `quant > 0.01`: `round(2 + 9 quant)` regions. Each draws `px = max(2, round(unit (0.5 + 1.6 r)))`, `rw = round((3 + 11 r) px)`, `rh = round((3 + 9 r) px)`, `rx = round((r W - rw / 2) / px) px`, `ry` the same with `H` and `rh`. Squares of side `px` step from `(rx, ry)` while below `rx + rw` and `ry + rh`; one is skipped when it lies wholly off the frame (`x + px < 0`, `y + px < 0`, `x > W` or `y > H`). Each takes the field pixel at `floor((x + px / 2) / W · fw)`, `floor((y + px / 2) / H · fh)`, clamped to the field.
3. `patches` patches. Each draws: `fine = r < 0.38`; `u = max(3, round(unit · (0.42 if fine else 1)))`; `pw = round((3 + r (16 or 9)) u)`, `ph = round((3 + r (14 or 8)) u)`; `px = round((r W - pw / 2) / u) u`, `py` the same. Then a local palette: a copy of the Palette sorted with a comparator that returns `r - 0.5` (below), cut to its first `2 + floor(3 r)` inks. Density `fill · (0.85 if fine else 1)`. Each cell of side `u` from `(px, py)` across `pw` by `ph`, rows outer: one draw, skip the cell when it is above the density, else a second draw picks its ink from the local palette.
4. `blocks` blocks. Each draws `bw = round((4 + 12 r) unit)`, `bh = round((2 + 8 r) unit)`, `bx = round(r cols) unit`, `by = round(r rows) unit`, then a pick. The rect is `bx - round(bw / 2)`, `by - round(bh / 2)`, `bw x bh`.
5. `bands` bands. Each draws `bh = round((1 + 4 r) unit)`, `by = round(r rows) unit`, a pick, then alpha `0.85 + 0.15 r`. A full-width rect at `by`, `bh` tall, at that alpha.
6. Slice, when `slice > 0.01`, from the slice stream: `round(2 + 14 slice)` strips. Each draws `sh = round((0.4 + 2.2 r) unit)`, `sy = round(r rows) unit`, `dx = round((r - 0.5) · 10 slice · unit)`. A strip with `dx = 0` is skipped. Otherwise read the full-width rows from `sy` (clamped to `H - 1`), `min(sh, H - sy)` tall, from the frame as it is now, and write them back unblended at `x` offsets `dx` and `dx - W` (or `dx + W` when `dx < 0`), clipped to the frame. A strip with `sy > H` reads a flipped range above the frame's foot and writes off the frame, so it changes nothing. A strip with `sy = H` is a zero-height read, which throws on the site and ends its paint, so no export comes out. We end the slices there.
7. The chassis grain, then dither.

### The random comparator

The local palette goes through V8's sort with a comparator that ignores its arguments, so its order depends on V8's algorithm and the stream. Each comparison is one draw; below 0.5 means "less". Checked against V8 for 2 to 12 elements:

1. Up to 7 elements, no run: the sorted prefix starts as element 0.
2. From 8 elements, a run first. Compare element 1 with element 0; "less" makes the run descending. Then compare each next element with the previous one until a draw breaks the direction ("not less" for a descending run, "less" for an ascending one). A descending run is reversed.
3. Every element after the prefix is inserted by binary search: `left = 0`, `right = start`; while `left < right`, `mid = left + floor((right - left) / 2)`, "less" sets `right = mid`, else `left = mid + 1`. The element moves to `left`.

From 64 elements V8 also merges runs; a Palette that long is not ported.

## Fidelity notes

- The site picks 16:9 as its opening ratio; Reference exports set 9:16.
- The PNG export repaints at the export size, so the field, `unit` and the grid follow the export.
- Bilinear upscale and alpha bands blend in 8 bits on both sides; every other mark is a hard whole-pixel rect.
- Reference exports do not match. Seeds 1 to 3 sit 48 to 76% off, though every `fillRect`, strip read and strip write the site makes is the same as ours, in order. The headless Chromium the refs rig drives paints those same calls differently on its GPU canvas. The glitch goes away with `--js-flags=--no-maglev`, with `--jitless`, on a `willReadFrequently` canvas, or as soon as anything watches the paint: a wrapped canvas method, a debugger logpoint, or an extra readback before the slices. Every one of those exports matches ours at 0.00%. Each stage alone, and each pair of stages, also exports the same as ours; the glitch needs translucent bands followed by strip reads.
