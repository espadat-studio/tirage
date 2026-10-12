---
title: "aura"
description: "A soft glow of melting inks. It renders a Loop of 72 frames at 12 fps, the inks sliding round a circle over a fixed warp."
---

A soft glow of melting inks. Each ink owns a warped noise field. Every pixel mixes all inks, weighted by how strong each field is there, so the inks bleed into tints with no hard edge. The field is painted on a small buffer and scaled up smooth.

aura renders a Loop: 72 frames at 12 fps, in which the point every ink field reads drifts once round a circle, so all the inks slide together. The warp under them holds still, so the blots bend as they pass through it. The inks are already shifted at frame 0, so frame 0 differs from the Still. Its site page is [playgrnd.tools/aura](https://www.playgrnd.tools/aura/).

## Still

![aura Still for Seed 4](../../../assets/tools/aura.png)

Seed 4, rendered by:

```sh
tirage derive --seed 4 --tool aura | tirage render --size 540x960 -o aura.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
aura  72 frames
  styles      Auto, Clouds, Mesh, Sweep
  scale       0.5..=2   step 0.05
  churn       0..=1     step 0.01
  punch       0..=1     step 0.01
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

A Seed draws `styles`, `scale`, `churn` and `punch`. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed. Dither and grain stay off.
