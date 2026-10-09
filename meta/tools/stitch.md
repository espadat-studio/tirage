# stitch

A coarse grid of flat cells with dark gutters, over a noise field smeared into long vertical streaks. Whole rows of cells slip sideways in bands, the way a glitched video frame tears. An accent ink pools into blobs, and a few stray cells take a random ink.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id         | range  | step | default | role                                         |
| ---------- | ------ | ---- | ------- | -------------------------------------------- |
| `cols`     | 16..96 | 1    | 48      | cells across                                 |
| `gutter`   | 0..1   | 0.01 | 0.2     | gutter width, a share of the cell            |
| `streak`   | 0..1   | 0.01 | 0.8     | how far the field is smeared down the frame  |
| `scale`    | 0..1   | 0.01 | 0.5     | field feature size                           |
| `slip`     | 0..1   | 0.01 | 0.4     | how far a band of rows slips sideways        |
| `band`     | 1..16  | 1    | 5       | rows per slipping band                       |
| `blobs`    | 0..1   | 0.01 | 0.35    | how much of the frame the accent blobs cover |
| `blobsize` | 0..1   | 0.01 | 0.5     | blob size                                    |
| `steps`    | 2..12  | 1    | 7       | how many levels the field is cut into        |
| `ground`   | 0..1   | 0.01 | 0.35    | share of the levels that take ink 0          |
| `stray`    | 0..1   | 0.01 | 0.3     | how many cells take a random ink             |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. stitch is a Still Tool: only frame 0 is rendered, and with motion off the drift offsets are 0, the slip factor is 1 and the flicker term is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, 6 inks: `#111111 #f6f23a #f01fd0 #62f19e #f5532b #ded7eb`. Inks go by position: ink 0 is the ground, ink 1 the accent, inks 2 and up the middle inks. The gutter is the darkest ink by luma. Any number of inks from 2 works. The site deals the sliders from the Tool seed, but not the Palette; we do not port that deal.

A Seed deals all 11 sliders.

## Random draws

The middle inks are shuffled once per frame by one xorshift32 stream. Its state starts at `s 2246822519 ^ 0x27d4eb2f` for Tool seed `s`, wrapping at 32 bits, and a zero state becomes 1. Each draw: xor in the state shifted left 13, then shifted right 17 (logical), then shifted left 5, all on 32 bits. The draw is the new state divided by 2^32.

All other randomness is the chassis integer hash and value noise, at seeds `7 s + 1` to `7 s + 4`, wrapping.

## Inks

- `steps' = max(2, floor(steps))`.
- The middle list is inks 2 to `n - 1` for `n` inks. When that is empty, it is ink 1 alone, or ink 0 for a 1-ink Palette.
- Shuffle it from the end: for `k` from its last index down to 1, `j = floor(draw (k + 1))`, swap `k` and `j`.
- `g = min(steps' - 1, round(steps' ground))` levels take ink 0. Level `l` at or past `g` takes middle ink `(l - g) mod len`.
- The accent is ink 1, or ink 0 for a 1-ink Palette.
- The gutter ink is the one with the lowest luma `0.2126 r + 0.7152 g + 0.0722 b`, the first on a tie.

`round` here and below is round half up.

## Grid

With `W x H` the frame:

- `cols' = max(4, floor(cols))`, cell width `cw = W / cols'`, `rows = max(1, round(H / cw))`, cell height `ch = H / rows`.
- The gutter is `G = round(0.35 gutter cw)` px.
- Field factors `kx = 3 + 6 (1 - scale)` and `ky = kx (1 - 0.92 streak) H / W`. The stripe share is `sw = 0.25 + 0.4 streak`.
- Rows per band `max(1, floor(band))`. The slip reach is `0.3 slip cols'`.
- Blob factor `kb = 3 + 6 (1 - blobsize)`, blob threshold `1 - 0.42 blobs`. The stray share is `0.06 stray`.

The frame is first filled with the gutter ink. Then each row `j` and cell `i`:

1. The row's band is `floor(j / rows per band)`. Its offset is `2 (hash(band, 3, 7 s + 1) - 0.5)` times the slip reach.
2. `u = (i + offset) / cols'`, `v = j / rows`.
3. `n = noise(kx u, ky v, 7 s + 2) (1 - sw) + noise(6 kx u + 7, 1.5 ky v + 3, 7 s + 3) sw`.
4. `val = clamp(2.2 (n - 0.5) + 0.5, 0, 0.9999)`. The ink is that of level `floor(val steps')`.
5. When `noise(kb i / cols' + 11, kb (H / W) j / rows + 5, 7 s + 4)` is above the blob threshold, the ink is the accent.
6. `h = hash(i, j, s + 9)`. When `h` is below the stray share, the ink is `floor(h / share n) mod n`.
7. Fill the cell from `(round(i cw) + G, round(j ch) + G)` to `(round((i + 1) cw), round((j + 1) ch))`, without anti-aliasing, when both sides are positive.

## Fidelity notes

- Edges sit on whole pixels, so a diff is a cell whose ink flipped, from a last-bit difference in the noise at a level boundary.
