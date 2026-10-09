---
title: "coral"
description: "A sea fan printed on textile. It renders a Still."
---

A sea fan printed on textile. Ribbed branches grow from a root off the edge of the frame and fork as they spread, so they stay evenly spaced. Torn paper lobes lie over and under them, with a dot screen printed on the paper.

coral renders a Still: one frame. Its site page is [playgrnd.tools/coral](https://www.playgrnd.tools/coral/).

## Still

![coral Still for Seed 8](../../../assets/tools/coral.png)

Seed 8, rendered by:

```sh
tirage derive --seed 8 --tool coral | tirage render --size 540x960 -o coral.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
coral  1 frame
  branches    2..=12    step 1
  spacing     0..=1     step 0.01
  spread      0..=1     step 0.01
  width       0.1..=1   step 0.01
  wobble      0..=1     step 0.01
  cover       0..=1     step 0.01
  size        0..=1     step 0.01
  beneath     0..=1     step 0.01
  ribs        0..=1     step 0.01
  dots        0..=1     step 0.01
  shadow      0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog` across its full range, the [Taste bounds](/concepts/taste-bounds/) shipped for coral. The grain post-pass is on, as the site page opens: Multiply, amount 0.45, specks 0.4, vignette 0.3. Dither stays off. coral reads its Palette by role: ground, branch, rib, lobe and dot. A shorter Palette derives the missing roles from the inks it has.
