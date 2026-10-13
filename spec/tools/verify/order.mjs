// 检查：参考输出的事实顺序 = 候选对按（P 的扫描序号, Q 的扫描序号）字典序？扫描序 = 按 W.min.x 升序、相等按节点顺序
import fs from 'fs';
import { compile, registerFactProvider } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const dir='/home/user/SVG3D/corpus/reference/examples/', exdir='/home/user/resistill/svg3d-prototype/examples/';
let captured=null; registerFactProvider({name:'capture',rels:[],derive(objs,ctx){captured={objs,ctx};return[];}});
for(const name of ['basics','curves','rotation','semantics','spaceport','spaceport_blueprint','spaceport_edit']){
  const ref=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8'));
  compile(fs.readFileSync(exdir+name+'.s3d','utf8'),{file:'x',resolve:(p)=>fs.readFileSync('/home/user/resistill/svg3d-prototype/'+p,'utf8'),factProviders:['capture']});
  const objs=captured.objs.map((o,i)=>({path:o.path,i,x:o.wb.min[0]}));
  const sweep=[...objs].sort((p,q)=>p.x-q.x||p.i-q.i); const sidx=new Map(sweep.map((o,k)=>[o.path,k]));
  let prev=[-1,-1], bad=0, dupX=0;
  for(const f of ref.facts){const a=sidx.get(f.a),b=sidx.get(f.b); const P=Math.max(a,b),Q=Math.min(a,b); const key=[P,Q]; if(key[0]<prev[0]||(key[0]===prev[0]&&key[1]<prev[1]))bad++; prev=key;}
  for(let k=1;k<sweep.length;k++)if(sweep[k].x===sweep[k-1].x)dupX++;
  console.log(name,'order violations',bad,'equal-x neighbours',dupX);
}
