import { readFileSync } from 'node:fs';
import { compile } from './s3d_dbg.js';
const src = readFileSync(process.argv[2], 'utf8');
const r = compile(src, { file: process.argv[2] });
console.log(JSON.stringify(r.patterns.map(p=>({n:p.name,nm:p.matches.length,e:p.error}))), JSON.stringify(r.errors));
