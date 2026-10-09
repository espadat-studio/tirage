---
title: "Taste bounds"
description: "Why any Seed gives a Visual that looks right, and how to narrow or widen what it draws from."
---

A slider on the site runs further than looks good. Push sonar's `level` to either end and the frame is nearly all water or nearly all land. The site allows it, but you would not pick it.

_Taste bounds_ are the sub-range of each Parameter that a Seed may draw from. They are picked by hand, per Tool, so that any Seed gives an on-brand Visual. They narrow only what the Seed deals: a [Pin](/concepts/pins/) can still use the whole slider.

Every Tool ships one set. sonar draws `level` from 0.25 to 0.7 and `grid` from 140 to 300, and draws its five other sliders from their full range. `tirage tools` prints the full range and step of every Parameter.

## Your own Taste bounds

Pass `derive --taste` a JSON file that names the Tool and gives `[min, max]` for each Parameter you want to change. Parameters you leave out keep the shipped bounds. Both ends have to be values the slider can take. The file pins its Tool, so `--taste` and `--tool` do not go together.

This file spells out sonar's shipped bounds, so it deals exactly what `--tool sonar` does:

```sh
cat > shipped.json <<'EOF'
{"tool":"sonar","level":[0.25,0.7],"grid":[140,300]}
EOF
tirage derive --seed 42 --taste shipped.json
```

Narrow `level` to the top of its slider and Seed 42 follows:

```sh
cat > high-water.json <<'EOF'
{"tool":"sonar","level":[0.9,1]}
EOF
tirage derive --seed 42 --taste high-water.json
```

`level` comes out at 0.96 instead of 0.64. Every other field is unchanged, because each one is drawn on its own.
