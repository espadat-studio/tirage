---
title: "Dither and grain"
description: "Two passes every Tool runs over its finished frame, with the same 11 Parameters on every Tool."
---

Every Tool ends with the same two passes over the frame it drew. Grain runs first. It adds film noise, specks and a dark vignette. Dither runs second. It cuts each colour channel down to a few levels, choosing between them with a Bayer or noise threshold pattern. Each pass has a toggle, and a pass that is off or has an amount of 0 leaves the pixels alone.

## Parameters

The same 11 rows close every Tool's block in [`tirage tools`](/cli-reference/tools/). Tool pages leave them out:

```
ditherTog   on/off
dthKinds    Bayer 8, Bayer 4, Noise
dthSize     1..=10    step 1
dthLevels   2..=8     step 1
dthAmount   0..=1     step 0.05
grainTog    on/off
grnBlends   Add, Overlay, Soft light, Multiply, Screen
grnAmount   0..=1     step 0.05
grnSize     0.5..=4   step 0.1
grnSpecks   0..=1     step 0.05
grnVignette 0..=1     step 0.05
```

| Parameter     | Does                                                               | Default   |
| ------------- | ------------------------------------------------------------------ | --------- |
| `ditherTog`   | Turns the dither pass on                                           | off       |
| `dthKinds`    | The threshold pattern: an 8×8 or 4×4 Bayer matrix, or random noise | `Bayer 8` |
| `dthSize`     | The size of each dither block, scaled to the frame's short edge    | 2         |
| `dthLevels`   | How many levels each colour channel keeps                          | 3         |
| `dthAmount`   | How far the result moves from the drawn frame to the dithered one  | 1         |
| `grainTog`    | Turns the grain pass on                                            | off       |
| `grnBlends`   | How the noise mixes into the frame                                 | `Add`     |
| `grnAmount`   | How strong the noise, specks and vignette are                      | 0.55      |
| `grnSize`     | The size of a grain cell, scaled to the frame                      | 1         |
| `grnSpecks`   | How many cells get a light speck                                   | 0.5       |
| `grnVignette` | How much the edges darken                                          | 0.5       |

## What a Seed does with them

A Seed never draws these 11 values. Each Tool keeps the values its site page opens with, so on most Tools a derived Recipe has both passes off. Four Tools open with grain on: coral, crowd, dahlia and vein. Their pages give the grain values. Dither starts off on every Tool.

To turn a pass on, edit its values in the Recipe JSON before `render`, as for any other [Pin](/concepts/pins/#pin-a-parameter-or-the-tool-seed). `render` checks each value against the range above.

Both passes change pixels, so an [SVG](/cli-reference/render/#svg) leaves them out even when the Recipe turns them on.

## A Tool's own grain or dither

Some Tools also have a Parameter of their own called `grain` or `dither`, such as pith, chaff and riso. That one is part of how the Tool draws. It sits with the Tool's own Parameters, and the Tool's page says what a Seed does with it.
