---
title: "Installation"
description: "Install the tirage CLI from crates.io with cargo, or depend on the library."
---

The CLI and the library are on crates.io. Both install through cargo.

You need Rust: tirage is built and tested with 1.99, and its 2024 edition needs at least 1.85.

## The CLI

```sh
cargo install --locked tirage-cli
```

cargo downloads the `tirage-cli` crate, builds the `tirage` binary from its `tirage-cli` crate and puts it in `~/.cargo/bin`. That directory has to be on your `PATH`.

```sh
tirage --version
```

```
tirage 0.1.1 (derivation major 0)
```

## The encode feature

`tirage encode`, the command that turns a Loop into an MP4, is behind the `encode` cargo feature:

```sh
cargo install --locked tirage-cli --features encode
```

The feature builds openh264 from source, so it needs a C++ compiler on the `PATH`, such as `g++` or `clang++`. Without the feature the install needs no C++ compiler and the binary has no `encode` command.

## The library

```sh
cargo add tirage
```

Commit the `Cargo.lock` cargo writes and build with `cargo build --locked`. `Cargo.lock` holds the version of tirage and of every dependency. With `--locked`, cargo fails the build instead of resolving them again.

`tirage-encode`, the crate behind `tirage encode`, is on crates.io too. It takes a Recipe and returns the MP4 bytes, and it needs the same C++ compiler.
