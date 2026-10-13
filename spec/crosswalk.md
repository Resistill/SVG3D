# 规范对语料的逐条对照表

**用途。** PLAN.md 第 9 节把"规范能解释 corpus/examples 里每一条语句的含义（逐条对照表）"列为 M0 的验收项。本文件对 corpus/examples 的 8 个文件（basics、curves、rotation、semantics、spaceport、spaceport_blueprint、spaceport_edit、lib/semantics）的**每一行源码**给出：它用到的语言构件、解释它的规范条款、参考输出包里能核对它的项。它只做索引，不定义任何规则；条款与本表不一致时以各章正文为准。

**读法。**

- 每个文件一张表，列是：行号 | 语句（可截短，`…` 表示省略） | 构件 | 条款 | 参考输出里对应的可核对项。
- 条款写章.节.小节号（"3.3.2"），不写决定编号；06-facts.md 已有正文，照常引用；09-compat.md 不引用。"示例原句"指该小节的 ```s3d 示例里逐字出现了这条语句（或它的核心片段）。
- 相邻的同构语句合并成一个行区间（"5–11"）；一条带块的语句连同它的块（含结束的 `}` 行）合并成一个区间。每个文件末尾有一行列出全部注释行与空行的行号，所以**每一行源码都被某一行表覆盖**；覆盖率按"语句行"算：语句行 = 非空、非纯注释的行（含块的 `}` 行）。
- "参考"指 corpus/reference/examples/<名>.compile.json（原型 commit 14582a3，MANIFEST.md 的字段说明）；"大纲"指同目录的 <名>.outline.txt（`--outline=2 --semantics`，末尾 〔…〕 是加上 lib/semantics.s3d 后的概念标注）。坐标保留 4 位小数；"N" 代表任一阵列下标，"K" 代表任一副本组序号。
- "※" 标出参考输出里**预期与本版不同**的项，差异与出处汇总在第 10 节；它们都是各章"因本章定义而预期不同的项"里已经列出的，本表只做索引。
- "缺口"指找不到能解释某一项的条款；本表逐条语句的解释没有缺口，三处**输出形状**的缺口列在第 11 节并进 cross_chapter_requests。

**统计。**

| 项 | 数 |
|---|---|
| 文件 | 8 |
| 源码行 | 504（basics 36、curves 71、rotation 70、semantics 146、spaceport 112、spaceport_blueprint 20、spaceport_edit 15、lib/semantics 34） |
| 语句行 | 352（basics 23、curves 47、rotation 56、semantics 115、spaceport 75、spaceport_blueprint 14、spaceport_edit 9、lib/semantics 13） |
| 注释行 / 空行 | 91 / 61 |
| 表行（不含每个文件末尾的注释 / 空行那一行） | 113 |
| 语句行覆盖率 | 8 个文件都是 100%（第 1–8 节末尾各给出计算） |
| 条款引用次数最多的章 | 第 3 章（186 次），其次第 1 章（90）、第 4 章（61）、第 2 章（55）、第 7 章（55）；第 9 节给出逐章计数 |
| 语句级缺口 | 0 |
| 输出形状缺口 | 3（第 11 节：`defs` 条目形状、顶层 `meta` 与原型的差别、`curves[]` / `items[]` 几何字段） |

**核对方法。** 第 5 列的每一个路径、个数与数值都由脚本从参考输出读出，不是凭记忆写的（脚本放在 scratchpad）：`bylines.mjs` 按 `nodes[].file:line` 分组列出每条语句生成的节点（路径、kind、types、ghost / hidden / skin / late），并打印 `concepts[]`、`patterns[]`、`checks[]`、`variants[]`、`materials`、`defs`、`errors`；`facts.mjs` 按文本过滤 `facts[]`；另一段脚本抽取指定节点的 `world.t`、`box`，`items[].material`，`curves[]` 的点数与 `closed`。各章自己的验证脚本（3.11、4.8、5.5.3、6.8、7.9、10.8）已经逐节点 / 逐事实核对了数值，本表引用它们的结论，只对表里直接写出的数再抽样读了一遍（第 1–8 节的每个数都对上）。

## 1 basics.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 3–6 | `material a color=#d9d4c7` … `material d color=#7aa35a` | 最外层 material；颜色位置的井号记号；没有 `default` | 1.3.7、1.4.6、1.2.7；10.1.1、10.1.2、10.1.3、10.1.4（没有默认材质时 $D = B$） | `materials` 恰 4 项 `a b c d`，颜色逐字相同；`defs: []`；`errors: []` |
| 8 | `box floor size=40,1,24 pin=top mat=a` | 实体 box、`size`、没有关系只有 `pin`、`mat` | 1.3.3、1.4.4、1.4.5；2.2.1；3.3.7（没有关系但写了 pin）、3.4.1；10.2 | 节点 `floor`[box]，`world.t` = (0, −1, 0)，`box` = [−20, −1, −12]…[20, 0, 12]；大纲 `floor 〔slab platform〕` |
| 11 | `box table size=8,3,5 on=floor[-0.5,1,0] mat=c` | `on` + 归一化锚点 `B[u,v,w]` | 3.3.1、3.3.2（示例原句）、3.2.2、3.2.3 | `table`；事实 `table on floor` gap 0 |
| 12 | `cyl cup r=0.6 h=1.2 on=table mat=b` | cyl；`on` 缺省锚点 `top` | 2.2.3（示例原句）；3.3.2 | `cup`；`cup on table` |
| 13 | `sphere ball r=0.8 on=table[0.6,1,-0.5] mat=d` | sphere；`on` 带锚点 | 2.2.4（示例原句）；3.3.2；6.5.2（球的凸块投影） | `ball`；`ball on table`；`ball` 与 `floor` 之间没有事实（6.2.3 示例） |
| 16 | `box shelf size=10,9,2 on=floor align=back mat=c` | `on` + `align` 面名 | 3.4.3（面对齐，示例原句）、3.4.5 | `shelf`；`shelf on floor` |
| 17 | `box book1 size=0.6,3,1.6 against=shelf.front align=bottom,left mat=b` | `against` 方向词锚点、自动选面、两项 `align` 按书写顺序 | 3.3.4（示例原句）、3.4.3（示例原句）、1.4.1（面名列表） | `book1`；`book1 against shelf.front`、`book1 on floor` |
| 18 | `box book2 size=0.6,2.6,1.6 against=book1.right align=bottom,back mat=d` | 同上 | 同上 | `book2`；`book2 against book1.right`、`book2 against shelf.front`、`book2 on floor` |
| 21–24 | `subtract crate on=floor[0.55,1,0.4] mat=a { box shell size=5,4,5 / box hollow size=4.4,4,4.4 at=shell.bottom pin=bottom offset=0,0.3,0 }` | 布尔容器与块；容器按包络定位；块内引用兄弟 `shell`；`at=` 带面锚点；`pin`；`offset`；容器的 `mat` 继承 | 1.3.3（示例原句）、1.3.15；2.5.1、2.5.4（subtract 的盒 = 第一个操作数）、2.5.6；3.7.1、3.7.2；3.3.6、3.4.1、3.4.4；1.6.5（第 2 步在 crate 的子节点里找到 shell）；10.4.4（示例） | `crate`[subtract] t = (11, 0, 4.8)，box 5 × 4 × 5；`crate.shell`、`crate.hollow`；`items[]` 里 crate 条目带两个操作数、材质 a；事实 `crate on floor`（※ approx） |
| 25 | `sphere orb r=1.4 inside=crate.hollow align=bottom mat=b` | `inside` + 路径下降到布尔操作数 + `align` | 3.3.5（示例原句）、3.4.3；1.6.5 第 5 步；6.5.4（空腔里的物体是 inside） | `orb` t = (11, 0.3, 4.8)；事实 `orb inside crate` gap −3.1（※ approx） |
| 28–29 | `box pillarL … on=floor[0.3,1,-0.6]`、`box pillarR … on=floor[0.85,1,-0.6]` | `on` 带锚点 | 3.3.2 | `pillarL on floor`、`pillarR on floor` |
| 30 | `span beam from=pillarL.top-right to=pillarR.top-left section=1,0.6 pin=top mat=b` | span 梁；方向组合词锚点；`section`；`pin` 是截面枚举 | 2.2.8（示例原句）；3.10.1（示例原句）、3.10.3；1.2.6（组合词是一个记号）、1.4.3、1.4.4（span 只接受 pin / mat / color） | `beam`[span] t = (11.5, 7, −7.2)，box y ∈ [6.4, 7]；`beam against pillarL.right`、`beam against pillarR.left` |
| 31 | `span ramp from=pillarL.bottom-right+(0,0,2) to=pillarR.top-left+(0,0,2) section=1.5,0.2 pin=bottom mat=d` | 锚点加偏移 | 1.5.2（引用 + 向量₃）、3.2.4（偏移在使用框架里加）；2.2.8（示例原句） | `ramp`；`ramp touching floor` gap 0 |
| 32 | `span stairs from=table.bottom-left+(-6,0,0) to=table.top-left steps=8 section=2,0.2 mat=d` | 台阶形态 | 2.2.8（steps；示例原句）、3.10.1 | `stairs` t = (−17, 1.5, 0)，box y ∈ [0.175, 3]；`stairs touching table`（6.5.2 说明里的例子） |
| 35 | `box turned size=6,3,3 on=floor[-0.1,1,0.55] turn=35 mat=c` | `turn` | 3.5.2、3.5.6；3.4.5（示例原句，逐步算出 t） | `turned` t = (−2, 0, 6.6)；`turned on floor` |
| 36 | `box sticker size=1,1.5,1 against=turned.right align=bottom mat=b` | 继承参照物的朝向 + `against` 自动选面 + `align` | 3.5.1、3.5.5、3.3.4、3.4.3；3.4.5（示例原句，t 的推导） | `sticker` t = (0.8671, 0, 4.5925)；`sticker against turned.right`、`sticker on floor` |
| 注释 / 空行 | 注释：1、10、15、20、27、34；空行：2、7、9、14、19、26、33 | — | 1.2.2 | — |

覆盖率：语句行 23（3–6、8、11–13、16–18、21–25、28–32、35–36），被上表覆盖 23，**100%**。

## 2 curves.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 5–11 | `material ground color=#6b7360 rough=1` … `material water color=#5d93c4 rough=0.2` | material 的 `rough`、`metal` 分量 | 1.4.6；10.1.2（范围 [0, 1]）、10.1.3 | `materials` 7 项，`steel` = {#7f8b96, 0.6, 0.4} |
| 13 | `box ground size=150,1,150 pin=top mat=ground` | 同 basics:8 | 3.3.7、3.4.1 | `ground` t = (0, −1, 0) |
| 16 | `arc loop r=40 angle=-60,240` | arc；`angle` 向量₂；一元负号；没有关系 → 圆心在父原点 | 2.3.2（示例原句；采样 76 点）、2.3.1、2.3.8；1.2.3（没有负数字面量）、1.4.5；3.3.7（原点落在父原点的说明用了这句）、3.10.6 | `loop`[arc] t = (0, 0, 0)；`curves[]` 里 `loop` 76 个点、`closed: false`；box z ∈ [−39.9756, 34.6410] |
| 17 | `sweep lane path=loop profile=rect section=8,0.1 pin=bottom mat=road` | sweep 沿曲线；rect 截面；`pin` 是截面参数 | 2.2.9（示例原句）；3.10.4；1.4.3（profile 缺省规则）、1.4.4 | `lane`[sweep] box x ∈ [−44, 44]、y ∈ [0, 0.1]；`lane on ground` |
| 18–22 | `repeat lamps n=11 along=loop mat=steel { cyl pole r=0.18 h=6 at=0,0,6 / sphere bulb r=0.45 on=pole mat=white }` | 沿曲线的阵列（`orient` 缺省 yaw）；实例段；体内 `at=` 向量在实例局部系里；阵列的 `mat` 继承 | 3.8.1、3.8.4（示例原句，`lamps.3` 的推导）、3.8.5；1.6.2；3.3.6；10.4.4 | `lamps`[repeat] + 11 个实例 `lamps.0 … lamps.10`；`lamps.3` t = (34.6410, 0, −20)；`pole`、`bulb` 各 11 个；事实 `lamps.N.bulb on lamps.N.pole`（11 条） |
| 25–31 | `group garage { cyl core r=4 h=14 / helix rampline r=7 pitch=4.5 turns=3 / sweep ramp path=rampline profile=rect section=5,0.4 pin=bottom / helix railline r=9.4 pitch=4.5 turns=3 at=0,1,0 pin=origin / sweep rail path=railline r=0.08 }` | 没有关系的 group；helix；sweep 沿 helix；helix 自己的 `at` / `pin=origin`；圆截面 sweep；曲线计入容器的盒 | 3.7.1、3.3.7；2.3.3（示例原句）；2.2.9（示例原句）；3.10.6（示例原句）、3.10.4；2.6（示例原句：曲线计入） | `garage` box 19 × 14.58 × 18.99（含两条螺旋线）；`curves[]` 里 `garage.rampline`、`garage.railline` 各 271 点；`garage.core on ground`、`garage.ramp on ground` |
| 34 | `box canal size=10,0.05,150 on=ground[-0.75,1,0] mat=water` | `on` 带锚点 | 3.3.2 | `canal`；`canal on ground` |
| 35 | `arc archline r=8 angle=0,180 plane=xy at=canal.top pin=origin` | `plane=xy`；arc 按一般节点定位；`pin=origin` 把圆心放到落点 | 2.3.2（示例原句）；3.10.6（示例原句）、3.4.1 | `archline` t = (−56.25, 0.05, 0)；46 个点；box y 顶 = 8.0451 = 0.05 + 8 sin 88°（2.3.8 的说明） |
| 36 | `sweep arch path=archline profile=rect section=3,0.7 mat=stone` | sweep 沿竖直平面里的弧 | 2.2.9、2.3.7（扫掠框架） | `arch`；`deck intersects arch` gap −0.3498 |
| 37 | `box deck size=18,0.5,3.6 on=archline mat=stone` | 曲线当参照物（按采样折线的盒） | 2.3.8；3.3.2（示例原句）、3.2.1 | `deck` t.y = 8.0451（弧的采样盒顶） |
| 38 | `span stepsW from=canal.top-left+(-13.5,0,0) to=deck.top-left steps=16 section=3.6,0.25 mat=stone` | 台阶；锚点加偏移 | 2.2.8；3.10.1；3.2.4 | `stepsW`；`deck touching stepsW` |
| 39 | `mirror stepsE of=stepsW across=archline axis=x` | 副本式 mirror；曲线作镜像参照（盒心 + 曲线局部 x 轴） | 3.9.1、3.9.3（第四行；"曲线可以作镜像参照"）、3.9.5；4.4.1、4.4.5 | `stepsE` t = (−42.5, 4.2976, 0)，`of: "stepsW"`；`kind` ※（参考 `mirror→span`，本版 `span`）；`deck touching stepsE` |
| 42 | `revolve tower profile=[(0.8,0); (0.7,9); …] smooth=true on=ground[0.5,1,-0.55] mat=white` | revolve；二元组列表；`smooth` | 2.2.7（示例原句）；1.5.3、1.4.3 | `tower`[revolve]；`tower on ground` gap 0 approx（revolve 的凸块是外接棱柱，6.2.2） |
| 45 | `mirror tower2 of=tower across=garage axis=x` | 容器作镜像参照：过包络盒心、法线取容器的 x 轴 | 3.9.3（第四行）、3.9.1；4.4.1 | `tower2` t = (−37.5, 0, −41.25)；`kind` ※（`mirror→revolve` → `revolve`）；`tower2 on ground` approx |
| 48–56 | `group plaza on=ground[-0.45,1,0.5] { revolve basin … / revolve spout … smooth=true on=basin / rotate benches n=5 about=basin { box seat size=3.2,0.5,0.9 at=0,0,7.5 / box back size=3.2,1,0.2 on=seat align=front } }` | 容器按包络定位；revolve；容器式 rotate，`about` 引用兄弟、`angle` 缺省 360/6；节点名 `back` 是软保留名；`align=front` | 3.7.2；2.2.7（示例原句 basin）；3.9.2、3.9.4（示例原句 benches；$\theta = 60°$）；1.4.5（rotate 容器式的 n、about、angle）；1.6.9（软保留名放行并警告，原话举了 `box back`）；3.4.3 | `plaza.benches`[rotate] + `r1 … r5`；`plaza.benches.r1` t = (−33.75, 0, 37.5)；`seat`、`back` 各 6 个（原件 + 5 个 ghost）；`plaza.spout on plaza.basin` approx；`plaza.benches.rK.back on plaza.benches.rK.seat`（6 条）；`plaza.benches.r2.seat intersects lane` −0.1 |
| 59–68 | `group gate at=0,0,58 { mirror halves across=x { box house size=5,5,5 at=7,0,0 / wedge ramp size=3,5,5 against=house.right / box flag size=0.2,7,0.2 on=house[-0.8,1,0.8] } / arc gatearch r=5 angle=0,180 plane=xy / sweep beam path=gatearch profile=rect section=1,0.8 }` | `at=` 向量（不继承）；容器式 mirror，`across=x` 过容器原点、缺省 axis；wedge 的高边在 +x、镜像后翻转；`against`；arc + sweep | 3.3.6；3.9.2、3.9.3（第一行）；2.2.2（示例原句 ramp）、2.1.1（镜像副本的点集是反射像）、3.9.6、3.9.7（"镜像后的楔形坡向翻转就是这个意思"）；3.3.4；2.3.2、2.2.9 | `gate.halves`[mirror] + `gate.halves.mirrored`；`house` / `ramp` / `flag` 各有原件与 ghost 影子；`gate.halves.mirrored.ramp against gate.halves.mirrored.house.right`、`gate.halves.ramp against gate.halves.house.right`；`gate.beam intersects gate.halves.house` −0.8474 |
| 71 | `rotate gate2 of=gate about=garage angle=180` | 副本式 rotate；`about` 引用容器；原件里的 mirror 容器连同副本组一起成为影子 | 3.9.1、3.9.4（示例原句）；4.4.1（示例原句）、4.4.2 第 3 条；4.8.4 | `gate2` t = (0, 0, −58)，`kind` ※（`rotate→group` → `group`）；`gate2.halves.mirrored` `kind` ※（`group` → `mirror`）；gate2 子树 11 个 ghost；`gate2.halves.house` box = gate 的 house 绕原点转 180° |
| 注释 / 空行 | 注释：1–3、15、19、24、33、41、44、47、51、58、62、70；空行：4、12、14、23、32、40、43、46、57、69 | — | 1.2.2（块内注释） | — |

覆盖率：语句行 47（5–11、13、16–18、20–22、25–31、34–39、42、45、48–50、52–56、59–61、63–68、71），被上表覆盖 47，**100%**。

## 3 rotation.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 3–8 | `material ground color=#6e7a5c rough=1` … `material red color=#c8463c` | material | 10.1.2、10.1.3 | `materials` 6 项 |
| 10 | `box ground size=160,1,160 pin=top mat=ground` | 同 basics:8 | 3.3.7 | `ground` |
| 13–21 | `def turbine(h=32, blade=13, phase=0) { cone tower r=1.3 r2=0.7 h=h mat=white / box nacelle size=2.2,2.2,6 on=tower / sphere hub r=1.1 at=nacelle.front pin=center / rotate rotor n=2 axis=z at=hub.center pin=origin rot=(0,0,1,phase) { box blade size=1.4,blade,0.35 pin=bottom turn=12 } }` | def 与参数表（参数名 `h` 允许；`h=h` 的右侧在模板体帧里解析为参数）；cone 的 `r2`；`at=` 带锚点 + `pin=center`；容器式 rotate 带 P 组、`axis=z`、`rot` 向量₄；参数 `blade` 遮蔽同名节点；块内 `pin=bottom` 没有关系；块内注释 | 1.3.4、4.2.1、4.2.2；1.6.5 第 1 步、1.5.8（模板体帧）；2.2.3（示例原句 `cone tower`）；3.3.6（示例 `at=hub.center`）、3.4.1；1.4.4（容器式 mirror / rotate 接受全部 P 组）、3.9.2（示例 rotor，$M_1 = R_{\text{axis}}(\hat z, 120°)$）、3.9.4、3.5.3（示例原句）；4.2.4（"`blade` 参数遮蔽体内名为 `blade` 的节点"）；3.3.7、3.5.2 | 每台风机 `farm.N.t.{tower, nacelle, hub, rotor}`；`farm.N.t.rotor.r1 / .r2`，`farm.N.t.rotor.rK.blade` ghost、`of` = `farm.N.t.rotor.blade`（4.8.4）；`farm.1.t.rotor.r1` t = (26.2793, 33.1, −45.5425)；`defs: ["turbine"]`（条目形状见第 11 节 G1）；事实 `farm.N.t.rotor.rK.blade intersects farm.N.t.hub` 等（叶片互相穿过，6.5.5） |
| 22–24 | `repeat farm n=3 step=26,0,-8 center=true at=ground[0.35,1,-0.6] pin=origin { turbine t turn=-35 phase=idx*40 }` | 线性阵列沿斜线 `step`；`center`；阵列 `pin=origin`；模板实例语句带 P 组属性与参数；`idx` 在实例语句处可见 | 3.8.1、3.8.3（"线性：下标走整个 step 向量"）、3.8.5；4.2.3（示例原句）、1.4.7、1.5.8（实例写明的参数值在实例语句的上下文求值） | `farm`[repeat] + 3 实例；`farm.1.t` `kind: turbine`、`types: ["turbine"]`、t = (28, 0, −48)（= ground 的 (0.35·80, 0, −0.6·80)，idx = 1 居中）；`farm.N.t.tower on ground`（3 条） |
| 27–31 | `group sat at=-60,58,-72 turn=30 tilt=20 { box body size=2,2,3 pin=center mat=white / box wing size=6,0.1,2 against=body.right mat=panel / mirror wing2 of=wing across=body axis=x }` | 容器 `at=` 向量 + `turn` + `tilt`；`pin=center` 无关系；`against` 自动选面；副本式 mirror 过 body 中心 | 3.7.2（示例原句，t 的推导）；3.5.2（示例：$R_y(30°)R_x(20°)$ 的三列）、3.5.6；3.3.7、3.4.1；3.3.4；3.9.1、3.9.3 | `sat` t = (−59.8290, 58.9397, −71.7038)；`sat.wing2` `of: "sat.wing"`、`kind` ※（`mirror→box` → `box`）；`sat.wing against sat.body.right`、`sat.wing2 against sat.body.left` |
| 32–38 | `group radar on=ground[-0.55,1,-0.45] { cyl base r=3.5 h=4 / cyl mast r=0.7 h=5 on=base / revolve dish profile=[…] on=mast aimwith=top aim=sat / span feed from=dish.bottom to=dish.top+(0,0,0) r=0.1 / sphere horn r=0.4 at=dish.top pin=center }` | `aim` + `aimwith=top`；目标在容器外 → 后置；span 杆、零偏移；`at=` 锚点 + `pin=center` | 3.5.4（示例原句，$A$ 的第二列）、3.5.6；3.6.1、3.6.3（第一个例子就是 radar）、3.6.4；2.2.8（杆）、3.10.1、1.5.2；3.3.6 | `radar` box 高 9（= base ∪ mast；dish / feed / horn 不计入）；`radar.dish`、`radar.feed`、`radar.horn` 带 `late: true`（全部语料里只有它们与 `street.h4.skin` 后置，3.11.3）；`radar.dish` R 第二列 = (−0.2497, 0.7877, −0.5632)，t = (−44, 9, −36)；`radar.feed intersects radar.dish` gap −2.8 approx（※ 6.8.5：本版 `inside`）；`radar.mast on radar.base` |
| 41–44 | `repeat solar n=(5,1,4) step=5.5,0,4.5 center=true at=ground[0.5,1,0.45] pin=origin { cyl post r=0.12 h=1.6 mat=steel / box panel size=5,0.1,2.8 on=post tilt=25 mat=panel }` | 网格阵列 `n` 向量₃；`center`；`on` + `tilt` | 3.8.1、3.8.3（网格：三个下标各走 step 的一个分量）；3.5.2（tilt）、3.5.5 | `solar` 20 实例；`solar.7` t = (40, 0, 33.75)；`solar.N.post on ground`、`solar.N.post intersects solar.N.panel` −0.0507（各 20 条） |
| 47–56 | `group pisa on=ground[-0.15,1,0.5] roll=5 mat=white { cyl core r=4 h=42 / repeat galleries n=7 step=0,5.6,0 { cyl ring r=4.8 h=0.5 at=0,5.6,0 / rotate cols n=15 { cyl col r=0.18 h=5.1 at=4.55,0.5,0 } } / cyl belfry r=3.2 h=6 on=core }` | `roll`（顶部倒向 +x）；容器的 `mat` 继承；嵌套：repeat 体里的容器式 rotate（`angle` 缺省 360/16）；块内 `at=` 向量 | 3.5.2（roll 的定义以 `pisa roll=5` 为据）、3.7.3；10.3.2、10.4.4；3.8.3；3.9.2、3.9.4、1.4.5；3.3.6 | `pisa` t = (−12, 0, 40)；`pisa.galleries` 7 实例；`pisa.galleries.N.cols` 各带 `r1 … r15`，共 112 个节点；`pisa.galleries.N.cols.rK.col on pisa.galleries.N.ring`（90 条）、`pisa.core intersects ground` −0.3486（滚转使底角沉入）、`pisa.belfry on pisa.core`；`items[]` 里 pisa 下未写材质的部件都是 white |
| 59–70 | `group coaster at=ground[0.05,1,-0.05] pin=origin { spline track closed=true path=[…9 点…] / sweep railL path=track profile=[(-0.75,0); …] mat=red / sweep railR … / sweep spine … mat=steel / repeat ties n=110 along=track orient=full { box tie size=0.18,0.12,1.7 pin=top } / repeat legs n=20 along=track { span col from=(0,-0.7,0) drop=ground r=0.22 } }` | `at=` 锚点 + `pin=origin`；闭合样条；多边形截面的 sweep；`orient=full`；沿闭合曲线 `s = idx/N`；span 的 `drop` 向无限平面求交 | 3.3.6、3.4.1；2.3.6（示例原句）、3.10.5；2.2.9（示例原句 railL）；3.8.4（示例段落：ties 与 legs 全部 130 个实例一致）；3.10.2（示例原句 legs）、2.2.8（drop 的公式） | `curves[]` 里 `coaster.track` `closed: true`、108 点（9 个控制点 × 12，2.3.6）；`coaster.ties` 110 实例、`coaster.legs` 20 实例；`coaster.ties.10` t = (14.3161, 8.7211, 0.8554)；`coaster.legs.5.col` box y ∈ [0, 13.6252]（落到地面）；`coaster.legs.N.col touching ground`（20 条）、`coaster.ties.N.tie touching coaster.railL`（106 条） |
| 注释 / 空行 | 注释：1、12、17、26、40、46、58；空行：2、9、11、25、39、45、57 | — | 1.2.2 | — |

覆盖率：语句行 56（3–8、10、13–16、18–24、27–38、41–44、47–56、59–70），被上表覆盖 56，**100%**。

## 4 semantics.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 8–11 | `let bridge_len = 36` / `let n_piers = 4` / `let pier_h = 5` / `let house_w = 7` | 最外层 let（全局帧，惰性） | 1.3.5、1.5.8（全局帧；惰性求值）；5.3.5 | 没有节点；`check "桥墩数量与变量一致"` 为真（`count(pier) == 4`） |
| 13–22 | `material ground …` … `material brick color=#b5654a` | material（含 `metal` / `rough`） | 10.1.2、10.1.3 | `materials` 10 项 |
| 24 | `box ground size=120,1,90 pin=top mat=ground` | 同 basics:8 | 3.3.7 | `ground` |
| 25 | `box river size=bridge_len-6,0.05,90 on=ground mat=water` | 属性值里的算术与 let 引用 | 1.5.1、1.5.2；1.6.5 第 1 步 | `river` box x ∈ [−15, 15]（36 − 6 = 30 宽）；`river on ground` |
| 28–30 | `repeat piers n=n_piers step=(bridge_len-8)/(n_piers-1),0,0 center=true on=river mat=plain { cyl p r=0.9 h=pier_h }` | `n` 是引用 let 的常量；`step` 第一分量是带括号的表达式；线性阵列 `center`；阵列按包络 `on` | 3.8.1（n 可以引用 let）、3.8.3、3.8.5；1.5.1（tuple 的项是 or_expr）；1.4.1（常量） | `piers` 4 实例，box x ∈ [−14.9, 14.9]（间距 28/3）；`piers.N.p on river`（4 条） |
| 31 | `box slab size=bridge_len,0.6,6 on=piers.*.p mat=plain` | 通配集合作参照物（总包围盒顶面中心；继承世界朝向） | 1.5.4（通配）、1.7.1；3.2.8（示例原句）、3.3.2 | `slab` t = (0, 5.05, 0)；`slab on piers.N.p`（4 条）；大纲 `slab · on piers.0.p` |
| 32 | `span rampW from=slab.top-left to=(slab.left.x-12,0,slab.z) section=6,0.5 pin=top mat=plain` | 坐标分量 `锚点.x`、`B.z`；向量里的表达式 | 3.2.7（示例原句）；2.2.8、3.10.1 | `rampW`；`rampW intersects slab` −0.213、`rampW intersects ground` −0.4524；大纲 `rampW span 12×6.1×6 〔wall〕` |
| 33 | `mirror rampE of=rampW across=slab axis=x` | 副本式 mirror | 3.9.1、3.9.3（第四行）；4.4.1 | `rampE` t = (24, 2.825, 0)（rampW 关于 x = 0 的镜像），`kind` ※（`mirror→span` → `span`）；`rampE intersects slab` |
| 34–36 | `repeat rails n=(12,1,2) step=bridge_len/11.5,0,5.6 center=true on=slab mat=plain { cyl post r=0.08 h=1.1 }` | 网格阵列 | 3.8.1、3.8.3 | `rails` 24 实例；`rails.N.post on slab`（24 条） |
| 38–41 | `repeat lamps n=n_piers step=… center=true at=slab.top pin=origin mat=plain { cyl pole r=0.12 h=4 at=0,0,2.4 / sphere bulb r=0.35 on=pole }` | 阵列 `at=` 锚点 + `pin=origin`；体内 `at=` 向量 | 3.8.3、3.7.2、3.3.6 | `lamps` t = (0, 5.65, 0)，4 实例；`lamps.N.bulb on lamps.N.pole`（4 条） |
| 44–49 | `def house(w=house_w, floors=1) { box body size=w,3*floors,w*0.8 mat=plain / wedge half size=w/2,2.2,w*0.8 on=body align=left / mirror half2 of=half across=body axis=x / box door size=1.2,2.2,0.2 against=body.front align=bottom }` | 默认值引用全局 let；模板体内的副本式 mirror（每个实例一个）；`align=left`；`against` | 4.2.2（示例 `def f(… c=house_w)`）、4.2.4（示例原句）；4.4.1；3.4.3、3.3.4 | 每栋房子 `street.hN.{body, half, half2, door}`；`street.h1.half2 against street.h1.half.right`（6.5.6 的平局例子）；half2 `kind` ※（`mirror→wedge` → `wedge`） |
| 51–53 | `def chimney_house : house(floors=2) { box chimney size=0.8,3,0.8 on=body[0.55,1,0.3] mat=plain }` | 派生模板、覆盖表、派生体引用基体的节点 | 1.3.4（示例原句）；4.2.5（示例原句）、4.2.7 | `street.h2`、`street.h3` `kind: chimney_house`、`types: ["chimney_house","house"]`，子节点顺序 `body half half2 door chimney`，body 高 6；`concept building` 含它们 |
| 54–59 | `group street on=ground[0.6,1,0] { house h1 at=-7,0,-8 / chimney_house h2 at=7,0,-8 / chimney_house h3 at=-7,0,8 / house h4 floors=3 at=7,0,8 }` | 模板实例语句（默认 / 写明的参数；`at=` 向量） | 4.2.3（示例原句 `house h4 floors=3 at=7,0,8`）、1.3.3（示例原句）、3.3.6 | `street.h2` t = (43, 0, −8.1)；`street.h4.body` 高 9；`street.h4` `hidden: true`（随变体，第 130–131 行）；大纲 `street · on ground`（7.2.4 的上提） |
| 62–66 | `group watch on=ground[-0.55,1,-0.45] mat=plain { repeat legs n=(2,1,2) step=3,0,3 center=true { box leg size=0.3,9,0.3 } / box cabin size=4.2,2.5,4.2 on=legs / wedge cap size=4.2,1.5,4.6 on=cabin }` | 单行块（`{ … }` 与语句头同一逻辑行）；repeat 节点作参照物（按包络）；容器的 `mat` 继承 | 1.3.2；3.7.1、3.7.5（repeat 节点的包络）、3.3.2；10.4.4 | `watch` 子节点 `legs cabin cap`；`watch.cabin on watch.legs.N.leg`（4 条）、`watch.cap on watch.cabin`；`concept stilt` 含 `watch.legs.N.leg` |
| 68–72 | `repeat trees n=(4,1,3) step=7,0,6.5 center=true on=ground[-0.55,1,0.45] mat=plain { let s = 0.8 + 0.12*i / cyl trunk r=0.3*s h=3*s / sphere crown r=1.6*s on=trunk }` | repeat 体内的 `let`（块帧）读阵列帧的 `i` | 1.5.8（块帧、阵列实例帧；`i j k idx` 在 repeat 体含其中的 let 里可见）、3.8.2、3.8.3 | `trees` 12 实例；`trees.5.trunk`（i = 1，s = 0.92）h = 2.76、t.x = −36.788；`trees.N.crown on trees.N.trunk`（12 条） |
| 73 | `box crate size=1.5,1.5,1.5 at=ground[-0.3,1,0.3]+(0,2.5,0) mat=plain` | 锚点加偏移作 `at=` | 1.5.2、3.2.4、3.3.6 | `crate` t = (−18, 2.5, 13.5)；没有任何事实；`check "没有悬空的实体"` 的 violators = ["crate"]；大纲 `crate 〔prop〕` |
| 76 | `let solids = kind(box, cyl, cone, sphere, wedge)` | 集合值的最外层 let，只在语义层被读 | 1.5.8（几何时机读到的 let 不能用 all() 等；语义层可以）、1.5.6；5.4.1 第 6 条；7.2.1 | 第 138 行检查的结果 |
| 78–88 | `concept pier = kind(cyl) & on(river) & supports(kind(box))` … `concept tree = …` | concept；关系谓词（含第二个参数 `on(pier, 2)`、`on(stilt, 4)`）；`where` 字段 `h`、`w`；运算优先级（`&` 高于 `\|`，`where` 最低，括号）；`river`、`ground` 从根解析；概念互相引用 | 1.3.9（示例原句 pier）、1.5.1（优先级）、1.5.5；7.4.1（示例原句）、7.4.3、7.4.4；7.2.3、7.2.4（示例原句 pier / deck / tree）；7.3.1；7.2.1；6.7.1、6.7.2 | `concepts[]` 11 条，成员数：pier 4、deck 1（slab）、bridge 7、railing 24、lamp 8（※ 成员顺序，7.9.4）、roof 9、building 4、tall_building 1（street.h4）、stilt 8、cabin 2、tree 24（※ 顺序）；节点标注 `slab.concepts` = ["crossing.deck","deck","bridge"]、`rampW` = ["crossing.ramps","bridge"] |
| 91–98 | `style pier mat=stone` / `style deck mat=wood` / `style bridge-pier-deck mat=stone` / `style railing\|lamp mat=metal` / `style roof mat=tile` / `style cabin mat=wood` / `style tree mat=leaf` / `style kind(cyl)&tree mat=wood` | style；选择器是概念表达式；`bridge-pier-deck` 是减法（`bridge - pier - deck`）；`\|` `&` 不要求空白；后写的样式赢 | 1.3.12（示例原句）、1.2.6（"`bridge-pier-deck` 都是减法"）、1.2.8；7.7.1、7.7.2、7.7.3（示例原句）；10.3.3、10.5.1 | `items[]` 的材质：`piers.N.p` stone、`rampE` stone、`trees.N.trunk` wood / `trees.N.crown` leaf、`watch2.cabin` wood、`slab.skin.pl.N.plank` wood（经 $E(N)$，10.4.3）；10.8 表 |
| 101–108 | `pattern crossing(deck, piers*, ramps*) { deck in kind(box) / piers in pier / ramps in kind(span) / deck on piers / ramps touching deck / count(piers) >= 2 }` | pattern；标量 / 集合角色；域约束、关系约束、布尔约束；角色名遮蔽同名概念 `deck` 与节点 `piers` | 1.3.10（示例原句）；7.5.1、7.5.2（角色名遮蔽）、7.5.3、7.5.4（示例原句，逐条标注）、7.5.5 | `patterns[0]`：roles ["deck","piers*","ramps*"]，1 个匹配 {deck: slab, piers: 4 根 p, ramps: [rampW, rampE]}；节点标注 `crossing.deck`、`crossing.ramps` |
| 109–115 | `pattern home(body, roof*, door) { body in kind(box) where h > 2 / roof in kind(wedge) / door in kind(box) where h < 2.5 and d < 0.5 / roof on body / door against body }` | 同上；域里的 `where`；角色 `roof` 遮蔽概念 roof | 7.5.1（示例原句）、7.5.3、7.5.4 | `patterns[1]` 4 个匹配（h1 … h4 各一，roof 含 half 与 half2）；节点标注 `home.body` 等 |
| 118–122 | `def planks(w=1, h=1, d=1) { repeat pl n=floor(w/0.9) step=w/floor(w/0.9),0,0 center=true { box plank size=w/floor(w/0.9)-0.08,h,d } }` | 变体模板的 w / h / d 接口；`n` 是含 `floor()` 与参数的常量 | 4.7（"语料里的三个皮肤"）、7.8.3；3.8.1；1.5.6（floor） | `slab.skin.pl` 40 实例（floor(36/0.9) = 40）；`slab.skin.pl.N.plank` 0.82 × 0.6 × 6 |
| 123–129 | `def townhouse(w=6, h=8, d=5) { box core size=w,h-0.6,d mat=brick / box parapet size=w+0.3,0.6,d+0.3 on=core mat=plain / repeat win n=(3,2,1) step=w/3.3,2.4,0 center=true at=core.front+(0,0.2,0) pin=back { box pane size=1.1,1.5,0.12 mat=glass } }` | `at=` 锚点加偏移 + `pin=back`；网格阵列 n=(3,2,1)；皮肤部件自己的 `mat` | 3.3.6（示例 `at=core.front+(0,0.2,0)`）、3.4.1；3.8.3；10.4.3（皮肤自己写的材质优先） | `street.h4.skin.core` 7 × 10.6 × 5.8；`street.h4.skin.win` 6 个 pane；`items[]` core brick、pane glass、parapet plain |
| 130 | `variant deck use=planks` | variant；自动传参 w h d；皮肤定位到宿主包络底面中心 | 1.3.13；7.8.1（示例原句）、7.8.2、7.8.3、7.8.4；4.7；3.7.6 | `variants[0]` {line 130, use planks, members ["slab"]}；`slab` `hidden: true`；`slab.skin`[planks] `skin: true`，t = (0, 5.05, 0) = slab 的底面中心 |
| 131 | `variant tall_building use=townhouse` | 同上；宿主是模板实例（容器）→ 皮肤后置 | 7.8.3；3.7.6（参考输出 `street.h4.skin` 为 late 的说明）；4.4.5（h4 下的副本式节点 half2 随宿主隐藏） | `variants[1]` members ["street.h4"]；`street.h4` 及其 body、half、half2、door 共 5 个 `hidden`（全文件 6 个 hidden，另一个是 slab）；`street.h4.skin` `late: true`，t = (43, 0, 8) |
| 134–138 | `check "桥墩数量与变量一致" count(pier) == n_piers` / `check "每栋房子都有屋顶" kind(house) where count(descendants(it) & roof) == 0` / `check "每座房子都匹配到 home 模式" count(home) == count(kind(house))` / `check "高脚屋的每根桩都落地" stilt - on(ground)` / `check "没有悬空的实体" solids - on(all()) - ground` | check（布尔 / 集合两种结果）；`it` 作集合；`count(模式)`；对象 `ground` 转单元素集合；集合差 | 1.3.11；7.6.1（示例原句）、7.6.3；7.3.1（`it`）；7.5.7（`count(home)`）；7.2.3 第 2 条；1.5.5 | `checks[]` 5 条：ok = true、true、true、true、false；最后一条 violators = ["crate"] |
| 141–142 | `over watch.cabin size=5,2.5,5` / `over watch.cap size=5,1.5,5.4` | 几何时机的 over（含 `{mat,color,use}` 之外的键）；就地修改 | 1.3.8；4.5.1、4.5.2、4.5.3（示例原句 `over watch.cabin size=5,2.5,5`） | `watch.cabin` 5 × 2.5 × 5（大纲第 22 行）、`watch.cap` 5 × 1.5 × 5.4；`watch2.cabin` 仍是 4.2 宽 |
| 143–146 | `copy watch2 of=watch on=ground[-0.3,1,-0.62] { drop cap / repeat rail n=(2,1,2) step=3.8,0,3.8 center=true on=cabin { cyl post r=0.06 h=1 } }` | copy 复制定义（不含对原件的 over）；copy 的 `on=` 替换原定位；copy 块 = 对 R 的 over 块（`drop cap` 删 watch2.cap；`on=cabin` 绑定 watch2.cabin）；块的展开位置 | 1.3.14；4.3.1、4.3.2（第 3 条用了这句）、4.3.3、4.3.4（示例原句）、4.3.5；4.1.2（示例原句）、4.5.6（示例）；10.4.2 | `watch2` t = (−18, 0, −27.9)（= ground 的 (−0.3·60, ·, −0.62·45)），子节点 `legs cabin rail`（没有 cap）；`watch2.cabin` 4.2 × 2.5 × 4.2；`watch2.rail.0.post` t = (−19.9, 11.5, −29.8)；`watch2.rail.N.post on watch2.cabin`（4 条）；`watch2.cabin` 材质 wood（`style cabin` 选中两个成员） |
| 注释 / 空行 | 注释：1–5、7、27、37、43、50、61、67、75、90、100、117、133、140；空行：6、12、23、26、42、60、74、77、89、99、116、132、139 | — | 1.2.2 | — |

覆盖率：语句行 115（8–11、13–22、24–25、28–36、38–41、44–49、51–59、62–66、68–73、76、78–88、91–98、101–115、118–131、134–138、141–146），被上表覆盖 115，**100%**。

## 5 spaceport.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 4–10 | `material ground color=#59604f rough=1` … `material hazard color=#e0a92e` | material | 10.1.2、10.1.3 | `materials` 7 项；spaceport_blueprint 用同名重定义覆盖它们（第 6 节） |
| 12 | `box ground size=200,1,200 pin=top mat=ground` | 同 basics:8 | 3.3.7 | `ground` |
| 15 | `box pad size=26,2,26 on=ground[0.25,1,0] mat=concrete` | `on` 带锚点 | 3.3.2 | `pad` t = (25, 0, 0)；`pad on ground` |
| 17–27 | `group rocket on=pad mat=white { cyl stage1 r=2.4 h=28 / cyl band r=2.45 h=0.8 on=stage1 gap=-0.8 mat=hazard / cyl stage2 r=2 h=14 on=stage1 / revolve nose profile=[…] smooth=true on=stage2 / rotate fins n=3 about=stage1 { wedge fin size=3.5,6,0.4 turn=180 at=stage1.w/2,0,0 pin=bottom-right mat=hazard } }` | 容器按包络定位；负 `gap`；revolve；容器式 rotate，`about` 引用兄弟、`angle` 缺省 90°；`at=` 向量的分量是表达式（度量 `stage1.w`）；`pin` 组合词同时是旋转支点；`turn=180`；容器 `mat` 的继承与覆盖 | 3.7.2（示例原句 rocket）、2.6（示例原句）；3.4.2（示例原句 band）；2.2.7；3.9.2（示例原句 fins：$R_y(90°)$，$t = 0$）、3.9.4、1.4.5；3.3.6（示例 `at=stage1.w/2,0,0`）、3.2.6；3.4.1（示例 `pin=bottom-right`，"定位点同时是旋转支点"）、2.2.2（示例原句 fin）；10.2（示例）、10.3.2、10.4.1 | `rocket` t = (25, 2, 0)，box 高 49；`rocket.band intersects rocket.stage1` −0.8；`rocket.stage2 on rocket.stage1`、`rocket.stage2 on rocket.band`、`rocket.nose on rocket.stage2` approx；`rocket.fins` + `r1 r2 r3`，`rocket.fins.rK.fin` ghost；`rocket.fins.r1` t = (25, 2, 0)、R = $R_y(90°)$；大纲 `stage1 〔member column volume tank〕` |
| 30–39 | `def truss(w=6, h=48, lv=8) { repeat legs n=(2,1,2) step=w,0,w center=true { box leg size=0.6,h,0.6 } / repeat decks n=lv step=0,h/lv,0 { box deck size=w+1.2,0.4,w+1.2 at=0,h/lv-0.4,0 / span brace from=(-w/2,0.4,w/2) to=(w/2,h/lv-0.4,w/2) r=0.15 } }` | 三个可选参数；`n` 引用参数的常量；`step` / `at` 的表达式；span 杆的向量端点；模板体内注释 | 4.2.1、4.2.2（示例原句）；3.8.1（n 可以引用模板参数）、3.8.3；3.10.1；2.2.8 | `tower.legs` 4 实例、`tower.decks` 8 实例，各带 deck 与 brace；`defs` 含 `truss`（条目形状见 G1） |
| 41 | `truss tower on=pad[-0.7,1,-0.7] mat=steel` | 模板实例节点是容器，按包络定位；`mat` 落在实例节点上 | 4.2.3（示例原句）、3.7.2；10.4.4（示例） | `tower` `kind: truss`、`types: ["truss"]`，t = (15.9, 2, −9.1)，box min (12.3, 2, −12.7)；`items[]` 里 `tower.legs.N.leg`、`tower.decks.N.deck` 都是 steel |
| 44 | `span arm from=tower.decks.5.deck.top-right-front to=rocket.stage2[-0.6,0.14,-0.6] section=1.6,0.3 pin=bottom mat=steel` | 经阵列实例段的路径；三个方向词的组合词；归一化锚点（分量不限于 ±1）；端点在不同容器里 | 1.6.2、1.2.6；3.2.2；3.10.1（"端点可以在不同容器里…`tower.decks.5.deck.top-right-front`"）；2.2.8 | `arm` t = (21.65, 37.99, −3.35)；`items[]` geom {beam, L = 6.0812, w 1.6, t 0.3, dy 0.15}；`arm intersects rocket.stage2` −0.2358；大纲 `arm · touching tower.decks.5.deck` |
| 47–48 | `mirror tower2 of=tower across=rocket axis=x` / `mirror arm2 of=arm across=rocket axis=x` | 副本式 mirror 过容器盒心、法线取容器的 x 轴；副本树是原件的函数；副本的材质链 | 3.9.1（示例原句 tower2）、3.9.3（第四行）；4.4.1（示例原句）、4.4.2、4.4.5；10.4.1 | `tower2` t = (34.1, 2, −9.1)，R = diag(−1, 1, 1)；`tower2.legs.N.leg`、`tower2.decks.N.deck` ghost、`of` 指向 tower 的部件；`kind` ※（`mirror→group` → `truss`）、`types` ※（null → ["truss"]）；`tower2.legs.N` `kind` ※（`group` → `instance`）；`arm2 intersects rocket.stage2`；`items[]` 里 tower2 的部件都是 steel |
| 51–54 | `repeat lights n=(2,1,2) step=40,0,40 center=true at=pad.bottom pin=origin mat=steel { cyl pole r=0.4 h=30 / box lamp size=2.4,1.2,2.4 on=pole mat=white }` | 网格阵列；阵列原点钉在锚点 | 3.7.2（示例原句）、3.8.3；10.4.4（示例原句） | `lights` t = (25, 0, 0)，4 实例；`lights.N.lamp on lights.N.pole`（4 条）；pole steel、lamp white |
| 57–66 | `def shed(w=34, h=22, d=26, t=0.8, dw=12) { subtract body mat=concrete { box outer size=w,h,d / box inner size=w-2*t,h-t,d-2*t / box opening size=dw,h*0.7,t*3 at=outer.front pin=center align=bottom / cyl arch r=dw/2 h=t*3 axis=z at=opening.top pin=center } / box roof size=w+2,0.8,d+2 on=body mat=steel }` | 模板体里的布尔容器；`at=` 引用形式作 `align` 的对齐目标；`cyl axis=z`（原点在盒底面中心，轴线在 y = R）；布尔容器作 `on` 的参照物（subtract 的盒 = 第一个操作数） | 4.2.2（示例原句）；2.5.1、2.5.4、2.5.6（示例 body）；3.4.3（对齐目标"或 `at=` 引用形式的参照物"）、3.3.6；2.2.3（axis=z 的轴系与盒）；3.7.5；10.4.4（示例） | `hangar.body`[subtract] 4 个操作数；`hangar.body.arch` t = (−45, 9.4, −7)，box y ∈ [9.4, 21.4]（圆心钉在 opening 顶 15.4）；`hangar.roof on hangar.body`、`hangar.body on ground`（※ approx）；`items[]` body 条目 concrete、roof steel |
| 68 | `shed hangar on=ground[-0.45,1,-0.2]` | 模板实例（全部默认参数） | 4.2.3（示例原句） | `hangar` `kind: shed`，t = (−45, 0, −20)，box 36 × 22.8 × 28；大纲 `body subtract 34×22×26 〔volume〕` |
| 71 | `cyl spare r=2.4 h=20 axis=x inside=hangar.body align=bottom mat=white` | `cyl axis=x`（局部盒 20 × 4.8 × 4.8）；`inside` 布尔容器 + `align=bottom` | 2.2.3（示例原句）、2.1.3；3.3.5（示例原句）、3.4.3（示例原句）；6.5.4 | `spare` t = (−45, 0, −20)，box x ∈ [−55, −35]；`spare inside hangar.body` −4.8（※ approx）；大纲 `spare cyl 20×4.8×4.8 〔member beam volume tank〕`（7.9.3 的要点） |
| 74–78 | `mirror wings across=hangar.body axis=x { box annex size=12,8,14 against=hangar.body.right align=back,bottom mat=concrete / span stair1 from=annex.bottom-front+(0,0,12) to=annex.top-front steps=16 section=2.4,0.25 mat=steel / span stair2 from=annex[1,1,0] to=hangar.roof[1,1,-0.43] steps=20 section=2,0.2 mat=steel }` | 容器式 mirror，镜像面引用容器外的节点（C 自己没有关系，不后置）；`against` + 两项 `align`；归一化锚点作 span 端点；多段路径 `hangar.body.right` 从根解析 | 3.9.2（"镜像面引用容器外的节点时…语料的写法"）、3.9.3；4.4.1（示例原句）、4.4.5（wings 的盒跨两翼）；3.3.4、3.4.3；3.2.2、3.10.1；1.6.5 | `wings` 子节点 `annex stair1 stair2 mirrored`，box x ∈ [−74.00, −16.00]；`wings.mirrored.annex` ghost、`of: "wings.annex"`，t = (−68, 0, −26)；`wings.annex against hangar.body.right`、`wings.mirrored.annex against hangar.body.left`（※ approx）；`wings.stair2 touching hangar.roof` −0.0018 |
| 81–82 | `spline roadline path=[hangar.body.bottom-front+(0,0,10); (-28,0,12); (-4,0,9); pad.bottom-left+(-6,0,0)]` / `sweep road path=roadline profile=rect section=9,0.08 pin=bottom mat=asphalt` | 开放样条，路径项混用锚点加偏移与向量；局部系就是父系；沿样条扫掠 | 2.3.6（示例原句）、2.3.4、3.10.5；2.2.9、3.10.4 | `curves[]` 里 `roadline` `closed: false`、37 点（3 段 × 12 + 1），box x ∈ [−45, 6]；`road on ground`；大纲 `road sweep 57×0.1×20 〔slab platform〕` |
| 85–89 | `repeat tanks n=3 step=10,0,0 center=true on=ground[0.3,1,0.55] mat=white { cyl tank r=3.5 h=9 / sphere cap r=3.5 on=tank gap=-3.5 / cyl ring r=3.6 h=0.6 on=tank gap=-3 mat=hazard }` | 线性阵列；负 `gap`（球心落在圆柱顶面）；阵列的 `mat` 与部件自己的 `mat` | 3.8.3；3.4.2；2.2.4（示例原句 cap）；10.4.4（示例原句） | `tanks` 3 实例；`tanks.0.cap` t = (20, 5.5, 55)；`tanks.N.cap intersects tanks.N.tank`、`tanks.N.ring intersects tanks.N.tank`；`items[]` tank white、ring hazard；大纲 `tank 〔… tank〕`（库概念） |
| 92–95 | `bezier feedline1 path=[tanks.0.tank[0,-0.7,-1]; tanks.0.tank[0,-0.7,-1]+(0,0,-22); pad.top-front+(-4,0.4,14); pad.top-front+(-4,0.4,0)]` / `bezier feedline2 …` / `sweep feed1 path=feedline1 r=0.35 mat=steel` / `sweep feed2 …` | Bézier 控制点是经阵列段的归一化锚点（含加偏移）；圆截面 sweep（`r`，20 点截面） | 2.3.5（示例）、2.1.6.5、3.10.5；1.6.2、3.2.2、3.2.4；2.2.9 | `curves[]` 里 `feedline1`、`feedline2` 各 65 点（2.3.5：$t = j/64$）；`feed1`、`feed2`；大纲 `feed1 · touching tanks.0.tank`；spaceport_edit 里它们随 `over tanks n=5` 移动（5.5.3） |
| 98–101 | `box bunker size=14,5,10 on=ground[-0.05,1,0.55] turn=30 mat=concrete` / `wedge berm size=7,5,10 against=bunker.left align=back,bottom mat=earth` / `cyl mast r=0.25 h=7 on=bunker[0.6,1,0] mat=steel` / `sphere radome r=1.6 on=mast mat=white` | `turn`；贴在转过物体上的东西继承 30°；`on` 带锚点 | 3.5.2、3.5.1（"`bunker turn=30` 上的 `mast`、`berm` 都是 30°"）、3.5.5；3.3.4（示例原句 berm）、3.4.3 | `bunker` t = (−5, 0, 55)，box 17.12 × 5 × 15.66（转过的 AABB）；`berm` t = (−14.0933, 0, 60.25)；`berm against bunker.left`、`mast on bunker`、`radome on mast`（事实在 bunker 的局部系里判定，6.5.2） |
| 104–112 | `group spiral on=ground[-0.18,1,0.5] mat=steel { cyl core r=0.35 h=6.5 / helix coil r=1.2 pitch=6.24 turns=1 / repeat treads n=24 along=coil { box tread size=0.7,0.12,2 pin=top } / helix railline r=2.1 pitch=6.24 turns=1 at=0,1,0 pin=origin / sweep rail path=railline r=0.05 }` | helix（91 点）；沿开放曲线的阵列（两端各一个实例）；`pin=top` 把踏面顶放到螺旋线上；helix 的 `at` / `pin=origin`；圆截面 sweep | 2.3.3；3.8.4（"`box tread pin=top` 把踏面顶放到螺旋线上"）；3.10.6；2.2.9 | `spiral.treads` 24 实例；`spiral.treads.0` t = (−16.7928, 0.12, 50)；`curves[]` 里 `spiral.coil`、`spiral.railline` 各 91 点（2.3.3）；大纲 `core · on spiral.treads.0.tread 〔member column〕`、`rail sweep 4.3×6.3×4.3 〔volume〕` |
| 注释 / 空行 | 注释：1、2、14、21、23、29、36、43、46、50、56、62、70、73、80、84、91、97、103；空行：3、11、13、16、28、40、42、45、49、55、67、69、72、79、83、90、96、102 | — | 1.2.2 | — |

覆盖率：语句行 75（4–10、12、15、17–20、22、24–27、30–35、37–39、41、44、47–48、51–54、57–61、63–66、68、71、74–78、81–82、85–89、92–95、98–101、104–112），被上表覆盖 75，**100%**。

## 6 spaceport_blueprint.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 4 | `import "spaceport.s3d"` | import；相对路径按写它的文件所在目录；语句级内联；被导入的语句在 $P$ 里先于本文件其余语句 | 1.3.6、1.8；5.1.2、5.2.1、5.2.2、5.2.3、5.2.4 | 193 个 `file: examples/spaceport.s3d` 的节点与基线逐位相同（5.5.3）；`errors: []` |
| 6–12 | `material ground color=#14365e rough=1` … `material hazard color=#f3c969 rough=0.8` | 同名材质后写整体替换（跨文件同一规则）；重定义已有名字属于外观层 | 10.1.3；5.3.4、5.4.1 第 7 条、5.4.5 | `materials` 仍是 7 个名字；`steel` = {#9fc4ea, 0.1, 0.6}、`ground` = {#14365e, rough 1}；`items[]` 全部换用新值 |
| 15–18 | `def ribbed(w=1, h=1, d=1) { box base size=w,h*0.35,d / repeat rib n=floor(w/1.6) step=w/floor(w/1.6),0,0 center=true on=base { box r size=0.3,h*0.65,d } }` | 只被 `use=` 引用的模板（外观层）；w / h / d 接口；节点名 `r`（属性名不是保留名） | 4.7（"语料里的三个皮肤"）、7.8.3；5.4.1 第 5 条、5.4.3；1.6.9（"节点可以叫 r、gap"） | `defs: ["truss","shed","ribbed"]`；`hangar.roof.skin.base` 36 × 0.28 × 28（0.35 h）；`hangar.roof.skin.rib` 22 实例（floor(36/1.6)）；皮肤子树 47 个 `skin: true`，`concepts: []` |
| 19 | `over hangar.roof use=ribbed` | 外观时机的 over（最外层、无块、无换形、键 ⊆ {mat, color, use}）→ 登记为 variant；挂载、隐藏、皮肤定位 | 1.3.8（示例原句）；4.5.1（示例原句）、4.7；7.8.1（示例原句）、7.8.2、7.8.3、7.8.4、7.8.9；3.7.6；5.4.1 | `variants[]` 恰一条 {line 19, use ribbed, src "hangar.roof", members ["hangar.roof"]}；`hangar.roof` `hidden: true`；`hangar.roof.skin`[ribbed] t = (−45, 22, −20) = roof 的底面中心，box 与 roof 相同；`skin` 的 `file` ※（参考 null，本版 spaceport_blueprint.s3d）；203 条事实与基线逐条相同（6.1.2、7.1.1） |
| 20 | `over wings.annex color=#cfe3f7` | 外观时机的 over → 登记为 style；样式压过自有 `mat`；影子跟随 | 4.5.1（示例原句）；7.7.4；10.3.3、10.4.1（原句）、10.5.2 | `items[]` 里 `wings.annex` 与 `wings.mirrored.annex` 的 `material` 都是 {color #cfe3f7}（※ 10.6 形状：本版 concrete 改色，name / rough 保留）；没有 style 的输出条目 |
| 注释 / 空行 | 注释：1–3、14；空行：5、13 | — | 1.2.2 | — |

覆盖率：语句行 14（4、6–12、15–20），被上表覆盖 14，**100%**。文件整体满足 5.4.5 的"外观层文件"充分条件，5.4.4 的保证在参考输出上成立（5.5.3）。

## 7 spaceport_edit.s3d

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 4 | `import "spaceport.s3d"` | 同第 6 节第 4 行 | 1.8；5.1.2、5.2.3 | 根的 24 个子节点 = 基线 23 个 + `hangar2`（5.5.3） |
| 6 | `over rocket.stage2 h=20` | 几何时机的 over（`h` 不是外观键）；就地修改；依赖它的节点在求解时跟动 | 4.5.1（示例原句）、4.5.3（示例原句）；4.8.1 | `rocket.stage2` box y ∈ [30, 50]（基线 [30, 44]）；`rocket.nose` 从 50 起；`arm` 终点上移 |
| 7 | `over tower lv=10` | 重建型修改（模板参数）；P 组保留；副本按重建后的原件生成 | 4.5.4（示例原句）、4.1.3；4.4.2；4.8.1 | `tower.decks` 10 实例；`tower.decks.9.deck` y ∈ [49.6, 50]；tower 的 box min 仍 (12.3, 2, −12.7)；`tower2.decks.9.deck` ghost，t = (34.1, 49.6, −9.1) |
| 8 | `over hangar dw=16` | 重建型修改（模板参数）：body 重新展开 | 4.5.4（示例原句） | `hangar.body.opening` x ∈ [−53, −37]（16 宽；基线 12）；`hangar.body.arch` 宽 16 |
| 9 | `over tanks n=5` | 重建型修改（阵列 `n`）；`center=true` 保留 | 4.5.4（示例原句）、4.5.3 第 3 条 | `tanks` 5 实例；`tanks.0.tank` x ∈ [6.5, 13.5]，间距 10；`feedline1 / 2`、`feed1 / 2` 随 `tanks.0.tank`、`tanks.2.tank` 移动 |
| 10 | `drop lights.3` | drop 单个阵列实例；其余不重编号 | 1.3.8（示例原句）；4.5.6（示例原句）、1.6.2 | `lights` 下只有 `0 1 2`；节点数 217 = 193 + 6 + 6 + 8 − 3 + 7（5.5.3 的加减法） |
| 13–15 | `copy hangar2 of=hangar on=ground[-0.45,1,0.42] { over roof color=#4a5560 }` | copy 复制源语句（不含第 8 行对 hangar 的修改，不论书写先后）；copy 的 `on=` 替换原定位；copy 块里的 over 是几何时机的（即使只有 `color`），`color` 成为 hangar2.roof 的自有属性；copy 占自己语句的位置 | 1.3.14；4.3.1、4.3.2（第 4、7 条引用了这句）、4.3.3、4.3.4、4.3.5；4.5.1（示例原句）、5.4.1 第 2 条；10.4.2、10.5.2；1.6.1、5.3.2 | `hangar2` `kind: shed`、t = (−45, 0, 42)，子节点 `body roof`；`hangar2.body.opening` 12 宽（模板默认，不是 16）；`hangar2.roof` 条目 `material` = {color #4a5560}（※ 10.6：本版 steel 改色）；`hangar2` 的 `file` / `line` ※（参考 spaceport.s3d:68，本版 spaceport_edit.s3d:13）；`hangar2.body on ground`、`hangar2.roof on hangar2.body`（※ approx）；根子节点以 `hangar2` 结尾 |
| 注释 / 空行 | 注释：1–3、12；空行：5、11 | — | 1.2.2 | — |

覆盖率：语句行 9（4、6–10、13–15），被上表覆盖 9，**100%**。

## 8 lib/semantics.s3d

库文件没有单独的参考编译输出；它的效果体现在三份大纲（`*.outline.txt` 由 `--semantics` 把本文件追加到场景末尾后生成）末尾的 〔…〕 标注里，7.9.3 已按本规范逐节点核对（semantics 24/24、spaceport 41/41、spaceport_blueprint 41/41 一致）。下表第 5 列引用大纲里的标注与 7.9.3 算得的成员数。

| 行 | 语句 | 构件 | 条款 | 参考输出里对应的可核对项 |
|---|---|---|---|---|
| 11 | `let solid_kinds = kind(box, cyl, cone, sphere, wedge, plate, hull, revolve, span, sweep, pipe, mesh, union, subtract, intersect)` | 集合值的最外层 let；`kind()` 多参数（含布尔容器种类） | 1.3.5、1.5.6、1.6.7；5.4.1 第 6 条；7.2.1（let 在语义层被读到时合法） | 下面各概念的候选集 |
| 14 | `concept terrain = solid_kinds where min(w,h,d) < 3 * metre and w+h+d-max(w,h,d)-min(w,h,d) > 40 * metre and top - bottom < 3 * metre` | `where` 字段 `w h d top bottom`；`min` / `max`；内置常量 `metre`；`and` | 7.3.1（字段表；"语义库用它判断竖放还是平放"）、1.7.3；1.5.6；1.5.8（`metre`）；1.5.5 | 大纲 `ground 〔terrain slab〕`（三个文件）；成员数 semantics 1、spaceport 1 |
| 16 | `concept slab = solid_kinds where min(w,h,d) < 2.5 * metre and … and top - bottom < 1.5 * min(w,h,d) + 0.01 * metre` | 同上 | 7.3.1、7.4.1 | 大纲 `river 〔slab platform〕`、`pad 〔slab platform〕`、`road 〔slab platform〕`；成员数 semantics 3、spaceport 27 |
| 17 | `concept platform : slab = (on(all()) \| supports(all())) - terrain` | 层级概念的第二形态（`=` 右边不以 `where` 开头 → `slab & (…)`）；`on(all())`、`supports(all())`；集合差 | 1.3.9（示例原句）；7.4.2（示例原句）、7.2.4；1.5.5 | 大纲 `river 〔slab platform〕`、`roof 〔slab platform〕`、`band 〔slab platform〕`；成员数 semantics 2、spaceport 4 |
| 19 | `concept wall = solid_kinds where min(w,h,d) < 0.8 * metre and … and top - bottom > 0.95 * (…)` | `top − bottom` 对实体按紧包围盒（倾斜的 span 因此算 wall） | 7.3.1（`top bottom` 行）、7.9.3（要点：rampW / rampE 是 wall） | 大纲 `rampW span 12×6.1×6 〔crossing.ramps bridge wall〕`、`rampE 〔… wall〕`；成员数 semantics 2、spaceport 4 |
| 21 | `concept member = solid_kinds where max(w,h,d) > 4 * (w+h+d-max(w,h,d)-min(w,h,d))` | `w h d` 按局部盒、不随朝向变 | 7.3.1（示例原句）；2.1.3 | 大纲 `spare cyl 20×4.8×4.8 〔member beam volume tank〕`（横放圆柱的 w = 20 是局部盒边长）；成员数 semantics 53、spaceport 33 |
| 23 | `concept column : member = where max(w,h,d) > 2.5 * metre and top - bottom > 0.95 * max(w,h,d)` | 层级概念的第一形态（`= where P` → `member where P`） | 1.3.9（示例原句）；7.4.2（示例原句）、7.3.1（示例原句） | 大纲 `stage1 〔member column volume tank〕`、`mast 〔member column〕`、`core 〔member column〕`；成员数 semantics 21、spaceport 15 |
| 24 | `concept beam : member = where top - bottom < 0.5 * max(w,h,d)` | 同上 | 7.4.2、7.3.1（示例原句） | 大纲 `spare 〔… beam …〕`、`feed1 〔member beam〕`；成员数 semantics 1、spaceport 2 |
| 26 | `concept roof = kind(wedge) & on(volume) \| slab & on(wall \| column)` | 引用后面才定义的概念 `volume`（惰性）；`&` 高于 `\|`；库的 `roof` 在大纲里**覆盖**了 semantics.s3d 自己的 `roof`（后写的赢） | 7.4.3（前向引用）；1.5.1；7.2.2（示例原句：库里的 roof 定义）、5.3.5 | semantics 大纲里 `watch.cap` 没有 `roof` 标注（库定义要求 `on(volume)`，cabin 不是 volume），而 semantics.compile.json（不带库）的 `concept roof` 含 `watch.cap`——7.9.3 指出这正是覆盖；成员数 semantics 6、spaceport 0 |
| 28 | `concept volume = solid_kinds where min(w,h,d) > 3 * metre and top - bottom > 3 * metre` | 同 14 行 | 7.3.1 | 大纲 `hangar.body 〔volume〕`、`bunker 〔volume〕`、`annex 〔volume〕`；成员数 semantics 9、spaceport 19 |
| 30 | `concept tank = kind(cyl) where w > 2.5 * metre and h > 2.5 * metre` | `kind(cyl)` + `where`（1.7.3 的示例原句） | 1.7.3（示例原句）；7.3.1 | 大纲 `stage1 〔… tank〕`、`stage2 〔volume tank〕`、`spare 〔… tank〕`；成员数 spaceport 7（3 个 `tanks.N.tank`、stage1、stage2、spare，以及 `hangar.body.arch`：`cyl axis=z` 的 w、h 按 2.1.3 都是 2R = 12） |
| 32 | `concept prop = solid_kinds where max(w,h,d) < 3 * metre` | 同上 | 7.3.1 | 大纲 `crate 〔prop〕`、basics 的 `cup 〔prop〕`、`ball 〔prop〕`；成员数 semantics 49、spaceport 28 |
| 34 | `concept step = (kind(box) where min(w,h,d) < 0.4 * metre and max(w,h,d) < 3.5 * metre and top - bottom < 0.5 * metre) & on(all()) - slab` | 括号限定 `where` 的范围后再 `&`、`-` | 1.5.1（"要表示筛选后再并写 `(A where P) \| B`"）、1.5.5 | 成员数 semantics 0、spaceport 1 |
| 注释 / 空行 | 注释：1–10、13、15、18、20、22、25、27、29、31、33；空行：12、35 | — | 1.2.2；注释第 4 行的 `import "../lib/semantics.s3d"` 写法对应 1.8 / 5.2.2 的示例原句 | — |

覆盖率：语句行 13（11、14、16、17、19、21、23、24、26、28、30、32、34），被上表覆盖 13，**100%**。

## 9 逐章引用计数

按上面 8 张表第 4 列里出现的章节号统计（脚本 `cite.mjs`：每行表里同一小节只计一次，一行引用了同一章的 k 个不同小节计 k 次；表尾的注释行不计；第 5 列里作核对项引用的章节号不计）：

| 章 | 引用次数 | 最常被引用的小节 |
|---|---|---|
| 1 词法与语法 | 90 | 1.5.8（求值环境，6 次）、1.4.5、1.6.5、1.5.1、1.5.5（各 5 次） |
| 2 几何 | 55 | 2.2.8（span，9 次）、2.2.9（sweep，8 次）、2.2.3（4 次）、2.3.2、2.2.7 |
| 3 定位与朝向 | 186 | 3.3.6（at，13 次）、3.8.3（阵列实例变换，11 次）、3.4.3（10 次）、3.3.7、3.4.1、3.3.2（各 9 次） |
| 4 派生与修改 | 61 | 4.4.1（7 次）、4.5.1（5 次）、4.4.5、4.2.2、4.2.3、4.7（各 4 次） |
| 5 层 | 19 | 5.4.1（6 次）、5.3.5、5.1.2、5.2.3 |
| 6 事实 | 5 | 6.5.4、6.5.2、6.7.1、6.7.2（事实大多出现在第 5 列作核对项，不计入） |
| 7 语义层 | 55 | 7.3.1（11 次）、7.8.3（5 次）、7.2.1、7.4.2（各 3 次） |
| 8 元数据 | 0 | 语料里没有元数据（只有文件头缺省的 `lang`） |
| 10 材质 | 36 | 10.4.4（8 次）、10.1.3（6 次）、10.1.2（5 次）、10.4.1 |

第 3 章最多（每条节点语句都要经它定位），其次第 1、4、2、7 章。第 8 章没有被任何语句用到：8 个文件都没有 `~k=v`，没有文件头；按 8.1.5 它们的 `lang` 缺省为 2（见第 11 节 G2）。

## 10 参考输出里预期与本版不同的项（※ 的索引）

这些项在上表里以 ※ 标出；每条都已在相应章的"因本章定义而预期不同的项"里给出原型行为、新行为与理由，这里只列索引，供第 9 章汇总与 M1 对拍时排除：

| 表里的行 | 项 | 参考输出 | 本版 | 出处 |
|---|---|---|---|---|
| curves 39、45、71；rotation 30；semantics 33、44–49；spaceport 47–48 | 副本式节点的 `kind` | `mirror→span`、`mirror→revolve`、`rotate→group`、`mirror→box`、`mirror→wedge`、`mirror→group` | 原件的种类 `span`、`revolve`、`group`、`box`、`wedge`、`truss` | 4.8.5；1.6.7 |
| spaceport 47–48 | 副本式节点的 `types` | `null` | 原件的类型链（`tower2` → `["truss"]`） | 4.8.5；4.2.7 |
| spaceport 47–48；curves 71 | 影子的 `kind`（原件是阵列实例 / 副本组） | `group` | `instance` / `mirror` | 4.8.5；1.6.7 |
| spaceport_edit 13–15 | copy 节点的 `file` / `line` | spaceport.s3d:68 | spaceport_edit.s3d:13 | 4.3.2 第 7 条、4.8.5、5.5.4 |
| spaceport_blueprint 19 | 皮肤节点 `skin` 的 `file` | `null` | 变体语句的文件 | 5.2.5、5.5.4、4.8.5 |
| spaceport_blueprint 20；spaceport_edit 13–15 | 被 `color` 改色的条目的 `material` 形状 | `{color}` | 基础材质的五个分量 + 新颜色（concrete / steel 改色） | 10.6、10.3.2 |
| basics 21–25；spaceport 57–66、68、71、74–78；spaceport_edit 13–15 | 一方是布尔容器的事实的 `approx` | 不标 | `approx: true` | 6.8.5、2.5.6 |
| rotation 32–38 | `radar.feed` 与 `radar.dish` 的关系 | `intersects` | `inside`（revolve 的近似凸块允许作 $B$） | 6.8.5、6.5.4 |
| semantics 78–88 | 概念 `lamp`、`tree` 的成员顺序 | 并集按操作数拼接 | 节点顺序 | 7.9.4、7.2.3、1.5.5 |
| 全部文件 | 事实的 `gap` 精度与 `approx` 字段是否总是输出 | 4 位小数；只在为真时输出 | 完整双精度；总是输出布尔 | 6.6.1、6.8.5 |
| 全部文件 | `items[]` 条目材质、`materials` 条目的缺省分量 | 只输出写了的键 | 五个键总是都有 | 10.6 |

这些差别都不涉及任何节点的 `world`、`box`、`tight`、事实的 (rel, a, b) 与语义结果，所以参考输出仍逐节点、逐事实适用于 M1 的验收（3.11.2、6.8.3、7.9.1）。

## 11 缺口

逐条语句的解释没有缺口：352 条语句行每一条都能在上面找到条款。核对第 5 列时发现三处**输出形状**在各章里没有定义或没有标出与原型的差别；它们不影响任何语句的含义，但 M1 对拍时需要一个口径：

| 编号 | 缺口 | 现状 | 建议补在 |
|---|---|---|---|
| G1 | `defs` 输出条目的形状 | 参考输出是模板名数组（`["truss","shed","ribbed"]`）；4.2.7 只说"`defs` 输出条目"，8.3.8 / 8.4.3 要求每个条目带 `meta`，隐含条目是对象；两处没有给字段表 | 04-derivation.md 4.2.7：给出 `defs[]` 条目的字段（至少 `name`、`line`、`file`、`params`、`base`、`meta`），并在 4.8.5 列出"原型输出名字数组"这一预期不同 |
| G2 | 顶层 `meta` 与原型的差别 | 8.4.3 要求顶层 `meta` 总是存在且 `lang: 2`；参考输出没有顶层 `meta` 字段（原型不输出）。规则本身完整，但这条"原型 vs 本版"没有在第 8 章里标出 | 08-metadata.md 8.4.3 加一句"原型不输出顶层 `meta`；本版总是输出"，供第 9 章汇总 |
| G3 | `curves[]` 与 `items[]` 的几何字段 | 本表第 5 列用到 `curves[]` 的 `path` / `closed` / `pts`（点数 76、271、46、65）与 `items[].geom`（span 的 `type L w t dy`）；2.3.8 只说"输出：世界系折线 $R p_i + t$ 与 `closed`"，2.2 各节没有给条目的字段名；10.6 只定义 `material` 字段 | 02-geometry.md 2.3.8 与 2.1.5（或输出格式文档）：给出 `curves[]` 条目与 `items[]` 几何部分的字段表，或明确写明"字段名由输出格式文档定义，规范只定义值" |
