---
title: "motley"
description: "A patchwork of glyph fields on a grid. It renders a Still."
---

A patchwork of glyph fields on a grid. Ragged blobs and stepped blocks are laid over one another, and each patch fills its cells with one glyph in one ink: discs, pluses, stripes, squares, ringed discs or dots. Some patches keep only every other cell so the one beneath shows through, some are stitched along their edge, and some are left as bare ground.

motley renders a Still: one frame. Its site page is [playgrnd.tools/motley](https://www.playgrnd.tools/motley/).

## Still

![motley Still for Seed 9](../../../assets/tools/motley.png)

Seed 9, rendered by:

```sh
tirage derive --seed 9 --tool motley | tirage render --size 540x960 -o motley.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
motley  1 frame
  cells       16..=120  step 1
  mark        0.4..=1   step 0.01
  patches     1..=60    step 1
  size        0..=1     step 0.01
  ragged      0..=1     step 0.01
  blocks      0..=1     step 0.01
  sparse      0..=1     step 0.01
  stitch      0..=1     step 0.01
  gaps        0..=0.5   step 0.01
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

A Seed draws all 9 art Parameters. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `mark` starts at 0.75, `sparse` stops at 0.6 and `gaps` at 0.3, because past them the dark ground and thinned patches turn the frame to mush. Dither and the grain post-pass stay off. On the site page a new variation number also moves the sliders. tirage leaves that to the Seed.

The Palette takes any number of inks from 2. The first is the ground between the glyphs, and every patch takes its ink from the rest. With 2 inks every patch is drawn in the second, so ring centres and stitched edges blend into their patch.
