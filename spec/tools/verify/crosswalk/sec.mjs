// 用法：node sec.mjs 3.3.2 [maxLines]  打印该小节正文；node sec.mjs --examples 核对全部“示例原句”声明
import fs from 'node:fs';
const dir = '/home/user/SVG3D/spec/';
const files = fs.readdirSync(dir).filter(f => /^(0\d|10)-/.test(f));
const bodies = {};
for (const f of files) {
  const lines = fs.readFileSync(dir + f, 'utf8').split('\n');
  let cur = null, level = 0;
  lines.forEach((l, i) => {
    const m = l.match(/^(#{2,4}) (\d+(?:\.\d+){1,3})\b/);
    if (m) { cur = m[2]; level = m[1].length; bodies[cur] = { file: f, start: i + 1, lines: [] }; }
    else if (cur) bodies[cur].lines.push(l);
  });
  // 二级 / 三级小节的正文应包含其下级：重新拼接
  const nums = Object.keys(bodies).filter(k => bodies[k].file === f);
  for (const k of nums) for (const sub of nums) if (sub !== k && sub.startsWith(k + '.')) bodies[k].lines.push(...bodies[sub].lines);
}
export const body = n => bodies[n];
if (process.argv[2] === '--examples') {
  const md = fs.readFileSync(dir + 'crosswalk.md', 'utf8');
  for (const l of md.split('\n')) {
    if (!l.startsWith('| ') || l.startsWith('| 行') || l.startsWith('| 注释')) continue;
    const cells = []; let cur = ''; for (let i = 1; i < l.length; i++) { const ch = l[i]; if (ch === '\\' && l[i + 1] === '|') { cur += '|'; i++; continue; } if (ch === '|') { cells.push(cur.trim()); cur = ''; continue; } cur += ch; }
    if (cells.length < 5) continue;
    const stmt = cells[1].replace(/`/g, ''), col4 = cells[3];
    for (const m of col4.matchAll(/(\d+\.\d+(?:\.\d+)?)（([^）]*示例原句[^）]*)）/g)) {
      const sec = m[1], note = m[2]; const b = bodies[sec]; if (!b) { console.log('NO SECTION', sec); continue; }
      const code = b.lines.filter(x => !x.startsWith('#')).join('\n');
      // 取声明里提到的节点名；否则取语句里的所有节点声明（种类 名字）
      let names = [...note.matchAll(/`([^`]+)`/g)].map(x => x[1]);
      const decls = [...stmt.matchAll(/(?:^|[{/]\s*)([a-z_]+)\s+([A-Za-z_][\w]*)(?=\s|$)/g)].map(x => x[1] + ' ' + x[2]);
      const probes = names.length ? names : decls;
      const hits = probes.map(p => [p, code.includes(p)]);
      const ok = hits.every(h => h[1]);
      console.log(ok ? 'ok  ' : 'MISS', cells[0].padEnd(8), sec.padEnd(7), JSON.stringify(hits.filter(h => !h[1]).map(h => h[0])), '|', note.slice(0, 50));
    }
  }
} else if (process.argv[2]) {
  const b = bodies[process.argv[2]]; if (!b) { console.log('no such section'); process.exit(1); }
  console.log(`# ${b.file}:${b.start}`); const max = +(process.argv[3] || 80); console.log(b.lines.slice(0, max).join('\n'));
}
