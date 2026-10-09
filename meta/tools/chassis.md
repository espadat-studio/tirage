# chassis

Grain and dither: two post-passes the site runs over a finished frame, on every Tool. Grain runs first, then dither, so a dithered picture is dithered grain and all. Both read and write straight RGBA, leave alpha alone, and are off by default.

Written from reading the sonar page to learn the algorithm. No site code is copied.

## Shared maths

- The integer hash and value noise are the ones [sonar](sonar.md#integer-hash) describes. They live in the chassis, and sonar uses them from there.
- Rounding a float to a stored byte clamps to 0..255 and rounds half to even, as a `Uint8ClampedArray` store does.
- Elsewhere rounding goes half up, as `Math.round` does.

## Grain

Inputs, by site control id:

| id            | range                                      | step | default |
| ------------- | ------------------------------------------ | ---- | ------- |
| `grainTog`    | toggle                                     |      | off     |
| `grnBlends`   | Add, Overlay, Soft light, Multiply, Screen |      | Add     |
| `grnAmount`   | 0..1                                       | 0.05 | 0.55    |
| `grnSize`     | 0.5..4                                     | 0.1  | 1       |
| `grnSpecks`   | 0..1                                       | 0.05 | 0.5     |
| `grnVignette` | 0..1                                       | 0.05 | 0.5     |

Nothing happens when it is off or the amount is 0.

Sizes, for a `W x H` frame:

- The grain cell is `max(1, sqrt(W * H) / 640 * size)` px. A pixel's grain cell is its coordinate divided by that, truncated.
- The clump scale is 11 grain cells.
- Specks cut at `specks * 0.035`. The vignette strength is `vignette * 0.35`.

Clumps are value noise at seed 41. They are sampled once per grain row, at `(gx * cell / clump, y / clump)` for each grain column `gx` in `0..ceil(W / cell) + 1`, where `y` is the first pixel row of that grain row. Each sample is kept as a 32-bit float.

Per pixel, row by row:

1. `h` is the hash of the pixel's grain cell at seed 3.
2. The grain is `k = (h - 0.5) * 120 * (0.5 + clump)`, with the clump of the pixel's grain column.
3. The blend gives a target per channel `c`. Add is `c + k`. The others read the grain as a grey layer `n = (128 + k) / 255`:
   - Multiply: `c * n`
   - Screen: `255 - (255 - c) * (1 - n)`
   - Overlay: `2 * c * n` when `c < 128`, else `255 - 2 * (255 - c) * (1 - n)`
   - Soft light: `(1 - 2n) * c * c / 255 + 2n * c`
4. Each channel moves toward its target by `amount`.
5. A speck: when `h > 1 - speck cut`, add `95 * amount` to each channel.
6. The vignette: `e` is the larger of `|x / W - 0.5| * 2` and `|y / H - 0.5| * 2`. Each channel is scaled by `1 - strength * amount * e³`.
7. Store as a byte.

All of it stays in 64-bit floats until the store, in the order above.

## Dither

Inputs, by site control id:

| id          | range                   | step | default |
| ----------- | ----------------------- | ---- | ------- |
| `ditherTog` | toggle                  |      | off     |
| `dthKinds`  | Bayer 8, Bayer 4, Noise |      | Bayer 8 |
| `dthSize`   | 1..10                   | 1    | 2       |
| `dthLevels` | 2..8                    | 1    | 3       |
| `dthAmount` | 0..1                    | 0.05 | 1       |

Nothing happens when it is off or the amount is 0.

- The block side is `max(1, round(size * min(W, H) / 700))` px. Blocks tile from the top left, and the last row and column are cut short by the frame.
- There are `levels - 1` steps per channel, at least 1.
- The matrix is an `n x n` Bayer matrix: 4 for Bayer 4, 8 for the others. It grows from a 1x1 zero by quadrants: each value is multiplied by 4, then 0 is added top left, 2 top right, 3 bottom left and 1 bottom right. Its values run `0..n²`.

Each block, row by row:

1. Average each channel over the block's pixels, truncated to an integer.
2. Pick the block's matrix value. Bayer reads the matrix at row `by mod n`, column `bx mod n`, for block coordinates `(bx, by)`. Noise reads `min(n² - 1, floor(noise * n²))` with `n = 8`.
3. The threshold is `(value + 0.5) / n²`. A channel average `v` scales to `q = v / 255 * steps`. It rounds up a step when `q - floor(q)` is above the threshold, capped at `steps`. The result is mapped back with `round(step / steps * 255)`.
4. At amount 1, every pixel in the block takes that colour. Below 1, each pixel channel moves toward it by `amount` and is stored as a byte.

### Noise hash

The noise is an unseeded hash of the block coordinates, and unlike the integer hash its multiply is a float multiply:

1. `a = x * 374761393 + y * 668265263`, exact, then wrapped to signed 32 bits.
2. `a ^ (a >> 13)`, arithmetic shift, then multiplied by `1274126177` as a 64-bit float. The product loses low bits.
3. Wrap the product to signed 32 bits as `ToInt32` does, xor in itself shifted right by 16 (arithmetic), read as unsigned and divide by 2^32.
