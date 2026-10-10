# oddgrid

A patchwork of hard-edged pixel regions. Three noise fields decide each cell of a coarse grid: one picks its colour, one decides whether it is filled at all, and a fine one breaks the edges into pixels. Blocks of cells can be pulled toward a tone of their own, so the regions square off into a quilt. A share of filled cells can carry a small motif in a fixed ink.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id         | range                                 | step | default | role                                           |
| ---------- | ------------------------------------- | ---- | ------- | ---------------------------------------------- |
| `cols`     | 12..110                               | 1    | 44      | grid columns                                   |
| `scale`    | 2..60                                 | 0.5  | 10      | field size, in cells                           |
| `density`  | 0..1                                  | 0.01 | 0.55    | how much of the grid is filled                 |
| `grain`    | 0..1.4                                | 0.01 | 0.35    | how far the fine field breaks the region edges |
| `variety`  | 0..1.6                                | 0.01 | 0.5     | how much blocks differ in coverage and grain   |
| `block`    | 0..1                                  | 0.01 | 0.4     | how far regions square off into blocks         |
| `bsize`    | 2..16                                 | 1    | 6       | block size, in cells                           |
| `speck`    | 0..0.6                                | 0.01 | 0.08    | share of filled cells given a random ink       |
| `motifs`   | None, Dot, Ring, Square, Wedge, Mixed |      | None    | the motif drawn in marked cells                |
| `motifAmt` | 0..1                                  | 0.01 | 0       | share of filled cells that carry the motif     |
| `markSize` | 0.15..1                               | 0.01 | 0.52    | motif size against the cell                    |
| `balance`  | -1.2..1.2                             | 0.05 | 0       | skews the colour field toward low or high inks |

The site's Looks are presets that set several sliders at once. They are not Parameters. The site also offers a Custom motif from an uploaded SVG, with an ink toggle. tirage ships the built-in motifs only, so `motifs` has no Custom and the toggle is not a Parameter. On this page the export size is `size`, 500..6000, default 2000.

Motion: four modes, Flow (default), Scan, Pulse and Shimmer, off by default. oddgrid is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the fields' two extra noise axes are 0 and the shimmer salt is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, the 5 swatches: `#ff5e5b #ffed66 #00cecb #f2ede4 #9b5de5`. The site's palette family also sets a ground and a motif ink, which are Tool constants at the site defaults: ground `#101820`, motif ink `#f2ede4`. The family picker is not a Parameter. The Palette is uncapped and painted with every ink it has.

## Noise

The 5-argument integer hash and the fbm are [terrain](terrain.md)'s, as for [stipple](stipple.md): the site's 4D noise with its extra axes at 0 is exactly the 2D noise. `seed` is the Tool seed. There are no sequential random draws.

`lerp(a, b, t) = a + (b - a) t`, and `stretch(v, k) = clamp((v - 0.5) k + 0.5, 0, 1)`.

## Grid

- `rows = max(4, round(cols * H / W))`. `n` is the number of Palette inks, `B = max(2, bsize)`, `s = scale`, `p = 2^-balance`.
- For each cell `(x, y)`, rows then columns, its block is `(rx, ry) = (floor(x / B), floor(y / B))`.

1. Block-pulled point: `bx = lerp(x + 0.5, B rx + B / 2, block)`, `by` likewise.
2. Colour field: `c = fbm(bx / s, by / s, seed, 2)`. Block tone `0.5 c + 0.5 hash(rx, ry, 0, 13, seed + 404)`. `base = stretch(lerp(c, tone, block), 2)`.
3. Coverage: `mask = stretch(fbm((x + 0.5) / (1.15 s) + 91, (y + 0.5) / (1.15 s) - 17, seed + 9111, 3), 1.8)`.
4. Fine field: `det = fbm((x + 0.5) / (0.26 s) + 37, (y + 0.5) / (0.26 s) - 11, seed + 7717, 2)`.
5. Block bias `(hash(rx, ry, 0, 11, seed + 808) - 0.5) * variety * 0.6`. Level `0.5 + (density - 0.5) * 1.1 + bias`.
6. Local grain `grain * lerp(1, 0.15 + 2.1 hash(rx, ry, 0, 17, seed + 909), clamp(0.8 variety, 0, 1))`.
7. The cell is filled when `mask + (det - 0.5) * localGrain * 1.15 < level`. Otherwise it is empty and shows the ground.
8. A filled cell takes ink `min(n - 1, floor(base^p * n))`. When `speck > 0` and `hash(x, y, 0, 3, seed + 21) < speck`, it takes ink `floor(hash(x, y, 0, 4, seed + 33) * n) mod n` instead.
9. A filled cell is marked when the motif is not None, `motifAmt > 0` and `hash(x, y, 0, 5, seed + 64) < motifAmt`.

## Painting

- Fill the frame with the ground.
- Cell edges are snapped to whole pixels: column edge `i` at `round(i W / cols)`, row edge `j` at `round(j H / rows)`.
- Each row, left to right. Empty cells are skipped. A marked cell fills its own rect in its ink, then its motif. Unmarked filled cells merge with following cells of the same ink that are not marked, and the run fills one rect. All these rects have whole-pixel edges, so nothing is anti-aliased.

Motif in a cell at `(px, py)`, `cw` by `ch`, in the motif ink, with `cs = min(cw, ch)`, `m = markSize` and centre `(px + cw / 2, py + ch / 2)`:

- Dot: a disc of radius `cs m / 2`.
- Ring: a circle stroked `lw = max(1, 0.28 cs m)` wide, radius `max(0.6 lw, cs m / 2 - lw / 2)`.
- Square: `cs m` on a side, centred.
- Wedge: the triangle on the cell's bottom-left, bottom-right and top-right corners, filled.
- Mixed picks per cell from `k = hash(x, y, 0, 9, seed + 404)`: under 0.4 a dot, under 0.65 a square, under 0.85 a ring, else a wedge.

Motifs are anti-aliased.

## Fidelity notes

- Cells are whole-pixel rects, so most of the frame matches exactly. Motif edges blend by coverage.
- The PNG export snaps cells to whole pixels: the canvas is `cols * cs` by `rows * cs`, with `cs = max(1, round(size / cols))`. It is exactly 540x960 only when `cs` divides both edges, so on 9:16 `cols` must be 18, 27, 36, 45, 54, 90 or 108. Reference exports pin `cols` to one of those, the one nearest the Seed's draw, and are named for it.
- Reference exports use the 5-ink default Palette, as the site's My colors pads a shorter Palette.
