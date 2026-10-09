---
title: "sonar"
description: "A one-bit chart of a coastline. It renders a Loop of 24 frames at 10 fps."
---

A one-bit chart of a coastline. A smooth random height field is cut at a water level. Land above the cut is a grid of square dots, sized by how far inland they sit. A thin band either side of the cut breaks up into sparse coloured specks.

sonar renders a Loop: 24 frames at 10 fps. The land keeps its shape while the water level rises and falls once per Loop, so the coast moves in and out. Frame 0 is the Still. Its site page is [playgrnd.tools/sonar](https://www.playgrnd.tools/sonar/).

## Still

![sonar Still for Seed 1](../../../assets/tools/sonar.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool sonar | tirage render --size 540x960 -o sonar.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
sonar  24 frames
  level       0..=1     step 0.01
  scale       1..=10    step 0.1
  warp        0..=1     step 0.01
  grid        40..=320  step 2
  depth       0..=1     step 0.01
  fringe      0..=1     step 0.01
  spark       0..=1     step 0.01
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

A Seed draws `level`, `scale`, `warp`, `grid`, `depth`, `fringe` and `spark`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and grain stay off.
