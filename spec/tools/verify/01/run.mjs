// 批量编译小例子：node run.mjs a.s3d b.s3d …  打印错误、警告、节点路径与位置、检查、元数据
import { readFileSync } from 'node:fs';
import { compile } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
for (const f of process.argv.slice(2)) {
  const r = compile(readFileSync(f, 'utf8'), { file: f, resolve: (p) => readFileSync(p, 'utf8') });
  console.log(`=== ${f}`);
  for (const e of r.errors) console.log(`  ERR L${e.line}: ${e.message}`);
  for (const w of r.warnings) console.log(`  WARN L${w.line}: ${w.message}`);
  for (const n of r.nodes) console.log(`  node ${n.path} kind=${n.kind} t=${n.world ? JSON.stringify(n.world.t.map(x=>+x.toFixed(4))) : null} box=${n.box ? JSON.stringify([n.box.min, n.box.max].map(v=>v.map(x=>+x.toFixed(3)))) : null} tags=${JSON.stringify(n.tags)} meta=${JSON.stringify(n.meta)}`);
  for (const c of r.checks) console.log(`  check "${c.message}" ok=${c.ok} violators=${JSON.stringify(c.violators)} ${c.error ?? ''}`);
  for (const c of r.concepts) console.log(`  concept ${c.name} = ${JSON.stringify(c.members)} ${c.error ?? ''}`);
  for (const p of r.patterns) console.log(`  pattern ${p.name} matches=${JSON.stringify(p.matches)} ${p.error ?? ''}`);
  console.log(`  materials=${JSON.stringify(r.materials)} items=${r.items.map(i=>i.path+':'+JSON.stringify(i.material)).join(' ')}`);
}
