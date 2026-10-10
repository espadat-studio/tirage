---
title: "Seed and Recipe"
description: "How one integer turns into a whole Recipe, and what tirage promises about that mapping."
---

A _Tool_ is one of the things tirage draws, such as sonar, aura or mist. [`tirage tools`](/cli-reference/tools/) lists them all. Each is named by the slug of its page on the site it reproduces. A Tool has _Parameters_, one per slider, toggle or picker on that page, with the same id and the same range as the control there. It paints with a _Palette_, an ordered list of inks. Its own random draws start from a _Tool seed_, the integer you would type into the seed box on that page.

A _Recipe_ is a Tool, a value for each of its Parameters, a Palette where the Tool takes one, and a Tool seed. That is everything a render needs, so a Recipe fully determines one [Visual](/concepts/still-and-loop/).

`tirage derive` prints a Recipe as one line of JSON:

```sh
tirage derive --seed 42 --tool sonar
```

Spread over lines, Seed 42 gives sonar this Recipe:

```json
{
  "tirage": 0,
  "tool": "sonar",
  "tool_seed": 4243409203,
  "palette": ["#0a0f1c", "#3ddc97", "#4361ee", "#ffd166", "#ef476f", "#f1faee"],
  "params": {
    "level": 0.64,
    "scale": 6.3,
    "warp": 0.14,
    "grid": 286,
    "depth": 0.58,
    "fringe": 0.68,
    "spark": 0.09,
    "ditherTog": false,
    "dthKinds": "Bayer 8",
    "dthSize": 2,
    "dthLevels": 3,
    "dthAmount": 1.0,
    "grainTog": false,
    "grnBlends": "Add",
    "grnAmount": 0.55,
    "grnSize": 1.0,
    "grnSpecks": 0.5,
    "grnVignette": 0.5
  }
}
```

`tirage render` reads that JSON and draws it. The Seed is not in it: once the Recipe exists, the Seed has done its job. The `tirage` field is the derivation major, explained below.

## From Seed to Recipe

A _Seed_ is one unsigned 64-bit integer. It derives a whole Recipe, and the same Seed with the same [Pins](/concepts/pins/) always gives the same Recipe.

Each field is drawn on its own. A draw hashes the Seed together with the name of what it is for: sonar's `level` comes from the Seed, the word `sonar` and the word `level`, mapped into that Parameter's [Taste bounds](/concepts/taste-bounds/). No draw reads another draw. Adding a Parameter to a Tool, or listing its Parameters in another order, leaves every existing value where it was.

Two things are not drawn. The Palette is the Tool's default unless you pin one. The grain and dither Parameters stay at the site's defaults, both off, unless you pin them.

The Tool seed is drawn the same way and is never 0, because the site reads a seed of 0 as 1.

## Which Tool a Seed gets

With no Tool pinned, each Tool hashes the Seed with its own slug and the highest hash wins. The order Tools are registered in plays no part. Adding the nth Tool moves only the Seeds it wins, about one in n. The rest keep their Tool.

Seed 42 deals whorl. Pin sonar with `--tool sonar` and the same Seed gives the Recipe above, with every Parameter drawn as if sonar had won.

## What stays fixed

Seed to Recipe is stable within a derivation major. The major is the `tirage` field of the JSON. `tirage --version` prints the crate version and then the derivation major in brackets:

```sh
tirage --version
```

Any change that would hand some Seed a different Recipe bumps the major. A build refuses a Recipe written under another major and asks you to derive it again from its Seed. Until 1.0 the major stays at 0 while the derivation settles. From 1.0 on, every change to it bumps the major.

Pixels are not promised. Rendering can change within a major when a Tool moves closer to the output of its site page, so a Seed's Visual may shift a little while its Recipe stays the same.
