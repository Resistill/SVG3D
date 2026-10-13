import json,glob,os
root='/home/user/SVG3D/corpus/reference/examples'
for f in sorted(glob.glob(root+'/*.compile.json')):
    j=json.load(open(f)); name=os.path.basename(f).split('.')[0]
    mx=0; dets=set(); sub=0; tot=0; eq=0; al=0
    for n in j['nodes']:
        if not n.get('world'): continue
        R=n['world']['R']
        for i in range(3):
            for k in range(3):
                s=sum(R[a][i]*R[a][k] for a in range(3))-(1 if i==k else 0)
                mx=max(mx,abs(s))
        d=(R[0][0]*(R[1][1]*R[2][2]-R[1][2]*R[2][1])-R[0][1]*(R[1][0]*R[2][2]-R[1][2]*R[2][0])+R[0][2]*(R[1][0]*R[2][1]-R[1][1]*R[2][0]))
        dets.add(round(d,6))
        if n.get('tight') and n.get('box'):
            tot+=1
            t=n['tight']; b=n['box']
            if all(t['min'][k]>=b['min'][k]-1e-9 and t['max'][k]<=b['max'][k]+1e-9 for k in range(3)): sub+=1
            perm=all(abs(x)<1e-9 or abs(abs(x)-1)<1e-9 for r in R for x in r)
            if perm:
                al+=1
                if t==b: eq+=1
    print(name,'maxRtR-I=%.2e'%mx,'dets',dets,'tight⊆box %d/%d'%(sub,tot),'aligned tight==box %d/%d'%(eq,al))
