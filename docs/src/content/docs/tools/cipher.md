---
title: "cipher"
description: "A grid of small glyphs whose bands trace a figure, a relief or a slope. It renders a Still."
---

A coded sheet. A grid of small glyphs sits on a dark ground: squares, diamonds, ringed squares and x's. A field is cut into bands, and every cell takes the glyph and ink of its band, so the bands draw a figure, a contour relief or a slope. Stray squares in other inks, x's along the band edges and smeared rows break up the grid.

cipher renders a Still: one frame. Its site page is [playgrnd.tools/cipher](https://www.playgrnd.tools/cipher/).

## Still

![cipher Still for Seed 1](../../../assets/tools/cipher.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool cipher | tirage render --size 540x960 -o cipher.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
cipher  1 frame
  cells       20..=140  step 1
  bands       2..=7     step 1
  fields      Figure, Relief, Slope
  wave        0..=1     step 0.01
  tilt        -1..=1    step 0.01
  detail      0..=1     step 0.01
  size        0.3..=1   step 0.01
  edges       0..=1     step 0.01
  flecks      0..=0.3   step 0.01
  smears      0..=0.6   step 0.01
  dots        0..=1     step 0.01
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

A Seed draws every Parameter above the dither ones. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `size` stays within 0.48..=1, because smaller glyphs fade into the ground. `fields` picks what the bands trace. `cells` is how many glyphs span the frame. The last band is always the ground, with a faint dot in each cell that `dots` sizes. Dither and grain stay off.

The Palette takes any number of inks from 2. The first ink is the ground, and every later ink can colour a band, a stray square or an edge mark.
