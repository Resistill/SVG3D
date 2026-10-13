import fs from 'fs';
const name = process.argv[2];
const j = JSON.parse(fs.readFileSync(`corpus/reference/examples/${name}.compile.json`,'utf8'));
console.log('KEYS', Object.keys(j).join(' '));
console.log('counts nodes', j.nodes.length, 'items', (j.items||[]).length, 'curves', (j.curves||[]).length, 'facts', (j.facts||[]).length, 'errors', JSON.stringify(j.errors), 'warnings', JSON.stringify(j.warnings));
console.log('materials', Object.keys(j.materials||{}).join(' '), '| defs', JSON.stringify(j.defs), '| factProviders', JSON.stringify(j.factProviders), 'factsSkipped', j.factsSkipped);
console.log('meta', JSON.stringify(j.meta));
// group nodes by file,line
const by = new Map();
for (const n of j.nodes) { const k = `${n.file}:${n.line}`; if(!by.has(k)) by.set(k, []); by.get(k).push(n); }
const keys = [...by.keys()].sort((a,b)=>{const [fa,la]=a.split(':');const [fb,lb]=b.split(':'); return fa===fb? (+la)-(+lb) : fa<fb?-1:1;});
for (const k of keys) {
  const ns = by.get(k);
  const show = ns.slice(0,4).map(n=>`${n.path}[${n.kind}${n.types?'/'+n.types.join(',') :''}${n.ghost?' ghost':''}${n.hidden?' hidden':''}${n.skin?' skin':''}${n.late?' late':''}]`).join(' ');
  console.log(`${k}  n=${ns.length}  ${show}${ns.length>4?' …':''}`);
}
console.log('--- semantic');
for (const c of j.concepts||[]) console.log('concept', c.name, 'line', c.line, 'parent', c.parent, 'members', c.members.length, JSON.stringify(c.members.slice(0,6)), c.error?('ERR '+c.error):'');
for (const p of j.patterns||[]) console.log('pattern', p.name, 'line', p.line, 'roles', JSON.stringify(p.roles), 'matches', p.matches.length, JSON.stringify(p.matches[0]), p.error||'');
for (const c of j.checks||[]) console.log('check', c.line, JSON.stringify(c.message), 'ok', c.ok, 'violators', JSON.stringify(c.violators), c.error||'');
for (const v of j.variants||[]) console.log('variant', v.line, v.use, v.src, JSON.stringify(v.members));
