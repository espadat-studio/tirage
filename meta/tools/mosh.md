# mosh

A stack of horizontal bands of broken video data, each a different failure: confetti runs, a torn mosaic, long smears, whole scan rows, and a chevron shear. Every band is flat rectangles in a few hard inks, with a share of the frame fallen to the darkest ink.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range   | step | default | role                                            |
| -------- | ------- | ---- | ------- | ----------------------------------------------- |
| `bands`  | 1..14   | 1    | 6       | how many bands are stacked                      |
| `cols`   | 24..420 | 2    | 150     | columns across, the cell size                   |
| `mix`    | 0..1    | 0.01 | 0.62    | how many inks a pick draws from                 |
| `tears`  | 0..1    | 0.01 | 0.55    | how many diagonal tears cut a mosaic band       |
| `runs`   | 0..1    | 0.01 | 0.5     | smear run length                                |
| `bright` | 0..1    | 0.01 | 0.3     | how often ink 1 cuts in on smear and scan bands |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. mosh is a Still Tool: only frame 0 is rendered, and with motion off the shuffle seed is the Tool seed and the roll is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The page turns image smoothing off but never draws an image, so there is nothing to port for it.

Default Palette, 8 inks: `#000000 #ffffff #ff3131 #31ff6b #3164ff #31f0ff #ff31e0 #ffee31`. Inks are picked by index from the whole Palette, any number from 2. Ink 1 is the bright cut-in. The dark ink is the one with the lowest luma `0.299 r + 0.587 g + 0.114 b`, the first on a tie.

A Seed deals all 6 sliders.

## Random draws

None. All randomness is the chassis integer hash, `hash(x, y, seed)`, at seeds offset from the Tool seed `s`, wrapping at 32 bits.

## Picks

`pick(v, bias)`: when `v < 0.16`, the dark ink. Otherwise, with `n` inks and `k = max(1, round(n (0.25 + 0.75 mix)))`, ink `(floor(v k) + bias) mod n`.

`round` is round half up throughout.

## Bands

With `W x H` the frame, `N = max(6, floor(cols))` columns and `cw = W / N`:

- `nb = max(1, floor(bands))`. Band `i` weighs `0.4 + 1.6 hash(i, 3, s + 11)`. Its height is its share of the weights times `H`.
- The kinds are confetti, mosaic, smear, scan, chevron, shuffled from the end: for `i` from 4 down to 1, swap `i` with `floor(hash(i, 7, s + 13) (i + 1))`. Band `b` takes kind `b mod 5`.
- A running `y` starts at 0. Band `b` spans `round(y)` to `round(y + h)`, the last band to `H`. Then `y += h`.
- Each band's seed is `bs = s + 101 b`.

Each band paints solid rectangles in order, later ones over earlier ones, without anti-aliasing. A rectangle with no width or height paints nothing. Rows are `rh = band height / rows` tall and each row's rectangle starts at `round(top + j rh)` and is `ceil(rh) + 1` tall, so it overlaps the next row.

Confetti:

- `rows = max(1, round(bh / cw))`.
- Along each row `j`, from column `i = 0`: `long = hash(i, j, bs + 101)`. The run is `5 + floor(260 long)` when `long < 0.1`, else `1 + floor(3 hash(i, j, bs + 17))`. Ink `pick(hash(i, j, bs + 19), j)`. It spans `round(i cw)` to `round(min(N, i + run) cw)`. Then `i += run`.

Mosaic:

- Block width `step = max(2, round(4 + 5 hash(b, 5, bs + 23)))` columns. `cols2 = ceil(N / step)`, `rows = max(1, round(bh / (cw step)))`.
- Tear count: 0 when `tears` is 0, else `1 + floor(2.6 tears hash(b, 9, bs + 29))`. Tear `t` has offset `rows hash(t, 11, bs + 31)`, slope `1.8 (2 hash(t, 13, bs + 37) - 1) rows / cols2`, and faces up when `hash(t, 15, bs + 41) < 0.5`.
- Block `(i, j)` is torn when there is a tear and, for every tear, `j` is below its edge `offset + slope i` when it faces up, or above it otherwise.
- `v = hash(i step, j, bs + 43)`. A torn block is the dark ink when `v < 0.82`, else `pick(v, b)`. Otherwise `pick(v, b + j)`.
- It spans `round(i step cw)` to `round(min(N, (i + 1) step) cw)`.

Smear:

- `rows = max(2, round(bh / max(2, 0.6 cw)))`. Base run `L = 6 + 46 runs`.
- Along each row, from column 0: the run is `max(1, round(L (0.25 + 1.5 hash(i, 3j, bs + 47))))`. The ink is ink 1 when `hash(i, j, bs + 59) < 0.35 bright`, else `pick(hash(i, j, bs + 53), j)`. Spans as confetti.

Scan:

- From `yy` at the band top, row `j` from 0, while `yy` is inside the band: `rh = max(1, round(cw (0.4 + 3.2 hash(0, j, bs + 61))))`, clipped to the band as `hh`.
- A full-width rectangle at `round(yy)`, `ceil(hh) + 1` tall, ink `pick(hash(1, j, bs + 67), j)`.
- When `hash(2, j, bs + 71) < 0.8 bright`, a segment in ink 1: width `W (0.08 + 0.5 hash(3, j, bs + 73))`, left edge `x0 = 0.92 W hash(4, j, bs + 79)`. It is drawn at `round(x0)`, `round(min(width, W - x0))` wide.
- `yy += hh`.

Chevron:

- `rows = max(1, round(bh / cw))`. Period `per = max(3, round(4 + 10 hash(b, 17, bs + 83)))`, shift `amp = max(1, round(2 + 6 hash(b, 19, bs + 89)))`.
- Row `j`: `t = j mod 2 per`, the shift is `round(tri amp / per)` with `tri = t` below `per`, else `2 per - t`.
- Each column `i`: ink `pick(hash((i + shift) mod per, 0, bs + 97), b)`, at `round(i cw)`, `ceil(cw) + 1` wide.

## Fidelity notes

- Edges sit on whole pixels and inks are exact, so any diff is a rectangle whose ink or extent differs.
