# terrain

A warped noise field cut into flat bands, one band per swatch, like a contour map with no lines. Grain shakes each pixel across band edges, and an optional block size coarsens the map into square cells.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range     | step | default | role                                         |
| ---------- | --------- | ---- | ------- | -------------------------------------------- |
| `scale`    | 0.6..9    | 0.1  | 2.6     | how many features fit across the frame       |
| `warp`     | 0..3      | 0.02 | 1.1     | how far the field is pushed by its warp      |
| `oct`      | 1..7      | 1    | 5       | octaves in the field                         |
| `contrast` | 0.5..4    | 0.05 | 1.7     | how hard the field is pushed to its extremes |
| `balance`  | -1.2..1.2 | 0.05 | 0       | leans the bands toward later or earlier inks |
| `grain`    | 0..1.2    | 0.01 | 0.3     | the Tool's own per-pixel noise, not chassis  |
| `block`    | 0..14     | 1    | 0       | cell size in px; 0 reads every pixel         |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. terrain is a Still Tool: only frame 0 is rendered, and with motion off the field has no time axis, so it is 2D. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. The `grain` slider above is separate from the chassis grain.

terrain is the first palette-family Tool. Its Palette is the swatch row only, read by position, any number of inks from 2. It has no ground, rule or ink fields of its own. The site's family picker only loads swatches and is not a Parameter. Default Palette, 6 inks: `#264653 #2a9d8f #8ab17d #e9c46a #f4a261 #e76f51`.

A Seed deals `scale`, `warp`, `oct`, `contrast`, `balance` and `block`. `grain` stays at 0.3, the site default, like the chassis passes.

## Hash and noise

terrain has its own integer hash of `(x, y, z, w, seed)`, all wrapping 32-bit:

1. `n = x 374761393 ^ y 668265263 ^ z 1440662683 ^ w 1274126177 ^ seed 1013904223`.
2. `n = (n ^ (n >> 15)) 2246822519`, then `n = (n ^ (n >> 13)) 3266489917`, then `n ^= n >> 16`, with logical shifts.
3. The draw is `n / 2^32`.

Value noise at `(x, y)` takes the integer corners at `z = w = 0`, smoothsteps the fractions with `t² (3 - 2t)`, and lerps the top pair, the bottom pair, then the two rows, each lerp `a + (b - a) t`.

fbm at a seed and octave count: weight starts at 0.5 and frequency at 1. Octave `i` reads noise at the frequency times the point, at `seed + 1319 i`. Each octave halves the weight and doubles the frequency. The sum is divided by the total weight.

## Field

With `W x H` the frame, the field is sampled every 5 px on a grid `ceil(W / 5) + 2` wide and `ceil(H / 5) + 2` tall, stored as 32-bit floats.

Cell `(i, j)` for Tool seed `s`:

1. `u = 5 i / W scale` and `v = 5 j / H scale (H / W)`.
2. Warps: `wx = fbm(u + 5.2, v + 1.3)` at `s + 11`, and `wy = fbm(u + 9.1, v + 7.7)` at `s + 29`, both 2 octaves.
3. `val = fbm(u + 2 warp (wx - 0.5), v + 2 warp (wy - 0.5))` at `s`, `floor(oct)` octaves.
4. Store `clamp((val - 0.5) contrast + 0.5, 0, 1)`.

## Painting

Every pixel is written directly. No shape is rasterised.

- The power is `2^-balance`. `n` is the ink count. With a block size `b`, the read point is `(floor(x / b) b, floor(y / b) b)`, else `(x, y)`.
- The field value is a bilinear read at the read point over 5. The cell is the truncated coordinate and the fractions are what is left over.
- Add `(hash(x, y, 0, 1, s) - 0.5) grain`, then clamp to 0..1.
- The pixel is ink `clamp(floor(val^power n), 0, n - 1)`, alpha 255.

## Fidelity notes

- Output is flat inks, so a diff is a pixel whose band flipped. `pow` and the 32-bit field store can move a pixel sitting on a band edge, a rare event.
