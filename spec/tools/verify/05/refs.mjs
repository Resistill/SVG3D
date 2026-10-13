import fs from 'node:fs';
const spec = '/home/user/SVG3D/spec/';
const files = {1:'01-syntax.md',2:'02-geometry.md',3:'03-solving.md',4:'04-derivation.md',5:'05-layers.md',6:'06-facts.md',7:'07-semantics.md',8:'08-metadata.md',9:'09-compat.md',10:'10-materials.md'};
const heads = {};
for (const [n,f] of Object.entries(files)) {
  heads[n] = new Set();
  for (const line of fs.readFileSync(spec+f,'utf8').split('\n')) {
    const m = /^#+\s+(\d+(?:\.\d+)*)\b/.exec(line); if (m) heads[n].add(m[1]);
  }
}
const txt = fs.readFileSync(spec+'05-layers.md','utf8');
const refs = new Map();
for (const m of txt.matchAll(/\b(\d{1,2})\.(\d+)(?:\.(\d+))?\b/g)) {
  const ch = m[1]; if (!files[ch]) continue;
  const sec = m[3] ? `${ch}.${m[2]}.${m[3]}` : `${ch}.${m[2]}`;
  if (!refs.has(sec)) refs.set(sec, []);
  const lineNo = txt.slice(0, m.index).split('\n').length;
  refs.get(sec).push(lineNo);
}
const missing = [];
for (const [sec, lines] of refs) { const ch = sec.split('.')[0]; if (!heads[ch].has(sec)) missing.push(`${sec} (lines ${[...new Set(lines)].join(',')})`); }
console.log('missing:', missing.join('\n'));
