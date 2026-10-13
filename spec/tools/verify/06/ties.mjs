// 检查 6.5.6：体积相等的 a/b 平局是否都满足 σ(a) > σ(b)（扫描序 = (Ω_min_x, 节点序)）；并检查 on 以外的事实 vol(a) <= vol(b)
import fs from 'fs';
import { compile, registerFactProvider } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const dir='/home/user/SVG3D/corpus/reference/examples/', exdir='/home/user/resistill/svg3d-prototype/examples/';
let captured=null; registerFactProvider({name:'capture',rels:[],derive(objs,ctx){captured={objs,ctx};return[];}});
const vol=(b)=>(b.max[0]-b.min[0])*(b.max[1]-b.min[1])*(b.max[2]-b.min[2]);
let ties=0,bad=0,badvol=0,allpairs=0,xties=0;
for(const name of ['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit']){
  const ref=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8'));
  compile(fs.readFileSync(exdir+name+'.s3d','utf8'),{file:'examples/'+name+'.s3d',resolve:(p)=>fs.readFileSync('/home/user/resistill/svg3d-prototype/'+p,'utf8'),factProviders:['capture']});
  const objs=captured.objs.map((o,i)=>({path:o.path,i,x:o.wb.min[0],v:vol(o.box)}));
  const sweep=[...objs].sort((p,q)=>p.x-q.x||p.i-q.i); const sidx=new Map(sweep.map((o,k)=>[o.path,k])); const byP=new Map(objs.map(o=>[o.path,o]));
  for(const f of ref.facts){ if(f.rel==='on')continue; const a=byP.get(f.a),b=byP.get(f.b); allpairs++;
    if(a.v>b.v){badvol++;console.log('VOL',name,f.text,a.v,b.v);}
    if(a.v===b.v){ties++; if(sidx.get(f.a)<sidx.get(f.b)){bad++;console.log('TIE-BAD',name,f.text);} else {console.log('tie ok',name,f.text,'x',a.x,b.x,'i',a.i,b.i); if(a.x===b.x)xties++;}}
  }
}
console.log('non-on facts',allpairs,'ties',ties,'bad',bad,'vol-order bad',badvol,'ties with equal x (decided by node order)',xties);
