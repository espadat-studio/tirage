# playgrnd.tools terms for a public reimplementation

Ticket: [#26](https://github.com/espadat-studio/tirage/issues/26). Checked 2026-10-08.

This is a report of what the site says. It is not legal advice.

## What was checked

| Source | URL | Result |
|---|---|---|
| Homepage HTML, text, meta tags, comments | https://www.playgrnd.tools/ | One rights line: "Exports are yours." No author, no licence, no copyright notice. |
| All 52 Tool pages (`/<slug>/`) | e.g. https://www.playgrnd.tools/vein/ | Grepped full source for licence, copyright, ©, author, terms, MIT, CC, attribution, commercial, permission, github, mailto. No hits. |
| `robots.txt` | https://www.playgrnd.tools/robots.txt | Cloudflare Content Signals preamble only. No `User-agent`, no `Content-Signal` lines. |
| `/terms`, `/privacy`, `/about`, `/faq`, `/license`, `/legal`, `/LICENSE`, `/humans.txt`, `/.well-known/security.txt`, `/sitemap.xml` | under https://www.playgrnd.tools/ | All 404. |
| WHOIS `playgrnd.tools` | registry | Registrant not public. |
| Web search for site or author | | No author, repo or contact found. |

## Primary text

Homepage, left rail note (https://www.playgrnd.tools/):

> Every tool runs in the browser. Nothing is uploaded and nothing is stored. Exports are yours.

Homepage meta description:

> A collection of free, single-purpose generative design tools. No accounts, no uploads, everything runs in your browser.

`robots.txt`. The origin returns 404 for this file (`x-vercel-error: NOT_FOUND`). Cloudflare serves its managed Content Signals preamble in its place:

> As a condition of accessing this website, you agree to abide by the following content signals:
> (a) If a content-signal = yes, you may collect content for the corresponding use.
> (b) If a content-signal = no, you may not collect content for the corresponding use.
> (c) If the website operator does not include a content signal for a corresponding use, the website operator neither grants nor restricts permission via content signal with respect to the corresponding use.

The file sets no signal. Its uses are `search`, `ai-input` and `ai-train` only. None of them covers code reuse or image fixtures.

Page source comments. These describe the code. None of them states rights. Example from `/vein/`:

> Instrument chassis runtime: hairline slider fills, capture dots, the corner readouts and the Animate / Export disclosures. The tool's own code is untouched.

## Per question

| Question | Explicitly allowed | Explicitly forbidden | Unstated |
|---|---|---|---|
| 1. Public repo with our clean-room Rust reimplementation of the look | Nothing | Nothing | No licence on the site's JS. No terms on reading it, studying it or reimplementing it. |
| 2. Commit Reference exports to the public repo as fixtures | "Exports are yours." | Nothing | No statement on redistribution, or on publishing exports to test against their tools. "Yours" is not defined as a licence. |
| 3. Publish the crate on crates.io under an OSS licence | Nothing | Nothing | No statement on derivative software, the "playgrnd" name or the Tool names. |

## Reading of the text

- The site publishes no licence. Without a licence, default copyright applies to their JS and page code. So we must not copy their code into the repo. Reading it to learn algorithms is not addressed by the site.
- "Exports are yours" is the only rights grant. It covers what a user generates. It is the strongest support for committing Reference exports.
- Tool names (Vein, Atlas and others) and the "playgrnd" name have no trademark notice. The site also gives no permission to use them.
- No author or contact is published anywhere checked. There is no address to ask for permission.

## Needs a lawyer or the author

- Whether "Exports are yours" lets us redistribute exports in a public OSS repo, and relicense them as part of it.
- Whether a reimplementation that matches their output pixel for pixel counts as a derivative work of their code.
- Whether the crate and its docs can use the Tool names and "playgrnd".
- Author: not identified. Contact: none published. The registrar's layered WHOIS access is the only listed route to the registrant.
