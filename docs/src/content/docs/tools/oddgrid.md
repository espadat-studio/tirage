---
title: "oddgrid"
description: "A patchwork of hard-edged pixel regions. It renders a Still."
---

A patchwork of hard-edged pixel regions on a coarse grid. Three noise fields decide each cell: one picks its colour, one decides whether it is filled at all, and a fine one breaks the region edges into pixels. Blocks of cells can be pulled toward a tone of their own, so the regions square off into a quilt. Some filled cells carry a small motif.

oddgrid renders a Still: one frame. Its site page is [playgrnd.tools/oddgrid](https://www.playgrnd.tools/oddgrid/).

## Still

![oddgrid Still for Seed 4](../../../assets/tools/oddgrid.png)

Seed 4, rendered by:

```sh
tirage derive --seed 4 --tool oddgrid | tirage render --size 540x960 -o oddgrid.png
```

## Parameters

As [`tirage tools`](/cli-reference/tools/) lists them, each with its range and step, or its choices:

```
oddgrid  1 frame
  cols        12..=110  step 1
  scale       2..=60    step 0.5
  density     0..=1     step 0.01
  grain       0..=1.4   step 0.01
  variety     0..=1.6   step 0.01
  block       0..=1     step 0.01
  bsize       2..=16    step 1
  speck       0..=0.6   step 0.01
  motifs      None, Dot, Ring, Square, Wedge, Mixed
  motifAmt    0..=1     step 0.01
  markSize    0.15..=1  step 0.01
  balance     -1.2..=1.2 step 0.05
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

A Seed draws `motifs` and all 11 art ranges. Each range comes from its [Taste bounds](/concepts/taste-bounds/), which sit within the range listed: `density` starts at 0.4, because below it the grid empties to bare ground, and `speck` stops at 0.45, past which the frame turns to noise. Dither and the grain post-pass stay off. The `grain` among the art ranges is the Tool's own: it breaks up region edges. The site page also offers a motif from an uploaded SVG and a few Looks that set several sliders at once. tirage leaves both out.

The Palette is the cell inks, any number from 2. `balance` leans the colour field toward the first or the last of them. The ground `#101820` and the motif ink `#f2ede4` are fixed, as on the site page by default.
