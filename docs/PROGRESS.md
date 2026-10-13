# 进度

每个里程碑结束前在这里记 corpus 的全部指标（见 PLAN.md 第 9 节）。

## M0 规范

### 2026-10-13：10 章写完，进入审查

- `spec/` 10 章全部有正文（约 6 700 行）；`decisions.md`（D 编号索引，从各章引用处重建）、`crosswalk.md`（语料逐条对照，8 个文件 352 条语句行，覆盖 100%）写完。
- 每章末尾的"用参考输出验证"都用脚本对 `corpus/reference/examples/*.compile.json` 重算过：第 3 章 7 个示例 1644 个节点的 world / box 全部在 1e-6 内一致；第 5 章 spaceport 与 spaceport_blueprint 的 193 个节点、203 条事实逐项相同；第 4、6、7 章的 copy / 副本 / 事实 / 概念 / 变体数值一致；预期不同的项各章列表，第 9 章 9.3 汇总并在 9.5 对 19 个语料文件逐个核查（没有文件触发新的错误）。
- 写作时提出的 52 条跨章配合请求已归档，正由逐章"对齐 → 对抗审查 → 修正"处理（审查对着原型源码与参考输出）。
- 环境：原型推到 `Resistill/SVG3D-Prototype`，云端 clone 在 `/home/user/resistill/svg3d-prototype`；`corpus/reference/work/`（230 MB）与 `work/` 不在云端，M3 前再安排。
- 未决：外部数据候选清单（`docs/external-data-candidates.md`）待用户确认。
