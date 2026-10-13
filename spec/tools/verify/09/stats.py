import json,glob,os,math,sys
root='/home/user/SVG3D/corpus/reference'
tot={'ex':[0,0],'xp':[0,0]}
for grp,pat in (('ex','examples/*.compile.json'),('xp','exports/*.compile.json')):
    for f in sorted(glob.glob(os.path.join(root,pat))):
        j=json.load(open(f))
        name=os.path.basename(f).replace('.compile.json','')
        nodes=j['nodes']; facts=j['facts']
        tot[grp][0]+=len(nodes); tot[grp][1]+=len(facts)
        combo=[n['path']+':'+n['kind'] for n in nodes if '→' in n['kind']]
        ghosts=[n for n in nodes if n.get('ghost')]
        ghost_group=[n['path'] for n in ghosts if n['kind']=='group']
        ghost_kinds={}
        for n in ghosts: ghost_kinds[n['kind']]=ghost_kinds.get(n['kind'],0)+1
        types_null=[n['path'] for n in nodes if n.get('types') is None and ('→' in n['kind'] or n.get('ghost'))]
        types_nonnull_ghost=[n['path']+':'+str(n.get('types')) for n in nodes if n.get('types') is not None and n.get('ghost')]
        file_null=[n['path'] for n in nodes if n.get('file') is None]
        skins=[(n['path'],n.get('file'),n.get('line'),n.get('late')) for n in nodes if n.get('skin')]
        bool_kinds={'union','subtract','intersect'}
        boolnodes=[n['path']+' tight='+str(n.get('tight')) for n in nodes if n['kind'] in bool_kinds]
        kindmap={n['path']:n['kind'] for n in nodes}
        bfacts=[f2 for f2 in facts if kindmap.get(f2['a']) in bool_kinds or kindmap.get(f2['b']) in bool_kinds]
        approx_present=sum(1 for f2 in facts if 'approx' in f2)
        approx_true=sum(1 for f2 in facts if f2.get('approx') is True)
        gapdec=max((len(str(f2['gap']).split('.')[1]) if isinstance(f2['gap'],float) and '.' in str(f2['gap']) else 0) for f2 in facts) if facts else 0
        on0=sum(1 for f2 in facts if f2['rel']=='on'); on_nz=sum(1 for f2 in facts if f2['rel']=='on' and f2['gap']!=0)
        eps=set(f2['eps'] for f2 in facts)
        # min margin | |gap| - eps | for touching/intersects
        marg=[abs(abs(f2['gap'])-f2['eps']) for f2 in facts if f2['rel'] in ('touching','intersects') and f2['gap'] is not None]
        mats_only_color=[i['path']+' '+json.dumps(i['material']) for i in j['items'] if set(i['material'].keys())=={'color'} and False]
        onlycolor=[i['path'] for i in j['items'] if 'name' not in i['material']]
        # material shapes
        shapes={}
        for i in j['items']:
            k=','.join(sorted(i['material'].keys())); shapes[k]=shapes.get(k,0)+1
        print(f"== {name}: nodes={len(nodes)} facts={len(facts)} items={len(j['items'])} curves={len(j.get('curves',[]))} errors={len(j['errors'])} warnings={j['warnings']} defs={j.get('defs')} factsSkipped={j.get('factsSkipped')} providers={j.get('factProviders')}")
        print("  combo kinds:",combo)
        print("  ghost kinds:",ghost_kinds, "ghost group:",len(ghost_group), ghost_group[:6])
        print("  types null (combo/ghost):",len(types_null)," ghost with types:",types_nonnull_ghost[:5])
        print("  file null:",file_null)
        print("  skins:",skins)
        print("  bool nodes:",boolnodes)
        print("  bool facts:",len(bfacts),[ (x['rel'],x['a'],x['b'],x.get('approx')) for x in bfacts])
        print("  approx present:",approx_present,"true:",approx_true,"gap max decimals:",gapdec,"on:",on0,"on nonzero gap:",on_nz,"eps:",eps)
        print("  min margin:",min(marg) if marg else None, "material shapes:",shapes, "items without name:",onlycolor)
        print("  top keys:",list(j.keys()), "meta" in j)
print(tot)
