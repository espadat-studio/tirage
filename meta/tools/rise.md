# rise

A field of even vertical bars with giant concentric bands rising from one anchor on the frame edge or its centre. Every band shares the field's bar grid, so the stripes line up across it; a band only swaps which two inks are its ground and its bars.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range                                  | step | default | role                                    |
| -------- | -------------------------------------- | ---- | ------- | --------------------------------------- |
| `styles` | Auto, Bottom, Top, Left, Right, Centre |      | Auto    | the anchor, or let the seed choose it   |
| `count`  | 6..28                                  | 1    | 14      | bars across the short edge              |
| `weight` | 0.3..0.7                               | 0.01 | 0.5     | share of a bar period that is bar       |
| `bands`  | 1..8                                   | 1    | 3       | how many bands                          |
| `depth`  | 0.1..0.4                               | 0.01 | 0.24    | band thickness, share of the short edge |

Auto picks the anchor from the Tool seed (see Draws). A Recipe names the anchor it draws, so the Choice is Bottom, Top, Left, Right and Centre without Auto. The site's Auto is the same as naming the anchor it picks.

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. rise is a Still Tool: only frame 0 is rendered, and with motion off the ripple, slide and pulse terms are all 0. Motion stays at site defaults and is not a Parameter. Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

The site deals `count`, `weight`, `bands` and `depth` from the seed on every seed change. That deal is not ported: `derive` deals every Parameter (ADR 0003), and a Reference export sets every slider after the seed.

## Palette

Fixed roles, 3 inks: Field, Bars, Accent. Default `#ff9f1c #2ec4b6 #011627`. A Palette may have at most 3 inks; a shorter one wraps by position.

## Draws

Only Auto draws. One xorshift stream, the arithmetic-shift one [warp](warp.md#draws) describes (aura's), started from `s · 2654435761` as a double, modulo 2^32, 0 becoming 1. One draw `r`: the anchor is `[bottom, bottom, bottom, centre, left, right, top][floor(7 r)]`. A named anchor takes no draw, so a Recipe never draws.

## Geometry

For a `W x H` frame, `md = min(W, H)`:

- bar period `P = md / max(4, count)`, bar width `w = P · weight`
- band thickness `bw = depth · md`, band count `N = max(1, bands)`
- anchor `A`: bottom `(W/2, H)`, top `(W/2, 0)`, left `(0, H/2)`, right `(W, H/2)`, centre `(W/2, H/2)`

Bars, in one ink: bar `k` starts at `x_k = W/2 + k P - w/2` for every integer `k` from `ceil((-W/2 - w) / P)` to `floor((W/2 + w) / P)`. Each is a rect from `round(x_k)` to `round(x_k + w)` across, `-2` to `H + 2` down, with `round` going half up. A rect of zero width draws nothing.

Band `i` (`0` innermost) is a disc at `A` of radius `(i + 0.8) bw`. Its ground and bar inks are pair `i mod 4` of:

| pair | ground | bars   |
| ---- | ------ | ------ |
| 0    | Bars   | Accent |
| 1    | Accent | Field  |
| 2    | Bars   | Field  |
| 3    | Accent | Bars   |

## Painting

1. Fill the frame with the Field.
2. The bars in Bars.
3. Bands from the outermost (`N - 1`) in, each clipped to its disc: fill the frame with the band's ground, then the bars in the band's bar ink. An inner band paints over the outer ones.
4. The chassis grain, then dither.

## Fidelity notes

- Every fill is opaque and the bar edges sit on whole pixels, so the only soft edges are the antialiased disc clips.
- The site picks 1:1 as its opening ratio; Reference exports set 9:16.
- The PNG export repaints at the export size, so `md` and the bar grid follow the export, not the preview.
