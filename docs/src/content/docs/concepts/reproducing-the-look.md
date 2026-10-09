---
title: "Reproducing the look"
description: "What tirage shares with the site it reproduces, what it does not, and how the match is checked."
---

tirage reproduces the look of playgrnd.tools generators; not affiliated. Each Tool is written in Rust from studying what its page on the site does. tirage copies no code from the site and shows none of its logos or screenshots.

## What lines up

The names match on purpose, so a Recipe can be replayed on the site:

- A Tool carries the slug of its site page.
- A Parameter has the id, range and step of the control it mirrors, and a Recipe's `params` keys are those ids.
- The Tool seed is the integer the page's seed box takes.
- An unpinned Palette is the Tool's default colours on the site.

To replay a Recipe there, enter the Tool seed first and the Parameters after it. Some pages, vein among them, reset their sliders when the seed changes, so the other order loses your values.

## Reference exports

A _Reference export_ is a file exported from the live site page for one of our own Recipes. The tests render that Recipe with tirage, compare the result to the export pixel by pixel, and fail the Tool when more than 1.6% of pixels differ in brightness by more than 48 of 255. Loop Tools have exports for frames past 0 too, so the motion is checked as well as the Still.

The exports live in the repository as test fixtures, with a record of the page each one came from. The site's one line on rights says "Exports are yours.", and these files are exports of our own Recipes. They are not covered by any licence tirage grants, and they will be removed if the site's author asks.

A Tool can keep moving closer to its exports within a derivation major, which is why [pixels are not promised](/concepts/seed-and-recipe/#what-stays-fixed).
