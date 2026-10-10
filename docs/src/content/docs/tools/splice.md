---
title: "splice"
description: "A screen-printed collage of one picture cut into slices. It renders a Still."
---

A screen-printed collage. One field of soft hills is cut into tones, with a dark line where two tones meet. The sheet is then cut into slices, and each slice prints the same field shifted along, through its own screen and its own turn of the inks. A few long arcs are drawn over the top.

splice renders a Still: one frame. Its site page is [playgrnd.tools/splice](https://www.playgrnd.tools/splice/).

## Still

![splice Still for Seed 2](../../../assets/tools/splice.png)

Seed 2, rendered by:

```sh
tirage derive --seed 2 --tool splice | tirage render --size 540x960 -o splice.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
splice  1 frame
  dirs        Columns, Rows, Blocks
  slices      1..=40    step 1
  shift       0..=1     step 0.01
  scale       0..=1     step 0.01
  tones       2..=14    step 1
  key         0..=1     step 0.01
  screen      0..=1     step 0.01
  mix         0..=1     step 0.01
  grain       0..=1     step 0.01
  arcs        0..=24    step 1
  weight      0..=1     step 0.01
  bend        0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog` except `grain`, each across its full range. `dirs` cuts the sheet into columns, rows or blocks, and `slices` is how many pieces there are. `mix` is the chance that a piece picks its own screen: dots, lines, noise or flat. `grain`, the Tool's own per-pixel noise, stays at 0.18. Dither and the grain post-pass stay off.

The Palette takes any number of inks from 2. The first ink draws the lines between tones and also prints as a tone. The arcs are drawn in Palette inks at 85% opacity.
