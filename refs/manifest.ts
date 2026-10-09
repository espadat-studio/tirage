export type ParamValue = number | boolean | string;

export interface Recipe {
  tirage: number;
  tool: string;
  tool_seed: number;
  palette: string[];
  params: Record<string, ParamValue>;
}

export interface Fixture {
  name: string;
  frame?: number;
  recipe: Recipe;
}

export interface Threshold {
  max: number;
  reason: string;
}

export interface ToolEntry {
  page_sha256?: string;
  chromium?: string;
  font_sha256?: string;
  threshold?: Threshold;
  fixtures: Fixture[];
}

export interface Manifest {
  tools: Record<string, ToolEntry>;
}

export const EXPORT_CONTROLS = ["seed", "scrub", "motionTog", "exportTog", "mycTog", "sizePx"];

const SLUG = /^[a-z]+$/;
const NAME = /^[a-z0-9]+(-[a-z0-9]+)*$/;
const HEX = /^#[0-9a-f]{6}$/i;
const SHA256 = /^[0-9a-f]{64}$/;
const CEILING = 0.016;

type Json = Record<string, unknown>;

export function fail(path: string, message: string): never {
  throw new Error(`${path}: ${message}`);
}

export function expectObject(value: unknown, path: string, allowed: string[]): Json {
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail(path, "expected an object");
  for (const key of Object.keys(value)) if (!allowed.includes(key)) fail(path, `unknown key ${key}`);
  return value as Json;
}

function expectInteger(value: unknown, path: string, min: number, max: number, message: string): number {
  if (!Number.isInteger(value) || (value as number) < min || (value as number) > max) fail(path, message);
  return value as number;
}

function expectText(value: unknown, path: string, pattern: RegExp, message: string): string {
  if (typeof value !== "string" || !pattern.test(value)) fail(path, message);
  return value;
}

function parseRecipe(value: unknown, path: string, slug: string): Recipe {
  const r = expectObject(value, path, ["tirage", "tool", "tool_seed", "palette", "params"]);
  if (r.tool !== slug) fail(`${path}.tool`, `expected ${slug}, got ${String(r.tool)}`);
  if (!Array.isArray(r.palette) || !r.palette.length) fail(`${path}.palette`, "expected a non-empty array");
  const params = expectObject(r.params, `${path}.params`, Object.keys(r.params ?? {}));
  for (const [id, v] of Object.entries(params)) {
    if (EXPORT_CONTROLS.includes(id)) fail(`${path}.params.${id}`, "set by the export, not the Recipe");
    if (!["number", "boolean", "string"].includes(typeof v)) {
      fail(`${path}.params.${id}`, "expected a number, boolean or string");
    }
  }
  return {
    tirage: expectInteger(r.tirage, `${path}.tirage`, 0, Number.MAX_SAFE_INTEGER, "expected an integer >= 0"),
    tool: slug,
    tool_seed: expectInteger(r.tool_seed, `${path}.tool_seed`, 1, 0xffffffff, "expected an integer in 1..4294967295"),
    palette: r.palette.map((c, i) => expectText(c, `${path}.palette[${i}]`, HEX, "expected #rrggbb")),
    params: params as Record<string, ParamValue>,
  };
}

function parseFixture(value: unknown, path: string, slug: string): Fixture {
  const f = expectObject(value, path, ["name", "frame", "recipe"]);
  const fixture: Fixture = {
    name: expectText(f.name, `${path}.name`, NAME, "expected kebab-case"),
    recipe: parseRecipe(f.recipe, `${path}.recipe`, slug),
  };
  if (f.frame !== undefined) {
    fixture.frame = expectInteger(f.frame, `${path}.frame`, 0, Number.MAX_SAFE_INTEGER, "expected an integer >= 0");
  }
  return fixture;
}

function parseThreshold(value: unknown, path: string): Threshold {
  const t = expectObject(value, path, ["max", "reason"]);
  if (typeof t.max !== "number" || t.max <= 0 || t.max >= CEILING) {
    fail(`${path}.max`, `expected a share in (0, ${CEILING}), an override may only tighten the ceiling`);
  }
  if (typeof t.reason !== "string" || !t.reason.trim()) fail(`${path}.reason`, "expected a non-empty string");
  return { max: t.max, reason: t.reason };
}

function parseTool(value: unknown, path: string, slug: string): ToolEntry {
  const t = expectObject(value, path, ["page_sha256", "chromium", "font_sha256", "threshold", "fixtures"]);
  if (!Array.isArray(t.fixtures) || !t.fixtures.length) fail(`${path}.fixtures`, "expected at least one fixture");
  const names = new Set<string>();
  const fixtures = t.fixtures.map((f, i) => {
    const fixture = parseFixture(f, `${path}.fixtures[${i}]`, slug);
    if (names.has(fixture.name)) fail(`${path}.fixtures[${i}].name`, `duplicate ${fixture.name}`);
    names.add(fixture.name);
    return fixture;
  });
  const entry: ToolEntry = { fixtures };
  if (t.page_sha256 !== undefined) {
    entry.page_sha256 = expectText(t.page_sha256, `${path}.page_sha256`, SHA256, "expected a SHA-256 hex digest");
  }
  if (t.chromium !== undefined) {
    entry.chromium = expectText(t.chromium, `${path}.chromium`, /^\d+(\.\d+)+$/, "expected a Chromium version");
  }
  if (t.font_sha256 !== undefined) {
    entry.font_sha256 = expectText(t.font_sha256, `${path}.font_sha256`, SHA256, "expected a SHA-256 hex digest");
  }
  if (t.threshold !== undefined) entry.threshold = parseThreshold(t.threshold, `${path}.threshold`);
  return entry;
}

export function parseManifest(value: unknown): Manifest {
  const m = expectObject(value, "manifest", ["tools"]);
  const tools = expectObject(m.tools, "tools", Object.keys(m.tools ?? {}));
  const parsed: Manifest = { tools: {} };
  for (const [slug, entry] of Object.entries(tools)) {
    if (!SLUG.test(slug)) fail(`tools.${slug}`, "expected a site slug");
    parsed.tools[slug] = parseTool(entry, `tools.${slug}`, slug);
  }
  return parsed;
}

export function checkPage(slug: string, recorded: string | undefined, live: string, update: boolean): void {
  if (recorded === undefined || recorded === live || update) return;
  throw new Error(
    `${slug} page changed: manifest ${recorded}, live ${live}. Review the site change, then rerun with --update`,
  );
}
