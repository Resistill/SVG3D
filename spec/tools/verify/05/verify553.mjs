import fs from 'node:fs';
const R = (n) => JSON.parse(fs.readFileSync(`/home/user/SVG3D/corpus/reference/examples/${n}.compile.json`,'utf8'));
const base = R('spaceport'), edit = R('spaceport_edit'), bp = R('spaceport_blueprint');
const log = (...a) => console.log(...a);
log('base nodes', base.nodes.length, 'ghost', base.nodes.filter(n=>n.ghost).length, 'items', base.items.length, 'curves', base.curves.length, 'facts', base.facts.length, 'errors', base.errors.length, 'warnings', base.warnings.length);
log('root children', base.nodes.filter(n=>n.depth===0).map(n=>n.path).join(' '));
log('files', [...new Set(base.nodes.map(n=>n.file))]);
log('fact keys', Object.keys(base.facts[0]));
log('edit nodes', edit.nodes.length, 'facts', edit.facts.length, 'errors', JSON.stringify(edit.errors), 'variants', JSON.stringify(edit.variants));
log('edit root children', edit.nodes.filter(n=>n.depth===0).length, edit.nodes.filter(n=>n.depth===0).at(-1).path);
const bp_ = new Set(base.nodes.map(n=>n.path)), ep = new Set(edit.nodes.map(n=>n.path));
log('only in edit', [...ep].filter(p=>!bp_.has(p)).length, [...ep].filter(p=>!bp_.has(p)).join(' '));
log('only in base', [...bp_].filter(p=>!ep.has(p)).join(' '));
const eq = (a,b)=>JSON.stringify(a)===JSON.stringify(b);
const bm = new Map(base.nodes.map(n=>[n.path,n])), em = new Map(edit.nodes.map(n=>[n.path,n]));
let same=0, diff=[];
for (const [p,n] of em) { const b = bm.get(p); if(!b) continue; if (eq(n.world,b.world)&&eq(n.box,b.box)) same++; else diff.push(p); }
log('common', same+diff.length, 'same', same, 'diff', diff.length); log('diff list', diff.join(' '));
const h2 = em.get('hangar2'), h2r = em.get('hangar2.roof'), hr = em.get('hangar.roof');
log('hangar2', h2.file, h2.line, 'hangar2.roof', h2r.file, h2r.line);
log('items hangar2.roof', JSON.stringify(edit.items.filter(i=>i.path==='hangar2.roof').map(i=>i.material)), 'hangar.roof', JSON.stringify(edit.items.filter(i=>i.path==='hangar.roof').map(i=>i.material)));
log('edit hangar2.body.opening box', JSON.stringify(em.get('hangar2.body.opening')?.box));
// blueprint
log('bp nodes', bp.nodes.length, 'skins', bp.nodes.filter(n=>n.skin).length, 'hidden', bp.nodes.filter(n=>n.hidden).map(n=>n.path), 'items', bp.items.length, 'facts', bp.facts.length, 'errors', bp.errors.length, 'warnings', bp.warnings.length);
log('materials base', JSON.stringify(base.materials), '\nmaterials bp', JSON.stringify(bp.materials));
log('defs', JSON.stringify(bp.defs), 'variants', JSON.stringify(bp.variants));
log('ribbed nodes', bp.nodes.filter(n=>n.kind==='ribbed').map(n=>n.path));
const nonskin = bp.nodes.filter(n=>!n.skin);
log('nonskin', nonskin.length, 'seq same', eq(nonskin.map(n=>n.path), base.nodes.map(n=>n.path)));
const keys = ['path','kind','types','line','file','depth','late','ghost','of','world','box','tight','tags','concepts','id','meta','curve'];
let bad=[]; for (let i=0;i<nonskin.length;i++) for (const k of keys) if(!eq(nonskin[i][k], base.nodes[i][k])) bad.push(nonskin[i].path+':'+k);
log('nonskin field diffs', bad.length, bad.slice(0,10).join(' '));
log('facts same', eq(bp.facts.map(f=>[f.rel,f.a,f.b,f.gap,f.eps,f.provider,f.text]), base.facts.map(f=>[f.rel,f.a,f.b,f.gap,f.eps,f.provider,f.text])), 'facts eq full', eq(bp.facts, base.facts));
log('factsSkipped', bp.factsSkipped, base.factsSkipped, 'providers', JSON.stringify(bp.factProviders));
const skins = bp.nodes.filter(n=>n.skin); const cnt = {}; for (const s of skins) cnt[s.id]=(cnt[s.id]??0)+1; log('skin ids', JSON.stringify(cnt));
log('skin concepts nonempty', skins.filter(s=>s.concepts.length).length);
const bi = new Set(base.items.map(i=>i.path)), pi = new Set(bp.items.map(i=>i.path));
log('items only base', [...bi].filter(p=>!pi.has(p)), 'only bp', [...pi].filter(p=>!bi.has(p)).length);
log('wings.annex mats', JSON.stringify(bp.items.filter(i=>/annex$/.test(i.path)).map(i=>[i.path,i.material])));
for (const p of ['hangar.roof.skin','hangar.roof.skin.base','hangar.roof.skin.rib']) { const n = bp.nodes.find(x=>x.path===p); log(p, n.file, n.line, n.kind); }
// position of skin: after hangar.roof's children?
const idx = bp.nodes.findIndex(n=>n.path==='hangar.roof'); log('after hangar.roof:', bp.nodes.slice(idx,idx+3).map(n=>n.path));
log('base concepts', base.concepts.length, 'patterns', base.patterns.length, 'checks', base.checks.length, 'bp', bp.concepts.length, bp.patterns.length, bp.checks.length);
log('skin late', skins.filter(s=>s.late).map(s=>s.path));
