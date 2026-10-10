# relief

Tumbling blocks. The frame is tiled with cubes seen from a corner, each a hexagon cut into three rhombi: top, right and left. Each cube's three faces are one ink at three lightnesses. A small unit of cubes is worked out once and repeated across the frame, and turning a cube's tones round is what breaks the plain field into interlocking hooks. Every face is a filled quadrilateral.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range   | step | default | role                                       |
| --------- | ------- | ---- | ------- | ------------------------------------------ |
| `scale`   | 12..180 | 1    | 89      | cube size in export pixels                 |
| `repeat`  | 1..12   | 1    | 3       | cubes per side of the repeating unit       |
| `variety` | 0..1    | 0.01 | 0.38    | how many ways a cube's tones may be turned |
| `solids`  | 0..1    | 0.01 | 0.14    | share of cubes painted as one flat tone    |
| `relief`  | 0..1    | 0.01 | 0.9     | how far apart the three face tones sit     |
| `accent`  | 0..1    | 0.01 | 0.34    | share of cubes painted in an accent ink    |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. relief is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off no cube is re-dealt, so every cube draws from the Tool seed. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

## Palette

Variable length, default 6 inks: `#1f1f1f #f4d35e #ee964b #f95738 #0d3b66 #faf0ca`. Ink 1 leads: every cube that is not an accent uses it. The other inks are the accents.

## Hash

The chassis integer hash of two cell coordinates and a seed, read as a value in [0, 1). The seed is the Tool seed plus a fixed offset, wrapped to 32 bits. For unit cell `(i, j)`, in order:

1. `rot = floor(hash(i, j, t + 11)·turns) mod 3`, with `turns = max(1, round(1 + 2·variety))`, rounded half up.
2. The cube is solid when `hash(i, j, t + 29) < solids`.
3. With at least one accent ink, the cube is an accent when `hash(i, j, t + 41) < accent`. Its ink is accent `floor(hash(i, j, t + 53)·accents)`. Otherwise its ink is ink 1.

Cells are visited row by row, `j` then `i`, over `repeat × repeat` cells. The order does not matter: no draw depends on another.

## Face tones

An ink is read as hue `h` in degrees, saturation `s` and lightness `l`, the usual HSL from 0..1 channels. Then, with `r = relief`:

- `base = clamp(l, 0.38, 0.62)`, `sat = clamp(s, 0.55, 1)`.
- Light tone: hue `h + 26r`, saturation `sat`, lightness `min(0.92, base + 0.22r)`.
- Mid tone: hue `h`, saturation `sat`, lightness `base`.
- Dark tone: hue `h - 22r`, saturation `min(1, 1.06·sat)`, lightness `max(0.16, base - 0.16r)`.

Back to RGB: hue wrapped into 0..360, saturation and lightness clamped to 0..1, then the usual HSL formula per channel, each scaled by 255 and rounded half up to a byte.

## Lattice

With `W` and `H` the frame size and `√3` in full double precision:

- `want = max(6, scale)`.
- `cubesX = max(1, round(W/(√3·want)))`, `cubesY = max(1, round(H/(1.5·want)))`, both rounded half up.
- `P = min(repeat, cubesX)` and `Q = min(repeat, cubesY)`, each at least 1.
- `cubesX` is raised to a multiple of `P`, `cubesY` to a multiple of `Q`. When `cubesY` is then odd, `Q` is added once, so the half-offset rows wrap.
- `sx = W/(√3·cubesX)`, `sy = H/(1.5·cubesY)`. The cube is stretched a little so a whole number of units lands exactly.

## Painting

The whole frame is first filled with the dark tone of unit cell `(0, 0)`.

Then for each row `j` from -1 to `cubesY + 1` and in each row each column `i` from -1 to `cubesX + 1`:

- The centre is `cx = (i + ½ on odd rows)·√3·sx`, `cy = 1.5·j·sy`. Odd means `j & 1`, so row -1 is odd.
- The unit cell is `(i mod P, j mod Q)`, both taken non-negative. Note it is indexed by `repeat`, not `P`, as the row stride: `cell = pj·repeat + pi`.
- Faces `k = 0, 1, 2` in order, with `hx = √3/2·sx` and `hy = sy/2`:
  - Top: `(cx, cy - sy)`, `(cx + hx, cy - hy)`, `(cx, cy)`, `(cx - hx, cy - hy)`.
  - Right: `(cx, cy)`, `(cx + hx, cy - hy)`, `(cx + hx, cy + hy)`, `(cx, cy + sy)`.
  - Left: `(cx, cy)`, `(cx - hx, cy - hy)`, `(cx - hx, cy + hy)`, `(cx, cy + sy)`.
- A face's tone is the mid tone on a solid cube. Otherwise it is tone `(k + rot) mod 3` of light, mid, dark.
- Each face is filled anti-aliased, then stroked 1 px wide in the same tone to close the hairline seams between neighbours. Later faces paint over earlier ones.

## Fidelity notes

- The cell index uses `repeat` as the row stride while the lattice wraps at `Q`. When `Q < repeat` that only picks different unit cells, which a port must copy.
- The site strokes with the canvas default miter join. Rhombus corners of 60° extend a miter 1 px past the vertex, a round join only half that. The difference is a few sub-pixel slivers at corners, under the same ink as the neighbours almost everywhere.
