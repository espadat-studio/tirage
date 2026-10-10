# sampler

A tiled sampler of hard-edged marks. The frame is a grid of cells cut into horizontal bands. Each band picks one or a few tile shapes (triangles, diagonals, bars, elbows, steps, halves, notches, arrows) and two inks, then fills its cells with those shapes turned a quarter at a time over a dark ground.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id        | range  | step | default | role                                        |
| --------- | ------ | ---- | ------- | ------------------------------------------- |
| `grid`    | 4..48  | 1    | 16      | columns                                     |
| `bands`   | 1..14  | 1    | 6       | how many bands the rows aim for             |
| `mix`     | 0..1   | 0.01 | 0.55    | how many tile shapes a band may mix         |
| `turn`    | 0..1   | 0.01 | 0.75    | how freely a tile turns from its neighbours |
| `density` | 0.1..1 | 0.01 | 0.95    | share of cells that get a tile              |
| `weight`  | 0..1   | 0.01 | 0.5     | lean of the tiles toward a band's first ink |

The site has no seed deal: changing the seed leaves every slider where it is.

Motion: two modes, Shuffle (default) and Turn, off by default. sampler is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the band seed is the Tool seed and the extra spin is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

## Palette

Variable, any number of inks. Default, 7 inks: `#101010 #d6ff3c #6f6a1c #ff7a1f #f4786e #c9c4ca #a2218e`.

The ground is the darkest ink by `0.299 r + 0.587 g + 0.114 b` on 0..255; on a tie the first one wins. The tile inks `use` are every ink not equal to the ground. If that leaves none, `use` is the whole Palette.

## Random numbers

No stream. Every draw is one integer hash `h(x, y, s)` on 32-bit wrapping integers:

1. `n = 374761393 x + 668265263 y + 1274126177 s`
2. `n = 1274126177 (n xor (n >> 13))`
3. `n = n xor (n >> 16)`
4. return `n / 2^32`, `n` read unsigned

Shifts are logical. `s` is the Tool seed plus a small offset, also wrapped to 32 bits.

## Grid

`cols = grid`. `rows = max(3, round(grid H / W))`, halves rounding up. The site reads the chosen ratio here; for the frame's own aspect the value is the same. A cell is `cw = W / cols` by `ch = H / rows` pixels.

## Bands

`want = bands`. The run starts at row `y0 = floor(rows h(0, 71, s + 3))` and walks down, wrapping. For band `i = 0, 1, …` at row `y`, while `y - y0 < rows`:

- height `hb = max(1, round(rows / want (0.55 + 0.95 h(i, 11, s + 13))))`, cut to `rows - (y - y0)` when it would pass the end of the run
- shape count `nk = 1 + floor(h(i, 17, s + 19) (1 + 2.2 mix))`
- shape `k` of `0..nk` is `TILES[floor(8 h(7i + k, 23, s + 29))]`, repeats allowed, from the order triangles, diagonals, bars, elbows, steps, halves, notches, arrows
- first ink `a = use[floor(len h(i, 31, s + 37))]`
- second ink: among `use` without `a`, the one at `floor(len h(i, 41, s + 43))`; the ground when `use` holds only `a`
- band seed `bs = s + 257 i`

The band covers rows `y..y + hb`, then `y += hb`. A row past the bottom wraps to `ry mod rows`.

## Tiles

For each band in order, each run row `ry` in it, each column `x`:

- skip the cell when `h(3x + 1, 5ry + 7, bs) > density`
- shape `kinds[floor(len h(x, ry, bs + 61)) mod len]`
- quarter turns `q = floor(4 h(11x, 13ry, bs + 67) (0.25 + 0.75 turn)) mod 4`
- ink `a` when `h(17x, 19ry, bs + 73) < 0.5 + 0.3 (weight - 0.5)`, else `b`

All hashes use the run row `ry`, not the wrapped row.

Each shape is a closed polygon in the unit square:

| shape     | points                                                          |
| --------- | --------------------------------------------------------------- |
| triangles | (0,0) (1,0) (0,1)                                               |
| diagonals | (0,0.34) (0.66,0) (1,0) (1,0.66) (0.34,1) (0,1)                 |
| bars      | (0,0.3) (1,0.3) (1,0.7) (0,0.7)                                 |
| elbows    | (0,0.3) (0.7,0.3) (0.7,1) (0.3,1) (0.3,0.7) (0,0.7)             |
| steps     | (0,0.5) (0.5,0.5) (0.5,0) (1,0) (1,0.5) (0.5,0.5) (0.5,1) (0,1) |
| halves    | (0,0) (1,0) (1,0.5) (0,0.5)                                     |
| notches   | (0,0) (1,0) (1,1) (0.5,1) (0.5,0.5) (0,0.5)                     |
| arrows    | (0,0) (0.5,0.5) (0,1) (0.34,1) (0.84,0.5) (0.34,0)              |

## Painting

1. Fill the frame in the ground.
2. Each tile, in the order above, fills its polygon nonzero under the transform: translate to `(x cw, (ry mod rows) ch)`, scale by `(cw, ch)`, translate `(0.5, 0.5)`, rotate `q π/2`, translate `(-0.5, -0.5)`. Scaling before the turn keeps a turned tile inside its own cell on a non-square cell.
3. The chassis grain, then dither.

## Fidelity notes

- Tiles are filled one at a time and anti-aliased, so two touching tiles of one ink leave a faint seam of ground where their edges blend.
- The site opens at 3:4; Reference exports set 9:16.
