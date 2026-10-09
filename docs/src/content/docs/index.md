---
title: "tirage"
description: "Original generative artwork and animations, rendered from code. One Seed gives one Still or Loop."
---

> Original generative artwork and animations, rendered from code.

tirage reproduces the look of [playgrnd.tools](https://playgrnd.tools) generators. It is not affiliated with them.

Each generator is a _Tool_, named by its site slug: aura, sonar, kiosk and more. A Tool paints with a _Palette_ and takes a few _Parameters_, one per slider on its site page.

A single integer, the _Seed_, derives a whole _Recipe_: the Tool, a value for each Parameter, a Palette and the Tool's own seed. The same Seed always gives the same Recipe. Any Seed gives an on-brand result, because each Parameter is drawn from a hand-picked sub-range.

A Recipe renders a _Visual_. That is either a _Still_, one frame, or a _Loop_, an animation that ends where it starts. A Loop's first frame is the Still of the same Recipe.

## Install and render

```sh
cargo install --git https://github.com/espadat-studio/tirage --features cli
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

`derive` prints the Recipe for Seed 42 as JSON. `render` reads it and writes a 1080x1920 PNG. `tirage tools` lists every Tool, and `tirage --help` lists the rest.
