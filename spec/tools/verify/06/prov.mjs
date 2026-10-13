import { compile, registerFactProvider } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
registerFactProvider({name:'boom',rels:['x'],derive(){throw new Error('kaput');}});
registerFactProvider({name:'uni',rels:['tall'],derive(objs){return [{rel:'tall',a:objs[0].path,text:objs[0].path+' tall'}];}});
const src='box ground size=10,1,10 pin=top\nbox a size=1,1,1 on=ground\n';
for(const fp of [['nope'],['contact','boom'],['uni','contact'],undefined]){
  const r=compile(src,{factProviders:fp});
  console.log(JSON.stringify(fp),'providers',JSON.stringify(r.factProviders),'facts',JSON.stringify(r.facts),'errors',JSON.stringify(r.errors.map(e=>({m:e.message,line:e.line,file:e.file,node:e.node}))));
}
const r2=compile(src,{facts:false}); console.log('skipped',r2.factsSkipped,JSON.stringify(r2.facts),JSON.stringify(r2.factProviders));
const r3=compile(src+'concept c = on(ground)\n',{facts:false}); console.log('skipped+concept',JSON.stringify(r3.concepts),JSON.stringify(r3.errors),JSON.stringify(r3.warnings));
