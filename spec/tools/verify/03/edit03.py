import io,sys
p='/home/user/SVG3D/spec/03-solving.md'
s=io.open(p,encoding='utf-8',newline='').read()
assert '\r' not in s
def rep(old,new,count=1):
    global s
    assert s.count(old)==count, (s.count(old), old[:80])
    s=s.replace(old,new)

# A. 范围段：容差引用精确到 9.4.2 / 9.4.4
rep('两份独立实现按本章得到的 $R$、$t$ 与包围盒必须在 09-compat.md 规定的容差内一致。',
    '两份独立实现按本章得到的 $R$、$t$ 与包围盒必须在 09-compat.md 的 9.4.2 规定的容差内一致（`world.R` 的九个元素各在 $\\tau_{\\text{rot}}$ 内，`world.t` 与 `box` 的分量各在 $\\tau_{\\text{len}}$ 内）；由这些量经阈值判定得到的离散量只允许在 9.4.4 的边界对上不同（`late` 没有阈值，必须相同）。')

# B. 3.1.2 表
rep('| 影子（副本里对应原件子节点的节点，01-syntax.md 的 1.6.3） | $= T$（原件或其孪生体，04-derivation.md） | 3.9.7 |',
    '| 影子（副本里对应原件子节点的节点，01-syntax.md 的 1.6.3） | $= T$（原件，或有影子覆盖时原件的孪生体，04-derivation.md 的 4.4.4） | 3.9.7 |')
rep('| 变体皮肤 `skin` | 放到宿主包围盒的底面中心（07-semantics.md） | 3.7.6 |',
    '| 变体皮肤 `skin` | $(I,\\ b_N - \\pi_K)$：皮肤盒的底面中心放到宿主包络的底面中心（04-derivation.md 的 4.7 第 5 条） | 3.7.6 |')

# C. 3.3.2 / 3.3.4 / 3.3.5 指向 6.5.7
rep('- 继承 $B$ 的朝向：$B$ 倾斜，自己跟着倾斜，底面贴在 $B$ 的顶面上（3.5.5）。\n',
    '- 继承 $B$ 的朝向：$B$ 倾斜，自己跟着倾斜，底面贴在 $B$ 的顶面上（3.5.5）。\n- `on=B` 只决定**位置**，不产生空间事实；事实只看求解后的几何（06-facts.md 的 6.5.7）：`box a on=B` 在缺省 gap 下得到事实 `a on B`，`on=B gap=-0.1` 得到的是 `a intersects B`，`B` 是容器时事实落在容器里的实体上。\n')
rep('- 继承 $B$ 的朝向；$B$ 是镜像时按 3.5.7。\n- `gap` 为正时沿 $\\nu$ 离开面。\n',
    '- 继承 $B$ 的朝向；$B$ 是镜像时按 3.5.7。\n- `gap` 为正时沿 $\\nu$ 离开面。\n- `against` 只决定位置，不产生事实；事实按几何（06-facts.md 的 6.5.7）：两个轴对齐的盒类贴面时得到 `a against B.<面>`，$B$ 是圆柱时得到 `a touching B`。\n')
rep('- `inside` 只按盒定位，不检查是否真的在 $B$ 内，也不看 $B$ 的形状（`inside=crate.hollow` 把球心放到被减体的盒心；`align=bottom` 再把它落到底）。\n',
    '- `inside` 只按盒定位，不检查是否真的在 $B$ 内，也不看 $B$ 的形状（`inside=crate.hollow` 把球心放到被减体的盒心；`align=bottom` 再把它落到底）。是否真的得到事实 `a inside B` 由 06-facts.md 的 6.5.7 按几何判定（$B$ 单块、体积够大）；$B$ 是 group 时事实落在 group 里的实体上。\n')

# D. 3.6.1 表
rep('| $L(n)$，影子（3.9.7） | 原件（或孪生体）的 $L$ |',
    '| $L(n)$，影子（3.9.7） | 原件（或其孪生体，04-derivation.md 的 4.4.4）的 $L$ |')
rep('| $L(n)$，span（3.10.1） | `from`、`to` / `drop` 的点与法线、`dir` |',
    '| $L(n)$，变体皮肤（3.7.6） | 宿主 $N$ 的 $X(N)$ 与皮肤自己的 $X(K)$ |\n| $L(n)$，span（3.10.1） | `from`、`to` / `drop` 的点与法线、`dir` |')
rep('intersect：全部操作数）；有 `envelope` 的 group：无 |',
    'intersect：全部操作数）；段为 `skin` 的子节点一律不计（3.7.6）；有 `envelope` 的 group：无 |')

# E. 3.6.3 皮肤条
rep('- 变体皮肤 `skin` 永远后置（3.7.6）。\n',
    '- 变体皮肤 `skin` 不经本节判定：它不计入宿主的包络（3.6.1 的 $X$ 行没有到它的边），`late` 由 3.7.6 直接规定。\n')

# F. 3.6.4
rep('- 输出里每个节点带 `late`（布尔）：3.6.3 判为后置的为真，其余为假。',
    '- 输出里每个节点带 `late`（布尔）：3.6.3 判为后置的为真，其余为假；变体皮肤按 3.7.6，影子按 3.9.7（取原件的）。')

# G. 3.6.6
rep('- $T_{B \\to F}$ 必须按 3.1.1 的两条链计算，不得用 $W_F^{-1} \\circ W_B$ 代替（结果在容差内相同，但不逐位相同；09-compat.md 规定两份实现对比用容差，但同一实现在不同平台上要逐位一致）。',
    '- $T_{B \\to F}$ 必须按 3.1.1 的两条链计算，不得用 $W_F^{-1} \\circ W_B$ 代替：两种算法的结果在 09-compat.md 的 9.4.2 的容差内相同（语料里最大偏差 $10^{-12}$ 量级，3.11.1），但不逐位相同。09-compat.md 的 9.4 区分三种比较：两份独立实现互比、实现对参考输出（规范化后）比都用 9.4.2 的容差，而**同一实现**在不同平台上必须逐位一致（1.1.3）——所以同一实现不得在两种算法之间按平台或并行度切换。')

# H. 3.7.1
rep('容器的局部包围盒 $B_A$ 按 02-geometry.md 的 2.6（子节点盒之并，后置子节点不计，曲线计入）',
    '容器的局部包围盒 $B_A$ 按 02-geometry.md 的 2.6（子节点盒之并，后置子节点不计，段为 `skin` 的皮肤不计，曲线计入）')

# I. 3.7.6 重写
old_376 = s[s.index('### 3.7.6 变体皮肤'):s.index('## 3.8 repeat')]
new_376 = '''### 3.7.6 变体皮肤

变体（07-semantics.md 的 7.8）给被选中的宿主节点 $N$ 挂一个段为 `skin` 的子节点 $K$（模板实例，04-derivation.md 的 4.7 第 2 条）。$K$ 的局部变换由本节固定，不走 3.1.3 的一般规则，也不接受定位属性（`at`、`pin` 在 variant 上报错，01-syntax.md 的 1.3.13）：

$$R_K = I,\\qquad t_K = b_N - \\pi_K,\\qquad b_N = (c_{N,x},\\ \\min_{N,y},\\ c_{N,z}),$$

$b_N$ 是宿主 $N$ 的**包络**（3.7.1：实体的局部包围盒、容器的子节点之并、有 `envelope` 的 group 用 envelope；退化节点或空容器为零盒，$b_N = 0$）的底面中心，$\\pi_K$ 是 $K$ 自己的局部包围盒（体展开出的子节点之并，02-geometry.md 的 2.6）的底面中心。即皮肤的**局部包围盒底面中心**放到宿主包络的底面中心，轴向与 $N$ 相同（$K$ 是 $N$ 的子节点，所以跟随 $N$ 的世界朝向）；皮肤的尺寸由模板按自动传入的 `w` `h` `d`（$N$ 包络的边长，07-semantics.md 的 7.8.3 第 3 条）决定，不强制等于宿主。这等价于给皮肤语句写 `at=N.bottom pin=bottom`（原型 compileOnce 正是这样构造）；这条对祖先 $N$ 的引用由本节规定，不受 3.6.2 的限制。04-derivation.md 的 4.7 第 5 条是同一条公式。

**依赖。** $L(K)$ 依赖 $X(N)$ 与 $X(K)$（3.6.1）；但 $K$ **不计入** $N$ 与 $N$ 任何祖先的包络（02-geometry.md 的 2.6；04-derivation.md 的 4.7），所以 3.6.1 里没有 $X(N) \\to L(K)$ 的边，皮肤不会使依赖图成环，也不经 3.6.3 判定——变体不改变任何非皮肤节点的变换、包围盒与事实（07-semantics.md 的 7.8.4）。

**后置标记。** 皮肤节点 $K$ 的 `late` 由下面的可判定规则直接给出：**$N$ 的包络按 3.6.1 的 $X(N)$ 行要读到它的全部子节点时 $K$ 为 `late`，否则不为 `late`**。即：

| 宿主 $N$ | $K$ 的 `late` |
|---|---|
| 没有 `envelope` 的 group、union、intersect、repeat 节点、阵列实例、容器式 mirror / rotate 节点、模板实例、copy 节点 | 真 |
| 实体、曲线、有 `envelope` 的 group、subtract（只读第一个操作数）、退化节点 | 假 |

这个标记只是输出字段，不改变任何量：$K$ 的子树照常求解，$K$ 自己的子节点按 3.6.3 对 $K$ 判定。规则的来源：原型在求容器盒时遍历全部子节点、遇到读自己盒的子节点就标 `late`，皮肤的 `at=N.bottom` 正是这样的子节点，而实体、有 envelope 的 group 与 subtract（只遍历第一个操作数）求盒时不遍历到皮肤，于是不标。参考输出与此一致：semantics.s3d 的 `slab.skin`（宿主是 box）与 spaceport_blueprint.s3d 的 `hangar.roof.skin`（宿主是 box）`late` 为假，semantics.s3d 的 `street.h4.skin`（宿主是模板实例）`late` 为真；用原型编译的小例子（3.11.4）确认了表里其余各行。

参考输出的数值：`street.h4.skin` 的世界 $t = (43, 0, 8)$ = h4 的包络底面中心（h4 的门向前凸 0.2，使盒心 $z$ 比 h4 的原点多 0.1）；`slab.skin` 的 $t = (0, 5.05, 0)$ = slab 的底面中心；`hangar.roof.skin` 的 $t = (-45, 22, -20)$ = roof 的底面中心。

'''
s = s.replace(old_376, new_376)

# J. 3.9.7 重写
old_397 = s[s.index('### 3.9.7 影子'):s.index('## 3.10 span / sweep / pipe 的定位')]
new_397 = '''### 3.9.7 影子

副本树里的每个影子 $g$ 对应原件子树里的一个节点 $o$（04-derivation.md 的 4.4.2）。影子不接受定位属性；影子的子节点是 $o$ 的子节点的影子。

**局部变换。**
$$T_g = T_{\\tilde o},$$
$\\tilde o$ 是 $o$ 的**孪生体**（04-derivation.md 的 4.4.4）：父节点 = $o$ 的父节点，种类、源语句、参数绑定、子节点 = $o$ 的，有效属性 = $o$ 的有效属性被 $g$ 上的影子覆盖（路径经过副本段的几何时机 `over` 写的属性）逐键替换；$g$ 没有覆盖时 $\\tilde o = o$，于是 $T_g = T_o$。$T_{\\tilde o}$ 在 **$o$ 的父框架**里按 3.1.2 的表求（$o$ 是一般节点时即 3.1.3：关系、pin、gap、align、offset、朝向都按 $\\tilde o$ 的有效属性，参照物从 $o$ 所在处解析），$\\tilde o$ 不进场景树、不输出、不进任何集合。影子的局部盒同理取 $\\tilde o$ 的盒（覆盖了 `size` 的影子有自己的盒；$\\tilde o$ 是容器时按 $o$ 的子节点算）。

- 影子的世界变换 $W_g = W_{\\text{副本根}} \\circ T_{\\tilde o_1} \\circ \\cdots \\circ T_{\\tilde o_m}$（从副本根到 $g$ 的影子链各取其孪生体的 $T$），即原件的世界几何经副本变换映射（3.9.1、3.9.2）。
- 影子里不再求解关系：原件 `against=house.right` 在镜像里仍是"贴在镜像后的 house 的（镜像后的）右面"，因为整体一起映射；镜像后的楔形坡向翻转就是这个意思（curves.s3d 的 `gate`）。有覆盖时同理：`over tower2.legs.0.leg on=other` 里的 `other` 从 tower 所在处解析，孪生体在 tower 的框架里落到 other 上，再被镜像带到 tower2 那边。
- 依赖：$L(g)$ 依赖 $L(\\tilde o)$（3.6.1），$L(\\tilde o)$ 的依赖按 $\\tilde o$ 的属性与 3.6.1 的一般规则；孪生体不是任何容器的子节点，不参与 3.6.3 的后置判定。
- 影子的后置标记 `late` 取原件 $o$ 的（输出照抄）：副本的包络等于原件的包络（3.7.5），原件里不计入包络的子节点在副本里同样不计入。原型不复制这个标记（影子一律为假）；语料里没有后置节点被复制，参考输出不受影响（3.11.5）。

'''
s = s.replace(old_397, new_397)

# K. 3.11.1 容差注
rep('容差 $10^{-6}$（场景单位 / 无量纲；实际最大偏差在 $10^{-12}$ 量级，来自 $T_{B \\to P}$ 的两种计算顺序）。',
    '容差 $10^{-6}$（场景单位 / 无量纲；即 09-compat.md 的 9.4.2 的 $\\tau_{\\text{rot}}$ 与场景尺度 $L \\le 10^3$ 时的 $\\tau_{\\text{len}}$；实际最大偏差在 $10^{-12}$ 量级，来自 $T_{B \\to P}$ 的两种计算顺序）。')

# L. 3.11.3
rep('参考输出里的 `late` 节点恰好是：rotation.s3d 的 `radar.dish`、`radar.feed`、`radar.horn`（3.6.3 第一个例子）与 semantics.s3d 的 `street.h4.skin`（3.7.6）；其他 1640 个节点都不是。按 3.6.3 的图规则推导得到同一集合（dish 经 $T_{\\text{sat} \\to \\text{radar}}$ 可达 $L(\\text{radar}) \\to X(\\text{radar})$；feed、horn 经 $L(\\text{dish})$ 可达；skin 直接读 $X(\\text{h4})$）。',
    '参考输出里的 `late` 节点恰好是：rotation.s3d 的 `radar.dish`、`radar.feed`、`radar.horn`（3.6.3 第一个例子）与 semantics.s3d 的 `street.h4.skin`（3.7.6）；其他 1640 个节点都不是，其中包括另外两个皮肤 `slab.skin`（semantics）与 `hangar.roof.skin`（spaceport_blueprint）。按 3.6.3 的图规则推导得到前三个（dish 经 $T_{\\text{sat} \\to \\text{radar}}$ 可达 $L(\\text{radar}) \\to X(\\text{radar})$；feed、horn 经 $L(\\text{dish})$ 可达）；三个皮肤按 3.7.6 的规则：h4 是模板实例（包络读全部子节点）→ 真，slab 与 roof 是 box → 假。')

# M. 3.11.4 小例子表追加
rep('| `group g pin=origin at=… { box p at=g.origin }` | 不后置、不报错 | 报错（3.6.2：引用祖先） |\n',
    '| `group g pin=origin at=… { box p at=g.origin }` | 不后置、不报错 | 报错（3.6.2：引用祖先） |\n| `variant N use=cap`，$N$ 依次为 box、无 envelope 的 group、有 envelope 的 group、subtract、union、intersect、repeat 节点 | 皮肤 `late` 依次为假、真、假、假、真、真、真 | 相同（3.7.6 的表） |\n| `mirror radar2 of=radar …`，radar 里的 dish 后置 | `radar2.dish` 的 `late` 为假（标记不复制） | 真（3.9.7：取原件的） |\n')

# N. 3.11.5 追加影子 late 行
rep('| 3.3.8 | 多个定位关系 | `on` 优先、其余忽略（`relation`） | 报错（已在 3.3.8 列出） | 1.4.4 |\n',
    '| 3.3.8 | 多个定位关系 | `on` 优先、其余忽略（`relation`） | 报错（已在 3.3.8 列出） | 1.4.4 |\n| 3.9.7 | 后置节点被 mirror / rotate 复制时影子的 `late` | 假（标记不复制） | 取原件的（真） | 副本的包络等于原件的包络，影子同样不计入；只是输出字段，不改变任何量 |\n')

io.open(p,'w',encoding='utf-8',newline='').write(s)
print('ok')
