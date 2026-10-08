# Tool shortlist for recordreel

The first shortlist is 6 Tools: `vein`, `aura`, `kiosk`, `husk`, `sonar` and `frond`. Each is the lead playgrnd layer of a shipped recordreel design (Variant Press, Photocard Binder, Wheatpaste Wall, Test Pressing, One-bit Crate, Paste-up Tropical). A Visual can only fill a slot a design already has, so we picked from the 10 Tools recordreel ships today, not from all 52. Picking one Tool per design lets a Seed vary every design for each Collector with the fewest ports. Sealed Release has no playgrnd layer and takes no Visual.

## Considered Options

- **All 10 shipped-design Tools**: also ports `mist`, `riso`, `splice` and `coral`. These are secondary layers (an extra pressing, the old sheet, grain, leaf panels), where a per-Collector change barely shows. That is 4 more ports for little visible gain.
- **A fresh pick from all 52**: needs new slots, so it means design work in recordreel, which is outside this spec.
- **1 to 3 Tools to prove the workflow**: smaller, but leaves most designs without a Visual. Port order is decided separately.

## Consequences

- `mist`, `riso`, `splice` and `coral` stay fixed rasters in recordreel.
- `aura` blends inks into tints, which suits its foil and pressing slots. It must not be used where a design forbids tints, as Sealed Release does.
- 3 of the 6 (`aura`, `husk`, `frond`) export PNG only on playgrnd.tools, so their Reference exports are rasters.
- `prism` is the only Tool that ignores custom inks. It needs special handling if it is ever added.
- Evidence: the contact sheet on branch `prototype/tool-shortlist` (52 Tools × 3 seeds in recordreel inks).
