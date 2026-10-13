import io,sys
p='/home/user/SVG3D/spec/08-metadata.md'
s=io.open(p,encoding='utf-8',newline='').read()
assert '\r' not in s
def rep(old,new,count=1):
    global s
    assert s.count(old)==count, (s.count(old), old[:60])
    s=s.replace(old,new)

# 依赖的章
rep("被依赖：05-layers.md（文件头元数据的输出）、09-compat.md（行为变化清单）、10-materials.md（material 条目的 meta）。",
    "被依赖：04-derivation.md（4.2.7 的 `defs[]` 条目、4.3.6、4.6.2）、05-layers.md（文件头元数据的输出）、07-semantics.md（7.3.1 的字段表、7.8.4）、09-compat.md（9.3 的行为变化清单、9.4.5 的规范化 N12）、10-materials.md（material 条目的 meta）。")

# 8.3.5 第三条
rep("- copy 复制的是 A 的**定义**（源语句）：A 在被 copy 前后通过 over 得到的修改不进 R，但 over 不改元数据，所以这一点对元数据没有影响；A 被重建型修改后再 copy，复制的是重建后的语句（元数据相同，参数可能不同）。",
    "- copy 复制的是 A 的**源语句**（04-derivation.md 的 4.3.2 第 4 条）。全部普通 copy 在任何几何时机的修改之前展开（阶段 4 先于阶段 5，4.1.1、4.3.5），所以不论书写先后，R 的元数据都按**未修改**的源语句、在原语句的环境与原参数绑定里求值：A 此前或此后被就地修改（本来不碰元数据，8.3.6）或被重建型修改（模板参数、阵列 `n`、换形）都不进 R——A 自己的元数据在重建后按新参数重新求值，R 仍按原参数（语料 spaceport_edit.s3d：`over hangar dw=16` 写在 `copy hangar2 of=hangar` 之前，参考输出里 `hangar2.body.opening` 仍是 12 宽，04-derivation.md 的 4.8.1）。唯一的例外是 **over 块里的 copy**：它在阶段 5 展开该 over 块时立即展开（4.3.5 末段），复制的是那一刻 A 的有效语句（4.1.3）——A 已被重建时就是重建后的语句，R 的元数据按它求值（元数据项与源语句相同，依赖参数的值随新参数变）。")

# 8.3.6 D11 表
rep("换形对 D11 表的**每一种**\"原 → 目标\"组合都保留元数据（表里\"标签与元数据\"列恒为\"保留\"），包括容器 → 基本体被拒绝的情形（节点不变，元数据自然不变）。",
    "换形对 04-derivation.md 4.6.2 保留表的**每一种**\"原 → 目标\"组合都保留元数据（表里每格的\"标签元数据\"恒为 ✓），包括容器 → 原子被拒绝的情形（4.6.1：节点不变，元数据自然不变）。")

# 8.3.7 第三条
rep("- 皮肤子树不进 `all()`，所以 `all() where …` 读不到皮肤节点的元数据；通配、`children()`、`descendants()` 选中的皮肤节点在 where 里按常规读（1.6.4）。",
    "- 皮肤对各种机制的可见与不可见的精确清单在 07-semantics.md 的 7.8.4：皮肤子树不进 `all()`、`kind()`、`#tag` 与全局索引，而且编译里没有任何选择器在皮肤存在之后求值（7.2.5 第 3 条），所以 concept、pattern、check、style、variant 与外观时机 over 的 `where` 读不到皮肤节点的元数据。通配、`children()`、`descendants()`、直接路径对皮肤的可见性（1.6.4）只在皮肤模板体内的表达式与编译后的工具查询里有意义：那里的 `where` 按常规读皮肤节点的元数据（`skin` 为空，皮肤体内节点各带自己语句的）。")

# 8.3.9 汇总表 copy 行
rep("| copy 节点 | 原件语句的 ∪ copy 语句的（后者逐键覆盖） |",
    "| copy 节点 | 原件**未修改**的源语句的 ∪ copy 语句的（后者逐键覆盖）；over 块里的 copy 按原件那一刻的有效语句 |")

# 8.4.3
rep("- 编译输出的顶层有 `meta` 字段：主文件的文件元数据，键顺序同文件头；`lang` **总是存在**且值为 `2`（文件头没写时补上，写在最前）。没有文件头时是 `{ \"lang\": 2 }`。",
    "- 编译输出的顶层有 `meta` 字段：主文件的文件元数据，键顺序同文件头；`lang` **总是存在**且值为 `2`（文件头没写时补上，写在最前）。没有文件头时是 `{ \"lang\": 2 }`。原型 compile 的输出**没有**顶层 `meta` 字段（参考输出 `.compile.json` 的顶层键只有 `items` `curves` `facts` `factsSkipped` `factProviders` `nodes` `concepts` `patterns` `variants` `checks` `defs` `materials` `errors` `warnings`）；本版总是输出 `meta: { \"lang\": 2, … }`，对拍前按 09-compat.md 9.4.5 的 N12 给参考输出补 `meta: { \"lang\": 2 }`（8.6.2）。")
rep("- 非节点语句的元数据只在该语句有输出条目时输出，字段名一律是 `meta`、形状同 8.4.1：`materials` 的每个条目（10-materials.md）、`defs` 的每个条目（04-derivation.md）、`concepts[]`、`patterns[]`、`checks[]`、`variants[]` 的每个条目（07-semantics.md）。",
    "- 非节点语句的元数据只在该语句有输出条目时输出，字段名一律是 `meta`、形状同 8.4.1：`materials` 的每个条目（10-materials.md 的 10.6）、`defs[]` 的每个条目（04-derivation.md 的 4.2.7：`{name, line, file, params, base, meta}`）、`concepts[]`、`patterns[]`、`checks[]`、`variants[]` 的每个条目（07-semantics.md 的 7.6.3、7.8.9；登记失败而带 `error` 的条目同样带 `meta`）。原型这些条目都没有 `meta`（`defs` 还只是模板名数组，4.8.5）；本版各带 `meta`，没有元数据时 `{}`（9.4.5 的 N12）。")

# 8.6 验证节
s = s.rstrip('\n') + """

## 8.6 用参考输出验证

### 8.6.1 一致的项

语料里的元数据只出现在 corpus/exports 的 10 个文件（节点语句上的 `~fit=数`、`~src="字符串"`，共 348 / 248 项）；corpus/examples 的 7 个文件没有任何元数据项、没有文件头。参考输出（corpus/reference/exports/*.compile.json，原型 14582a3）的 `nodes[].meta` 与 8.3、8.4 的规则逐条一致：

| 规则 | 参考输出里的例子 |
|---|---|
| 节点带自己语句的元数据，键顺序同源语句（8.3.1、8.4.1） | spaceport_edit.L0 的 `spare`：`{ "fit": 0.9202, "src": "spare" }`（源语句 `~fit=0.9202 ~src="spare"`） |
| 没有元数据项的语句输出 `{}`（8.4.1） | rotation.L0 的 `curve1`（spline，源语句无元数据）、l2fb 的 `volume_4`（`mirror volume_4 of=volume_3`：R 只带自己语句的元数据，不取 `volume_3` 的 `{fit:0, src:"merged_2#6"}`，8.3.4） |
| repeat 节点与阵列实例为空，体内语句每个实例一份（8.3.3） | rotation.L0 的 `coaster_legs`、`coaster_legs.0` 是 `{}`，`coaster_legs.0.col` … 每个实例都是 `{ "fit": 0.1867 }` |
| 影子读原件的（8.3.4），副本组为空 | rotation.L0 的 `pisa_galleries_0_cols.r1.col` 与原件 `pisa_galleries_0_cols.col` 同为 `{ "fit": 0.909 }`，`pisa_galleries_0_cols.r1` 是 `{}`；l2fb 的 `columns_3.0.column`（阵列实例的影子的子节点）是 `{ "fit": 1 }`，`columns_3.0` 是 `{}` |
| 数按 8.4.2 写出 | `fit: 1` 写成整数 `1`，`0.9202` 保持最短往返表示 |

原型的实现与这些规则相同：`staticMeta` 在展开时按语句求值（每个节点一次）；输出 `meta: (n.ghostOf ?? n).meta ?? {}`（影子读原件）；`where` 帧把 `(m.ghostOf ?? m).meta` 的各键惰性放进环境（8.2.2 第 2 步）。语料里没有 `where` 读元数据键的语句，8.2.2 / 8.2.3 只能按设计定下。

### 8.6.2 因本章定义而预期不同的项

| 项 | 原型行为（函数） | 本版 | 语料 |
|---|---|---|---|
| 顶层 `meta` | compile 不输出 | `meta: { "lang": 2, … }` 总是存在（8.4.3） | 全部文件；9.4.5 的 N12 给参考输出补 `{ "lang": 2 }` |
| `concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`defs[]`、`materials` 条目的 `meta` | 不输出（`defs` 是模板名数组） | 每个条目带 `meta`，缺省 `{}`（8.4.3） | 全部条目；N12 补 `{}`（`defs` 的形状按 4.8.5） |
| copy 语句带元数据 | expandCopies 整体替换：`meta: s.meta?.size ? s.meta : A.stmt.meta` | 逐键合并，copy 自己的覆盖同名键（8.3.5） | 无（语料的 copy 语句没有元数据） |
| 元数据值引用对象、`#tag`、`where`、求解后才可用的函数 | staticMeta 用带节点的上下文求值：退化成字符串或在求值时失败 | 报错"元数据不能引用对象 X"等，该键不存在、节点不退化（8.1.3、8.1.6） | 无（exports 的值全是数与字符串） |
| 元数据对 `let`、`def`、`import`、`concept` 等语句 | 各有不同的失败方式（09-compat.md 的 9.2.1） | 对所有语句合法，非节点语句只保存不作用（8.1.2、8.3.8） | 无 |
| 警告"元数据键 X 与 where 字段同名…"、"lang 只在文件头有意义"、"没有节点带元数据 X" | 无 | 8.1.4、8.2.2 | 无（9.3.4） |
| 文件头 `~lang=2` | 报错"未知的类型 “~lang=2”" | 文件元数据（8.1.5） | 无（语料没有文件头） |

本章定义而语料没有覆盖、只能按设计定下的行为：向量与列表值及其 JSON 映射（8.4.2）；重建型修改后的重新求值与换形的保留（8.3.6）；over 块里 copy 的元数据（8.3.5）；皮肤体内节点的元数据（8.3.7）；文件头与被 import 文件的 `meta`（8.4.3）。
"""
io.open(p,'w',encoding='utf-8',newline='\n').write(s)
print("ok")
