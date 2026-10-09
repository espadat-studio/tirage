---
title: "Installation"
description: "Install the tirage CLI from git with cargo, or depend on the library pinned to a commit."
---

tirage is not on crates.io or npm. The CLI and the library both come straight from the git repository, through cargo.

You need Rust: tirage is built and tested with 1.99, and its 2024 edition needs at least 1.85.

## The CLI

```sh
cargo install --git https://github.com/espadat-studio/tirage tirage-cli
```

cargo clones the repository, builds the `tirage` binary from its `tirage-cli` crate and puts it in `~/.cargo/bin`. That directory has to be on your `PATH`.

```sh
tirage --version
```

```
tirage 0.1.1 (derivation major 0)
```

## The encode feature

`tirage encode`, the command that turns a Loop into an MP4, is behind the `encode` cargo feature:

```sh
cargo install --git https://github.com/espadat-studio/tirage tirage-cli --features encode
```

The feature builds openh264 from source, so it needs a C++ compiler on the `PATH`, such as `g++` or `clang++`. Without the feature the install needs no C++ compiler and the binary has no `encode` command.

## The library

Depend on `tirage` by git, pinned to a commit:

```toml
[dependencies]
tirage = { git = "https://github.com/espadat-studio/tirage", rev = "e43e5b8" }
```

Commit the `Cargo.lock` cargo writes and build with `cargo build --locked`. `Cargo.lock` holds the full commit the pin resolved to and the version of every dependency. With `--locked`, cargo fails the build instead of resolving them again.

The same repository holds `tirage-encode`, the crate behind `tirage encode`. It takes a Recipe and returns the MP4 bytes, and it needs the same C++ compiler.
