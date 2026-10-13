# 外部数据候选清单（M0 交付物，待用户确认）

状态：**草稿，待用户确认**。起草日期 2026-10-12；网络核实日期见各条目。

依据：`docs/PLAN.md` 第 6 节（校准集 = 三个实战工程 + 外部数据；外部数据没有"原文"，只验校准序前三项与 compare 的 missing / extra）、6.1 节（三类候选、入选标准写成数字、`corpus/external/<名>/` + `SOURCES.md`、下载每次单独确认）、第 9 节 M0 行（"外部数据候选清单经用户确认"）、决定 6（几何同形类型检测用外部模型验收）。

本文只列候选、不下载、不建目录。候选经用户确认后才建 `corpus/external/<名>/`。

## 0. 为什么要外部数据

现有校准集只有三个实战工程（`corpus/reference/MANIFEST.md`：dorm 4540 节点 / 9683 条事实，h1 3874 / 15232，sc 149508 / 275343；单位 stud = 0.28 m），三者来自同一个美术团队、同一条导出流水线、同一套命名习惯——`Dormitory_02_ExterWall_01` 这类 **PascalCase 词段 + 下划线 + 序号、中心词在最后** 的写法。原型的 `tools/roles.mjs` 词表与 `corpus/intents/roles.spacecenter.json` 的八条规则型意图就是对这套习惯写的（"名字末尾的词是 Wall / Stair / Platform → structure"），换一个团队就失效。所以凡是在三个工程上调出来的东西——校准序、检测器阈值、具名约束的默认值、角色规则——都有"对一套资产过拟合"的风险，而我们看不出来。

外部数据补三样东西：

1. **另一套命名习惯**：验证规则型意图是数据不是代码（PLAN 原则 15、4.5 节）。合格的检验是：SpaceCenter 的意图文件放到外部图上应当大量落空，而针对外部图新写一份意图文件应当覆盖它的绝大多数部件，两者都不改代码。
2. **显式的实例结构**：模块套件拼出来的图天然是"同一网格多实例 + 格点阵列 + 镜像街区"，而且拼图脚本知道真值（哪些实例是同一件、哪些成阵列）。这是阵列 / 镜像召回和"享元 → 几何同形"类型检测（决定 6：进 M3，用外部模型验收）唯一能拿到真值的地方；三个工程里没有这种真值。
3. **不规则几何**：扫描的岩石、地形、雕塑写不出基本体，用来验 `L_mesh`（网格按真实信息量计）和写不出时的退路（网格引用 / 凸包 / 代理体），以及 compare 的 missing / extra 在非基本体上是否仍然有意义。

外部数据没有"原文"，所以只验校准序的前三项（全网格 > 全坐标 > 逆向）与 compare 的 missing / extra（PLAN 第 6 节）。"原文"这一项只能由 2c 的合成基线提供——它不需要外部资源。

## 1. 入选标准（数字）

每条标准给阈值和量法。"部件"指 GLB 里带网格的节点（不含纯变换的组节点）；"唯一网格"按 glTF 的 mesh 索引数。

| # | 标准 | 阈值 | 怎么量 |
|---|---|---|---|
| 1 | 部件数 | 整图候选：**≥ 2000 部件，至少一张**；上限 50 000（分块验收已有 sc 的 149508，外部数据不需要更大）。不规则模型：单件即可，但 **≥ 10 000 三角形**（低于这个数的"岩石"往往是手工低模，不算不规则） | 读 GLB 的 `nodes[]` 里带 `mesh` 的个数；三角形数读 accessor |
| 2 | 实例结构 | 部件数 / 唯一网格数 **≥ 4**（即同一网格平均 4 个以上实例，整图的多实例部件占比 ≥ 50%）；**≥ 10 处**平移阵列（n ≥ 3、等间距）与 **≥ 2 处**镜像；拼图脚本要把阵列 / 镜像清单写成真值 JSON | 从 GLB 统计 mesh 复用；阵列 / 镜像由拼图脚本输出清单 |
| 3 | 命名习惯 | 节点名有语义（**≥ 80%** 的部件名含可读词），且与 SpaceCenter 的 `PascalCase_Index` 风格不同；与 `roles.spacecenter.json` 的词表重叠 **< 30%**，使 SpaceCenter 意图在它上面命中率 **< 30%**，而为它新写的意图能覆盖 **≥ 90%** 的部件 | 用 `nameWords` 的切词法统计；意图命中率在 M3a 实测 |
| 4 | 碰撞 / 辅助体 | 至少一张图含 **≥ 50 个**碰撞体或辅助体（隐形挡墙、触发区、白模体积），在名字或分组路径上可识别，用的词 **不在** SpaceCenter 词表里 | 套件本身一般不带，拼图时按另一套词加进去并记真值 |
| 5 | 许可证 | **CC0 优先，其次 CC-BY 4.0**（要在 `SOURCES.md` 记署名行）；**不收** CC-BY-NC / ND、自定义许可、不明许可。资产页面与包内 LICENSE 冲突时按更严的算，并列为待确认 | 抄页面原文与核实日期进 `SOURCES.md` |
| 6 | 格式 | glTF 2.0 / GLB 直接可读优先；FBX / OBJ 需转换的记录转换工具版本与命令；**不收**只有 .blend / .max 源的 | — |
| 7 | 规模上限 | 单文件 **≤ 100 MB**，单个候选目录 **≤ 150 MB**，`corpus/external/` 总计 **≤ 500 MB**；贴图一律取 1K（逆向、评分、compare 都不用贴图） | GLB 不进 git（与 `work/` 同样 gitignore），只有 `SOURCES.md`、拼图脚本、真值清单进 git |
| 8 | 单位与尺度 | 单位明确（米）；拼图的格距明确、写进真值清单；与 stud 工程混用时按 `unit` 换算 | — |
| 9 | 可复现 | 下载 URL 固定且带版本号或 commit；拼出来的图由脚本从套件生成，脚本、种子、套件版本号进 git，任何人能重新拼出同一张图 | — |

一张图不必满足全部 9 条：2a 的拼图要满足 1–4、9；2b 的不规则模型满足 1（三角形数）、5–7 即可；5–8 对所有候选都适用。

## 2. 候选条目

### 2a. 模块套件拼图（Kenney / Quaternius / 同类 CC0 套件）

先说结论性的事实（核实日期 2026-10-12）：**没有找到任何一套 CC0 套件自带 ≥ 2000 部件的现成关卡文件**。Kenney、Quaternius、KayKit 三家的官方页面都不提示例场景；Kenney 的 Godot 启动包带一张预置地图但只有 14 KB（Godot 二进制资源，规模远小于 2000，且要用 Godot 才能导出 GLB）；Quaternius 的付费 Source 层才带引擎工程。所以 2a 的任何一个候选都要**自己拼**：写一个确定性脚本（固定种子）按格点放置套件件，同时输出 GLB 与真值 JSON（实例、阵列、镜像、碰撞体清单）。这件事本身是 M2 / M3 前的一项小工作，见第 6 节的问题 2。

| | A1 Kenney City Kit 四件套 | A2 KayKit Dungeon Remastered | A3 Quaternius Downtown City MegaKit | A4 Quaternius Medieval Village MegaKit |
|---|---|---|---|---|
| 官方页面 | [Roads](https://kenney.nl/assets/city-kit-roads) · [Suburban](https://kenney.nl/assets/city-kit-suburban) · [Commercial](https://kenney.nl/assets/city-kit-commercial) · [Industrial](https://kenney.nl/assets/city-kit-industrial)；拼图参考 [Starter Kit City Builder](https://github.com/KenneyNL/Starter-Kit-City-Builder) | [itch.io](https://kaylousberg.itch.io/kaykit-dungeon-remastered)；1.0 镜像 [GitHub](https://github.com/KayKit-Game-Assets/KayKit-Dungeon-Remastered-1.0) | [quaternius.com](https://quaternius.com/packs/downtowncitymegakit.html) | [quaternius.com](https://quaternius.com/packs/medievalvillagemegakit.html) |
| 许可证 | **CC0 1.0**（四个页面都标 "Creative Commons CC0" 并链到 CC0 deed；启动包代码 MIT、资产 CC0） | **CC0 1.0**（页面原文 "Creative Commons Zero v1.0 Universal"；仓库根有 LICENSE.txt） | **冲突，待确认**：套件页标 CC0 并链到 CC0 deed，但 [quaternius.com/license.html](https://quaternius.com/license.html) 是自定义的 "Quaternius Asset License (QAL) v1.0"（最后更新 2026-08-28；不要求署名、禁止转售或再分发资产本身；§7 说改版不溯及已获取的资产），全文不提 CC0 | 同 A3 |
| 资产数（页面原文） | Roads "Files 90×"、Suburban "40×"（itch 页 "Over 35 models"）、Commercial "50×"、Industrial "40×"；合计约 220 个文件；版本 Roads 1.1、Suburban 2.0、Commercial 2.1、Industrial 2.0 | 免费层 "200 beautifully crafted 3d dungeon assets"，Extra（付费）275+；GitHub 1.0 镜像可见 ≥ 114 个 `.glb`（列表被截断，实际更多）、204 个 `.fbx` | 页面计数 315（"over 300"），2026-05 发布；免费层 "60-70% of my pack"，其余 Pro / Source | 页面计数 304，2025-01 发布；Standard 与 Pro 层都免费（合计 100%），Source 付费 |
| 格式 | GLB（Kenney 知识库："The glTF file format is distributed as GLB"）+ FBX + OBJ（Suburban 另有 DAE / STL） | glTF（1.1 起拆成 .gltf + .bin）+ FBX + OBJ | OBJ / FBX / glTF；Source 层另有 .blend | 同 A3；Source 层带"每个模型的自定义碰撞体"（付费，不收） |
| 示例场景 | 官方页无；启动包有 `sample map/map.res`（14 103 字节，Godot 4.6 资源，F3 加载；只用 15 种模型）——规模不够、格式不直接 | 官方页与 GitHub 树里都没有 example / sample / scene / demo | 页面写 "7 pre-built example buildings"，不说在哪个层、什么格式；没有整城示例 | 无 |
| 节点命名风格举例 | **kebab-case 小写**：`building-small-a`、`building-garage`、`road-straight-lightposts`、`road-intersection`、`pavement-fountain`、`grass-trees-tall`（启动包 `models/` 的 GLB 文件名；GLB 内部节点名未核实，预期与文件名相同） | **snake_case 小写 + 变体后缀**：`floor_tile_big_grate_open`、`floor_foundation_front_and_sides`、`barrier_half`、`column`、`barrel_small_stack`、`banner_patternA_red`、`chest_gold`（GitHub 镜像 `Assets/gltf/` 的文件名） | 未核实。旁证：第三方 Godot 移植的 Modular Sci-Fi 包（[Malcolmnixon](https://github.com/Malcolmnixon/Quaternius-Modular-Scifi-Pack)）里是 `wall_1`、`column_slim`、`floor_tile_corner`、`door_single_wall_side_a`、`props_crate`——移植者可能改过大小写 | 未核实 |
| 碰撞 / 辅助体 | 无（拼图时自己加） | 无 | 无（Source 层有，付费） | 无（Source 层有，付费） |
| 单位 | 1 单位 = 1 格（Kenney 城市件按格拼） | 按格，格距未核实 | 按格，格距未核实 | 按格，格距未核实 |
| 单文件大小 | 每套件 zip 2–3 MB（Suburban 1.9 MB、Roads 2.3 MB） | 免费层 31 MB | 未核实 | 未核实 |
| 核实日期 | 2026-10-12 | 2026-10-12 | 2026-10-12 | 2026-10-12 |

与 SpaceCenter 词表的重叠：Kenney 的 `building`、`road`、`pavement`、`grass`、`tree` 都在 SpaceCenter 词表里（building / road / grass / tree 分别归 structure / terrain / decor / decor，pavement 归 terrain），但切词后落在词尾的多是 `a`、`b`、`lightposts`、`fountain`、`tall` 这类词，SpaceCenter 的"末尾词"规则会大面积落空或误判——这正是想要的效果（第 1 节标准 3 要的是意图落空，不是词完全不相交）。KayKit 的 `floor`、`column`、`barrel`、`chair` 也在词表里，情况类似。

其他看过但不列为候选的：KayKit Medieval Hexagon（200+，CC0，六边形地块，拓扑不是正交格，阵列检测口径不同）、KayKit City Builder Bits（"32+" 件，CC0，太少）、Quaternius Modular Sci-Fi MegaKit（277 件，许可证问题同 A3）、Quaternius Ultimate Modular Ruins（90 件，只有 FBX / OBJ / Blend，无 glTF）、Amazon Lumberyard Bistro（CC-BY 4.0，FBX，整条街是一个美术手工场景、不是套件，实例真值拿不到）。

**2a 推荐：A1（Kenney 四件套自拼）为主，A2（KayKit Dungeon）为备选或第二张图。** 理由：A1 许可证最干净、GLB 直接可读、文件最小、四个套件合起来约 220 种件足够多样，kebab-case 与 SpaceCenter 差异最大，而且室外城市格点与 SpaceCenter（室外园区）题材相近、便于对照解释分数差异；A2 是室内格点，墙 / 地板 / 楼梯的阵列最密，snake_case 是第三种命名风格，若预算允许作为"一外一内"的第二张图。A3 / A4 在许可证冲突澄清前不收。

### 2b. 不规则模型（Poly Haven / Sketchfab 按 CC0 筛选）

Poly Haven 全站 CC0、无需登录、每个模型有 glTF（.gltf + .bin + 贴图，1K / 2K / 4K / 8K）与 Blend / FBX / USD，面数与尺寸由 `api.polyhaven.com/info/<id>` 直接给出（下表数字即取自该 API，核实日期 2026-10-12；尺寸字段无单位，按 boulder_01 页面的 "1.8m" 推断为 mm）。Sketchfab 用 `license=cc0` 筛选可得博物馆扫描，但**下载需登录 Sketchfab 账号**（平台惯例，未逐一核实），只作备选。

| | B1 Poly Haven `boulder_01` | B2 Poly Haven `mountainside` | B3 Poly Haven `gothic_statue` | B4 Sketchfab / Smithsonian CC0 扫描 |
|---|---|---|---|---|
| 类别 | 岩石（单块巨石） | 地形 / 崖面（整片山坡） | 雕塑（哥特式石像） | 雕塑 / 岩石 / 结构体扫描 |
| URL | [polyhaven.com/a/boulder_01](https://polyhaven.com/a/boulder_01) | [polyhaven.com/a/mountainside](https://polyhaven.com/a/mountainside) | [polyhaven.com/a/gothic_statue](https://polyhaven.com/a/gothic_statue) | Smithsonian "George Washington Bust"（300 000 面，uid `23630d35f855409e9c00c810b1416c71`）；Smithsonian "Apollo 11 Command Module (Combined)"（721 399 面，uid `372bb6781922471cada4e0a9bd5c61fb`）；DigitalAtlasOfAncientLife "Igneous rock: Basalt (A'a Lava Rock)"（363 958 面，uid `cbbe1e1ae71b4e44acc6bf1b5245f1bb`）；nebulousflynn "Lion Statue - optimized"（28 488 面，uid `62662d27c94d4994b2479b8de3a66ca7`）。地址形如 `https://sketchfab.com/3d-models/<uid>` |
| 三角形数 | **123 976**，带 LOD | **287 755**，带 LOD | **27 739** | 2.8 万 – 72 万（见左） |
| 尺寸 | 1.27 × 1.83 × 1.00 m | 10.2 × 5.3 × 10.5 m | 1.48 × 1.56 × 1.74 m | 未核实 |
| 许可证 | CC0（页面原文 "CC0"） | CC0 | CC0 | 全部为 Sketchfab 标注 "CC0 Public Domain"；Smithsonian 另在 [3d.si.edu](https://3d.si.edu/collections/openaccesshighlights) 提供 OBJ / glTF 的 Open Access 下载（是否需账号未核实） |
| 格式 / 大小 | glTF：`.gltf` 2.8 KB + `.bin` 2 938 808 字节，1K 贴图三张合计约 2.8 MB；总计约 6 MB | glTF，`.bin` 未核实（按 boulder 的每三角形约 24 字节估约 7 MB），1K 贴图约 3 MB | glTF，估 < 2 MB + 贴图 | GLB，Sketchfab 列的包大小 7 – 130 MB 不等 |
| 作者 / 日期 | Rico Cilliers，2024-02-06 | Dario Barresi（摄影）/ Rico Cilliers（处理），2024-02-06 | Benny Weimer，2026-06-30 | 各馆 |
| 核实日期 | 2026-10-12 | 2026-10-12 | 2026-10-12 | 2026-10-12（Sketchfab API） |

Poly Haven 还有可替换的同类：`namaqualand_boulder_02`（183 681 面，2.5 × 1.2 × 0.9 m）、`namaqualand_cliff_01`（176 938 面，8.3 × 4.4 × 5.0 m）、`rock_face_01`（20 174 面，7.1 × 5.6 × 5.0 m，低面数崖面）、`marble_bust_01`（17 456 面，0.27 × 0.30 × 0.51 m）、`horse_head`（12 068 面）、`concrete_cat_statue`（11 950 面）。rocks 类别共 37 个模型，decorative 类别 76 个。

**2b 推荐：B1 + B2。** 理由：两者都是直接下载、CC0、带 LOD（同一个物体的 4 档面数可以顺带验 `L_mesh` 对信息量的单调性），一个是手持尺度的凸块（124 k 面），一个是场景尺度的大片崖面（288 k 面、10 m 跨度）——后者还能测几何判角色里"又大又扁 → terrain"这条默认行为。B3 作为可选第三件：面数低但形状有细长、凹陷与底座，是基本体代理最吃力的类型。B4 只在前三者不够或要验"结构体扫描"（Apollo 11 有内部与分离部件）时再提，因为要登录下载。

### 2c. 合成基线（新语言手写小场景，导出 L0 / L1 / L2）

合成基线补的是**真值原文**：外部数据只有网格没有原文，校准序的第四项（原文）和"逆向结果与原文的真值指标"（如 `eval.json` 的 objects / kinds / facts / compression）只能在有原文的场景上验。它不需要任何外部资源，产物是 `.s3d` 原文 + 新编译器导出的 L0 / L1 / L2 GLB（M1 之后才能导出；M0 只写原文）。

| | C1 新写 `depot`（货场）场景 | C2 扩写现有 spaceport | C3 把 2a 的拼图写成 `.s3d` |
|---|---|---|---|
| 内容 | 用新语言手写 150 – 300 条语句，展开后 ≥ 1000 部件：`repeat` 阵列（含带例外的）、`mirror`、`def` 类型多实例、`subtract` 外壳开口、`span` 斜撑；**显式写碰撞体与辅助体**（隐形挡墙、触发区、白模体积，带 `~role=`）；节点命名用**第三套习惯**（例如 snake_case 带角色前缀：`col_fence_e_03`、`trg_gate_in`、`mass_block_b`） | 在 `corpus/examples/spaceport.s3d`（112 行；已有 L0 / L1 / L2 导出与参考输出包）基础上加碰撞体、辅助体和第二种命名，保持原有语句不动 | 2a 的拼图脚本不直接写 GLB，而是生成一份 `.s3d`：每个套件件是一个网格引用，摆放用 `repeat` / `mirror`，再由编译器导出 L0 GLB 作为"外部图" |
| 补什么 | 真值原文；阵列 / 镜像 / 类型的真值清单；碰撞 / 辅助体的真值；第三套命名 | 真值原文已有；新增碰撞 / 辅助体真值 | 让外部图也有原文，且阵列 / 镜像 / 类型真值天然就是原文里的语句 |
| 代价 | 一天左右手写 + 自审；不依赖外部资源 | 半天；但 spaceport 的命名（`stage1`、`tower2`、`legs`、`decks`：小写单词无前缀）与 SpaceCenter 差异不大，规模也小（M3 验收已经在用它） | 依赖新语言的网格引用语法（规范第 M0 章节要先定）与 2a 的下载 |
| 风险 | 自己写的场景会无意中迎合自己的检测器（"写得出来的才写"）；要靠 2a / 2b 的外部图制衡 | 同上，且规模不够 | 它不再是"外部数据没有原文"的那一类，评分口径要分开记 |

**2c 推荐：C1 必做；C3 作为 2a 拼图脚本的产物形式一并做（若网格引用语法在 M0 定下来）；C2 不单列。** 理由：C1 是唯一同时提供原文、碰撞 / 辅助体真值和第三套命名的地方，成本最低；C3 几乎不增加工作量却让外部图有了原文，但要先在第 6 节确认口径。

## 3. 候选对照入选标准的打分表

列号对应第 1 节的 9 条标准。"满足（拼后）"指套件本身不满足、按第 6 节问题 3 的目标规模拼出来后满足；"—"指该标准对此类候选不适用。

| 候选 | 1 部件数 | 2 实例结构 | 3 命名习惯 | 4 碰撞 / 辅助 | 5 许可证 | 6 格式 | 7 规模上限 | 8 单位 | 9 可复现 |
|---|---|---|---|---|---|---|---|---|---|
| A1 Kenney City Kit 四件套 | 满足（拼后） | 满足（拼后；约 220 种件，比值可 ≥ 10） | 满足（kebab-case 文件名）；GLB 内部节点名**未知** | 满足（拼时加） | 满足（CC0） | 满足（GLB） | 满足（每套 2 – 3 MB） | 满足（1 单位 = 1 格；是否按米**未知**） | 满足（版本号 + 脚本 + 种子） |
| A2 KayKit Dungeon Remastered | 满足（拼后） | 满足（拼后） | 满足（snake_case） | 满足（拼时加） | 满足（CC0） | 满足（.gltf + .bin） | 满足（31 MB） | **未知**（格距未核实） | 满足（itch 1.1 / GitHub 1.0） |
| A3 Quaternius Downtown City | 满足（拼后） | 满足（拼后） | **未知** | 满足（拼时加） | **不满足**（页面 CC0 与 QAL v1.0 冲突，按更严算） | 满足（glTF） | **未知** | **未知** | **未知**（页面无版本号） |
| A4 Quaternius Medieval Village | 满足（拼后） | 满足（拼后） | **未知** | 满足（拼时加） | **不满足**（同 A3） | 满足（glTF） | **未知** | **未知** | **未知** |
| B1 Poly Haven boulder_01 | 满足（123 976 面） | — | — | — | 满足（CC0） | 满足（glTF） | 满足（约 6 MB） | 满足（1.83 m） | 满足（API 给 files_hash） |
| B2 Poly Haven mountainside | 满足（287 755 面） | — | — | — | 满足（CC0） | 满足（glTF） | 满足（估 ≈ 10 MB） | 满足（10.5 m） | 满足 |
| B3 Poly Haven gothic_statue | 满足（27 739 面） | — | — | — | 满足（CC0） | 满足（glTF） | 满足 | 满足（1.74 m） | 满足 |
| B4 Sketchfab / Smithsonian 扫描 | 满足（2.8 万 – 72 万面） | — | — | — | 满足（标注 CC0） | 满足（GLB） | **未知**（7 – 130 MB，逐件挑） | **未知** | **不满足**（需登录；无版本号） |
| C1 合成 depot | 部分（按设计 ≥ 1000，不要求 2000） | 满足（原文即真值） | 满足（第三套） | 满足 | 满足（本仓库） | 满足 | 满足 | 满足 | 满足 |
| C2 扩写 spaceport | **不满足**（规模小） | 满足 | **不满足**（与 SpaceCenter 差异小） | 满足（扩写后） | 满足 | 满足 | 满足 | 满足 | 满足 |
| C3 拼图写成 .s3d | 满足（拼后） | 满足（语句即真值） | 同所用套件 | 满足 | 同所用套件 | 满足 | 满足 | 满足 | 满足 |

## 4. 用途映射：每个候选验什么

| 用途 | 用哪些候选 | 具体验什么、过线是什么 |
|---|---|---|
| 校准序前三项（全网格 > 全坐标 > 逆向） | A1 / A2（整图）、B1 / B2 / B3（单件） | 对每个候选分别写三份：全网格引用版、全坐标基本体版（拼图脚本直接给；不规则模型用包围盒 / 凸包代理）、逆向版；S 必须严格单调。整图验的是"结构档逆向能压过全坐标"，单件验的是"网格信息量项不会让一块岩石的网格引用比它的包围盒代理还便宜" |
| compare 的 missing / extra | A1 / A2、B1 / B2 / B3 | 整图：三视角 missing / extra 与 `corpus/reference/work/panels-g10` 的 dorm / h1 同量级（iso missing ≤ 0.01、extra ≤ 0.06）；不规则模型：全网格版 missing = extra = 0（自检），代理版的 missing / extra 随面数档位（LOD）单调 |
| 规则型意图不特化（原则 15、4.5） | A1（必）、A2、C1 | 把 `roles.spacecenter.json` 原样跑在外部图上，命中率 < 30%；为外部图新写一份意图（只改 JSON），覆盖 ≥ 90% 的部件；两次都不改代码。C1 用第三套命名做同样的事 |
| 几何同形的类型检测（决定 6） | A1 / A2（拼图脚本输出两个版本：享元版——同一 mesh 多实例；拷贝版——每个实例独立 mesh，顶点相同） | 享元版走"同网格实例"路径，拷贝版走"几何同形"路径，两者得到的类型划分与真值清单一致（≥ 95% 实例归入正确类型）；拷贝版比享元版多付的时间有上界 |
| 阵列 / 镜像召回 | A1 / A2（真值清单）、C1（原文） | 真值清单里的平移阵列召回 ≥ 90%、镜像召回 ≥ 80%，误报（把不是阵列的写成阵列）≤ 5%；带例外的阵列（拼图脚本故意挖掉的格）要写成"阵列 + 例外"而不是散件 |
| 碰撞 / 辅助体：角色执行与"只比看得见的" | A1 / A2（拼时加）、C1 | 一条规则型意图把 `ghost-barrier` 一类的词标成 collision 后：它们照常拟合、但不当参照物、compare 不计入；辅助体不进几何 |
| `L_mesh` 按真实信息量 | B1 / B2（Poly Haven 自带 4 档 LOD） | 同一物体 4 档面数的网格引用代价单调递增，且与三角形数的关系近似对数（不是线性） |
| 几何判角色的默认行为（无意图时） | B2（10 m 跨度的扁平崖面）、A1 的地块 | 无意图时所有部件当同一种；带"又大又扁 → terrain"意图时 B2 归 terrain |
| 真值原文 / 校准序第四项 / `eval.json` 真值指标 | C1（必）、C3 | 原文 S 最低；逆向结果的 objects / kinds / facts / compression 不低于 spaceport 在参考输出包里的同名字段 |
| 单位换算与跨工程混用 | A1（格单位）与三个 stud 工程 | 评分与 compare 的输出在换算前后只差单位因子 |

分块验收（M3 的"整体 / 分块不做第二遍 / 分块 + 边缘第二遍"三份报告）不靠外部数据，sc 已经够大；但 A1 若拼到 3000 部件以上，可以把分块阈值临时调低做一次小规模的跨块阵列测试（阵列跨块边时的召回）。

## 5. 推荐组合

**A1（Kenney City Kit 四件套，脚本拼一张约 3000 部件的城区，kebab-case 命名，自加碰撞 / 辅助体）+ B1（Poly Haven boulder_01）+ B2（Poly Haven mountainside）+ C1（手写 depot 场景，第三套命名，原文即真值）；A2（KayKit Dungeon）与 B3（gothic_statue）列为可选的第二批，A3 / A4 / B4 不收。**

全部 CC0、全部直接下载、总下载量约 30 MB（四个 Kenney 套件 < 10 MB，两个 Poly Haven 模型 1K 贴图约 16 MB），`corpus/external/` 远低于 500 MB 上限。工作量：拼图脚本约一天（含真值清单与享元 / 拷贝两个版本），C1 原文约一天，下载与 `SOURCES.md` 半天。

## 6. 待用户确认的问题

每条给了默认假设；不回复就按默认走，但下载仍然逐次确认。

1. **是否接受 CC-BY 4.0。** 推荐组合全是 CC0，不需要；只有在想收 Amazon Lumberyard Bistro 这类手工场景时才涉及。默认：不收 CC-BY，标准 5 保留"其次 CC-BY"只作后备。
2. **自己拼图，还是继续找现成关卡。** 核实结果是没有 CC0 的现成 ≥ 2000 部件关卡（Kenney 启动包的 sample map 14 KB、Quaternius 示例在付费层）。拼图的布局是我们定的，外部的只是几何与命名——这样算不算"外部数据"？默认：接受脚本拼图，布局规则写进 `SOURCES.md`。
3. **每张图的目标规模。** 建议 A1 为 3000 ± 500 部件（约 50 × 50 格的道路 / 人行道 / 草地 + 300 – 500 栋建筑与路灯等小件），A2 若做为 2000 ± 300。要不要一张 2 万部件级的大图？默认：不做，分块规模用 sc。
4. **一张图还是两张（A1 单独，还是 A1 + A2 一外一内）。** 默认：M0 只确认 A1，A2 留到 M3 看阵列检测是否需要室内格点样本。
5. **Quaternius 的许可证冲突怎么处理。** 套件页标 CC0，站点许可证页是 QAL v1.0（2026-08-28）。选项：直接排除 / 发邮件问作者 / 按 QAL 收（它允许用、不要求署名，只禁转售资产本身——我们不转售，但它不是 CC）。默认：排除。
6. **拼图的节点命名与碰撞 / 辅助体用词。** 建议节点名 = 套件文件名 + 格位置（`road-straight_12_07`、`building-small-a_03_18`），分组路径按街区（`district-b/block-03/…`）；碰撞 / 辅助体用不在 SpaceCenter 词表里的词（如 `ghost-barrier`、`zone-event`、`massing-proxy`），这样 SpaceCenter 的意图对它们必然落空。默认：如此。
7. **不规则模型选 B1 + B2，还是 B1 + B3（雕塑）。** 要不要 B4 这种结构体扫描（Apollo 11，有内部与分离部件，但要登录 Sketchfab 下载——我不能登录，需用户自己下）。默认：B1 + B2，B3 可选，B4 不收。
8. **存储方式。** GLB 不进 git（与 `work/` 一样 gitignore，`SOURCES.md` 记 URL 与哈希，换机器重新下载），还是走 git LFS？贴图统一取 1K？默认：gitignore + 哈希，1K。
9. **C3 的口径。** 拼图若写成 `.s3d`（网格引用 + `repeat` / `mirror`），它就有原文了，与"外部数据没有原文"的口径不同。要不要同时输出一份 GLB-only 版本按无原文口径记分？默认：两份都出、分开记；前提是网格引用语法在 M0 的规范里定下来。
10. **C1 的题材、规模与时机。** 建议题材"货场"（集装箱阵列、围栏、门架、灯杆、挡墙、触发区），≥ 1000 部件，M0 就写原文（作为规范逐条对照的例子之一），M1 后导出。默认：如此。
11. **Sketchfab 账号。** 仅当问题 7 选 B4 时需要用户自己登录下载并交给我文件。

## 7. 确认后的流程

1. 候选经用户确认后才建目录：`corpus/external/<名>/`，建议名 `kenney-city`、`kaykit-dungeon`、`polyhaven-boulder_01`、`polyhaven-mountainside`、`synthetic-depot`（后者也可以放 `corpus/examples/`，见问题 10）。
2. 每个目录带 `SOURCES.md`：来源 URL、版本号或哈希（Poly Haven 的 `files_hash`、Kenney 的 zip 版本号、GitHub commit）、许可证原文与核实日期、作者、下载日期、文件清单与大小、转换工具与命令（若有）、拼图脚本路径与种子、真值清单路径。
3. **每次下载单独确认**：下载前列出文件名、来源、大小；下载到独立的空目录，视为不可信数据，不在其中运行任何脚本。
4. GLB / 贴图不进 git（gitignore，与 `work/` 同样处理）；`SOURCES.md`、拼图脚本、真值清单、`.s3d` 原文进 git；每完成一项提交一次。
5. 入选与指标写进 `docs/PROGRESS.md`，`corpus/README.md` 的 `external/` 行补上目录名。

## 附：本次核实用到的来源（2026-10-12）

- Kenney：kenney.nl 的四个 City Kit 页面（许可证、文件数、版本）；[kenney.nl 知识库：导入 3D 模型](https://kenney.nl/knowledge-base/game-assets-3d/importing-3d-models-into-game-engines)（GLB 说明）；[KenneyNL/Starter-Kit-City-Builder](https://github.com/KenneyNL/Starter-Kit-City-Builder)（README 与文件树：MIT / CC0、`models/*.glb`、`sample map/map.res`）；itch.io 与 OpenGameArt 页面（Suburban "Over 35 models" / "40 objects"、zip 大小）。
- KayKit：itch.io 的 Dungeon Remastered、Medieval Hexagon 页面；[kaylousberg.com/game-assets](https://kaylousberg.com/game-assets) 与 City Builder Bits 页；GitHub `KayKit-Game-Assets/KayKit-Dungeon-Remastered-1.0` 的文件树（命名风格）。
- Quaternius：quaternius.com 的 Downtown City MegaKit、Medieval Village MegaKit、Modular Sci-Fi MegaKit、Ultimate Modular Ruins 页面；[quaternius.com/license.html](https://quaternius.com/license.html)（QAL v1.0）；第三方移植 `Malcolmnixon/Quaternius-Modular-Scifi-Pack` 的文件树（命名旁证）。
- Poly Haven：`api.polyhaven.com/assets?t=models&c=rocks`（37 个）、`…&c=decorative`（76 个）、`api.polyhaven.com/info/<id>`（面数、尺寸、作者、日期）、`api.polyhaven.com/files/boulder_01`（glTF 文件大小）、polyhaven.com/a/boulder_01 页面（"1.8m"、"CC0"）。
- Sketchfab：`api.sketchfab.com/v3/licenses`（CC0 的 uid `7c23a1ba438d4306920229c12afcb5f9`）、`api.sketchfab.com/v3/search?type=models&downloadable=true&license=cc0&q=…`（rock / statue / smithsonian 三次检索的面数与 uid）。
- Smithsonian Open Access：2020 年 Khronos / Smithsonian 新闻稿与报道（OBJ / glTF、CC0）；[3d.si.edu 的 Open Access 集合页](https://3d.si.edu/collections/openaccesshighlights)（未能直接打开，下载是否需账号未核实）。
- Bistro：ASWF 资产库 wiki 列为 CC BY 4.0（ORCA）；仅作"不收"的记录。
