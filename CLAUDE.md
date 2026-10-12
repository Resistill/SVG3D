# 给 Claude 的工作约定（SVG3D 第二版）

- 用中文回复。仓库里的文档、代码注释用中文；提交信息用英文。
- 每完成一项就提交一次。
- 计划：`docs/PLAN.md`（第 9 节是里程碑与验收，第 10 节是已定的决定，不要重开已定的决定）。进度：`docs/PROGRESS.md`。
- **M0 是硬门槛**：`spec/` 的 10 章没写完不开 M1 的代码。
- 原型仓库在同级目录 `../SVG3D`（已冻结，只读）：语料、参考输出包、经验（docs/PLAN2.md 的 D1–D11）都从那里来；新代码不复制原型的实现。
- 子 agent 的模型：M0 到 M4 的规范、几何内核、编译器、评分、逆向、意图机制不降级（用主会话同级的模型）；M5 的导出、界面外壳、语料搬迁、文档整理可以派 Opus 或 Sonnet。
- 确定性（PLAN.md 原则 18 的落地清单）进 CI：禁用 std 的 sin/cos/exp/pow/hypot/atan2，统一 libm；并行只允许静态分块 + 按块号顺序归约；不迭代 HashMap；glam 开 libm + scalar-math，parry3d 开 enhanced-determinism、不开 simd；wasm 不开 simd128。
- 验收对着 `corpus/reference/` 的参考输出包比，按定义与容差，不逐字节。
- 实战工程（work/）不进 git；`../SpaceCenter_Rebuild` 与本仓库必须同级（旁路表相对路径）。

## 坑

- Windows 上子 agent 会写出 CRLF；仓库有 `.gitattributes`（eol=lf），合并后检查。
- Git Bash 里 `node -` 加 heredoc 会吃掉反斜杠；编辑脚本写成文件再跑。
