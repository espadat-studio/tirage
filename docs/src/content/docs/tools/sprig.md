---
title: "sprig"
description: "A hand-drawn repeat of little plants. It renders a Still."
---

A hand-drawn repeat of little plants. Blooms, leaves, petals, daisies, dots and stems are spread evenly over a plain ground in one ink. Each line swells and thins with a ragged edge, as a marker would lay it down. The pattern tiles: whatever runs off one edge comes back on the other.

sprig renders a Still: one frame. Its site page is [playgrnd.tools/sprig](https://www.playgrnd.tools/sprig/).

## Still

![sprig Still for Seed 4](../../../assets/tools/sprig.png)

Seed 4, rendered by:

```sh
tirage derive --seed 4 --tool sprig | tirage render --size 540x960 -o sprig.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
sprig  1 frame
  dirs        Garden, Blooms, Leaves
  count       2..=120   step 1
  size        0..=1     step 0.01
  vary        0..=1     step 0.01
  solids      0..=1     step 0.01
  weight      0..=1     step 0.01
  rough       0..=1     step 0.01
  wobble      0..=1     step 0.01
  detail      0..=1     step 0.01
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

A Seed draws `dirs` and all 8 art ranges. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `count` runs 20 to 100, `size` up to 0.6 and `weight` up to 0.8, because past them the ink floods the ground. Dither and the grain post-pass stay off. `dirs` picks the mix of motifs: Garden has them all, Blooms leaves out leaves and stems, and Leaves is mostly leaves.

The Palette takes 2 inks: the first is the ground, the second the line. Inks after the second are not drawn.
