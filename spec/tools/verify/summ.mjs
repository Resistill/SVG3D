import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
for (const f of fs.readdirSync(dir).filter(x=>x.endsWith('.compile.json'))) {
  const j=JSON.parse(fs.readFileSync(dir+f));
  const kinds={}; const late=[];
  for (const n of j.nodes){kinds[n.kind]=(kinds[n.kind]||0)+1; if(n.late) late.push(n.path);}
  console.log(f, 'nodes', j.nodes.length, 'errors', j.errors.length, 'warnings', j.warnings.length);
  console.log('  kinds', JSON.stringify(kinds));
  console.log('  late', JSON.stringify(late));
}
