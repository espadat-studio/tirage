# Public repo with committed Reference exports

tirage stays public, and its Reference exports are committed to it. playgrnd.tools publishes no terms, licence or contact. Its only rights line is "Exports are yours." Reference exports are outputs of our own Recipes, which is the case that line covers. A `README` next to the fixtures records each file's source page and page hash and links that line. It states that the files are test fixtures outside any licence tirage grants later, and that they will be removed on request. Public text names playgrnd.tools nominatively, once, as "reproduces the look of playgrnd.tools generators; not affiliated". Tool slugs stay as Tool names. "playgrnd" is never a crate, binary, feature or module name, and we show no logos or screenshots of their UI. One courtesy note goes to the author through the registrar's forwarding address, and nothing waits on a reply.

## Considered Options

- **Private repo**: removes every unstated-terms question, but recordreel's CI then needs a deploy key, and the project gives up being public.
- **Keep exports out of git and regenerate them with `refs` before tests**: the repo holds nothing of theirs, but CI needs network and pinned Chromium, so it is no longer offline.
- **Exports in a private sibling repo or bucket**: CI needs a token, which brings back the private-repo cost without its benefit.
- **Scrub the playgrnd and Tool names**: avoids any naming question, but breaks the Tool-named-by-site-slug rule and the JSON Recipe keyed by site control ids.
- **Gate going public on the author's reply**: there may never be a reply.

## Consequences

- Committed exports stay in git history and forks. A takedown only stops future copies.
- If the author objects, exports move out of git, and the harness falls back to regenerating them with `refs`.
- The licence chosen later must exclude the fixtures directory.
