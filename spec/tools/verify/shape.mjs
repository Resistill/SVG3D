import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const j=JSON.parse(fs.readFileSync(dir+'basics.compile.json','utf8'));
console.log('ITEM sample', JSON.stringify(j.items[0]).slice(0,600));
console.log('ITEM keys', [...new Set(j.items.flatMap(i=>Object.keys(i)))]);
console.log('NODE sample', JSON.stringify(j.nodes[1]).slice(0,800));
console.log('FACTS', j.facts.map(f=>JSON.stringify(f)).join('\n'));
const names=['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit'];
const kinds={};const geomTypes={};
for(const n of names){const J=JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
 for(const f of J.facts) if(f.approx) console.log('APPROX',n,JSON.stringify(f));
 for(const it of J.items){kinds[it.kind]=(kinds[it.kind]||0)+1; if(it.geom) geomTypes[it.geom.type]=(geomTypes[it.geom.type]||0)+1;}
}
console.log('item kinds',kinds,'geom types',geomTypes);
