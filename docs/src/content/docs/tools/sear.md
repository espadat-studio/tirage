---
title: "sear"
description: "A smeared thermal image. It renders a Still."
---

A smeared thermal image. A warped noise field is spread flat so each ink of the ramp covers about the same share of the frame, then cut into flat blocks. Each row holds a colour until the field changes enough, so edges drag into streaks, and some bands of rows tear sideways.

sear renders a Still: one frame. Its site page is [playgrnd.tools/sear](https://www.playgrnd.tools/sear/).

## Still

![sear Still for Seed 12](../../../assets/tools/sear.png)

Seed 12, rendered by:

```sh
tirage derive --seed 12 --tool sear | tirage render --size 540x960 -o sear.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
sear  1 frame
  scale       0..=1     step 0.01
  warp        0..=1     step 0.01
  detail      1..=5     step 1
  smear       0..=1     step 0.01
  drag        0..=1     step 0.01
  tear        0..=1     step 0.01
  steps       2..=24    step 1
  grain       0..=1     step 0.01
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

A Seed draws every Parameter above `grain`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `tear` stays within 0..=0.6. `grain`, the Tool's own grain, stays at 0.1. Dither and the grain post-pass stay off. The Palette is a ramp read coolest first, so the first ink is the lowest heat and the last the peaks.
