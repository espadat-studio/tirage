---
title: "atlas"
description: "A map drawn in monospace type. It renders a Still."
---

A map drawn in monospace type. Warped noise splits the page into terraces. Each terrace fills its cells with one ink and scatters characters over them from its own part of a character set, which makes the terraces look like different materials. Higher terraces carry more type.

atlas renders a Still: one frame. Its site page is [playgrnd.tools/atlas](https://www.playgrnd.tools/atlas/).

## Still

![atlas Still for Seed 9](../../../assets/tools/atlas.png)

Seed 9, rendered by:

```sh
tirage derive --seed 9 --tool atlas | tirage render --size 540x960 -o atlas.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
atlas  1 frame
  scale       1..=9     step 0.1
  warp        0..=1.6   step 0.01
  bands       2..=14    step 1
  mix         0..=1     step 0.01
  sets        DOS, Stipple, Blocks, Code, Digits, Runes
  cols        24..=260  step 1
  density     0..=1     step 0.01
  weight      0..=1.5   step 0.01
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

A Seed draws all 7 art ranges and the character set. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `bands` starts at 3, because 2 terraces leave one flat ground. Dither and the grain post-pass stay off. `weight` sets how much of the character set each terrace draws from, not how heavy the type is. tirage leaves out the site's Custom set.

The Palette takes any number of inks from 2. Terraces take their ground ink by position, wrapping when there are more terraces than inks. The type is the lightest ink, or the darkest one where the ground is closer to the lightest.
