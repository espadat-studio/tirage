# warp

Op-art in one ink on one ground. The Tool seed picks a checkerboard or a field of slanted bars, then bends every corner of it through a wave, a taper or a bulge. The shapes are real polygons, filled in one pass.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                | step | default | role                                   |
| -------- | -------------------- | ---- | ------- | -------------------------------------- |
| `styles` | Auto, Checker, Slash |      | Auto    | the pattern, or let the seed choose it |
| `scale`  | 0.5..2               | 0.05 | 1       | cell size                              |
| `warp`   | 0..1                 | 0.01 | 0.7     | how hard the pattern is bent           |

Auto picks Checker or Slash from the Tool seed (draw 1 below). A Recipe names the style it draws, so the Choice is Checker and Slash without Auto. The site's Auto is the same as naming the style it picks.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. warp is a Still Tool: one frame, motion stays off and is not a Parameter. With motion off the bend factor is 1 and the march offset is 0. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

## Palette

Fixed roles, 5 inks: Base, Ink 1, Ink 2, Ink 3, Ink 4. Default `#f7f3ea #f72585 #4361ee #f7b32b #1b1b1b`. A Palette may have at most 5 inks; a shorter one wraps by position.

Each Visual uses exactly two of them: the Base and one of Ink 1..4, picked by the seed. Which of the two is the ground is also picked by the seed.

## Draws

Two seeds come from the Tool seed `s`:

- The draw stream: a 32-bit xorshift (left 13, right 17, left 5), read as unsigned over 2^32. Unlike the chassis xorshift, the right shift is arithmetic: the state is read as signed 32-bit for it, so a set top bit fills in from the left. It starts from `s · 2654435761` multiplied as a double, not a 32-bit integer product. Above 2^53 the product is rounded to a double first. That double is then taken modulo 2^32, and 0 becomes 1.
- The shift seed `sw = s xor 0x2c1b3c6d`, unsigned 32-bit.

Draws, in this order:

1. `kpick = floor(3·draw)`. 0 is checker, 1 is slash, 2 takes checker when `sw` is even and slash when it is odd. A named style replaces this pick, but the draw is always taken.
2. `ink = 1 + floor(4·draw)`.
3. `cell = 0.09 + 0.09·draw`.
4. The bar angle: a sign draw, negative when below 0.5, then `0.35 + 0.55·draw` radians.
5. `duty = 0.45 + 0.18·draw`.
6. Two draws for a centre, unused.
7. The bend: `floor(3·draw)` picks wave, taper or bulge.
8. `wa = 0.5 + 0.8·draw`, `wf = 2 + 3.5·draw`, `ph = TAU·draw`.
9. `brk = 2.5 + 3·draw`.
10. `inv`: true when the draw is below 0.4.
11. A march direction, unused in a Still.

## Shift hash

Slash bars use a second hash of three integers and a seed, all 32-bit wrapping: multiply each of the four by its own large odd constant (374761393, 668265263, 1440662683, 1013904223) and xor the four products. Then twice xor in the value shifted right (15, then 13) and multiply by a constant (2246822519, then 3266489917). Last, xor in the value shifted right by 16. Read as unsigned over 2^32.

## Geometry

`md = min(W, H)`, the bend strength `WA = warp · wa`, the cell side `cs = cell · scale · md`, and a margin `m = 0.35 · md` so bent shapes still reach the frame edge.

Checker: for each row `j` from `floor(-m/cs)` up to but not including `ceil((H + m)/cs)`, and each column `i` from `floor(-m/cs)` to before `ceil((W + m)/cs)`, keep the square at `(i·cs, j·cs)` when `i + j` is even.

Slash: the bars run along a direction `(cos a, sin a)` rotated about the frame's top-left corner. A point `(q, r)` in bar space sits at `(q·cos a - r·sin a, q·sin a + r·cos a)`. `bs = cs · brk` and `R = hypot(W, H) + m`.

- Bands `b` run from `floor(-R/bs)` to before `ceil(R/bs)`, covering `r` from `b·bs` to `(b + 1)·bs`.
- Each band slides along by `shift = 2·cs·hash(b, 3, 7, sw)`.
- In a band, bars `k` run from `floor((-R - shift)/cs)` to before `ceil((R - shift)/cs)`. A bar covers `q` from `k·cs + shift` to that plus `cs · duty`.

Each square or bar is a quad. Each of its 4 sides is cut into 5 equal steps, start included and end left out, giving 20 points. Every point is bent.

## Bend

A point `(u, v)` in pixels:

- Wave: `u + WA·0.15·sin(v/md·wf·2.1 + ph)·md` and `v + WA·0.08·sin(u/md·wf·1.6 + 1.7·ph)·md`.
- Taper: `g = 1 + 0.9·WA·(v/H - 0.5)`; the point moves to `((u - W/2)·g + W/2, v)`.
- Bulge: about the centre `(W/2, H/2)`, `(dx, dy)` is the offset over `md` and `rr = hypot(dx, dy) + 1e-6`. With `k = rr/0.72`, the scale is `f = k^(1 + 1.2·WA) / k`; the point moves to the centre plus `(dx, dy)·f·md`.

## Painting

1. Fill the frame with the ground.
2. Every bent polygon goes into one path, each closed, and the path is filled once with the other colour under the non-zero rule, anti-aliased.

The ground is the Base and the shapes are ink `ink`, swapped when `inv` is true.

## Fidelity notes

- `sin`, `pow` and `hypot` run per point. V8's results may differ in the last bit, far below a pixel.
- Coverage on polygon edges is Skia's on the site and tiny-skia's here, so edge pixels can differ by a few levels.
