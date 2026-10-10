# cipher

A coded sheet. A grid of small glyphs on a dark ground: squares, diamonds, ringed squares and x's. Each cell takes the glyph and ink of the band of a field it sits in, so the bands read as a figure, a contour relief or a slope drawn in code.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                 | step | default | role                                           |
| -------- | --------------------- | ---- | ------- | ---------------------------------------------- |
| `cells`  | 20..140               | 1    | 80      | grid columns across the frame                  |
| `bands`  | 2..7                  | 1    | 5       | how many bands the field is cut into           |
| `fields` | Figure, Relief, Slope |      | Figure  | which field the bands are cut from             |
| `wave`   | 0..1                  | 0.01 | 0.6     | how far the field swells and breaks            |
| `tilt`   | -1..1                 | 0.01 | 0.2     | lean of the field                              |
| `detail` | 0..1                  | 0.01 | 0.5     | noise frequency of the field                   |
| `size`   | 0.3..1                | 0.01 | 0.8     | glyph size as a share of the cell              |
| `edges`  | 0..1                  | 0.01 | 0.6     | chance a cell on a band edge becomes an x      |
| `flecks` | 0..0.3                | 0.01 | 0.06    | chance a cell is thrown as a square in any ink |
| `smears` | 0..0.6                | 0.01 | 0.15    | chance a row carries a smeared streak          |
| `dots`   | 0..1                  | 0.01 | 0.6     | ground dot size                                |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. cipher is a Still Tool: only frame 0 is rendered. Motion stays fixed at site defaults and is not a Parameter. At frame 0 with motion off the drift offsets, the flicker offset and the breathe swell are all 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The page deals every slider from the Tool seed when the seed changes. That deal is not ported: `derive` deals each Parameter ([ADR 0003](../adr/0003-seed-derivation-of-recipes.md)), and a Reference export sets every slider after the page loads.

cipher is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. Ink 0 is the ground; inks 1 on are the marks. Default Palette, 7 inks: `#2A0A14 #DCE0F0 #F5D20A #7CC4F0 #F58A1E #E8412C #2E6BC0`.

## Random numbers

`s` is the Tool seed read as a signed 32-bit integer. All products and sums below wrap to 32 bits.

- The band stream is the chassis xorshift32 with unsigned shifts (13 left, 17 right, 5 left), a zero start taken as 1, the draw the state over 2^32. It starts at `s * 2246822519 xor 0x27d4eb2f`.
- `h(x, y, seed)` is the chassis integer hash and `vn(x, y, seed)` the chassis value noise on it (smoothstep fade, lerp along x then y).
- The field seeds are `sd1 = 7s + 1`, `sd2 = 7s + 2`, `sd3 = 7s + 3`.

## Inks

- `ground` is ink 0. `marks` are inks 1 onwards.
- `dim` is the ground moved 22% of the way to white, per channel, rounded half up.

## Bands

`N = bands`. From the band stream, for band `b = 0..N`, in order:

1. Glyph `g = floor(4r)`, one of square, diamond, ring, x. If it equals the previous band's glyph, `g = (g + 1) mod 4`.
2. Ink `k = floor(r * marks.len)`. If it equals the previous band's ink index, `k = (k + 1) mod marks.len`.

The first band has no previous. Then three more draws, taken whatever the field:

3. `spine = 0.5 + 0.2 (r - 0.5)`
4. `fy2 = 0.55 + 0.3r`
5. `fx2 = 0.3 + 0.4r`

## Field

`det = 1 + 3 detail`. The field gives a value in 0..1 at `(u, v)`, `u` across and `v` down the frame, each 0..1: 0 at the heart, 1 at the ground.

- Figure: two shapes, the nearer wins.
  - A strip about a leaning spine. Half-width `wv = 0.16 + 0.5 wave (0.3 + vn(1.6 det v + 3, 0.5, sd1) + 0.35 vn(4 det v + 9, 1.5, sd1))`. Centre `spine + tilt (v - 0.5)`. `s1 = |u - centre| / wv`.
  - A blob. `wv2 = 0.12 + 0.36 wave (0.4 + vn(1.9 det v + 13, 2.5, sd1))`. `s2 = 0.9 sqrt(((u - fx2) / wv2)^2 + ((v - fy2) / (1.6 wv2))^2)`.
  - Value `min(1, s1, s2)`.
- Relief: `w = v + tilt u`. `n = 0.65 vn(1.3 det u + 5, 1.3 det w + 2, sd2) + 0.35 vn(3.1 det u + 8, 3.1 det w + 4, sd2)`. Value `clamp((n - 0.2) / 0.6 (0.6 + 0.6 wave), 0, 1)`.
- Slope: `v + tilt (u - 0.5) + 0.7 wave (vn(1.5 det u + 7, 0.5, sd3) - 0.5) + 0.16 wave (vn(6 det u + 3, 1.5, sd3) - 0.5)`, clamped to 0..1.

## Grid

`cols = cells`, `cs = W / cols` px a cell, `rows = ceil(H / cs)`, glyph size `g = cs * size`. Cell `(i, j)` has centre `((i + 0.5) cs, (j + 0.5) cs)` and band `min(N - 1, floor(N * field((i + 0.5) / cols, (j + 0.5) / rows)))`. Band `N - 1` is the ground.

Smears, per row `j`: when `h(j, 1, s) < smears`, the row's run starts at column `floor(h(j, 2, s) cols)` and is `3 + floor(h(j, 3, s) min(14, 0.2 cols))` cells long. Otherwise the row has no run.

## Painting

Fill the frame with the ground. Then each cell, row by row, left to right, with `hc = h(i, j, s)` and `fleck ink = marks[floor(h(i, j + 7, s) marks.len)]`:

- Ground cell:
  - When `dots > 0` and `hc > 0.4 flecks`: a `dim` disc, radius `max(0.3, 0.09 cs dots)`.
  - Else when `flecks > 0` (so `hc <= 0.4 flecks`): a square of side `0.6g` in the fleck ink.
  - Nothing more.
- Any other cell starts with its band's glyph and ink:
  - When `hc < flecks`, it becomes a square in the fleck ink.
  - Else, when a 4-neighbour (right, left, down, up; off the grid counts as the same band) is in another band and `h(i, j + 3, s) < edges`, it becomes an x. Its ink is `marks[(p + 1 + floor(h(i, j + 11, s) (marks.len - 1))) mod marks.len]`, where `p` is the first band whose ink equals this band's ink. `p` is a band index used as a marks index; that is the site's rule.
  - When the cell is inside its row's smear run, it is drawn as a streak instead: a box from `x - cs/2`, `y - 0.32g`, `cs + 0.5` wide and `0.64g` tall, in the cell's ink.
  - Otherwise the glyph, centred on the cell:
    - square: side `g`.
    - diamond: a filled quad through `(x, y - d)`, `(x + 0.78d, y)`, `(x, y + d)`, `(x - 0.78d, y)`, `d = 0.55g`.
    - ring: the square, then a ground circle of radius `0.3g` stroked `max(0.6, 0.1g)` wide, then a filled ground disc of radius `0.1g`.
    - x: two diagonals from `(x ∓ 0.36g, y - 0.36g)` to `(x ± 0.36g, y + 0.36g)`, one stroked path `max(0.6, 0.16g)` wide.

Strokes have butt caps and miter joins. Every shape is anti-aliased; boxes land off the pixel grid.

The site also has a dot glyph, but no band can pick it, so it never draws.
