# pane

Stained glass of colour ramps. The frame is cut into rows of uneven height, each row into its own count of uneven cells, and every cell is a linear gradient between two inks of the Palette, running along an edge or a corner diagonal of the cell.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range | step | default | role                                           |
| -------- | ----- | ---- | ------- | ---------------------------------------------- |
| `rows`   | 1..10 | 1    | 3       | how many rows                                  |
| `cells`  | 1..16 | 1    | 6       | mean cells per row                             |
| `vary`   | 0..1  | 0.01 | 0.55    | how uneven row heights and cell widths are     |
| `diag`   | 0..1  | 0.01 | 0.45    | chance a cell's ramp runs corner to corner     |
| `soft`   | 0..1  | 0.01 | 0.85    | how much of a cell is blend rather than flat   |
| `spread` | 0..1  | 0.01 | 0.55    | how far apart in the Palette a cell's inks sit |

Motion (`modes` Shuffle or Turn, `amt`, `fps` 8, `frames` 24) is off by default and not part of a Still. pane is a Still Tool: only frame 0 is rendered, and at frame 0 with motion off the cell seed is the Tool seed and the direction turn is 0. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, both off by default.

The seed does not move any slider: a new seed only redraws.

pane is a palette-family Tool. Its Palette is the swatch row, read by position, any number of inks from 2. Default Palette, 8 inks: `#22223B #4A4E69 #9A8C98 #C9ADA7 #F2E9E4 #FF7B54 #FFB26B #FFD56F`. No ink has a fixed role; every cell picks two.

## Random numbers

`s` is the Tool seed. There is no stream. Every draw is the chassis integer hash `h(x, y, k)` (`chassis::hash`), with the seed slot `k` the Tool seed plus a salt, added mod 2^32:

| draw                   | `x` | `y`        | `k`            |
| ---------------------- | --- | ---------- | -------------- |
| row weight `j`         | `j` | 3          | `s + 3`        |
| cell count of row `j`  | `j` | 5          | `s + 7`        |
| cell weight `i`, row j | `i` | `11 + 7 j` | `s + 11 + 7 j` |
| diagonal or flat       | `i` | `j`        | `s + 13`       |
| direction member       | `i` | `j`        | `s + 17`       |
| first ink              | `i` | `j`        | `s + 19`       |
| ink step               | `i` | `j`        | `s + 23`       |
| start of the ramp      | `i` | `j`        | `s + 29`       |
| end of the ramp        | `i` | `j`        | `s + 31`       |

## Layout

Weights for `n` slots with salt `q`: `v_i = 1 + (2 h(i, q, s + q) - 1) 0.85 vary`, each divided by their sum.

Rows: `R = max(1, rows)` slots with salt 3. A running share `y` starts at 0. Row `j` spans `y0 = round(y H)` to `y1 = round((y + r_j) H)`, or `H` for the last row, then `y += r_j`.

Cells of row `j`: `C = max(1, round(cells (0.55 + 0.9 h(j, 5, s + 7))))` slots with salt `11 + 7 j`, laid out across the width the same way with a running share `x` and the last cell ending at `W`.

`round` is half up, as JavaScript's `Math.round`. A cell of zero width or height is skipped.

## A cell

Cell `i` of row `j`, rect `x, y, w, h`:

1. family: diagonal when `h(i, j, s + 13) < diag`, else flat
2. direction index `k = 2 floor(4 h(i, j, s + 17)) + 1` for a diagonal, `+ 0` for a flat. The 8 directions, as fractions of the cell from start to end: `(0,0)→(1,0)`, `(0,0)→(1,1)`, `(0,0)→(0,1)`, `(1,0)→(0,1)`, `(1,0)→(0,0)`, `(1,1)→(0,0)`, `(0,1)→(0,0)`, `(0,1)→(1,0)`. Even indices run along an edge, odd ones corner to corner
3. inks, with `n` Palette inks: `a = floor(n h(i, j, s + 19))`, step `1 + floor(h(i, j, s + 23) max(1, round(1 + spread (n - 2))))`, `b = (a + step) mod n`
4. ramp ends: `sp = 0.5 (1 - soft)`, `p0 = min(0.49, sp (0.4 + 1.2 h(i, j, s + 29)))`, `p1 = max(0.51, 1 - sp (0.4 + 1.2 h(i, j, s + 31)))`
5. the cell is filled with a linear gradient along that direction

## The ramp

The ramp walks hue the short way round rather than blending in RGB, so opposite inks meet through saturated colour instead of grey. With ink `a` as HSL `A` and ink `b` as `B` (hue in degrees, s and l in 0..1):

- an end with saturation under 0.04 takes the other end's hue
- `dh = B_h - A_h`, wrapped into -180..180 by adding or subtracting 360 once
- stops: ink `a` at 0, ink `a` again at `p0` when `p0 > 0`, then for `q = 1..7` the colour `HSL(A_h + dh q/8, lerp(A_s, B_s, q/8), lerp(A_l, B_l, q/8))` at `p0 + (p1 - p0) q/8`, ink `b` at `p1` when `p1 < 1`, ink `b` at 1

Each walked stop is turned back into an 8-bit `#rrggbb`: hue taken mod 360 into 0..360 as `((h mod 360) + 360) mod 360` with a truncated mod, s and l clamped to 0..1, then the usual HSL to RGB with each channel `round(255 v)`, half up. The gradient interpolates in RGB between those stops and pads past either end.

## Painting

1. Rows top to bottom, cells left to right, each a gradient-filled rect. The cells tile the frame, so no ground is painted.
2. The chassis grain, then dither.

## Fidelity notes

- Every cell edge is on a whole pixel, so the only soft edges are the ramps.
- The site picks 16:9 as its opening ratio; Reference exports set 9:16.
