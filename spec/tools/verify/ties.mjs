import fs from 'fs';
import { compile, registerFactProvider } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const dir='/home/user/SVG3D/corpus/reference/examples/', exdir='/home/user/resistill/svg3d-prototype/examples/';
let captured=null; registerFactProvider({name:'capture',rels:[],derive(objs,ctx){captured={objs,ctx};return[];}});
const boxVol=(b)=>(b.max[0]-b.min[0])*(b.max[1]-b.min[1])*(b.max[2]-b.min[2]);
for(const name of ['rotation','semantics','spaceport','curves','spaceport_edit','basics']){
  const ref=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8'));
  compile(fs.readFileSync(exdir+name+'.s3d','utf8'),{file:'x',resolve:(p)=>fs.readFileSync('/home/user/resistill/svg3d-prototype/'+p,'utf8'),factProviders:['capture']});
  const byPath=new Map(captured.objs.map((o,i)=>[o.path,{o,i}]));
  for(const f of ref.facts){ if(f.rel==='on')continue; const A=byPath.get(f.a),B=byPath.get(f.b);
    if(boxVol(A.o.box)===boxVol(B.o.box)){ console.log(name,f.text,'| a idx',A.i,'b idx',B.i,'| a wb.min.x',A.o.wb.min[0].toFixed(4),'b wb.min.x',B.o.wb.min[0].toFixed(4),'| a later in node order:',A.i>B.i,'| a has larger minx:',A.o.wb.min[0]>B.o.wb.min[0]); } }
}
