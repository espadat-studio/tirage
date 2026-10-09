# tirage

Original generative artwork and animations, rendered from code. It reproduces the look of playgrnd.tools generators; not affiliated.

See [`CONTEXT.md`](./CONTEXT.md). [`encode/`](./encode) turns a Loop into an MP4 (ADR 0004).

## CLI

```sh
cargo install --git https://github.com/espadat-studio/tirage --features cli
tirage derive --seed 42 --tool sonar | tirage render --size 1080x1920 -o out.png
```

`tirage --help` lists the rest.

## Develop

Requires [mise](https://mise.jdx.dev).

```sh
mise install
mise run check
```

`mise tasks` lists the rest.
