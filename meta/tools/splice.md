# splice

A screen-printed collage. One field of soft hills is stepped into tones, then the sheet is cut into slices and each slice prints that same field shifted along, through a screen and an ink rotation of its own. A few long arcs are stroked over the top.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                 | step | default | role                                                 |
| -------- | --------------------- | ---- | ------- | ---------------------------------------------------- |
| `dirs`   | Columns, Rows, Blocks |      | Columns | how the sheet is cut                                 |
| `slices` | 1..40                 | 1    | 1       | how many pieces                                      |
| `shift`  | 0..1                  | 0.01 | 0.35    | how far each piece slides along the field            |
| `scale`  | 0..1                  | 0.01 | 0.5     | hill size                                            |
| `tones`  | 2..14                 | 1    | 5       | how many steps the field is cut into                 |
| `key`    | 0..1                  | 0.01 | 0.4     | width of the key-ink line where two steps meet       |
| `screen` | 0..1                  | 0.01 | 0.4     | screen cell size                                     |
| `mix`    | 0..1                  | 0.01 | 0.7     | chance a piece picks its own screen                  |
| `grain`  | 0..1                  | 0.01 | 0.18    | the Tool's own per-pixel grain, not the chassis pass |
| `arcs`   | 0..24                 | 1    | 5       | how many arcs                                        |
| `weight` | 0..1                  | 0.01 | 0.45    | arc line width                                       |
| `bend`   | 0..1                  | 0.01 | 0.55    | how round the arcs are                               |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. splice is a Still Tool: only frame 0 is rendered. Motion stays fixed at site defaults and is not a Parameter. `grain` is the Tool's own and stays at its site default, as mist's does. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

splice is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. Slot 0 is the key: it draws the step lines and also takes part in the ink rotation. Default Palette, 7 inks: `#1B1B1B #FF6B35 #004E89 #F7C59F #EFEFD0 #1A936F #FF3CAC`.

## Random numbers

`s` is the Tool seed. All random draws come from one xorshift32 stream (the chassis one: unsigned right shift), started at `s * 2654435761` wrapped to 32 bits, a zero start being 1. Each draw is the state over 2^32.

The field uses the chassis integer hash `h(x, y, seed)` and its value noise `vn(x, y, seed)` ([chassis](chassis.md)). Seed offsets below are added to `s` with 32-bit wrap.

## Cuts

`n = max(1, round(slices))`, rounding half up.

`splits(m)` draws `m` gaps `0.45 + r`, and returns `m + 1` cut points: 0, then the running sum of each of the first `m - 1` gaps over the gap total, then 1.

A piece draws, in order: `off = 2 (r - 0.5)`, `rot = floor(97 r)`, `sc = 0.6 + 1.5 r`, then one draw `r < mix`. Only when it passes, a second draw picks its screen `["dots", "lines", "noise", "flat"][floor(4 r)]`; otherwise the piece takes the base screen.

Draw order:

1. Base screen: `["dots", "lines", "noise"][floor(3 r)]`. Flat is never the base.
2. Pieces by cut:
   - Columns: `splits(n)` across, then `n` pieces left to right, each a full column.
   - Rows: `splits(n)` down, then `n` pieces top to bottom, each a full row.
   - Blocks: `nx = max(1, round(sqrt(1.3 n)))` columns from `splits(nx)`. Per column, left to right: `ny = max(1, round(n / nx (0.6 + 0.9 r)))`, then `splits(ny)` down, then `ny` pieces top to bottom.
3. Arcs, `round(arcs)` of them. Each draws, in order: `a0 = 2π r`, `cx = -0.3 + 1.6 r`, `cy = -0.3 + 1.6 r`, `rx = 0.35 + 1.1 r`, `ry = (0.35 + 1.1 r) (0.15 + 0.85 bend)`, `rot = π r`, `a1 = a0 + 0.9 + 2.6 r`, `k = floor(97 r)`.

## Field

`U = sqrt(W H)`, `fs = U (0.09 + 0.42 scale)`, `shift' = 0.55 U shift`, `cellB = U (0.004 + 0.022 screen)`, `kw = 0.085 key`, `T = max(2, round(tones))`, `gl = 54 grain`.

Per pixel `(x, y)`:

1. The column is the first whose right cut is past `(x + 0.5) / W`, else the last. In it, the piece is the first whose bottom cut is past `(y + 0.5) / H`.
2. Field position: Rows slides x, `sx = x + off shift'`, `sy = y`. Columns and Blocks slide y, `sx = x`, `sy = y + off shift'`.
3. `F = 0.45 vn(sx/fs, sy/fs, s+3) + 0.26 vn(sx/(0.42 fs), sy/(0.42 fs), s+17) + 0.18 vn(sx/(0.17 fs), sy/(0.17 fs), s+41) + 0.11 vn(sx/(0.065 fs), sy/(0.065 fs), s+83)`, clamped to `0..0.999999`.
4. `g = F T`, step `t = min(floor g, T - 1)`, coverage `cov = g - t`.
5. Key line: when `kw > 0.0005` and `min(cov, 1 - cov) < kw`, the pixel is ink 0.
6. Otherwise the screen decides `on`. `cs = max(1, cellB sc)`, cell `(cx, cy) = (floor(x/cs), floor(y/cs))`, from the pixel, not the slid position:
   - flat: `cov > 0.5`
   - dots: with `fx = x/cs - cx - 0.5`, `fy = y/cs - cy - 0.5`, `fx^2 + fy^2 < (0.72 sqrt(cov))^2`
   - lines: `y/cs - cy < cov`
   - noise: `h(cx, cy, s+7) < cov`

   The ink is slot `(t + on + rot) mod k` for `k` inks.
7. Own grain: when `gl > 0.002`, add `gl (h(x, y, s+71) - 0.5)` to each channel. Channels are clamped to 0..255 and stored rounding to nearest, ties to even.

## Arcs

Drawn when there is at least one arc and `weight > 0.004`, after the field, each its own path:

- ink slot `k mod k-inks`, at alpha 0.85, source-over
- width `max(0.9, U (0.0009 + 0.0042 weight))`, round caps
- canvas `ellipse(cx W, cy H, rx U, ry U, rot, a0, a1)`, clockwise

## Fidelity notes

- The arcs land mostly off the frame; only a sweep shows.
- The site leaves the line join at its miter default. An arc is one smooth curve, so the join never shows.
