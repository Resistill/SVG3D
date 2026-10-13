import sys
p='/home/user/SVG3D/spec/crosswalk.md'
s=open(p,encoding='utf-8').read()
i=s.index('## 10 参考输出里预期与本版不同的项')
head=s[:i]
new='''## 10 参考输出里预期与本版不同的项（※ 的索引）

这些项在上表里以 ※ 标出（`tight`、顶层 `meta`、条目的 `meta` 这类第 5 列不引用其数值的全局项只列在这里）；每条都已在相应章的"因本章定义而预期不同的项"里（第 1 章没有这样的小节，软保留名警告在 1.6.9 正文与 1.10.3 的警告表里）给出原型行为、新行为与理由，这里只列索引，供第 9 章汇总与 M1 对拍时排除：

| 表里的行 | 项 | 参考输出 | 本版 | 出处 |
|---|---|---|---|---|
| curves 39、45、71；rotation 30；semantics 33、44–49；spaceport 47–48 | 副本式节点的 `kind` | `mirror→span`、`mirror→revolve`、`rotate→group`、`mirror→box`、`mirror→wedge`、`mirror→group` | 原件的种类 `span`、`revolve`、`group`、`box`、`wedge`、`truss` | 4.8.5；1.6.7 |
| spaceport 47–48 | 副本式节点的 `types` | `null` | 原件的类型链（`tower2` → `["truss"]`） | 4.8.5；4.2.7 |
| spaceport 47–48；curves 71 | 影子的 `kind`（原件是阵列实例 / 副本组） | `group` | `instance` / `mirror` | 4.8.5；1.6.7 |
| spaceport_edit 13–15 | copy 节点的 `file` / `line` | spaceport.s3d:68 | spaceport_edit.s3d:13 | 4.3.2 第 7 条、4.8.5、5.5.4 |
| spaceport_blueprint 19 | 皮肤节点 `skin` 的 `file` | `null` | 变体语句的文件 | 5.2.5、5.5.4、4.8.5 |
| rotation 13–21；semantics 51–53；spaceport 30–39；spaceport_blueprint 15–18 | `defs[]` 条目的形状 | 模板名的字符串数组（`["turbine"]`、`["house","chimney_house","planks","townhouse"]`、`["truss","shed"]`、`["truss","shed","ribbed"]`） | 对象条目 `{name, line, file, params, base, meta}`；`name` 的序列与参考数组相同 | 4.2.7、4.8.5；8.4.3 |
| spaceport_blueprint 20；spaceport_edit 13–15 | 被 `color` 改色的条目的 `material` 形状 | `{color}` | 基础材质的五个分量 + 新颜色（concrete / steel 改色） | 10.6、10.3.2 |
| basics 21–25；spaceport 57–66、68、71、74–78；spaceport_edit 13–15 | 一方是布尔容器的事实的 `approx` | 不标 | `approx: true` | 6.8.5、2.5.6 |
| rotation 32–38 | `radar.feed` 与 `radar.dish` 的关系 | `intersects` | `inside`（revolve 的近似凸块允许作 $B$） | 6.8.5、6.5.4 |
| rotation 13–24 | 涉及 cone（`farm.N.t.tower`）的 6 条事实的 `gap` | 顶点处用半径 $10^{-4}$ 的小环算凸块 | 顶点是一个点；`gap` 的差 $\\le 10^{-4}$，(rel, a, b) 不变 | 2.2.3、2.8.3、6.8.3 |
| semantics 78–88 | 概念 `lamp`、`tree` 的成员顺序 | 并集按操作数拼接 | 节点顺序 | 7.9.4、7.2.3、1.5.5 |
| basics 8；curves 48–56 | `warnings` | `[]` | 各一条"节点名 floor / back 与保留名相同，作为路径的后续段时不能按名字引用" | 1.6.9、1.10.3 |
| curves 18–22；rotation 13–24、32–38、47–56、59–70；spaceport 30–39、98–101（spaceport_blueprint、spaceport_edit 里的同一些节点） | 非轴对齐的 cyl / cone / rod 的 `tight`（`lamps.N.pole`、`farm.N.t.tower`、`radar.feed`、pisa 的 core / ring / col / belfry、`coaster.legs.N.col`、`tower.decks.N.brace`、`mast`） | `tightBox` 取 12 边内接棱柱顶点的盒（偏小） | 两端面圆的轴向极值（2.1.8）；每面偏差 $\\le R(1 - \\cos 15°)$ | 2.8.3、2.1.8 |
| spaceport 92–95 | 圆截面 sweep `feed1` 的 `tight`（spaceport 与 spaceport_blueprint） | 每环只取 20 点里的 8 个 | 全部 20 个环点；偏差 $4.3 \\times 10^{-6}$ | 2.8.3、2.2.9 |
| rotation 32–38 | revolve `radar.dish` 的 `tight` | 外接 12 边棱柱的盒（大 1.94） | 轮廓各点的圆 / 弧极值 | 2.8.3、2.2.7 |
| basics 21–24；spaceport 57–66、68；spaceport_edit 13–15 | 布尔容器（`crate`、`hangar.body`、`hangar2.body`）的 `tight` | `null` | 操作数的紧包围盒按运算组合 | 2.5.4、2.8.3 |
| 全部文件 | 顶层 `meta` | 没有该字段 | 总是 `{ "lang": 2, … }`（语料没有文件头，所以是 `{ "lang": 2 }`） | 8.4.3、8.6.2 |
| 全部文件 | `concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`defs[]`、`materials` 条目的 `meta` | 不输出 | 每个条目带 `meta`，语料里都是 `{}` | 8.4.3、8.6.2 |
| 全部文件 | 事实的 `gap` 精度与 `approx` 字段是否总是输出 | 4 位小数；只在为真时输出 | 完整双精度；总是输出布尔 | 6.6.1、6.8.5 |
| 全部文件 | `items[]` 条目材质、`materials` 条目的缺省分量 | 只输出写了的键 | 五个键总是都有 | 10.6 |

除 `tight` 的四行与 cone 的 `gap` 一行外，这些差别都不涉及任何节点的 `world`、`box`、事实的 (rel, a, b) 与语义结果；`tight` 与 cone `gap` 的差别有 2.8.3 给出的上界（本表第 5 列不引用任何 `tight` 的数值），对参考比较时的额外容许量由 09-compat.md 的 9.4 规定。所以参考输出仍逐节点、逐事实适用于 M1 的验收（3.11.2、6.8.3、7.9.1）。语料里没有触发的差别（3.11.5、4.8.5、5.5.4、6.8.5、7.9.4、8.6.2、2.8.3 的其余各行，例如 R3 之前曾讨论过的皮肤 `late` 判定——按 3.7.6 的规则，三个皮肤的 `late` 与参考输出完全一致）不在本表。

## 11 缺口

逐条语句的解释没有缺口：352 条语句行每一条都能在上面找到条款。初稿核对第 5 列时发现三处**输出形状**在各章里没有定义或没有标出与原型的差别；它们不影响任何语句的含义，现已全部补齐，本表第 5 列与第 10 节按补齐后的条款写：

| 编号 | 缺口（初稿） | 补齐的条款 | 本表的处理 |
|---|---|---|---|
| G1 | `defs` 输出条目的形状：参考输出是模板名数组，8.3.8 / 8.4.3 要求条目带 `meta`，但没有字段表 | 04-derivation.md 的 4.2.7 给出 `defs[]` 条目的字段表 `{name, line, file, params, base, meta}`（顺序 = 名字首次登记的程序顺序，与原型数组的序列相同），4.8.5 列出"原型输出模板名数组"为预期不同；8.4.3 引用它 | 第 5 列凡引用 `defs` 的地方改写成"`name` 序列"并标 ※；第 10 节加一行 |
| G2 | 顶层 `meta` 与原型的差别没有在第 8 章标出 | 08-metadata.md 的 8.4.3 写明原型不输出顶层 `meta`、本版总是输出 `{ "lang": 2, … }`，8.6.2 列为预期不同 | 第 10 节加两行（顶层 `meta`；各类条目的 `meta`） |
| G3 | `curves[]` 与 `items[]` 几何部分的字段名没有定义 | 02-geometry.md 的 2.1.11 定义 `curves[]` 条目（`path`、`closed`、`pts`）与 `items[]` 实体 / 布尔条目每种实体的字段（`type`、`w h d`、`r1 r2 h axis`、`L w t dy` …，字段名沿用原型 `buildItems`），只定义值的含义 | "读法"注明第 5 列的字段名按 2.1.11；第 5 列原有的 `pts`、`closed`、`geom {beam, L, w, t, dy}` 等写法与 2.1.11 一致，不改 |

现在没有未补齐的缺口。
'''
s=head+new
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
