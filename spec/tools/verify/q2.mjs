import fs from 'node:fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const L=n=>JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
const sp=L('spaceport'), bp=L('spaceport_blueprint'), ed=L('spaceport_edit');
// facts compare
const ft=j=>j.facts.map(f=>f.text);
const eq=(a,b)=>a.length===b.length&&a.every((x,i)=>x===b[i]);
console.log('facts sp vs bp identical:',eq(ft(sp),ft(bp)));
const fkeys=j=>j.facts.map(f=>JSON.stringify([f.rel,f.a,f.b,f.gap,f.eps,f.provider]));
console.log('facts (rel,a,b,gap,eps,provider) sp vs bp identical:',eq(fkeys(sp),fkeys(bp)));
// nodes compare: for each node in sp, find in bp same path; compare world, box, tight, kind, hidden
const bm=new Map(bp.nodes.map(n=>[n.path,n]));
let diff=0, cnt=0;
for (const n of sp.nodes){ const m=bm.get(n.path); if(!m){console.log('missing',n.path);continue;} cnt++;
  for (const k of ['world','box','tight','kind','types','ghost','of','late','depth','line','file','tags','concepts']) if(JSON.stringify(n[k])!==JSON.stringify(m[k])){diff++;console.log('diff',n.path,k,JSON.stringify(n[k]),JSON.stringify(m[k]));}
  if(n.hidden!==m.hidden) console.log('hidden diff',n.path,n.hidden,m.hidden);
}
console.log('compared',cnt,'diffs',diff);
const extra=bp.nodes.filter(n=>!sp.nodes.some(x=>x.path===n.path));
console.log('extra in bp',extra.length, extra.every(n=>n.skin), extra.slice(0,4).map(n=>n.path+'|'+n.file+'|'+n.line));
// order of nodes same?
console.log('sp node order prefix of bp order?', JSON.stringify(sp.nodes.map(n=>n.path))===JSON.stringify(bp.nodes.filter(n=>!n.skin).map(n=>n.path)));
// items: annex color
for (const j of [sp,bp]) for (const it of j.items) if(/annex/.test(it.path||it.node||'')) console.log(JSON.stringify(it).slice(0,300));
console.log('item keys', Object.keys(sp.items[0]));
// item count difference bp vs sp: 133-111=22
const ip=j=>new Set(j.items.map(i=>i.path));
console.log('items in bp not in sp', [...ip(bp)].filter(p=>!ip(sp).has(p)).length, 'items in sp not in bp', [...ip(sp)].filter(p=>!ip(bp).has(p)));
// edit: node count breakdown
console.log('ed facts', ed.facts.length, 'sp facts', sp.facts.length);
const em=new Map(ed.nodes.map(n=>[n.path,n]));
const onlyEd=ed.nodes.filter(n=>!sp.nodes.some(x=>x.path===n.path)).map(n=>n.path);
const onlySp=sp.nodes.filter(n=>!em.has(n.path)).map(n=>n.path);
console.log('only in edit',onlyEd.length,onlyEd.join(' '));
console.log('only in sp',onlySp.length,onlySp.join(' '));
let same=0, changed=[];
for (const n of sp.nodes){const m=em.get(n.path); if(!m) continue; if(JSON.stringify([n.world,n.box])===JSON.stringify([m.world,m.box])) same++; else changed.push(n.path);}
console.log('edit: unchanged world+box',same,'changed',changed.length, changed.join(' '));
for (const it of ed.items) if(/hangar2\.roof|hangar\.roof/.test(it.path)) console.log(JSON.stringify(it).slice(0,300));
console.log('hangar2 node', JSON.stringify(ed.nodes.find(n=>n.path==='hangar2')).slice(0,400));
console.log('hangar2.roof node', JSON.stringify(ed.nodes.find(n=>n.path==='hangar2.roof')).slice(0,400));
console.log('skin node', JSON.stringify(bp.nodes.find(n=>n.path==='hangar.roof.skin')).slice(0,500));
console.log('variants', JSON.stringify(bp.variants));
