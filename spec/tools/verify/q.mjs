// 用法: node q.mjs <名> <路径正则> [字段,字段,...]
import fs from 'node:fs';
const [name, re, fields] = process.argv.slice(2);
const j = JSON.parse(fs.readFileSync(`/home/user/SVG3D/corpus/reference/examples/${name}.compile.json`, 'utf8'));
const rx = new RegExp(re || '.');
const fl = fields ? fields.split(',') : null;
const r = (v) => Array.isArray(v) ? v.map(x => typeof x === 'number' ? +x.toFixed(4) : r(x)) : (typeof v === 'number' ? +v.toFixed(4) : v);
for (const n of j.nodes) {
  if (!rx.test(n.path)) continue;
  if (fl) { const o = {}; for (const f of fl) o[f] = r(n[f]); console.log(n.path, JSON.stringify(o)); }
  else console.log(JSON.stringify({ ...n, world: undefined, box: undefined, tight: undefined }));
}
