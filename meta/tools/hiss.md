# hiss

A stack of pointed leaves filled with television snow over a marbled colour field. Black blots break up the marble, comb stripes fringe two corners, checkered lenses sit on the seams between leaves, and small white asterisks are sprinkled on top.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range    | step | default | role                                            |
| ---------- | -------- | ---- | ------- | ----------------------------------------------- |
| `leaves`   | 1..5     | 1    | 3       | how many leaves in the stack                    |
| `size`     | 0.4..1.4 | 0.01 | 0.9     | leaf width                                      |
| `overlap`  | 0..0.5   | 0.01 | 0.15    | how far neighbouring leaves overlap             |
| `tilt`     | -1..1    | 0.01 | 0.2     | how far the leaves lean, alternating            |
| `shift`    | 0..1     | 0.01 | 0.5     | how far the leaves step off centre, alternating |
| `swirl`    | 0..1     | 0.01 | 0.6     | how hard the marble is warped                   |
| `black`    | 0..1     | 0.01 | 0.45    | how much of the marble is black                 |
| `scale`    | 0..1     | 0.01 | 0.5     | marble feature size                             |
| `checkers` | 0..4     | 1    | 2       | how many checkered lenses                       |
| `steps`    | 0..1     | 0.01 | 0.35    | stair-step size of every shape edge             |
| `comb`     | 0..1     | 0.01 | 0.4     | how far the corner comb stripes reach           |
| `marks`    | 0..24    | 1    | 8       | how many white asterisks                        |
| `flecks`   | 0..1     | 0.01 | 0.4     | share of snow grains printed in an ink          |
| `coarse`   | 0..1     | 0.01 | 0.15    | snow grain size                                 |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. hiss is a Still Tool: only frame 0 is rendered, and with motion off every motion term (boil, drift, breathe) is zero. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

hiss is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. Default Palette, 5 inks: `#FF1E8C #FF7A1A #2B4BFF #FFD400 #E8262B`.

The site deals every slider and the Palette from its seed when the seed changes (a "deal"). That deal is not ported: our `derive` replaces it (ADR 0003). Reference exports set the seed first and the Palette and every Parameter after; the export checks the swatch row holds the fixture Palette.

## Random numbers

`s` is the Tool seed, as a 32-bit integer; all products below wrap to 32 bits.

The hash and value noise are the [chassis](chassis.md) ones: `h(x, y, seed)` and value noise with smoothstep fade, corners from `h`.

Two xorshift32 streams, the chassis kind (unsigned shifts 13, 17, 5, draw = state / 2^32, a zero start becomes 1):

- the layout stream starts at `(s · 2246822519) xor 0x27d4eb2f`
- the mark stream starts at `(s · 3266489917) xor 0x165667b1`

Noise seeds: `sd1..sd4 = 7s + 1 .. 7s + 4`. The snow uses the fixed seeds 3 and 77, so the snow is the same for every Tool seed.

## Layout

`W x H` is the frame, `U = sqrt(W H)`. The frame is portrait or square here (`H >= W`), so the leaves point up and down.

`n = leaves`, `o = overlap`, `h = 1.32 H / (2 (n - o (n - 1)))`.

Leaf `i` in `0..n`, with `sign = -1` for odd `i` and `1` for even:

- `cy = -0.16 H + h + 2 i h (1 - o)`, `cx = W/2 + (shift - 0.5) 0.4 W sign`
- draw `sz = size (0.9 + 0.25 r)`
- draw `ang = tilt 0.25 sign (0.6 + 0.8 r)`
- half-length along the pointed axis `ha = h`, half-width `hb = W/2 sz`

Lens `k` in `0..checkers`:

- `k < n - 1`, on the seam below leaf `k`: `cy` is the mean of leaf `k`'s and `k + 1`'s `cy`. Draw `cx = W/2 + 0.2 W (r - 0.5)`, then `hl = h (0.14 + 0.1 r)`, then `wl = W (0.22 + 0.18 r)`.
- otherwise, inside leaf `L = (k - (n - 1)) mod n`: draw `cy = cy_L + 0.3 h (r - 0.5)`, then `cx = cx_L + 0.2 W (r - 0.5)`, then `hl = h (0.1 + 0.08 r)`, then `wl = W (0.12 + 0.12 r)`.

Then the comb inks: `combA = ink[floor(k r)]`, then `combB = ink[floor(k r)]`, `k` the ink count.

## Scales

- stair cell `cell = max(1, 0.04 steps U)` when `steps > 0`, else none
- marble cell `mc = max(1, 0.008 U)`, marble scale `sc = U (0.3 + 0.8 scale)`, `warp = 1.4 swirl`, `blackAmt = 0.85 black`
- comb pitch `sp = max(1, 0.007 U)`, reach `combR = 0.9 comb`
- checker cell `ch = max(2, 0.02 U)`
- snow cell `gsc = max(1, U / 640 (1 + 3 coarse))`, `fleck = 0.03 flecks`

## Pixels

For pixel `(x, y)`, the stepped point is `qx = (floor(x / cell) + 0.5) cell` (and `qy` alike), or `(x, y)` when there is no stair cell. Later layers win.

1. Marble. `mx = floor(x / mc) mc`, `my` alike, `p0 = mx / sc`, `p1 = my / sc`. With `bn = vn(0.8 p0 + 40, 0.8 p1 + 7, sd4)`, the pixel is black `(8, 8, 10)` when `2.2 (bn - 0.5) + 0.5 < blackAmt`. Otherwise:
   - `q1 = vn(2 p0 + 1.7, 2 p1 + 9.1, sd1)`, `q2 = vn(2 p0 + 5.3, 2 p1 + 2.2, sd2)`
   - `m = vn(2 p0 + 4 warp (q1 - 0.5), 2 p1 + 4 warp (q2 - 0.5), sd3)`
   - `p = floor(7.5 k m) / 6` (computed as `m k 1.25 6`), `j = floor p`, `f = p - j`; blend ink `j mod k` toward ink `j + 1 mod k` by `f^2 (3 - 2f)`
   - multiply by `0.55 + 0.45 q1`
2. Comb, when `combR > 0`. `ax = qx / W`, `ay = qy / H`, `si = floor(qx / sp)`, `sj = floor(qy / sp)`. If `ax - ay > 1 - combR + 0.14 (h(si, 0, s + 5) - 0.5)` the pixel is `combA` for even `si`, else `(8, 8, 10)`. Else if `ay - ax > 1 - combR + 0.14 (h(sj, 1, s + 6) - 0.5)` the same with `sj` and `combB`.
3. Leaves. For each leaf, `ex = qx - cx`, `ey = qy - cy`, rotated `rx = ex cos ang - ey sin ang`, `ry = ex sin ang + ey cos ang`. Along the axis `la = ry`, across `lb = rx`. The pixel is in the leaf when `|la| < ha` and `|lb| < hb (1 - t^1.5)^0.6667` with `t = |la| / ha`. In any leaf, it is snow: `gx = floor(x / gsc)`, `gy = floor(y / gsc)` on the unstepped pixel, grey `12 + 220 h(gx, gy, 3)^1.6`. When `h2 = h(gx, gy, 77) < fleck`, it is ink `floor(k h2 / fleck) mod k` instead.
4. Lenses, the first that holds. `la = qy - cy`, `lb = qx - cx`, in when `|lb| < wl` and `|la| < hl (1 - t^1.5)^0.6667` with `t = |lb| / wl`: grey 245 when `floor(qx / ch) + floor(qy / ch)` is even, else 10.

Channels clamp to 0..255 and round to the nearest, ties to even, as a clamped byte array stores them.

## Marks

From the mark stream, `marks` times, in order: `x = W (0.04 + 0.92 r)`, `y = H (0.04 + 0.92 r)`, `sz = 0.016 U (0.7 + 0.8 r)`, then `arms = 3` when `r < 0.5` else 4.

Each mark is drawn in `#F4F4F2`, source-over, opaque:

1. One path of `arms` segments through `(x, y)`: arm `a` at angle `π a / arms + π/2` from `(x, y) - sz (cos, sin)` to `(x, y) + sz (cos, sin)`. Stroked once, width `max(1, 0.2 sz)`, round caps.
2. `2 arms` filled dots: dot `a` at angle `2π a / (2 arms) + π/2`, centre `(x, y) + 1.05 sz (cos, sin)`, radius `max(0.8, 0.16 sz)`.

## Fidelity notes

- The pixel field is written straight into the frame with no smoothing: every pixel is computed.
- The marks are the only anti-aliased shapes.
