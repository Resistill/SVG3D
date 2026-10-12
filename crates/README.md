# crates/

Rust 工作区，M1 再建 Cargo 工程；现在只记布局（PLAN.md 第 2 节）。crate 前缀 `s3d-`。

| crate | 职责 | 出 wasm |
|---|---|---|
| `s3d-geom` | 几何内核：基本体（SDF、射线求交、包围盒、采样）、三角网格 BVH 最近点（parry3d）、凸块 SAT（自写）、容差简化凸包（自写，≤ 24 点）、非封闭网格的体积与内外近似（自写）、平面聚类、多边形、CPU 软件光栅（compare 用） | 是 |
| `s3d-lang` | 编译器：解析 → 展开 → 派生与修改 → 求解 → 事实 → 语义；只依赖 `s3d-geom` | 是 |
| `s3d-score` | 自评分数（描述长度、网格计价、双向残差、体积误差、坐标依赖；增量）+ 真值评测（原 eval.mjs） | 是 |
| `s3d-reverse` | 逆向：部件读取（含决定的执行）、检测器流水线、候选筛选、子集逆向（refine / detect 的底层）、稳定命名、分块 | 是 |
| `s3d-render` | 渲染：wgpu，基本体实例化直绘；无头（PNG）与 WebGPU 共用 WGSL | 是 |
| `s3d-project` | 工程目录、层、意图（读写、重绑、生命周期）、提议、日志 | 否（native） |
| `s3d-tool` | 命令表（一张表，三种外壳：CLI、MCP、HTTP）；crop、sidecar、unit 猜测 | 否（native） |
| `s3d-export` | GLB、Roblox | 否（native） |

确定性要求（原则 18，进 CI）：禁用 std 的 sin / cos / exp / pow / hypot / atan2，统一走 libm；并行只允许静态分块 + 按块号顺序归约；不迭代 HashMap；glam 开 `libm` + `scalar-math`；parry3d 开 `enhanced-determinism`、不开 simd；wasm 不开 `+simd128`。
