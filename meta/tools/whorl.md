# whorl

Op-art stripes. A handful of centres each add their distance to a single field, some counted positive and some negative. The bands of equal total are rings round the centres, hyperbolas between them, and pinched eyes where two families meet. A warp bends the field before the bands are read off it. Every pixel is worked out at full size.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range                     | step | default | role                                  |
| --------- | ------------------------- | ---- | ------- | ------------------------------------- |
| `centres` | 1..14                     | 1    | 4       | how many centres                      |
| `pull`    | 0..1                      | 0.01 | 0.62    | how sharp each centre's cone is       |
| `push`    | 0..1                      | 0.01 | 0.35    | the share of centres counted negative |
| `stripes` | 2..90                     | 1    | 16      | bands per unit of field               |
| `weight`  | 0..1                      | 0.01 | 0.5     | how much of each band is ink          |
| `dirs`    | Smooth, Turbulent, Ripple |      | Smooth  | how the field is warped               |
| `warp`    | 0..1                      | 0.01 | 0.5     | how far it is warped                  |
| `detail`  | 0..1                      | 0.01 | 0.3     | how fine the warp is                  |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. whorl is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the flow and orbit are 0 and the breathe factor is 1. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

## Palette

Variable length, default 2 inks: `#003049 #f77f00`. Ink 1 is the ground. Every later ink is a stripe colour, taken in turn as the bands count up. The duotone is what the page is made for.

## Noise and draws

The chassis hash and value noise, on seeds `t + 11`, `t + 29`, `t + 53` and `t + 71`, with `t` the Tool seed read as signed 32-bit and the sums wrapped to 32 bits.

One draw stream, the chassis xorshift with logical shifts, started from `t·2654435761` as a 32-bit wrapping multiply, 0 becoming 1. For each of `n = max(1, centres)` centres, in order:

1. Only for centres after the first: one draw, the centre is negative when it is below `push`. The first centre is always positive and takes no draw.
2. `x = -0.25 + 1.5·draw`, then `y = -0.25 + 1.5·draw`.
3. Weight `w = ±(0.45 + 0.9·draw)`, signed by step 1.
4. One more draw, a motion phase, unused in a Still but drawn all the same.

## Field

Frame units: `x` runs 0..1 across the width and `y` runs 0..`H/W` down. A pixel `(px, py)` is read at its centre, `fx = (px + 0.5)/W` and `fy = (py + 0.5)/H·(H/W)`.

- `wamp = 0.42·warp`, `wfrq = 1.1 + 5.5·detail`, `soft² = (0.36·(1 - pull))²`.
- The warp, only when `wamp > 0.001`, starting from `(wx, wy) = (x, y)`:
  - Ripple: `wx += 0.3·wamp·sin(6·wfrq·y + 1.7·x)` and `wy += 0.3·wamp·sin(5.1·wfrq·x + 1.3·y)`, the products grouped left to right.
  - Smooth: `wx += wamp·(vn(wfrq·x, wfrq·y, t + 11) - 0.5)` and `wy += wamp·(vn(wfrq·x + 7.3, wfrq·y + 3.1, t + 29) - 0.5)`.
  - Turbulent: Smooth, then `wx += 0.45·wamp·(vn(2.7·wfrq·x + 2.2, 2.7·wfrq·y, t + 53) - 0.5)` and `wy += 0.45·wamp·(vn(2.7·wfrq·x, 2.7·wfrq·y + 5.5, t + 71) - 0.5)`.
- The field is `Σ w·sqrt((wx - cx)² + (wy - cy)² + soft²)` over the centres, in order.

## Bands

- `u = field·max(1, stripes)`.
- The edge width `g` is how far `u` moves per pixel: 0.0025, raised to `|u - left|` when there is a pixel to the left on this row, then to `|u - up|` when there is a row above. The row above is kept as 32-bit floats; the pixel to the left is not rounded.
- `v = u - floor(u)`, `duty = 0.06 + 0.88·weight`.
- Signed distance to the band edge: inside the ink part, `v < duty`, it is `min(v, duty - v)`; outside it is `-min(v - duty, 1 - v)`.
- Coverage `a = clamp(0.5 + sd/g, 0, 1)`.
- The band's ink is stripe ink `floor(u)` modulo their count, taken non-negative.
- Each channel is `ground + (ink - ground)·a`, stored as a byte, clamped and rounded half to even. Alpha is opaque.

## Fidelity notes

- Ripple calls `sin` per pixel. V8's result may differ in the last bit, which flips at most a byte on a band edge.
- The 32-bit row above only moves `g`, so a missed rounding shows as a soft edge one level off, not a moved band.
