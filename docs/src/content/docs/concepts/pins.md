---
title: "Pins"
description: "Fix part of a Recipe and let the Seed deal the rest."
---

A _Pin_ is a part of a Recipe you fix instead of letting the Seed derive it. You can pin the Tool, the Palette, any Parameter or the Tool seed. Whatever you leave unpinned still comes from the Seed, exactly as it would without the Pin. Pinning a field to the value the Seed would have dealt anyway changes nothing.

## Pin the Tool

Seed 42 deals whorl on its own. Name a Tool and the Seed derives a Recipe for that Tool instead:

```sh
tirage derive --seed 42 --tool sonar
```

The result is the Recipe Seed 42 would have given if it had dealt sonar by itself.

## Pin the Palette

`--palette` takes at least two `#rrggbb` inks, separated by commas:

```sh
tirage derive --seed 42 --tool sonar --palette '#000000,#ffffff'
```

Only `palette` changes. The Tool seed and the Parameters stay as Seed 42 deals them. A Tool reads inks by position and wraps around when the Palette is shorter than it needs. Some Tools cap the Palette at the inks they draw: aura 4, mist 4, warp 5, quilt 4, static 2, rise 3. Without `--palette` the Palette is the Tool's default, and the Seed never changes it, static 2. Without `--palette` the Palette is the Tool's default, and the Seed never changes it. prism takes no Palette, since it fixes its own colours. `--palette` on a Seed that deals prism fails, and the hint says to pin a Tool that takes one with `--tool`.

## Pin a Parameter or the Tool seed

`render` takes a Recipe, not a Seed, so these Pins are edits to the JSON between `derive` and `render`:

```sh
tirage derive --seed 42 --tool sonar > recipe.json
```

Open `recipe.json`, set `level` to `0.9`, then render it:

```sh
tirage render recipe.json --size 1080x1920 -o out.png
```

A pinned value can sit anywhere on the site's slider, outside the [Taste bounds](/concepts/taste-bounds/) the Seed draws from. It does have to stay inside that slider's range, which `render` checks. `tirage tools` lists the range and step of every Parameter. `tool_seed` takes any integer from 1 to 4294967295.
