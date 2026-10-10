---
title: "derive"
description: "Derive a Recipe from a Seed and print it as JSON."
---

```sh
tirage derive [OPTIONS]
tirage d [OPTIONS]
```

Turns a Seed into a Recipe and prints it on stdout as one line of JSON. Nothing is rendered. Pipe the line into [render](/cli-reference/render/), or save it and edit it.

The same Seed with the same Pins gives the same Recipe, for as long as the derivation major stays the same.

## Options

| Option             | Value                                                                                          | Without it                                            |
| ------------------ | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `--seed <N>`       | An integer from 0 to 18446744073709551615                                                      | Prompted on a terminal, with a random Seed as default |
| `--tool <SLUG>`    | A Tool slug, as [`tirage tools`](/cli-reference/tools/) lists it                               | The Seed deals the Tool                               |
| `--taste <FILE>`   | Path to a Taste bounds JSON file. It Pins the Tool named inside, so it conflicts with `--tool` | Each Parameter draws from its shipped Taste bounds    |
| `--palette <INKS>` | Two or more `#rrggbb` inks, comma separated                                                    | The Tool's own Palette                                |

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command. See [tirage](/cli-reference/tirage/).

## Seed

Without `--seed` on a terminal, derive asks for one and offers a random Seed as default. Once it has the Seed, it prints the full command on stderr so you can rerun it:

```
→ tirage derive --tool sonar --seed 1234
```

With prompts off, a missing Seed exits 2.

## Tool

Without `--tool`, the Seed deals one. Each Tool hashes against the Seed and the highest hash wins, so adding a Tool later moves only the Seeds that Tool wins. Every other Seed keeps its Tool. `tirage tools` lists the slugs.

## Taste bounds

A Taste bounds file narrows the range a Seed may draw from. It is a JSON object with the Tool's slug under `tool`, and `[min, max]` under each Parameter id you want to narrow:

```json
{ "tool": "sonar", "level": [0.2, 0.4], "grid": [100, 120] }
```

```sh
tirage derive --seed 42 --taste taste.json
```

Parameters you leave out keep their shipped bounds. Only slider Parameters take bounds. Each bound must be a slider value, inside the Parameter's range and on its step, with min at or below max. `tirage tools` prints the range and step of every Parameter. A bound off its slider or an unknown Parameter id exits 2. A file that cannot be read exits 1.

## Palette

`--palette` Pins the inks a Tool paints with. Pass two or more `#rrggbb` colours separated by commas. Some Tools cap the inks, as [Pins](/concepts/pins/) lists. prism takes no Palette, so `--palette` fails on it. Uppercase hex is accepted and printed lowercase.

```sh
tirage derive --seed 42 --tool sonar --palette '#000000,#ffffff'
```

Quote the value, since `#` starts a comment in most shells.

## Output

One line of JSON, ending in a newline. Pretty printed, the Recipe for Seed 42 Pinned to sonar reads:

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

| Field       | Holds                                                                                                 |
| ----------- | ----------------------------------------------------------------------------------------------------- |
| `tirage`    | The derivation major this Recipe was derived under                                                    |
| `tool`      | The Tool's slug                                                                                       |
| `tool_seed` | The integer the Tool's own random draws start from, the one typed into its site page                  |
| `palette`   | The inks, as `#rrggbb`. Left out for prism, which takes no Palette                                    |
| `params`    | One value per Parameter: a number for a slider, `true` or `false` for a toggle, a string for a choice |

Editing the file is how you Pin anything derive has no flag for. Change a value under `params`, then render the file. render rejects a value outside its range, an unknown Parameter, a duplicate key, and a Recipe from another major.

## Exit codes

| Code | When                                                                                                                                                                       |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | A Recipe was printed                                                                                                                                                       |
| `1`  | The Taste bounds file could not be read                                                                                                                                    |
| `2`  | A bad value: an unknown Tool, a malformed ink, too many inks for the Tool, a Taste bounds file that is not valid JSON, a bound off its slider, or no Seed with prompts off |

## Examples

```sh
tirage derive --seed 42 > recipe.json
tirage derive --seed 42 --tool sonar --palette '#000000,#ffffff'
tirage derive --seed 42 --taste taste.json
```
