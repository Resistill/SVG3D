// 第 6 章核对脚本的工程版：对 work-snapshot 的 dorm / h1 / sc（ε = FACT_EPS / unit）
// 第 6 章验证：从原型只取凸块（第 2 章）与世界变换（第 3 章），事实判定按 06-facts.md 自行实现，再与参考输出 facts[] 比
import fs from 'fs';
import { compile, registerFactProvider, FACT_EPS } from '/home/user/resistill/svg3d-prototype/src/s3d.js';
const P = { FACT_EPS };
const dir='/home/user/SVG3D/corpus/reference/examples/', exdir='/home/user/resistill/svg3d-prototype/examples/';
const names=process.argv.slice(2).length?process.argv.slice(2):['dorm','h1'];
const WORKDIR = {dorm: 'dorm/final', h1: 'h1/final', sc: 'sc/full'};
const projOf = (name) => { const dir = '/home/user/resistill/svg3d-prototype/work-snapshot/' + WORKDIR[name] + '/'; const cfg = JSON.parse(fs.readFileSync(dir + 'project.json', 'utf8')); const unit = cfg.unit ?? 1; return { dir, unit, text: fs.readFileSync(dir + 'scene.s3d', 'utf8'), opts: { file: 'scene.s3d', factEps: P.FACT_EPS / unit, metre: 1 / unit, resolve: (p) => fs.readFileSync(dir + p, 'utf8') } }; };

const CSG=new Set(['union','subtract','intersect']);
const sub=(a,b)=>[a[0]-b[0],a[1]-b[1],a[2]-b[2]], dot=(a,b)=>a[0]*b[0]+a[1]*b[1]+a[2]*b[2];
const cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const len=(a)=>Math.sqrt(dot(a,a)), scl=(a,s)=>[a[0]*s,a[1]*s,a[2]*s];
const mv=(M,v)=>M.map(r=>r[0]*v[0]+r[1]*v[1]+r[2]*v[2]);
const tr=(A)=>[0,1,2].map(i=>[A[0][i],A[1][i],A[2][i]]);
const EPS=0.02;
// 6.3 凸块包围盒
const partBox=(P)=>P.sphere?{min:sub(P.c,[P.r,P.r,P.r]),max:[P.c[0]+P.r,P.c[1]+P.r,P.c[2]+P.r]}:bounds(P.verts);
function bounds(pts){const mn=[Infinity,Infinity,Infinity],mx=[-Infinity,-Infinity,-Infinity];for(const p of pts)for(let k=0;k<3;k++){if(p[k]<mn[k])mn[k]=p[k];if(p[k]>mx[k])mx[k]=p[k];}return{min:mn,max:mx};}
const unionBox=(bs)=>({min:[0,1,2].map(k=>Math.min(...bs.map(b=>b.min[k]))),max:[0,1,2].map(k=>Math.max(...bs.map(b=>b.max[k])))});
const ov=(a,b,k)=>Math.min(a.max[k],b.max[k])-Math.max(a.min[k],b.min[k]);
const ovI=(a,b)=>Math.min(a[1],b[1])-Math.max(a[0],b[0]);
// 6.4 投影区间与分离量
function proj(P,L){if(P.sphere){const c=dot(P.c,L);return[c-P.r,c+P.r];}let lo=Infinity,hi=-Infinity;for(const v of P.verts){const x=dot(v,L);if(x<lo)lo=x;if(x>hi)hi=x;}return[lo,hi];}
function partGap(A,B){
  const axes=[];
  if(!A.sphere)axes.push(...A.normals); if(!B.sphere)axes.push(...B.normals);
  if(!A.sphere&&!B.sphere)for(const a of A.edges)for(const b of B.edges)axes.push(cross(a,b));
  if(A.sphere&&B.sphere)axes.push(sub(B.c,A.c));
  else if(A.sphere||B.sphere){const [S,P]=A.sphere?[A,B]:[B,A];for(const v of P.verts)axes.push(sub(v,S.c));}
  let best=-Infinity;
  for(let L of axes){const l=len(L);if(l<1e-9)continue;L=scl(L,1/l);const[a0,a1]=proj(A,L),[b0,b1]=proj(B,L);best=Math.max(best,b0-a1,a0-b1);}
  return best;
}
function objGap(P,Q,eps){let best=Infinity;const touching=[];
  for(let i=0;i<P.parts.length;i++)for(let j=0;j<Q.parts.length;j++){const a=P.pb[i],b=Q.pb[j];
    if(ov(a,b,0)<-eps||ov(a,b,1)<-eps||ov(a,b,2)<-eps)continue;
    const g=partGap(P.parts[i],Q.parts[j]);if(g<best)best=g;if(g<=eps)touching.push([i,j]);}
  return{gap:best,touching};}
const boxVol=(b)=>(b.max[0]-b.min[0])*(b.max[1]-b.min[1])*(b.max[2]-b.min[2]);
const isAxisAligned=(M)=>M.every(r=>r.every(x=>Math.abs(x)<1e-6||Math.abs(Math.abs(x)-1)<1e-6));
const apply=(T,p)=>{const q=mv(T.R,p);return[q[0]+T.t[0],q[1]+T.t[1],q[2]+T.t[2]];};
const invT=(A)=>{const Rt=tr(A.R);return{R:Rt,t:scl(mv(Rt,A.t),-1)};};
const compose=(A,B)=>({R:A.R.map(r=>[0,1,2].map(j=>r[0]*B.R[0][j]+r[1]*B.R[1][j]+r[2]*B.R[2][j])),t:apply(A,B.t)});
const corners=(b)=>[0,1,2,3,4,5,6,7].map(i=>[i&1?b.max[0]:b.min[0],i&2?b.max[1]:b.min[1],i&4?b.max[2]:b.min[2]]);
// 6.5.2 on
function onTest(A,B,touching,eps){
  const rel=compose(invT(B.W),A.W); if(Math.abs(Math.abs(rel.R[1][1])-1)>1e-6)return null;
  const up=mv(B.W.R,[0,1,0]),bx=mv(B.W.R,[1,0,0]),bz=mv(B.W.R,[0,0,1]);
  const bottom=Math.min(...A.parts.map(P=>proj(P,up)[0]));
  for(const[i,j]of touching){const top=proj(B.parts[j],up)[1]; if(Math.abs(bottom-top)>=eps)continue;
    if(ovI(proj(A.parts[i],bx),proj(B.parts[j],bx))>eps&&ovI(proj(A.parts[i],bz),proj(B.parts[j],bz))>eps)return bottom-top;}
  return null;}
function insideTest(A,B,eps){
  if(B.parts.length!==1||boxVol(A.box)>=boxVol(B.box)*0.9)return false;const PB=B.parts[0];
  if(PB.sphere)return A.parts.every(P=>(P.sphere?len(sub(P.c,PB.c))+P.r:Math.max(...P.verts.map(v=>len(sub(v,PB.c)))))<=PB.r+eps);
  return PB.normals.every(n=>{const[lo,hi]=proj(PB,n);return A.parts.every(P=>{const[a0,a1]=proj(P,n);return a1<=hi+eps&&a0>=lo-eps;});});}
let captured=null;
registerFactProvider({name:'capture',rels:[],derive(objs,ctx){captured={objs,ctx};return[];}});
const tot={ok:0,gapdiff:0,approxdiff:0,missing:0,extra:0,flip:0,ties:0,cands:0,pairs:0,objs:0};
for(const name of names){
  const ref=JSON.parse(fs.readFileSync('/home/user/resistill/svg3d-prototype/out/work/'+name+'.compile.json','utf8'));
  const pj=projOf(name);
  captured=null;
  const r=compile(pj.text,{...pj.opts,factProviders:['capture']});
  const EPS_P = FACT_EPS/pj.unit;
  const objs=captured.objs.map((o,idx)=>({idx,path:o.path,kind:o.kind,n:o.n,W:o.W,box:o.box,parts:o.parts,pb:o.parts.map(partBox),
    approx:o.approx||CSG.has(o.kind), // 2.5.6：布尔代理 approx=true
    isBoxy:o.parts.length===1&&(['box','beam','wedge'].includes(o.n.geom?.type)||CSG.has(o.kind)||o.n.geom?.type==='mesh')}));
  for(const o of objs)o.wb=unionBox(o.pb);
  const eps=EPS_P; const facts=[];
  let cands=0,pairs=0;
  // 6.2.3 扫描序：按 (对象盒 x 最小值, 节点序号) 升序；候选对 (P, Q) 里 P 是扫描序靠后者（与原型 broadPairs 同）
  const order=[...objs].sort((p,q)=>(p.wb.min[0]-q.wb.min[0])||(p.idx-q.idx));
  const active=[]; const candPairs=[];
  for(const P of order){
    for(let i=active.length-1;i>=0;i--)if(active[i].wb.max[0]<P.wb.min[0]-eps)active.splice(i,1);
    for(const Q of active)if(ov(P.wb,Q.wb,1)>=-eps&&ov(P.wb,Q.wb,2)>=-eps)candPairs.push([P,Q]);
    active.push(P);
  }
  for(const [P,Q] of candPairs){ cands++;
    if(P.n.isAncestorOf(Q.n)||Q.n.isAncestorOf(P.n))continue;
    const{gap,touching}=objGap(P,Q,eps); if(gap>eps)continue; pairs++;
    const approx=P.approx||Q.approx;
    let g;
    if((g=onTest(P,Q,touching,eps))!==null){facts.push({rel:'on',a:P.path,b:Q.path,gap:g,approx});continue;}
    if((g=onTest(Q,P,touching.map(([a,b])=>[b,a]),eps))!==null){facts.push({rel:'on',a:Q.path,b:P.path,gap:g,approx});continue;}
    const vP=boxVol(P.box),vQ=boxVol(Q.box); let tie=vP===vQ; if(tie)tot.ties++;
    const[A,B]=vP<=vQ?[P,Q]:[Q,P];
    if(A.isBoxy&&B.isBoxy){const rel=compose(invT(B.W),A.W);
      if(isAxisAligned(rel.R)){const a=bounds(corners(A.box).map(p=>apply(rel,p))),b=B.box;const[ox,oy,oz]=[0,1,2].map(k=>ov(a,b,k));let face=null;
        if(oy>eps){if(Math.abs(a.max[0]-b.min[0])<eps&&oz>eps)face='left';else if(Math.abs(a.min[0]-b.max[0])<eps&&oz>eps)face='right';else if(Math.abs(a.max[2]-b.min[2])<eps&&ox>eps)face='back';else if(Math.abs(a.min[2]-b.max[2])<eps&&ox>eps)face='front';}
        if(face){facts.push({rel:'against',a:A.path,b:B.path,face,gap,approx,tie});continue;}}}
    if(insideTest(A,B,eps)){facts.push({rel:'inside',a:A.path,b:B.path,gap,approx});continue;}
    facts.push({rel:gap>=-eps?'touching':'intersects',a:A.path,b:B.path,gap,approx,tie});
  }
  // 比较
  const key=(f)=>f.rel+'|'+f.a+'|'+f.b, skey=(f)=>f.rel+'|'+[f.a,f.b].sort().join('|');
  const mine=new Map(facts.map(f=>[key(f),f])), theirs=new Map(ref.facts.map(f=>[key(f),f]));
  const mineS=new Map(facts.map(f=>[skey(f),f])), theirsS=new Map(ref.facts.map(f=>[skey(f),f]));
  let ok=0,gapdiff=0,approxdiff=0,missing=0,extra=0,flip=0; const notes=[];
  for(const[k,f]of theirs){const m=mine.get(k); let mm=m;
    if(!m){const s=mineS.get(skey(f)); if(s){flip++;mm=s;notes.push('FLIP '+f.text+' (mine a='+s.a+' vol tie='+s.tie+')');} else {missing++;notes.push('MISSING '+f.text+' gap='+f.gap);continue;}}
    if(Math.abs(mm.gap-f.gap)>1e-4+1e-9){gapdiff++;notes.push('GAP '+f.text+' ref='+f.gap+' mine='+mm.gap);}
    if(!!mm.approx!==!!f.approx){approxdiff++;notes.push('APPROX '+f.text+' ref='+!!f.approx+' mine='+mm.approx);}
    if(f.rel==='against'){const face=/\.(\w+)$/.exec(f.text)[1]; if(mm.face!==face)notes.push('FACE '+f.text+' mine='+mm.face);}
    ok++;}
  for(const[k,f]of mine)if(!theirs.has(k)&&!theirsS.has(skey(f))){extra++;notes.push('EXTRA '+f.rel+' '+f.a+' '+f.b+' gap='+f.gap);}
  console.log(`${name}: objs ${objs.length} cands ${cands} pairs(gap<=eps) ${pairs} mine ${facts.length} ref ${ref.facts.length} matched ${ok} flipped ${flip} gapdiff ${gapdiff} approxdiff ${approxdiff} missing ${missing} extra ${extra}`);
  for(const n of notes)console.log('   '+n);
  tot.ok+=ok;tot.gapdiff+=gapdiff;tot.approxdiff+=approxdiff;tot.missing+=missing;tot.extra+=extra;tot.flip+=flip;tot.cands+=cands;tot.pairs+=pairs;tot.objs+=objs.length;
}
console.log('TOTAL',JSON.stringify(tot));
