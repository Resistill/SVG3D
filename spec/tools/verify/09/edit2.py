import sys
p='/home/user/SVG3D/spec/09-compat.md'
s=open(p,encoding='utf-8').read()
R=[]
def rep(o,n): R.append((o,n))
rep('行为分五类：\n','行为分六类：\n')
rep('| E 保留 | 明确保留的原型行为（写出来是为了不被"看起来像 bug"而改掉） |\n',
'| E 保留 | 明确保留的原型行为（写出来是为了不被"看起来像 bug"而改掉） |\n| F 警告 | 原型没有、本版新增的警告；不影响结果（1.1.1） |\n')
rep('只含 `color` 的材质条目、一方是布尔容器的事实、概念成员的顺序、`errors` / `warnings`。',
'只含 `color` 的材质条目、一方是布尔容器的事实、概念成员的顺序、`tight` 为 `null` 的布尔容器与 $R$ 不是带符号置换的 revolve、`defs` 数组、`errors` / `warnings`。')
rep('| 6.3.2 | 请求不存在的提供者报错并用缺省列表；提供者出错只丢它的事实 |',
'| 6.3.2 | 请求不存在的提供者报错并用缺省列表（`compileOnce` 相同）；提供者出错只丢它的事实（`deriveFacts` 的 `onError` 相同） |')
for o,n in R:
    c=s.count(o)
    if c!=1: print('COUNT',c,o[:60]); sys.exit(1)
    s=s.replace(o,n)
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
