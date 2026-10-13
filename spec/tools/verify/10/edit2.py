import sys
p='/home/user/SVG3D/spec/10-materials.md'
s=open(p,encoding='utf8').read()
edits=[
("指它在几何时机的修改全部应用之后（04-derivation.md）所带的", "指它在几何时机的修改全部应用之后（4.1.1 的阶段 5–6、4.5）所带的"),
("（或与之等价的只改外观的 `over … mat=… color=…`，07-semantics.md）在求解与语义层之后被求值，得到它的选择集 $S$（07-semantics.md）", "（或与之等价的只改外观的 `over … mat=… color=…`，7.7.4）在求解与语义层之后被求值（7.1.3 第 3 步），得到它的选择集 $S$（7.7.2）"),
("副本跟随原件的外观，就像它跟随原件的几何（04-derivation.md）：", "副本跟随原件的外观，就像它跟随原件的几何（4.4.2）："),
]
for old,new in edits:
    n=s.count(old)
    if n!=1: print("COUNT",n,old[:50]); sys.exit(1)
    s=s.replace(old,new)
open(p,'w',encoding='utf8',newline='\n').write(s); print("ok")
