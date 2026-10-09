# Seed derivation of Recipes

A Seed is a `u64` that deals a whole Recipe: the Tool, then the Palette, then each Parameter and the Tool seed. Each draw is its own keyed hash, `hash(Seed, field)`, and for a Parameter `hash(Seed, Tool, Parameter name)` mapped into that Tool's Taste bounds. Any field can be Pinned, and Pinning a field to the value the Seed would have dealt gives the same Recipe. recordreel Pins the Tool and Palette (its design slot and inks), so for it a Seed only varies the Parameters and Tool seed. Rendering takes a Recipe, not a Seed: `derive` gives a Recipe, `render` draws it, and a caller Pins by editing the Recipe in between.

## Considered Options

- **Seed derives only Parameters and Tool seed**: fits recordreel exactly, but leaves the CLI no way to deal a random Visual from one number.
- **One RNG drawn in declared order**: same cost, but adding or reordering a Parameter reshuffles every later value, so a Seed's Visual changes on unrelated edits.
- **One `u32` shared with the site**: one number end to end, but the whole Recipe hangs on 32 bits, so a pair of identical Visuals becomes likely at about 77k per Tool.
- **Derivation frozen forever**: suggests a print never changes, but pixels change anyway when a port gets closer to its Reference export.

## Consequences

- Seed to Recipe is stable within a major version. A change bumps the major, and recordreel's cache key carries the version. Pixels are not promised.
- An unpinned Tool is dealt by rendezvous hash: the ported Tool with the highest `hash(Seed, "tool", slug)` wins. Adding a Tool moves only the Seeds it now wins, about 1/N of them; every other Seed keeps its Tool. Registry order does not matter.
- Before 1.0, ports land under major 0. At 1.0 the 6-Tool edition freezes, and any Tool added after it bumps the derivation major.
- An unpinned Palette is the Tool's playgrnd default. The library ships no other Palettes.
- The Tool seed is a nonzero `u32`: playgrnd reads it as `(seed>>>0)||1`, so 0 and 1 render alike. A Recipe replays on the site by setting the seed before the Parameters, since deal Tools like `vein` rewrite Parameters from the seed.
- The library ships one set of Taste bounds per Tool. A caller may replace them, for example where `aura`'s two recordreel slots need different ranges.
