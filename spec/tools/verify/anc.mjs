import fs from 'node:fs';
import * as P from './s3d_patched.js';
const EX = '/home/user/SVG3D/corpus/examples/';
for (const name of ['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit']) {
  const file = EX + name + '.s3d';
  P.compile(fs.readFileSync(file, 'utf8'), { file, resolve: (p) => fs.readFileSync(p, 'utf8') });
  const S = P.LAST_S; let hits = 0, refs = 0;
  for (const n of S.flat) {
    if (!n.parent || n.ghostOf || n.inst || n.copyOf) continue;
    for (const k of ['on','under','against','inside','at','alignto','aim','from','to','drop','about','across','along','path']) {
      if (!n.props.has(k)) continue;
      let v; try { v = S.eval(n, k); } catch { continue; }
      const items = v && v.list ? v.list : [v];
      for (const it of items) { const nodes = it && it.ref ? [it.node] : it && it.nodes ? it.nodes : []; for (const b of nodes) { refs++; if (b === n || b.isAncestorOf(n)) { hits++; console.log(name, n.path, k, '->', b.path); } } }
    }
  }
  console.log(name, 'refs', refs, 'ancestor/self hits', hits);
}
