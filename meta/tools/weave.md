# weave

Horizontal bands of woven cloth. Each band has its own ground colour and a run of vertical stripes in one or two thread colours, at its own pitch, duty and height.

Written from reading the site page to learn the algorithm. No site code is copied.

## Inputs

Site control ids, slider ranges and site defaults:

| id       | range    | step | default | role                                |
| -------- | -------- | ---- | ------- | ----------------------------------- |
| `bands`  | 3..12    | 1    | 6       | how many bands stack down the frame |
| `stripe` | 0.5..2.2 | 0.05 | 1       | scales every band's stripe pitch    |

Motion (`modes`, `amt`, `fps`, `frames`) is off by default and not part of a Still. The Loop turns motion on at site defaults, and motion is not a Parameter. See [Motion](#motion). Grain and dither are the shared [chassis](chassis.md) post-passes, off by default.

weave is a palette-family Tool. Its Palette is the swatch row only, read by position, any number of inks from 2. It has no ground, rule or ink fields of its own. The site's colour shuffle only loads swatches and is not a Parameter. Default Palette, 7 inks: `#22223b #f2e9e4 #c9ada7 #9a8c98 #4a4e69 #e63946 #f4d35e`.

## Random numbers

One xorshift32 stream per frame: xor in the state shifted left 13, then the state shifted right 17, then shifted left 5, and the draw is the state over 2^32. A zero state starts at 1. The right shift is arithmetic: it reads the state as signed 32-bit, so a state with its top bit set shifts ones in. This is not the logical shift the shared chassis stream uses.

The stream's seed is the Tool seed times 2654435761, worked out as a 64-bit float, then reduced mod 2^32. For Tool seeds past about 3.4 million the float product is rounded before the reduction, so the seed is not the 32-bit wrapping product. A port must do the float multiply.

## Layout

Work in a view box 1000 wide and `1000 H / W` tall, here 1777.7… for 9:16. `n = bands` and `P` is the Palette with `len` inks.

1. Draw `n` weights, each `0.55 + 1.3 r`. `sum` is their total.
2. Walk the bands top down from `y = 0`. Band `i` ends at `min(VH, y + w_i / sum VH)`, and the last band ends at `VH` exactly.

Then, per band in order, these draws:

1. Ground: `bg = floor(r len)`.
2. First thread: `c1 = floor(r len)`, redrawn while it equals `bg` and `len > 1`.
3. Two threads: draw `r` always. The band has two threads when that draw is under 0.4 and `len > 2`.
4. Only with two threads: `c2 = floor(r len)`, redrawn while it equals `bg` or `c1`. One thread means `c2 = c1`.
5. Thin: `r < 0.45`.
6. Pitch: thin bands `1000 (0.012 + 0.02 r)`, others `1000 (0.04 + 0.07 r)`. Times `stripe`.
7. Duty: thin bands `0.28 + 0.3 r`, others `0.42 + 0.35 r`.
8. Extent draw `e = r`.
9. Height share: `1` when `e < 0.55`, else `0.6 + 0.3 r`.
10. Alignment: `full` when `e < 0.55`. Else one draw: under 0.5 is `top`. Else a second draw: under 0.5 is `bottom`, else `center`.
11. Three more draws: direction, repeat count and phase. Direction is `-1` when the draw is under 0.5, else `1`. The repeat count is `1 + floor(2r)`, so 1 or 2. Both only drive motion, but they are drawn for a Still too. The phase is the third.

## Stripes

A band's period is `pitch * threads`. Its offset at frame 0 is `phase * period`. Stripe `k` starts at `x = k pitch + offset` and is `pitch * duty` wide. Its thread is `k mod threads`, kept non-negative.

The visible run takes `k` from `floor((-offset - width) / pitch) - 1` to `ceil((1000 - offset) / pitch) + 1`. Stripes that end before 0 or start past 1000 are skipped.

The stripe rect is the band height times its share. `top` and `full` start at the band top, `bottom` ends at the band bottom, `center` is centred.

## Painting

`k = W / 1000` maps the view box to pixels. Every rect is whole pixels and unblended.

Per band in order:

1. Ground: from `floor(y0 k)`, full width, `ceil((y1 - y0) k) + 1` tall. The extra pixel is covered by the next band.
2. Each stripe, in `k` order: left `round(x k)`, top `round(sy k)`, width `max(1, round(w k))`, height `round(sh k)`, in its thread colour. Rounding goes half up, as JavaScript's `Math.round` does. A stripe can start left of 0 and is clipped.

## Motion

The site has two motion modes: Slide and Blink. Slide is the default and the only one ported: amount 0.8, 120 frames at 24 fps.

- Frame `f` sits at `T = f / frames`.
- Slide adds `dir * T * speed * period` to each band's offset, before the visible run is worked out. `speed = max(1, round(repeats * amount))`, so a band with 1 repeat slides 1 period per Loop and a band with 2 slides 2. Bands slide left or right by their own direction draw, at their own speed.
- Nothing else moves: the band heights, grounds, threads, pitches, duties and stripe rects hold.
- The Loop closes by whole periods: at `T = 1` each offset has moved `speed` periods, so stripe `k` sits where stripe `k - dir * speed * threads` was, and that stripe has the same thread. Every rect lands on its frame 0 pixels, unless the float sum tips a `round` on a stripe edge, which the closure test would catch.
- At `f = 0` the slide is 0, so frame 0 is the Still, byte for byte.
- The mid frame is 60: half a Loop, a 1-repeat band half a period along and a 2-repeat band one whole period along.
- Blink is not ported. It deals each band's inks again at staggered steps and is not reachable from a Recipe.

## Fidelity notes

- Every edge is a whole pixel with no anti-aliasing, so a port can match the export byte for byte.
- The only float traps are the seed's float product and `Math.round` half up on negative stripe lefts.
