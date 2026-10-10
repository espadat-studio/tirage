# dahlia

A chrysanthemum firework on a paper ground. Rays leave one centre, each a chain of short dashes in stacked inks, bowed a little and beaded with the accent ink.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range    | step | default | role                                   |
| -------- | -------- | ---- | ------- | -------------------------------------- |
| `rays`   | 8..300   | 1    | 160     | how many rays                          |
| `size`   | 0..1     | 0.01 | 0.85    | burst radius                           |
| `ragged` | 0..1     | 0.01 | 0.4     | how unevenly the rays end              |
| `cx`     | 0.1..0.9 | 0.01 | 0.5     | centre, share of the width             |
| `cy`     | 0.1..0.9 | 0.01 | 0.5     | centre, share of the height            |
| `width`  | 0..1     | 0.01 | 0.6     | dash width                             |
| `dash`   | 0..1     | 0.01 | 0.5     | dash length                            |
| `gap`    | 0..1     | 0.01 | 0.4     | gap between dashes, relative to a dash |
| `bend`   | 0..1     | 0.01 | 0.3     | how far each ray bows                  |
| `caps`   | 0..1     | 0.01 | 0.8     | chance a dash ends in a bead           |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. dahlia is a Still Tool: only frame 0 is rendered. At frame 0 every motion term is 0. Motion stays fixed at site defaults and is not a Parameter. Dither is the shared [chassis](chassis.md) post-pass, off by default.

Grain is the chassis pass too, but this page opens with it on: multiply, amount 0.22, size 1, specks 0.5, vignette 0.15.

The site deals the sliders from the seed on every seed change. That deal is not ported: `derive` deals every Parameter (ADR 0003), and a Reference export sets every slider after the seed.

dahlia is a palette-family Tool. Its Palette is the swatch row, read by role, any number of inks from 2. Default Palette, 5 inks: `#F6EEDC #3FD3F0 #FF6FD8 #FFF23A #F52A2A`.

- ink 0 is the paper
- the last ink is the accent, used for the beads
- the inks between are the ray inks. With 2 inks there are none between, and the rays use the accent

## Random numbers

`s` is the Tool seed. One xorshift32 stream with the logical right shift (the chassis stream), started at `(s * 2246822519 mod 2^32) xor 0x27d4eb2f`, a zero start becoming 1. It gives exactly one draw: the lean.

Everything else comes from the chassis integer hash `h(x, y, s)` (`chassis::hash`), with `x` the ray index and `y` a role number.

## Geometry

`U = sqrt(W H)`. With `n = max(8, rays)`:

- centre `(cx W, cy H)`
- burst radius `R = U (0.3 + 0.55 size)`
- base width `w0 = U (0.012 + 0.03 width)`
- dash length `D = U (0.03 + 0.07 dash)`, gap `G = D (0.15 + 1.1 gap)`, period `P = D + G`
- lean `L = (r - 0.5) 0.5`, the one stream draw

Ray `i`, with `h_k = h(i, k, s)`:

1. angle `a = (i + 0.5) / n 2π + (h_1 - 0.5) (2π / n) 1.6`
2. length `len = R (1 - 0.9 ragged h_2) (0.55 + 0.45 h_3)`
3. bow `b = (h_4 - 0.5 + 0.5 L) bend len 0.9`
4. a quadratic Bezier from the centre through `m = c + (cos a, sin a) len/2 + (-sin a, cos a) b` to `e = c + (cos a, sin a) len`

## Dashes

Along each ray, dash `k = 0, 1, …` starts at `s_k = h_5 P - P + k P` and the walk runs while `s_k < len`.

For each dash:

1. `t0 = max(0, s_k) / len`, length `dl = D (0.45 + 0.8 t0)`, `t1 = min(len, s_k + dl) / len`
2. skip unless `t1 > t0` and `t1 > 0`
3. width `wd = w0 (0.4 + t0)`
4. `hk = h(i, k + 11)`, `hk2 = h(i, k + 31)`. With `m` ray inks: first ink `0` when `hk < 0.55`, else `floor(hk m)`; layer count `min(m, 1 + floor(2.5 hk2))`
5. layer `l` uses ray ink `(first + l) mod m`, side offset `0, 0.32 wd, -0.28 wd` and line width `wd, 0.62 wd, 0.36 wd` for `l = 0, 1, 2`. It is a 5-point polyline at `t = t0 + (t1 - t0) q / 4`, `q = 0..=4`: the Bezier point moved by the offset along the left normal `(-dy, dx) / |d|` of the curve's derivative there. A zero derivative counts as length 1. Each layer is stroked on its own, round cap and join
6. bead: when `h(i, k + 51) < caps`, a filled accent circle at the Bezier point at `t1` with radius `wd 0.5 (0.8 + 0.5 h(i, k + 61))`. Then, when `h(i, k + 71) < 0.35 caps`, a second at `t0` with 0.8 of that radius

## Painting

1. Fill the frame with the paper.
2. Rays in index order, each dash's layers then its beads.
3. The chassis grain, then dither.

## Fidelity notes

- Every stroke is opaque, so the only alpha is the antialiased edge.
- The site picks 3:4 as its opening ratio; Reference exports set 9:16.
