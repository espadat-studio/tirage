const path = process.argv[2];
const { Renderer } = require(path);
const r = new Renderer();
for (let f = 0; f < 10; f++) r.frame(f);
const n = 120;
const paths = [];
const grain = [];
for (let f = 0; f < n; f++) {
  const t0 = performance.now();
  r.paths(f);
  const t1 = performance.now();
  r.grain(f);
  paths.push(t1 - t0);
  grain.push(performance.now() - t1);
}
const med = (v) => v.sort((a, b) => a - b)[v.length >> 1];
const p = med(paths);
const g = med(grain);
console.log(`paths_ms=${p.toFixed(2)} grain_ms=${g.toFixed(2)} total_ms=${(p + g).toFixed(2)} checksum=${r.frame(0)}`);
