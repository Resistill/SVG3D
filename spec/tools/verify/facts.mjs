import fs from 'fs';
const name = process.argv[2]; const filt = process.argv[3];
const j = JSON.parse(fs.readFileSync(`corpus/reference/examples/${name}.compile.json`,'utf8'));
for (const f of j.facts) { if (!filt || f.text.includes(filt)) console.log(f.text, 'gap', f.gap, f.approx?'approx':''); }
