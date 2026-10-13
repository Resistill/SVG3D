// 从 crosswalk.md 的 8 张表里抽出第 4 列（与第 5 列）的章节号，核对存在性并逐章计数
import fs from 'node:fs';
const md = fs.readFileSync('/home/user/SVG3D/spec/crosswalk.md', 'utf8');
const secs = new Set(fs.readFileSync('/tmp/claude-0/-home-user-SVG3D/460dd295-d45e-5ee1-9860-e88303001ef6/scratchpad/crosswalk/secnums.txt', 'utf8').split('\n').filter(Boolean));
// 还要加上二级标题（如 2.6、2.7、4.7、1.8）
for (const f of fs.readdirSync('/home/user/SVG3D/spec')) if (/^(0\d|10)-/.test(f)) for (const m of fs.readFileSync('/home/user/SVG3D/spec/' + f, 'utf8').matchAll(/^## (\d+\.\d+) /gm)) secs.add(m[1]);
const lines = md.split('\n');
let section = '';
const counts = {}, missing = [], col5cites = {};
let rows = 0;
const splitRow = l => { // 表格行按 | 分列，忽略 \| 转义
  const cells = []; let cur = ''; for (let i = 1; i < l.length; i++) { const ch = l[i]; if (ch === '\\' && l[i + 1] === '|') { cur += '|'; i++; continue; } if (ch === '|') { cells.push(cur.trim()); cur = ''; continue; } cur += ch; } return cells; };
for (const l of lines) {
  if (/^## /.test(l)) section = l;
  if (!/^## [1-8] /.test(section)) continue;
  if (!l.startsWith('| ')) continue;
  const cells = splitRow(l);
  if (cells.length < 5 || cells[0] === '行' || cells[0].startsWith('---') || cells[0].startsWith('注释')) continue;
  rows++;
  const col4 = cells[3], col5 = cells[4];
  const cites = new Set([...col4.matchAll(/\b(\d{1,2}\.\d{1,2}(?:\.\d{1,2})?)\b/g)].map(m => m[1]));
  for (const c of cites) { const ch = c.split('.')[0]; counts[ch] = counts[ch] || {}; counts[ch][c] = (counts[ch][c] || 0) + 1; if (!secs.has(c)) missing.push([section.slice(3, 20), cells[0], c]); }
  for (const m of col5.matchAll(/\b(\d{1,2}\.\d{1,2}\.\d{1,2})\b/g)) { col5cites[m[1]] = (col5cites[m[1]] || 0) + 1; if (!secs.has(m[1])) missing.push([section.slice(3, 20), cells[0], m[1], 'col5']); }
}
console.log('rows', rows);
for (const ch of Object.keys(counts).sort((a, b) => a - b)) { const tot = Object.values(counts[ch]).reduce((a, b) => a + b, 0); const top = Object.entries(counts[ch]).sort((a, b) => b[1] - a[1]).slice(0, 7); console.log('ch', ch, 'total', tot, top.map(x => x.join(':')).join(' ')); }
console.log('missing', missing);
console.log('col5 cites', col5cites);
// 第 10 / 11 节与读法里引用的小节号也核对存在性
const other = new Set([...md.matchAll(/\b(\d{1,2}\.\d{1,2}\.\d{1,2})\b/g)].map(m => m[1]));
console.log('nonexistent anywhere', [...other].filter(c => !secs.has(c)));
