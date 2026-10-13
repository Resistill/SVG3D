# 进度

每个里程碑结束前在这里记 corpus 的全部指标（见 PLAN.md 第 9 节）。

## M0 规范

### 2026-10-13：10 章写完，进入审查

- `spec/` 10 章全部有正文（约 6 700 行）；`decisions.md`（D 编号索引，从各章引用处重建）、`crosswalk.md`（语料逐条对照，8 个文件 352 条语句行，覆盖 100%）写完。
- 每章末尾的"用参考输出验证"都用脚本对 `corpus/reference/examples/*.compile.json` 重算过：第 3 章 7 个示例 1644 个节点的 world / box 全部在 1e-6 内一致；第 5 章 spaceport 与 spaceport_blueprint 的 193 个节点、203 条事实逐项相同；第 4、6、7 章的 copy / 副本 / 事实 / 概念 / 变体数值一致；预期不同的项各章列表，第 9 章 9.3 汇总并在 9.5 对 19 个语料文件逐个核查（没有文件触发新的错误）。
- 写作时提出的 52 条跨章配合请求已归档，正由逐章"对齐 → 对抗审查 → 修正"处理（审查对着原型源码与参考输出）。
- 环境：原型推到 `Resistill/SVG3D-Prototype`，云端 clone 在 `/home/user/resistill/svg3d-prototype`；`corpus/reference/work/`（230 MB）与 `work/` 不在云端，M3 前再安排。
- 未决：外部数据候选清单（`docs/external-data-candidates.md`）待用户确认。

### 2026-10-13：审查完成，M0 规范部分收口

- 对抗审查（每章一名审查员，对着原型 `src/s3d.js` 与参考输出，整章逐节过）：共 158 条 finding（blocker 5、major 60、minor 93），除 16 条"规范一侧有意为之、只需进第 9 章"外全部修正。5 条 blocker：3.6.3 后置判定的容器处理顺序写反；3.5.4 `aim` 在只写 `pin` 的节点上起点取错；4.5.1 与 5.4.1 把带参数的 `over … use=T k=v` 误判为几何时机；7.1.4 与 6.6.4 关于 factsSkipped 警告的矛盾。
- 两轮跨章对齐（52 + 100 条请求），裁决记录在本会话；争议点的结论：where 里裸标识符按 8.2.2 的顺序（字段 → 元数据键 → 词法环境 → 对象）；皮肤的 `late` 按 3.7.6 的可判定规则；副本根退化时影子照常生成；import 路径先绝对化再规范化。
- 第 1 章新增 1.11，列出 40 条与原型的有意差异（如 `kind(group)` 不再选中阵列实例、向量 `==` 逐分量、`round` 远离零、字面量优先于名字查找、`g.top` 永远是锚点）；第 9 章 9.3.6 由脚本从各章差异表逐行转引（153 行），9.3.1–9.3.5 是分类摘要。
- `spec/tools/check-refs.py`：3 段以上的小节引用 0 条悬空。
- 待你过目：`docs/spec-open-items.md`（70 条按设计定下、语料验证不到的规则；20 条审查时的选择；16 条有意保留的原型差异）。
- 仍待确认：外部数据候选清单（`docs/external-data-candidates.md`）。
- 本阶段成本备忘：两轮 Workflow 的子 agent 约 800 万 token（写作 274 万、审查 527 万）。审查轮每章一名审查员从头读整章与被引用章，重复阅读多；下次同类工作改为"一名审查员带清单 + 主会话直接改"，不再每章一个全量 agent。

### 2026-10-13：用实战工程核对规范

- 原型推到 `Resistill/SVG3D-Prototype`，三个实战工程的源文件（不含缓存、渲染、评分）放在它的 `work-snapshot/`（54 MB）。用原型重算 compile JSON，节点数 / 事实数与 MANIFEST 完全一致。
- 第 3 章：dorm / h1 / sc 共 157 919 个节点的世界变换与 `box` 按规范公式重算，全部在 1e-6 内一致（3.11.6）。
- 第 6 章：300 258 条事实里 293 230 条逐字段一致，其余全部落在已列的三类预期差异（近似容器的 `inside`、mesh 的 `against`、布尔容器的 `approx`）；$a$ / $b$ 平局 19 084 对与扫描序规则一致（6.8.6）。
- **发现**：原型逆向写出大量零尺寸实体（三个工程共 7323 个节点），按现行规范是错误；处置方案待用户拍板（`docs/spec-open-items.md` 第 0 节、9.5.5）。另：软保留名警告在逆向输出上命中 1639 条，M3 命名要避开种类词。
- 脚本：`spec/tools/verify/verify3-work.mjs`、`verify6-work.mjs`。

### 2026-10-13：M0 收口

- 验收（PLAN 9 节）：规范能解释 examples 每一条语句——`spec/crosswalk.md` 覆盖 8 个文件 352 条语句行 100%，缺口 0；参考输出包齐全（`corpus/reference/`，work 部分在原型仓库 `work-snapshot/` 可重算）。
- 交付物：`spec/` 10 章（约 7 000 行，逐章对抗审查并对着原型源码、参考输出、三个实战工程核对）、`decisions.md`、`crosswalk.md`、第 9 章兼容性声明与对拍容差、外部数据候选清单（已确认，C1 推到 M1 末）。
- 带进 M1 的清单：`docs/spec-open-items.md`（按设计定下、语料验证不到的规则，M1 实现时优先补测试）；零尺寸实体保持报错（旧工程文件不是兼容目标）。
- M0 结束，开 M1（`s3d-geom`、`s3d-lang`、事实、语义、eval、CLI compile；验收见 PLAN 9 节）。

## M1 内核 + 编译器

### 2026-10-13：M1.0–M1.3 几何与派生对上参考输出

- 工作区：`s3d-geom`（f64 + libm，AABB、变换、实体局部盒、曲线、扫掠、二维多边形）、`s3d-lang`（词法、语法、登记、展开、copy、over / drop / 换形、副本树、求解、JSON 输出）、`s3d-cli`（`compile`、`verify`）。工具链 1.97.0，clippy 守确定性清单，CI 跑 fmt / clippy / test / wasm 构建。
- 对参考输出（`s3d verify`，world / box 按 9.4.2 的容差）：basics 18/18、rotation 615/615、spaceport 193/193、curves 94/94、spaceport_edit 217/217 节点全部一致，错误 0；semantics、spaceport_blueprint 只缺变体皮肤子树（阶段 8，M1.4）。
- 过程中按规范修正的实现点：`pin=origin` 不读自身盒（3.6.1；原先把 coaster.legs 误判后置）；revolve 的轮廓按 2.2.7 第 1–6 步处理（smooth 的过冲进包围盒）；sweep / pipe 的环点、倒圆、框架按 2.2.9 / 2.3.7。
- 已知偏差（记入 `docs/spec-open-items.md` 第 4 节）：模板体里 copy 块内的修改与同一体里稍后的 over 的相对顺序。
- 成本：本段全部由主会话手写，没有再开子 agent。

### 2026-10-13：M1.4 事实、语义层、材质、输出全部对上参考输出

- 新增：凸块与分离量（`s3d-geom::convex`，第 2 章各实体的凸块 + 6.4）、紧包围盒（`tight`，2.1.8 / 2.5.4）、空间事实与关系索引（`s3d-lang::facts`，第 6 章）、材质表与有效材质（`material`，第 10 章）、语义层（`semantic`：concept / pattern / check / style / variant 与皮肤挂载，第 7 章 + 4.7）、完整的输出 JSON（items / curves / facts / concepts / patterns / checks / variants / defs / materials / files / meta）。
- `s3d verify` 按 9.4 的规则与 9.4.5 的规范化（N1、N2、N5、N7、N8、N10、N11、N15、N16）对拍全部字段。结果：**17 个参考文件（7 个 examples + 10 个 exports）全部 0 失败**；N8 预期改变的事实恰为 9.5.3 预测的 345 条（已回填）；另有 12 条相邻换序属 9.4.4 的扫描序平局（舍入噪声）。
- 规范修正：2.2.9 扫掠凸块的面片顶点序是定义的一部分（棱集按先出现去重，沿路径的四条棱只差微小角度；按原型的顶点序才能复现 rotation 的 12 条枕木 `gap`）。
- 未做、留到后面：hull 条目的 `faces`（语料没有 hull，凸块已有面片但条目先输出空数组）；影子覆盖（4.4.4）在求解侧生效；皮肤里的 mirror / rotate 副本树（4.7 第 6 条；语料的皮肤里没有副本）。
- 成本：本段全部由主会话手写（约 10 次修改—对拍循环），没有再开子 agent。

### 2026-10-13：M1.5 收口，M1 验收

- **真值评测**：`s3d-score::eval`（原型 `tools/eval.mjs` 的移植），CLI `s3d eval <原文> <逆向>`；9 个参考 `eval.json` 的 objects / kinds / compression / semantic / description 全部一致，`facts.*` 一组按预期不同（逆向侧 N8 的 345 条事实改变，rotation 的 `radar.feed`，rotation.L0 的 meanIoU 1.0 vs 0.99 来自圆柱紧盒按真实几何）。
- **wasm 与三处哈希**：`s3d-wasm`（裸 C ABI，不依赖 wasm-bindgen，2.4 MB）；`s3d hash` 与 `tools/wasm-hash.mjs` 对同一个文件包算同一份紧凑 JSON 的 SHA-256。Linux 原生与 wasm 的 18 个语料文件哈希逐个相同；Windows 原生由 CI 的 `hash-windows` 任务产出、`three-way` 任务做三方 diff（PLAN 第 9 节 M1 验收项）。
- **微基准**：`s3d bench-nearest`（parry3d-f64 BVH，enhanced-determinism；GLB 自带最小读取器）。H1 GLB 不在仓库，用 curves.L0.glb 代测：单线程 2.8 µs/次、4 线程 0.58 µs/次，多线程归约逐位等于单线程；门槛（单线程 ≤ 3 s / 262 万次）未达到，已按 PLAN 第 8 节约定记入待修订。
- **C1 货场**：`corpus/examples/depot.s3d`，5031 节点 / 2577 条目 / 7594 事实，本版与原型逐项一致，原型输出收进参考包（`depot.compile.json`），CI 的 verify 与哈希步骤自动覆盖它。
- **M1 验收（PLAN 第 9 节）**：examples 全部编译 ✓；事实在参考输出包的容差内一致、不一致逐条列出并解释 ✓（verify 的报告 + 9.5.3 回填）；三处哈希 ✓（CI 任务，Linux 与 wasm 已在本机确认）；最近点微基准 ✗（未达标，待 M2 前修订）。
- 遗留清单见 `docs/spec-open-items.md` 第 4 节（hull 条目 `faces`、影子覆盖在求解侧生效、皮肤里的副本树、copy 块内修改的展开序）。

## M2 评分 + 渲染

### 2026-10-13：M2.1 几何支撑、M2.2 自评分数、校准对拍

- **几何（`s3d-geom`）**：`tri`（全部 12 种实体按 2.1.9 与各节的三角化，分段参数 `TriParams`，默认规范、另有 `PROTOTYPE` 预设只作对拍）、`csg`（BSP 网格布尔，2.5.3）、`bvh`（静态 BVH：带封顶的最近点、带符号的射线穿越计数）、`mesh::load_parts`（GLB 按节点 × 图元出部件：材质 sRGB 颜色、`名字@k`、享元键、`merged_` 连通分量拆分）。hull 条目的 `faces` 所需的面片枚举顺带补齐（`convex::hull_faces`），条目输出仍待接。
- **评分（`s3d-score`）**：`score`（原型 `tools/score.mjs` 的移植：L_stmt 的词计价与换形折扣、L_mesh 的享元与角色计价、R_src / R_recon / R_vol、S_mesh 基线、byRole、worst、`unattributedLines`；增量缓存 `ScoreCache`：源侧只算一次，R_recon 按"路径 + 颜色 + 几何哈希"缓存，R_src 逐点只重算受影响的点，与全量逐位相同）、`edit`（E_edit 报告：按 PLAN 第 6 节基于事实的坐标依赖定义，权重默认 0、只报告）、`roles`、`sample`、`surfaces`、`volume`、`objects`（条目树 → 世界三角形，mesh 引用取源部件、布尔体走网格布尔）。`s3d-lang` 新增 `items`（条目树与节点摘要，不经 JSON）、节点的 `placement_ref`。CLI：`s3d score`（`--glb`、常数、`--segments prototype`、`--full`）、`s3d score-hash`、`s3d objects`（调试）、`s3d bench-bvh`；wasm 新增 `s3d_score`；`tools/wasm-hash.mjs --score`。
- **对原型核对**（`corpus/reference/exports/scores.json`，原型 commit 439abb8 对 9 个场景 × 4 个输入；输入里新收的 `<名>.abs.s3d` / `<名>.rev.s3d` 见 corpus/reference/MANIFEST.md）：用原型分段时 L_stmt / L_mesh / S_mesh 逐位相同，R_src 绝大多数逐位相同、其余 ≤ 3.4%，R_recon ≤ 7%，R_vol 本版高 30–36%（原型 `insideMesh` 漏判，本版按绕数交叉验证）；差异全部写进 spec/09 的 9.5.6 与 9.6.2。回归测试 `crates/s3d-score/tests/corpus_score.rs`（含增量 = 全量）。
- **评分三处哈希**：Linux 原生与 wasm 对 27 个输入（9 × recon / abs / rev）逐个相同；Windows 由 CI 的 `hash-windows` / `three-way` 任务覆盖，run #23 三方一致 ✓。
- **具名约束**（`s3d-score::constraints`，PLAN 第 6 节）：`solidFill`（maxWrong / minVolume）与 `keepLargeParts`（minSize），每条可关、带路径模式选择器（默认空集 = 不约束），作用在 `score --full` 的逐物体 / 逐源部件结果上（物体新增 `vol` / `volWrong`），报告列出被否的项；M3 的候选筛选把它当过滤器。hull 条目的 `faces` 已输出。
- **校准序（M2 验收第一项）的实际读数**（原型自己的分数，`scores.json`）：以 corpus 冻结的逆向结果为"逆向"项时，`S_mesh > abs > recon > orig` 在 9 个场景里只有 rotation、semantics 成立——冻结的逆向结果保留了 mesh 引用，而原型（含 `prototype` 标签那一版）的 `MESH_PRICE` 默认按 full（4.5 × 顶点 + 3 × 三角形）计价，basics / curves / spaceport 的 recon 因此高于 abs；换成同一 commit 重新逆向的 `rev`（不再发射 mesh 引用）后 spaceport L0/L1/L2、rotation、curves、semantics 成立，basics 的 rev（284.5）比 abs（267.5）高 17。本版的分数与原型同序。**需要拍板**：验收用哪组输入（见本节末）。
- **微基准（PLAN 第 8 节）**：自写 BVH 单线程 3.9 µs/次（parry 2.7 µs/次），都未到 1.1 µs 的门槛；评分走自写 BVH（要封顶查询与穿越计数）。全量评分 spaceport.L0 0.5 s（111 个部件）。
- **未做**：CPU 光栅 compare（原型 `raster.mjs` + PNG，可派便宜模型）、`s3d-render`（wgpu / WebGPU；云端容器没有 GPU，SpaceCenter 预览 60 帧只能在用户机器上测）、hull 条目的 `faces` 输出、E_edit 的权重（决定 7，M3 末）。
- **已定（2026-10-13，用户）**：(2) 分段差异接受为定义差异，M5 用本版导出重做 GLB；(3) PLAN 第 8 节按实测修订：BVH 查询改用固定栈（原先每次查询分配一个 Vec，多线程下互相争锁、4 线程反而比单线程慢），封顶 dMax 的查询单线程 0.69 µs/次、4 线程 0.18 µs/次，原门槛在评分的真实负载下达标，于是数字不改、改工作负载的定义（写进第 8 节）；(4) `s3d-render`、CPU compare、E_edit 权重派便宜模型，在用户机器上做。
- **已定（2026-10-13，用户）**：(1) 校准序验收改用同一 commit 重新逆向的 `rev` 作"逆向"项，basics 记为已知例外，M3 再用本版逆向复验（PLAN 第 9 节 M2 行已改）。
- **M2 验收（PLAN 第 9 节）的本版读数**（`s3d score --segments prototype`，与 `scores.json` 的原型读数同序）：

| 场景 | 全网格 S_mesh | 全坐标 abs | 逆向 rev | 原文 orig | 序 |
|---|---|---|---|---|---|
| basics.L0 | 18733.5 | 267.5 | 284.5 | 258.5 | ✗（已知例外：rev 比 abs 多 17） |
| curves.L0 | 251135 | 5437.1 | 5012.0 | 409 | ✓ |
| rotation.L0 | 199474.5 | 22334.9 | 15195.0 | 506.1 | ✓ |
| semantics.L0 | 177055.5 | 2106 | 862.5 | 568.5 | ✓ |
| spaceport.L0 | 133164.5 | 2810.3 | 1585.3 | 726.0 | ✓ |
| spaceport.L1 | 133164.5 | 2854.8 | 1623.5 | 724.7 | ✓ |
| spaceport.L2 | 134116.5 | 4083.8 | 1817.5 | 723.6 | ✓ |

  评分三处哈希 ✓（CI #23 起）；渲染两项（60 帧、无头渲染对拍）与 CPU compare 派便宜模型，M2 的这两项待其完成。

## M3a 意图的执行

### 2026-10-13：意图的数据形状、源侧决定、重绑

- **`s3d-reverse`（出 wasm）**：`intent`（PLAN 4.2 的 JSON 形状，serde，未知字段原样保留；4.2.1 的词汇表校验：不在表里的操作、缺参数、选择器与阶段不配都报错；4.6 的优先级）、`parts`（读取部件阶段执行 drop / role / merge / type / keep-mesh / shape；选择器 sources 或 rule；每个部件每个键取优先级最高的决定；具体对象意图之间的不同值记冲突、规则型之间按 id 压过不算冲突；`drop` 被更高优先级的具体对象决定压过；报告 applied / unsatisfied / dangling / conflicts；`source_roles` 产出与原型 `state/source-roles.json` 同形的表）。
- **`s3d-project`（只出 native）**：`store`（`intents/NNN.json` 读写、样本集文件读取、校验、id 永不回收、状态变迁 proposed → accepted → retired 带 history）、`rebind`（4.3：路径命中须经指纹或来源交叉验证才算 bound，只命中路径的记 suspicious；路径没命中按指纹（同种类、IoU ≥ 0.5）重绑并改写路径；还找不到的 dangling；来源锚按源部件 id 存在与否；区域锚数区域里的物体；稳定命名将共用 `best_by_fingerprint`）。
- CLI：`s3d intents <意图文件|工程目录> --glb|--names [--roles-out]`、`s3d roles-diff`、`s3d intent-rebind <工程目录> <场景> [--glb] [--iou] [--write]`。
- **M3a 验收（规则样本意图产出与原型可比的角色）**：三个工程里只有 sc 的 `state/source-roles.json` 在原型仓库的 work-snapshot 里（dorm / h1 的 state 目录没有它），规则型意图只看部件名，所以在云端直接对 32 157 个名字执行样本的 8 条规则：与原型一致 24 076（74.87%）；**规则命中的 16 771 个部件与原型全部一致**（collision 1735、decal 1205、decor 13752、terrain 76、helper 3，混淆矩阵的这些行没有一个落在对角线外），不一致的 8081 个全是规则没命中、按默认 structure、而原型判成 decor（7690）/ terrain（333）/ helper（57）/ decal（1）的——都是样本 `_note` 自述的近似：原型取"最后一个有归属的词"（`Lighting_Spotlight_01E`、`Resort_Curtain_09b` 末尾是变体字母，`GenerateRoad_Asphalt_Mesh_x86_y60_13648_HD` 末尾是 Mesh / x / y / HD），样本的正则只认"角色词在名字末尾"；另有几何回退（很薄且散成多块 → decal、默认名的大盒子 → helper）规则表达不了。spaceport.L0：ground / road → terrain，4 盏 lamp → decor，其余默认。测试 `crates/s3d-project/tests/sample_intents.rs`（sc 一项在原型 clone 不在时跳过）。
- **待拍板**：规则选择器是否加一种"词"谓词（`rule.words`：名字按下划线 / 驼峰拆词，取最后一个落在给定词表里的词），把原型的"最后一个有归属的词"写进样本意图。加了预计一致率到 95% 以上（只剩几何回退的部分）；不加就按 PLAN 4.5 的"正则"口径，接受 74.9% 并记为可比。这改的是意图 JSON 的选择器形状，所以要你定。
- 未做（M3）：物体阶段的决定（hollow / tol / lock / array / mirror）要等逆向主体；`lock` 的"沿用上一次的那一行"在原型里是对文本做的，本版要在发射阶段做。
