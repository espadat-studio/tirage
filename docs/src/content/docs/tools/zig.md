---
title: "zig"
description: "Two-ink stripes with zigzag, stepped or wavy edges. It renders a Still."
---

Two-ink stripes. The frame is one ink, and every other band between a stack of boundary lines is filled with a second. The lines are teeth, chevrons, stairs, ricrac, waves or scales, and every corner is rounded off.

zig renders a Still: one frame. Its site page is [playgrnd.tools/zig](https://www.playgrnd.tools/zig/).

## Still

![zig Still for Seed 1](../../../assets/tools/zig.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool zig | tirage render --size 540x960 -o zig.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
zig  1 frame
  styles      Teeth, Chevron, Stairs, Ricrac, Waves, Scales
  width       0.5..=2   step 0.05
  depth       0.2..=1.4 step 0.02
  tooth       0.5..=2   step 0.05
  round       0..=1     step 0.01
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

A Seed draws every Parameter above `ditherTog`, each across its full range. The site's Auto style is left out: it picks one of the six from the Tool seed, so a Recipe names that style instead. Dither and grain stay off.

The Palette takes any number of inks from 2. Each Still paints two of them, and the Tool seed picks the pair.
