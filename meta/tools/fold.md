# fold

A collage of pixel op-art patches, folded across the sheet like a kaleidoscope. Zebra stripes, staircases, dot rows, bursts, chevrons and checks lie over a posterised burst ground, all worked out on a coarse grid and blown up with hard edges.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id        | range                                                        | step | default   | role                                   |
| --------- | ------------------------------------------------------------ | ---- | --------- | -------------------------------------- |
| `folds`   | Four ways, Kaleidoscope, Across, Down, None                  |      | Four ways | how the sheet mirrors                  |
| `kinds`   | Mixed, Zebra, Staircases, Dot rows, Bursts, Chevrons, Checks |      | Mixed     | which patch kind; Mixed deals each one |
| `foldx`   | 0.2..0.8                                                     | 0.01 | 0.5       | where the vertical fold line sits      |
| `foldy`   | 0.2..0.8                                                     | 0.01 | 0.5       | where the horizontal fold line sits    |
| `patches` | 1..16                                                        | 1    | 9         | how many patches                       |
| `size`    | 0..1                                                         | 0.01 | 0.5       | patch size                             |
| `scale`   | 0..1                                                         | 0.01 | 0.5       | pattern frequency inside a patch       |
| `wave`    | 0..1                                                         | 0.01 | 0.5       | how far zebra stripes bend             |
| `levels`  | 2..8                                                         | 1    | 5         | tones in a burst and in the ground     |
| `tilt`    | 0..1                                                         | 0.01 | 0.3       | random turn added to each patch        |
| `px`      | 0..1                                                         | 0.01 | 0.35      | grid cell size                         |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. fold is a Still Tool: only frame 0 is rendered. Motion stays fixed at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default. `sizePx` is the export size, not a Parameter.

fold is a palette-family Tool: the Palette is the swatch row, any number of inks from 2, read by position and by brightness. Default Palette, 6 inks: `#1A1A1A #F5F1E8 #FF5C39 #3B5BFF #FFD23F #7B2CBF`.

The site picks `patches`, `size`, `scale`, `wave`, `levels` and `tilt` from its seed when the seed changes (a "deal"). That deal is not ported: our `derive` replaces it (ADR 0003). Reference exports set the seed first and every Parameter after.

## Inks by role

`lum(c) = 0.2126 r + 0.7152 g + 0.0722 b` on 0..255.

- dark: the first ink with the lowest `lum`, light: the first with the highest.
- ground: ink 0.
- others: inks 2 onward. With fewer than two of those, every ink.
- the ground burst runs from `others[0]` to the ground ink.

## Random numbers

The chassis xorshift32 with logical shifts ([vein](vein.md#randomness)), started at `s * 2246822519` (wrapping) xor `0x27d4eb2f`, 0 replaced by 1.

## Patches

`U = sqrt(W H)`, and the source space is `aw = W / U` by `ah = H / U`. Each of `n = patches` patches takes these draws, in order:

1. Kind: `pool[floor(r len)]`. The pool is the six kinds for Mixed, or the one chosen kind. The draw happens either way.
2. Centre `cx = r aw`, then `cy = r ah`.
3. `sz = (0.18 + 0.32r)(0.5 + size)`, then `w = sz (0.7 + 0.8r)`, then `h = sz (0.7 + 0.8r)`.
4. Angle: `snap = r`; `0` under 0.55, `π/2` under 0.8, else `π/4`. Plus `(r - 0.5) 0.8 tilt`.
5. Inks `a = others[floor(r len)]`, then `b` the same. When `b == a`, `b` is the next entry of `others` after `a`, wrapping.
6. `freq = (4 + 8r)(0.5 + scale)`.
7. Shape: rect when `r < 0.75`, else oval.
8. Phase `ph = 2πr`.

## Grid

`cell = max(2, U (0.005 + 0.02 px))`. `gw = max(8, round(W / cell))`, `gh = max(8, round(H / cell))`.

Cell `(i, j)` samples `x = (i + 0.5) / gw`, `y = (j + 0.5) / gh`.

Fold, with `fx = foldx`, `fy = foldy`:

- Across, Four ways, Kaleidoscope: `sx = |x - fx| / max(fx, 1 - fx)`.
- Down, Four ways, Kaleidoscope: `sy = |y - fy| / max(fy, 1 - fy)`.
- Kaleidoscope: swap so `sx <= sy`.
- Otherwise the coordinate is unchanged. The source point is `(u, v) = (sx aw, sy ah)`.

The top patch wins: walk from the last patch to the first. In patch space `lx = dx cos + dy sin`, `ly = -dx sin + dy cos` with `(dx, dy)` from the centre, and `nx = lx / (w/2)`, `ny = ly / (h/2)`. A rect holds `|nx| <= 1` and `|ny| <= 1`, an oval `nx^2 + ny^2 <= 1`.

Inside a patch, with `F = freq`:

- zebra: on when `sin((nx (1 + 0.6 ny) + 0.35 wave sin(3.1 ny + ph)) F π) > 0`. On is dark, off is light. Zebra ignores `a` and `b`.
- stairs: `q = floor((nx + 1) F/2) + floor((ny + 1) F/2)`; on when `floor(q / 2)` is even.
- dots: `g = F/2`, cell offsets `ddx = frac((nx + 1) g) - 0.5`, `ddy` the same for `ny`. On when `hypot(ddx, ddy) < 0.18 + 0.32 (0.5 + 0.5 sin(2.2 (nx + 1) + ph))`.
- burst: tone `t = clamp(hypot(nx, ny), 0, 1)`, painted as below.
- chevron: on when `floor((|nx| + ny + 1) F/2)` is even.
- checks: on when `floor((nx + 1) F/2) + floor((ny + 1) F/2)` is even.

On is `a`, off is `b`. A tone `t` paints `a + (b - a) floor(t L) / (L - 1)` per channel, `L = levels`. At `t = 1` that steps past `b`; the byte store clamps it.

No patch: the ground burst. `d = hypot(u, v) / hypot(aw, ah)`, and the colour is `others[0] + (ground - others[0]) floor(min(0.999, d) L) / (L - 1)`.

Channels are stored to bytes rounding half to even and clamping, as a canvas `ImageData` does. The grid is drawn over the frame with image smoothing off.

## Fidelity notes

- Patch edges are whole grid cells. The only soft spots are the nearest upscale's choice at each cell edge.
