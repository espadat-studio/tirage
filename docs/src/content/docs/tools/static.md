---
title: "static"
description: "A one-bit glitch poster of stacked pattern bands in two inks. It renders a Still."
---

A one-bit glitch poster. The frame splits into stacked bands, each with its own pattern: moire, bars, static noise, blocks, rings or zigzags. Sometimes an inset panel sits on top. A glitch pass then shifts rows, smears them and punches out boxes.

static renders a Still: one frame. Its site page is [playgrnd.tools/static](https://www.playgrnd.tools/static/).

## Still

![static Still for Seed 1](../../../assets/tools/static.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool static | tirage render --size 540x960 -o static.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
static  1 frame
  regions     2..=5     step 1
  res         48..=160  step 4
  glitch      0..=1     step 0.01
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

A Seed draws `regions`, `res` and `glitch`, each across its full range. `regions` is how many bands stack down the frame, `res` how many grid columns span it, and `glitch` how hard the glitch pass hits. Dither and grain stay off.

The Palette is two inks: the ink, then the ground. static takes at most 2. The site's swap button only reverses them, so swapping the Palette's order does the same.
