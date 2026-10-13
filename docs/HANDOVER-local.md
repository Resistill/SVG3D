# 交接：在本机继续 SVG3D 第二版（2026-10-13）

云端会话（分支 `main-1jd9ye`，71 个提交，CI 三方哈希全绿）把 M0–M2 的核心与 M3a 做完了。下面是本机要接的东西：哪些只能在本机做、怎么搭环境、每个命令怎么跑、派给便宜模型的任务怎么写、还有什么没定。读完这一份就够，细节再去 `docs/PROGRESS.md`、`docs/PLAN.md`、`spec/09-compat.md`。

## 0. 现状一句话

| 里程碑 | 状态 | 验收 |
|---|---|---|
| M0 规范 | 完成 | 10 章 + 参考输出包 |
| M1 内核 + 编译器 | 完成 | 18 个参考文件 verify 0 失败；三处哈希相同（CI） |
| M2 评分 + 渲染 | 评分、CPU compare 完成；**`s3d-render`（GPU）未做** | 校准序成立（PLAN 第 9 节 M2 行的口径）；评分三处哈希相同；渲染两项待本机 |
| M3a 意图的执行 | 完成 | SpaceCenter 32 157 个部件角色与原型一致 99.82% |
| M3 逆向 | 未开 | 验收要对三个实战工程，只能在本机 |

## 1. 只能在本机做的事（按优先级）

1. **`s3d-render`（wgpu / WebGPU，PLAN 第 2 节与第 9 节 M2 行）**：云端容器没有 GPU。建议派 Opus 写无头 PNG 版（第 5 节有现成的任务书），本机验证两项：无头渲染与 CPU 采样对拍、SpaceCenter 预览 60 帧且 overdraw 在预算内（PLAN 第 7 节的每像素 ≤ 4 次求交）。
2. **PLAN 第 8 节的回填**：全量评分 SpaceCenter（`../SpaceCenter_Rebuild/whitebox/SpaceCenter_Whitebox.glb` + 原型 `work-snapshot/sc/full/base.s3d` 系列）的单线程耗时；云端按 spaceport / rotation 外推是 145 s 单线程。评分现在没有并行，超过预算就按原则 18 加"按部件静态分块 + 按块号顺序归约"。
3. **M3 的实战工程验收基线**：`corpus/reference/work`（230 MB，dorm / h1 / sc 的 score.json、compare、panels-g10）不在云端；M3 的 compare 与 S 要对它比。dorm / h1 的 `state/source-roles.json` 原型 work-snapshot 里也没有，角色对拍只做了 sc。
4. **Windows 原生哈希**：CI 的 `hash-windows` 任务已经在跑并通过，本机不必重复；但 `cargo test` 里两处用 `python3` 的测试（`crates/s3d-geom/src/png.rs` 的 zlib 对拍、它会在没有 python3 时跳过）在 Windows 上命令名可能是 `python`，跳过不算失败。

## 2. 环境

- Rust 1.97.0（`rust-toolchain.toml` 固定，含 wasm32 目标、rustfmt、clippy）。三处哈希对拍要求同一编译器版本，不要换。
- Node ≥ 22（`tools/wasm-hash.mjs` 用；CI 用 v22）。
- 原型仓库 clone 到同级目录 `../SVG3D`（CLAUDE.md 的约定）或任意位置；只读。本机有它的 `node_modules` 时可以重跑原型的评分对照脚本（见第 4.3 节）。
- `../SpaceCenter_Rebuild` 与本仓库同级（旁路表相对路径，`work-snapshot/*/project.json` 里的 `source` 字段就是这么写的）。
- `work/`、`corpus/reference/work` 不进 git。

常用检查（CI 跑的就是这几条）：

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings      # clippy.toml 里是确定性禁用表
cargo test --workspace
cargo build --release --target wasm32-unknown-unknown -p s3d-geom -p s3d-lang -p s3d-score -p s3d-reverse -p s3d-wasm
python3 spec/tools/check-refs.py
```

## 3. 命令表（`cargo build --release -p s3d-cli` 后在 `target/release/s3d`）

| 命令 | 用途 |
|---|---|
| `s3d compile <文件.s3d>` | 编译输出 JSON（与参考输出包同形） |
| `s3d verify <文件.s3d> <参考.compile.json>` | 按 spec/09 的 9.4 对拍参考输出 |
| `s3d hash <文件…>` / `node tools/wasm-hash.mjs <wasm> <文件…>` | 编译输出的 SHA-256（三处对拍） |
| `s3d score <文件.s3d> --glb <源.glb> [--delta --unit --samples --cres --cmesh --dmax --edit-weight] [--segments prototype] [--no-facts] [--full]` | 自评分数；`--full` 带逐物体 / 逐源部件；`--segments prototype` 只用于对原型导出的 GLB 对拍 |
| `s3d score-hash <文件…>` / `node tools/wasm-hash.mjs --score <wasm> <文件…>` | 评分输出的哈希（源模型取同目录 `<名>.glb`） |
| `s3d eval <原文> <逆向> [--lib]` | 真值评测 |
| `s3d render <场景.s3d> [--glb] [--view …] [--mode color\|id\|labels\|error] [--size] [--out] [--source] [--only 角色] [--box] [--ortho] [--cut] [--elev]` | CPU 光栅 PNG |
| `s3d compare <场景.s3d> --glb <源.glb> [--views iso,iso2,top] [--size] [--blur] [--meshes] [--only] [--out-dir]` | 源 vs 重建的 missing / extra / diff 与差异图 |
| `s3d intents <意图文件\|工程目录> (--glb \| --names <id→角色 JSON>) [--roles-out]` | 读取部件阶段执行源侧决定，产出角色表 |
| `s3d roles-diff <ours.json> <theirs.json>` | 两份角色表的一致率与混淆矩阵 |
| `s3d intent-rebind <工程目录> <场景.s3d> [--glb] [--iou 0.5] [--write]` | 意图重绑 |
| `s3d objects <场景.s3d> [--glb]` | 调试：每个物体 / 源部件的三角形数、体积、面积、包围盒 |
| `s3d parts <场景.s3d> <路径>` | 调试：节点的凸块 |
| `s3d bench-bvh <glb> [次数] [--cap d] [--sort] [--threads n]`、`s3d bench-nearest` | 最近点微基准（PLAN 第 8 节） |

## 4. 本机要跑的具体步骤

### 4.1 SpaceCenter 全量评分与耗时（回填 PLAN 第 8 节）

```
# 场景用原型 work-snapshot 的 sc/full（base.s3d import 各 tile；单位 stud = 0.28 m）
target/release/s3d score <原型>/work-snapshot/sc/full/scene.s3d --glb ../SpaceCenter_Rebuild/whitebox/SpaceCenter_Whitebox.glb --unit 0.28 --delta 0.05 > work/sc.score.json
```

（work-snapshot 的 `sc/full/scene.s3d` 还 import 了 `style.s3d`，快照里没有这个文件，先建一个空文件，否则多一条 import 报错；dorm / h1 的 `final/` 有 `style.s3d`。）

记：墙钟、`S` / 各分项、与 `corpus/reference/work/sc/score.stdout.json` 的差（口径不同：原型按原型分段导出、本版按规范 48 段，差异说明见 spec/09 的 9.5.6；意图的角色现在不进评分，要先 `s3d intents` 出角色表再接，见第 6 节）。注意原型工程的 `~src` 写的是带分组路径的节点名，本版 `load_parts` 取 GLB 节点名原样，对得上。

### 4.2 三个工程的 compare 基线（M3 用）

```
target/release/s3d compare <工程>/scene.s3d --glb <源.glb> --views iso,iso2,top --out-dir work/<名>/renders
```

与 `corpus/reference/work/panels-g10/<名>.compare.json` 对照；云端在 spaceport / rotation 上与原型逐项相同。

### 4.3 重跑原型的评分对照（只在想复核 `corpus/reference/exports/scores.json` 时）

云端用的脚本在会话 scratchpad 里没有进仓库，逻辑很短：对每个 `corpus/exports/<名>.glb` 用原型 `partsOfDocument` 读部件、`reverse(parts, {srcFile, level})` 得 `src` / `absSrc`，用原型 `score({result: compile(text), files, sourceParts})` 给 recon / abs / rev / orig 打分；要在原型目录下用 `node --import ./bin/node-hooks.mjs` 跑。

## 5. 派给便宜模型的任务书（可直接贴）

### 5.1 `s3d-render`（Opus）

> 你在 Rust 工作区 SVG3D 里工作，先读 CLAUDE.md 并遵守（中文注释、英文提交信息、每项一提交；确定性禁用表；GPU 输出不进确定性承诺）。任务：新建 crate `crates/s3d-render`（wgpu + WGSL，PLAN 第 2 节与第 7 节："基本体实例化直绘 + 片元布尔"是目标路线，第一版可以先做"三角网格直绘"：用 `s3d_score::objects::scene_objects` 的世界三角形），只用 `Features::all_webgpu_mask()` 里的特性与 WebGPU 默认 Limits（原生与浏览器共用同一份 WGSL）。交付：(1) 无头渲染 `s3d render --gpu …`，参数与 CPU 版 `s3d render` 相同，输出 PNG；(2) 一个对拍测试：同一取景下 GPU 图与 CPU 图（`s3d_geom::raster`）模糊后的灰度差 < 0.02（PLAN 第 9 节 M2 行"无头渲染与 CPU 采样对拍"），GPU 不可用时测试跳过并打印原因；(3) `docs/PROGRESS.md` 的 M2 节追加一段，写明在什么 GPU 上验过、帧率怎么量（预览 60 帧与 overdraw 预算是本机的事，给出量法即可）。不要改 PLAN 第 10 节的决定；提交到当前分支，不推送。

### 5.2 CI 补一项（Sonnet 即可）

把 `s3d compare` 的 PNG 与 JSON 也加进 CI 的三方哈希（`.github/workflows/ci.yml` 的 `hashes` / `hash-windows` / `three-way` 三步，照 `score-hash` 的写法）。

## 6. 没定、没做的清单

- **E_edit 的权重**（PLAN 决定 7）：M3 末定；现在 `--edit-weight` 默认 0，只报告坐标依赖的物体数。
- **意图的角色进评分 / compare**：`s3d score` 与 `s3d compare` 的源部件角色还是默认规则（`s3d_score::roles::assign_roles`），没有读意图；M3 接逆向时把 `s3d intents` 的角色表接进去（`Part.role`），PLAN 第 5 节末的默认行为"无意图时不区分角色"也在那时落实。
- **物体阶段的决定**（hollow / tol / lock / array / mirror）：词汇表与校验已有，执行要等 M3 逆向主体。
- **几何回退的角色判定**（白模体积 → helper、薄片散块 → decal）：意图词汇表里没有对应谓词，SpaceCenter 上差 58 个部件；留给 M3 的检测器当事实报。
- **spec-open-items 第 4 节**：影子覆盖在求解侧生效、皮肤里的副本树、copy 块内修改的展开序（hull 的 `faces` 已做）。
- **评分并行**：没做；SpaceCenter 实测超预算再按原则 18 加。
- **查询点 Morton 排序**：bench 里量到单线程再快 30%，评分里没用（逐部件 600 点、收益小）。

## 7. 本机常见坑（CLAUDE.md 之外）

- Windows 上子 agent 会写 CRLF；`.gitattributes` 已 `eol=lf`，合并后 `git diff --stat` 看一眼。
- Git Bash 里 `node -` 加 heredoc 吃反斜杠；编辑脚本写成文件再跑。
- `s3d` 的路径处理把 `\` 换成 `/`（`abs_path`），import 的解析按 spec/01 的 1.8；工程目录用正斜杠写。
- `cargo test` 的 `corpus_score` 与 `sample_intents` 要读 `corpus/`，在 crate 目录下跑也行（用 `CARGO_MANIFEST_DIR` 定位）；`sample_intents` 的 SpaceCenter 一项按 `S3D_PROTOTYPE` 环境变量找原型 clone，找不到就跳过。
- 原型的 `work-snapshot/*/project.json` 里的 `source` 是相对路径，指向 `../../../../SpaceCenter_Rebuild/...`，按同级目录摆放才对。
