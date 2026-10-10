# stipple

A dot print of a noise field. A warped fbm field is sampled on a square or hex lattice and each sample may become a dot on a dark ground. In Lattice mode the field's high ground gets dots, bigger the higher it is. In Contour mode the dots gather where the field changes fastest, tracing its contours. A few dots take a bright accent.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults, in page order:

| id         | range                      | step  | default | role                                        |
| ---------- | -------------------------- | ----- | ------- | ------------------------------------------- |
| `modesDot` | Lattice, Contour           |       | Contour | which samples become dots                   |
| `lattices` | Square, Hex                |       | Square  | the sample grid                             |
| `shapes`   | Circle, Square             |       | Circle  | the dot shape                               |
| `res`      | 12..200                    | 1     | 72      | lattice columns                             |
| `dot`      | 0.05..1.4                  | 0.01  | 0.62    | dot size against the cell                   |
| `vary`     | 0..1                       | 0.01  | 0.7     | how much dot size follows the field         |
| `cut`      | 0..0.95                    | 0.01  | 0.34    | Lattice: field level below which no dot     |
| `jit`      | 0..1                       | 0.01  | 0       | how far dots stray from their lattice point |
| `edge`     | 0..1                       | 0.01  | 0.85    | Contour: how strongly dots follow the edges |
| `loose`    | 0..0.6                     | 0.01  | 0.12    | Contour: odds of a dot away from the edges  |
| `zoom`     | 0.5..9                     | 0.1   | 2.4     | field frequency                             |
| `warp`     | 0..3                       | 0.02  | 0.8     | how far the field is pushed around          |
| `oct`      | 1..7                       | 1     | 4       | field octaves                               |
| `contrast` | 0.4..4                     | 0.05  | 1.6     | field contrast                              |
| `syms`     | None, Mirror, Quad, Radial |       | None    | how the field is folded                     |
| `folds`    | 2..16                      | 1     | 6       | Radial: wedges round the centre             |
| `accRate`  | 0..0.15                    | 0.001 | 0.055   | share of dots in the accent ink             |

`cut` only acts in Lattice mode, `edge` and `loose` only in Contour mode, `folds` only with Radial.

Motion: four modes, Drift (default), Turn, Breathe and Twinkle, off by default. stipple is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the field's two extra noise axes are 0, the field is not turned, and the twinkle salt is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

Default Palette, the 3 swatches: `#3a86ff #8ab6ff #d6e6ff`. The site's palette family also sets a ground and an accent, which are Tool constants at the site defaults: ground `#0b132b`, accent `#ffd166`. The family picker is not a Parameter. The Palette is uncapped and painted with every ink it has.

## Noise

The 5-argument integer hash and the fbm are [terrain](terrain.md)'s: value noise with smoothstep fade on the hash of the four lattice corners, extra axes 0, and `fbm(x, y, s, n)` summing `n` octaves at weight 0.5, 0.25, … and frequency 1, 2, …, octave `i` at seed `s + 1319 i`, divided by the weight total. The site's noise is 4D, but with motion off its third and fourth axes sit at 0 and the fade weight there is 0, so it is exactly the 2D noise.

`seed` is the Tool seed. There are no sequential random draws.

## Field

The page is `1000` units wide and `h = 1000 * H / W` tall, `ar = h / 1000`. A point `(nx, ny)` in 0..1 page fractions:

1. Fold: `u = nx - 0.5`, `v = ny - 0.5`.
   - Mirror: `u = |u|`.
   - Quad: `u = |u|`, `v = |v|`.
   - Radial: `r = hypot(u, v)`, wedge `w = 2π / max(2, folds)`, `a = atan2(v, u)` taken into `0..w` by a float remainder, and `a = w - a` when `a > w / 2`. Then `u = r cos a`, `v = r sin a`.
2. `su = (u + 0.5) * zoom`, `sv = (v + 0.5) * zoom * ar`.
3. Warp: `wx = fbm(su + 5.2, sv + 1.3, seed + 11, 2)`, `wy = fbm(su + 9.1, sv + 7.7, seed + 29, 2)`.
4. `f = fbm(su + warp * (wx - 0.5) * 2, sv + warp * (wy - 0.5) * 2, seed, oct)`.
5. The level is `(f - 0.5) * contrast + 0.5`, clamped to 0..1.

## Dots

- `cols = max(2, res)`, `rows = max(2, round(cols * ar))`, times 1.1547 inside the round on a hex lattice. Cell `cw = 1000 / cols`, `ch = h / rows`, `cell = min(cw, ch)`.
- `eps = 1 / max(cols, rows)`.

First pass, rows then columns. Point `(i, j)` sits at `cx = (i + 0.5 + o) cw`, `cy = (j + 0.5) ch`, where `o` is 0.5 on odd rows of a hex lattice and 0 otherwise. A point with `cx > 1000` is dropped. Read its level `L` at `(cx / 1000, cy / h)`. In Contour mode also its gradient: `g = hypot(L(nx + eps, ny) - L(nx - eps, ny), L(nx, ny + eps) - L(nx, ny - eps))`. Track the lowest and highest level, and of `g`, over all points.

Second pass, same order. `v = (L - low) / max(1e-6, high - low)`.

- Contour: `g' = clamp((g - gLow) / max(1e-6, gHigh - gLow), 0, 1)`. The dot is kept when `hash(i, j, 0, 3, seed + 61) < loose + edge * g'`. Radius `cell * 0.5 * dot * (1 - vary * 0.5 + vary * 0.5 * g')`.
- Lattice: kept when `v > cut`. With `q = (v - cut) / max(0.001, 1 - cut)`, radius `cell * 0.5 * dot * (1 - vary + vary * clamp(q, 0, 1))`.
- A dot whose radius is at most 0.05 is dropped.
- Jitter, when `jit > 0`: `cx += (hash(i, j, 0, 7, seed + 3) - 0.5) * cw * jit`, `cy += (hash(i, j, 0, 8, seed + 4) - 0.5) * ch * jit`.
- Ink: the accent when `accRate > 0` and `hash(i, j, 0, 9, seed + 17) < accRate`. Otherwise Palette ink `min(n - 1, floor(v * n))` of `n` inks.

## Painting

Fill the frame with the ground. Scale `k = W / 1000`. Each dot in order, in its ink: a Circle is a disc at `(x k, y k)` with radius `r k`; a Square is the rect from `((x - r) k, (y - r) k)`, `2 r k` on a side. Both are anti-aliased.

## Fidelity notes

- `hypot` and `atan2` may differ from V8's in the last bit. A dot right at a keep threshold can flip, which costs one dot.
- Reference exports use the 3-ink default Palette, as the site's My colors pads a shorter Palette.
