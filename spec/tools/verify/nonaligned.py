import json,glob,os,collections
def isperm(R):
    # R is 3x3 nested list? check
    rows=R if isinstance(R[0],list) else [R[0:3],R[3:6],R[6:9]]
    for r in rows:
        if sum(1 for v in r if abs(abs(v)-1)<1e-9)!=1 or sum(1 for v in r if abs(v)>1e-9)!=1: return False
    return True
tot=collections.Counter()
for f in sorted(glob.glob('/home/user/SVG3D/corpus/reference/examples/*.compile.json')):
    d=json.load(open(f)); name=os.path.basename(f).split('.')[0]
    geom={it['path']:it.get('geom',{}).get('type') for it in d['items'] if it.get('kind')=='solid'}
    def walk(it):
        for c in it.get('children',[]):
            if c.get('kind')=='solid': geom[c['path']]=c.get('geom',{}).get('type')
            walk(c)
    for it in d['items']: walk(it)
    rows=collections.defaultdict(list)
    for n in d['nodes']:
        k=n['kind'].split('→')[-1]
        g=geom.get(n['path'])
        if k in('cyl','cone') or g=='rod':
            R=n['world']['R']
            if not isperm(R):
                rows[(n['file'],n['line'],k if k!='span' else 'rod')].append(n['path'])
                tot[k if k!='span' else 'rod']+=1
    print(name)
    for key,v in sorted(rows.items(),key=lambda x:(x[0][0],x[0][1])):
        print('  ',key,len(v),v[:3])
    # csg tight null
    print('  csg nodes:',[ (n['path'],n['tight']) for n in d['nodes'] if n['kind'] in('union','subtract','intersect')])
print(tot)
