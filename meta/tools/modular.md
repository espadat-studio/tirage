# modular

A Swiss-grid poster. The frame is a grid of modules, some merged into wider or taller ones. Each module is empty, a flat colour, a gradient, a field of blocks, a cluster of dots or a ruled grid. Thin rules run over the whole grid.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id          | range    | step | default | role                                            |
| ----------- | -------- | ---- | ------- | ----------------------------------------------- |
| `gcols`     | 2..12    | 1    | 6       | grid columns                                    |
| `unit`      | 2..12    | 1    | 4       | unit cells across one module                    |
| `merge`     | 0..1     | 0.01 | 0.45    | chance a module spans two cells                 |
| `wEmpty`    | 0..100   | 1    | 34      | weight of empty modules                         |
| `wSolid`    | 0..100   | 1    | 20      | weight of flat colour modules                   |
| `wBlocks`   | 0..100   | 1    | 24      | weight of block field modules                   |
| `wDots`     | 0..100   | 1    | 14      | weight of dot cluster modules                   |
| `wLines`    | 0..100   | 1    | 12      | weight of ruled modules                         |
| `wGrad`     | 0..100   | 1    | 10      | weight of gradient modules                      |
| `blockFill` | 0.1..0.9 | 0.01 | 0.5     | share of unit cells a block or dot field lights |
| `dot`       | 0.2..1   | 0.01 | 0.62    | dot diameter, share of a unit cell              |
| `rules`     | 0..1     | 0.01 | 0.22    | opacity of the grid rules                       |
| `ruleW`     | 0.5..4   | 0.5  | 1       | line width of every rule, in px                 |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. modular is a Still Tool: only frame 0 is rendered, and with motion off the noise has no time terms. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The page opens on seed 1842; a Reference export sets the seed first. The site deals nothing from the seed beyond the layout below, so every slider keeps the value it is given.

## Palette

modular is a palette-family Tool. Its Palette is the swatch row, any number of inks from 2, read by index. Default Palette, 4 inks: `#E63946 #1D3557 #F1C40F #1B1B1B`.

The ground and the rule colour have their own colour inputs on the page, outside the swatch row. They are Tool constants, not part of the Palette: ground `#F4EFE3`, rule `#1B1B1B`.

## Random draws

One xorshift32 stream, [aura](aura.md)'s: its middle step `a xor= a >> 17` shifts `a` read as signed 32-bit, so the sign bit fills in. Its start is `s · 2654435761 + 17` as a 64-bit float, then wrapped to 32 bits, a zero start becoming 1. `s` is the Tool seed. Past 2^53 the float product loses low bits, and the port does the same float sum.

The cell noise uses a stateless hash of five 32-bit integers `(x, y, z, w, k)` with wrapping multiplies:

- `n = x·374761393 xor y·668265263 xor z·1440662683 xor w·1274126177 xor k·1013904223`
- `n = (n xor n >> 15) · 2246822519`
- `n = (n xor n >> 13) · 3266489917`
- `n = n xor n >> 16`, unsigned, divided by 2^32

Shifts are logical. With motion off `z = w = 0`, and the hash is aura's noise hash on channel 0.

## Layout

With `gc = gcols` columns, the grid has `gr = max(2, round(gc · H / W))` rows. Cells are walked row by row, left to right. A cell already covered by a merged module is skipped. For every other cell, 9 stream draws in this order:

1. `k`. When `k < 0.32 merge` and a 2x2 block from this cell is inside the grid and free, the module is 2x2. Else when `k < 0.66 merge` and 2x1 is free, it is 2 wide. Else when `k < merge` and 1x2 is free, it is 2 tall. Else 1x1. The module's cells are marked taken.
2. Type: `n = draw · total`, `total` the sum of the six weights, or 1 when they sum to 0. Walk empty, solid, blocks, dots, lines, grad, subtracting each weight from `n`; the first type that brings `n` to 0 or below wins. If none does, the module is empty.
3. `ci = floor(draw · m)`, the module ink, `m` the Palette length.
4. `ci2 = (ci + 1 + floor(draw · (m - 1))) mod m`, a second ink, never `ci`.
5. `sub`: the cluster inset is 0 below 0.55, 1 below 0.8, else 2.
6. `onBg = draw < 0.45`: the field sits on the ground instead of a `ci2` panel.
7. `corner = floor(draw · 4)`, the corner a dot cluster hugs.
8. `angle = floor(draw · 4)`, the gradient direction.
9. A phase draw, used only by motion.

Modules are numbered by their order in this walk.

## Edges

`edges(n, a, b)[i] = round(a + (b - a) · i / n)` for `i = 0..=n`, rounding halves up. Column edges are `edges(gc, 0, W)` and row edges `edges(gr, 0, H)`. A module spans from its first cell's edge to the edge past its last cell. Inside it, `uw = w · unit` and `uh = h · unit` unit cells, with edges `edges(uw, x0, x1)` and `edges(uh, y0, y1)`.

## Cell noise

For a unit cell `(gx, gy)` in whole-grid unit coordinates (`gx = module x · unit + i`) and a key `r`:

1. noise seed `q = s + 131 r + 9`, wrapped to 32 bits
2. `v = (0.5 vn(0.55 gx, 0.55 gy, q) + 0.25 vn(1.1 gx, 1.1 gy, q + 1319)) / 0.75`, where `vn` is value noise on the hash with smoothstep weights: corners `a` (floor), `b` (+x), `c` (+y), `d` (+x +y), `lerp(lerp(a, b, u), lerp(c, d, u), v)`
3. `v = clamp((v - 0.5) · 1.9 + 0.5, 0, 1)`
4. the cell is lit when `v < blockFill`

Block fields use the module number as the key, dot clusters the module number plus 77.

## Painting

1. Fill the frame with the ground.
2. Modules in walk order, `col = ink ci`, `col2 = ink ci2`:
   - empty: nothing.
   - solid: the module rect in `col`.
   - grad: the module rect filled with a linear gradient from `col` at 0 to `col2` at 1. Its line by `angle`: 0 top-left to top-right, 1 top-left to bottom-left, 2 top-left to bottom-right, 3 top-right to bottom-left.
   - blocks: unless `onBg`, the module rect in `col2`. Then each unit row, left to right, fills each run of lit cells as one rect in `col`, from the run's first edge to the edge past its last.
   - dots: unless `onBg`, the rect in `col2`. The cluster is `cw = max(1, uw - 2 inset)` cells wide, the inset counting only when `uw > 2`, and `ch` likewise. It sits at the right when `corner & 1`, at the bottom when `corner & 2`. Each lit cell in it gets a filled `col` circle at the cell centre, radius `min(cell width, cell height) · dot / 2`.
   - lines: unless `onBg`, the rect in `col2`. One path of vertical lines at each inner unit column edge plus 0.5, from the module top to bottom, and horizontal lines at each inner unit row edge plus 0.5, across the module. Stroked once in `col` at alpha 0.85, width `ruleW`, butt caps.
3. When `rules > 0`: one path of vertical lines at each inner column edge plus 0.5 over the full height, and horizontal lines at each inner row edge plus 0.5 over the full width, stroked once in the rule colour at alpha `rules`, width `ruleW`.
4. The chassis grain, then dither.

## Fidelity notes

- Each ruled path is stroked once, so crossings inside it do not stack their alpha.
- Rects sit on whole pixels; only circles, gradients and odd rule widths antialias.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
