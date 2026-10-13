# 9 兼容性声明

**范围。** 本章规定本版（语言版本 2）与原型（`../SVG3D`，commit 14582a3，参考输出包的来源）之间的兼容关系：承诺什么、不承诺什么（9.1）；本版新增的语法与放宽的限制（9.2）；全部行为变化的清单——每条给出原型行为（括号里注明函数名）、本版行为、理由与对语料的影响（9.3）；两份输出"在容差内一致"的精确定义——与参考输出包比、两份独立实现互比、同一实现跨平台比各用什么容差与比较规则（9.4）；对 corpus/examples 与 corpus/exports 逐文件的影响核查结果（9.5）；旧文件碰到不兼容项时的改法（9.6）。

本章**不**定义任何语言规则：每条行为的定义在它所属的章，本章只引用章节号并汇总。本章也不定义工具层面的东西（输出文件的字节格式、哈希算法、报告格式、`factsSkipped` 的阈值），那些是工具约定。

**依赖的章。** 01 到 08、10 的全部"原型 …；本版 …"陈述与各章末尾"因本章定义而预期不同的项"小节。被依赖：03-solving.md（3.1 节与 3.6.6 引用本章的容差）、10-materials.md（10.6 引用本章的对拍规则）、05-layers.md（5.2.5 的 `file` 字段清单）。

**记法。** "原型"指上述 commit 的 `src/s3d.js`；原型行为的陈述都在括号里给出函数名（`elaborate`、`applyOpinions`、`Solver.local` ……），并经读源码或在原型上编译小例子确认（scratchpad 的 `c9/*.s3d`）。"参考输出"指 corpus/reference/examples/*.compile.json（7 个）与 corpus/reference/exports/*.compile.json（11 个）。"语料"指 corpus/examples/*.s3d（7 个示例与 lib/semantics.s3d）与 corpus/exports/*.s3d（11 个逆向输出）。行为分五类：

| 类 | 含义 |
|---|---|
| A 新增 | 原型不接受（报错或当作别的东西）、本版合法 |
| B 诊断 | 原型静默接受（忽略、猜一个值、或产生不自洽的结果）、本版报错或警告；节点按 01-syntax.md 1.1.4 退化或语句无效 |
| C 语义 | 两版都接受，但结果（节点、变换、事实、集合）不同 |
| D 输出 | 结果相同，只有输出字段的形状、顺序、精度不同 |
| E 保留 | 明确保留的原型行为（写出来是为了不被"看起来像 bug"而改掉） |

## 9.1 兼容性原则与版本号

### 9.1.1 承诺

1. **语法向后兼容。** 原型能解析的每一条语句（01-syntax.md 1.3 节的全部形态）在本版都能解析为同一种语句；本版没有删除任何语句种类、属性或表达式写法，也没有改变任何记号的含义。唯一的词法变化是 `~` 在**不紧跟标识符**时成为运算符（1.2.10）：原型里这个位置的 `~` 是"无法识别的字符"，所以没有任何原型能编译的文件因此改变含义。
2. **旧文件全部可编译。** 编译从不中止（1.1.4 第 7 条），所以"可编译"的含义是：原型编译无错误的文件，在本版编译后 `errors` 仍为空，除非它触发了 9.3.1 表里某一项（那些项在原型里是静默接受的无效写法）。9.5 对全部 19 个语料文件逐一核查：没有一个触发 9.3.1 的任何一项，`errors` 全部为空；4 个文件各多出一条**警告**（软保留名，9.3.4）。
3. **结果在容差内一致。** 对原型编译无错误的文件，本版的 $R$、$t$、包围盒、紧包围盒、事实、概念 / 模式 / 检查的集合与参考输出按 9.4 的规则比较一致，差异只出现在 9.3.2 / 9.3.3 列出的项上；9.4.5 把这些项写成对参考输出的**规范化变换**，变换之后两边必须在容差内相同。
4. **诊断只增不减。** 原型报的每一种错误本版也报（文案可以不同，1.10.1 只要求关键信息）；本版新增的错误与警告只落在原型静默接受的写法上。

### 9.1.2 不承诺

- **向前兼容**：本版新增的写法（9.2）原型不接受。特别是文件头 `~lang=2`：原型对它报"未知的类型 “~lang=2”"（`makeStmt`，经 scratchpad `c9/t_lang.s3d` 确认），其余语句照常编译。要让一个文件在两版都无错误，就不要写文件头（它是可选的，9.1.3）。
- **逐字节相同的输出**：输出字段的形状与精度有变（9.3.3），数值按容差比（9.4）。同一实现在不同平台、不同线程数下的输出必须逐位相同（1.1.3、3.6.6、6.6.3），这是对实现的要求，不是对原型的兼容承诺。
- **诊断文案逐字相同**：文案只须包含各章引号里的关键信息（1.10.1）。
- **工具层**：命令行参数、工程文件、`docs/agent.md` 里的键名约定、渲染结果，不在本规范内。

### 9.1.3 版本号与文件头

- 本规范只定义**一个**语言版本，编号 2；原型是未编号的版本 1。版本由文件头声明：文件里第一个非空、非注释的行若只由元数据项组成，它是文件元数据行，键 `lang` 的值必须求值为 2（`2`、`2.0`、`1+1` 都可以；1.3.1、8.1.5）。
- **没有文件头、或文件头里没有 `lang` 的文件按版本 2 处理**（"缺省等价于 2"）。因此全部旧文件（原型的语料与任何原型工程）无须改动就按本版编译；本章其余各节描述的就是这种情形下的差异。
- `lang` 的值不是 2 时报错"不支持的语言版本"，该文件不编译（主文件：结果没有节点与输出，错误列表只有这一条；被 import 的文件：只有它不编译，仍算已加载，5.1.4）。按**文件**独立判定，不随 import 传递、不与根文件合并。
- `lang` 是**声明**，不切换语义：本版只有一种语义。将来若有版本 3，它对版本 2 文件的处理由那一版的兼容性声明规定；本版遇到 `~lang=3` 一律报"不支持的语言版本"，不猜。
- 编译输出顶层的 `meta` 字段总是带 `lang: 2`（8.4.3），文件头没写时补上；参考输出没有这个字段（9.4.5 的 N11）。
- 原型 `loadProgram` 不检查 `lang`，也不接受文件头（上条）。

## 9.2 新增项

本节是 A 类（原型不接受、本版合法）的完整清单。每项给原型的实际反应（在原型上编译 scratchpad 的小例子确认）与本版定义所在的章节。

### 9.2.1 元数据对所有语句合法

元数据 `~k=v` 在本版对**全部**语句合法（1.9、8.1.2）：写在属性区，与属性、标签的先后任意；`def` 与 `pattern` 写在块之前；`over` 的元数据项写在选择器之后并结束选择器（1.3.8）。原型只对走 `makeStmt` 通用分支的语句接受元数据（节点语句、material、copy、mirror / rotate、repeat、group、span 等），其余各有不同的失败方式：

| 语句 | 原型（函数） | 本版 |
|---|---|---|
| 实体、曲线、容器、模板实例、repeat、mirror / rotate、copy、span、sweep、pipe | 接受，写进节点的 `meta`（`makeStmt` 通用分支） | 相同（8.3） |
| `material` | 接受但**不输出**（`makeStmt` 存进 `st.meta`，`elaborate` 的材质表不读它） | 接受，随 `materials` 条目输出 `meta`（8.4.3、10.1.1） |
| `let` | 报错"无法识别的字符 “~”"（值整体交给 `parseValue`） | 接受，记录在语句上，不进输出（8.3.8） |
| `def` | 报错"def 语法应为 …"（`makeStmt` 的 def 正则不匹配） | 接受，随 `defs` 条目输出 |
| `import` | 报错"import 写成：import "文件.s3d""（`makeStmt` 只接受一个字符串） | 接受，随语句记录、不传给被导入文件（5.1.4） |
| `over` | 报错"over：属性写完之后不能再出现选择器 “~k=1”"（写在属性后）或"无法识别的字符 “~”"（写在选择器后） | 接受；`~k=` 结束选择器（1.3.8） |
| `drop`、`concept`、`check` | 报错"无法识别的字符 “~”" | 接受（concept、check 随条目输出） |
| `pattern` | 报错"pattern 写成：pattern 名字(角色, …) { 约束 }"（签名正则不匹配） | 接受，写在块之前 |
| `style` | 报错"style 目前只支持 mat 和 color，不认识 ~k" | 接受，不进输出 |
| `variant` | 报错"模板 T 没有参数 ~k"——**对每个宿主各报一次**（`compileOnce` 挂载时） | 接受，随 `variants` 条目输出 |
| 文件头（只含元数据项的首行） | 报错"未知的类型 “~lang=2”" | 文件元数据（1.3.1、8.1.5） |

元数据的求值、类型与两条保证见 08-metadata.md；语料里的元数据全部写在节点语句上（corpus/exports 的 `~fit` / `~src`），原型与本版都接受。

### 9.2.2 where 的正则谓词与 `mat` 字段

`where` 里新增（1.7.3、1.5.5、8.2.2、10.5.3）：

| 写法 | 原型 | 本版 |
|---|---|---|
| `name ~ /正则/`、`kind ~ /正则/` | 词法错误"无法识别的字符 “~”"（`exprTokens`；`c9/t_regex.s3d` 确认），整条语句丢弃 | 正则匹配（搜索语义；正则子集见 1.2.9） |
| `<元数据键> ~ /正则/`（含 `src ~ /re/`） | 同上 | 同上；成员没有该键时为缺失值，谓词为假（8.2.3） |
| `mat == 材质名`、`mat ~ /正则/` | `evalAst` 的 where 帧没有 `mat`，`mat` 当作找不到的名字 → 条件恒假、选中空集（`c9/t_mat.s3d`：无错误、无效果） | `mat` 是成员在几何时机的有效材质名（10.3.2）；静态量，几何时机的选择器也可用 |

正则字面量只在运算符 `~` 之后出现，其他位置的 `/` 仍是除号（1.2.9）；`~k=v` 仍是元数据（1.2.10）。两者在词法上由"`~` 后面是否紧跟标识符字符"区分，所以不会改变任何原型文件的含义。

### 9.2.3 文件头 `~lang=2`

见 9.1.3。文件头可以带其他键（`~generator="…"`，8.1.5），它们按文件记录并输出到顶层 `meta`，不属于任何节点。

### 9.2.4 其他放宽

下列写法在原型里报错、被忽略或没有定义，本版合法（各条的定义在所注章节）：

| 节 | 写法 | 原型（函数） | 本版 |
|---|---|---|---|
| 1.3.10、7.5.1 | pattern 约束行的关系词 `intersects` | 关系词表没有它，`a intersects b` 被当作布尔表达式解析而失败（`makePattern`） | 七个谓词名都是关系词 |
| 1.4.2、7.7.1、10.5.1 | `style … mat="steel"`、`variant … use="T"`（字符串写名字） | style 不接受字符串（`compileOnce` 的 style 分支按符号取名）；节点上的 `mat="steel"` 原型已接受 | 名字字面量类型一律接受内容相同的字符串（便于经 let / 参数传递） |
| 10.1.2 | `material` 的属性值引用最外层 `let` | `let` 对 material 不可见（`elaborate` 在全局帧建好前登记材质） | 可见（全局帧） |
| 4.4.4 | `over R across=… / axis=… / about=… / angle=… / mat=… / color=…`，R 是副本式节点 | 对 R 上任何几何时机的 over 报"副本跟随原物体"（`applyOpinions`） | 允许就地修改 R 自己的这几个键；定位朝向键、模板参数、换形仍报错 |
| 4.3.5 | over 块里的 `copy` | 登记进已处理完的 copy 表而静默丢失（`applyOpinions`） | 在展开该 over 块时立即展开，`of` 对那一刻的树解析 |
| 4.3.4 | `copy R of=阵列 { … }` 的块 | 块里的语句直接挂在 repeat 节点下，不属于任何实例（`expandCopies`） | 块等价于 `over R { }`：对每个实例各展开一份，块帧含 `i j k idx` |
| 4.5.5 | `over 阵列.3 { … }`（成员是单个阵列实例）的块帧 | 只有 `it` | 另有该实例的 `i j k idx` |
| 4.1.3、4.5.4 | 先 `over x turn=45` 再 `over x n=2`（就地修改后再重建） | 重建用源语句的原始属性，`turn` 丢失（`applyOpinions` 读 `m.stmt.props`） | 按有效语句重建，`turn` 保留 |
| 1.5.6 | `kind("box")`（字符串参数） | 只接受裸标识符 | 接受名字字面量或字符串 |
| 6.5.3 | mesh 外包络、布尔容器代理盒的贴面 | 不是盒类，只能得到 `touching`（`boxy` 要求凸块精确） | 盒类，可得到 `against`，事实标 `approx: true`（C 类，9.3.2 另列） |
| 6.5.4 | 近似的 $B$（revolve、mesh、布尔容器）作 `inside` 的容器 | 不判 `inside`（`insideTest` 要求 `convex`） | 判 `inside` 并标 approx（C 类，9.3.2 另列） |
| 7.8.9 | 无效的 variant 语句 | 没有输出条目 | 有条目，`members: []` 带 `error` |

## 9.3 行为变化清单

本节汇总全部 10 章里以"原型 …；本版 …"标出的差异与各章"因本章定义而预期不同的项"小节，按类别分表。每行：定义所在的节、情形、原型行为（函数）、本版行为、理由（引用已定的规则所在节）、语料里是否命中（9.5 给文件与行号）。同一件事在多章提到的只列一次，节号列出主要的一处。

### 9.3.1 诊断：原型静默接受、本版报错或语句无效（B 类）

本表每一项在原型里都是**无效写法被静默接受**（忽略、猜一个值、或产生不自洽的结果）。本版的恢复按 1.1.4：属性错误与求解错误使节点退化（$T = \mathbb 1$、零盒、不参与事实、子节点照常），定义类与修改类语句出错则整条无效。

**结构、命名、层**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 1.6.6、5.3.2 | 同级重名（同一父节点下两个具名子节点段相同；跨文件的最外层同名亦然） | `elaborate` 不检查，产生两个同路径的节点（`c9/t_misc.s3d`：`g.a` 两次，无错误） | 报错"同级重名 X"，报在后出现的语句上，后者连同块不生成节点，引用得到先出现的 | D18；路径必须唯一地标识节点（1.6.10） | 无 |
| 1.8、5.2.3 | 同一文件第二次 import | `loadProgram` 只有加载链 `stack`，不在链上的已加载文件再次 import 时静默重复内联（13 个节点、路径两两重复） | 报错"文件 X 已经 import 过"，不导入；不做 include-once | D19；重复内联必然造成同级重名并压掉中间的定义 | 无（每个文件至多一条 import） |
| 5.2.3 | import 成环且根文件在环上 | 链不含根文件，a → b → a 时 a 的语句被读入两次才报错 | 根文件始终在链上，第一次重复即报"import 形成了循环"，不重复任何语句 | 5.1.3 等价原则 | 无 |
| 5.2.2、1.8 | import 路径 `..` 越出根目录；路径用 `\` | `joinPath` 保留多出的 `..`、接受 `\` | 分别报错"import 路径越出根目录"、"import 路径用 / 分隔" | 1.8 已定 | 无 |
| 5.2.1 | 块内的 import | 报"未知的类型 import" | 报"import 只能写在最外层"（位置错误） | 1.3.15 | 无 |
| 10.1.1、5.3.4、1.3.7 | 块内、模板体内的 `material` | `elaborate` 的 `one` 静默忽略（`c9/t_misc.s3d`：`group g { material inner … }`，无错误、材质表里没有 inner） | 报错"material 只能写在最外层"，语句无效（不登记） | D6 | 无 |
| 10.1.2 | `material` 写了五个分量之外的键 | 接受任意键并原样输出（`rust: {color, extra: 5}`） | 报错"material 没有属性 k"，该键忽略，材质仍登记 | D17；附加信息走元数据 | 无 |
| 10.1.2 | `default=false`、`default=1` | 值按字符串存，`'false'` 为真——`default=false` 的材质成为默认材质（`c9/t_misc.s3d`：没写 `mat` 的节点拿到了它） | `default` 只接受布尔字面量，`false` 为假；其他值报错"default 只能是 true / false"并当作假 | 1.4.3 | 无（语料没有 default） |
| 10.1.2 | `metal` / `rough` / `opacity` 不在 $[0,1]$ 内；`color` 不是合法颜色 | 原样输出 | 报错，该分量取缺省值 | 1.4.6 | 无 |
| 10.2、1.4.1 | 节点的 `color=#wall`（井号后不是 3 / 4 / 6 / 8 位十六进制） | 不报错，条目输出 `color: "[object Object]"` | 报错"颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa"，节点退化；颜色位置**不查标签** | D5 | 无（语料颜色全是 6 位十六进制） |
| 1.6.5、1.4.2 | 找不到的名字（裸标识符、路径首段） | 部分上下文退化成字符串，只在当作引用使用时（`Solver.ref`）才报"找不到对象" | 在解析名字那一步报错"找不到对象 X"（或"名字 X 不唯一"） | D4 | 无 |
| 8.1.3、1.9 | 元数据值引用对象、`#tag`、用 `where` / `all()` | 退化成字符串或在求值时失败 | 报错"元数据不能引用对象 X"等，该键不存在，节点不退化 | D4、8.2.1 | 无（exports 的值全是数与字符串） |
| 4.2.1 | 模板名与内置种类同名、是关键字或保留名、与 concept / pattern 同名 | `elaborate` 登记 def 时不检查 | 报错"模板不能与内置种类同名"等，整条 def 无效 | D27 | 无 |
| 1.3.4、10.2 | 模板参数叫 `mat` / `color` | 参数吞掉实例语句上的 `mat=` | def 处报错 | 外观属性不能被参数遮蔽 | 无 |
| 4.2.6、5.3.3 | `def X : X { … }` 而 X 尚无定义 | 报错但仍登记为只有扩展体的定义 | 报错"要扩展的类型 X 还没有定义"，不登记 | 1.1.4 第 4 条 | 无 |
| 4.2.5 | 派生签名里同一参数既在覆盖表又重新声明 | 先施加覆盖再施加参数表，自己的声明静默赢 | 报错"参数 k 既被覆盖又被重新声明"，该 def 无效 | 两种写法含义冲突 | 无 |
| 5.3.5、7.2.2、1.3.9 | concept、pattern、最外层 let 异种同名 | pattern 静默压过同名 concept（`semantic.setLazy` 覆盖） | 报错"名字 X 已经用作 concept / pattern / let"，报在后出现的语句上，该语句无效 | 三者共用语义帧与全局帧的查找链 | 无 |
| 1.6.9 | 名字用了硬保留名（`it kind name mat idx i j k`、`mirrored`、`skin`、`instance`、关键字……） | 不检查 | 报错"X 是保留名，不能用作 …"，语句丢弃 | 1.6.9 | 无 |

**定位与朝向（第 3 章）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 3.3.8、3.1.4 | 写了多个定位关系（`on` 与 `at` 同写等） | `on` 优先于其他，多写的静默忽略（`relation`） | 报错"定位关系只能写一个"，节点退化 | 1.4.4 | 无 |
| 3.4.2 | `gap` 写在法线为零的关系上（`inside`、`at=` 向量 / 集合 / 无锚点引用 / 曲线取点、没有关系） | 静默忽略（`Solver.local`：`t += gap·nrm`，`nrm = 0` 无效果） | 报错"gap 需要有法线的定位关系（on / under / against，或 at= 带面锚点）"，节点退化 | 1.1.1：不得静默接受无效写法 | 无（语料的 gap 都在 `on` 上） |
| 3.5.3 | `rot` 的轴是零向量 | `axisAngle(unit(0), θ)` 得 $cI$（非正交矩阵），不报错 | 报错"rot 的轴不能是零向量"，节点退化 | 结果无定义 | 无 |
| 3.5.4 | `aimwith` 不是方向词 / 组合词 | 退回缺省 `front`（`Solver.sym`） | 报错"aimwith 需要方向词"，节点退化 | 类型规则 1.4.1 | 无（语料只有 `aimwith=top`） |
| 3.6.2 | 定位 / 点属性引用自己的祖先 | `Solver.ref` 只检查自己与后代；祖先引用经 `Solver.box` 的 `Cycle` 捕获成为后置并得到一个解 | 报错"不能相对包含自己的对象定位"，节点退化（皮肤对宿主的引用由 3.7.6 规定，不受限） | 容器的盒由子节点决定，引用没有不循环的含义 | 无 |
| 3.6.5 | 定位依赖成环 | `compileOnce` 的 `fail`：只把先碰到忙标记的节点退化并报一条，其余按退化值求解（`box x on=y; box y on=x` 只有 x 退化） | 去掉后置边后的图上每个环上的节点都退化、各报一条 | 错误局部化与确定性 | 无 |
| 3.8.1 | `repeat` 的 `n` 非整数或负数 | `Math.max(0, Math.round(x))`（`elaborate`：2.4 → 2 个、−1 → 0 个） | 分量必须在 $\varepsilon_{\text{int}} = 10^{-9}$ 内是非负整数，否则报错"n 需要非负整数"；`n=0` 仍合法 | 不猜 | 无 |
| 3.8.1 | `n` 引用对象或用 `where` | 在求解期求值 | 报错"n 必须是常量" | 1.4.1 | 无 |
| 3.8.4、3.8.6 | `along` 与 `step` / `center` 同写；`orient` 没有 `along`；`orient=true/false` | `instLocal` 忽略 step / center；orient 只在 along 下读取；true / false 当 yaw / none | 三者都报错；`orient` 只接受 `yaw` / `full` / `none` | 互斥与枚举规则 1.4.3 | 无 |
| 3.9.1 | 副本式 `mirror` / `rotate … of=` 写定位 / 朝向属性；`rotate … of=` 写 `n` | `Solver.local` 的 `n.src` 分支直接 `copyLocal`，忽略 at / turn / pin …；`rotationIn` 对 `n.src` 取 count = 1，忽略 `n` | 报错"副本跟随原物体，不接受定位属性" / "rotate 写了 of= 就不能再写 n"，节点退化 | 1.4.4、1.4.5 | 无 |
| 3.9.2、4.4.1 | 容器式 `rotate` 的 `n` 非整数、不在 1..360、引用对象 | `Math.round(S.num(n))` 后检查范围（`buildCopies`：2.6 → 3 个副本组），可引用求解期的量 | 报错"rotate 的 n 需要 1..360 的整数" / "rotate 的 n 必须是常量"，不生成副本组 | 不猜；常量 | 无 |
| 3.9.3 | `across=x/y/z` 时同时写 `axis` | `mirrorIn` 对字符串 across 直接取坐标轴，axis 被忽略 | 报错"across 是轴名时不能再写 axis"，节点退化 | 互斥 | 无 |
| 3.9.5 | 方向规格（`axis` / `dir`）求得零向量；镜像面法线为零 | `unit` 对零向量返回零向量，后续矩阵无定义 | 报错"不能是零向量" / "镜像面的法线为零"，节点退化 | 结果无定义 | 无 |

**派生与修改（第 4 章）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 4.5.1 | 没有属性也没有块的 `over 选择器` | 静默接受为空操作（`elaborate` / `applyOpinions`；`c9/t_misc.s3d` 确认） | 报错"over 需要属性或块"，语句无效 | 没有含义的语句不该静默 | 无 |
| 4.6.1 | 容器 → 实体的保名换形 | 交给重新展开，因"box 不能有子语句"失败而使节点**消失**（`applyOpinions`） | 换形前拒绝：报错"不能把容器 X 换成 T：它的子物体会丢失，先 drop 再新建"，该成员不变 | D11 | 无 |
| 4.6.2 | 模板 → 模板换形，参数不在目标参数表 | 保留全部参数，重新展开时报"模板 X 没有参数 k"且节点退化重建 | 派生链上保留目标参数表里有的；否则按名字匹配，多余的报错"模板 T 没有参数 k（换形自 X）"，该成员不被修改 | D11 | 无 |
| 4.4.4 | 对影子写 `repeat` 的 `n` | 不拦截（孪生体与原件共享子节点，结果不自洽） | 报错"X 是副本，不能单独改阵列数量：请改用 copy" | 副本是原件的函数（4.4.2） | 无 |
| 4.4.6 | `of=` 不是单个对象路径 | 各种失败 | 报错"of 需要写一个对象名" | 1.4.5 | 无 |

**层与两个时机（第 5 章）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 5.4.2、1.7.4 | 几何时机的选择器读 `w h d x y z top bottom` | `applyOpinions` 不报错；`evalAst` 的 where 分支在求解器上按需求盒并缓存，之后的修改对已缓存的盒不再生效（`box a size=1,3,1` 后 `over all() where h > 2 size=2,2,2`：无错误、a 仍是 1×3×1） | 报错"几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）"，整条语句无效；判定是静态的（求值前检查表达式树），不依赖成员集合 | D10；原型的结果依赖缓存状态，没有可判定的定义 | 无 |
| 5.4.2、1.5.6 | 几何时机的选择器调用关系谓词 | 报"over 改的是几何，在求解之前生效，不能按空间事实选择（只改外观的 over 可以）" | 报"X() 依赖求解后的空间事实，不能用来决定位置 / 几何"；语句无效 | 1.5.6 的文案；含义相同 | 无 |
| 5.4.2 | 几何时机的选择器写概念名 / 模式名 / 未知名字 | 未解析的名字当字符串，报"需要集合（对象、#标签、kind(…)、列表…），得到 X" | 概念或模式名 → 时机错误；否则"找不到对象 X" | 1.6.5：找不到的名字不退化成字符串 | 无 |
| 5.4.2、1.7.1 | 外观时机的 `over` 选择集为空 | `compileOnce` 的样式 / 变体循环不报（`over #nothing color=#00ff00` 静默） | 报错"X 没有选中任何对象"，语句无效；`style` / `variant` 仍不报（这是外观时机 over 与 style 的唯一差别） | 1.7.1 | 无 |
| 5.4.2 | 几何时机的选择器读到了用了不可用构件的 `let` | 无专门规则（随 `evalAst` 的行为） | 报一条时机错误，归属该 let 语句，读它的 over / drop 整条无效、不另报；外观时机读同一 let 不受限，两个时机的缓存分开 | 5.2.5 的归属 | 无 |

**语义层（第 7 章）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 7.2.3 | 带锚点 / 偏移的引用当集合（`a.top`、`a + (0,1,0)`） | `asSet` 取其节点、丢掉锚点 | 报错"X 不是集合（锚点与偏移不能当集合）" | 写了锚点说明要的不是集合 | 无 |
| 7.2.4、1.5.6 | 关系谓词的第二个参数 $k$ 不是整数 $\ge 1$ | `relSet` 接受任何数（`on(ground, 0)` 选中索引里有任何 on 项的全部节点；1.5 等价于 2） | 报错"X() 的第二个参数需要不小于 1 的整数"，语句无效 | 1.5.6 | 无（语料的 $k$ 是 2 与 4） |
| 7.4.2 | `concept N : U` 的 U 不是概念名 | `makeStmt` 把 U 当普通路径求值，可以是对象（`concept E : ground = where …` 得到 {ground}） | 报错"上位概念 U 没有定义"，语句无效 | `parent` 字段的含义；层级是概念之间的 | 无（lib 的 U 都是概念） |
| 7.5.1 | 同一角色两条 `in` 约束 | `makePattern` 后一条覆盖前一条 | 报错"角色 X 的 in 约束重复"，pattern 无效 | 覆盖没有理由 | 无 |
| 7.5.2 | 域引用了未绑定 / 更晚的角色；关系约束里集合角色不止一个或不单独成一侧 | 匹配时才发现：未绑定名字退化成字符串，报"需要集合（…），得到 y"；集合角色规则相同但在匹配时报 | 登记时静态报错"角色 X 的 in 不能引用 Y（只能引用更早声明的标量角色）" / "一条关系约束里最多一个集合角色…"，pattern 无效 | 1.6.5；错误更早、可判定 | 无 |
| 7.5.3 | 布尔约束的值不是布尔 | `asBool`：非空集合与非零数为真 | 报错"模式 X 的约束需要真假值，得到 Y"，模式失败 | 1.5.5 | 无 |
| 7.6.1 | `check` 的结果是数或字符串 | `asBool`：非零数、`"true"` 为真（`check "c" count(kind(box))` 通过，`c9/t_misc.s3d`） | 只接受布尔与集合类，其他报错"check 的结果需要真假值或集合，得到 X"，`ok: false` | 没有确定含义 | 无（语料的 check 都是比较或集合） |
| 7.6.2 | `check` 的求值错误 | 只写进 `checks[].error`，不进 `errors[]` | 两处都记，归属 check 语句 | 1.1.4 第 4 条："有没有错误"只看 `errors[]` | 无 |
| 7.8.1 | `use=` 的模板不存在 | 挂载时对**每个宿主**报"variant 的模板 T 没有定义"，宿主不隐藏 | 登记时在语句上报一次，语句无效，`variants[]` 条目带 `error` | 静态可判定 | 无 |
| 7.8.1 | variant 的键不是模板参数 | 挂载时报"模板 X 没有参数 k"，仍挂载、宿主已隐藏 | 静态报错，语句无效 | 1.3.13、1.1.4 第 4 条 | 无 |
| 7.8.1 | 模板必填参数未给（且不是 `w` / `h` / `d`） | 挂载时报"模板 X 缺少参数 q"，宿主隐藏、皮肤退化为零盒——物体从画面消失 | 静态报错，语句无效，宿主不隐藏 | 错误不应使物体消失 | 无 |
| 7.8.1、1.3.13 | variant 上的 `at` / `pin` | 静默用 `at=宿主底面 pin=bottom` 覆盖（`compileOnce`） | 报错"变体的位置由外包络决定"，语句无效 | 1.3.13 | 无 |
| 7.8.6 | 宿主是布尔容器的操作数 | 操作数标 hidden 但 `buildItems.single` 不看 hidden：操作数仍参与布尔并渲染，皮肤既不渲染也不出条目 | 该成员不挂载，报错"X 是布尔容器的操作数，不能挂变体：请对整个布尔容器挂"；其他成员照常 | 原型行为不是任何可取的定义 | 无 |

**错误的范围（两版都报错，但本版只影响出错的节点或语句）**

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 10.2、1.1.4 第 6 条 | `mat=` 引用未定义的材质 | `materialOf` 抛 S3DError，`buildItems` 中断，一个未定义材质清空**全部**输出条目 | 报错"未定义的材质 X"，该节点按没写 `mat` 处理、不退化，其他节点与条目不受影响 | D16 | 无 |
| 4.4.2、1.1.4 第 6 条 | 副本依赖成环（`mirror a of=b` 与 `mirror b of=a`） | `buildCopies` 的 `ensure` 抛错后停止**全部**后续副本生成 | 只有环上的副本根报错"复制关系形成循环：X"并退化，其他副本根照常 | D16 | 无 |
| 7.8.3 第 7 条 | 一个皮肤体内的错误 | 挂载循环的 try / catch 按宿主隔离（相同） | 相同，写成规则 | — | 无 |

### 9.3.2 语义：两版都接受，结果不同（C 类）

| 节 | 情形 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 3.6.3 | 后置（`late`）的判定 | `Solver.box` 在惰性求值中用忙标记 `localBusy` / `Cycle(e.node === n)` 顺带标记，结果依赖求值顺序 | 在显式依赖图上按"从 $\{L(c), X(c)\}$ 可达 $X(A)$"判定，容器按节点顺序逆序、子节点按节点顺序处理 | 确定性（1.1.3） | 语料全部 1644 个节点与原型一致（3.11.3）；只在一个节点同时属于多个嵌套循环的罕见情形可能不同 |
| 4.3.5 | copy 节点在兄弟中的位置与匿名段计数 | `expandCopies` 把 R 追加在父节点全部兄弟之后，且不计入匿名段计数（`group g { box p; copy q of=p; box }` → `box#2`） | R 占 copy 语句的位置，阶段 3 的占位计入匿名计数（→ `box#3`） | 1.6.1 已定：节点顺序 = 源码顺序 | 无影响：语料里 copy 都是所在块的最后一条（semantics.s3d:143、spaceport_edit.s3d:13） |
| 4.3.6、8.3.5 | copy 语句带元数据 | 有元数据时**整体替换**原语句的元数据（`expandCopies`：`s.meta?.size ? s.meta : A.stmt.meta`） | 逐键合并，copy 自己的覆盖同名键 | 8.3.5 已定 | 无（语料的 copy 没有元数据） |
| 4.6.2 | 容器 → 容器的保名换形 | 保留源语句的块（`over gg : group { box j }` 得到 inner 与 j） | 丢弃源语句的子语句，只有 over 块里写的 | D11（PLAN 第 3 节第 4 条） | 无（语料没有换形） |
| 4.1.3、4.5.4 | 重建型修改前对该节点自身的就地修改 | 丢弃（用 `m.stmt.props` 重建） | 保留（按有效语句重建） | 就地修改是节点状态的一部分 | 无 |
| 6.2.2、6.5.1 | 一方是布尔容器的事实的 `approx` | `localParts` 对没有 `geom` 的节点返回 `approx: false`，事实不标 approx | 布尔容器的代理盒按 2.5.6 是 `approx=true`，事实输出 `approx: true` | 2.5.6；近似几何由 approx 标出（6.2.2） | **命中** 19 条：basics 2、spaceport 5、spaceport_blueprint 5、spaceport_edit 7（9.5） |
| 6.5.4 | `inside` 的容器 $B$ 是近似的单凸块（revolve、mesh、布尔容器、凹截面扫掠段） | `insideTest` 要求 `B.convex`（精确单块）；revolve / mesh 作 $B$ 时不判 inside，得 `intersects` / `touching` | $B$ 只需恰有一个凸块（可近似），成立时输出 `inside` 并标 `approx: true`，gap 不变 | 与 2.5.6 对布尔容器的要求一致；近似几何参与全部判定 | **命中**：rotation 的 `radar.feed intersects radar.dish` → `inside`（1 条，6.8.3）；exports 里 mesh 作 $B$ 的 `intersects` 预计有 1 + 1 + 13 + 249 + 11 × 4 = 308 条改为 `inside`（9.5.3，按参考盒预测，M1 逐条确认） |
| 6.5.3 | mesh 外包络、布尔容器代理盒的贴面 | `boxy` 要求凸块精确，mesh 只能得到 `touching`（approx） | 盒类，可得到 `against`，事实标 `approx: true` | 贴面是外包络层面的陈述 | 示例无；exports 预计 1 + 2 + 4 + 6 + 8 + 8 + 8 = 37 条 `touching` 改为 `against`（9.5.3） |
| 6.2.3、6.4.3 | 对象盒 / 块盒恰好相距 $\varepsilon$ | `broadPairs` 的 y / z 用 `> -margin`（排除恰好相等）、x 用 `>=`，`gapBetween` 用 `< -eps` 跳过（保留恰好相等），三处不一致 | 三处统一为重叠量 $\ge -\varepsilon$ 保留 | 边界只按一条不等式（2.1.4） | 无（只在恰好相等时不同：`box exact on=ground gap=0.02` 原型无事实、本版 `touching` 0.02） |
| 10.3.2 | 节点或样式只写 `color` | `color` 把整个材质换成只有颜色的匿名材质（metal、rough 回到缺省、name 丢失；条目输出 `{color}`） | 只覆盖 color 分量，name / metal / rough / opacity 保留（`where mat == steel` 仍为真） | 10.3.2 | **命中** 3 个条目：spaceport_blueprint 的 `wings.annex`、`wings.mirrored.annex`（concrete 改色）、spaceport_edit 的 `hangar2.roof`（steel 改色）；变换、包围盒、事实不变 |
| 10.3.1、10.4.1 | 副本式 `mirror R of=A mat=…`，$A$ 是容器 | $R$ 的 `mat` 只对原件是实体时生效，对容器原件的影子被忽略 | $R$ 的 `mat` / `color` 插在原件 $A$ 之前，对沿链到 $A$ 之间没有自有外观属性的影子生效 | 统一规则 | 无（语料的副本式语句没有外观属性） |
| 10.3.3 | 同一条样式里 `mat` 写在 `color` 之后 | `mat` 丢掉 `color` | `mat` 先于 `color` 应用，与书写顺序无关 | 10.3.3 | 无 |
| 10.3.3 | 直接选中副本的样式与原件的样式 | 直接选中副本的无条件压过原件的（不看顺序） | 按语句顺序一起应用，后写的赢 | 7.7.3 | 无（两者不同时出现） |
| 10.1.3、5.4.1 | 外观层新增或删除材质**名字** | 无区分 | `material` 属外观层，但材质名的集合对几何时机可见（`mat=` 未定义报错、`where mat` 是静态量），所以新增 / 删除名字可以改变几何时机的选择器与错误；只重定义已有名字不改阶段 1–7 的任何量 | 5.4.4 的保证按此表述 | spaceport_blueprint 只重定义已有名字，不受影响 |
| 2.2.3（经 6.8.3） | 圆锥凸块的顶点 | 半径 $10^{-4}$ 的小环 | 一个顶点 | 精确 | rotation 涉及 cone 的 6 条事实的 gap 差 $\le 10^{-4}$，在 9.4 的容差内 |
| 2.2.9（经 6.8.3） | 凹截面扫掠段的凸块 | 直接用截面环（非凸） | 取凸包并标 `approx` | 凸块必须是凸的 | 无（语料的截面都是凸的） |
| 2.1.6.2（经 6.8.3） | 凹轮廓 plate 的三角化 | `triangulate2` | 耳切（2.1.6.2），可能切出不同的三角形 | 定义唯一的构造 | 无（语料的事实不涉及 plate） |

### 9.3.3 输出：结果相同，形状、顺序或精度不同（D 类）

| 节 | 字段 | 原型（函数） | 本版 | 理由 | 语料 |
|---|---|---|---|---|---|
| 4.2.7、1.6.7 | 副本式节点 R 的 `kind` | 组合串 `mirror→group`、`mirror→span`、`rotate→group`…（`compileOnce`） | 原件的种类（truss / span / wedge / box / revolve / group） | 组合串不是任何种类名，`where kind == …` 无法匹配 | **命中** 15 个节点：curves 3、rotation 1、semantics 5、spaceport 系 2 × 3；exports 共 39 个（9.5） |
| 4.2.7 | 副本式节点与影子的 `types` | `null`（`buildCopies` 不复制 `types`，`kind()` 内部经 `n.src.types` 查） | 原件的类型链 | `kind(truss)` 在原型里已经选中 tower2，输出应与之一致 | **命中**：spaceport 系三个文件的 `tower2`（`["truss"]`） |
| 4.4.2、1.6.7 | 影子的 `kind`（原件是阵列实例或副本组） | shadow 建成 `group` 节点，输出 `group`（`tower2.legs.0`、`gate2.halves.mirrored`） | `instance` / `mirror` / `rotate`（影子种类 = 原件种类） | 否则 `kind(instance)` 漏掉副本里的实例 | **命中**：curves 1、spaceport 12、spaceport_blueprint 12、spaceport_edit 14；exports 共 102 |
| 4.3.2、5.2.5 | copy 节点的 `file` / `line` | 源语句的（`expandCopies`） | copy 语句自己的；子节点仍归各自源语句 | 节点的名字与位置来自 copy 语句 | **命中** 2 个节点：spaceport_edit 的 `hangar2`（参考 spaceport.s3d:68 → spaceport_edit.s3d:13）、semantics 的 `watch2`（参考 :62 → :143） |
| 4.7、5.2.5、7.8.3 | 皮肤节点 `skin` 的 `file` | `null`（`compileOnce` 建皮肤语句时不带 file） | 变体语句的文件与行 | 不允许没有文件归属的输出 | **命中** 3 个节点：semantics 的 `slab.skin`（:130）、`street.h4.skin`（:131）、spaceport_blueprint 的 `hangar.roof.skin`（:19） |
| 5.2.5、1.10.1 | 错误的 `file` | 根文件的解析错误、style / variant / 几何 over 的选择器错误、事实提供者错误的 `file` 为空 | 一律带文件 | 1.10.1 | 无（语料无错误） |
| 6.6.1 | `facts[].gap` 的精度 | 四舍五入到 4 位小数（`+gap.toFixed(4)`） | 完整双精度 | 小单位场景下 4 位不够；确定性按逐位 | 全部 1414 + 2408 条事实（对拍用 9.4.2 的 $\tau_{\text{gap}}^{\text{ref}}$） |
| 6.6.1 | `facts[].approx` | 只在为真时出现 | 总是输出布尔 | 形状固定 | 全部事实（规范化 N6） |
| 6.6.1、6.5.6 | 事实的输出顺序与 $a$ / $b$ 的平局 | 按 `broadPairs` 的扫描顺序产出、`boxVol(P) <= boxVol(Q) ? [P,Q] : [Q,P]`（未成文） | 写成可判定规则：候选对 $(\sigma(P), \sigma(Q))$ 的字典序；平局 $a$ = 扫描序靠后者 | 确定性 | 参考输出 1414 条零处违反、13 对平局一致（6.8.3）——相同 |
| 7.2.3、1.5.5 | 集合的成员顺序（概念成员、关系谓词结果、并集） | `evalAst` 的 `or` 按操作数拼接、`relSet` 按索引插入顺序（`lamp`：4 pole 后 4 bulb；`tree`：12 trunk 后 12 crown） | 一律节点顺序 | D15 | **命中** semantics 的 `lamp`、`tree` 两个概念的 `members` 顺序；成员集合相同 |
| 7.4.4 | `nodes[].concepts` 标注 | 不去重、模式内按绑定 Map 的顺序追加 | 去重；模式按输出顺序、角色按声明顺序，再概念按输出顺序 | 可判定 | 无（语料 267 个节点逐字相同） |
| 7.2.2、5.3.5 | 同名 `pattern` 的输出 | 两条都输出、匹配取最后一条 | 只输出最终定义，位置是最终定义在 $P$ 里的位置 | 与 concept 一致 | 无 |
| 7.5.8 | pattern 失败文案 | 约束行号写进文案（"第 31 行：…"） | 文案不含行号，行在 `line` 字段 | 1.10.1 | 无 |
| 7.5.6 | 匹配数达到 500 | 静默截断 | 保留前 500 个并**警告** | 截断的结果不是真值 | 无（语料匹配 ≤ 4） |
| 7.8.2、7.8.9 | `variants[].members` 的顺序；无效 variant 的条目 | 路径字符串排序；只为登记成功的语句输出 | 节点顺序；每条通过文法检查的 variant / 外观 over-use 都有条目 | D15；与 concept / pattern 一致 | 无（语料每条至多一个成员） |
| 7.6.3、8.4.3 | `checks[]`、`concepts[]`、`patterns[]`、`variants[]`、`materials`、`defs` 的条目 | 没有 `meta` | 各带 `meta`（没有元数据时 `{}`） | 8.4.3 | 全部条目（规范化 N10、N11） |
| 8.4.3 | 编译输出顶层 | 没有 `meta` | `meta: { lang: 2, … }` 总是存在 | 8.4.3 | 全部文件（规范化 N11） |
| 10.6 | `items[].material` | 只输出写了的分量（`{name, color, metal?, rough?}`），颜色覆盖时 `{color}` | 统一五键 `{name, color, metal, rough, opacity}`，无名材质 `name: ""` | 形状固定 | 全部条目（规范化 N9） |
| 10.6、10.1.3 | `materials` | 只输出写了的键 | 五个分量与 `default` 总是都有 | 形状固定 | 全部文件（规范化 N10） |
| 7.1.1 | 编译遍数 | 有变体的文件编译两遍（第二遍挂皮肤后重算一切） | 一遍、两个阶段；因皮肤对阶段一不可见，结果逐位相同（7.1.1 的保证清单） | 实现自由 | 无差别 |

### 9.3.4 新增的警告（F 类）

警告不影响结果（1.1.1）。本版定义而原型没有的警告：

| 节 | 警告 | 触发 | 语料 |
|---|---|---|---|
| 1.6.9 | "节点名 X 与保留名相同，不能按路径引用" | 节点名是软保留名（语句种类词、方向词、`w h d x y z`、`start mid end side surface origin`、内置函数名与常量、关系词） | **命中** 4 处：basics.s3d:8 与 basics.L0.s3d:8 的 `box floor`（`floor` 是内置函数名）、curves.s3d:54 与 curves.L0.s3d:38 的 `box back`（方向词）。这些节点作为路径**首段**仍可引用（1.6.5 第 5 步："首段不受后缀保留词的限制"），basics.s3d 里的 `on=floor[…]` 照常解析；只是不能作为后续段（`g.floor`）引用 |
| 1.6.5 第 4 步、8.2.2 | "没有节点带元数据 X" | `where` 里读了整个场景没有任何节点带的元数据键 | 无 |
| 8.1.4 | "元数据键 X 与 where 字段同名，在 where 里读不到"；"lang 只在文件头有意义" | 元数据键叫 `w h d x y z top bottom kind name mat it`；节点语句上写 `~lang=` | 无 |
| 1.5.6 | "kind() 不认识 X" | `kind()` 的参数既不是内置种类词也不是已定义模板 | 无（lib/semantics.s3d 的 `kind(…)` 列出的 15 个都是内置种类词） |
| 6.6.4、6.9 | "空间事实未推导，X() 为空集" | `factsSkipped` 为真时每条调用了关系谓词的语句一条 | 无（参考输出 `factsSkipped` 全为 false） |
| 7.5.6 | "模式 X 的匹配超过 500 个，只保留前 500 个" | 匹配数达到上限 | 无 |
| 2.4.1 | 网格曲面锚点的近似 | `mesh.top[u,w]` 在没有源网格时用外包络近似 | 无 |

原型唯一的警告来源（`compileOnce` 对 `kind()` 未知名字之外的情形）在语料上也为空；参考输出 18 个文件的 `warnings` 全部为 `[]`。

### 9.3.5 明确保留的原型行为（E 类）

下列行为看起来像可以"修正"，但各章已把它们定为规则，语料依赖它们；实现不得改动：

| 节 | 行为 |
|---|---|
| 3.4.4 | `offset` 在**父框架**里加，不随节点自身的旋转、不随参照物旋转 |
| 3.3.4、3.4.3 | `against` 自动选面只在没写 `pin` 时；`align` 的面名沿目标面法线、轴名对盒心 |
| 3.5.1、3.5.7 | 朝向从参照物继承；参照物是镜像时 $R_{\text{base}} = R_{B \to P}\,\mathrm{diag}(-1,1,1)$ |
| 3.7（D13） | 容器按包络定位（没有 `envelope` 时包络 = 子节点盒之并），事实与渲染按内容 |
| 3.7.6、4.7 | 变体皮肤放在宿主盒底面中心（等价于注入 `at=宿主.bottom pin=bottom`），总是后置 |
| 3.8.1 | 实例总数上限 $N_{\max} = 5000$ |
| 3.9.4 | `angle` 缺省 $360/(n+1)$；`about` 引用取盒心；`axis` 按使用处的框架 |
| 2.1.4、6.1.3 | 容差常数不随 `metre` 换算，唯一例外是接触容差 $\varepsilon = 0.02 \cdot \mathtt{metre}$ |
| 2.2.3 | 12 边内接棱柱是圆柱在事实里的几何，`approx=false` |
| 4.4.4 | 修改的两轮施加（阶段 5 失败或为空的推迟到副本树生成后再试一次）；`kind()` / `#tag` / `all()` 只改原件、影子跟随 |
| 4.5.2 | 块内 over / drop 的作用范围；范围外的被选中节点被排除、不报错 |
| 4.6.2 | 保名换形保留 定位与朝向 ∪ {mat, color}，专有属性一律不带过去 |
| 6.5.2 | `on` 在 $B$ 的局部系里判定；倒扣的 $A$ 也算 `on`；水平重叠严格大于 $\varepsilon$ |
| 6.6.1、6.5.6 | 事实的输出顺序与 $a$ / $b$ 的平局规则（写成规则，结果与原型相同） |
| 6.3.2 | 请求不存在的提供者报错并用缺省列表；提供者出错只丢它的事实 |
| 7.2.4 | 关系索引的上提：容器因其任一部件满足关系而满足关系，但不跨过公共祖先；`on` / `against` / `intersects` 都派生 `touching` |
| 7.4.3 | 概念互相引用时按忙标记报"X 循环引用了自己"，失败不缓存、每条语句各报一次 |
| 7.5.4、7.5.5 | 模式匹配算法（步数计法、去重键、集合角色的确定方式）、步数上限 200000 |
| 7.8.2 | 变体"后写的赢"与祖先裁决 |
| 10.3（D8） | 材质继承顺序 style > color > mat > 父级 > default > 内置；任何作用于节点或其祖先的样式压过节点自己的 `mat` / `color` |
| 10.3.1、10.4.1 | 副本式节点与影子走**原件**的材质链，$R$ 所在位置的祖先不参与；`group G mat=rust { mirror R of=A }` 的 rust 对 $R$ 无效 |
| 10.1.4 | 内置缺省分量 metal 0.05、rough 0.78、opacity 1、color `#c9c9c9` |

## 9.4 对拍容差与比较规则

03-solving.md 开头要求"两份独立实现按本章得到的 $R$、$t$ 与包围盒必须在 09-compat.md 规定的容差内一致"；PLAN 第 9 节的 M1 验收要求"事实在参考输出包的定义容差内一致"。本节给出这些比较的精确定义。三种比较的要求不同：

| 比较 | 对象 | 要求 |
|---|---|---|
| **跨平台**：同一实现、同一输入，在不同操作系统 / 架构 / 线程数 / wasm 上 | 全部输出 | **逐位相同**（1.1.3、3.6.6、6.6.3；PLAN 原则 18）。不用本节的容差；工具以哈希核对 |
| **互比**：两份独立实现（例如几何内核与渲染器、或两种语言的编译器）对同一输入 | 9.4.3 的键 | 数值在 9.4.2 的容差内，离散量相同；差异只允许出现在 9.4.4 的边界对上 |
| **对参考**：实现对 corpus/reference 的 compile.json | 9.4.3 的键 | 先按 9.4.5 规范化参考输出，再按互比的规则比；9.3.2 / 9.3.3 列出、9.5 逐条给出的"预期不同"项在规范化里消化 |

"在容差内一致"不是"差不多"：下面每条规则都是可判定的——给定两份输出，比较程序的结果唯一。

### 9.4.1 比较的单位与场景尺度

- 全部长度量用**场景单位**（文件里数字的单位，1.1.3），不换算成米；`metre` 只影响 $\varepsilon$（6.1.3）。
- **场景尺度** $L$：被比较的两份输出里全部非皮肤节点的世界包围盒 `box` 之并的对角线长，取两份里的较大者（空场景 $L = 0$）。语料的 $L$ 在 48（basics）到 289（spaceport 系）之间；实战工程以 stud 为单位时 $L$ 可达 $10^4$。

### 9.4.2 容差常数

| 名字 | 值 | 用于 |
|---|---|---|
| $\tau_{\text{len}}$ | $\max(10^{-6},\ 10^{-9} L)$ 场景单位 | 平移 $t$、`box` / `tight` 的六个数、`gap`、`eps`、`items[].geom` 的长度参数、`curves[].pts`、`items[].world.t` |
| $\tau_{\text{rot}}$ | $10^{-6}$（无量纲） | 旋转矩阵 $R$ 的九个元素（`world.R`、`items[].world.R`） |
| $\tau_{\text{gap}}^{\text{ref}}$ | $0.5 \times 10^{-4} + \tau_{\text{len}}$ | 只在**对参考**时用于 `gap`：参考输出的 `gap` 被四舍五入到 4 位小数（原型 `+gap.toFixed(4)`，9.3.3），与真值的差不超过半个单位 |
| $\tau_{\text{num}}$ | $10^{-9} \cdot \max(1, \lvert a \rvert, \lvert b \rvert)$ | 无量纲的其他数：元数据里的数、`items[].geom` 的角度与无量纲参数、`materials` 的 metal / rough / opacity |

数 $a$、$b$ 在容差 $\tau$ 内一致当且仅当 $\lvert a - b \rvert \le \tau$；两边都是 `null` 也算一致；一边 `null` 一边是数不一致。向量、矩阵、盒逐分量比。

**理由与余量。** 容差要同时满足：远大于两份正确实现之间的计算噪声，远小于任何会改变判定或几何的量。

- 噪声：第 3 章用参考输出的 $W_P$ 与本版的 $T_n$ 复合后与参考的 $W_n$ 比，1644 个节点的最大偏差在 $10^{-12}$ 量级（3.11.1；来源是 $T_{B \to P}$ 的两种计算顺序）；参考输出里 $R^{\mathsf T} R - I$ 的最大元素 $8.9 \times 10^{-16}$（9.4.7）。不同 libm 的 sin / cos 相差 1 ulp（$\sim 10^{-16}$ 相对），经十几层复合与 $10^{4}$ 量级的坐标放大后不超过 $10^{-11}$。$\tau_{\text{len}}$ 的 $10^{-9} L$ 项就是为 $L \sim 10^4$ 的场景留这个余量。
- 判定：事实的判定量与阈值 $\pm\varepsilon$（0.02）的最小距离，在 7 个示例里是 $2 \times 10^{-4}$（rotation 的过山车枕木 `coaster.ties.82.tie touching coaster.railR` gap $-0.0195$ 与 `…78.tie intersects coaster.railL` gap $-0.0202$，9.4.7），在 11 个 exports 里是 $4.3 \times 10^{-3}$；都比 $\tau_{\text{len}}$ 大两个量级以上，所以语料里没有一条事实会因容差内的差异改变判定（9.4.4 的边界对在语料里不出现）。
- 几何：语料里最薄的实体是 0.05（curves 的 `canal`、semantics 的 `river`）；$\tau_{\text{len}}$ 比它小四个量级。

实现**不得**为了通过比较而放宽这些常数；实现之间若出现超过容差的差异，是其中至少一方不符合规范（或规范有缺陷，1.1.1 最后一条）。

### 9.4.3 比较的键与规则

两份输出按下表逐项比较；"相同"指 JSON 值相等（字符串按码点、布尔与 `null` 按值、数组按长度与逐项），"容差"指 9.4.2。字段的定义在所注章节；本节不定义任何字段。

**节点 `nodes[]`**（1.6.1 的节点顺序；两份的节点序列按 `path` 一一对应，顺序必须相同）

| 字段 | 规则 | 定义 |
|---|---|---|
| `path` | 相同；序列长度与顺序相同 | 1.6.1、1.6.10 |
| `kind`、`types`、`tags`、`id`、`depth`、`late`、`ghost`、`of`、`hidden`、`skin`、`curve` | 相同 | 1.6.7、3.6.4、4.4、7.8 |
| `line`、`file` | 相同（`file` 按 9.4.5 N1 归一后比） | 1.3.16、5.2.5 |
| `world.R` | 九个元素各在 $\tau_{\text{rot}}$ 内；$\det R$ 的符号相同 | 3.1.1 |
| `world.t` | 三个分量各在 $\tau_{\text{len}}$ 内 | 3.1.1 |
| `box`、`tight` | `min` / `max` 六个数各在 $\tau_{\text{len}}$ 内；`null` 只与 `null` 一致 | 3.1.6、2.1.8、2.6 |
| `meta` | 键集合与键顺序相同；数在 $\tau_{\text{num}}$ 内，其他按值 | 8.4.1 |
| `concepts` | 相同（列表，含顺序） | 7.4.4 |

**事实 `facts[]`**（6.6.1）

| 项 | 规则 |
|---|---|
| 条数与顺序 | 相同（6.6.1 的顺序是定义的一部分）；两份按位置一一对应 |
| `rel`、`a`、`b`、`provider`、`approx` | 相同 |
| `text` | 相同（`against` 含面名） |
| `gap` | 互比：$\tau_{\text{len}}$；对参考：$\tau_{\text{gap}}^{\text{ref}}$ |
| `eps` | $\tau_{\text{len}}$ |
| `factsSkipped`、`factProviders` | 相同 |

事实的差异分三种，比较报告必须区分：(a) 两边都有、字段不同；(b) 一边有一边没有；(c) 边界对（9.4.4）。只有 (c) 不算失败。

**语义层**（7.4.4、7.5.8、7.6.3、7.8.9）

| 项 | 规则 |
|---|---|
| `concepts[]` | 条数与顺序相同；每条的 `name`、`parent`、`line`、`file`、`src`、`members`（含顺序）、`error` 的有无相同；`meta` 同节点 |
| `patterns[]` | 条数与顺序相同；`name`、`roles`、`matches`（匹配列表含顺序；每个匹配按角色名取值，标量角色是路径、集合角色是路径列表含顺序）、`error` 的有无相同 |
| `checks[]` | 条数与顺序相同；`message`、`ok`、`violators`（含顺序）、`error` 的有无相同 |
| `variants[]` | 条数与顺序相同；`use`、`src`、`members`（含顺序）、`error` 的有无相同 |
| `defs` | 相同（模板名列表，含顺序；条目的其他字段由 04-derivation.md 定义后按"相同"比） |
| 错误文案 | `error` 字段与 `errors[]` 的文案不逐字比（1.10.1），只比有无与归属 |

**材质与条目**（10.6）

| 项 | 规则 |
|---|---|
| `materials` | 键集合与顺序相同；每个条目的 `color`、`default` 相同，`metal`、`rough`、`opacity` 在 $\tau_{\text{num}}$ 内 |
| `items[]` | 条数相同；按 `path`（布尔容器条目再按操作数顺序）一一对应；`kind`、`geom.type`、`tags` 相同；`geom` 的长度参数在 $\tau_{\text{len}}$ 内、角度与无量纲参数在 $\tau_{\text{num}}$ 内、点列表逐点 $\tau_{\text{len}}$；`world` 同节点；`material` 的 `name`、`color` 相同，三个数在 $\tau_{\text{num}}$ 内 |
| `curves[]` | 条数相同，按 `path` 对应；`closed` 相同；`pts` 点数相同、逐点 $\tau_{\text{len}}$（采样点数由 2.3.1 定义） |

**诊断**（1.1.5、1.10）

| 项 | 规则 |
|---|---|
| `errors[]`、`warnings[]` | 条数相同；按 1.1.5 的顺序一一对应；每条的 `file`（归一后）、`line` 相同；文案含同样的关键信息（各章引号里的部分），不逐字比 |

**顶层 `meta`**（8.4.3）：键集合、顺序与值相同（数按 $\tau_{\text{num}}$）。

### 9.4.4 边界对

事实是对连续量做阈值判定的结果（6.5）。两份都正确的实现，其判定量可以相差一个计算噪声；若某个判定量恰好落在阈值附近，两边可能得出不同的事实。本节把这种情况从"失败"里分出来，但要求它被报告：

- 候选对 $(P, Q)$（6.2.3）的**判定量**是 6.5 各条不等式左右两边之差：$\mathrm{gap}(P,Q)$ 与 $\varepsilon$、$-\varepsilon$ 的差；`on` 的 $\lvert \beta_A - \tau_j \rvert$ 与 $\varepsilon$ 的差、水平重叠量与 $\varepsilon$ 的差、$\big\lvert\lvert (R_{A\to B})_{yy}\rvert - 1\big\rvert$ 与 $\varepsilon_{\text{dir}}$ 的差；`against` 的面距与 $\varepsilon$ 的差、$o_y$ / $o_x$ / $o_z$ 与 $\varepsilon$ 的差、轴对齐判定的元素与 $\varepsilon_{\text{dir}}$ 的差；`inside` 的外溢量与 $\varepsilon$ 的差、$V_A - \kappa_{\text{in}} V_B$；候选对与块对粗筛（6.2.3、6.4.3）的重叠量与 $-\varepsilon$ 的差。
- 一个候选对是**边界对**，当且仅当在任一份输出的实现里，它的某个判定量的绝对值 $\le \tau_{\text{len}}$（无量纲的判定量用 $\tau_{\text{rot}}$）。
- 边界对上的事实差异（有无、`rel`、面名、$a$ / $b$ 顺序）**不算失败**，但比较报告必须列出每一个边界对与两边的判定量。其他任何事实差异都是失败。
- 实现**应**能按请求输出候选对的判定量（工具约定，形式不在本规范内）；不能输出时，比较程序无法把差异归为边界对，一律按失败处理。
- 语料里没有边界对：全部 3822 条参考事实的 $\lvert\,\lvert\mathrm{gap}\rvert - \varepsilon\,\rvert \ge 2 \times 10^{-4} \gg \tau_{\text{len}}$（9.4.7），其他判定量的最小裕度由 6.8.3 的逐条一致间接保证（任何一条落在 $10^{-6}$ 以内都会在原型与本章独立实现之间表现为差异，而 6.8.3 的 1413 条无一差异）。

同样的原则适用于其他由阈值决定的离散量：后置 `late`（3.6.3，按依赖图判定，无阈值，必须相同）、`inside` 的体积比、曲线采样点数（2.3.1 按长度与常数算出，是整数，两份必须相同；长度恰在取整边界 $\pm \tau_{\text{len}}$ 内时按边界对处理并报告）、`repeat` 的 $n$（$\varepsilon_{\text{int}}$ 判定，常量表达式，必须相同）。

### 9.4.5 参考输出的规范化

对参考（corpus/reference）比较前，对**参考输出**做下列变换，使 9.3.2 / 9.3.3 里"预期不同"的项消失；变换只作用于参考侧，不改实现的输出。每条给出可机械执行的规则；语料里受影响的条目在 9.5 逐一列出，比较程序应核对规范化实际改动的条目恰是那些。

| 编号 | 字段 | 变换 | 依据 |
|---|---|---|---|
| N1 | 全部 `file`（节点、concepts、patterns、checks、variants、errors、warnings） | 参考里是相对原型仓库根的路径（`examples/spaceport.s3d`、`out/basics.L0.s3d`）；去掉第一段目录，只留文件名；实现侧按宿主给出的可显示路径同样取文件名后比 | 1.8、1.10.1 |
| N2 | `nodes[].kind` 为组合串 `X→Y` 的节点（副本式 mirror / rotate） | 改为 `of` 所指节点规范化后的 `kind` | 1.6.7、4.2.7 |
| N3 | `nodes[].kind` 为 `group` 且 `ghost: true`、`of` 所指节点的 `kind` 不是 `group` 的节点（阵列实例与副本组的影子） | 改为 `of` 所指节点规范化后的 `kind`（`instance`、`mirror`、`rotate`）；阵列实例在参考里的 `kind` 是 `instance`，不改 | 1.6.7、4.4.2 |
| N4 | `nodes[].types` 为 `null` 且 `kind` 含 `→` 或 `ghost: true` 的节点 | 改为 `of` 所指节点的 `types` | 4.2.7 |
| N5 | copy 节点（由 copy 语句生成、`kind` 不含 `→`、`ghost: false`、路径最后一段是 copy 语句的名字）的 `file` / `line` | 改为 copy 语句的文件与行。参考输出里没有字段能机械地识别 copy 节点，所以按 9.5 的清单改：semantics 的 `watch2` → semantics.s3d:143，spaceport_edit 的 `hangar2` → spaceport_edit.s3d:13 | 4.3.2 第 7 条 |
| N6 | 皮肤节点（`skin: true` 且路径最后一段是 `skin`）的 `file` 为 `null` | 改为该变体语句的 `file`（`line` 已是变体语句的行）：取 `variants[]` 里 `line` 相同的条目的 `file` | 7.8.3、5.2.5 |
| N7 | `facts[]` 缺 `approx` 的条目 | 补 `approx: false`；再把 `a` 或 `b` 的 `kind` 是 `union` / `subtract` / `intersect` 的条目改为 `approx: true` | 6.6.1、6.5.1、2.5.6 |
| N8 | `facts[]` 的 `rel` | 按 6.5.3 / 6.5.4 预期改变的条目改 `rel`（`against` 还要改 `text` 并加面名）：示例里只有 rotation 的 `radar.feed intersects radar.dish` → `inside`（6.8.3）；exports 的清单在 9.5.3，标"预测"，M1 用实现跑出后核对并回填 | 6.5.3、6.5.4 |
| N9 | `facts[].gap` | 不改值；比较用 $\tau_{\text{gap}}^{\text{ref}}$ | 6.6.1 |
| N10 | `concepts[].members` | 按参考 `nodes[]` 的顺序重排（语料里只有 semantics 的 `lamp`、`tree` 会变） | 7.2.3 |
| N11 | `items[].material`、`materials` | 条目补 `name: ""`（缺失时）、`metal: 0.05`、`rough: 0.78`、`opacity: 1`（缺失时）；`materials` 的条目补同样的缺省与 `default: false`、`meta: {}`；只含 `color` 的条目（语料 3 个）按 10.6 改成"基础材质改色"：`wings.annex`、`wings.mirrored.annex` → name concrete、metal 0.05、rough 0.9；`hangar2.roof` → name steel、metal 0.6、rough 0.4 | 10.6、10.3.2 |
| N12 | 顶层与条目的 `meta` | 顶层补 `meta: { "lang": 2 }`；`concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`defs` 的条目补 `meta: {}` | 8.4.3 |
| N13 | `warnings[]` | 追加 9.3.4 预期的警告（语料 4 条：basics 与 basics.L0 的 `floor`、curves 与 curves.L0 的 `back`），`file` / `line` 按 9.5 | 1.6.9 |
| N14 | `variants[].members` | 按节点顺序重排（语料不变） | 7.8.2 |

规范化之后仍存在的差异都是失败（或 9.4.4 的边界对）。N8 的 exports 部分是**预测**（由参考输出的盒与变换按 6.5.3 / 6.5.4 推算，脚本见 9.5.1），不是定义：M1 的实现跑出结果后，每条预测要么被确认、要么作为规范或实现的缺陷查明原因，不得静默改成实现的输出。

### 9.4.6 验收判据与报告

- **通过**：按 9.4.3 的全部项，规范化后的参考与实现输出没有失败项；边界对（9.4.4）全部列出。
- **报告**必须按文件列出：规范化实际改动的条目数（与 9.5 的清单核对）、每个失败项（字段、路径、两边的值、差值）、每个边界对（判定量）、`errors` / `warnings` 两边的全文。"不一致逐条列出并解释"（PLAN 第 9 节）指的就是这份报告。
- 比较是**对称**的：互比两份实现时任何一份都可以当"参考"，结果相同（容差对称、离散量相等关系对称）。

### 9.4.7 用参考输出验证

脚本（scratchpad 的 `c9/*.py`）对 18 个参考 compile.json 只读 `nodes[]`、`facts[]`、`items[]`，核对本节用到的事实：

| 核对项 | 结果 |
|---|---|
| 数值精度：`world` / `box` / `tight` 的数字是否是完整双精度 | 是：小数位最多 17–20 位（`repr` 的最短往返表示），没有四舍五入；所以对这些字段用 $\tau_{\text{len}}$ 不另加舍入项 |
| `facts[].gap` 的小数位 | $\le 4$ 位（`toFixed(4)`），530 条 `on` 全是 0；所以对参考用 $\tau_{\text{gap}}^{\text{ref}}$ |
| 旋转矩阵的正交性 $\max \lvert (R^{\mathsf T} R - I)_{ij} \rvert$ | 7 个示例分别 $4.4 \times 10^{-16}$、$1.1 \times 10^{-16}$、$8.9 \times 10^{-16}$、0、$4.4 \times 10^{-16}$ ×3；$\det R \in \{+1, -1\}$（到 $10^{-6}$）；$\tau_{\text{rot}} = 10^{-6}$ 比噪声大 9 个量级 |
| 坐标量级与 $L$ | 示例 $\max \lvert t \rvert, \lvert\text{box}\rvert$ 为 20（basics）到 100（spaceport 系）；$L$ = 48 到 289；$10^{-9} L < 10^{-6}$，所以语料用 $\tau_{\text{len}} = 10^{-6}$ |
| `tight` ⊆ `box` | 1644 个节点全部成立（容差 $10^{-9}$）；轴对齐节点（$R$ 是带符号置换）里 `tight` 与 `box` 逐位相同的比例：basics 14/14、rotation 27/27、spaceport_edit 81/81、curves 34/38、semantics 136/145、spaceport 67/68（不相同的是 revolve、sweep、span 杆、球等按真实形状算紧盒的实体）——与 2.1.8"轴对齐时多面体类的 $T$ 与局部盒的变换一致"相符 |
| 事实判定量距阈值的最小裕度 $\min \lvert\,\lvert\mathrm{gap}\rvert - \varepsilon\,\rvert$（touching / intersects） | 示例：basics 0.02、curves 0.02、rotation $2.0 \times 10^{-4}$、semantics 0.193、spaceport 系 0.0182；exports：最小 $4.3 \times 10^{-3}$（rotation.L0），其余 $\ge 0.011$。没有一条在 $10^{-3}$ 以内除 rotation 的 4 条枕木事实（$-0.0195$、$-0.0202$、$-0.0206$、$-0.0197$），它们与阈值差 $\ge 2 \times 10^{-4}$ |
| 第 3 章的独立复算（3.11.1） | 1644 个节点的 `world` 与 `box` 偏差 $\le 10^{-12}$ 量级 |
| 第 6 章的独立复算（6.8.3） | 1413 条事实 `gap` 偏差 $< 10^{-4}$（受参考舍入限制）、关系与面全部一致 |
| 5.4.4 的逐位保证 | spaceport 与 spaceport_blueprint 的 193 个非皮肤节点、203 条事实逐项相同（5.5.3） |

这些数字说明 9.4.2 的常数在语料上既不会误报（噪声 $\ll \tau$）也不会漏报（判定裕度与最小几何 $\gg \tau$）。本节无法用参考输出验证的只有 $L \sim 10^4$ 的场景（实战工程不在 git 里）与 9.4.4 的边界对处理（语料里没有边界对）。

## 9.5 语料影响核查结果

### 9.5.1 方法

对 corpus/examples 的 8 个文件（7 个示例与 lib/semantics.s3d）与 corpus/exports 的 11 个文件逐条语句核查 9.3 的每一项是否命中，用三种手段：

1. **源码扫描**（scratchpad `c9/scan.py`）：按块结构解析每条语句，检查同级重名、五个定位关系键的个数、块内 `material`、元数据值是否引用名字、`gap` 所在关系是否有法线、`rot` 零轴、`repeat` 的 `n` 与 `along` / `orient` 组合、副本式语句的定位键与 `n`、`across` 轴名加 `axis`、`aimwith` 的值、`over` 是否为空、换形、几何时机选择器里的求解量、`concept : U`、`variant` 的 `at` / `pin`、`check` 的结果种类、关系谓词的第二个参数、pattern 的 `intersects`、import 条数、模板名与种类词、参数名 `mat` / `color`、节点名是否为保留名。
2. **参考输出扫描**（`c9/*.py`，9.4.7）：从 compile.json 取 `kind` 含 `→` 的节点、影子的 `kind`、`types` 为 `null` 的副本、`file` 为 `null` 的节点、只含 `color` 的材质条目、一方是布尔容器的事实、概念成员的顺序、`errors` / `warnings`。
3. **事实预测**（`c9/predict.py`）：对每条一方是 mesh 的 `touching` / `intersects` 事实，用参考输出的 `world`、`items[].geom` 与 `tight`，按 6.5.3（盒类、轴对齐、面距 $< \varepsilon$、$o_y > \varepsilon$ 与侧向重叠 $> \varepsilon$）与 6.5.4（$A$ 的角点在 $B$ 局部盒 $\pm\varepsilon$ 内、$V_A < 0.9\,V_B$）判定本版会得到什么。$A$ 是 box / wedge / mesh 时用局部盒的 8 个角经 $W_A$ 变换（精确），其他实体用紧盒的 8 个角（保守：可能漏报、不会多报）。revolve 作 $B$ 的情形由 6.8.3 的独立实现给出（1 条）。

原型在语料上的实际反应取自参考输出：18 个 compile.json 的 `errors` 与 `warnings` 全部为空（MANIFEST：examples 与 exports 的编译 errors / warnings 均为 0）。

### 9.5.2 逐文件结果

**corpus/examples**

| 文件 | 命中的项（节；位置） | 本版的编译结果 |
|---|---|---|
| basics.s3d | 1.6.9 软保留名：`box floor`（:8）→ 警告；6.5.1 布尔容器事实 `approx`：`crate on floor`、`orb inside crate` → `approx: true` | errors 0、warnings 1；18 个节点的 `world` / `box` / `tight` 与 20 条事实的 (rel, a, b, gap) 不变 |
| curves.s3d | 1.6.9：`box back`（:54）→ 警告；4.2.7 副本式 `kind`：`stepsE`（`mirror→span` → `span`，:39）、`tower2`（`mirror→revolve` → `revolve`，:45）、`gate2`（`rotate→group` → `group`，:71）；4.4.2 影子 `kind`：`gate2.halves.mirrored`（`group` → `mirror`） | errors 0、warnings 1；94 个节点的几何与 69 条事实不变 |
| rotation.s3d | 4.2.7：`sat.wing2`（`mirror→box` → `box`，:30）；6.5.4：`radar.feed intersects radar.dish` → `radar.feed inside radar.dish`（gap $-2.8$、approx 不变；`radar.horn`、`radar.mast` 仍 `intersects`，6.8.3） | errors 0、warnings 0；615 个节点不变；562 条事实里 1 条 `rel` 不同 |
| semantics.s3d | 4.2.7：`rampE`（`mirror→span` → `span`，:33）、`street.h1.half2` … `street.h4.half2`（`mirror→wedge` → `wedge`，:47 经模板）；7.2.3 成员顺序：`lamp`（:82）、`tree`（:88）的 `members` 改为节点顺序，集合相同；4.3.2 copy 节点 `file` / `line`：`watch2`（参考 semantics.s3d:62 → :143）；7.8.3 皮肤 `file`：`slab.skin`（:130）、`street.h4.skin`（:131）从 `null` → semantics.s3d | errors 0、warnings 0；267 个节点的几何、118 条事实、2 个模式、5 条检查的结果不变 |
| spaceport.s3d | 4.2.7：`tower2`（`mirror→group` → `truss`，`types` `null` → `["truss"]`，:47）、`arm2`（`mirror→span` → `span`，:48）；4.4.2 影子：`tower2.legs.0..3`、`tower2.decks.0..7`（`group` → `instance`，12 个）；6.5.1：`hangar.body on ground`、`hangar.roof on hangar.body`、`spare inside hangar.body`、`wings.annex against hangar.body.right`、`wings.mirrored.annex against hangar.body.left` → `approx: true`（5 条） | errors 0、warnings 0；193 个节点、203 条事实的 (rel, a, b, gap) 不变 |
| spaceport_blueprint.s3d | 同 spaceport 的全部项，另：7.8.3 皮肤 `file`：`hangar.roof.skin`（:19）→ spaceport_blueprint.s3d；10.3.2：`wings.annex`、`wings.mirrored.annex` 的条目材质从 `{color: #cfe3f7}` → concrete 改色（name concrete、metal 0.05、rough 0.9、opacity 1）；10.1.3：7 条 `material` 重定义已有名字，材质名集合不变、几何时机不受影响 | errors 0、warnings 0；240 个节点、203 条事实不变 |
| spaceport_edit.s3d | 同 spaceport 的 `tower2` / `arm2` 项；影子 `kind` 14 个（`tower2.legs.0..3`、`tower2.decks.0..9`）；4.3.2：`hangar2`（参考 spaceport.s3d:68 → spaceport_edit.s3d:13；`hangar2` 的子节点仍归 spaceport.s3d）；6.5.1：7 条布尔容器事实（spaceport 的 5 条加 `hangar2.body on ground`、`hangar2.roof on hangar2.body`）；10.3.2：`hangar2.roof` 的条目材质从 `{color: #4a5560}` → steel 改色（metal 0.6、rough 0.4）；4.3.5：`copy hangar2` 是文件的最后一条语句，位置规则不改变节点顺序 | errors 0、warnings 0；217 个节点、239 条事实不变 |
| lib/semantics.s3d | 无命中：`concept platform : slab`（:17）、`column : member`（:23）、`beam : member`（:24）的上位都是概念（7.4.2）；`kind(…)` 列出的 15 个名字都是内置种类词（1.5.6 不警告）；`let solid_kinds`（:11）与各概念名不重名；没有节点语句 | 单独编译：errors 0、warnings 0、没有节点；概念全部为空集（7.1.4） |

**corpus/exports**（原型逆向的输出，只用语言的子集：实体、`mesh`、`repeat`、容器式与副本式 `rotate` / `mirror`、`material`、`~fit` / `~src` 元数据；没有 def、copy、over、语义语句、变体）

| 文件 | 命中的项 | 本版的编译结果 |
|---|---|---|
| basics.L0.s3d | 1.6.9：`box floor`（:8）→ 警告；6.5.4 预测：`orb intersects crate` → `inside`（crate 是 mesh，approx）；6.5.3 预测：`stairs touching table` → `stairs against table.left`（stairs 是 mesh，approx） | errors 0、warnings 1；16 个节点不变；20 条事实里 2 条预测改变 |
| curves.L0.s3d | 1.6.9：`box back`（:38）→ 警告；6.5.3 预测：`deck touching stepsW` → `against stepsW.right`、`deck touching stepsE` → `against stepsE.left`；6.5.4 预测：`lamps_2.r8.bulb intersects stepsE` → `inside` | errors 0、warnings 1；106 个节点不变；87 条事实里 3 条预测改变 |
| l2fb.s3d | 4.2.7：`volume_4`（`mirror→mesh` → `mesh`）、`stairs_2`、`stairs_4`（→ `span`）、`columns_3`、`slabs_2`（`mirror→repeat` → `repeat`）、`slab_4`（→ `box`）；4.4.2 影子：12 个（`columns_3.0..3`、`slabs_2.0..7`，→ `instance`）；6.5.4 预测 13 条 `intersects` → `inside`（`stairs_3` / `stairs_4` / `tank` 对 `volume_1`，`steps.4/8/9/10/13/14/15/19/20/21` 对 `volume_7`）；6.5.3 预测 4 条：`walls.wall` 与三个副本对 `column_2` 的 `touching` → `against`（right / left / back / front） | errors 0、warnings 0；170 个节点不变；212 条事实里 17 条预测改变 |
| rotation.L0.s3d | 6.5.4 预测 249 条 `intersects` → `inside`：`radar_horn`、`radar_feed` 对 `radar_dish`；247 条 `coaster_ties.*.tie` 对 `coaster_railL` / `coaster_railR` / `coaster_spine`（三条轨道是 mesh，枕木落在它们的外包络里） | errors 0、warnings 0；608 个节点不变；709 条事实里 249 条预测改变 |
| semantics.L0.s3d | 4.2.7：`street_h1_half2`、`street_h3_half2`（`mirror→wedge` → `wedge`） | errors 0、warnings 0；249 个节点、128 条事实不变 |
| spaceport.L0.s3d | 4.2.7：`wings_mirrored_annex`、`arm2`（→ `box`）、`wings_mirrored_stair1`、`wings_mirrored_stair2`（`mirror→mesh` → `mesh`）、`tower2_legs`、`tower2_decks`（→ `repeat`）；影子 12 个；6.5.3 预测 6 条（`wings_annex` / `wings_mirrored_annex` 对 `hangar_body`、`wings_stair1` 对 `wings_annex`、`wings_mirrored_stair1` 对 `wings_mirrored_annex`、`wings_stair2` / `wings_mirrored_stair2` 对 `hangar_roof`）；6.5.4 预测 11 条（`spare` 对 `hangar_body`；`spiral_treads.*.tread` 10 条对 `spiral_rail`） | errors 0、warnings 0；177 个节点不变；222 条事实里 17 条预测改变 |
| spaceport.L1.s3d | 4.2.7 与影子同 L0 的形态（`volume_3`、`mesh_2`、`columns_3`、`slabs_2` 等 6 个副本式、12 个影子）；6.5.3 预测 8 条；6.5.4 预测 11 条（`tank_1` 对 `volume_1`；`steps.*.step` 10 条对 `volume_6`） | errors 0、warnings 0；177 个节点不变；227 条事实里 19 条预测改变 |
| spaceport.L2.s3d | 4.2.7：4 个副本式（`mirror→repeat` 3、`mirror→box` 1）；影子 28 个；没有 mesh | errors 0、warnings 0；272 个节点、277 条事实不变 |
| spaceport_edit.L0.s3d | 4.2.7：9 个副本式；影子 24 个；6.5.3 预测 8 条（spaceport.L0 的 6 条加 `feed1 touching pad`、`feed2 touching pad` → `against pad.front`）；6.5.4 预测 11 条 | errors 0、warnings 0；222 个节点不变；263 条事实里 19 条预测改变 |
| spaceport_edit.L1.s3d | 4.2.7：6 个副本式；影子 14 个；6.5.3 预测 8 条；6.5.4 预测 11 条 | errors 0、warnings 0；196 个节点不变；263 条事实里 19 条预测改变 |

exports 里 `~fit` / `~src` 元数据的值全是数与字符串（8.1.3 不报错）；`mesh` 的 `src` / `size` / `pivot` 都写了（1.4.5）；`rotate … about=(x,y,z)`、`axis=(x,y,z)` 是 3.9.4 / 3.9.5 接受的点与方向规格；`repeat … turn=` 是线性阵列的递增角（3.8.3）；没有任何文件触发 9.3.1。

### 9.5.3 预期改变的事实（exports，预测）

按 9.5.1 第 3 条的脚本，exports 里本版预期与参考不同的事实共 345 条：`intersects` → `inside` 308 条（basics.L0 1、curves.L0 1、l2fb 13、rotation.L0 249、spaceport.L0 11、spaceport.L1 11、spaceport_edit.L0 11、spaceport_edit.L1 11），`touching` → `against` 37 条（basics.L0 1、curves.L0 2、l2fb 4、spaceport.L0 6、spaceport.L1 8、spaceport_edit.L0 8、spaceport_edit.L1 8）。全部涉及 mesh（逆向把 revolve、sweep、布尔体写成了引用源网格的 `mesh`，其外包络是近似凸块），`gap` 与 `approx: true` 不变，只有 `rel`（与 `against` 的 `text`）变。两点说明：

- 这是**预测**，不是定义：脚本按 6.5.3 / 6.5.4 用参考输出的盒推算，对 box / wedge / mesh 作 $A$ 的情形精确，对其他实体保守（可能漏报）。M1 用实现跑出结果后按 9.4.5 N8 逐条核对。
- 数量集中在 rotation.L0（249 / 709）：过山车的 110 根枕木与 20 根立柱落在三条轨道 mesh 的外包络盒里。这是 6.5.4 允许近似 $B$ 的直接后果——细长的 mesh 外包络会"包住"途经它的一切。事实带 `approx: true`，读事实的一方据此决定信不信；本章只记录影响，不重开 6.5.4 的决定。

示例（corpus/examples）里预期改变的事实只有 1 条（rotation 的 `radar.feed`），由 6.8.3 的独立实现确认，不是预测。

### 9.5.4 结论

- **旧文件全部可编译**成立：19 个语料文件在本版编译后 `errors` 全部为空——没有一个文件触发 9.3.1 的任何一项（没有块内 material、同级重名、多个定位关系、无法线的 gap、零轴、非整数阵列数、重复 import、引用对象的元数据、非法颜色、非整数的关系谓词参数、非布尔的 check……）。
- **多出的警告** 4 条，全部是 1.6.9 的软保留名（`floor` ×2、`back` ×2）；这些节点作为路径首段照常可引用，语料里对它们的引用（basics.s3d 的 7 处 `on=floor…`）全部解析。
- **结果不同的项**全部在 9.3.2 / 9.3.3 的清单内，并由 9.4.5 的规范化消化：节点 `kind` / `types`（示例 15 个副本式 + 39 个影子，exports 39 + 102）、`file` / `line`（2 个 copy 节点、3 个皮肤节点）、事实的 `approx`（19 条）与 `rel`（示例 1 条确认、exports 345 条预测）、`gap` 的精度、概念成员顺序（2 个）、材质条目形状（全部）与 3 个改色条目。全部 1644 + 2408 个节点的 `world` / `box` / `tight` 与全部事实的 `gap` 预期在容差内一致（3.11、6.8）。
- 语料**没有覆盖**、只能按设计定下的规则：9.3.1 的每一项（语料里没有任何无效写法）、9.3.2 里标"无"的项、9.4.4 的边界对。

## 9.6 不兼容的处置建议

本节给旧文件的作者与工具作者。每条对应 9.3 的一项；"改法"是让文件在本版无错误且含义不变的最小改动。规范不要求任何工具自动做这些改动。

### 9.6.1 原型静默接受、本版报错的写法

| 情形（节） | 本版的错误 | 改法 |
|---|---|---|
| 块内的 `material`（10.1.1） | material 只能写在最外层 | 把语句移到最外层（材质表是全局的，位置不影响含义） |
| 同级重名（1.6.6） | 同级重名 X | 给后一个改名；如果本意是覆盖前一个，用 `drop X` 后再声明、或 `over X …` 改属性 |
| 跨文件最外层同名（5.3.2） | 同上 | 同上；层之间共用的对象只在一个文件里声明 |
| 同一文件 import 两次（5.2.3） | 文件 X 已经 import 过 | 删掉后一条；多个层共用一个库时由序列里最早的层 import 一次（5.5.2） |
| `material` 的非五分量键（10.1.2） | material 没有属性 k | 改成元数据 `~k=v` |
| `default=false`（10.1.2） | 无错误，但含义从"默认"变为"非默认" | 本意是默认材质就写 `default=true`；原型里 `default=false` 等于 `default=true`，依赖这一点的文件要改 |
| 多个定位关系（3.3.8） | 定位关系只能写一个 | 只留原型实际生效的那个（`on` 优先；其次按原型 `relation` 的分支顺序） |
| `gap` 写在 `inside` / `at=` 向量 / 无关系上（3.4.2） | gap 需要有法线的定位关系 | 删掉 `gap`（原型里它本来无效） |
| `rot` 零轴、方向规格零向量（3.5.3、3.9.5） | 不能是零向量 | 给出轴；原型的结果本就无定义 |
| `aimwith` 非方向词（3.5.4） | aimwith 需要方向词 | 写 `front`（原型的实际退路） |
| 引用祖先定位（3.6.2） | 不能相对包含自己的对象定位 | 改引用兄弟或祖先之外的对象；原型给出的解依赖后置，不稳定 |
| `repeat n=2.4` / `n=-1`（3.8.1） | n 需要非负整数 | 写 `n=2` / `n=0`（原型的实际值） |
| `along` 加 `step` / `center`；`orient` 无 `along`；`orient=true/false`（3.8.4） | 互斥 / 枚举错误 | 删掉无效的键；`true` → `yaw`，`false` → `none` |
| 副本式 `mirror` / `rotate … of=` 带定位属性或 `n`（3.9.1） | 副本跟随原物体 | 删掉（原型忽略它们）；要独立摆放用 `copy` |
| 容器式 `rotate n=2.6`（3.9.2） | rotate 的 n 需要 1..360 的整数 | 写 `n=3`（原型四舍五入的值） |
| `across=x` 加 `axis`（3.9.3） | across 是轴名时不能再写 axis | 删掉 `axis` |
| 空的 `over X`（4.5.1） | over 需要属性或块 | 删掉这条语句 |
| 容器 → 实体换形（4.6.1） | 不能把容器 X 换成 T | 先 `drop X`，再新建 |
| 模板 → 模板换形带多余参数（4.6.2） | 模板 T 没有参数 k | 在 over 的属性区只写 T 有的参数 |
| 几何时机的 `where h > …`、关系谓词、概念名（5.4.2） | 几何时机的选择器只能用静态量 | 改成路径、`#tag`、`kind()`、`name` / 元数据 / `mat`；要按尺寸或事实改几何，先给对象打标签 |
| 外观时机 `over` 选中空集（5.4.2） | X 没有选中任何对象 | 删掉或修正选择器；只想"有则改"用 `style` |
| 带锚点的引用当集合（7.2.3） | X 不是集合 | 去掉锚点 |
| 关系谓词 `on(S, 0)` / `on(S, 1.5)`（7.2.4） | 第二个参数需要不小于 1 的整数 | `0` → 删掉第二个参数并改用 `touching`/`on` 的含义核对；`1.5` → `2`（原型的实际值） |
| `concept N : 对象`（7.4.2） | 上位概念 U 没有定义 | 去掉 `: U` 或改成概念名 |
| pattern 同一角色两条 `in`（7.5.1） | 角色 X 的 in 约束重复 | 合并成一条（原型只用后一条） |
| pattern 域引用更晚的角色（7.5.2） | 角色 X 的 in 不能引用 Y | 调整角色声明顺序 |
| `check count(S)`（7.6.1） | check 的结果需要真假值或集合 | 写 `count(S) > 0`（原型"非零为真"的含义） |
| `variant … at=… pin=…`（7.8.1） | 变体的位置由外包络决定 | 删掉 `at` / `pin`（原型覆盖了它们） |
| 对布尔容器的操作数挂变体（7.8.6） | X 是布尔容器的操作数 | 对整个布尔容器挂 |
| 元数据值写名字（8.1.3） | 元数据不能引用对象 X | 写成字符串 `"X"` |
| `color=#名字`（10.2） | 颜色写成 #rgb… | 写十六进制颜色 |
| 模板名与种类同名、参数叫 `mat` / `color`（4.2.1、1.3.4） | 模板不能与内置种类同名 / 参数名 | 改名 |
| 硬保留名作名字（1.6.9） | X 是保留名 | 改名 |

### 9.6.2 结果不同的项

| 情形（节） | 旧文件要不要改 |
|---|---|
| 布尔容器 / 近似 $B$ 的事实标 `approx`、`inside` / `against` 的扩展（6.5） | 不改文件。读事实的工具按 `approx` 字段决定信任度；依赖"mesh 只会 touching"的 check / concept 要复查（exports 的 345 条预测事实，9.5.3） |
| 副本式节点与影子的 `kind` / `types`（4.2.7） | 不改文件。依赖 `where kind == "mirror→group"` 之类组合串的选择器（规范里从来不合法）改成原件的种类 |
| copy 节点、皮肤节点的 `file` / `line`（4.3.2、7.8.3） | 不改文件。按行归因的工具（评分、报告）按新归属读 |
| `color` 只改颜色分量（10.3.2） | 只写 `color` 的节点在本版保留原材质的 metal / rough；要原型的"换成匿名材质"效果，写 `mat=` 一个只有颜色的材质 |
| 集合成员顺序、`nodes[].concepts`、`variants[].members`（7.2.3、7.4.4、7.8.2） | 不改文件。按顺序读集合的工具改按节点顺序 |
| copy 位置与匿名段计数（4.3.5） | 写在块中间的 `copy` 后面若有匿名兄弟，它们的编号会 +1；引用 `box#2` 之类路径的 over / drop 要相应改 |
| copy 的元数据合并（4.3.6） | 依赖"copy 带元数据时整体替换"的文件，在 copy 语句上显式写出要覆盖的全部键 |
| 容器 → 容器换形的子语句（4.6.2） | 要保留的子语句在 over 块里重写 |
| 就地修改在重建后保留（4.1.3） | 依赖"重建会丢掉之前的 over"的文件，在重建型 over 里显式写回想要的值 |
| 副本式节点带 `mat`（10.3.1） | 原件是容器时本版会生效；不想生效就删掉 |

### 9.6.3 给工具的建议

- 对参考输出包比较前按 9.4.5 规范化；报告按 9.4.6。
- 新写的文件**应**带文件头 `~lang=2`，并避免软保留名作节点名（1.6.9）；但要同时供原型编译的文件不能带文件头（9.1.2）。
- 逆向输出（exports 的生成器）在新版里应把 revolve / sweep / 布尔体识别为对应实体而不是 `mesh`，否则 6.5.4 会让细长 mesh 的外包络包住途经的一切（9.5.3）；这是工具质量问题，不是语言规则。

### 9.6.4 本章的错误与警告

本章不定义任何错误或警告；各项的诊断由所引用的章定义。
