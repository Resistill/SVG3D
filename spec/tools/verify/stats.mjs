import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const names=['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit'];
const rels={},provs={},eps={},keys={},gaps=[];let total=0;
for(const n of names){
  const j=JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
  console.log(n,'facts',j.facts.length,'factsSkipped',JSON.stringify(j.factsSkipped),'factProviders',JSON.stringify(j.factProviders),'nodes',j.nodes.length,'items',j.items.length);
  for(const f of j.facts){total++;rels[f.rel]=(rels[f.rel]||0)+1;provs[f.provider]=(provs[f.provider]||0)+1;eps[f.eps]=(eps[f.eps]||0)+1;keys[Object.keys(f).join(',')]=(keys[Object.keys(f).join(',')]||0)+1;gaps.push([f.rel,f.gap]);}
}
console.log('total',total);console.log('rels',rels);console.log('providers',provs);console.log('eps',eps);console.log('key shapes',keys);
const byrel={};for(const [r,g] of gaps){(byrel[r]??=[]).push(g);}
for(const r in byrel){const g=byrel[r].sort((a,b)=>a-b);console.log(r,'gap min',g[0],'max',g[g.length-1],'zero',g.filter(x=>x===0).length,'neg',g.filter(x=>x<0).length,'pos',g.filter(x=>x>0).length);}
