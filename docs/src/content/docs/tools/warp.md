---
title: "warp"
description: "Op-art checks or slanted bars, bent through a wave, a taper or a bulge. It renders a Loop of 72 frames at 12 fps, the bend deepening and relaxing."
---

Op-art in one ink on one ground. A checkerboard or a field of slanted bars is bent through a wave, a taper or a bulge. The shapes are polygons, filled in one pass.

warp renders a Loop: 72 frames at 12 fps, in which the bend deepens and relaxes once while the pattern and inks stay put. Each Seed starts the swing at its own point, so frame 0 differs from the Still. Its site page is [playgrnd.tools/warp](https://www.playgrnd.tools/warp/).

## Still

![warp Still for Seed 1](../../../assets/tools/warp.png)

Seed 1, rendered by:

```sh
tirage derive --seed 1 --tool warp | tirage render --size 540x960 -o warp.png
```

SVG: yes. `-o warp.svg` writes this Still as an [SVG](/cli-reference/render/#svg).

## Parameters

From [`tirage tools`](/cli-reference/tools/), without the 11 shared [dither and grain](/concepts/dither-and-grain/) Parameters:

```
warp  72 frames, svg
  styles      Checker, Slash
  scale       0.5..=2   step 0.05
  warp        0..=1     step 0.01
```

A Seed draws `styles`, `scale` and `warp`, each across its full range. The site's Auto style is left out: it picks checks or bars from the Tool seed, so a Recipe names that style instead.

The Palette has 5 inks by role: a Base, then Ink 1 to Ink 4. warp takes at most 5. Each Still paints the Base and one of the four inks, and the Tool seed picks which ink and which of the two is the ground.
