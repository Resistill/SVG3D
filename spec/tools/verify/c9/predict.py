# 预测 6.5.3（mesh 外包络 against）与 6.5.4（近似 B 的 inside）在参考输出里会改变的事实（只用参考输出的 world/box/tight 与 items.geom）
import json,glob,os,math
EPS_DIR=1e-6
def mT(R): return [[R[j][i] for j in range(3)] for i in range(3)]
def mv(R,v): return [sum(R[i][k]*v[k] for k in range(3)) for i in range(3)]
def mm(A,B): return [[sum(A[i][k]*B[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
def localbox(geom):
    t=geom['type']
    if t in ('box','wedge','mesh'): w,h,d=geom['w'],geom['h'],geom['d']; return ([-w/2,0,-d/2],[w/2,h,d/2])
    return None
files=sorted(glob.glob('corpus/reference/examples/*.compile.json'))+sorted(glob.glob('corpus/reference/exports/*.compile.json'))
for f in files:
    d=json.load(open(f))
    byp={n['path']:n for n in d['nodes']}
    geom={}
    for it in d['items']:
        if it.get('geom'): geom[it['path']]=it['geom']
    def kindof(p):
        k=byp[p]['kind']
        if '→' in k: k=k.split('→')[1]
        return k
    def basekind(p):
        n=byp[p]
        if n.get('of') and n.get('ghost'): return basekind(n['of'])
        return kindof(p)
    flips=[]
    for fa in d['facts']:
        a,b=fa['a'],fa['b']; rel=fa['rel']; eps=fa['eps']
        ka,kb=basekind(a),basekind(b)
        A,B=byp[a],byp[b]
        ga,gb=geom.get(a),geom.get(b)
        if rel in ('touching','intersects') and kb=='mesh' and gb:
            # inside test: A's corners (box-like) or tight corners, in B frame vs B local box ± eps
            lbB=localbox(gb); RB=B['world']['R']; tB=B['world']['t']; RBT=mT(RB)
            if ga and localbox(ga) and ka in ('box','wedge'):
                lo,hi=localbox(ga); RA=A['world']['R']; tA=A['world']['t']
                corners=[mv(RA,[x,y,z]) for x in (lo[0],hi[0]) for y in (lo[1],hi[1]) for z in (lo[2],hi[2])]
                corners=[[c[i]+tA[i] for i in range(3)] for c in corners]
                volA=(hi[0]-lo[0])*(hi[1]-lo[1])*(hi[2]-lo[2])
            else:
                tb=A['tight'] or A['box']
                corners=[[x,y,z] for x in (tb['min'][0],tb['max'][0]) for y in (tb['min'][1],tb['max'][1]) for z in (tb['min'][2],tb['max'][2])]
                volA=(tb['max'][0]-tb['min'][0])*(tb['max'][1]-tb['min'][1])*(tb['max'][2]-tb['min'][2])
            volB=gb['w']*gb['h']*gb['d']
            inB=[mv(RBT,[c[i]-tB[i] for i in range(3)]) for c in corners]
            ok=all(lbB[0][i]-eps<=p[i]<=lbB[1][i]+eps for p in inB for i in range(3))
            if ok and volA<0.9*volB: flips.append(('inside',fa['text'],fa['gap'],ka)); continue
        if rel=='touching' and abs(fa['gap'])<=eps and ('mesh' in (ka,kb)) and ka in ('box','wedge','mesh') and kb in ('box','wedge','mesh') and ga and gb:
            # against test 6.5.3 in B frame
            RA=A['world']['R']; RB=B['world']['R']; Rab=mm(mT(RB),RA)
            if not all(abs(r)<=EPS_DIR or abs(abs(r)-1)<=EPS_DIR for row in Rab for r in row): continue
            lo,hi=localbox(ga); tA=A['world']['t']; tB=B['world']['t']; RBT=mT(RB)
            corners=[mv(RA,[x,y,z]) for x in (lo[0],hi[0]) for y in (lo[1],hi[1]) for z in (lo[2],hi[2])]
            inB=[mv(RBT,[c[i]+tA[i]-tB[i] for i in range(3)]) for c in corners]
            amin=[min(p[i] for p in inB) for i in range(3)]; amax=[max(p[i] for p in inB) for i in range(3)]
            bmin,bmax=localbox(gb)
            ov=lambda k: min(amax[k],bmax[k])-max(amin[k],bmin[k])
            if ov(1)<=eps: continue
            face=None
            if abs(amax[0]-bmin[0])<eps and ov(2)>eps: face='left'
            elif abs(amin[0]-bmax[0])<eps and ov(2)>eps: face='right'
            elif abs(amax[2]-bmin[2])<eps and ov(0)>eps: face='back'
            elif abs(amin[2]-bmax[2])<eps and ov(0)>eps: face='front'
            if face: flips.append(('against.'+face,fa['text'],fa['gap'],ka+'/'+kb))
    print('==',os.path.basename(f),'facts',len(d['facts']),'predicted flips',len(flips))
    agg={}
    for fl in flips: agg.setdefault(fl[0],[]).append(fl[1])
    for k,v in agg.items(): print('   ',k,len(v),v[:6])
