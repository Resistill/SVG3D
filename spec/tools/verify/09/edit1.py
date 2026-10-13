import sys
p='/home/user/SVG3D/spec/09-compat.md'
s=open(p,encoding='utf-8').read()
R=[]
def rep(old,new):
    R.append((old,new))

rep('**依赖的章。** 01 到 08、10 的全部"原型 …；本版 …"陈述与各章末尾"因本章定义而预期不同的项"小节。被依赖：03-solving.md（3.1 节与 3.6.6 引用本章的容差）、10-materials.md（10.6 引用本章的对拍规则）、05-layers.md（5.2.5 的 `file` 字段清单）。',
'**依赖的章。** 01 到 08、10 的全部"原型 …；本版 …"陈述与各章末尾"因本章定义而预期不同的项"小节（2.8.3、3.11.5、4.8.5、5.5.4、6.8.5、7.9.4、8.6.2、10.6）。被依赖：01-syntax.md（1.6.9、1.10.3 引用 9.3.4 的警告汇总）、02-geometry.md（2.1.8、2.8.3 引用 9.4 对 `tight` 的额外容许量与规范化）、03-solving.md（3.1 节与 3.6.6 引用本章的容差）、04-derivation.md（4.2.7 的 `defs[]` 条目按 9.4.3 比）、05-layers.md（5.2.5 引用 9.3.3 的 `file` 字段清单与 9.4.5 的 N1、N5、N6）、06-facts.md（6.6.1、6.8.5 引用 9.4.3 与 9.5.3）、08-metadata.md（8.4.3、8.6.2 引用 9.4.5 的 N12）、10-materials.md（10.6 引用本章的对拍规则）。')

rep('并经读源码或在原型上编译小例子确认（scratchpad 的 `c9/*.s3d`）。',
'并经读源码或在原型上编译小例子确认（scratchpad 的 `c9/*.s3d`；各章自己确认过的小例子同样是依据：03-solving.md 的 3.11.4，07-semantics.md 的 7.9.4 所用的 `e7/a.s3d`、`b.s3d`、`c.s3d`——第 7 章的差异里只有集合成员顺序一项在语料里实际出现，其余都是用这些小例子在原型上确认的边界情形）。')

rep('文件头没写时补上；参考输出没有这个字段（9.4.5 的 N11）。','文件头没写时补上；参考输出没有这个字段（9.4.5 的 N12）。')

# 9.2.4 new row
rep('| 1.5.6 | `kind("box")`（字符串参数） | 只接受裸标识符 | 接受名字字面量或字符串 |\n',
'| 1.5.6 | `kind("box")`（字符串参数） | 只接受裸标识符 | 接受名字字面量或字符串 |\n| 7.8.3 | variant 写明的参数值里引用概念名 / 模式名 | 参数值在全局帧求值（`compileOnce` 的 `new Env(elab.globals)`），概念名是找不到的名字 | 在语义帧求值，概念名、模式名可见；全局帧里能写的表达式值相同 |\n')

# 9.3.1 结构 table: lang row after 块内 import
rep('| 5.2.1 | 块内的 import | 报"未知的类型 import" | 报"import 只能写在最外层"（位置错误） | 1.3.15 | 无 |\n',
'| 5.2.1 | 块内的 import | 报"未知的类型 import" | 报"import 只能写在最外层"（位置错误） | 1.3.15 | 无 |\n| 5.1.4、9.1.3 | 被 import 的文件带 `~lang=`，值不是 2 | `loadProgram` 不检查 `lang`（文件头本身在原型里报"未知的类型"，9.2.1） | 报错"不支持的语言版本"，只有该文件不编译、仍算已加载；按文件独立判定 | 8.1.5 | 无（语料没有文件头） |\n')

# 派生 table function names
rep('| 4.6.2 | 模板 → 模板换形，参数不在目标参数表 | 保留全部参数，重新展开时报"模板 X 没有参数 k"且节点退化重建 |',
'| 4.6.2 | 模板 → 模板换形，参数不在目标参数表 | 保留全部参数，重新展开时报"模板 X 没有参数 k"且节点退化重建（`applyOpinions`） |')
rep('| 4.4.4 | 对影子写 `repeat` 的 `n` | 不拦截（孪生体与原件共享子节点，结果不自洽） |',
'| 4.4.4 | 对影子写 `repeat` 的 `n` | 不拦截（`applyOpinions` 对影子只拒绝换形与模板参数；孪生体与原件共享子节点，结果不自洽） |')
rep('| 4.4.6 | `of=` 不是单个对象路径 | 各种失败 | 报错"of 需要写一个对象名" | 1.4.5 | 无 |',
'| 4.4.6 | `of=` 不是单个对象路径 | 按值的种类各有不同的失败（`buildCopies` 直接对 `of` 的表达式求引用） | 报错"of 需要写一个对象名" | 1.4.5 | 无 |')
rep('| 4.2.5 | 派生签名里同一参数既在覆盖表又重新声明 | 先施加覆盖再施加参数表，自己的声明静默赢 |',
'| 4.2.5 | 派生签名里同一参数既在覆盖表又重新声明 | `elaborate` 登记时不检查；实例化时先施加覆盖再施加参数表，自己的声明静默赢 |')

# 层 table: empty selection row (R2)
rep('| 5.4.2、1.7.1 | 外观时机的 `over` 选择集为空 | `compileOnce` 的样式 / 变体循环不报（`over #nothing color=#00ff00` 静默） | 报错"X 没有选中任何对象"，语句无效；`style` / `variant` 仍不报（这是外观时机 over 与 style 的唯一差别） | 1.7.1 | 无 |',
'| 1.7.1、5.4.2 第 7 条、7.7.4 第 1 条 | 外观时机的 `over`（登记为 style / variant 的）选择集为空 | `compileOnce` 的样式 / 变体循环不报（`over #nothing color=#00ff00` 静默） | 报错"X 没有选中任何对象"，语句无效。over / drop 在**两个时机**都报这条错误：几何时机的原型 `applyOpinions` 第二轮已报同一文案（相同，9.3.5）；`style` / `variant` 在任何情形都不报（这是外观时机 over 与 style 的唯一差别） | 1.7.1 按语句种类规定 | 无 |')

# 9.3.2 tight rows after 2.1.6.2 row
rep('| 2.1.6.2（经 6.8.3） | 凹轮廓 plate 的三角化 | `triangulate2` | 耳切（2.1.6.2），可能切出不同的三角形 | 定义唯一的构造 | 无（语料的事实不涉及 plate） |\n',
'| 2.1.6.2（经 6.8.3） | 凹轮廓 plate 的三角化 | `triangulate2` | 耳切（2.1.6.2），可能切出不同的三角形 | 定义唯一的构造 | 无（语料的事实不涉及 plate） |\n'
'| 2.1.8、2.2.3、2.2.8 | 非轴对齐 cyl / cone / rod 的 `tight` | `tightBox`：凸块（12 边内接棱柱）顶点经世界变换的盒 | 两端面圆的轴向极值（真实几何） | 2.1.8 已定：紧包围盒按真实形状算 | **命中** 87 个 cyl、21 个 rod（2.8.2）：参考比本版小，每面偏差 $\\le R(1 - \\cos 15°)$，最大 0.00613；cone 顶点处参考另可大 $\\le 10^{-4}$（小环）——对参考用 9.4.2 的 $\\delta_{\\text{tight}}$ |\n'
'| 2.1.8、2.2.9 | 圆截面 sweep / pipe 的 `tight` | `tightBox`：每环只取 20 个环点里的 8 个 | 全部 20 个环点 | 同上 | **命中** 2 个（spaceport、spaceport_blueprint 的 `feed1`），偏差 $4.3 \\times 10^{-6}$；一般 $\\le r(1 - \\cos 27°)$，参考 ⊆ 本版——$\\delta_{\\text{tight}}$ |\n'
'| 2.1.8、2.2.7 | 非轴对齐或 `angle` $< 360$ 的 revolve 的 `tight` | `tightBox`：$R_{\\max} \\times [y_{\\min}, y_{\\max}]$ 的外接 12 边棱柱的盒，忽略轮廓形状与 `angle` | 轮廓各点的圆 / 弧极值 | 同上 | **命中** 1 个（rotation 的 `radar.dish`，参考大 1.94）——规范化 N15 |\n'
'| 2.5.4 | 布尔容器的 `tight` | `null`（`tightBox` 只对有 `geom` 的叶子算） | 操作数的 $T_i$ 按运算组合 | 布尔容器是唯一有紧包围盒的容器 | **命中** 5 个（basics 的 `crate`、spaceport 系三个文件的 `hangar.body`、spaceport_edit 的 `hangar2.body`）——规范化 N15 |\n')

# 9.3.3 N refs
rep('| 6.6.1 | `facts[].approx` | 只在为真时出现 | 总是输出布尔 | 形状固定 | 全部事实（规范化 N6） |',
'| 6.6.1 | `facts[].approx` | 只在为真时出现 | 总是输出布尔 | 形状固定 | 全部事实（规范化 N7） |')
rep('| 各带 `meta`（没有元数据时 `{}`） | 8.4.3 | 全部条目（规范化 N10、N11） |','| 各带 `meta`（没有元数据时 `{}`） | 8.4.3 | 全部条目（规范化 N12） |')
rep('| 8.4.3 | 编译输出顶层 | 没有 `meta` | `meta: { lang: 2, … }` 总是存在 | 8.4.3 | 全部文件（规范化 N11） |',
'| 8.4.3 | 编译输出顶层 | 没有 `meta` | `meta: { lang: 2, … }` 总是存在（原型没有顶层 `meta`） | 8.4.3 | 全部文件（规范化 N12） |')
rep('统一五键 `{name, color, metal, rough, opacity}`，无名材质 `name: ""` | 形状固定 | 全部条目（规范化 N9） |',
'统一五键 `{name, color, metal, rough, opacity}`，无名材质 `name: ""` | 形状固定 | 全部条目（规范化 N11） |')
rep('| 10.6、10.1.3 | `materials` | 只输出写了的键 | 五个分量与 `default` 总是都有 | 形状固定 | 全部文件（规范化 N10） |\n',
'| 10.6、10.1.3 | `materials` | 只输出写了的键 | 五个分量与 `default` 总是都有 | 形状固定 | 全部文件（规范化 N11） |\n'
'| 4.2.7、8.4.3 | `defs` | 模板名的字符串数组 `[...defs.keys()]`（`compile`） | 对象条目 `{name, line, file, params: [{name, required, default}], base, meta}` 的数组，`name` 的序列与原型数组相同 | 4.2.7；8.4.3 要求每个条目带 `meta` | **命中** 有 def 的 5 个文件：semantics（4 个模板）、spaceport 与 spaceport_edit（2 个）、spaceport_blueprint（3 个）、rotation（1 个）；规范化 N16，对参考只比 `name` 序列 |\n'
'| 3.9.7、4.4.5 | 影子的 `late` | `buildCopies` 的 `shadow` 不复制标记，影子一律 `false` | 照抄原件的 | 副本的包络等于原件的包络 | 无（语料里没有后置节点被复制，3.11.5） |\n'
'| 2.1.11、2.2.5 | plate 条目的 `geom.pts` | 未规整的输入点（`buildItems`） | 规整后的轮廓 | 2.2.5 的规整是定义的一部分 | 无（语料没有 plate） |\n'
'| 2.1.11、2.2.6 | hull 条目的 `geom.faces` | `convexHull3` 增量算法得到的三角形 | 面片按 2.1.9 的扇形规则三角化 | 定义唯一的构造 | 无（语料没有 hull） |\n'
'| 2.5.5、2.1.11 | 布尔容器里的曲线 | `buildItems` 不进入布尔容器收集曲线，没有 `curves[]` 条目 | 有条目 | 曲线不是操作数 | 无（语料的布尔容器里没有曲线） |\n')

# 9.3.4 warning text (R11) and factsSkipped row
rep('| 1.6.9 | "节点名 X 与保留名相同，不能按路径引用" |','| 1.6.9 | "节点名 X 与保留名相同，作为路径的后续段时不能按名字引用" |')
rep('| 6.6.4、6.9 | "空间事实未推导，X() 为空集" | `factsSkipped` 为真时每条调用了关系谓词的语句一条 | 无（参考输出 `factsSkipped` 全为 false） |',
'| 6.6.4、6.9、7.1.4 | "空间事实未推导，X() 为空集" | `factsSkipped` 为真时每条调用了关系谓词的语句（concept、pattern、check、style、variant、外观时机的 over；pattern 的关系约束算调用）一条，不论调用了几次。原型 `compileOnce` 用 `buildRelations([])` 静默给出空集，不警告 | 无（参考输出 `factsSkipped` 全为 false） |')

# 9.3.5 E rows
rep('| 3.7.6、4.7 | 变体皮肤放在宿主盒底面中心（等价于注入 `at=宿主.bottom pin=bottom`），总是后置 |\n',
'| 3.7.6、4.7 | 变体皮肤放在宿主包络的底面中心（等价于注入 `at=宿主.bottom pin=bottom`）；皮肤的 `late` 按可判定规则：宿主的包络由子节点算出（容器且没写 `envelope`）时为真，否则为假——原型在求容器盒时顺带标记的结果恰是这条规则，参考输出一致（semantics 的 `slab.skin`、spaceport_blueprint 的 `hangar.roof.skin` 为假，semantics 的 `street.h4.skin` 为真） |\n'
'| 4.3.5、8.3.5 | 普通 copy 永远复制**未修改**的源语句（在全部几何时机修改之前展开）：spaceport_edit 的 `over hangar dw=16` 写在 `copy hangar2 of=hangar` 之前，`hangar2.body.opening` 仍是 12 宽；只有 over 块里的 copy（阶段 5 立即展开，9.2.4）看到重建后的有效语句 |\n'
'| 4.3.1、4.3.3 | copy 的 `of` 指向影子、副本组或皮肤时报"找不到对象 X"（阶段 4 它们还不存在）；copy 换定位关系时连 `align` `alignto` `gap` `offset` `pin` 一起删除（`expandCopies` 相同） |\n'
'| 4.5.1 | 外观时机 over 的判定条件：最外层、无块、无换形、键 ⊆ {mat, color, use}（`elaborate` 相同） |\n'
'| 4.5.3 | over 换定位关系时 `pin` `gap` `offset` `align` `alignto` 保留（`applyOpinions` 相同；与 copy 不同） |\n'
'| 1.7.1、4.5.2 | 几何时机的 over / drop 选择集为空（含推迟到阶段 6 再求一次仍为空）报错"X 没有选中任何对象"（`applyOpinions` 第二轮相同） |\n'
'| 4.6.1 | 换形的目标种类限制：不能换成 repeat / mirror / rotate / copy / 曲线，未知类型报错（`applyOpinions` 相同） |\n'
'| 7.2.5、7.8.4 | 皮肤对 `all()` / `kind()` / `#tag` / 全局索引 / 事实 / 样式作用范围不可见（原型 `allNodes` 过滤 `skin` 相同）；语义层与外观时机的选择器都在皮肤生成前求值，所以任何选择集都不含皮肤；通配、`children()`、`descendants()`、直接路径只在皮肤模板体内的表达式与编译后的工具查询里看到皮肤（精确清单 7.8.4） |\n')

# 9.4.2 delta_tight row
rep('| $\\tau_{\\text{num}}$ | $10^{-9} \\cdot \\max(1, \\lvert a \\rvert, \\lvert b \\rvert)$ | 无量纲的其他数：元数据里的数、`items[].geom` 的角度与无量纲参数、`materials` 的 metal / rough / opacity |\n',
'| $\\tau_{\\text{num}}$ | $10^{-9} \\cdot \\max(1, \\lvert a \\rvert, \\lvert b \\rvert)$ | 无量纲的其他数：元数据里的数、`items[].geom` 的角度与无量纲参数、`materials` 的 metal / rough / opacity |\n'
'| $\\delta_{\\text{tight}}$ | cyl / cone / rod：$R(1 - \\cos 15°)$，$R = \\max(r_1, r_2)$；圆截面 sweep / pipe：$r(1 - \\cos 27°)$；其他实体 0 | 只在**对参考**时用于 `tight`（9.4.3）：参考的 `tight` 由原型 `tightBox` 从凸块顶点算出，圆用内接多边形代替（12 边棱柱；圆截面每环取 8 点，2.1.8、2.8.3），所以比本版按真实几何算的 $T$ **小**，每面至多 $\\delta_{\\text{tight}}$；cone 半径为 0 的一端原型用半径 $10^{-4}$ 的小环，参考在该处可比本版**大**至多 $\\delta_{\\text{out}} = 10^{-4}$（其他实体 $\\delta_{\\text{out}} = 0$） |\n')

# 9.4.3 box/tight row
rep('| `box`、`tight` | `min` / `max` 六个数各在 $\\tau_{\\text{len}}$ 内；`null` 只与 `null` 一致 | 3.1.6、2.1.8、2.6 |',
'| `box` | `min` / `max` 六个数各在 $\\tau_{\\text{len}}$ 内；`null` 只与 `null` 一致 | 3.1.6、2.6 |\n'
'| `tight` | 互比：六个数各在 $\\tau_{\\text{len}}$ 内，`null` 只与 `null` 一致。对参考：先按 N15 规范化（布尔容器补算、特定 revolve 不比），再逐面比——记本版的面坐标 $v$、参考的 $u$（三轴的 min 面与 max 面各一），要求参考盒不比本版盒**向外**超出 $\\delta_{\\text{out}} + \\tau_{\\text{len}}$（$u_{\\min} \\ge v_{\\min} - \\delta_{\\text{out}} - \\tau_{\\text{len}}$、$u_{\\max} \\le v_{\\max} + \\delta_{\\text{out}} + \\tau_{\\text{len}}$），且每面**向内**的偏差不超过 $\\delta_{\\text{tight}} + \\tau_{\\text{len}}$（$u_{\\min} \\le v_{\\min} + \\delta_{\\text{tight}} + \\tau_{\\text{len}}$、$u_{\\max} \\ge v_{\\max} - \\delta_{\\text{tight}} - \\tau_{\\text{len}}$）；$\\delta$ 按该节点的实体种类（9.4.2），轴对齐节点两版相同、也满足这条 | 2.1.8、2.5.4、2.8.3 |')

# 9.4.3 defs row
rep('| `defs` | 相同（模板名列表，含顺序；条目的其他字段由 04-derivation.md 定义后按"相同"比） |',
'| `defs` | 条数与顺序相同；每条的 `name`、`line`、`file`（归一后）、`base` 相同，`params` 逐项的 `name` / `required` / `default` 相同；`meta` 同节点（字段定义 4.2.7）。对参考只比 `name` 序列：参考是模板名数组，N16 |')

# items/curves rows
rep('| `items[]` | 条数相同；按 `path`（布尔容器条目再按操作数顺序）一一对应；',
'| `items[]` | 字段按 2.1.11（字段名沿用原型，值的含义由 02 定义）。条数相同；按 `path`（布尔容器条目再按操作数顺序）一一对应；')
rep('点列表逐点 $\\tau_{\\text{len}}$；`world` 同节点；`material` 的 `name`、`color` 相同，三个数在 $\\tau_{\\text{num}}$ 内 |',
'点列表逐点 $\\tau_{\\text{len}}$（plate 的 `pts`、hull 的 `pts` / `faces` 两份实现按此比，对参考不涉及——语料没有 plate / hull）；`world` 同节点；`material` 的 `name`、`color` 相同，三个数在 $\\tau_{\\text{num}}$ 内 |')
rep('| `curves[]` | 条数相同，按 `path` 对应；`closed` 相同；`pts` 点数相同、逐点 $\\tau_{\\text{len}}$（采样点数由 2.3.1 定义） |',
'| `curves[]` | 字段按 2.1.11（`path`、`closed`、`pts`）。条数相同，按 `path` 对应；`closed` 相同；`pts` 点数相同、逐点 $\\tau_{\\text{len}}$（采样点数由 2.3.1 定义） |')

# 9.4.4 late sentence
rep('后置 `late`（3.6.3，按依赖图判定，无阈值，必须相同）、',
'后置 `late`（3.6.3 按依赖图判定、皮肤按 3.7.6 的规则、影子照抄原件 4.4.5，都无阈值，必须相同）、')

# 9.4.5 N15, N16
rep('| N14 | `variants[].members` | 按节点顺序重排（语料不变） | 7.8.2 |\n',
'| N14 | `variants[].members` | 按节点顺序重排（语料不变） | 7.8.2 |\n'
'| N15 | `nodes[].tight` | (a) 布尔容器（`kind` 是 `union` / `subtract` / `intersect`）的 `null` 改为按 2.5.4 由其操作数的参考 `tight` 组合：union 取并的盒、subtract 取第一个操作数的、intersect 取交（空则零盒）；容器操作数取其内部实体 `tight` 的并（语料 5 个：basics 的 `crate`、spaceport 系三个文件的 `hangar.body`、spaceport_edit 的 `hangar2.body`，都是 subtract）。(b) revolve 节点里 $R$ 不是带符号置换、或 `angle` $< 360$ 的，把 `tight` 标为**不比较**：参考值是 $R_{\\max} \\times [y_{\\min}, y_{\\max}]$ 外接 12 边棱柱的盒（2.2.7 的凸块），不是本版定义的量；比较程序应按该棱柱公式重算并核对参考值，但不计入失败（语料 1 个：rotation 的 `radar.dish`）。其余节点不改值，按 9.4.3 的 $\\delta_{\\text{tight}}$ 规则比 | 2.5.4、2.2.7、2.8.3 |\n'
'| N16 | `defs` | 字符串数组的每个名字改为 `{ "name": 名字 }`；比较只对 `name` 序列（`line`、`file`、`params`、`base`、`meta` 参考没有，不比；语料 5 个文件有 def） | 4.2.7、4.8.5 |\n')

# 9.4.7 tight row
rep('| `tight` ⊆ `box` | 1644 个节点全部成立',
'| `tight` 与 2.1.8 的独立复算（2.8.2） | 923 个有 `tight` 的实体节点里 812 个逐位一致（$\\le 10^{-9}$）；不同的 87 个 cyl、21 个 rod 每面偏差 $\\le R(1 - \\cos 15°)$（最大 0.00613，恰为 $0.0341\\,R$）、2 个 sweep（`feed1`）偏差 $4.3 \\times 10^{-6}$，都在 $\\delta_{\\text{tight}}$ 内且参考 ⊆ 本版；1 个 revolve（`radar.dish`，参考大 1.94）与 5 个布尔容器（`null`）由 N15 处理 |\n'
'| `tight` ⊆ `box` | 1644 个节点全部成立')

# 9.5.2 rows
rep('| basics.s3d | 1.6.9 软保留名：`box floor`（:8）→ 警告；6.5.1 布尔容器事实 `approx`：`crate on floor`、`orb inside crate` → `approx: true` |',
'| basics.s3d | 1.6.9 软保留名：`box floor`（:8）→ 警告；6.5.1 布尔容器事实 `approx`：`crate on floor`、`orb inside crate` → `approx: true`；2.5.4：`crate` 的 `tight` 从 `null` → 操作数组合（N15） |')
rep('| rotation.s3d | 4.2.7：`sat.wing2`（`mirror→box` → `box`，:30）；',
'| rotation.s3d | 4.2.7：`sat.wing2`（`mirror→box` → `box`，:30）；`defs` 1 个条目 `turbine`（N16）；2.2.7：`radar.dish` 的 `tight`（倾斜 revolve，参考大 1.94，N15 不比较）；')
rep('| semantics.s3d | 4.2.7：`rampE`（`mirror→span` → `span`，:33）、',
'| semantics.s3d | 4.2.7：`defs` 4 个条目 house / chimney_house / planks / townhouse（N16）；`rampE`（`mirror→span` → `span`，:33）、')
rep('| spaceport.s3d | 4.2.7：`tower2`（`mirror→group` → `truss`，`types` `null` → `["truss"]`，:47）、`arm2`（`mirror→span` → `span`，:48）；',
'| spaceport.s3d | 4.2.7：`defs` 2 个条目 truss / shed（N16）；`tower2`（`mirror→group` → `truss`，`types` `null` → `["truss"]`，:47）、`arm2`（`mirror→span` → `span`，:48）；2.5.4：`hangar.body` 的 `tight`（N15）；2.2.9：`feed1` 的 `tight` 偏差 $4.3 \\times 10^{-6}$（$\\delta_{\\text{tight}}$ 内）；')
rep('| spaceport_blueprint.s3d | 同 spaceport 的全部项，另：',
'| spaceport_blueprint.s3d | 同 spaceport 的全部项（`defs` 3 个条目 truss / shed / ribbed），另：')
rep('| spaceport_edit.s3d | 同 spaceport 的 `tower2` / `arm2` 项；',
'| spaceport_edit.s3d | 同 spaceport 的 `tower2` / `arm2` 项与 `defs`（2 个条目）；2.5.4：`hangar.body`、`hangar2.body` 的 `tight`（N15）；')
rep('**corpus/examples**\n\n| 文件 | 命中的项（节；位置） | 本版的编译结果 |',
'**corpus/examples**（2.8.3 的 `tight` 差异里，87 个非轴对齐 cyl 与 21 个 rod 分布在各文件、按 9.4.2 的 $\\delta_{\\text{tight}}$ 比较而不逐一列出；布尔容器、revolve 与 sweep 的 5 + 1 + 2 个节点列在各行）\n\n| 文件 | 命中的项（节；位置） | 本版的编译结果 |')

# 9.5.4
rep('语料里对它们的引用（basics.s3d 的 7 处 `on=floor…`）全部解析。','语料里对它们的引用（basics.s3d 的 6 处 `on=floor…`）全部解析。')
rep('`gap` 的精度、概念成员顺序（2 个）、材质条目形状（全部）与 3 个改色条目。',
'`gap` 的精度、概念成员顺序（2 个）、材质条目形状（全部）与 3 个改色条目、`defs` 的条目形状（5 个文件）、`tight`（87 个 cyl 与 21 个 rod、2 个 sweep 在 $\\delta_{\\text{tight}}$ 内；1 个 revolve 不比；5 个布尔容器补算）。')

# 9.6.2 rows
rep('| 副本式节点带 `mat`（10.3.1） | 原件是容器时本版会生效；不想生效就删掉 |\n',
'| 副本式节点带 `mat`（10.3.1） | 原件是容器时本版会生效；不想生效就删掉 |\n'
'| `defs` 的条目形状（4.2.7） | 不改文件。按名字数组读 `defs` 的工具改读每个条目的 `name` |\n'
'| 非轴对齐圆类实体与布尔容器的 `tight`（2.1.8、2.5.4） | 不改文件。本版的紧盒按真实几何，比原型大至多 $\\delta_{\\text{tight}}$；布尔容器有了紧盒。按 `tight` 做布局或评分的工具要重新取基线 |\n')

for old,new in R:
    c=s.count(old)
    if c!=1:
        print('COUNT',c,'for:',old[:80]); sys.exit(1)
    s=s.replace(old,new)
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok',len(R))
