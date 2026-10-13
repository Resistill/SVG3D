# corpus/

测试语料。全部来自原型仓库 `C:\Workspace\SVG3D`（同级目录 `../SVG3D`），原型 commit：`14582a3fd4b0299680a18e5d63107f66bf322243`（2026-10-12 搬入时的 HEAD）。

| 子目录 | 来源 | 说明 |
|---|---|---|
| `examples/` | 原型 `examples/*.s3d` | 7 个示例：basics、curves、rotation、semantics、spaceport、spaceport_blueprint、spaceport_edit |
| `examples/lib/semantics.s3d` | 原型 `lib/semantics.s3d` | 通用语义库 |
| `exports/` | 原型 `out/` 下的 `*.glb`、`*.s3d`、`*.report.json` | L0/L1/L2 导出与原型的逆向结果（`*.L0.s3d` 等是原型逆向的输出，不是手写）。**注意**：原型的 `out/` 被 git 忽略，这些文件是 2026-10-12 工作目录里的状态，不对应某个 commit，可用原型的 `bin/export.mjs` 重新生成。总大小约 22 MB，未超过 200 MB，全部复制，没有省略；原型 `out/reference/`（参考输出包，另一个任务在填）不在其中 |
| `exports/<名>.abs.s3d`、`exports/<名>.rev.s3d` | 原型 commit `439abb8` 对 `exports/<名>.glb` 重新逆向的全坐标版与具名版（M2 补，2026-10-13） | 评分校准序的固定输入；与 `reference/exports/scores.json` 配套，见 `reference/MANIFEST.md` |
| `intents/roles.spacecenter.json` | 翻译自原型 `tools/roles.mjs` 的词表 | 规则型意图样本：是数据不是代码，只对 SpaceCenter 的命名习惯有效；M3 验收用 |
| `reference/` | 原型 `out/reference/`（另一个任务填入） | 参考输出包，见该目录的 README |

## import 路径

示例里只有两处 `import`：`spaceport_blueprint.s3d` 与 `spaceport_edit.s3d` 的 `import "spaceport.s3d"`，二者与被引用文件同在 `corpus/examples/`，路径在新位置仍然成立。`semantics.s3d` 的注释提到“可以 import 上面这份”，不是 import 语句；示例没有任何一处写 `import "../lib/semantics.s3d"`，所以 `examples/lib/semantics.s3d` 目前没有被示例引用（原型的 `docs/agent.md` 提到工程的 `semantics.s3d` 会 import 通用语义库，那是工程文件，不在本语料里）。示例内容未做任何修改。

## 规则

- `work/`（实战工程）与 `reference/work/` 不进 git。
- 外部数据放 `external/<名>/`，带 `SOURCES.md`（PLAN.md 6.1，M0 确认候选后再建）。
