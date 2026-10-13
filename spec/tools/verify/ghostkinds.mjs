import fs from 'node:fs';
const dir = '/home/user/SVG3D/corpus/reference/examples/';
for (const f of fs.readdirSync(dir).filter(x => x.endsWith('.compile.json'))) {
  const j = JSON.parse(fs.readFileSync(dir + f, 'utf8'));
  const byPath = new Map(j.nodes.map(n => [n.path, n]));
  const pairs = new Map();
  for (const n of j.nodes) {
    if (!n.of) continue;
    const o = byPath.get(n.of);
    const key = `${n.ghost ? 'ghost' : 'R'} kind=${n.kind} types=${JSON.stringify(n.types)} <- of.kind=${o?.kind} of.types=${JSON.stringify(o?.types)}`;
    pairs.set(key, (pairs.get(key) || 0) + 1);
  }
  console.log('##', f, 'defs=', JSON.stringify(j.defs));
  for (const [k, v] of pairs) console.log('  ', v, k);
}
