import fs from 'node:fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
for (const n of ['spaceport','spaceport_blueprint','spaceport_edit']) {
  const j=JSON.parse(fs.readFileSync(dir+n+'.compile.json','utf8'));
  const nodes=j.nodes;
  const files={}; for (const x of nodes) files[x.file]=(files[x.file]||0)+1;
  const skin=nodes.filter(x=>x.skin).length, hidden=nodes.filter(x=>x.hidden).length, ghost=nodes.filter(x=>x.ghost).length;
  console.log(n,'nodes',nodes.length,'items',j.items.length,'curves',j.curves.length,'facts',j.facts.length,'skipped',j.factsSkipped?.length,'errors',JSON.stringify(j.errors),'warnings',JSON.stringify(j.warnings));
  console.log('  files',JSON.stringify(files),'skin',skin,'hidden',hidden,'ghost',ghost);
  console.log('  concepts',(j.concepts||[]).map(c=>c.name+':'+(c.members?.length??c.count)).join(' '));
  console.log('  patterns',JSON.stringify(j.patterns).slice(0,200),'variants',JSON.stringify(j.variants),'checks',JSON.stringify(j.checks));
  console.log('  defs',JSON.stringify(j.defs),'materials',JSON.stringify(j.materials).slice(0,600));
  console.log('  top-level',nodes.filter(x=>x.depth===0).map(x=>x.path).join(' '));
}
