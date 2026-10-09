---
title: "completions"
description: "Print a shell completion script."
---

```sh
tirage completions [OPTIONS] <SHELL>
```

Prints a completion script for `SHELL` on stdout. Save it where your shell loads completions from, and the Tab key completes commands, aliases, options and Tool slugs.

`SHELL` is one of `bash`, `elvish`, `fish`, `powershell` or `zsh`.

`-h`, `--help`, `--no-color` and `--no-input` work here as on every command, though none of them changes the script. See [tirage](/cli-reference/tirage/).

## Install

```sh
tirage completions bash > ~/.local/share/bash-completion/completions/tirage
tirage completions fish > ~/.config/fish/completions/tirage.fish
```

For zsh, write the script to a file named `_tirage` in a directory on your `fpath`. Open a new shell afterwards.

## Exit codes

| Code | When                                   |
| ---- | -------------------------------------- |
| `0`  | The script was printed                 |
| `2`  | No shell given, or one not in the list |
