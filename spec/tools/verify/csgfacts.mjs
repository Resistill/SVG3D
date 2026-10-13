import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const names=['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit'];
const faces={};
for(const n of names){const J=JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
 const byPath=new Map(J.nodes.map(x=>[x.path,x]));
 const csg=new Set(J.nodes.filter(x=>['union','subtract','intersect'].includes(x.kind)).map(x=>x.path));
 for(const f of J.facts){ if(csg.has(f.a)||csg.has(f.b)) console.log(n,'CSG',JSON.stringify(f)); if(f.rel==='against'){const m=/\.(\w+)$/.exec(f.text); faces[m[1]]=(faces[m[1]]||0)+1;} }
 // kinds of participants
 const kinds={}; for(const f of J.facts){ for(const p of [f.a,f.b]){const k=byPath.get(p)?.kind; kinds[k]=(kinds[k]||0)+1;} }
 console.log(n,'participant kinds',JSON.stringify(kinds));
 // ghosts participating
 console.log(n,'facts with ghost', J.facts.filter(f=>byPath.get(f.a)?.ghost||byPath.get(f.b)?.ghost).length, 'hidden', J.facts.filter(f=>byPath.get(f.a)?.hidden||byPath.get(f.b)?.hidden).length);
}
console.log('against faces',faces);
