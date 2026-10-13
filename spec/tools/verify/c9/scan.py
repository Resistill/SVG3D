import re,sys,glob,os
KINDS=set("box wedge plate hull mesh cyl cone sphere revolve span sweep pipe line bezier spline arc helix group union subtract intersect repeat mirror rotate copy".split())
SOFT=set("box wedge plate hull mesh cyl cone sphere revolve span sweep pipe line bezier spline arc helix group union subtract intersect repeat mirror rotate copy def let import material over drop concept pattern check style variant top up bottom down left west right east front south back north center w h d x y z start mid end side surface origin all count children descendants sin cos tan asin acos atan atan2 sqrt abs min max floor ceil round pow metre on supports against inside contains touching intersects".split())
REL=["at","on","under","against","inside"]
files=sorted(glob.glob('corpus/examples/*.s3d'))+['corpus/examples/lib/semantics.s3d']+sorted(glob.glob('corpus/exports/*.s3d'))
def strip(s):
    out='';inq=False
    for i,c in enumerate(s):
        if c=='"': inq=not inq
        if not inq and s[i:i+2]=='//': break
        out+=c
    return out
def props(s):
    # top-level key=value tokens, respecting parens/brackets
    toks=[];cur='';depth=0;inq=False
    for c in s:
        if c=='"': inq=not inq
        if not inq:
            if c in '([': depth+=1
            if c in ')]': depth-=1
            if c in ' \t' and depth==0:
                if cur: toks.append(cur); cur=''
                continue
        cur+=c
    if cur: toks.append(cur)
    return toks
for f in files:
    src=open(f,encoding='utf-8').read().split('\n')
    stack=[{}]  # sibling name sets per block
    defs=[];imports=0;hits=[]
    for ln,raw in enumerate(src,1):
        s=strip(raw).strip()
        if not s: continue
        opens=s.count('{');closes=s.count('}')
        body=s.rstrip('{').strip() if s.endswith('{') else s
        if body=='}' or body=='':
            for _ in range(closes): stack.pop()
            continue
        toks=props(body)
        kind=toks[0]
        depth=len(stack)-1
        if kind=='material' and depth>0: hits.append((ln,'块内 material'))
        if kind=='import': imports+=1
        if kind=='def':
            m=re.match(r'def\s+([\w]+)',body); name=m.group(1); defs.append(name)
            if name in KINDS: hits.append((ln,'模板与种类同名 '+name))
            pm=re.search(r'\((.*)\)',body)
            if pm:
                for p in pm.group(1).split(','):
                    pn=p.split('=')[0].strip()
                    if pn in ('mat','color'): hits.append((ln,'参数名 '+pn))
        if kind in KINDS or kind in defs or kind.startswith('~'):
            # name
            name=None
            for t in toks[1:]:
                if '=' in t or t.startswith('#') or t.startswith('~'): continue
                name=t;break
            if name:
                if name in stack[-1]: hits.append((ln,'同级重名 '+name))
                stack[-1][name]=ln
                if name in SOFT: hits.append((ln,'软保留名 '+name))
            kv={t.split('=',1)[0]:t.split('=',1)[1] for t in toks[1:] if '=' in t and not t.startswith('~')}
            meta={t.split('=',1)[0]:t.split('=',1)[1] for t in toks[1:] if t.startswith('~')}
            for k,v in meta.items():
                if re.match(r'^[A-Za-z_]',v) and v not in ('true','false'): hits.append((ln,'元数据值是名字 '+k+'='+v))
            rels=[k for k in REL if k in kv]
            if len(rels)>1: hits.append((ln,'多个定位关系 '+','.join(rels)))
            if 'gap' in kv:
                ok = any(k in kv for k in ('on','under')) or ('against' in kv) or ('at' in kv and re.search(r'\.(top|bottom|left|right|front|back)',kv['at']))
                if not ok: hits.append((ln,'gap 无法线 '+str(rels)))
            if 'rot' in kv and re.match(r'^\(?\s*0\s*,\s*0\s*,\s*0\s*,',kv['rot']): hits.append((ln,'rot 零轴'))
            if kind=='repeat':
                n=kv.get('n','')
                if re.match(r'^-?[\d.()]+$',n.replace(',','')) and re.search(r'\d\.\d',n): hits.append((ln,'n 非整数 '+n))
                if n.startswith('-'): hits.append((ln,'n 负'))
                if 'along' in kv and ('step' in kv or 'center' in kv): hits.append((ln,'along 与 step/center'))
                if 'orient' in kv and 'along' not in kv: hits.append((ln,'orient 无 along'))
                if kv.get('orient') in ('true','false'): hits.append((ln,'orient=true/false'))
            if kind in ('mirror','rotate'):
                if 'of' in kv:
                    bad=[k for k in kv if k in ('at','on','under','against','inside','pin','gap','offset','align','alignto','turn','tilt','roll','rot','aim','aimwith','n')]
                    if bad: hits.append((ln,'副本式带定位/n '+','.join(bad)))
                if kind=='mirror' and kv.get('across') in ('x','y','z') and 'axis' in kv: hits.append((ln,'across 轴名 + axis'))
                if kind=='rotate' and 'n' in kv and re.search(r'\d\.\d',kv['n']): hits.append((ln,'rotate n 非整数'))
            if 'aimwith' in kv and kv['aimwith'] not in SOFT: hits.append((ln,'aimwith 非方向词'))
            if kind=='copy' and ln!=len(src):
                pass
        if kind=='over':
            rest=toks[1:]
            if not any('=' in t for t in rest) and not s.endswith('{') and ':' not in body: hits.append((ln,'over 无属性无块'))
            if ':' in body: hits.append((ln,'换形'))
            if re.search(r'\b(w|h|d|x|y|z|top|bottom)\s*[<>=]',body) or re.search(r'\b(on|supports|against|inside|contains|touching|intersects)\(',body): hits.append((ln,'几何时机选择器用了求解量?'))
        if kind=='concept':
            m=re.match(r'concept\s+(\w+)\s*:\s*(\w+)',body)
            if m: hits.append((ln,'concept : 上位 '+m.group(2)))
        if kind=='variant' and ('at=' in body or 'pin=' in body): hits.append((ln,'variant at/pin'))
        if kind=='check':
            expr=re.sub(r'^check\s+"[^"]*"\s*','',body)
            if not re.search(r'[<>=!]=?|\bwhere\b|\bnot\b|\band\b|\bor\b| - |kind\(|^\w+$',expr): hits.append((ln,'check 表达式类型? '+expr))
        m=re.findall(r'\b(on|supports|against|inside|contains|touching|intersects)\(([^()]*)\)',body)
        for fn,args in m:
            parts=args.split(',')
            if len(parts)>1:
                k=parts[-1].strip()
                if not re.match(r'^\d+$',k): hits.append((ln,'关系谓词第二参数 '+k))
        if kind=='pattern' or (depth>0 and ' in ' in body and stack and False): pass
        if re.search(r'\w+\s+intersects\s+\w+',body) and kind not in ('concept','check','style','over') and '(' not in body: hits.append((ln,'pattern intersects'))
        for _ in range(opens): stack.append({})
        for _ in range(closes): stack.pop()
    if imports>1: hits.append((0,'多个 import'))
    print('==',f, 'defs',defs, 'imports',imports)
    for h in hits: print('   ',h)
