import { execSync } from 'node:child_process';
const out = execSync('node verify3.mjs', { encoding: 'utf8' });
const tot = {}; let cur;
for (const line of out.split('\n')) {
  const m = /^\s+(\S+)\s+ok\s+(\d+)\s+bad\s+(\d+)$/.exec(line);
  if (m) { tot[m[1]] ??= { ok: 0, bad: 0, ex: new Set() }; tot[m[1]].ok += +m[2]; tot[m[1]].bad += +m[3]; tot[m[1]].ex.add(cur); }
  const e = /^== (\S+):/.exec(line); if (e) cur = e[1];
}
let sum = 0;
for (const [k, v] of Object.entries(tot).sort()) { sum += v.ok; console.log(`| ${k} | ${v.ok} | ${v.bad} | ${[...v.ex].join(', ')} |`); }
console.log('total', sum);
