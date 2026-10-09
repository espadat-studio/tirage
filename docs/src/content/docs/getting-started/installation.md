---
title: "Installation"
description: "Install the tirage CLI from git with cargo, or depend on the library pinned to a commit."
---

tirage is not on crates.io or npm. The CLI and the library both come straight from the git repository, through cargo.

You need Rust: tirage is built and tested with 1.99, and its 2024 edition needs at least 1.85.

## The CLI

```sh
cargo install --git https://github.com/espadat-studio/tirage --features cli
```

cargo clones the repository, builds the `tirage` binary and puts it in `~/.cargo/bin`. That directory has to be on your `PATH`.

```sh
tirage --version
```

```
tirage 0.0.0 (derivation major 0)
```

## The library

Depend on `tirage` by git, pinned to a commit:

```toml
[dependencies]
tirage = { git = "https://github.com/espadat-studio/tirage", rev = "e43e5b8" }
```

Commit the `Cargo.lock` cargo writes and build with `cargo build --locked`. `Cargo.lock` holds the full commit the pin resolved to and the version of every dependency. With `--locked`, cargo fails the build instead of resolving them again.

The same repository holds `tirage-encode`, the crate that turns a Loop into an MP4. It builds openh264 from source, so it needs a C++ compiler. [Quick Start](/getting-started/quick-start/) uses both crates.
