import fs from 'fs';
const txt = fs.readFileSync('spec/crosswalk.md','utf8').split('\n');
let section = 0; const chap = {}; const sub = {}; let rows = 0; let noteRows = 0;
for (const line of txt) {
  const m = line.match(/^## (\d+) /); if (m) { section = +m[1]; continue; }
  if (section < 1 || section > 8) continue;
  if (!line.startsWith('|')) continue;
  const cells = line.replace(/\\\|/g,'\u0001').split('|').map(s=>s.trim());
  // cells[0] is '' ; cells[1] 行, cells[2] 语句, cells[3] 构件, cells[4] 条款, cells[5] 可核对项
  if (!/^\d/.test(cells[1])) { if (cells[1].startsWith('注释')) noteRows++; continue; }
  rows++;
  const set = new Set();
  for (const mm of cells[4].matchAll(/(?<![\d.])(10|[1-8])\.(\d+)(?:\.(\d+))?(?![\d])/g)) {
    const ch = mm[1]; const key = mm[3] ? `${ch}.${mm[2]}.${mm[3]}` : `${ch}.${mm[2]}`;
    set.add(key);
  }
  for (const k of set) { const ch = k.split('.')[0]; chap[ch]=(chap[ch]||0)+1; sub[k]=(sub[k]||0)+1; }
}
console.log('rows', rows, 'noteRows', noteRows);
for (const ch of ['1','2','3','4','5','6','7','8','10']) {
  const top = Object.entries(sub).filter(([k])=>k.split('.')[0]===ch).sort((a,b)=>b[1]-a[1]).slice(0,6).map(([k,v])=>`${k}(${v})`).join(' ');
  console.log('chap', ch, chap[ch]||0, '|', top);
}
