import { describe, expect, test } from "bun:test";

import { checkPage, parseManifest } from "./manifest";

const recipe = (over: Record<string, unknown> = {}) => ({
  tirage: 0,
  tool: "sonar",
  tool_seed: 7,
  palette: ["#000000", "#ffffff"],
  params: { level: 0.5, grainTog: false, modes: "Tide" },
  ...over,
});

const manifest = (fixture: Record<string, unknown> = {}, tool: Record<string, unknown> = {}) => ({
  tools: { sonar: { fixtures: [{ name: "seed-7", recipe: recipe(), ...fixture }], ...tool } },
});

describe("parseManifest", () => {
  test("accepts a Still and a Loop fixture", () => {
    const parsed = parseManifest({
      tools: {
        sonar: {
          page_sha256: "a".repeat(64),
          chromium: "153.0.8010.12",
          font_sha256: "b".repeat(64),
          threshold: { max: 0.02, reason: "grain specks land on different pixels" },
          fixtures: [{ name: "seed-7", recipe: recipe() }, { name: "seed-7-mid", frame: 12, recipe: recipe() }],
        },
      },
    });
    expect(parsed.tools.sonar.fixtures.map(f => f.frame)).toEqual([undefined, 12]);
  });

  test.each([
    [{ recipe: recipe({ tool: "aura" }) }, {}, "tools.sonar.fixtures[0].recipe.tool: expected sonar, got aura"],
    [
      { recipe: recipe({ tool_seed: 0 }) },
      {},
      "tools.sonar.fixtures[0].recipe.tool_seed: expected an integer in 1..4294967295",
    ],
    [{ recipe: recipe({ palette: ["#fff"] }) }, {}, "tools.sonar.fixtures[0].recipe.palette[0]: expected #rrggbb"],
    [
      { recipe: recipe({ params: { scrub: 3 } }) },
      {},
      "tools.sonar.fixtures[0].recipe.params.scrub: set by the export, not the Recipe",
    ],
    [
      { recipe: recipe({ params: { level: null } }) },
      {},
      "tools.sonar.fixtures[0].recipe.params.level: expected a number, boolean or string",
    ],
    [{ frame: -1 }, {}, "tools.sonar.fixtures[0].frame: expected an integer >= 0"],
    [{ name: "Seed 7" }, {}, "tools.sonar.fixtures[0].name: expected kebab-case"],
    [{ colour: 1 }, {}, "tools.sonar.fixtures[0]: unknown key colour"],
    [{}, { threshold: { max: 0.02 } }, "tools.sonar.threshold.reason: expected a non-empty string"],
    [{}, { threshold: { max: 1.5, reason: "x" } }, "tools.sonar.threshold.max: expected a share in (0, 1)"],
  ])("rejects %j %j", (fixture, tool, message) => {
    expect(() => parseManifest(manifest(fixture, tool))).toThrow(message);
  });

  test("rejects duplicate fixture names", () => {
    const fixtures = [{ name: "seed-7", recipe: recipe() }, { name: "seed-7", recipe: recipe() }];
    expect(() => parseManifest({ tools: { sonar: { fixtures } } })).toThrow(
      "tools.sonar.fixtures[1].name: duplicate seed-7",
    );
  });

  test("rejects a Tool with no fixtures", () => {
    expect(() => parseManifest({ tools: { sonar: { fixtures: [] } } })).toThrow(
      "tools.sonar.fixtures: expected at least one fixture",
    );
  });
});

describe("checkPage", () => {
  const old = "a".repeat(64);
  const live = "c".repeat(64);

  test("passes when the hash is new or unchanged", () => {
    expect(() => checkPage("sonar", undefined, live, false)).not.toThrow();
    expect(() => checkPage("sonar", live, live, false)).not.toThrow();
  });

  test("fails on a changed page", () => {
    expect(() => checkPage("sonar", old, live, false)).toThrow(
      `sonar page changed: manifest ${old}, live ${live}. Review the site change, then rerun with --update`,
    );
  });

  test("passes on a changed page with --update", () => {
    expect(() => checkPage("sonar", old, live, true)).not.toThrow();
  });
});
