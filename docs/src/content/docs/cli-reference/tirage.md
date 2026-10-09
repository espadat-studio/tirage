---
title: "tirage"
description: "Commands, global options, prompts, colour and exit codes shared by every tirage command."
---

```sh
tirage [OPTIONS] <COMMAND>
```

One binary, five commands, and a sixth in a build with the `encode` feature. A run usually pipes the first into the second.

| Command                                    | Alias     | Does                                                     |
| ------------------------------------------ | --------- | -------------------------------------------------------- |
| [derive](/cli-reference/derive/)           | `d`       | Derive a Recipe from a Seed and print it as JSON         |
| [render](/cli-reference/render/)           | `r`       | Render a Recipe to a PNG                                 |
| [encode](/cli-reference/encode/)           | `e`       | Encode a Loop to an H.264 MP4, with the `encode` feature |
| [roll](/cli-reference/roll/)               |           | Roll a random Seed and write its Visual to a file        |
| [tools](/cli-reference/tools/)             | `t`, `ls` | List Tools with their frame counts and Parameters        |
| [completions](/cli-reference/completions/) |           | Print a shell completion script                          |

```sh
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

`tirage help <command>` prints the same text as `tirage <command> --help`. Running `tirage` with no arguments prints a short reminder on stderr and exits 2.

## Options

`-h`, `--help`, `--no-color` and `--no-input` work on every command. `--version` only works on `tirage` itself.

| Option       | Does                                                                         |
| ------------ | ---------------------------------------------------------------------------- |
| `-h`         | Print short help: one line per option                                        |
| `--help`     | Print long help: full descriptions, ranges and prompt defaults               |
| `--version`  | Print the version and the derivation major                                   |
| `--no-color` | Turn off colour on stderr. A non-empty `NO_COLOR` or `TERM=dumb` do the same |
| `--no-input` | Never prompt. `TIRAGE_NO_INPUT=1` or `TERM=dumb` do the same                 |

## Prompts

On a terminal, derive, render and encode ask for the values you leave out. derive asks for a Seed. render asks for a Seed and a Tool when nothing is piped in, then for a frame size and an output path. encode asks for a Seed and a Loop Tool, then for an output path. Every prompt has a default, so pressing Enter through all of them gives a random Visual as a 1080x1920 PNG, or a random Loop as an MP4.

Once the prompts are answered, the command you could have typed instead is printed on stderr:

```
→ tirage derive --tool sonar --seed 1234
```

Prompts are off when stdin or stderr is not a terminal, with `--no-input`, with `TIRAGE_NO_INPUT=1`, or when `TERM` is `dumb`. With prompts off, a missing value is a usage error, and the message names the flag to pass.

## Colour

Errors and hints are coloured when stderr is a terminal. `--no-color`, a non-empty `NO_COLOR`, or `TERM=dumb` turn colour off. Help and listings are plain text, and stdout never carries colour codes.

## Version

```
$ tirage --version
tirage 0.1.0 (derivation major 0)
```

The derivation major is the version of the Seed to Recipe mapping. A Recipe records the major it came from in its `tirage` field, and a build reads only Recipes of its own major. A Seed gives the same Recipe for as long as the major stays the same. Pixels are not promised across releases.

## Exit codes

| Code  | When                                                                                                                                                                   |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`   | Done                                                                                                                                                                   |
| `1`   | Something failed at run time: a file could not be read or written, the Recipe JSON is invalid or from another major, or the frame size or frame number is out of range |
| `2`   | The command line is wrong: an unknown command or option, a bad value, a value missing with prompts off, a PNG or MP4 aimed at the terminal, or no arguments at all     |
| `130` | A prompt was cancelled with Esc or Ctrl-C                                                                                                                              |

Errors go to stderr as `error: <message>`. Most are followed by a `hint:` line with the fix, like the closest Tool slug after a typo.
