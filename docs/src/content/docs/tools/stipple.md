---
title: "stipple"
description: "A dot print of a noise field. It renders a Still."
---

A dot print of a noise field. A warped field is sampled on a square or hex grid, and each sample may become a dot on a dark ground. In Lattice mode the high ground of the field gets dots, larger the higher it is. In Contour mode the dots gather where the field changes fastest, so they trace its contours. The field can be mirrored or folded round the centre, and a few dots take a bright accent.

stipple renders a Still: one frame. Its site page is [playgrnd.tools/stipple](https://www.playgrnd.tools/stipple/).

## Still

![stipple Still for Seed 15](../../../assets/tools/stipple.png)

Seed 15, rendered by:

```sh
tirage derive --seed 15 --tool stipple | tirage render --size 540x960 -o stipple.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
stipple  1 frame
  modesDot    Lattice, Contour
  lattices    Square, Hex
  shapes      Circle, Square
  res         12..=200  step 1
  dot         0.05..=1.4 step 0.01
  vary        0..=1     step 0.01
  cut         0..=0.95  step 0.01
  jit         0..=1     step 0.01
  edge        0..=1     step 0.01
  loose       0..=0.6   step 0.01
  zoom        0.5..=9   step 0.1
  warp        0..=3     step 0.02
  oct         1..=7     step 1
  contrast    0.4..=4   step 0.05
  syms        None, Mirror, Quad, Radial
  folds       2..=16    step 1
  accRate     0..=0.15  step 0.001
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

A Seed draws the 4 choices and all 13 art ranges. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `dot` starts at 0.75, because smaller dots fade into the ground. Dither and the grain post-pass stay off. `cut` only acts in Lattice mode, `edge` and `loose` only in Contour mode, and `folds` only with Radial.

The Palette is the dot inks, any number from 2, taken from low to high ground. The ground `#0b132b` and the accent `#ffd166` are fixed, as on the site page by default.
