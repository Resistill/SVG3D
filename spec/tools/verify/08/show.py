import json,sys
for f in sys.argv[1:]:
    d=json.load(open(f))
    print('=====',f)
    for n in d['nodes']: print(' ', n['path'], n['kind'], json.dumps(n['meta'],ensure_ascii=False), 'ghost' if n.get('ghost') else '')
    print('  errors:', [e.get('message') if isinstance(e,dict) else e for e in d['errors']])
    print('  warnings:', d['warnings'])
    for k in ('concepts','defs','materials','checks'):
        if d.get(k): print(' ',k, json.dumps(d[k],ensure_ascii=False)[:400])
    print('  top keys:', list(d.keys()))
