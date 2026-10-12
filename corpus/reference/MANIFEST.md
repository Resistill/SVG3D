# 参考输出包 MANIFEST

用途：`docs/REWRITE.md` 第 0 节“带走”与第 9 节 M0 交付物；新项目 M1 到 M3 验收对着它比。由原型导出，原型不再改动这些文件的来源命令。

- 原型 commit：`14582a3fd4b0299680a18e5d63107f66bf322243`（main，工作区干净；导出期间没有改任何被 git 跟踪的文件，没有提交）
- Node：v22.20.0（Windows 11，Git Bash）
- 导出日期：2026-10-12
- 总大小：约 237 MB（248 472 685 字节，不含本文件；其中 `work/sc/scene.compile.json` 188 MB、`work/sc/score.json` 37 MB）
- 全部命令在仓库根目录跑。

## 字段说明

- `node bin/s3d.mjs <文件> --json` 的输出已经包含新项目要对拍的全部内容，所以 examples 与 exports 直接用它，没有用脚本：
  - 顶层键：`items curves facts factsSkipped factProviders nodes concepts patterns variants checks defs materials errors warnings`。
  - `nodes[]`：`path kind id tags concepts line file types meta depth late ghost of curve hidden skin world box tight`。`world = {R, t}`（3x3 旋转 + 平移，即该物体的世界变换）；`box` 是对象的轴对齐包围盒；`tight` 是紧包围盒（按实际几何，容差对拍用它）；`late` 为后置对象。
  - `facts[]`：`{rel, a, b, text, gap, eps, provider}`，空间事实（on / inside / against / touching 等）。
  - `items` 是几何条目（已含展开后的基本体），`errors` / `warnings` 编译诊断，`concepts` 语义库命中的概念。
- 三个实战工程的 `scene.compile.json` 由 `tools/compile-dump.mjs` 导出：`{meta, nodes, facts, errors, warnings, concepts}`。`meta` 记来源、单位 `unit`（0.28）、条数。三个都是完整版（sc 为 188 MB，没有超过 300 MB 的阈值，所以没有裁剪成 slim）。`nodes` 是编译器原样的节点，字段同上，坐标单位是**场景单位**（stud，0.28 m），不是米。脚本带 `--slim` 选项（只留 path / kind / depth / tight / center / worldT），这次没有用。工程编译用 `openProject(dir).compile()`，即与工具完全相同的 `factEps` / `metre` 选项（`FACT_EPS / unit`、`1 / unit`）；三个工程都没有曲面锚点，所以编译不需要源网格。
- `*.eval.json`：`bin/eval.mjs` 支持 `--json`，存的就是它的输出（`errors objects kinds facts compression semantic description` 等）。
- `score.json`（`work/<名>/`）是 `score` 写在工程里的完整文件（逐物体 / 逐源部件误差，dorm 1.2 MB、h1 1.2 MB、sc 37 MB）；`score.stdout.json` 是命令行 stdout 的摘要（S、explain、byRole、worst 等，约 6 KB）。
- `compare.json` 是 `compare` 的 stdout，里面的 `file` 是导出机器上的绝对路径，图在同目录 `compare-{iso,iso2,top}.png`。
- `render-{iso,top,iso2}.json` 是 `render` 的返回 JSON（含 legend），`render-*.png` 是它指向的图（默认 `--mode=labels`、`--size=1024`）。
- 坐标、变换在不同文件里的单位：examples / exports 是场景原单位（米级），work 工程是 stud。

## 文件与来源命令

| 路径 | 来源 |
|---|---|
| `examples/<名>.compile.json`（7 个：basics curves rotation semantics spaceport spaceport_blueprint spaceport_edit） | `node bin/s3d.mjs examples/<名>.s3d --json` |
| `examples/<名>.outline.txt`（同 7 个） | `node bin/s3d.mjs examples/<名>.s3d --outline=2 --semantics` |
| `exports/<名>.compile.json`（11 个：basics.L0 curves.L0 l2fb rotation.L0 semantics.L0 spaceport.L0/L1/L2 spaceport_edit.L0/L1） | `node bin/s3d.mjs out/<名>.s3d --json` |
| `exports/<名>.eval.json`（9 个：除 l2fb 外每个） | `node bin/eval.mjs examples/<原名>.s3d out/<名>.s3d --json`，原名 = 去掉 `.L<n>`；`spaceport_edit.*` 对 `examples/spaceport_edit.s3d` |
| `work/{dorm,h1,sc}/score.json` | `node bin/s3d-tool.mjs score --project=work/<dorm/final｜h1/final｜sc/full>` 后复制工程里的 `score.json` |
| `work/*/score.stdout.json` | 同一条 score 命令的 stdout |
| `work/*/compare.json`、`compare-{iso,iso2,top}.png` | `node bin/s3d-tool.mjs compare --project=<工程>` 的 stdout；PNG 取自工程 `renders/` |
| `work/*/render-{iso,top,iso2}.{json,png}` | `node bin/s3d-tool.mjs render --view=<视角> --project=<工程>` |
| `work/*/scene.compile.json` | `node out/reference/tools/compile-dump.mjs <工程> <输出>`（完整版） |
| `work/*/project.json` | 工程里原样复制 |
| `work/panels-g10/` | 见下“g10 对照” |
| `tools/compile-dump.mjs`、`tools/collect.mjs`、`tools/summ.mjs` | 导出辅助脚本（compile-dump 如上；collect 复制工程产物；summ 打印下表的数） |
| `logs/` | 各命令的 stderr 与 sc score 的耗时记录（stderr 均为空） |

`out/` 里没有 `curves.L2.s3d`（只有 `curves.L2.glb`），`l2fb.s3d` 没有对应的 examples 原文，所以没有 eval。`out/*.glb` 与 `*.report.json` 没有收进来（`out/` 可重新生成）。

## g10 对照

`work/panels/g10` 存在，已复制到 `work/panels-g10/`：`{sp,dorm,h1}.{score,compare,init}.json`、`{sp,dorm,h1}.compare-{iso,iso2,top}.png`（来自各自 `renders/`，前缀是工程名）、`{sp,dorm,h1}.project.json`、`g10.log`。注意：

- g10 是 `sh bin/panels_run.sh g10` 的结果：**结构档新建**的 spaceport L2（`sp`）、宿舍区、H1 三个工程，**没有 sc**（SpaceCenter 全图没有 g10 对照）。
- 这里的 `*.score.json` 是摘要（约 6 KB），不是工程里的完整 `score.json`。

## 三个工程的指标（直接抄自 JSON）

S、explain 取自 `score.stdout.json` 的 `S` / `explain`；missing / extra 取自 `compare.json` 的 `views.<视角>`。

| 工程 | fidelity / δ | S | explain | iso missing / extra | iso2 missing / extra | top missing / extra |
|---|---|---|---|---|---|---|
| work/dorm/final | semantic / 0.25 | 184894.67 | 0.6333 | 0.3144 / 0.0188 | 0.0672 / 0.0170 | 0.2468 / 0.0112 |
| work/h1/final | semantic / 0.25 | 588268.92 | 0.4678 | 0.0007 / 0.1951 | 0.0001 / 0.1981 | 0.0214 / 0.0187 |
| work/sc/full | structure / 0.05 | 1589738.12 | 0.7226 | 0.0021 / 0.0159 | 0.0017 / 0.0118 | 0.0027 / 0.0224 |

g10 对照（同一口径）：

| g10 工程 | S | explain | iso missing / extra | iso2 missing / extra | top missing / extra |
|---|---|---|---|---|---|
| sp（spaceport L2） | 1009.23 | 0.9925 | 0 / 0 | 0 / 0 | 0 / 0 |
| dorm | 97254.38 | 0.8071 | 0.0025 / 0.0162 | 0.0007 / 0.0054 | 0.0017 / 0.0124 |
| h1 | 111583.91 | 0.8991 | 0.0009 / 0.0545 | 0.0013 / 0.0237 | 0.0023 / 0.0147 |

编译规模：dorm 4540 个节点 / 9683 条事实；h1 3874 / 15232；sc 149508 / 275343；三个工程编译 errors、warnings 均为 0。examples 与 exports 的 compile 也全部 errors / warnings 为 0。

## 需要人判断的发现

1. **`work/dorm/final`、`work/h1/final` 是语义档（fidelity=semantic，δ=0.25）逆向 + `refine --auto --to=structure --budget=10`，不是 g10 那样的结构档新建。** 按当前代码重新跑 compare，它们比 g10 差很多（dorm 的 iso missing 0.3144、top 0.2468，g10 约 0.002；h1 的 iso extra 0.1951，g10 0.0545），S 和 explain 也比 g10 差（dorm 184895 对 97254；h1 588269 对 111584）。所以 M3 验收“三个实战工程的 compare 不差于 g10”如果拿这两个 final 工程做基准是自相矛盾的，要么验收基准改成“结构档新建的 g10 口径”，要么另导出一份结构档的 final。sc/full 本身是结构档，compare 数很小，但没有 g10 对照。
2. 这两个 final 工程之前没有 `renders/compare-*.png`（dorm 只有一张 `iso-labels.png`，h1 没有 renders 目录），compare 是这次第一次在它们上跑，没有“改动前”的数可比。
3. 工程的 `score.json` 是被这次 `score` 重写的（旧内容未备份），`renders/` 也被写入。没有改任何层文件。

## 各步耗时与问题

| 步骤 | 耗时 | 结果 |
|---|---|---|
| examples 的 compile + outline（7 个） | 5 s | 全部 exit 0 |
| out/*.s3d 的 compile + eval | 6 s | 全部 exit 0 |
| dorm：score / compare / render x3 / 编译导出 | 7 s / 3 s / 2 s 每张 / 2 s | 全部成功 |
| h1：score / compare / render x3 / 编译导出 | 9 s / 3 s / 2 s 每张 / 1 s | 全部成功 |
| sc：score | 325 s（与 dorm、h1 的步骤部分重叠，是在后台跑的） | 成功 |
| sc：compare / render x3 / 编译导出 | 56 s / 39 s、38 s、38 s / 29 s | 全部成功 |
| g10 复制 | 瞬间 | 成功 |

没有命令失败，没有超时，没有文件缺失（`work/<区域>/<区域>.glb`、`../SpaceCenter_Rebuild` 都在）。仅有的缺项：`curves.L2` 没有 .s3d、`l2fb` 没有原文（见上）。
