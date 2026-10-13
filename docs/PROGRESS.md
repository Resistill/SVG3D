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
