# optic

An op-art counterchange in two inks. An even field of vertical bars covers the frame, and up to three nested figures sit on its centre. Inside each figure the bars run again, moved against the field: shifted half a bar, turned 90°, or tilted 45° either way.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                               | step | default | role                                  |
| -------- | ----------------------------------- | ---- | ------- | ------------------------------------- |
| `styles` | Auto, Diamond, Circle, Peak, Square |      | Auto    | the figure, or let the seed choose it |
| `count`  | 6..28                               | 1    | 14      | bar periods across the short edge     |
| `weight` | 0.3..0.7                            | 0.01 | 0.5     | share of a bar period that is bar     |
| `sizeF`  | 0.4..1                              | 0.01 | 0.88    | outer figure size, share of `md / 2`  |
| `levels` | 1..3                                | 1    | 2       | how many nested figures               |

Auto is a Choice value of its own. Auto takes a draw for the figure before the level moves, so naming the figure Auto picks shifts every later draw and gives a different Visual.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. optic is a Still Tool: only frame 0 is rendered, and with motion off the slide offset, turn, breathe and flip terms are all 0. Motion stays at site defaults and is not a Parameter. One motion slider still reaches a Still: `amt`, at its default 0.8, sets the bar overscan below. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site deals `count`, `weight`, `sizeF` and `levels` from the seed on every seed change. That deal is not ported: `derive` deals every Parameter (ADR 0003), and a Reference export sets every slider after the seed.

## Palette

Fixed roles, 2 inks: Base, Ink. Default `#F5F0E6 #7B2CBF`. A Palette may have at most 2 inks; a shorter one wraps by position.

## Draws

One xorshift stream, the arithmetic-shift one [warp](warp.md#draws) describes (aura's), started from `s · 2654435761` as a double, modulo 2^32, 0 becoming 1. In order:

1. Auto only: one draw `r`, the figure is `[diamond, circle, peak, square][floor(4 r)]`. A named figure takes no draw.
2. For each of `levels` levels: one draw `r`, move `idx = [0, 0, 1, 2, 3][floor(5 r)]`. When `idx` equals the previous level's move, it becomes `(idx + 1) mod 4`. The first level has no previous move.

The moves:

| move | turn | phase |
| ---- | ---- | ----- |
| 0    | 0    | 0.5   |
| 1    | π/2  | 0     |
| 2    | π/4  | 0     |
| 3    | -π/4 | 0     |

## Geometry

For a `W x H` frame, centre `(cx, cy) = (W/2, H/2)`, `md = min(W, H)`:

- bar period `P = md / count`, bar width `w = P · weight`
- reach `R = hypot(W, H) / 2 + 4 P`. The 4 is `lap + 2` with `lap = max(1, round(2 · amt)) = 2` at the default `amt`
- level `i` has figure size `s_i = sizeF · md / 2 · [1, 0.55, 0.22][i]`, turn and phase `φ_i = phase · P` from its move

A bar set with phase `φ` has bars at offsets `x_k = k P + φ - w/2` for every integer `k` from `ceil((-R - φ - w) / P)` to `floor((R - φ + w) / P)`.

- Turn 0: each bar is a rect from `round(cx + x_k)` to `round(cx + x_k + w)` across, `-2` to `H + 2` down, with `round` going half up. A rect of zero width draws nothing.
- Any other turn: translate to `(cx, cy)`, rotate by the turn, then each bar is the rect `x_k, -R, w, 2R`, unsnapped.

Figures, all centred on `(cx, cy)` with size `s`:

- circle: radius `s`
- square: half side `0.9 s`
- diamond: corners `(cx, cy - s)`, `(cx + s, cy)`, `(cx, cy + s)`, `(cx - s, cy)`
- peak: a triangle `(cx, cy - 0.92 s)`, `(cx + 1.05 s, cy + 0.8 s)`, `(cx - 1.05 s, cy + 0.8 s)`

## Painting

1. Fill the frame with the Base.
2. The field: a turn-0 bar set with phase 0, in Ink.
3. Each level, outer first, clipped to its figure: fill the frame with the Base, then the level's bar set in Ink. Each clip stands alone; a level paints over the ones before it.
4. The chassis grain, then dither.

## Fidelity notes

- Turn-0 bars sit on whole pixels, so their edges are sharp. Tilted bars and the figure clips are antialiased.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
- The PNG export repaints at the export size, so `md` and the bar grid follow the export, not the preview.
