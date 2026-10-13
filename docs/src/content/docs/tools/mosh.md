---
title: "mosh"
description: "Stacked bands of broken video data. It renders a Loop of 24 frames at 12 fps, each band dealt again every frame."
---

A stack of horizontal bands of broken video data, each a different failure: confetti runs, a torn mosaic, long smears, whole scan rows, and a chevron shear. Every band is flat rectangles in a few hard inks, with a share of the frame fallen to the darkest ink.

mosh renders a Loop: 24 frames at 12 fps, in which every band deals its runs, blocks, tears, rows and chevrons again each frame while the bands keep their heights and kinds. Frame 0 is the Still. Its site page is [playgrnd.tools/mosh](https://www.playgrnd.tools/mosh/).

## Still

![mosh Still for Seed 17](../../../assets/tools/mosh.png)

Seed 17, rendered by:

```sh
tirage derive --seed 17 --tool mosh | tirage render --size 540x960 -o mosh.png
```

SVG: yes. `-o mosh.svg` writes this Still as an [SVG](/cli-reference/render/#svg).

## Loop

<video src="/tools/mosh.mp4" width="360" height="640" autoplay loop muted playsinline></video>

The same Seed 17, encoded at 360x640 by:

```sh
tirage derive --seed 17 --tool mosh | tirage encode --size 360x640 -o mosh.mp4
```

## Parameters

From [`tirage tools`](/cli-reference/tools/), without the 11 shared [dither and grain](/concepts/dither-and-grain/) Parameters:

```
mosh  24 frames, svg
  bands       1..=14    step 1
  cols        24..=420  step 2
  mix         0..=1     step 0.01
  tears       0..=1     step 0.01
  runs        0..=1     step 0.01
  bright      0..=1     step 0.01
```

A Seed draws all 6 art Parameters. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed.

The Palette takes any number of inks from 2, picked by position. The second ink cuts in as the bright strips, and the darkest ink fills the dead patches.
