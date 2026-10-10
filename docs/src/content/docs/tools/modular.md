---
title: "modular"
description: "A Swiss grid poster of flat, ramped, dotted and ruled modules under thin rules. It renders a Still."
---

A grid poster in the Swiss style. The frame is cut into modules, and some of them span two cells. Each module is left empty, filled flat, filled with a ramp between two inks, scattered with blocks, packed with dots in one corner, or ruled like graph paper. Thin rules run over the whole grid.

modular renders a Still: one frame. Its site page is [playgrnd.tools/modular](https://www.playgrnd.tools/modular/).

## Still

![modular Still for Seed 1](../../../assets/tools/modular.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool modular | tirage render --size 540x960 -o modular.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
modular  1 frame
  gcols       2..=12    step 1
  unit        2..=12    step 1
  merge       0..=1     step 0.01
  wEmpty      0..=100   step 1
  wSolid      0..=100   step 1
  wBlocks     0..=100   step 1
  wDots       0..=100   step 1
  wLines      0..=100   step 1
  wGrad       0..=100   step 1
  blockFill   0.1..=0.9 step 0.01
  dot         0.2..=1   step 0.01
  rules       0..=1     step 0.01
  ruleW       0.5..=4   step 0.5
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `gcols` is the number of grid columns, and the rows follow from the frame's shape. `unit` is how many small cells cross one module, and `merge` is the chance that a module spans two cells. The six weights, `wEmpty` to `wGrad`, decide how often each kind of module turns up. `blockFill` is the share of small cells that blocks and dots fill, and `dot` is the dot size. `rules` is how strongly the grid rules show, and `ruleW` is the width of every rule. Grain and dither stay off.

The Palette takes any number of inks from 2, read by position. Each module picks one ink, plus a second for its ramp or its backing panel. The cream ground and the near-black rules are fixed by modular and are not part of the Palette.
