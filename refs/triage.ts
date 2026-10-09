import { createHash } from "node:crypto";

import { expectObject, fail } from "./manifest";

export interface Slider {
  id: string;
  min: number;
  max: number;
  step: number;
  value: number;
}

export type Bounds = Record<string, [number, number]>;

const SWEEP_POINTS = 5;
export const DRAWS = 24;
const SITE_SEEDS = 99999;

const decimals = (n: number): number => (String(n).split(".")[1] ?? "").length;

function snap(slider: Slider, value: number): number {
  const ticks = Math.round((value - slider.min) / slider.step);
  const places = Math.max(decimals(slider.step), decimals(slider.min));
  return Number((slider.min + ticks * slider.step).toFixed(places));
}

export function sweep(slider: Slider): number[] {
  return Array.from({ length: SWEEP_POINTS }, (_, i) => {
    return snap(slider, slider.min + ((slider.max - slider.min) * i) / (SWEEP_POINTS - 1));
  });
}

const unit = (key: string): number =>
  parseInt(createHash("sha256").update(key).digest("hex").slice(0, 12), 16) / 2 ** 48;

export function draw(
  sliders: Slider[],
  key: string,
  bounds: Bounds = {},
): { toolSeed: number; values: Record<string, number> } {
  const toolSeed = 1 + Math.floor(unit(`${key}/seed`) * SITE_SEEDS);
  const values = Object.fromEntries(
    sliders.map(s => {
      const [lo, hi] = bounds[s.id] ?? [s.min, s.max];
      return [s.id, snap(s, lo + (hi - lo) * unit(`${key}/${s.id}`))];
    }),
  );
  return { toolSeed, values };
}

export function parseBounds(value: unknown, slug: string, sliders: Slider[]): Bounds {
  const given = expectObject(value, "bounds", ["tool", ...sliders.map(s => s.id)]);
  const bounds: Bounds = {};
  for (const [id, range] of Object.entries(given)) {
    const path = `bounds.${id}`;
    if (id === "tool") {
      if (range !== slug) fail(path, `expected ${slug}, got ${String(range)}`);
      continue;
    }
    const slider = sliders.find(s => s.id === id)!;
    if (!Array.isArray(range) || range.length !== 2 || !range.every(v => typeof v === "number" && Number.isFinite(v))) {
      fail(path, "expected [min, max]");
    }
    const [lo, hi] = range as [number, number];
    for (const v of [lo, hi]) {
      if (v < slider.min || v > slider.max) fail(path, `${v} is outside ${slider.min}..${slider.max}`);
      if (snap(slider, v) !== v) fail(path, `${v} is not a slider value, step ${slider.step}`);
    }
    if (lo > hi) fail(path, `${lo}..${hi} has min above max`);
    bounds[id] = [lo, hi];
  }
  return bounds;
}
