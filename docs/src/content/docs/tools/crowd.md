---
title: "crowd"
description: "A crowd of heads and shoulders in false colour, under patches of small arrows. It renders a Still."
---

A crowd seen through a heat camera. Each figure, a head on a pair of shoulders, is a soft blob in false colour: ground where nobody stands, a halo with a dark rim round every figure, and a solid core. Small arrows gather in patches over the top, each patch leaning its own way.

crowd renders a Still: one frame. Its site page is [playgrnd.tools/crowd](https://www.playgrnd.tools/crowd/).

## Still

![crowd Still for Seed 1](../../../assets/tools/crowd.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool crowd | tirage render --size 540x960 -o crowd.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
crowd  1 frame
  crowd       1..=48    step 1
  scale       0..=1     step 0.01
  wobble      0..=1     step 0.01
  blur        0..=1     step 0.01
  halo        0..=1     step 0.01
  edge        0..=1     step 0.01
  arrows      0..=1     step 0.01
  arrowSize   0..=1     step 0.01
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

A Seed draws `crowd` from 8 to 48 and `scale` from 0 to 0.6. One figure leaves the frame almost bare, and big figures flood it with core ink. Every other slider spans its full range. `crowd` is how many figures stand in the frame, `halo` how wide the band round each one is, and `edge` how dark its rim is. `arrows` sets how much of the frame the arrows cover, and `arrowSize` their size and spacing. Dither stays off. Grain stays on as the site opens it: overlay at 0.35.

The Palette takes any number of inks from 2, read by role. The first ink is the ground, the second the core of every figure and the arrows. Any further inks are halo bands, outermost first. With 2 inks the halo is the core ink.
