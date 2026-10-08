# visuals

Original generative artwork and animations that reproduce the look of the playgrnd.tools generators, rendered from code.

## Language

**Tool**:
One playgrnd.tools generator we reproduce, named by its site slug (aura, riso, mist).
_Avoid_: Effect, filter, generator

**Parameter**:
One named, bounded input of a Tool, matching a slider or toggle on its site page.
_Avoid_: Option, setting, knob

**Palette**:
The ordered colours a Tool paints with.
_Avoid_: Theme, scheme

**Recipe**:
A Tool plus a value for each of its Parameters, a Palette and a Tool seed. It fully determines one Visual.
_Avoid_: Preset, config

**Seed**:
One integer from which a whole Recipe is derived. The same Seed always gives the same Recipe.
_Avoid_: ID, hash

**Visual**:
What a Recipe renders: a Still or a Loop.

**Still**:
A single-frame Visual (PNG or SVG).

**Loop**:
An animated Visual that ends where it starts.
_Avoid_: GIF, clip

**Reference export**:
A file exported from the live playgrnd.tools page for a Recipe, used to check our Visual against theirs.
_Avoid_: Golden, snapshot
