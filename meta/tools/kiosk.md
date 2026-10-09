# kiosk

A pasted-up street poster. Big arcs of colour sweep in from a centre off the page, a block of vertical stripes covers the right part, a column of halftone and solid tiles runs down the right edge, and one dark panel sits low on the page. Over all of it goes a grid of bold monospace characters: a coarse rank of big ones at the cell centres, and smaller ones packed into the corners between them.

Written from reading the site page to learn the algorithm. No site code is copied. The text pipeline comes from the [kiosk text research](https://github.com/espadat-studio/tirage/blob/research/kiosk-text/meta/research/kiosk-text.md) (#20, #24).

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id          | range                                     | step | default | role                                    |
| ----------- | ----------------------------------------- | ---- | ------- | --------------------------------------- |
| `split`     | 0.15..0.95                                | 0.01 | 0.62    | where the stripe block starts, across   |
| `rings`     | 4..40                                     | 1    | 15      | how many arcs                           |
| `stripes`   | 2..24                                     | 1    | 9       | how many stripes                        |
| `sets`      | DOS, Stipple, Blocks, Code, Digits, Runes |      | DOS     | the characters drawn                    |
| `grid`      | 4..30                                     | 1    | 13      | character columns across the frame      |
| `bigSize`   | 0.3..1.6                                  | 0.01 | 0.5     | big character size, in cell heights     |
| `density`   | 0..1                                      | 0.01 | 0.84    | share of cells that get a big character |
| `smallSize` | 0.1..0.9                                  | 0.01 | 0.44    | small character size, in cell heights   |
| `small`     | 0..1                                      | 0.01 | 0.6     | share of corners that get a small one   |
| `blocks`    | 0..1                                      | 0.01 | 0.62    | share of edge tiles that are drawn      |

The site also has a Custom set with a free text field. tirage drops it (#24).

Character sets, in order:

- DOS: `0369#%&@!;,'()`
- Stipple: `.,;:'"^~*`
- Blocks: `░▒▓█■▪·`
- Code: `/\|_-+=<>[]{}`
- Digits: `0123456789`
- Runes: `†‡§¶®©≠∞≈`

All are in the Basic Multilingual Plane, so a set's length in UTF-16 units is its character count.

Motion: two modes, Shuffle (default) and Bloom. tirage ports Shuffle only, at the site defaults, and it is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. They wrap the painter, so the PNG export gets them too.

Default Palette, 8 inks: `#f3efe0 #141414 #d7263d #1b4079 #f4b942 #3c887e #f26430 #8e44ad`.

## Hash

The integer hash from [sonar](sonar.md#integer-hash), with the same 32-bit wrapping. The site sums the three products as a float before the first xor, and the xor wraps it to 32 bits, so the result is the same as wrapping the sum. Every draw is `hash(a, b, seed + salt)`, and the seed sum wraps at 32 bits too.

`seed` is the Tool seed. The text uses the text seed, which is the Tool seed on a Still.

## Ink roles

The Palette is read by role, not by position.

- An ink's lightness is `0.299 r + 0.587 g + 0.114 b`, on 0..255 channels.
- Dark is the first ink with the lowest lightness, paper the first with the highest. Ties keep the earlier ink.
- The rest are every ink that equals neither dark nor paper, in order. If none are left, the rest is just paper.
- Type is the first of the rest.
- The bands are the rest after the first. If the rest has only one ink, the bands are that one ink.

For the default Palette: dark `#141414`, paper `#f3efe0`, type `#d7263d`, and 5 bands from `#1b4079` to `#8e44ad`. `band(i)` is band `i mod n`, for `n` bands.

## Painting

Everything is drawn on a `W x H` canvas, in this order. Later paint covers earlier paint. All rects and discs are anti-aliased, as canvas draws them.

### 1. Rings

- Fill the whole frame with `band(0)`.
- The centre is `cx = W * (-0.22 + 0.30 * hash(1, 1, seed + 11))`, `cy = H * (-0.06 + 0.36 * hash(2, 2, seed + 13))`. It sits left of the page and near its top, so the rings arrive as arcs.
- `n = max(4, rings)`.
- The outer radius is `1.02 * hypot(max(|cx|, |W - cx|), max(|cy|, |H - cy|))`, which reaches the far corner. The ring step is that over `n`.
- For `k` from `n` down to 1, fill a disc of radius `k * step` at the centre in `band(k)`. Largest first, so each smaller disc leaves a ring of the one below.

### 2. Stripes

- The block starts at `sx = round(W * clamp(split, 0.15, 0.95))`.
- `m = max(2, stripes)`. Stripe `i` has weight `0.45 + hash(i, 7, seed + 31)`. The block width `W - sx` is shared out by weight.
- Walk `x` from `sx`. Each stripe fills `round(x)..round(x + w)` across and the full height, in `band(floor(hash(i, 9, seed + 37) * n))`, then `x += w`. A stripe whose rounded edges meet draws nothing.

### 3. Edge tiles

- The column is `0.115 W` wide and its left edge is `W - 0.115 W`. Both can be fractional.
- 8 tiles, each `H / 8` tall. Tile `i` runs from `round(i * H / 8)` to `round((i + 1) * H / 8)`.
- Tile `i` is skipped when `hash(i, 13, seed + 41) > blocks`.
- `flip` is 1 when `hash(0, 19, seed + 47) < 0.5`, else 0. A drawn tile is halftone when `i + flip` is even, else solid.
- Solid: fill the tile in dark.
- Halftone: fill the tile in paper, then dark dots on a 6-wide lattice of pitch `d = width / 6`. Rows `0..ceil(height / d)`. A dot at column `a`, row `b` is centred at `(left + (a + 0.5) d, top + (b + 0.5) d)` with radius `0.23 d`. A dot whose centre is below the tile bottom is skipped, so the last row can spill over the edge a little.

### 4. Panel

- Width `W * (0.26 + 0.18 * hash(3, 3, seed + 53))`, height `H * (0.14 + 0.11 * hash(4, 4, seed + 59))`.
- Left `W * (0.26 + 0.26 * hash(5, 5, seed + 61))`, top `H * (0.56 + 0.22 * hash(6, 6, seed + 67))`.
- Each of the four is rounded to a whole pixel, then the rect is filled in dark.

### 5. Type

- `cols = max(4, grid)`, cell width `W / cols`, `rows = max(2, round(H / cell width))`, cell height `ch = H / rows`.
- Big pass, rows then columns: cell `(i, j)` is skipped when `hash(i, j, text + 71) > density`. Otherwise draw a character at `((i + 0.5) cw, (j + 0.52) ch)`, size `ch * bigSize`, salt 71.
- Small pass, rows then columns: corner `(i, j)` is skipped when `hash(i, j, text + 91) > small`. Otherwise draw at `((i + 1) cw, (j + 1) ch)`, size `ch * smallSize`, salt 91. The last row and column of corners sit on the frame edge.
- One character, salt `s`: it is character `floor(hash(i, j, text + s + 3) * len)` of the set. Its ink is dark when `hash(i, j, text + s + 7) < 0.28`, else type.
- The font size is rounded to one decimal, halves up, as `toFixed(1)` does. That rounded size is what is drawn.

Rounding elsewhere goes half up, as `Math.round` does.

## Glyphs

Each character is one bold glyph, centred on its point both ways. The font is the bundled DejaVu Sans Mono Bold 2.37, the face Chromium resolves for kiosk's monospace stack when it is the only font installed.

- Across: the glyph's left edge is `x - advance / 2`, with the `hmtx` advance at the drawn size.
- Down: canvas `middle` puts the baseline at `y + (a - d) / 2`. `a` is `size * typoAscender / (typoAscender - typoDescender)` from the OS/2 table, rounded to 1/64 px. `d` is the size rounded to 1/64 px, minus `a`. For DejaVu the typo metrics are 1556 and -492 at 2048 units per em.
- Up to 256 px: the outline is hinted by the light autohinter, as FreeType's slight hinting does. The left edge is rounded to 1/4 px and the baseline to a whole pixel, both halves up.
- Above 256 px: the outline is not hinted, and the position is not rounded.
- The outline is filled nonzero and anti-aliased, in the glyph's ink.

## Motion

Shuffle, at the site defaults: amount 0.6, 24 frames at 6 fps.

- Frame `f` uses text seed `seed + 7919 f`. The cells kept, the characters and their inks are dealt again each frame.
- The rings, stripes, tiles, panel, grid, sizes and positions do not change.
- At `f = 0` the text seed is the Tool seed, so frame 0 is the Still. The Loop closes by cutting from the last frame back to the first, as the site's player does.

Bloom is not ported. It keeps the text and steps the band colours outward a ring at a time.

## Fidelity notes

- The text is most of the risk. Chrome's glyph pipeline must be copied: an unhinted render lands at 1.6-3.4% on its own (research #20).
- Disc and fractional rect edges are anti-aliased by both Skia and tiny-skia. Their coverage differs by a few levels at most, which stays under the luma cutoff.
- `hypot` may differ from V8's in the last bit. That moves a ring edge by far less than a pixel.
- Reference exports use the 8-ink default Palette. With a shorter Palette, the site's "My colors" pads the Tool's 8 swatches with derived tints, and the roles change.
