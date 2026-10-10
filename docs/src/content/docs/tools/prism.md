---
title: "prism"
description: "Pixel streams of white, hot and cool colour flooding a black dot grid. It renders a Still."
---

An aurora in coarse pixels. A few streams flow in from the corners and edges of a black field. Each cell of the grid is white near the middle of a stream, then yellow and orange, then pink and red, then blue and violet at the edge, and black beyond it. A faint white dot sits in the centre of every cell.

prism renders a Still: one frame. Its site page is [playgrnd.tools/prism](https://www.playgrnd.tools/prism/).

## Still

![prism Still for Seed 1](../../../assets/tools/prism.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool prism | tirage render --size 540x960 -o prism.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
prism  1 frame
  cols        24..=72   step 2
  streams     1..=5     step 1
  reach       0.3..=0.85 step 0.01
  fringe      0.5..=1.6 step 0.01
  dots        0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `cols` sets how many cells span the width. `streams` sets how many streams flow in, `reach` how far each one runs, and `fringe` how wide. `dots` is how strongly the dot grid shows; at 0 it is gone.

Grain and dither stay off, as the site page opens them.

prism takes no Palette. Its field, dots and colour rings are fixed, so its Recipe has no `palette` and `--palette` fails on it.
