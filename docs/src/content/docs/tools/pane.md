---
title: "pane"
description: "Rows of uneven cells, each a colour ramp between two inks, like a stained glass window. It renders a Still."
---

Stained glass made of colour ramps. The frame splits into rows of uneven height, and each row splits into its own count of uneven cells, so the cell edges never line up from top to bottom. Every cell fades from one ink to another along an edge or from corner to corner. The fade turns through the colour wheel the short way round, so two opposite inks meet through bright colour rather than grey.

pane renders a Still: one frame. Its site page is [playgrnd.tools/pane](https://www.playgrnd.tools/pane/).

## Still

![pane Still for Seed 1](../../../assets/tools/pane.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool pane | tirage render --size 540x960 -o pane.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
pane  1 frame
  rows        1..=10    step 1
  cells       1..=16    step 1
  vary        0..=1     step 0.01
  diag        0..=1     step 0.01
  soft        0..=1     step 0.01
  spread      0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`, each across its full range. `rows` sets how many rows there are and `cells` how many cells a row has on average. `vary` makes the row heights and cell widths uneven. `diag` is the chance that a cell fades from corner to corner instead of along an edge. `soft` sets how much of a cell is the fade: at 0 the two inks meet on a hard line. `spread` sets how far apart in the Palette a cell's two inks sit.

Grain and dither stay off, as the site page opens them.

The Palette takes any number of inks from 2, and no ink has a fixed role. Each cell picks its own two.
