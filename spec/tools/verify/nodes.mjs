import fs from 'node:fs';
const [,, file, ...pats] = process.argv;
const j = JSON.parse(fs.readFileSync(file, 'utf8'));
const re = pats.length ? new RegExp(pats.join('|')) : null;
const f = (x) => x == null ? '-' : Array.isArray(x) ? '[' + x.map((v) => typeof v === 'number' ? +v.toFixed(4) : v).join(',') + ']' : typeof x === 'object' ? JSON.stringify(x) : String(x);
for (const n of j.nodes) {
  if (re && !re.test(n.path)) continue;
  const b = n.box ? `box=[${n.box.min.map((v)=>+v.toFixed(3))}]..[${n.box.max.map((v)=>+v.toFixed(3))}]` : 'box=-';
  console.log(`${n.path}\tkind=${n.kind}\ttypes=${f(n.types)}\tghost=${n.ghost}\tof=${n.of}\thidden=${n.hidden}\tskin=${n.skin}\ttags=${f(n.tags)}\tmeta=${f(n.meta)}\tline=${n.line}\tfile=${n.file}\t${b}`);
}
console.log('--- counts: nodes', j.nodes.length, 'items', j.items.length, 'facts', j.facts.length, 'defs', JSON.stringify(j.defs), 'errors', JSON.stringify(j.errors), 'warnings', JSON.stringify(j.warnings));
