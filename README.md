# tirage

Original generative artwork and animations, rendered from code. It reproduces the look of playgrnd.tools generators; not affiliated.

Docs: https://tirage.espadat.com

See [`CONTEXT.md`](./CONTEXT.md). [`encode/`](./encode) turns a Loop into an MP4 (ADR 0004).

Tools: `sonar`, `husk`, `vein`, `aura`, `kiosk`, `frond` ([ADR 0002](./meta/adr/0002-tool-shortlist-for-recordreel.md)).

## Library

Not on crates.io. Depend on a [release](https://github.com/espadat-studio/tirage/releases) tag:

```toml
tirage = { git = "https://github.com/espadat-studio/tirage", tag = "vX.Y.Z" }
```

Your own `Cargo.lock` pins the tag's commit and tirage's dependencies. Build with `--locked` so they never move silently.

A Seed gives the same Recipe within a major version. A Seed→Recipe change bumps the major, or the minor before 1.0 ([ADR 0003](./meta/adr/0003-seed-derivation-of-recipes.md)). Key Seed caches on it.

## CLI

```sh
cargo install --locked --git https://github.com/espadat-studio/tirage --tag vX.Y.Z --features cli
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

Each [GitHub release](https://github.com/espadat-studio/tirage/releases) also ships prebuilt binaries. `tirage --help` lists the rest.

## Develop

Requires [mise](https://mise.jdx.dev).

```sh
mise install
mise run check
```

`mise tasks` lists the rest.

## Licence

AGPL-3.0-or-later ([LICENSE](./LICENSE)). The Reference exports under [`tests/refs/`](./tests/refs) are test fixtures outside it, as their [README](./tests/refs/README.md) states.
