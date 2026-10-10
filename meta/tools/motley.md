# motley

A patchwork of glyph fields on a grid. Ragged blobs and stepped blocks are laid over one another, each patch filling its cells with one glyph in one ink: discs, pluses, stripes, squares, ringed discs or dots. Some patches are sparse so the one beneath shows through, some are stitched along their edge, and some are left as bare ground.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id        | range   | step | default | role                                       |
| --------- | ------- | ---- | ------- | ------------------------------------------ |
| `cells`   | 16..120 | 1    | 56      | grid columns                               |
| `mark`    | 0.4..1  | 0.01 | 0.85    | glyph size against the cell                |
| `patches` | 1..60   | 1    | 24      | how many small patches, over 8 big ones    |
| `size`    | 0..1    | 0.01 | 0.5     | small patch size                           |
| `ragged`  | 0..1    | 0.01 | 0.55    | how ragged a blob's edge is                |
| `blocks`  | 0..1    | 0.01 | 0.4     | odds a small patch is a block, not a blob  |
| `sparse`  | 0..1    | 0.01 | 0.35    | odds a patch keeps only some of its cells  |
| `stitch`  | 0..1    | 0.01 | 0.4     | odds a patch is stitched along its edge    |
| `gaps`    | 0..0.5  | 0.01 | 0.1     | share of small patches left as bare ground |

On this page `size` is a Parameter and the export size is `sizePx`.

The site deals every slider from the variation number when it changes, and shows the deal on the sliders. tirage does not port that deal: `derive` deals the Parameters from the Seed instead ([ADR 0003](../adr/0003-seed-derivation-of-recipes.md)). The refs export sets the Tool seed first and every Parameter after it, so the site's deal is overwritten.

Motion: three modes, Flicker (default), Drift and Breathe, off by default. motley is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the flicker salt and drift are 0 and the glyph size is `mark`. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, 9 inks: `#111111 #f4f1ea #1e7a46 #e5352b #f5c400 #f28ab8 #2c7ac2 #f0801e #f3e2b0`. Ink 1 is the ground. The rest are the patch inks, any number of them.

## Noise and random draws

- The hash and value noise are [sonar](sonar.md)'s.
- One sequential generator: the xorshift from [sprig](sprig.md), started from `seed * 2246822519` wrapped to 32 bits, xor `0x27d4eb2f`, 0 becoming 1.

`seed` is the Tool seed, taken as a 32-bit signed integer.

## Patches

`m` is the number of patch inks. There are `max(1, patches) + 8` patches. Patch `k` draws, in order:

1. It is big when `k < 8`. A small patch draws: it is a block when the draw is under `blocks`. A big patch is never a block and draws nothing here.
2. Radius `R = cols * (0.45 + 0.4 draw)` when big, else `cols * (0.1 + size * 0.35 * (0.4 + 0.8 draw))`.
3. Centre `cx = cols * draw`, then `cy = rows * draw`, in cells.
4. Ink `a = floor(m * draw)`.
5. Stitch ink `b = floor(m * draw)`, moved on to `(b + 1) mod m` when it equals `a`.
6. Glyph `floor(6 draw)` of: disc, plus, stripes, square, ring, dot.
7. Sparse: a draw under `sparse` makes the patch sparse, and a second draw picks its kind `1 + floor(2 draw)`: 1 keeps a checkerboard, 2 keeps every other row. Otherwise it is full and draws once.
8. Stitched when the draw is under `stitch`.
9. Gap rank `e = draw`.
10. Block width `R * (0.6 + 0.9 draw)`, then block height `R * (0.5 + 0.9 draw)`. Blobs draw these too.
11. Ring dot ink `(a + 1 + floor((m - 1) * draw)) mod m`.

Its noise salt is `13 seed + 7 k`.

After all patches: sort the small ones by `e`, lowest first, and mark the first `round(gaps * count)` as empty.

A cell `(i, j)` is inside a patch when, with `dx = i + 0.5 - cx` and `dy = j + 0.5 - cy`:

- block: `|dx| ≤ width / 2` and `|dy| ≤ height / 2`
- blob: `hypot / R < 0.55 + ragged * 0.8 * (noise(i / (0.55 R) + 7, j / (0.55 R) + 3, salt) - 0.5) + 0.2`

## Painting

- `cols = max(8, cells)`, cell `cs = W / cols`, `rows = ceil(H / cs)`, glyph size `g = cs * mark`.
- Fill the frame with the ground.
- Each patch in order. Its reach is `max(width, height) / 2 + 1` for a block, `1.3 R + 1` for a blob. Walk rows `max(0, floor(cy - reach))` to `min(rows - 1, ceil(cy + reach))`, and in each the same span of columns, skipping cells not inside.
- Stitched patch, edge cell (one of its four neighbours is not inside): fill the cell's square in ground, `cs + 0.5` on a side from `(i cs, j cs)`, then a dot when `hash(i, j, salt) < 0.5`, else a plus, in the stitch ink. Next cell.
- Sparse kind 1 skips cells with `i + j` odd, kind 2 skips odd rows, and either skips cells with `hash(i, j, salt) < 0.12`. Skipped cells keep what lies beneath.
- Otherwise fill the cell's square in ground. An empty patch stops there. Else draw its glyph at the cell centre `((i + 0.5) cs, (j + 0.5) cs)` in its ink.

Glyphs at centre `(x, y)`:

- disc: radius `0.47 g`
- plus: a bar `0.92 g` long and `0.24 g` thick across, and one down
- stripes: three bars `0.9 g` long and `0.17 g` thick, centred at `y - 0.32 g`, `y`, `y + 0.32 g`
- square: `0.92 g` on a side
- ring: a disc of radius `0.47 g`, then a disc of radius `0.2 g` in the ring dot ink
- dot: radius `0.17 g`

All fills are anti-aliased.

## Fidelity notes

- Cell squares and bars sit on fractional pixels, so their edges blend by coverage, as canvas does.
- Reference exports use the 9-ink default Palette, as the site's My colors pads a shorter Palette.
