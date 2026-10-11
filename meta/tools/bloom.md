# bloom

Pixel rings. A grid of square cells is coloured by its distance from the frame centre, cut into bands that step through the Palette. Two noise fields push the rings out of round, a third roughens the band edges, and a quiet disc in the middle stays on the first ink. The grid is mirrored on both axes, so the four quadrants are one picture read four times.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id      | range | step | default | role                                     |
| ------- | ----- | ---- | ------- | ---------------------------------------- |
| `cols`  | 8..96 | 1    | 40      | cells across the frame                   |
| `rings` | 1..14 | 1    | 7       | bands from the quiet disc to the corners |
| `warp`  | 0..1  | 0.01 | 0.5     | how far the rings are pushed out of true |
| `grain` | 0..1  | 0.01 | 0.3     | how rough the band edges are             |
| `steps` | 2..14 | 1    | 6       | levels before the ramp repeats           |
| `calm`  | 0..1  | 0.01 | 0.35    | radius of the quiet disc                 |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. With motion off the ripple march is 0 and the breathe swell is 1. The Loop turns motion on at site defaults, and motion is not a Parameter. See [Motion](#motion). Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. The `grain` slider is bloom's own and has nothing to do with the chassis grain pass. It is grain, not art, so a Seed leaves it at the site default 0.3 and deals `cols`, `rings`, `warp`, `steps` and `calm`.

## Palette

Variable length, default 6 inks: `#1b2a49 #2e5c8a #3f9bc4 #7fd1d8 #beefdc #f3fbeb`. It is a ramp: ink 1 is the deepest band and the last ink the palest.

A level `k` maps to ink `floor(((k mod steps) / steps)·n)` of the `n` inks, with `k mod steps` taken non-negative. Every ink is reached whatever `steps` is, and a level past `steps` wraps back to ink 1.

## Noise

No draw stream. All randomness is the chassis hash and value noise, and fbm over it: octave `i` adds `131·i` to the seed, doubles the frequency and halves the weight, starting at weight 0.5, and the sum is divided by the total weight. Seeds are the Tool seed plus a constant, wrapped to 32 bits.

## Grid

- `cols' = max(4, cols)`, cell width `cw = W / cols'`.
- `rows = max(3, round(H / cw))`.
- Cell edges are mirrored: edge `k` of `n` is `round(min(k, n - k)·total/n)` when `k ≤ n - k`, else `total` minus that. So a column and its opposite are the same width to the pixel.
- Cells are visited row by row, left to right. Each is a whole-pixel rect from edge `i` to edge `i + 1` on each axis, filled in its ink. No ground is painted first; the cells tile the frame.

## Field

For cell `(i, j)`, the mirrored index is `mi = min(i, cols' - 1 - i)` and `mj = min(j, rows - 1 - j)`. Everything below reads only `mi` and `mj`.

- `u = (mi + 0.5)/cols'`, `v = (mj + 0.5)/rows`, `a = W/H`.
- `r = hypot((0.5 - u)·a, 0.5 - v) / hypot(0.5·a, 0.5)`, so 0 at the centre and 1 at a corner.
- `slow = fbm(3.2u + 3.1, 3.2v + 7.7, t + 11, 3 octaves) - 0.5`.
- `fast = fbm(9.5u + 13.7, 9.5v + 2.3, t + 17, 2 octaves) - 0.5`.
- `r += (1.35·slow + 0.5·fast)·warp`.

## Level

- The quiet radius is `c = 0.55·calm`.
- When `r ≤ c` the level is 0.
- Otherwise `g = (fbm(21u + 1.3, 21v + 9.1, t + 29, 2 octaves) - 0.5)·grain·2.4` and the level is `(r - c)/(1 - c)·max(1, rings) + g`.
- The cell's ink is the ramp at `floor(level)`. A negative level wraps to the top of the ramp.

## Motion

The site has two motion modes: Ripple and Breathe. Ripple is the default and the only one ported: amount 0.6, 24 frames at 8 fps.

- Frame `f` sits at `T = f / frames`.
- The ramp turns `max(1, round(2 * amount))` times per Loop, so once at the default. The amount sets the number of turns and nothing else.
- Ripple adds `march = T * steps * turns` to the level of every cell outside the calm centre, after the grain term. Bands step outward through the ramp a level at a time, in whole cells.
- Nothing else moves: the grid, both warp fields, the grain field and the calm centre hold. Calm cells stay at level 0.
- The Loop closes by whole periods: at `T = 1` the march is `steps * turns`, a whole number of ramp turns, so every cell lands on its frame 0 ink. Float addition could tip a level sitting a hair under a whole number, which the closure test would catch.
- At `f = 0` the march is 0, so frame 0 is the Still, byte for byte.
- The mid frame is 12: half a turn of the ramp, the frame furthest from frame 0.
- Breathe is not ported. It swells the calm centre on a sine and is not reachable from a Recipe.

## Fidelity notes

- Every cell is an axis-aligned rect on whole pixels, filled without anti-aliasing. A port can match the export byte for byte.
- `hypot` may differ from V8's in the last bit, which can flip a cell that sits exactly on a band edge.
