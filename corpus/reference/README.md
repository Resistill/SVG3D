# corpus/reference/

原型冻结前导出的参考输出包（2026-10-12，原型 commit `14582a3`，Node 22.20.0）。M1 到 M3 的验收都对着它比（按定义与容差，不逐字节）。每个文件的来源命令、耗时与指标表见 `MANIFEST.md`。

结构：

- `examples/`：7 个示例的编译 JSON（`<名>.compile.json`：逐物体 `world{R,t}`、`box`、`tight`、`facts`、`concepts`、`errors`、`warnings`）与 `--outline=2 --semantics` 的概要（`<名>.outline.txt`）。
- `exports/`：原型逆向结果（`../exports/*.s3d`）的编译 JSON 与真值评测（`<名>.eval.json`，`bin/eval.mjs --json` 的输出）。`curves.L2` 没有逆向结果，`l2fb` 没有原文，所以各缺一项。
- `work/`（**gitignore，230 MB**，要从原型机器单独拷）：三个实战工程的完整 `score.json`、`compare.json` 与差异图、iso / top / iso2 渲染图、`scene.compile.json`、`project.json`；另有 `panels-g10/`。
- `tools/`、`logs/`：导出用的脚本与日志。

## 验收时用哪份基准

三个实战工程有两种口径，不能混比：

| 基准 | 档位 | 用来验什么 |
|---|---|---|
| `work/panels-g10/`（sp、dorm、h1） | 结构档新建，不细化 | M3 的逆向本身：“三个实战工程的 compare 不差于 g10”指的是这份 |
| `work/dorm/`、`work/h1/` | 语义档逆向 + `refine --auto --to=structure --budget=10` | M3 的 `refine --auto` 验收：“细化后的 S 与 compare 不差于原型”指的是这份 |
| `work/sc/` | 结构档、分块、整张图 | M3 的分块三份报告与 M4 的规模验收；g10 没有 sc |

语义档 + 细化的 dorm / h1 比 g10 差很多（dorm 的 iso missing 0.31 对 0.0025），这是口径不同，不是退化。

## 已知的坑

- `work/*/compare.json` 与 `render-*.json` 里的 `file` 字段是导出机器上的绝对路径，图就在同目录下。
- 三个工程的单位是 stud（0.28 m），examples 与 exports 是米。
- `work/panels-g10/*.score.json` 是约 6 KB 的摘要，不是完整 `score.json`。
