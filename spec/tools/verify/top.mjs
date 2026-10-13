// 顶层键与非 nodes 部分
import fs from 'node:fs';
const [name] = process.argv.slice(2);
const j = JSON.parse(fs.readFileSync(`/home/user/SVG3D/corpus/reference/examples/${name}.compile.json`, 'utf8'));
for (const k of Object.keys(j)) {
  const v = j[k];
  if (k === 'nodes' || k === 'items' || k === 'facts' || k === 'curves') console.log(k, Array.isArray(v) ? v.length : typeof v);
  else console.log(k, JSON.stringify(v).slice(0, 1500));
}
console.log('nodes fields', Object.keys(j.nodes[0]));
console.log('items[0]', JSON.stringify(j.items[0]).slice(0, 600));
