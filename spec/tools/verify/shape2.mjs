import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const names=['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit'];
const seen={};
for(const n of names){const J=JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
 const walk=(it)=>{const t=it.geom?it.geom.type:it.kind+'/'+it.op; if(!seen[t]){seen[t]=1; let s=JSON.stringify(it); if(s.length>900) s=s.slice(0,900)+'...'; console.log(n,t,s);} if(it.children) it.children.forEach(walk);};
 J.items.forEach(walk);
 // hidden nodes present in items?
 const hidden=J.nodes.filter(x=>x.hidden).map(x=>x.path); const ip=new Set(); const w2=(it)=>{ip.add(it.path); if(it.children) it.children.forEach(w2);}; J.items.forEach(w2);
 if(hidden.length) console.log(n,'hidden',hidden.length,'in items',hidden.filter(h=>ip.has(h)).length, hidden.slice(0,5));
}
