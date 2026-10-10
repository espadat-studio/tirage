# parcel

A survey map. A coarse two-tone block field covers the sheet, and a few hairline grids float over it, each a ragged cluster of cells drawn as one-pixel lines.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range    | step | default    | role                                    |
| -------- | -------- | ---- | ---------- | --------------------------------------- |
| `cells`  | 8..28    | 1    | 16         | grid columns across the sheet           |
| `cover`  | 0.2..0.8 | 0.01 | 0.5        | share of the cells the ink blocks cover |
| `chunk`  | 0.5..2   | 0.05 | 1          | block size: higher makes bigger blobs   |
| `grids`  | 0..8     | 1    | 4          | how many hairline grid clusters         |
| `blends` | picker   |      | `Multiply` | line blend: `Multiply` or `Normal`      |

`blends` is a picker, so it is a Choice Parameter that `derive` leaves at its site default.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. parcel is a Still Tool: only frame 0 is rendered, and at frame 0 both motion steps are 0. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, both off by default.

The page does not deal sliders from the seed. A seed change only moves the noise and the clusters.

parcel is fixed-role: its Palette is exactly 3 inks, read by role, and `MAX_INKS` is 3. Default Palette, the site's opening set: `#E9F5DB #E63946 #2B2D42`.

- ink 0 is the base ground
- ink 1 is the block ink
- ink 2 is the line ink

## Random numbers

`s` is the Tool seed.

- One xorshift32 stream, the one aura uses (`aura::Xorshift`): started at `s × 2654435761` worked in floating point and taken mod 2^32, a zero start becoming 1, with an arithmetic right shift by 17. It gives three draws in this order: `ox = 43 r`, `oy = 37 r`, then `SW = floor(1e9 r)`.
- A noise seed `NS = s xor 0x51ed270b`.
- The 4-argument integer hash `h(x, y, z, seed)` that aura uses (`aura::hash`).

## Noise

Value noise `vn(x, y, k)`: hash the four lattice corners with `h(xi, yi, k, NS)`, ease the fractions with `t² (3 - 2t)`, and blend bilinearly. `fbm(x, y, k) = 0.62 vn(x, y, k) + 0.38 vn(2.17 x, 2.17 y, k + 9)`.

## Grid

The sheet is `gw = cells` columns by `gh = max(4, round(gw H / W))` rows, each cell `cw = W / gw` by `ch = H / gh`.

## Blocks

With the motion step at 0, the noise origin is `(ox + 0.9, oy)`. The scale is `sc = 0.2 / chunk` and the threshold `th = 0.5 + (0.5 - cover) 0.55`. Cell `(u, v)` is inked when `fbm(u sc + 0.9 + ox, v sc + oy, 5) > th`.

## Line clusters

For cluster `i = 0..grids`, with `h_k = h(i, k, 0, SW)`:

1. width `w = 2 + floor(6 h_1)`, height `h = 2 + floor(5 h_2)` in cells
2. corner `cx = round(h_3 max(0, gw - w))`, `cy = round(h_4 max(0, gh - h))`
3. cell `(a, b)` of the `w × h` box is kept when `h(a, b, 17 i + 7, SW) < 0.78`; it sits at `(cx + a, cy + b)`

Every kept cell adds its four border edges to a set: top and bottom as horizontal unit edges, left and right as vertical ones. A shared border is only listed once. Along each grid line the unit edges are merged into maximal runs of consecutive edges.

## Painting

1. Fill the frame with the base.
2. Each grid row, left to right: every run of inked cells is one rect from `round(start cw)` to `round(end cw)` across and `round(v ch)` to `round((v + 1) ch)` down, filled with the block ink.
3. All line runs form one path. A run from grid point `(x0, y0)` to `(x1, y1)` goes from `(round(x0 cw) + 0.5, round(y0 ch) + 0.5)` to the same for its end. The path is stroked once, 1 px wide, butt caps, miter joins, in the line ink. With `blends` at `Multiply` the stroke is multiplied onto what is under it; with `Normal` it is drawn over.
4. The chassis grain, then dither.

## Fidelity notes

- Every fill is on whole pixels, so only the hairlines are antialiased. The +0.5 offset centres each line on one pixel row or column, and a butt cap leaves half a pixel of coverage at each end.
- The lines are one stroke, so where two runs cross the multiply applies once.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
