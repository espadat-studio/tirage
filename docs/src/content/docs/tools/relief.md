---
title: "relief"
description: "Tumbling blocks: a field of cubes seen from a corner. It renders a Still."
---

Tumbling blocks. Cubes seen from a corner tile the frame, each one a hexagon cut into a top, a left and a right face. The three faces are one ink at three lightnesses, so a flat shape reads as a lit solid. A small block of cubes is worked out once and repeated. Some cubes have their three tones turned round, which breaks the plain field into hooks. Some are painted in one flat tone, and some take an accent ink.

relief renders a Still: one frame. Its site page is [playgrnd.tools/relief](https://www.playgrnd.tools/relief/).

## Still

![relief Still for Seed 2](../../../assets/tools/relief.png)

Seed 2, rendered by:

```sh
tirage derive --seed 2 --tool relief | tirage render --size 540x960 -o relief.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
relief  1 frame
  scale       12..=180  step 1
  repeat      1..=12    step 1
  variety     0..=1     step 0.01
  solids      0..=1     step 0.01
  relief      0..=1     step 0.01
  accent      0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `scale` stays within 12..=120 and `solids` within 0..=0.75. Dither and grain stay off. `scale` is the cube size in pixels and `repeat` the cubes per side of the repeated block. The first ink paints most cubes, and every later ink is an accent.
