import json,glob,os,math
root='/home/user/SVG3D/corpus/reference'
for pat in ('examples/*.compile.json','exports/*.compile.json'):
    for f in sorted(glob.glob(os.path.join(root,pat))):
        j=json.load(open(f)); name=os.path.basename(f).replace('.compile.json','')
        nodes=j['nodes']; byp={n['path']:n for n in nodes}
        # L: diag of union of boxes of non-skin nodes
        mn=[1e18]*3; mx=[-1e18]*3; mt=0
        for n in nodes:
            if n.get('skin') or not n.get('box'): continue
            for k in range(3):
                mn[k]=min(mn[k],n['box']['min'][k]); mx[k]=max(mx[k],n['box']['max'][k])
                mt=max(mt,abs(n['world']['t'][k]),abs(n['box']['min'][k]),abs(n['box']['max'][k]))
        L=math.sqrt(sum((mx[k]-mn[k])**2 for k in range(3)))
        ghost_of_typed=[(n['path'],n['of'],byp[n['of']].get('types')) for n in nodes if n.get('ghost') and n.get('of') in byp and byp[n['of']].get('types')]
        late_ghost=[(n['path'],n['late'],byp.get(n['of'],{}).get('late')) for n in nodes if n.get('ghost') and n.get('of') in byp and (n['late'] or byp[n['of']]['late'])]
        late_nodes=[n['path'] for n in nodes if n['late']]
        print(name,'L=%.1f maxabs=%.1f'%(L,mt),'ghost-of-typed:',ghost_of_typed[:5],len(ghost_of_typed),'late:',late_nodes,'late-ghost:',late_ghost[:5])
        if name in('spaceport_blueprint','spaceport_edit'):
            print('  materials',{k:v for k,v in j['materials'].items() if k in('concrete','steel')})
            print('  items only color',[ (i['path'],i['material']) for i in j['items'] if 'name' not in i['material']])
