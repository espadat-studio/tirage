---
title: "weave"
description: "Horizontal bands of woven cloth, each striped in its own threads. It renders a Still."
---

Bands of woven cloth stacked down the frame. Each band has a ground ink and vertical stripes in one or two thread inks, at its own pitch, width and height. Some bands are fine pinstripes and some are broad bars.

weave renders a Still: one frame. Its site page is [playgrnd.tools/weave](https://www.playgrnd.tools/weave/).

## Still

![weave Still for Seed 1](../../../assets/tools/weave.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool weave | tirage render --size 540x960 -o weave.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
weave  1 frame
  bands       3..=12    step 1
  stripe      0.5..=2.2 step 0.05
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

A Seed draws `bands` and `stripe`, each across its full range. `stripe` scales the pitch of every band. Dither and grain stay off.

The Palette takes any number of inks from 2. The Tool seed picks each band's ground and threads from it, never giving a thread the ground's ink.
