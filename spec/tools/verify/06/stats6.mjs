import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const names=['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit'];
const tot={on:0,against:0,inside:0,touching:0,intersects:0,facts:0,nodes:0,approx:0};
const faces={}; const gaps={on:[],against:[],inside:[],touching:[],intersects:[]};
const shapes=new Map(); let eps=new Set(), prov=new Set();
const ghostFacts={}, hiddenFacts={}, csgParts={};
for(const name of names){
  const j=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8'));
  const byPath=new Map(j.nodes.map(n=>[n.path,n]));
  const c={on:0,against:0,inside:0,touching:0,intersects:0};
  let g=0,h=0; const kinds=new Set(); const objs=new Set();
  for(const f of j.facts){c[f.rel]++; tot[f.rel]++; tot.facts++; eps.add(f.eps); prov.add(f.provider);
    if(f.approx) tot.approx++;
    const sh=Object.keys(f).join(','); shapes.set(sh,(shapes.get(sh)||0)+1);
    gaps[f.rel].push(f.gap);
    if(f.rel==='against'){const m=/\.(\w+)$/.exec(f.text)[1]; faces[m]=(faces[m]||0)+1;}
    const A=byPath.get(f.a),B=byPath.get(f.b);
    if(A.ghost||B.ghost) g++; if(A.hidden||B.hidden) h++;
    objs.add(f.a); objs.add(f.b); kinds.add(A.kind); kinds.add(B.kind);
    if(['union','subtract','intersect'].includes(A.kind)) (csgParts[name]??=new Set()).add(f.a);
    if(['union','subtract','intersect'].includes(B.kind)) (csgParts[name]??=new Set()).add(f.b);
  }
  tot.nodes+=j.nodes.length;
  console.log(name,'nodes',j.nodes.length,'facts',j.facts.length,JSON.stringify(c),'ghostFacts',g,'hiddenFacts',h,'skipped',j.factsSkipped,'providers',JSON.stringify(j.factProviders),'kinds',[...kinds].join(' '), 'csg', [...(csgParts[name]??[])].join(' '));
}
console.log('TOTAL',JSON.stringify(tot));
console.log('faces',JSON.stringify(faces));
console.log('shapes',[...shapes]);
console.log('eps',[...eps],'prov',[...prov]);
for(const r in gaps){const a=gaps[r]; if(!a.length)continue; const mn=Math.min(...a),mx=Math.max(...a); const z=a.filter(x=>x===0).length,neg=a.filter(x=>x<0).length,pos=a.filter(x=>x>0).length; console.log(r,'n',a.length,'min',mn,'max',mx,'zero',z,'neg',neg,'pos',pos);}
// approx facts list
for(const name of names){const j=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8')); for(const f of j.facts) if(f.approx) console.log(' approx',name,f.text,f.gap);}
