// 审查用：编译一个 .s3d，打印错误与每个节点的 late / R / t / box
import { readFileSync } from 'node:fs';
import { compile } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const file = process.argv[2];
const r = compile(readFileSync(file, 'utf8'), { file });
const f = (v) => v.map((x) => (Math.abs(x) < 5e-5 ? 0 : x).toFixed(4)).join(',');
for (const e of r.errors) console.log(`ERR line ${e.line}: ${e.message}`);
for (const n of r.nodes) {
  const w = n.world;
  console.log(`${n.path.padEnd(22)} ${n.kind.padEnd(8)} late=${n.late ? 1 : 0} ${w ? 'R=[' + w.R.map(f).join(' | ') + '] t=(' + f(w.t) + ')' : 'world=null'}${n.box ? ' box=[' + f(n.box.min) + '][' + f(n.box.max) + ']' : ''}`);
}
