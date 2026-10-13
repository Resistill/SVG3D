import fs from 'fs';
const dir='/home/user/SVG3D/corpus/reference/examples/';
const q=[['basics',/^(sticker|turned|stairs|beam|crate|orb|table|ball) /],['curves',/lamps\.\d+\.bulb on lamps\.\d+\.pole|plaza\.benches\.r2\.seat intersects lane|tower on ground|plaza\.spout/],
['rotation',/coaster\.legs\.\d+\.col touching ground|coaster\.ties\.\d+\.tie touching coaster\.railL|radar\.(feed|mast)/],
['semantics',/^(rampW|river) |trees\.\d+\.crown on trees\.\d+\.trunk|watch2\.rail\.\d+\.post on watch2\.cabin|street\.h1\.half2/],
['spaceport',/^(arm|spare|berm|mast|radome|wings\.stair2|wings\.annex|wings\.mirrored\.annex|hangar\.body|hangar\.roof|rocket\.nose) /]];
for(const [name,re] of q){const j=JSON.parse(fs.readFileSync(dir+name+'.compile.json','utf8')); const hits=j.facts.filter(f=>re.test(f.text)); const cnt={}; for(const f of hits){const k=f.text.replace(/\.\d+\./g,'.N.'); cnt[k]=(cnt[k]||0)+1;} console.log(name); for(const k in cnt){const ex=hits.find(f=>f.text.replace(/\.\d+\./g,'.N.')===k); console.log('  ',k,'x'+cnt[k],'gap',ex.gap,ex.approx?'approx':'');}}
