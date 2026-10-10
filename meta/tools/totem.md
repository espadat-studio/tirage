# totem

A screenprinted emblem. A mat of clotted marks frames a dark panel. Inside a keyline the panel's left half is cut into rectangles, each printed with a flat two-ink pattern, and every rectangle has a twin reflected across the vertical centre line. A nested emblem of rings sits at dead centre. Everything lands on one square cell grid, so nothing is anti-aliased.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id          | range   | step  | default | role                                                   |
| ----------- | ------- | ----- | ------- | ------------------------------------------------------ |
| `border`    | 0..0.4  | 0.005 | 0.15    | mat width against the short edge                       |
| `mat`       | 0..0.7  | 0.01  | 0.36    | how much of the mat carries marks                      |
| `matGrain`  | 1..6    | 1     | 2       | mat mark size, in cells                                |
| `keyline`   | 0..10   | 1     | 3       | dark rule between panel edge and composition, in cells |
| `regions`   | 1..30   | 1     | 14      | how many rectangles the half-panel is cut into         |
| `grain`     | 16..220 | 1     | 110     | cells across the short edge                            |
| `mirror`    | 0..1    | 0.01  | 1       | chance a twin keeps its original's pattern             |
| `variety`   | 0..1    | 0.01  | 0.7     | how many motifs the bag holds                          |
| `core`      | 0..0.6  | 0.01  | 0.22    | emblem width against the composition's short edge      |
| `coreRings` | 0..8    | 1     | 3       | emblem rings                                           |

`grain` here is the grid resolution. It sets the cell size, so it is structure, not a speck or texture layer. Like oddgrid's `grain` it is an art slider: a Parameter that a Seed deals inside Taste bounds.

Motion (`modes` Shuffle or Weave, `amt`, `fps`, `frames`) is off by default and not part of a Still. totem is a Still Tool: only frame 0 is rendered. With motion off the paint stream starts from the Tool seed itself and the pattern phase is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site has no seed deal on this page: a new seed changes only the paint stream.

## Palette

Variable, any number of inks. Default, 5 inks: `#1b1b1b #ff6b35 #f7b32b #2e86ab #f72585`.

Roles, from the Palette in order:

- `dark`: the ink of least `0.299 r + 0.587 g + 0.114 b`, the first such on a tie.
- `rest`: every other ink, in Palette order. With one ink, `rest` is that ink.
- `mat`: `rest[0]`.
- `mark`: `rest[max(0, len - 2)]` when `rest` has two or more inks, else `dark`.
- `inks`: `rest`.

Ink comparisons below are by colour value, so a repeated ink counts as the same ink.

## Space

Device pixels, `W` by `H`. The cell is `u = max(2, round(min(W, H) / grain))`, and `snap(v) = round(v / u) u`. Every `round` here rounds halves up. Every rect is whole pixels and is filled without anti-aliasing.

## Random numbers

One stream `r`: the arithmetic-shift xorshift32 [aura](aura.md) uses, but started at the Tool seed itself (0 becoming 1), not at its golden-ratio product. Draws below come off it in the order written. Nothing is hashed.

## Motifs

The 11 motifs, in bag order: solid, check, hline, vline, diag, diagB, brick, dash, grid, rings, noise. A motif marks cell `(i, j)` of a `cols` by `rows` grid:

| motif | marked when                                  |
| ----- | -------------------------------------------- |
| solid | always                                       |
| check | `i + j` even                                 |
| hline | `j` even                                     |
| vline | `i` even                                     |
| diag  | `(i + j) mod 4 < 2`                          |
| diagB | `(i - j) mod 4 < 2`, a non-negative mod      |
| brick | `(i + 2 (j mod 2)) mod 4 < 2`                |
| dash  | `j` even and `i mod 3 < 2`                   |
| grid  | `i mod 3 = 0` or `j mod 3 = 0`               |
| rings | `min(i, j, cols - 1 - i, rows - 1 - j)` even |
| noise | the next draw is below 0.5                   |

Printing motif `kind` into rect `x, y, w, h` at cell size `cs` with mark ink `a` and ground `b`: fill the rect in `b`. For solid, fill it again in `a` and stop. Otherwise `cols = max(1, round(w / cs))` and `rows = max(1, round(h / cs))`. For `j` in rows, then `i` in cols, a marked cell is the rect at `(x + i cs, y + j cs)` sized `min(cs, x + w - px)` by `min(cs, y + h - py)`, filled in `a`. noise draws once per cell in that order.

## Dealing a region

`deal` draws, in order:

1. `kind = bag[floor(r len)]`. If it is solid, draw again; below 0.55, redraw `kind` the same way.
2. `a = inks[floor(r len)]`.
3. `pool` = `inks` then `dark` twice, less every entry equal to `a`. If `pool` is non-empty, `b = pool[floor(r len)]`, else `b = dark` with no draw.
4. `cs = 2u` when the next draw is below 0.3, else `u`.

This runs inside the painter on the paint stream. It is not a seed deal.

## Carving

`carve(x, y, w, h, n)` starts from one rect and loops while it holds fewer than `n` rects, at most 400 times:

1. Take the rect of largest area, the first on a tie.
2. It can split vertically when `w ≥ 6u`, horizontally when `h ≥ 6u`. Neither: stop.
3. If both, split vertically when `w / h > 1.1`, horizontally when `h / w > 1.1`, else vertically when the next draw is below 0.5. If only one, take it.
4. `frac = 0.3 + 0.4 r`. Remove the rect from the list.
5. The cut along the split side `s` is `max(2u, round(s frac / u) u)`. If it is not inside `0 < cut < s`, put the rect back at the end and stop. Otherwise append the two halves, first the left or top one.

## Painting

1. Fill the frame in `mat`. `bw = snap(min(W, H) border)`. The panel is `px0 = py0 = bw`, `pw0 = snap(W - 2 bw)`, `ph0 = snap(H - 2 bw)`.
2. The mat, only when `bw > 0` and `mat > 0.01`. Cell `mu = u max(1, matGrain)`, grid `gw = ceil(W / mu) + 1` by `gh = ceil(H / mu) + 1`. Each cell, rows outer, is on when its draw is below `mat`. Two smoothing passes: a cell's sum covers its 3×3 block, a neighbour off the grid counting as the cell itself. Over 4 turns it on, under 4 off, exactly 4 keeps it. Then every on cell whose `mu` square at `(x mu, y mu)` misses the panel rect is filled in `mark`.
3. Stop when `pw0 < 4u` or `ph0 < 4u`.
4. Fill the panel in `dark`. `k = keyline u`. The composition is `cx = px0 + k`, `cy = py0 + k`, `cw = max(u, pw0 - 2k)`, `ch = max(u, ph0 - 2k)`.
5. The bag: `n = max(2, round(2 + 9 variety))` motifs, each drawn out of the remaining motifs by `floor(r len)` and removed.
6. `halfW = max(u, ceil(cw / u / 2) u)`. Carve `(cx, cy, halfW, ch)` into `max(1, regions)` rects.
7. For each rect `g`, in list order:
   - `L = deal`. `drawW = min(g.w, cx + cw - g.x)`. If positive, print `L` into `(g.x, g.y, drawW, g.h)` at `L.cs`.
   - `mx = cx + cw - (g.x - cx) - g.w`, the twin's left edge.
   - The twin is `L` when the next draw is below `mirror`, else a fresh `deal`.
   - `clipX = max(mx, cx + cw - halfW)`, `tw = min(g.w - (clipX - mx), cx + cw - clipX)`. When `tw > 0`: print the twin, reflected, into a `g.w` by `g.h` rect. Then, if `drawW > 0`, print `L` again over the original, a second full print that draws noise afresh.
8. The emblem, when `core > 0.01`:
   - `kw = snap(min(cw, ch) core)`, `kh = snap(kw (1.2 + 0.6 (s mod 5) / 5))` with `s` the Tool seed.
   - Clamp `kw` into `2u..cw - 2u` and `kh` into `2u..ch - 2u`, the lower bound winning. `kx = snap(cx + (cw - kw) / 2)`, `ky = snap(cy + (ch - kh) / 2)`.
   - Ring `i` of `coreRings`: fill `(kx, ky, kw, kh)` in `dark` for even `i`, `inks[(i + 1) mod len]` for odd. Step in by `u` on every side. If `kw` or `kh` fell below `2u`, raise each to at least `u` and stop.
   - `K = deal`. Print `K.kind` into the remaining rect at cell `u`, mark `K.a`, ground `dark`.
9. The chassis grain, then dither.

## Mirror

The twin is drawn in a frame translated to `mx + g.w` and flipped by `scale(-1, 1)`, its rect starting at local `x = 0`. Every coordinate is a whole pixel, so a local rect `(px, py, wd, ht)` is exactly the device rect `(mx + g.w - px - wd, py, wd, ht)`. The port draws it that way, with no transform.

## Fidelity notes

- `imageSmoothingEnabled = false` and whole-pixel rects: no edge is blended. A diff should be all or nothing per cell.
- The twin is not clipped. A wide twin can overprint the left half, which is why the original is printed again after it.
- The site picks 3:4 as its opening ratio. Reference exports set 9:16.
