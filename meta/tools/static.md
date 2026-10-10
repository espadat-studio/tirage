# static

A one-bit glitch poster. The frame splits into stacked bands, each running its own pattern engine, sometimes with an inset panel on top. A glitch pass then shifts, smears and punches the bits, and the grid is blown up with hard edges in two inks.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range   | step | default | role                                 |
| --------- | ------- | ---- | ------- | ------------------------------------ |
| `regions` | 2..5    | 1    | 3       | how many bands stack down the frame  |
| `res`     | 48..160 | 4    | 84      | grid columns across the frame        |
| `glitch`  | 0..1    | 0.01 | 0.5     | how many shifts, smears and dropouts |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. static is a Still Tool: only frame 0 is rendered. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The Palette is `[ink, ground]`: the page's ink field, then its background field, the order My colors fills them in. Default Palette: `#FF3CAC #1B1B2F`. The cap is 2. The colour inputs show other hexes in the page source until touched, but the art reads the page state, whose defaults are these. `#swapColors` is a button that swaps the two, not a Parameter.

## Random numbers

`s` is the Tool seed. Two xorshift32 streams with aura's signed right shift ([aura](aura.md)):

- the layout stream starts at aura's float product, as aura does
- the glitch stream starts straight at `s xor 0x9e3779b9`, 0 replaced by 1

The integer hash `h(x, y, c, seed)` is aura's four-input hash.

## Grid

`gw = res`, `gh = max(8, round(gw H / W))`. `round` is half up throughout.

## Layout

From the layout stream:

1. `n = regions` weights, each `0.5 + 1.2r`. `sum` is their total.
2. Walk the bands from `y = 0`. Band `i` ends at `min(gh, y + round(w_i / sum gh))`, the last band at `gh`. Each band is a region over the full width, made right after its end is known.
3. Inset: one draw. Under 0.4 there is an inset: `x0 = round(gw (0.1 + 0.25r))`, `x1 = round(gw (0.65 + 0.3r))`, `y0 = round(gh (0.15 + 0.3r))`, `y1 = round(gh (0.55 + 0.35r))`, then a region over that box.

A region takes these draws, in order:

| field   | draw                                                         |
| ------- | ------------------------------------------------------------ |
| type    | `[moire, bars, static, blocks, rings, zigzag][floor(6r)]`    |
| `a`     | `0.15 + 0.75r`                                               |
| `b`     | `0.06 + 0.4r`                                                |
| `warp`  | `6r`                                                         |
| `th`    | `0.8 (r - 0.5)`                                              |
| `cx`    | `(1.6r - 0.3) gw`                                            |
| `cy`    | `(1.6r - 0.3) gh`                                            |
| `rk`    | `0.12 + 0.4r`                                                |
| `ph`    | `2πr`                                                        |
| `dens`  | `0.2 + 0.5r`                                                 |
| `grad`  | `0.9 (r - 0.5)`                                              |
| `q`     | `2 + floor(5r)`                                              |
| `duty`  | `0.35 + 0.35r`                                               |
| `p`     | `5 + floor(10r)`                                             |
| `slope` | sign `1` under 0.5 else `-1`, times `0.3 + 1.2r` (two draws) |
| `vert`  | `r < 0.35`                                                   |
| `rs`    | `floor(1e9 r)`, the hash seed                                |

`cx` and `cy` use the whole grid even for the inset.

## Bits

Cell `(u, v)` belongs to the inset when it lies in `[x0, x1) x [y0, y1)`, else to the first band holding row `v`. A region with box `x0..x1` by `y0..y1` sets the bit:

- moire: `sin(a u + warp sin(b v) + ph) + sin(rk hypot(u - cx, v - cy)) > th`.
- rings: `sin(rk hypot(u - cx, v - cy) + ph) > th`.
- bars, horizontal: `uu = (u - x0) mod (x1 - x0)`. On when `h(0, v, 7) < 0.8` and `h(floor(uu / q), v, 1) < duty + 0.08 sin(0.15 u)`.
- bars, vertical: `vv = (v - y0) mod (y1 - y0)`. On when `h(u, 0, 7) < 0.8` and `h(u, floor(vv / q), 1) < duty + 0.1 sin(0.2 vv)`.
- static: `d = clamp(dens + 2 grad ((v - y0) / max(1, y1 - y0) - 0.5), 0.02, 0.95)`; on when `h(u, v, 0) < d`.
- blocks: on when `h(floor(u / 2q), floor(v / 2q), 0) < 0.5` differs from `h(floor(u / 5q), floor(v / 5q), 3) < 0.5`.
- zigzag: on when `(u + floor(slope v)) mod p < duty p`.

`mod` keeps the result non-negative.

## Glitch

From the glitch stream, `amt = glitch`:

1. `round(0.3 gh amt)` shifts. Each draws `y = floor(gh r)`, `x0 = floor(gw r)`, `len = floor(1 + 0.5 gw r)`, `dx = floor(1 + 7r)`. From a copy of row `y` taken first, cells `x0..min(gw, x0 + len)` take the copy's cell `(x + dx) mod gw`.
2. `round(0.08 gh amt)` smears. Each draws `y = 1 + floor((gh - 1) r)` and `reps = floor(1 + 3r)`, then copies row `y - 1` over rows `y .. y + reps`, stopping at the last row.
3. `round(6 amt)` dropouts. Each draws `x0 = floor(gw r)`, `y0 = floor(gh r)`, `w = floor(2 + 0.2 gw r)`, `h = floor(1 + 0.06 gh r)`, and a value, 0 under 0.5 else 1, then fills that box, clipped to the grid.

## Painting

Set bits are the ink, clear bits the ground. The grid is drawn over the frame with image smoothing off.

## Fidelity notes

- Every cell is a hard-edged block. `hypot` must give the same float as the site's for the moire and ring edges to land on the same cells; a differing last bit can flip a cell that sits on an edge.
