import sys
p='/home/user/SVG3D/spec/02-geometry.md'
s=open(p,encoding='utf-8').read()
def rep(a,b):
    global s
    assert s.count(a)==1,(s.count(a),a[:60])
    s=s.replace(a,b)
# 1. 2.1.8 末段
rep("对参考比较 `tight` 时的额外容许量由 09-compat.md 的 9.4 规定。",
    "对参考比较 `tight` 时的额外容许量与规范化由 09-compat.md 规定：9.4.2 的 $\\delta_{\\text{tight}}$ / $\\delta_{\\text{out}}$、9.4.3 的 `tight` 行与 9.4.5 的 N15。")
# 2. 2.8.3 开头
rep("（第 9 章汇总；比较 `tight` 的额外容许量与规范化由 9.4 规定）：",
    "（第 9 章汇总；比较 `tight` 的额外容许量由 09-compat.md 的 9.4.2（$\\delta_{\\text{tight}}$ / $\\delta_{\\text{out}}$）与 9.4.3 的 `tight` 行规定，规范化由 9.4.5 的 N15 规定）：")
# 3. 2.8.1 计数口径
rep("与参考的 `nodes[].tight` 逐分量比（容差 $10^{-9}$）；",
    "与参考的 `nodes[].tight` 逐分量比（容差 $10^{-9}$）；节点按 7 个文件逐个计数、不去重（spaceport_blueprint / spaceport_edit 里与 spaceport 相同的节点各算一次），\"轴对齐\"指 `world.R` 是带符号置换矩阵（坐标轴的置换与反射，每行每列恰一个 $\\pm 1$）；")
# 4. 2.8.2 cyl / rod 行
rep("| cyl（含 cone） | 262 | 175 | $0.00613$ | 87 个非轴对齐节点不同：",
    "| cyl（含 cone） | 262 | 175 | $0.00613$ | 135 个非轴对齐节点（132 个 cyl、3 个 cone）里 87 个不同：")
rep("| rod | 73 | 52 | $0.00551$ | 21 个节点不同，同上（最大 $0.025\\,R$，$R = 0.22$） |",
    "| rod | 73 | 52 | $0.00551$ | 73 个节点都非轴对齐，21 个不同，同上（最大 $0.025\\,R$，$R = 0.22$）；52 个的轴向极值恰落在棱柱顶点上、相同 |")
# 5. 2.8.3 表行
rep("| 87 个 cyl、21 个 rod；参考比本版小，每面偏差 $\\le R(1 - \\cos 15°)$；cone 顶点处参考另可大 $\\le 10^{-4}$（小环） |",
    "| 按 2.8.1 的口径（7 个文件不去重、`world.R` 不是带符号置换矩阵）共 135 个非轴对齐 cyl / cone（132 + 3）与 73 个 rod；其中参考 `tight` 与本版 $T$ **实际不同**（偏差 $> 10^{-9}$）的是 87 个 cyl / cone、21 个 rod（2.8.2），其余 48 + 52 个的轴向极值恰落在棱柱顶点上、两版相同；参考比本版小，每面偏差 $\\le R(1 - \\cos 15°)$；cone 顶点处参考另可大 $\\le 10^{-4}$（小环） |")
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
