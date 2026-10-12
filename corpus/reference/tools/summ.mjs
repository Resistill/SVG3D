import fs from 'node:fs';
const R='out/reference/work/';
for (const p of ['dorm','h1','sc']) {
  const s=JSON.parse(fs.readFileSync(R+p+'/score.stdout.json','utf8'));
  const c=JSON.parse(fs.readFileSync(R+p+'/compare.json','utf8'));
  console.log(p, 'S',s.S,'explain',s.explain, 'L_stmt',s.L_stmt,'L_mesh',s.L_mesh,'S_mesh',s.S_mesh);
  console.log(JSON.stringify(c));
}
for (const p of ['sp','dorm','h1']) { const s=JSON.parse(fs.readFileSync(R+'panels-g10/'+p+'.score.json','utf8')); const c=JSON.parse(fs.readFileSync(R+'panels-g10/'+p+'.compare.json','utf8')); console.log('g10',p,'S',s.S,'explain',s.explain); console.log(JSON.stringify(c)); }
for (const f of fs.readdirSync(R+'../examples').concat(fs.readdirSync(R+'../exports')).filter(f=>f.endsWith('compile.json'))) {
  const d=JSON.parse(fs.readFileSync(R+'../'+(fs.existsSync(R+'../examples/'+f)?'examples/':'exports/')+f,'utf8'));
  console.log(f,'nodes',d.nodes.length,'facts',d.facts.length,'err',d.errors.length,'warn',d.warnings.length);
}
for (const p of ['dorm','h1','sc']) { const d=JSON.parse(fs.readFileSync(R+p+'/scene.compile.json','utf8')); console.log(p,d.meta, 'err',d.errors.length,'warn',d.warnings.length); if(d.errors.length) console.log(JSON.stringify(d.errors.slice(0,3))); }
