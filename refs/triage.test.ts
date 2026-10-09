import { describe, expect, test } from "bun:test";

import { draw, DRAWS, parseBounds, sweep } from "./triage";
import type { Slider } from "./triage";

const level: Slider = { id: "level", min: 0, max: 1, step: 0.01, value: 0.5 };
const masses: Slider = { id: "masses", min: 0, max: 6, step: 1, value: 3 };
const scale: Slider = { id: "scale", min: 0.5, max: 2, step: 0.05, value: 2 };
const split: Slider = { id: "split", min: 0.15, max: 0.95, step: 0.01, value: 0.62 };
const sliders = [level, masses, scale, split];

describe("sweep", () => {
  test.each([
    [level, [0, 0.25, 0.5, 0.75, 1]],
    [masses, [0, 2, 3, 5, 6]],
    [scale, [0.5, 0.9, 1.25, 1.65, 2]],
    [split, [0.15, 0.35, 0.55, 0.75, 0.95]],
  ])("spreads 5 slider values from min to max for %j", (slider, points) => {
    expect(sweep(slider)).toEqual(points);
  });
});

describe("draw", () => {
  const onStep = (slider: Slider, v: number) =>
    Math.abs((v - slider.min) / slider.step - Math.round((v - slider.min) / slider.step)) < 1e-9;

  test("gives a tool seed and a slider value per slider, inside the full range", () => {
    for (let i = 0; i < DRAWS; i++) {
      const { toolSeed, values } = draw(sliders, `aura/draw/${i}`);
      expect(Number.isInteger(toolSeed) && toolSeed >= 1 && toolSeed <= 99999).toBe(true);
      expect(Object.keys(values)).toEqual(sliders.map(s => s.id));
      for (const s of sliders) {
        expect(values[s.id]).toBeGreaterThanOrEqual(s.min);
        expect(values[s.id]).toBeLessThanOrEqual(s.max);
        expect(onStep(s, values[s.id])).toBe(true);
      }
    }
  });

  test("keeps every draw inside candidate bounds and the rest at full range", () => {
    for (let i = 0; i < DRAWS; i++) {
      const { values } = draw(sliders, `aura/check/${i}`, { level: [0.9, 1], masses: [2, 2] });
      expect(values.level).toBeGreaterThanOrEqual(0.9);
      expect(values.masses).toBe(2);
      expect(values.scale).toBeGreaterThanOrEqual(0.5);
    }
  });

  test("is a pure function of its key", () => {
    expect(draw(sliders, "x")).toEqual(draw(sliders, "x"));
    expect(draw(sliders, "x")).not.toEqual(draw(sliders, "y"));
  });
});

describe("parseBounds", () => {
  test("accepts [min, max] per slider id and an optional matching tool", () => {
    expect(parseBounds({ tool: "aura", level: [0.25, 0.7], masses: [0, 6] }, "aura", sliders)).toEqual({
      level: [0.25, 0.7],
      masses: [0, 6],
    });
    expect(parseBounds({}, "aura", sliders)).toEqual({});
  });

  test.each([
    [[], "bounds: expected an object"],
    [{ tool: "husk" }, "bounds.tool: expected aura, got husk"],
    [{ lvl: [0, 1] }, "bounds: unknown key lvl"],
    [{ level: 0.5 }, "bounds.level: expected [min, max]"],
    [{ level: [0, "1"] }, "bounds.level: expected [min, max]"],
    [{ level: [0.2, 1.4] }, "bounds.level: 1.4 is outside 0..1"],
    [{ level: [0.255, 0.5] }, "bounds.level: 0.255 is not a slider value, step 0.01"],
    [{ level: [0.7, 0.25] }, "bounds.level: 0.7..0.25 has min above max"],
  ])("rejects %j", (value, message) => {
    expect(() => parseBounds(value, "aura", sliders)).toThrow(message);
  });
});
