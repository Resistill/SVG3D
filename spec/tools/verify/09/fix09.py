import re, sys
P = '/home/user/SVG3D/spec/09-compat.md'
s = open(P, encoding='utf-8').read()
assert '\r' not in s

def rep(old, new, count=1):
    global s
    n = s.count(old)
    if n != count:
        print('ANCHOR COUNT', n, 'for', old[:80]); sys.exit(1)
    s = s.replace(old, new)

def line_replace(prefix, new):
    """replace the whole line starting with prefix (must be unique)"""
    global s
    lines = s.split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(prefix)]
    if len(hits) != 1:
        print('LINE COUNT', len(hits), 'for', prefix[:80]); sys.exit(1)
    lines[hits[0]] = new
    s = '\n'.join(lines)

def line_delete(prefix):
    global s
    lines = s.split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(prefix)]
    if len(hits) != 1:
        print('LINE COUNT', len(hits), 'for', prefix[:80]); sys.exit(1)
    del lines[hits[0]]
    s = '\n'.join(lines)

def insert_after_line(prefix, text):
    global s
    lines = s.split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(prefix)]
    if len(hits) != 1:
        print('LINE COUNT', len(hits), 'for', prefix[:80]); sys.exit(1)
    lines.insert(hits[0] + 1, text)
    s = '\n'.join(lines)

def insert_before_line(prefix, text):
    global s
    lines = s.split('\n')
    hits = [i for i, l in enumerate(lines) if l.startswith(prefix)]
    if len(hits) != 1:
        print('LINE COUNT', len(hits), 'for', prefix[:80]); sys.exit(1)
    lines.insert(hits[0], text)
    s = '\n'.join(lines)

# ---- 范围 / 依赖 / 记法 ----
rep('01 到 08、10 的全部"原型 …；本版 …"陈述与各章末尾"因本章定义而预期不同的项"小节（2.8.3、3.11.5、4.8.5、5.5.4、6.8.5、7.9.4、8.6.2、10.6）。',
    '01 到 08、10 的全部"原型 …；本版 …"陈述与各章末尾"因本章定义而预期不同的项"小节（1.11、2.8.3（含其末段的 B 类报错清单）、3.11.5、4.8.5、5.5.4、6.8.5、7.9.4、8.6.2、10.6）。')
rep('"参考输出"指 corpus/reference/examples/*.compile.json（7 个）与 corpus/reference/exports/*.compile.json（11 个）。"语料"指 corpus/examples/*.s3d（7 个示例与 lib/semantics.s3d）与 corpus/exports/*.s3d（11 个逆向输出）。',
    '"参考输出"指 corpus/reference/examples/*.compile.json（7 个）与 corpus/reference/exports/*.compile.json（10 个：basics.L0、curves.L0、l2fb、rotation.L0、semantics.L0、spaceport.L0 / L1 / L2、spaceport_edit.L0 / L1；MANIFEST 写的"11 个"是笔误，它列出的名字就是这 10 个）。"语料"指 corpus/examples/*.s3d（7 个示例与 lib/semantics.s3d）与 corpus/exports/*.s3d（10 个逆向输出）。')

# ---- 9.1.1 ----
rep('1. **语法向后兼容。** 原型能解析的每一条语句（01-syntax.md 1.3 节的全部形态）在本版都能解析为同一种语句；本版没有删除任何语句种类、属性或表达式写法，也没有改变任何记号的含义。唯一的词法变化是 `~` 在**不紧跟标识符**时成为运算符（1.2.10）：原型里这个位置的 `~` 是"无法识别的字符"，所以没有任何原型能编译的文件因此改变含义。',
    '1. **语法向后兼容。** 原型能解析的每一条语句（01-syntax.md 1.3 节的全部形态）在本版都能解析为同一种语句，**除** 9.3.1 的"词法、语法与表达式"表所列的无效写法（单引号字符串、`1.`、一元 `+`、非 ASCII 空白、名字不紧跟种类词等——原型放行，本版报错）；本版没有删除任何语句种类、属性或表达式写法。记号含义的变化只有两处：`~` 在**不紧跟标识符**时成为运算符（1.2.10；原型里这个位置的 `~` 是"无法识别的字符"，所以没有任何原型能编译的文件因此改变含义），以及名字字面量类型的值不再做名字查找（1.4.2；原型里 `across=x` 在场景有名为 `x` 的节点时解析成该节点，9.3.2）。')
rep('9.5 对全部 19 个语料文件逐一核查：没有一个触发 9.3.1 的任何一项，`errors` 全部为空；4 个文件各多出一条**警告**（软保留名，9.3.4）。',
    '9.5 对全部 18 个语料文件逐一核查：没有一个触发 9.3.1 的任何一项，`errors` 全部为空；4 个文件各多出一条**警告**（软保留名，9.3.4）。')

# ---- 9.1.2 ----
rep('原型对它报"未知的类型 “~lang=2”"（`makeStmt`，经 scratchpad `c9/t_lang.s3d` 确认），其余语句照常编译。',
    '原型对它报"未知的类型 “~lang=2”"（`makeStmt` 的通用分支把它当作种类为 `~lang=2` 的节点语句，`elaborate` 的 `one` 报错；经 scratchpad `c9/t_lang.s3d` 确认），其余语句照常编译。')

# ---- 9.2.1 ----
rep('| `import` | 报错"import 写成：import "文件.s3d""（`makeStmt` 只接受一个字符串） | 接受，随语句记录、不传给被导入文件（5.1.4） |',
    '| `import` | 报错"import 写成：import "文件.s3d""（`makeStmt` 只接受一个字符串） | 接受，随 `files[]` 条目的 `import.meta` 输出（8.4.3）、不传给被导入文件（5.1.4） |')
rep('| 文件头（只含元数据项的首行） | 报错"未知的类型 “~lang=2”" | 文件元数据（1.3.1、8.1.5） |',
    '| 文件头（只含元数据项的首行） | 报错"未知的类型 “~lang=2”"（`elaborate` 的 `one`，9.1.2） | 文件元数据（1.3.1、8.1.5） |')

# ---- 9.2.4 ----
line_delete('| 1.5.6 | `kind("box")`（字符串参数） | 只接受裸标识符 |')
insert_after_line('| 7.8.9 | 无效的 variant 语句 | 没有输出条目 |',
'''| 1.2.1、1.3.3 | 属性值里记号之间有空白（`size=1, 1, 1`、`at=a.top + (0,1,0)`、`on=kind(box) where …`） | `lexStatements` 在深度 0 按空白切词，值被截断（"表达式不完整：1,"、"无法理解 “+”"） | 值延伸到第一个不能继续表达式的记号（1.5.1） |
| 1.2.4 | 字符串转义 `\\"`、`\\\\`、`\\n` | 没有转义：`"a\\"b"` 在 `\\"` 处结束，报"字符串缺少结束引号" | 三种转义合法 |
| 1.2.6、1.5.4 | 坐标后缀后的分量 `B[0.5].y`、`B.top[0,0].x` | `exprTokens`：`]` 后的 `.` 报"无法识别的字符 “.”" | 合法 |
| 1.3.8 | `over a: box`（冒号不带空白） | `makeStmt` 只认独立的 `:` 词，`a:` 交给 `parseValue` 报"无法识别的字符 “:”" | 合法 |
| 1.3.10 | pattern 关系约束两侧是带空白的表达式（`kind(box) where h > 1 on y`） | 两侧各只能是一个无空白的词（`makePattern` 的 `\\S+`） | 两侧是任意表达式 |
| 1.3.12、1.3.13 | style / variant 的选择器含空白（`style a \\| b mat=m`） | 选择器只能是一个词（`words[1]`），其余报"style 只接受 key=value，看不懂 “\\|”" | 选择器到第一个 `k=` / `~k=` 为止 |''')

# ---- 9.3.1：第 1 章表 + 第 2 章表 ----
insert_before_line('**结构、命名、层**',
'''**词法、语法与表达式（第 1 章，1.11 的 B 类）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 1.2.1 | 其他 Unicode 空白（如 U+00A0）出现在属性值里 | `exprTokens` 用 `\\s` 当空白跳过 | 报错"无法识别的字符" | 字符集封闭（1.2.1） | 无 |
| 1.2.3 | `1.`（小数点后没有数字） | 数字正则 `\\d+\\.?\\d*` 接受为 1 | 语法错误 | 不猜 | 无 |
| 1.2.4 | 单引号字符串 `'x'` | `exprTokens` 接受 `'` 作定界符 | 报错"无法识别的字符" | 只有一种字符串写法 | 无 |
| 1.2.6 | 方向组合词含 `center`、同轴重复、超过 3 个词（`top-center`、`left-right`） | `mergeDirs` 只要求每个词是方向词，`dirWord` 把向量相加（`left-right` = 零向量） | 报错"方向组合词矛盾" | 无定义的组合 | 无 |
| 1.3.3 | 名字不紧跟种类词（`box #t a`、`box size=1,1,1 c`） | `makeStmt` 把第一个不是 `k=v`、不以 `#` / `~` 开头的词当名字 | 报错"名字要写在种类词之后" | 1.3.3 | 无 |
| 1.3.3、1.9、8.1.4 | 同一语句里重复的属性 / 元数据键 / 标签 | `Map.set` 后写的覆盖先写的（`~fit=0.5 ~fit=0.9` 得 0.9）；标签数组重复 | 报错"重复的属性 / 元数据 / 标签"，节点退化（重复的元数据键只是该键不存在，8.1.4） | 不猜 | 无 |
| 1.3.3、1.3.15 | 模板实例语句带块 | `elaborate` 的模板分支不读 `s.children`，块静默丢弃 | 报错"X 不能有子语句" | 1.3.15 | 无 |
| 1.3.15 | 实体、曲线、副本式 mirror / rotate 带**空**块 `{ }`；material、let、check、style、variant 带块 | 只在 `children.length > 0` 时报错（`elaborate`）；material / check / style / variant 的块静默丢弃 | 报错（空块也报） | 1.3.15 | 无 |
| 1.3.4、1.3.15 | `def X` 没有块 | 接受为空体 | 报错"def 需要块" | 1.3.4 | 无 |
| 1.3.4、10.2 | 模板参数名与 P 组属性名、`use`、`mat` / `color` 同名（`def f(gap=1)`、`def f(mat=1)`） | 允许，参数优先于同名定位属性（`elaborate` 的 `isPlacement`）；`mat` / `color` 参数吞掉实例语句上的外观属性 | def 处报错"参数名与属性名重名"，整条 def 无效 | 定位与外观属性不能被参数遮蔽 | 无 |
| 1.4.4、3.10.5 | line / bezier / spline 写定位 / 朝向属性；span / sweep / pipe 写 `pin` `mat` `color` 之外的 P 组属性；arc / helix 写 `mat` / `color` | `KIND_PROPS ∪ PLACEMENT` 全部接受，`local` 的 `PATH_CURVES` / `LINKS` 分支不读它们（静默无效） | 报错"X 没有属性 k"，节点退化 | 1.4.4 | 无 |
| 1.4.4 | `alignto` 没有 `align`；`aimwith` 没有 `aim` | 静默忽略（`local` 只在有 `align` 时读 `alignto`；`orientation` 只在有 `aim` 时读 `aimwith`） | 报错"alignto 需要 align" / "aimwith 需要 aim"，节点退化 | 1.4.4 | 无 |
| 1.4.5、3.8.1 | `repeat` 没写 `n` | 缺省 1 个实例（`elaborate`） | 报错"缺少 n"，节点退化、不生成实例 | 1.4.5 必填 | 无（语料的 repeat 都写了 n） |
| 1.4.5、1.4.3 | span 的 `pin` 不是 `bottom` / `top` / `center` | `Solver.sym` 退回缺省 `center` | 报错"…只能是 bottom / top / center"，节点退化 | 枚举规则 1.4.3 | 无 |
| 1.5.1 | 一元 `+`（`size=+2,1,1`） | `unary` 接受并忽略 | 语法错误 | 1.5.1 | 无 |
| 1.5.2 | 除以零、结果不是有限数（`1/0`、`sqrt(-1)`） | 得到 Infinity / NaN，节点的盒与位置输出 `null`，不报错 | 报错"除以零" / "结果不是有限数"，节点退化 | 1.5.2 | 无 |
| 1.5.5 | `and` / `or` / `not` 的操作数是集合或数；`where` 的条件不是布尔（`where 1`） | `or` / `and` 对集合做并 / 交（与 `\\|` `&` 同一分支），`asBool` 把非零数、非空集合当真 | 报错"需要真假值，得到 X" / "where 的条件需要真假值"；集合运算只用 `\\|` `&` `-` | 1.5.5 | 无（语料的 `and` / `or` 两侧都是比较，1.11） |
| 1.5.6 | `kind()`、`min()`、`max()` 没有参数 | `kind()` 得空集；`min()` 得 Infinity | 报错"X() 需要 N 个参数" | 1.5.6 | 无 |
| 1.5.8 | 同一块里重复的 `let X` | 后写的覆盖先写的 | 报错"重复定义 X" | 1.5.8 | 无 |
| 1.5.8、5.4.2 | 属性值（或它读到的 `let`）用了 `where` 的尺寸字段、关系谓词、概念 | `let tall = kind(box) where h > 1.5` 后 `on=tall` 可用，结果取决于求解缓存 | 时机错误，节点退化 | D10 | 无 |

**几何（第 2 章，2.8.3 末段）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 2.2.1–2.2.4 | box / wedge 的 `size` 分量 $\\le 0$；cyl 的 `r` $\\le 0$；cone 的 `r` / `r2` 为负或同时为 0；cyl / cone / sphere 的 `h` / `r` $\\le 0$ | 照样建盒与凸块（`Solver.box`、`localParts`） | 报错，节点退化 | 几何参数不合法（1.10.2） | 无 |
| 2.2.5、2.1.6.2 | plate 轮廓自交；耳切失败的"轮廓退化" | 只查面积（`Solver.box` 的 plate 分支 `polyArea2`） | 报错"轮廓自交" / "轮廓退化" | 2.1.6.2 的简单性 | 无（语料没有 plate） |
| 2.2.7 | revolve 的 `angle` 不在 $(0, 360]$、轮廓面积为 0 或自交 | 照样生成（`angle=500`） | 报错，节点退化 | 同上 | 无 |
| 2.2.8 | span 的 `steps` 不是整数 $\\ge 1$；台阶两端在同一竖直线上；`steps` 与 `r`、`r` 与 `section` / `pin`、`steps` 与 `pin` 同写；`to` 与 `drop` 都写或都不写；`dir` 没有 `drop`；`section` 分量 $\\le 0$ | `Math.round(steps)`（`steps=2.4` 得 2 级）、零宽踏步、只取其中一种形态（`Solver.link`） | 报错，节点退化 | 互斥与范围规则（1.4.3、1.4.4） | 无 |
| 2.2.9 | sweep / pipe 的 `r` 与 rect / 多边形同写、`section` / `pin` 与 circle / 多边形同写、`smooth` 与曲线路径同写、路径相邻点重合、截面面积为 0 或自交 | 只取其中一种形态（`profile=circle section=3,1 r=0.2` 取圆截面并忽略 section；`Solver.link`） | 报错，节点退化 | 同上 | 无 |
| 2.3.1–2.3.6 | arc 的起止角相同；helix 的 `pitch` 或 `turns` 为 0；曲线总长为 0；闭合 spline 少于 3 个点 | `pitch=0` 得到闭合的平圆；闭合 spline 给 $12n$ 个点（`Solver.curve`、`catmullRom`） | 报错，节点退化 | 退化曲线无定义 | 无 |
| 2.4.1、2.4.2 | `B.top[u,w]` 竖直线没碰到实体；`B.side[θ,v]` 的 $v$ 或 $\\theta$ 超出范围 | 不检查范围（`surfaceAnchor`） | 报错，节点退化 | 2.4 | 无（语料没有曲面锚点） |
| 2.5.1 | 布尔容器没有操作数；subtract 的第一个操作数没有实体 | 不出条目、不报错（`buildItems` 的 `single`） | 报错 | 2.5.1 | 无 |
| 2.6 | `envelope` 的负尺寸或最小值大于最大值 | 接受（`Solver.box` 的 envelope 分支） | 报错 | 2.6 | 无 |
''')
line_delete('| 1.3.4、10.2 | 模板参数叫 `mat` / `color` | 参数吞掉实例语句上的 `mat=` | def 处报错 |')

# ---- 第 3 章表 ----
line_replace('| 3.5.4 | `aimwith` 不是方向词 / 组合词 | 退回缺省 `front`（`Solver.sym`） |',
'| 3.5.4 | `aimwith` 的值不是名字（`aimwith=5`） | `Solver.sym` 退回缺省 `front`（`c9/a_aim.s3d`：无错误、按 front 对准）；值是名字但不是方向词（`aimwith=foo`）时原型也报错"aimwith 需要方向词"（`orientation`，相同） | 一律报错"aimwith 需要方向词"，节点退化 | 类型规则 1.4.1 | 无（语料只有 `aimwith=top`） |')
line_delete('| 3.8.1 | `n` 引用对象或用 `where` | 在求解期求值 |')
line_replace('| 3.9.5 | 方向规格（`axis` / `dir`）求得零向量；镜像面法线为零 |',
'| 3.9.5 | 方向规格（rotate 的 `axis`、span 的 `dir`、`rot` 之外的方向）求得零向量 | `dirSpec` 对向量只做 `unit`，零向量继续算：`rotate r of=b axis=(0,0,0) angle=90` 得到退化矩阵（`c9/a_zero.s3d`：无错误，$R_{00} = 0$） | 报错"不能是零向量"，节点退化（镜像面法线为零原型已报错，9.3.5） | 结果无定义 | 无 |')
insert_after_line('| 3.3.8、3.1.4 | 写了多个定位关系',
'| 3.3.4 | `against=B[0,0,0]`（主法线为零的向量锚点） | 不报错：$\\nu = 0$，自动选面时全部得分为 0 取 `left`，gap 无效（`relation`） | 报错"against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]"，节点退化 | 1.4.4：against 必须带面锚点 | 无 |')

# ---- 第 4 章表 ----
line_replace('| 4.6.2 | 模板 → 模板换形，参数不在目标参数表 |',
'| 4.6.2 | 模板 → 模板换形，参数不在目标参数表 | 保留全部参数，重新展开时报"模板 X 没有参数 k"，节点仍按目标模板重建、多余的参数被忽略（`applyOpinions`；`c9/a_tt.s3d`：`over x : U w=3` 报错后 `x.b` 仍是 1 宽） | 派生链上保留目标参数表里有的；否则按名字匹配，多余的报错"模板 T 没有参数 k（换形自 X）"，该成员不被修改 | D11 | 无 |')
line_replace('| 4.4.6 | `of=` 不是单个对象路径 |',
'| 4.3.3、1.3.14 | copy 语句上既不是 P 组也不是模板参数的键（`copy b of=a size=2,2,2`） | 静默写进新语句并生效（`expandCopies`：得到 2 × 2 × 2） | 报错"copy 没有属性 k"，该 copy 无效 | 不猜（4.8.5） | 无（语料的 copy 只换 `on=`） |')

# ---- 第 7 章表 ----
rep('| 7.5.1 | 同一角色两条 `in` 约束 | `makePattern` 后一条覆盖前一条 |',
    '| 7.5.1 | 同一角色两条 `in` 约束 | `matchPattern` 的 `domains.set` 让后一条覆盖前一条（`makePattern` 不检查） |')

# ---- 9.3.2 ----
rep('在显式依赖图上按"从 $\\{L(c), X(c)\\}$ 可达 $X(A)$"判定，容器按节点顺序逆序、子节点按节点顺序处理',
    '在显式依赖图上按"从 $\\{L(c), X(c)\\}$ 可达 $X(A)$"判定，容器按节点顺序（父先于子）、子节点按节点顺序处理')
rep('`expandCopies` 把 R 追加在父节点全部兄弟之后，且不计入匿名段计数（`group g { box p; copy q of=p; box }` → `box#2`） | R 占 copy 语句的位置，阶段 3 的占位计入匿名计数（→ `box#3`）',
    '`expandCopies` 把 R 追加在父节点全部兄弟之后，且不计入匿名段计数（`group g { box p; copy q of=p; box }` 得到 `g.p`、`g.box#1`、`g.q` 的顺序；`c9/a_anon.s3d`） | R 占 copy 语句的位置，阶段 3 的占位计入匿名计数（`g.p`、`g.q`、`g.box#2`）')
rep('`broadPairs` 的 y / z 用 `> -margin`（排除恰好相等）、x 用 `>=`，`gapBetween` 用 `< -eps` 跳过（保留恰好相等），三处不一致 | 三处统一为重叠量 $\\ge -\\varepsilon$ 保留 | 边界只按一条不等式（2.1.4） | 无（只在恰好相等时不同：`box exact on=ground gap=0.02` 原型无事实、本版 `touching` 0.02） |',
    '`broadPairs` 的 y / z 用 `> -margin`（排除恰好相等）、x 用 `>=`，`gapBetween` 用 `< -eps` 跳过（保留恰好相等），三处不一致；`against` 的轴对齐判定 `isAxisAligned` 用严格的 `< 1e-6` | 三处统一为重叠量 $\\ge -\\varepsilon$ 保留；轴对齐用 $\\le \\varepsilon_{\\text{dir}}$（6.5.3） | 边界只按一条不等式（2.1.4） | 无（只在恰好相等时不同：`box exact on=ground gap=0.02` 原型无事实、本版 `touching` 0.02；6.8.4 的 `exact` / `ax2`） |')
insert_after_line('| 2.5.4 | 布尔容器的 `tight` | `null`（`tightBox` 只对有 `geom` 的叶子算）',
'''| 2.5.4、2.5.5 | 布尔容器的局部包围盒 | `Solver.box`：union / intersect 取全部子节点（含曲线）盒之并；subtract 取第一个**子节点**（可以是曲线）的盒 | intersect 取交；曲线不计；subtract 取第一个**操作数** | 2.5.4 | 无（5 个布尔容器都是 subtract 且首个子节点是实体，两版相同） |
| 2.4.1、2.4.2 | 曲面锚点 `B.top[u,w]`、`B.side[θ,v]` | `surfaceAnchor`：top 除 sphere、`axis=y` 圆台、revolve、mesh 外一律取盒顶，不检查范围，revolve 把两端半径都 $\\ge \\rho$ 的边算作候选；side 对 `axis=z` 是左手（$\\theta = 90$ 指 $+x$），横放圆台的法线轴向分量朝宽端，revolve 的法线水平 | 按真实表面（2.4.1 的表）、右手、法线朝窄端、revolve 取轮廓边的法线；范围报错（9.3.1） | 2.4 | 无（语料没有曲面锚点） |
| 2.3.1 | 零长度折线段上的切线（`C[t]`、`dir=`、`along`） | `curveAt` 给零向量 | 取相邻非零段的方向 | 2.3.1 | 无（语料曲线没有重合的相邻点） |
| 6.4.2 | 同心球（两个球的球心重合） | 没有候选轴，`gap` 为 $-\\infty$，JSON 输出 `null`（`partGap`） | $\\mathrm{gap} = -(r_A + r_B)$，是数 | `gap` 必须是数（6.6.1） | 无（6.8.4） |
| 1.3.2 | `{` 单独起一行 | `parseProgram` 报"“{” 前缺少语句"后只跳过 `{`，块内语句按外层语句解析并生成节点，配对的 `}` 再报"多余的 “}”" | 报错一次，整块到配对的 `}` 丢弃 | 1.3.2 | 无 |
| 1.4.2、1.4.3 | 名字字面量类型的值碰巧是某个节点的名字（场景里有节点 `x` 时写 `mirror m of=c across=x`、`cyl … axis=x`） | `evalPath` 先做名字查找：`across=x` 解析成节点 `x`，镜像面过该节点（1.11 的小例子：副本 $t = 15$ 而不是 $-3$） | 字面量不做名字查找：`across=x` 是坐标面 | D24 | 无（语料里没有叫 `x` `y` `z` 或方向词 / 面名的节点被这样引用，1.11） |
| 1.5.5 | 向量相等 `(1,2) == (1,2)` | `same` 对数组按引用比较，恒为假 | 逐分量比较，为真 | 1.5.5 | 无 |
| 1.5.5 | `kind == box`、`name == a`、`mat == steel` 的右侧碰巧是某个节点的名字 | 右侧先做名字查找，解析成引用后与字符串比较恒为假（场景里有 `box box` 时 `all() where kind == box` 为空） | 右侧单段裸标识符按名字字面量 | D24 | 无 |
| 1.5.6 | `round` 恰好 .5 | `Math.round` 向 $+\\infty$ 取整（$\\mathrm{round}(-2.5) = -2$） | 远离零（$-3$） | 1.5.6 | 无 |
| 1.6.1 | 匿名段的计数 $i$ | 取创建时的子节点下标，drop 之后新建的匿名节点会重用编号（`drop g.a` 后 over 块新建的匿名 box 与已有的 `g.box#1` 同路径） | 按此前已创建的总数，永不回收 | 1.6.1 | 无 |
| 1.5.4、1.6.5 | 路径下降时遇到后缀保留词（`g.top` 而 g 下有名为 `top` 的子节点） | `Solver.find` 的 `descend` 先按子节点名匹配，`g.top` 解析到子节点 | 后缀词永远按后缀解释，`g.top` 是锚点 | 1.6.5 第 5 步 | 无 |
| 1.6.7 | `kind(group)`、`kind(instance)`、`kind(mirror)` 与 `where kind` 对阵列实例、副本组 | 实例与副本组内部是 `group` 节点：`kind(group)` 选中它们，`kind(instance)` 为空，`where kind == "group"` 为真（输出的 `nodes[].kind` 却是 `instance` / `mirror`） | 种类是 `instance` / `mirror` / `rotate`，`kind(group)` 只选 `group` 语句生成的节点 | 1.6.7 | 无（语料的 `kind()` 参数都是实体种类词或模板名） |
| 1.6.7、4.4.5 | 副本式 `mirror R of=A` 的标签 | 只有 R 自己写的标签（`elaborate`：`new Node(…, s.tags, …)`），`#w` 选不中 A 的副本 | $A$ 的标签 ∪ R 自己写的 | 4.4.5 | 无（语料的副本式语句与原件都没有标签） |
| 7.3.1 | `where kind` 对副本式节点与影子（原件是模板实例） | `evalAst` 取 `m.template ?? K(m)`，得到 `group`（`kind(hut)` 却选中它；`07/t4.s3d`） | 原件的种类（模板名），与 `kind()` 一致 | 1.6.7 | 无 |
| 7.2.5、7.8.4、7.1.1 | 语义语句里的 `descendants()`、`children()`、通配、直接路径 `x.skin` 对皮肤 | 第二遍在**有皮肤的树**上求值语义语句：能选到皮肤节点，皮肤节点带 `concepts` 标注（`07/t2.s3d`：`concept desc = descendants(g)` 含 `g.skin`，`check count(descendants(g)) == 1` 不通过） | 语义语句在皮肤生成前求值，任何选择集都不含皮肤，皮肤的 `concepts` 恒为 `[]` | 7.1.1 的保证 | 无（语料的语义语句不对宿主用这些构件；7.9.1 的概念、模式、检查与参考一致） |
| 4.1.2、4.8.5 | copy 块 / over 块里嵌套的 over / drop 与其后最外层修改的先后 | 展开该块时追加到修改队列末尾，晚于全部最外层修改（`expandCopies` / `applyOpinions`：`copy b of=a { over c size=2,2,2 }` 后再 `over b.c size=3,3,3`，b.c 是 2） | 位置 = copy / over 语句的位置（4.1.2），b.c 是 3 | 4.1.2 的全序 | 无 |
| 4.7 第 6 条 | 皮肤模板体里的 over / drop / copy | 挂载时已过修改与 copy 阶段，登记后静默丢失（`compileOnce`） | 在皮肤子树内施加 / 立即展开 | 皮肤体与模板体同义（7.8.5） | 无（三个皮肤模板没有这些语句） |
| 4.7 第 3 条、7.8.3 | 变体自动传参 `w h d` 在基类型里声明 | 只看最终定义自己的 `def.params`，不传（`compileOnce`） | 按合并后的参数表传 | 4.2.5 | 无（皮肤模板都不是派生的） |''')

# ---- 9.3.3 ----
rep('| 7.1.1 | 编译遍数 | 有变体的文件编译两遍（第二遍挂皮肤后重算一切） | 一遍、两个阶段；因皮肤对阶段一不可见，结果逐位相同（7.1.1 的保证清单） | 实现自由 | 无差别 |',
    '| 7.1.1 | 编译遍数 | 有变体的文件编译两遍（第二遍挂皮肤后重算一切） | 一遍、两个阶段；变换、包围盒、事实、`all()` / `kind()` / `#tag` 逐位相同，只有 9.3.2 的"语义语句对皮肤的可见性"一行不同（7.9.4） | 7.1.1 的保证 | 无差别 |')
insert_after_line('| 8.4.3 | 编译输出顶层 | 没有 `meta` |',
'''| 8.4.3、5.2.3 | 编译输出顶层 `files` | 没有 | 已加载文件的数组 `[{file, meta, import}]`，主文件在前；无 import 时只有主文件一条 | 8.4.3 | 全部文件（规范化 N12） |
| 1.6.1 | 匿名模板实例的段 | `group#i`（`elaborate` 用 `group` 建实例节点） | `模板名#i` | 1.6.1 | 无（语料的模板实例都具名） |
| 1.3.11 | 没有说明的 check 的 `message` | 词之间的空白规整为单个空格（`words.join(' ')`） | 表达式原文（首尾空白去掉） | 1.3.11 | 无（语料的 check 都带说明） |
| 1.3.2 | 文件结束时块没有关闭的错误行号 | 文件末行（`parseProgram`） | `{` 所在行 | 1.3.2 | 无 |
| 1.1.5 | 错误的排序 | 只按行号（`compileOnce` 的 `errors.sort`），不分文件 | 文件序号、行号、发现先后 | 1.1.5 | 无（语料无错误） |
| 2.1.11、2.2.7 | revolve 条目的 `geom.prof` | 平滑后不截断负半径、不规整（`profile=[(3,0); (0,1); (1,2)] smooth=true` 出现 $r = -0.0575$；相邻重复点保留） | 第 3 步截断为 0、第 5 步去重 | 2.2.7 | 无（语料 revolve 的 `prof` 最小 $r$ 都是 0、没有重复点） |
| 7.5.8 | 失败的模式的 `roles` | `roles: []`（`compileOnce` 的 catch 分支） | 签名顺序照常，集合角色带 `*` | 签名是静态的 | 无（语料的模式都成功） |
| 8.1.3、8.4.2 | 元数据值 `true` / `false`；列表值 | 退化成字符串 `"true"` / `"false"`（原型没有布尔字面量）；列表输出 `{ "list": [1, 2] }`（`evalAst`） | 布尔 `true` / `false`；数组 `[1, 2]` | 8.4.2 | 无（exports 的值全是数与字符串） |
| 8.3.5 | copy 的原件是阵列实例时 R 的元数据 | 新建 `group` 时不写 `meta`，copy 语句自己的元数据丢失（`expandCopies`） | R 的元数据 = copy 语句自己的 | 8.3.5 | 无 |
| 8.1.3、8.1.6 | 元数据某一键求值失败时同一语句的其他键 | 整条语句的元数据全部丢失（`staticMeta`：`~a=1 ~s=all()` 得 `{}`） | 只有出错的键不存在 | 8.1.6 | 无 |''')

# ---- 9.3.4 ----
line_replace('| 2.4.1 | 网格曲面锚点的近似 |',
'| 2.1.6.2、2.2.5、2.2.7、2.2.9 | 轮廓规整（末点与首点重合时去掉末点；相邻重合点只保留第一个） | plate / revolve / sweep 截面的轮廓被规整 | 无 |')
rep('原型唯一的警告来源（`compileOnce` 对 `kind()` 未知名字之外的情形）在语料上也为空；参考输出 18 个文件的 `warnings` 全部为 `[]`。',
    '原型唯一的警告来源是 `Solver.warn`：`surfaceAnchor` 对 mesh 的 `top[u,w]` 取外包络顶面时的两条文案（与 2.4.1 相同，同一文案只报一次——这条警告两版相同，不在上表）；它在语料上为空，参考输出 17 个文件的 `warnings` 全部为 `[]`。')

# ---- 9.3.5 ----
line_replace('| 4.5.1 | 外观时机 over 的判定条件：最外层、无块、无换形、键 ⊆ {mat, color, use}（`elaborat',
'| 4.5.1 | 外观时机 over 的判定条件：最外层、无块、无换形、至少一个属性；没有 `use` 时键 ⊆ {mat, color}，有 `use` 时其余键是 `mat` / `color` 或传给模板的参数（`elaborate` 相同：有 `use` 时其余键都登记为 variant 的参数） |')
line_replace('| 7.2.5、7.8.4 | 皮肤对 `all()`',
'| 7.2.5、7.8.4 | 皮肤对 `all()` / `kind()` / `#tag` / 全局索引 / 事实 / 样式作用范围不可见（原型 `allNodes` 过滤 `skin` 相同）；通配、`children()`、`descendants()`、直接路径只在皮肤模板体内的表达式与编译后的工具查询里看到皮肤（精确清单 7.8.4）。原型第二遍的语义语句能用这四种构件选到皮肤，那是 9.3.2 列出的差异，不是保留项 |')
insert_after_line('| 3.9.4 | `angle` 缺省 $360/(n+1)$',
'''| 3.9.1、4.4.6 | 副本式语句与 copy 的 `of` 不是单个对象路径时报"of 需要写一个对象名"（`buildCopies` / `expandCopies` 相同；`c9/a_of.s3d`） |
| 3.9.3 | 镜像面法线为零报错"镜像面的法线为零"（`mirrorIn` 相同；`c9/a_zero.s3d`） |
| 3.8.1 | `repeat` 的 `n` 必须是常量：引用对象、用 `where` / `kind()` 都报错（原型 `elaborate` 在展开期求值，报"找不到对象 X" / "kind() 只能在求解阶段使用"，`c9/a_n.s3d`、`a_n2.s3d`；本版文案"n 必须是常量"，含义相同） |''')

# ---- 9.4.3 / 9.4.5 ----
rep('**顶层 `meta`**（8.4.3）：键集合、顺序与值相同（数按 $\\tau_{\\text{num}}$）。',
    '**顶层 `meta` 与 `files`**（8.4.3）：`meta` 的键集合、顺序与值相同（数按 $\\tau_{\\text{num}}$）；`files` 条数与顺序相同，每条的 `file`（归一后）、`meta`（同上）、`import` 的有无与其 `file`（归一后）/ `line` / `meta` 相同。')
rep('| N12 | 顶层与条目的 `meta` | 顶层补 `meta: { "lang": 2 }`；`concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`defs` 的条目补 `meta: {}` | 8.4.3 |',
    '| N12 | 顶层的 `meta` 与 `files`、条目的 `meta` | 顶层补 `meta: { "lang": 2 }`；顶层补 `files`：参与编译的每个文件一条 `{ file, meta: { "lang": 2 }, import }`，主文件在前、之后按 import 的先后，主文件的 `import` 为 `null`，被 import 文件的 `import` 为 `{ file, line, meta: {} }`（写它的文件与行；语料里 spaceport_edit 与 spaceport_blueprint 各两条，其余各一条）；`concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`defs` 的条目补 `meta: {}` | 8.4.3 |')

# ---- 9.4.7 ----
rep('脚本（scratchpad 的 `c9/*.py`）对 18 个参考 compile.json 只读',
    '脚本（scratchpad 的 `c9/*.py`；本次复核 `09/stats.py`、`09/stats2.py`、`09/orth.py`）对 17 个参考 compile.json 只读')
rep('| `facts[].gap` 的小数位 | $\\le 4$ 位（`toFixed(4)`），530 条 `on` 全是 0；所以对参考用 $\\tau_{\\text{gap}}^{\\text{ref}}$ |',
    '| `facts[].gap` 的小数位 | $\\le 4$ 位（`toFixed(4)`）；示例的 530 条 `on` 全是 0（exports 里 rotation.L0 的 16 条 `on` 的 gap 非零，仍 $\\le 4$ 位）；所以对参考用 $\\tau_{\\text{gap}}^{\\text{ref}}$ |')
rep('| `tight` ⊆ `box` | 1644 个节点全部成立（容差 $10^{-9}$）',
    '| `tight` ⊆ `box` | 示例里有 `tight` 的全部节点成立（容差 $10^{-9}$）')

# ---- 9.5 ----
rep('对 corpus/examples 的 8 个文件（7 个示例与 lib/semantics.s3d）与 corpus/exports 的 11 个文件逐条语句核查',
    '对 corpus/examples 的 8 个文件（7 个示例与 lib/semantics.s3d）与 corpus/exports 的 10 个文件逐条语句核查')
rep('原型在语料上的实际反应取自参考输出：18 个 compile.json 的 `errors` 与 `warnings` 全部为空',
    '原型在语料上的实际反应取自参考输出：17 个 compile.json 的 `errors` 与 `warnings` 全部为空')
rep('- **旧文件全部可编译**成立：19 个语料文件在本版编译后 `errors` 全部为空',
    '- **旧文件全部可编译**成立：18 个语料文件在本版编译后 `errors` 全部为空')
rep('全部 1644 + 2408 个节点的 `world` / `box` 与全部事实的 `gap` 预期在容差内一致',
    '全部 1644 + 2193 个节点的 `world` / `box` 与全部 1414 + 2408 条事实的 `gap` 预期在容差内一致')
rep('在 11 个 exports 里是 $4.3 \\times 10^{-3}$', '在 10 个 exports 里是 $4.3 \\times 10^{-3}$')

# ---- 9.6 ----
rep('| copy 位置与匿名段计数（4.3.5） | 写在块中间的 `copy` 后面若有匿名兄弟，它们的编号会 +1；引用 `box#2` 之类路径的 over / drop 要相应改 |',
    '| copy 位置与匿名段计数（4.3.5）；匿名模板实例的段（1.6.1） | 写在块中间的 `copy` 后面若有匿名兄弟，它们的编号会 +1；匿名的模板实例从 `group#i` 变成 `模板名#i`。匿名节点不能被表达式寻址（1.6.1），所以只影响按路径读输出的工具，不改文件 |')
insert_after_line('| 硬保留名作名字（1.6.9） | X 是保留名 | 改名 |',
'''| 单引号字符串、`1.`、一元 `+`、非 ASCII 空白、方向组合词矛盾（1.2、1.5.1） | 语法错误 / 无法识别的字符 / 方向组合词矛盾 | 改成 `"x"`、`1.0`、去掉 `+`、用普通空格、改成合法的组合词 |
| 名字不紧跟种类词；重复的属性 / 元数据 / 标签；模板实例带块；`def` 没有块（1.3.3、1.3.4、1.3.15） | 名字要写在种类词之后 / 重复的属性 / X 不能有子语句 / def 需要块 | 把名字移到种类词后；只留后写的那个（原型生效的）；删掉块 / 补 `{ }` |
| 参数名与 P 组属性名、`use`、`mat` / `color` 同名（1.3.4） | 参数名与属性名重名 | 改名 |
| 曲线写定位属性；span / sweep / pipe 写多余的 P 组属性；`alignto` 无 `align`、`aimwith` 无 `aim`（1.4.4） | X 没有属性 k / alignto 需要 align | 删掉（原型里它们本来无效） |
| `repeat` 没写 `n`（1.4.5） | 缺少 n | 写 `n=1`（原型的缺省） |
| span 的 `pin` 不是 bottom / top / center（1.4.5） | …只能是 bottom / top / center | 写 `center`（原型的退路） |
| 除以零、`and` / `or` 作集合运算、`where 1`、无参数的 `kind()`、重复的 `let`（1.5） | 各节文案 | 改成有限数、`\\|` `&`、布尔条件、带参数、唯一的 let |
| 几何参数不合法：非正尺寸、自交或零面积轮廓、`angle` 超范围、`steps` 非整数、互斥的 span / sweep 形态、退化曲线、空布尔容器、非法 `envelope`（2.2–2.6） | 各节文案 | 改成合法值；原型本就生成无定义或退化的几何 |
| `against=B[0,0,0]`（3.3.4） | against 要指明贴哪个面 | 写带法线的锚点（`B.left`） |
| copy 带 P 组与模板参数之外的键（4.3.3） | copy 没有属性 k | 改写成 copy 块里的 `over it k=…` |''')
insert_after_line('| 非轴对齐圆类实体与布尔容器的 `tight`（2.1.8、2.5.4） |',
'''| 名字字面量不再做名字查找（1.4.2）；`where kind == box` 的右侧按字面量（1.5.5） | 场景里有名为 `x` / `y` / `z`、方向词或面名的节点、且依赖原型把 `across=x` 之类解析成该节点的文件，改写成 `across=节点名`（本版 `x` 一律是坐标面） |
| 语义语句对皮肤不可见（7.2.5、7.8.4） | 用 `descendants()` / `children()` / 通配选到皮肤的 check / concept 要复查；本版它们只得到非皮肤节点 |
| `kind(group)` 不再选中阵列实例与副本组（1.6.7） | 要选实例写 `kind(instance)`，要选副本组写 `kind(mirror)` / `kind(rotate)` |
| 副本式节点带原件的标签（4.4.5）；`where kind` 对副本式节点给原件的模板名（7.3.1） | 不改文件；按标签 / 种类选择的语句现在也选中副本 |
| 布尔容器的局部包围盒、曲面锚点、同心球的 `gap`、零长度段的切线（2.5.4、2.4、6.4.2、2.3.1） | 不改文件；语料不涉及，只在这些边界构型下结果不同 |
| 嵌套 over / drop 的先后、皮肤体内的 over / drop / copy、派生皮肤模板的 `w h d`（4.1.2、4.7） | 依赖"块内修改晚于最外层修改"的文件按 4.1.2 的位置重排；皮肤体内的修改与 copy 现在生效 |''')

open(P, 'w', encoding='utf-8', newline='\n').write(s)
print('ok', len(s.split('\n')), 'lines')
