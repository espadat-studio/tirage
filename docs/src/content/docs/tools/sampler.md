---
title: "sampler"
description: "Bands of hard-edged tiles, each turned a quarter at a time, in two inks over a dark ground. It renders a Still."
---

A sampler of flat, straight-edged marks. The frame is a grid of cells cut into horizontal bands. Each band uses one to a few tile shapes: triangles, diagonals, bars, elbows, steps, halves, notches or arrows. It fills its cells with them in two inks, and turns each tile a quarter at a time. The darkest ink is the ground and shows through the empty cells.

sampler renders a Still: one frame. Its site page is [playgrnd.tools/sampler](https://www.playgrnd.tools/sampler/).

## Still

![sampler Still for Seed 1](../../../assets/tools/sampler.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool sampler | tirage render --size 540x960 -o sampler.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
sampler  1 frame
  grid        4..=48    step 1
  bands       1..=14    step 1
  mix         0..=1     step 0.01
  turn        0..=1     step 0.01
  density     0.1..=1   step 0.01
  weight      0..=1     step 0.01
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

A Seed draws every Parameter but the shared grain and dither, which stay off. `grid` sets the number of columns, and the rows follow from the frame's shape. `bands` sets how many bands the rows aim for, and `mix` how many tile shapes one band may use. `turn` sets how freely a tile turns away from its neighbours, and `weight` tips each band toward its first ink.

`density` is the share of cells that get a tile. A Seed draws it from 0.55 to 1: below that, most of the frame is bare ground with a few small marks scattered on it.

The Palette takes any number of inks. The darkest one is the ground, and each band picks its two tile inks from the rest.
