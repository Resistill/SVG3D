import sys
p='/home/user/SVG3D/spec/09-compat.md'
s=open(p,encoding='utf-8').read()
R=[('| errors 0、warnings 1；18 个节点的 `world` / `box` / `tight` 与 20 条事实的 (rel, a, b, gap) 不变 |',
    '| errors 0、warnings 1；18 个节点的 `world` / `box` 与 20 条事实的 (rel, a, b, gap) 不变，`tight` 除 `crate` 补算外不变 |'),
   ('全部 1644 + 2408 个节点的 `world` / `box` / `tight` 与全部事实的 `gap` 预期在容差内一致（3.11、6.8）。',
    '全部 1644 + 2408 个节点的 `world` / `box` 与全部事实的 `gap` 预期在容差内一致（3.11、6.8），`tight` 按 9.4.3 的对参考规则一致（2.8）。')]
for o,n in R:
    c=s.count(o)
    if c!=1: print('COUNT',c,o[:60]); sys.exit(1)
    s=s.replace(o,n)
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
