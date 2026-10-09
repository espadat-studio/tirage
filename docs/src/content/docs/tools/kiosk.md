---
title: "kiosk"
description: "A pasted-up street poster under a grid of characters. It renders a Loop of 24 frames at 6 fps."
---

A pasted-up street poster. Big arcs of colour sweep in from a centre off the page, a block of vertical stripes covers the right part, a column of halftone and solid tiles runs down the right edge, and one dark panel sits low on the page. Over all of it goes a grid of bold monospace characters: a coarse rank of big ones at the cell centres, and smaller ones packed into the corners between them.

kiosk renders a Loop: 24 frames at 6 fps. Each frame deals the characters, their cells and their inks again, while the arcs, stripes, tiles and panel stay put. Frame 0 is the Still. Its site page is [playgrnd.tools/kiosk](https://www.playgrnd.tools/kiosk/).

## Still

![kiosk Still for Seed 5](../../../assets/tools/kiosk.png)

Seed 5, rendered by:

```sh
tirage derive --seed 5 --tool kiosk | tirage render --size 540x960 -o kiosk.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
kiosk  24 frames
  split       0.15..=0.95 step 0.01
  rings       4..=40    step 1
  stripes     2..=24    step 1
  sets        DOS, Stipple, Blocks, Code, Digits, Runes
  grid        4..=30    step 1
  bigSize     0.3..=1.6 step 0.01
  density     0..=1     step 0.01
  smallSize   0.1..=0.9 step 0.01
  small       0..=1     step 0.01
  blocks      0..=1     step 0.01
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

A Seed draws `split`, `rings`, `stripes`, `sets`, `grid`, `bigSize`, `density`, `smallSize`, `small` and `blocks`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and grain stay off.
