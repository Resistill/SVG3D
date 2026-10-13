# 2.8.1 的核对脚本：用参考输出的 items[].geom + world 按 02-geometry.md 2.1.8 / 2.5.4 重算 tight，与 nodes[].tight 比较。
# 用法：python3 -I tight2.py  （在仓库根目录跑）
import json,glob,math,collections
def mat(W,p):
    R=W['R'];t=W['t']; return [sum(R[i][j]*p[j] for j in range(3))+t[i] for i in range(3)]
def mdir(W,p):
    R=W['R']; return [sum(R[i][j]*p[j] for j in range(3)) for i in range(3)]
def axis_aligned(W):
    R=W['R']; return all(abs(abs(R[i][j])-round(abs(R[i][j])))<1e-9 for i in range(3) for j in range(3))
def ours(it):
    g=it['geom']; W=it['world']; typ=g['type']; lo=[1e18]*3; hi=[-1e18]*3
    def addp(p):
        for k in range(3): lo[k]=min(lo[k],p[k]); hi[k]=max(hi[k],p[k])
    def circle(c,r,a):
        cw=mat(W,c); aw=mdir(W,a)
        for k in range(3):
            e=r*math.sqrt(max(0,1-aw[k]**2)); lo[k]=min(lo[k],cw[k]-e); hi[k]=max(hi[k],cw[k]+e)
    def boxpts(mn,mx):
        for x in (mn[0],mx[0]):
            for y in (mn[1],mx[1]):
                for z in (mn[2],mx[2]): addp(mat(W,[x,y,z]))
    if typ in('box','mesh'): w,h,d=g['w'],g['h'],g['d']; boxpts([-w/2,0,-d/2],[w/2,h,d/2])
    elif typ=='wedge':
        w,h,d=g['w'],g['h'],g['d']
        for p in [[-w/2,0,-d/2],[w/2,0,-d/2],[w/2,h,-d/2],[-w/2,0,d/2],[w/2,0,d/2],[w/2,h,d/2]]: addp(mat(W,p))
    elif typ=='beam': L,w,t,dy=g['L'],g['w'],g['t'],g['dy']; boxpts([-L/2,dy-t/2,-w/2],[L/2,dy+t/2,w/2])
    elif typ=='steps':
        for tr in g['treads']:
            c,s=tr['c'],tr['s']; boxpts([c[k]-s[k]/2 for k in range(3)],[c[k]+s[k]/2 for k in range(3)])
    elif typ=='sphere':
        c=mat(W,[0,g['r'],0])
        for k in range(3): lo[k]=c[k]-g['r']; hi[k]=c[k]+g['r']
    elif typ=='cyl':
        R=max(g['r1'],g['r2']); h=g['h']
        c0,a={'y':([0,0,0],[0,1,0]),'x':([-h/2,R,0],[1,0,0]),'z':([0,R,-h/2],[0,0,1])}[g['axis']]
        circle(c0,g['r1'],a); circle([c0[i]+h*a[i] for i in range(3)],g['r2'],a)
    elif typ=='rod': L=g['L'];r=g['r']; circle([-L/2,0,0],r,[1,0,0]); circle([L/2,0,0],r,[1,0,0])
    elif typ=='sweep':
        for fr in g['frames']:
            for (u,v) in g['prof']: addp(mat(W,[fr['p'][k]+u*fr['side'][k]+v*fr['up'][k] for k in range(3)]))
    elif typ=='revolve':
        ang=g.get('angle',360)
        for (r,y) in g['prof']:
            r=abs(r)
            if ang>=360-1e-9: circle([0,y,0],r,[0,1,0])
            else:
                for j in range(2001):
                    th=math.radians(ang*j/2000); addp(mat(W,[r*math.cos(th),y,-r*math.sin(th)]))
    elif typ=='hull':
        for p in g['pts']: addp(mat(W,p))
    elif typ=='plate':
        for (x,z) in g['pts']: addp(mat(W,[x,0,z])); addp(mat(W,[x,g['h'],z]))
    else: return None
    return lo,hi
def comb(it):
    if it['kind']=='solid': return ours(it)
    ch=[c for c in (comb(c) for c in it['children']) if c]
    if not ch: return None
    if it['op']=='subtract': return ch[0]
    if it['op']=='union':
        return [min(l[k] for l,h in ch) for k in range(3)],[max(h[k] for l,h in ch) for k in range(3)]
    return [max(l[k] for l,h in ch) for k in range(3)],[min(h[k] for l,h in ch) for k in range(3)]
stats=collections.OrderedDict(); details=[]
for f in sorted(glob.glob('corpus/reference/examples/*.compile.json')):
    d=json.load(open(f)); fn=f.split('/')[-1].replace('.compile.json','')
    nodes={n['path']:n for n in d['nodes']}
    def walk(it):
        yield it
        if it['kind']!='solid':
            for c in it['children']: yield from walk(c)
    for top in d['items']:
        for it in walk(top):
            n=nodes.get(it['path']); typ=it['geom']['type'] if it['kind']=='solid' else 'csg:'+it['op']
            if not n: continue
            if not n.get('tight'):
                if it['kind']!='solid': details.append((fn,it['path'],typ,'reference tight is null; box',n['box']))
                continue
            r=comb(it)
            if not r: continue
            lo,hi=r; T=n['tight']
            dev=max(max(abs(lo[k]-T['min'][k]),abs(hi[k]-T['max'][k])) for k in range(3))
            aa=axis_aligned(it['world']) if it['kind']=='solid' else None
            s=stats.setdefault(typ,[0,0,0.0,0,0]); s[0]+=1
            if dev<=1e-9: s[1]+=1
            s[2]=max(s[2],dev)
            if aa is False: s[3]+=1
            if aa is False and dev<=1e-9: s[4]+=1
            if dev>1e-9 and typ not in('cyl','rod'): details.append((fn,it['path'],typ,round(dev,6),aa,n.get('late')))
    print(fn,'curves',collections.Counter((nodes[c['path']]['kind'],len(c['pts']),c['closed']) for c in d['curves']))
print('type: total, equal(1e-9), maxdev, non-axis-aligned, non-aa equal')
for k,v in stats.items(): print(k,v)
for x in details: print(x)
