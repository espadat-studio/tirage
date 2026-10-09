---
title: "stitch"
description: "A coarse pixel grid torn sideways like a glitched video frame. It renders a Still."
---

A coarse grid of flat cells with dark gutters, over a noise field smeared into long vertical streaks. Whole rows of cells slip sideways in bands, the way a glitched video frame tears. An accent ink pools into blobs, and a few stray cells take a random ink.

stitch renders a Still: one frame. Its site page is [playgrnd.tools/stitch](https://www.playgrnd.tools/stitch/).

## Still

![stitch Still for Seed 13](../../../assets/tools/stitch.png)

Seed 13, rendered by:

```sh
tirage derive --seed 13 --tool stitch | tirage render --size 540x960 -o stitch.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
stitch  1 frame
  cols        16..=96   step 1
  gutter      0..=1     step 0.01
  streak      0..=1     step 0.01
  scale       0..=1     step 0.01
  slip        0..=1     step 0.01
  band        1..=16    step 1
  blobs       0..=1     step 0.01
  blobsize    0..=1     step 0.01
  steps       2..=12    step 1
  ground      0..=1     step 0.01
  stray       0..=1     step 0.01
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

A Seed draws all 11 art Parameters. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and the grain post-pass stay off.

The Palette takes any number of inks from 2. The first ink takes the lowest levels, the second is the blob accent, and the rest fill the middle levels in a shuffled order. The gutter takes the darkest ink.
