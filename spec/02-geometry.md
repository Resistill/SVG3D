# 2 实体与曲线的几何定义

**范围。** 本章定义每种实体（solid）与曲线（curve）在其**局部坐标系**里的几何：参数、点集、有符号距离场（SDF）、射线求交、局部包围盒、紧包围盒、表面采样、体积、凸块分解（第 6 章空间事实用的碰撞代理）、三角化（渲染与导出用的网格）；曲线的采样、参数化、取点与扫掠框架；曲面锚点；布尔容器的操作数规则、距离场组合与三角化路径；容器的包围盒。几何内核与渲染着色器按本章同一份定义实现，两者的差异由对拍测试封顶。

本章**不**定义：实体怎样被放进父坐标系（定位、朝向、锚点的求值、后置，见 03-solving.md）；语句的词法与属性表（见 01-syntax.md）；凸块之间的分离量与事实判定（见 06-facts.md）；材质与外观（见 10-materials.md）。

**依赖的章。** 01-syntax.md（属性语法；数值、向量、列表、枚举字面量；报错格式与错误恢复）、03-solving.md（旋转矩阵 $R$ 与变换 $p' = R\,p + t$ 的精确定义；锚点与定位点；后置子物体；span 的端点、sweep / pipe 的路径点在父框架里的求值）。被依赖：03、04、06、07、09。

**记法。** 向量是列向量，$\hat x, \hat y, \hat z$ 是局部坐标轴的单位向量；$\lVert v \rVert$ 是欧氏长度，$\mathrm{unit}(v) = v / \lVert v \rVert$；$\mathrm{clamp}(x, a, b) = \min(\max(x, a), b)$；角度一律用度，三角函数的参数按度理解（实现换算成弧度）；$\mathrm{atan2}(y, x)$ 返回 $(-180, 180]$ 里的度数。点集 $S$ 指实体占据的闭集，$\partial S$ 是它的边界（表面）。

## 2.1 总约定

### 2.1.1 局部坐标系与方向词

- 右手系，Y 向上。$+x$ = right / east，$-x$ = left / west；$+z$ = front / south，$-z$ = back / north；$+y$ = top / up，$-y$ = bottom / down。方向词与局部轴的对应对每种实体、每个容器都相同，不随实体种类变化。
- 每个实体、每条曲线、每个容器都有一个**局部坐标系**（局部系）。它在父坐标系里的位置由一个变换 $p' = R\,p + t$ 给出，$R$ 是 3×3 正交矩阵（$\det R = \pm 1$；$\det R = -1$ 出现在镜像副本上），$t$ 是平移。$R$、$t$ 由第 3 章求解；本章的全部定义都在局部系里给出，经 $R$、$t$ 变换后得到世界几何：$S_{\text{world}} = \{R\,p + t : p \in S\}$。几何定义**不区分** $\det R$ 的正负：镜像副本的点集就是局部点集经反射变换的像（例如楔形的高边因此到了另一侧）。三角化时为了法线朝外要按 $\det R$ 调整顶点顺序，见 2.1.9。
- **面**：一个节点的 top / bottom / left / right / front / back 六个面指它**局部包围盒**（2.1.2）的六个面：top 是 $y = y_1$ 的面，bottom 是 $y = y_0$，right 是 $x = x_1$，left 是 $x = x_0$，front 是 $z = z_1$，back 是 $z = z_0$。对不是盒子的实体，面仍按包围盒定义，即使实体表面只在一条线或一个点上碰到它：横放的圆柱（`cyl axis=x`）的 top 面是 $y = 2R$ 的平面（沿一条母线与圆柱相切），left / right 面是两个端面圆盘所在的平面；球的六个面都只在一个点上相切；楔形的 top 面在斜面之上。要取真实表面上的点，用曲面锚点（2.4 节）。
- 本章里"底面"指 $y = y_0$ 的面，"顶面"指 $y = y_1$ 的面。

### 2.1.2 局部包围盒、原点与定位点

- **局部包围盒**（下文简称"盒"）$B = [x_0, x_1] \times [y_0, y_1] \times [z_0, z_1]$ 是局部系里包住实体点集的轴对齐盒；每种实体在 2.2 节给出 $B$ 的公式。记盒心 $c = \tfrac12(\min + \max)$、半尺寸 $h_B = \tfrac12(\max - \min)$（下标 $B$ 用来与实体的高 $h$ 区分）、**底面中心** $b = (c_x, y_0, c_z)$。盒是定位（第 3 章）、方向词锚点、`B.w / B.h / B.d`、容器包围盒（2.6 节）用的量；它必须包住点集，但不要求最紧（例如 revolve 的盒按整圈算，2.2.7）。
- **原点**：局部系的原点 $(0,0,0)$。"实体的原点在底面中心"的精确含义是：对 box、wedge、cyl、cone、sphere、mesh，局部包围盒的底面中心 $b$ **就是**原点，即 $c_x = c_z = 0$、$y_0 = 0$。注意 `cyl axis=x` / `axis=z` 的原点在包围盒底面中心，**不在轴线上**（轴线在 $y = R$，2.2.3）。
- 其余实体与曲线的原点由其参数的坐标决定，不一定是盒的底面中心：plate、hull 的原点是 `points` 坐标的原点；revolve 的原点在旋转轴上 $y = 0$ 处（轮廓的最低点可以不在 $y = 0$）；span 的原点是两端点的中点；sweep、pipe 与 line、bezier、spline 的局部系就是父坐标系（$R = I$、$t = 0$），原点是父系原点；arc、helix 的原点是圆心（在起点所在的高度）。
- **定位点**（pin）由第 3 章定义；本章只约定它用到的两个几何量：缺省定位点是局部包围盒的底面中心 $b$（对 `inside=`、`under=` 等关系缺省另有规定，见第 3 章）；`pin=origin` 取原点 $(0,0,0)$。因此 `plate … at=P` 把轮廓包围盒的底面中心放到 P，`plate … at=P pin=origin` 才把轮廓坐标原点放到 P；hull、revolve、arc、helix 同理。
- **零盒** $Z = [0,0]^3$：空容器（2.6 节）与出错后退化的节点（第 1 章）用它。零盒的 $c = h_B = b = 0$。

### 2.1.3 尺寸词 w / h / d

节点的 $w = x_1 - x_0$、$h = y_1 - y_0$、$d = z_1 - z_0$ 是**局部包围盒**沿局部 $x$、$y$、$z$ 的边长（第 3 章的 `B.w / B.h / B.d` 读的就是它们；有 `envelope` 的 group 读 envelope 的边长）。对带 `size=w,h,d` 的实体它们就是参数；对其他实体是盒的边长：

| 实体 | $w$ | $h$ | $d$ |
|---|---|---|---|
| box、wedge、mesh | `size` 的三个分量 | | |
| cyl / cone `axis=y` | $2R$ | 参数 `h` | $2R$ |
| cyl / cone `axis=x` | 参数 `h`（长度） | $2R$ | $2R$ |
| cyl / cone `axis=z` | $2R$ | $2R$ | 参数 `h` |
| sphere | $2r$ | $2r$ | $2r$ |
| plate | 轮廓 $x$ 的跨度 | 参数 `h` | 轮廓 $z$ 的跨度 |
| hull、revolve、span、sweep、pipe、曲线、容器 | 盒的边长（各节公式） | | |

$R = \max(r_1, r_2)$，见 2.2.3。方向词 `w / h / d` 只用于读尺寸，不改变方向词与轴的对应（2.1.1）。

### 2.1.4 数值与容差常数

数值是 IEEE 754 双精度；长度的单位是场景单位（文件里数字的单位）。本章用到的常数全部列在这里，各节按名字引用；带单位的常数**不随 `metre` 换算**（它们与原型一致，语料依赖它们）。

| 名字 | 值 | 单位 | 用途 |
|---|---|---|---|
| $\varepsilon_{\text{len}}$ | $10^{-9}$ | 场景单位 | 退化长度：零长度的边、重合的点、span 两端重合、曲线总长为零 |
| $\varepsilon_{\text{dir}}$ | $10^{-6}$ | 无 | 方向去重（$\lvert n \cdot m \rvert \ge 1 - \varepsilon_{\text{dir}}$ 视为同向）、单位向量的退化判定、倒圆的最小偏移 |
| $\varepsilon_{\text{up}}$ | $10^{-4}$ | 无 | 扫掠框架：$\lVert T \times \hat y \rVert < \varepsilon_{\text{up}}$ 视为竖直段（2.3.7） |
| $\varepsilon_{\text{area}}$ | $10^{-12}$ | 场景单位² | 二维多边形面积为零的判定（带符号面积的两倍 $\lvert 2A \rvert < \varepsilon_{\text{area}}$） |
| $\varepsilon_{\text{cap}}$ | $10^{-6}$ | 场景单位 | revolve 自动封口：端点半径 $> \varepsilon_{\text{cap}}$ 才补轴上的点（2.2.7） |
| $\varepsilon_{\text{closed}}$ | $10^{-9}$ | 度 | arc 闭合判定：$\lvert a_1 - a_0 \rvert \ge 360 - \varepsilon_{\text{closed}}$（2.3.2） |
| $\varepsilon_{\text{hull}}$ | $10^{-9} \cdot \max(e, 10^{-9})$ | 场景单位 | 凸包共面判定，$e$ 是输入点三个坐标跨度的最大值（2.2.6） |
| $\varepsilon_{\text{hit}}$ | $10^{-4} \cdot \mathrm{diag}(B)$ | 场景单位 | 距离场步进的命中容差与最小步长，$\mathrm{diag}(B)$ 是该实体局部包围盒的对角线长（2.1.6.6） |
| $K_{\max}$ | 256 | 次 | 距离场步进的最大步数 |
| $n_{\text{cyl}}$ | 48 | 段 | 圆柱、圆台、圆杆、旋转体三角化的周向段数（2.1.9） |
| $n_{\text{sph}}$ | 48 × 24 | 段 | 球三角化的经向 × 纬向段数 |
| $n_{\text{circ}}$ | 20 | 点 | 圆截面扫掠的截面点数（2.2.9；影响点集，不只是渲染） |
| $n_{\text{part}}$ | 12 | 边 | 圆柱、圆台、圆杆、旋转体凸块的边数（2.2.3、2.2.7） |

"$\lvert v \rvert$"对向量表示逐分量取绝对值；判定用的比较一律写成明确的不等式，边界情况（恰好相等）按不等式的写法归属，两份实现只要按同一不等式比较就得到同一结果。

### 2.1.5 每种实体给出的运算

2.2 节对每种实体按同一顺序给出下列运算的定义。每一项都在局部系里定义；世界系里的对应量由 $R$、$t$ 变换得到。

1. **参数表**：名、类型、默认值、取值范围、非法值的报错。属性是否允许出现在该语句上由第 1 章的属性表决定，本章只定义几何上的合法取值。
2. **几何**：实体占据的点集 $S$（闭集、有界）。这是其余各项的依据：任何两项定义若与点集矛盾，以点集为准。
3. **SDF**：函数 $f : \mathbb R^3 \to \mathbb R$，$f(p) < 0$ 当且仅当 $p$ 在 $S$ 内部，$f(p) > 0$ 当且仅当 $p$ 在 $S$ 外，$f(p) = 0$ 当且仅当 $p \in \partial S$，且 $\lvert f(p) \rvert \le \mathrm{dist}(p, \partial S)$。处处取等号的叫**精确** SDF；否则叫**下界** SDF（内外判断仍然精确）。下界 SDF 足以做距离场步进（2.1.6.6）与布尔组合（2.5.2）；需要真实距离的运算（最近点）用各节的"精确距离"定义。
4. **射线求交**：射线 $p(s) = o + s\,d$，$\lVert d \rVert = 1$，$s \ge 0$。命中参数 $s^* = \inf\{ s \ge 0 : p(s) \in S \}$（$o \in S$ 时 $s^* = 0$）；无此 $s$ 为未命中。命中点 $p(s^*)$ 与该点的外法线 $n$（$s^* = 0$ 且 $o$ 在内部时 $n = -d$）。解析求交的实体给公式；步进求交的实体给步进规则（2.1.6.6）。有的实体还给出射线在 $S$ 内的**参数区间集** $I = \{[s_{\text{in}}, s_{\text{out}}]\}$，供解析布尔用。
5. **局部包围盒** $B$（2.1.2）。
6. **紧包围盒** $T(R, t)$：给定世界变换后，世界系里包住 $R\,S + t$ 的最小轴对齐盒，写成有限点集的包围盒或闭式公式（2.1.8）。
7. **表面采样**：面片列表与每个面片的参数映射；给定点数 $N$ 后，2.1.7 的规则唯一确定 $N$ 个点。
8. **体积** $V$。
9. **凸块**：第 6 章空间事实用的碰撞代理，一组凸多面体或球（2.1.6.1），带 `approx` 标记：`approx=false` 表示代理按本规范就是该实体在事实里的几何；`approx=true` 表示代理是包住实体的外接形状，与实体可以相差任意多。
10. **三角化**：渲染与导出用的闭合三角网格，顶点顺序使法线朝外（2.1.9）。
11. **示例**。

实体按几何路径分两类（2.5 节用）：**距离场实体** box、wedge、cyl、cone、sphere、plate、hull、revolve、span（三种形态）——有 SDF，渲染时在片元里按种类解析求交或步进；**三角化路径实体** sweep、pipe、mesh——点集由三角网格定义，没有 SDF。

### 2.1.6 通用构造

下面的构造被多种实体共用，各节按编号引用。

#### 2.1.6.1 凸多面体

- **表示**：顶点列表 $V = (v_0, \dots, v_{m-1})$ 与面片列表 $F$；每个面片是顶点序号的有序列表，从外面看逆时针。面片是**极大的共面多边形**（相邻共面三角形合并成一个面片；共面判定按各实体给出的容差，默认 $\varepsilon_{\text{hull}}$）。
- **面法线**用 Newell 公式：面片顶点 $q_0, \dots, q_{k-1}$，$n' = \sum_{i} (q_i - q_{i+1}) \times (q_i + q_{i+1})$（下标模 $k$ 求和的次序按 $i = 0, 1, \dots$），$n = \mathrm{unit}(n')$；面方程 $n \cdot p = d_f$，$d_f = n \cdot q_0$。
- **凸块用的法线集与棱集**：法线集是全部面法线按方向去重的结果（$\lvert n \cdot m \rvert \ge 1 - \varepsilon_{\text{dir}}$ 视为同一个，保留先出现的；正负不分）；棱集是全部面片边的方向 $\mathrm{unit}(q_{i+1} - q_i)$ 按同样规则去重的结果，长度 $< \varepsilon_{\text{len}}$ 的边丢弃。去重的扫描顺序：按面片顺序、面片内按顶点顺序。第 6 章的分离量只用顶点、法线集与棱集。
- **半空间 SDF**：$f(p) = \max_{f \in F} (n_f \cdot p - d_f)$。它在内部精确，在外部是下界 SDF（对凸多面体 $\max_f(\cdot) \le \mathrm{dist}(p, \partial S)$）。
- **精确距离**：$\mathrm{dist}(p, \partial S) = \min_f \mathrm{dist}(p, \text{面片 } f)$，点到多边形的距离 = 投影落在多边形内时到平面的距离，否则到多边形边界各线段的距离的最小值；内部点取负号。
- **射线裁剪**：对射线 $o + s d$，令 $a_f = n_f \cdot d$、$b_f = d_f - n_f \cdot o$。$s_{\text{in}} = \max\{ b_f / a_f : a_f < 0 \}$（无此面时 $s_{\text{in}} = 0$），$s_{\text{out}} = \min\{ b_f / a_f : a_f > 0 \}$（无此面时 $+\infty$）；若存在 $a_f = 0$ 且 $b_f < 0$ 的面则未命中。命中当且仅当 $s_{\text{in}} \le s_{\text{out}}$ 且 $s_{\text{out}} \ge 0$；$s^* = \max(s_{\text{in}}, 0)$，法线取给出 $s_{\text{in}}$ 的面的 $n_f$（并列时取 $F$ 里靠前的面）；$s_{\text{in}} < 0$（起点在内部）时 $s^* = 0$、$n = -d$。区间集 $I = \{[\max(s_{\text{in}},0), s_{\text{out}}]\}$。
- **体积**：$V = \tfrac16 \sum_{(a,b,c)} a \cdot (b \times c)$，对面片按 2.1.9 的扇形规则三角化后逐三角形求和，求和次序按面片、面片内按三角形顺序。
- **盒**：顶点的轴向最小 / 最大值。

#### 2.1.6.2 二维多边形

二维多边形是顶点序列 $q_0, \dots, q_{m-1}$（$m \ge 3$），边 $e_i = (q_i, q_{i+1 \bmod m})$。坐标记 $(u, v)$；plate 用 $(x, z)$、revolve 用 $(\rho, y)$、sweep 截面用 $(u, v)$ 代入。

- **带符号面积** $A = \tfrac12 \sum_{i} (u_i v_{i+1} - u_{i+1} v_i)$。$\lvert 2A \rvert < \varepsilon_{\text{area}}$ 为退化（面积为零）。
- **规整**：末点与首点重合（距离 $< \varepsilon_{\text{len}}$）时去掉末点；相邻重合点（距离 $< \varepsilon_{\text{len}}$）只保留第一个；两种情况都给**警告**。规整后少于 3 个点报错（文案由各实体给出）。
- **简单性**：任意两条不相邻的边不得相交（含端点相碰），相邻边只在公共顶点相交；判定用线段相交的标准方向测试（叉积 $(b - a) \times (q - a)$ 的符号，$\lvert \cdot \rvert < \varepsilon_{\text{area}}$ 视为共线，共线时按区间重叠判定）。不满足的多边形叫**自交**，各实体对自交报错。
- **绕向**：$A > 0$ 叫正向，$A < 0$ 叫反向。需要固定绕向的地方（三角化、sweep 截面）把反向序列整个反转。注意 plate 的 $(u, v) = (x, z)$：俯视（北朝上）逆时针的轮廓有 $A < 0$（因为 $+z$ 朝向观察者的下方）。
- **点在多边形内**（交叉数法）：$q$ 在内当且仅当下述计数为奇数：对每条边 $(a, b)$，若 $(a_v > q_v) \ne (b_v > q_v)$ 且 $q_u < a_u + (q_v - a_v)(b_u - a_u)/(b_v - a_v)$，计数加一。边界上的点按该不等式归属。
- **二维 SDF**：$d_{\text{seg}}(q; a, b) = \lVert q - a - \mathrm{clamp}\big(\tfrac{(q - a)\cdot(b - a)}{(b - a)\cdot(b - a)}, 0, 1\big)(b - a) \rVert$；$f_2(q) = \sigma\,\min_i d_{\text{seg}}(q; q_i, q_{i+1})$，$\sigma = -1$ 当 $q$ 在内（交叉数法），否则 $+1$。对简单多边形它是精确 SDF。各实体可以指定**排除**某些边不参与 $\min$（revolve 的轴边，2.2.7），排除边只参与内外判定。
- **耳切三角化**（确定性）：先把序列规整并反转成正向（$A > 0$）；去掉所有"直"顶点（$\lvert (q_{i} - q_{i-1}) \times (q_{i+1} - q_i) \rvert < \varepsilon_{\text{area}}$ 的 $q_i$，从 $i = 0$ 起反复扫描直到没有）；然后循环：从当前序列的 $i = 0$ 起找第一个**耳**——$(q_{i-1}, q_i, q_{i+1})$ 的带符号面积 $> 0$ 且序列里其余顶点都不在该三角形内或边界上（边界判定用 $\varepsilon_{\text{area}}$ 容差：三个叉积都 $\ge -\varepsilon_{\text{area}}$ 算在内）；输出三角形 $(q_{i-1}, q_i, q_{i+1})$，从序列删除 $q_i$；剩 3 个顶点时输出它并结束。找不到耳（只在数值退化时发生）则取第一个带符号面积 $> 0$ 的顶点当耳；仍没有则报错"轮廓退化"。输出的三角形序列就是该多边形的三角化，各实体的凸块与三角网格按它构造。
- **凸性**：所有相邻边叉积同号（$\ge 0$ 或 $\le 0$，容差 $\varepsilon_{\text{area}}$）为凸。

#### 2.1.6.3 三角网格实体

三角化路径实体（sweep、pipe、mesh）与所有实体的三角化结果都是**三角网格** $M = (t_0, \dots, t_{k-1})$，$t_i = (a_i, b_i, c_i)$，法线 $n_i = \mathrm{unit}((b_i - a_i) \times (c_i - a_i))$ 朝外。

- **闭合**：每条边恰好被两个三角形以相反方向使用。本章构造的网格都闭合；mesh 引用的外部网格不保证（2.2.11）。
- **内外**：$p$ 在内当且仅当广义绕数 $w(p) = \tfrac{1}{4\pi} \sum_i \Omega_i(p) > \tfrac12$，$\Omega_i$ 是三角形 $t_i$ 对 $p$ 张的有符号立体角（van Oosterom–Strackee 公式）。对闭合网格它等价于射线奇偶。
- **射线求交**：对每个三角形做 Möller–Trumbore 求交（$s \ge 0$，重心坐标 $\ge 0$、和 $\le 1$，边界算命中），$s^* = \min$；并列时取序号小的三角形；法线取该三角形的 $n_i$。起点在内（绕数）时 $s^* = 0$。
- **精确距离**：$\min_i \mathrm{dist}(p, t_i)$（点到三角形的距离），内部取负。网格实体没有 SDF；步进求交、布尔组合不适用于它们（2.5.3）。
- **面积与体积**：$A = \sum_i \tfrac12 \lVert (b_i - a_i) \times (c_i - a_i) \rVert$；$V = \lvert \tfrac16 \sum_i a_i \cdot (b_i \times c_i) \rvert$，按 $i$ 顺序求和。不闭合的网格的 $V$ 按同一公式算并标为近似。
- **采样**：面片 = 三角形，按 $i$ 顺序（2.1.7）。
- **盒**：顶点的最小 / 最大；**紧包围盒**：变换后顶点的最小 / 最大。

#### 2.1.6.4 轴系、圆与圆弧的轴向极值

- **轴系** $(c_0, a, u_0)$：$c_0$ 是轴上一点，$a$ 是轴的单位方向，$u_0 \perp a$ 是角度零点方向；$\mathrm{dir}(\theta) = \cos\theta\, u_0 + \sin\theta\, (a \times u_0)$，即 $\theta$ 按右手法则绕 $a$ 增大。轴上距 $c_0$ 为 $t$、半径 $r$、角 $\theta$ 的点是 $c_0 + t\,a + r\,\mathrm{dir}(\theta)$。
- **圆的轴向极值**：世界系里圆心 $c$、半径 $r$、轴 $a$（单位）的圆，沿世界轴 $k \in \{x, y, z\}$ 的范围是 $[c_k - r\sqrt{1 - a_k^2},\ c_k + r\sqrt{1 - a_k^2}]$。
- **圆弧的轴向极值**：弧 $p(\theta) = c + r(\cos\theta\, u + \sin\theta\, v)$，$\theta \in [\theta_0, \theta_1]$（$u, v$ 单位、正交）。沿轴 $k$：$\rho_k = \sqrt{u_k^2 + v_k^2}$，$\psi_k = \mathrm{atan2}(v_k, u_k)$；候选角 = $\{\theta_0, \theta_1\} \cup \{\psi_k + 180 j : j \in \mathbb Z,\ \theta_0 \le \psi_k + 180 j \le \theta_1\}$；范围是候选角处 $p_k$ 的最小 / 最大。

#### 2.1.6.5 Catmull-Rom 与 Bézier

- **均匀 Catmull-Rom**：控制点 $P_0, \dots, P_{n-1}$（$n \ge 2$），每段 $m$ 个样本。开放曲线：段 $i = 0..n-2$，邻点 $P_{i-1}$、$P_{i+2}$ 越界时取端点（$P_{-1} = P_0$、$P_n = P_{n-1}$）；样本 $t = j/m$，$j = 0..m-1$，最后补 $P_{n-1}$，共 $(n-1)m + 1$ 个点。闭合曲线：段 $i = 0..n-1$，邻点下标模 $n$，共 $nm$ 个点，不重复首点。每段
$$C(t) = \tfrac12\big[\,2P_1 + (P_2 - P_0)\,t + (2P_0 - 5P_1 + 4P_2 - P_3)\,t^2 + (-P_0 + 3P_1 - 3P_2 + P_3)\,t^3\,\big],$$
其中 $P_0..P_3$ 是该段的四个控制点（$P_1$、$P_2$ 是段的两端）。$n = 2$ 的开放曲线直接返回两个控制点（不细分）。
- **Bézier**：控制点 $P_0, \dots, P_{n-1}$（$n \ge 2$），$n - 1$ 次曲线 $C(t) = \sum_i \binom{n-1}{i}(1-t)^{n-1-i} t^i P_i$，按 de Casteljau 递推计算；样本 $t = j/m$，$j = 0..m$，共 $m + 1$ 个点。

#### 2.1.6.6 距离场步进（射线求交的步进规则）

没有解析求交的距离场实体（plate、revolve、布尔体）按下面的规则求 $s^*$。实现可以用任何能得到相同结果（在 $\varepsilon_{\text{hit}}$ 内）的方法；本规则是对拍的依据。

1. 用 2.1.6.1 的射线裁剪求射线与**局部包围盒**（布尔体用 2.5.4 的盒）的区间 $[s_0, s_1]$；未命中盒即未命中。
2. $s \leftarrow s_0$；若 $f(p(s_0)) < 0$（起点在内）则 $s^* = s_0$，法线按 $s_0 = 0$ 时 $-d$、否则盒面法线。
3. 重复至多 $K_{\max}$ 次：$\delta = f(p(s))$；若 $\delta < \varepsilon_{\text{hit}}$ 则命中，$s^* = s$；否则 $s \leftarrow s + \max(\delta, \varepsilon_{\text{hit}})$；若 $s > s_1$ 则未命中。
4. 步数用尽视为未命中。命中点的法线 $n = \mathrm{unit}(\nabla f)$，梯度用中心差分，步长 $\varepsilon_{\text{hit}}$。

只有下界 SDF 才能保证步进不穿过表面；本章给出的 SDF 都满足。

### 2.1.7 表面采样的统一方案

采样点数 $N$ 由调用者给出；本节定义给定 $N$ 时的点集，使两份实现得到同一组点（同一顺序）。采样只用实体自身的三角网格或解析面片，**不**把网格顶点当样本。

**面片**。每个实体在 2.2 节给出一个有序的面片列表 $(\Pi_1, \dots, \Pi_m)$；每个面片有面积 $A_j$ 与映射 $\varphi_j : [0,1)^2 \to \partial S$，映射对面积均匀（参数域里均匀分布的点在面片上按面积均匀）。面片类型：

| 类型 | 参数 | $\varphi(u, v)$ | $A$ |
|---|---|---|---|
| 矩形 $\mathrm{Rect}(a; e_1, e_2)$ | 角点 $a$，边向量 $e_1, e_2$ | $a + u\,e_1 + v\,e_2$ | $\lVert e_1 \times e_2 \rVert$ |
| 三角形 $\mathrm{Tri}(a, b, c)$ | 三顶点 | $a + \sqrt u\,[(1 - v)(b - a) + v(c - a)]$ | $\tfrac12 \lVert (b - a) \times (c - a) \rVert$ |
| 母线面 $\mathrm{Lat}(c_0, a, u_0; r_0, t_0, r_1, t_1; \theta_0, \theta_1)$ | 轴系（2.1.6.4）；两端半径与轴向坐标；角范围（度） | $\omega = \begin{cases} v & r_0 = r_1 \\ \dfrac{\sqrt{r_0^2 + v(r_1^2 - r_0^2)} - r_0}{r_1 - r_0} & r_0 \ne r_1 \end{cases}$，$r = r_0 + \omega(r_1 - r_0)$，$t = t_0 + \omega(t_1 - t_0)$，$\theta = \theta_0 + u(\theta_1 - \theta_0)$；$c_0 + t\,a + r\,\mathrm{dir}(\theta)$ | $\tfrac{\theta_1 - \theta_0}{360}\,\pi(r_0 + r_1)\sqrt{(r_1 - r_0)^2 + (t_1 - t_0)^2}$ |
| 球面 $\mathrm{Sph}(c, r)$ | 球心、半径 | $\bar y = 2v - 1$，$\rho = r\sqrt{1 - \bar y^2}$，$\theta = 360 u$；$c + (\rho\cos\theta,\ r\bar y,\ -\rho\sin\theta)$ | $4\pi r^2$ |

母线面覆盖圆柱侧面（$r_0 = r_1$）、圆台侧面、圆锥侧面（一端 $r = 0$）与圆盘 / 圆环（$t_0 = t_1$）。

**分配**。$A = \sum_j A_j$（按 $j$ 顺序求和）。$n_j = \lfloor N A_j / A \rfloor$；余数 $\Delta = N - \sum_j n_j$ 按小数部分 $N A_j / A - n_j$ 从大到小（相等时 $j$ 小的优先）分给前 $\Delta$ 个面片各加一。$A_j = 0$ 的面片 $n_j = 0$。

**取点**。第 $k$ 个样本（$k = 1..N$）属于满足 $\sum_{i<j} n_i < k \le \sum_{i \le j} n_i$ 的面片 $j$，参数 $(u, v) = (H_2(k), H_3(k))$，点 $p_k = \varphi_j(u, v)$。$H_b(k)$ 是以 $b$ 为基的 radical inverse：$r \leftarrow 0,\ f \leftarrow 1/b$；当 $k > 0$：$r \leftarrow r + f\cdot(k \bmod b),\ k \leftarrow \lfloor k / b \rfloor,\ f \leftarrow f / b$；返回 $r$。运算按此顺序用双精度完成。

采样点的输出顺序就是 $k$ 的顺序。世界系里的样本是 $R\,p_k + t$。

### 2.1.8 紧包围盒的计算规则

紧包围盒 $T$ 是世界系里 $R\,S + t$ 的最小轴对齐盒，按真实形状算，不按局部盒的 8 个角。每种实体给出一个有限的**极值点集** $E$ 和 / 或若干**圆 / 圆弧**，使 $T$ = 它们变换后沿各轴的最小 / 最大：

- 点 $p \in E$ 贡献 $R\,p + t$ 的三个坐标；
- 圆（圆心 $c$、半径 $r$、轴 $a$，局部系）贡献 2.1.6.4 的圆极值，代入世界圆心 $R\,c + t$ 与世界轴 $R\,a$；
- 圆弧贡献 2.1.6.4 的弧极值，代入 $R\,c + t$、$R\,u$、$R\,v$。

多面体类实体（box、wedge、hull、plate、beam、steps、三角网格）的 $E$ 是全部顶点；圆类实体（cyl、cone、rod、revolve）用端面圆（或弧）；sphere 用球心 $\pm r$（三轴各自）。两份实现按同一公式得到同一 $T$。轴对齐（$R$ 是坐标轴的置换与反射）时 $T$ 与局部盒的变换一致。容器与曲线没有紧包围盒（输出为空）；布尔容器的紧包围盒见 2.5.4。

### 2.1.9 三角化的通用规则

三角化给渲染（不走距离场的路径）、导出与评分用，结果是 2.1.6.3 意义下的闭合三角网格，顶点在局部系。规则：

- **朝外**：每个三角形 $(a, b, c)$ 的 $(b - a) \times (c - a)$ 指向实体外。变换到世界系时若 $\det R < 0$，交换每个三角形的第 2、3 个顶点。
- **四边形**：从外面看逆时针的四边形 $(a, b, c, d)$ 分成 $(a, b, c)$、$(a, c, d)$。
- **扇形**：从外面看逆时针的凸多边形 $(q_0, \dots, q_{k-1})$ 分成 $(q_0, q_i, q_{i+1})$，$i = 1..k-2$；非凸多边形用 2.1.6.2 的耳切。
- **周向分段**：圆柱、圆台、圆杆、旋转体周向 $n_{\text{cyl}} = 48$ 段，顶点在 $\theta_i = 360 i / n_{\text{cyl}}$；球 $n_{\text{sph}} = 48 \times 24$。旋转体 `angle` $< 360$ 时周向段数 $n = \max(1, \lceil n_{\text{cyl}} \cdot \text{angle} / 360 \rceil)$。圆截面扫掠的截面点数 $n_{\text{circ}} = 20$ 是点集的一部分（2.2.9），不是渲染分段。
- **顺序**：三角形的输出顺序按各节给出的构造顺序；顶点坐标按公式直接计算，不做焊接或去重。
- 镜像副本、`copy`、模板实例的三角化与原件相同（在各自的局部系里）。

### 2.1.10 参数读取与报错

属性值的词法形式由第 1 章定义；本节约定几何参数的读取规则，各实体的参数表引用这些类型：

| 类型 | 合法值 | 不合法时 |
|---|---|---|
| 数 | 有限的双精度数（表达式求值后） | 报错"`<路径>` 的 `<属性>` 需要数字" |
| 正数 | 数且 $> 0$ | 报错"`<路径>` 的 `<属性>` 要大于 0" |
| 非负数 | 数且 $\ge 0$ | 报错"`<路径>` 的 `<属性>` 不能是负数" |
| 整数 | 数且等于某整数 | 报错"`<路径>` 的 `<属性>` 需要整数" |
| 三维向量 | 三个数 `x,y,z` 或 `(x,y,z)`；单个数 $v$ 展成 $(v, v, v)$ | 报错"`<路径>` 的 `<属性>` 需要三维向量 x,y,z" |
| 二元组 | 两个数 `a,b` | 报错"`<路径>` 的 `<属性>` 需要两个数：…"（各节给出文案） |
| 二元组列表 | `[(a,b); (a,b); …]` | 各节给出文案 |
| 三元组列表 | `[(x,y,z); …]`，每项可以是坐标或锚点（锚点在父框架里求值，第 3 章） | 各节给出文案 |
| 枚举 | 第 1 章列出的字面量之一 | 报错"`<路径>` 的 `<属性>` 只能是 …" |
| 布尔 | `true` / `false` | 报错"`<路径>` 的 `<属性>` 只能是 true / false" |
| 字符串 | 引号字符串 | 报错 |

报错后的处理按第 1 章的错误恢复：该语句的节点退化（零盒、放在父原点、不参与事实、不渲染），不影响其他语句。一条语句上写了该种类不接受的属性、或同时写了互斥的属性（各节列出），也按第 1 章报错。

## 2.2 实体

### 2.2.1 box

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `size` | 三维向量 $(w, h, d)$ | $(1, 1, 1)$ | 三个分量都 $> 0$ | 报错"`<路径>` 的 size 三个分量都要大于 0" |

**几何** $S = [-\tfrac w2, \tfrac w2] \times [0, h] \times [-\tfrac d2, \tfrac d2]$。盒心 $c = (0, \tfrac h2, 0)$，半尺寸 $h_B = (\tfrac w2, \tfrac h2, \tfrac d2)$。顶点编号（下文各处共用）：
$v_0 = (-\tfrac w2, 0, -\tfrac d2)$，$v_1 = (\tfrac w2, 0, -\tfrac d2)$，$v_2 = (\tfrac w2, 0, \tfrac d2)$，$v_3 = (-\tfrac w2, 0, \tfrac d2)$，$v_{4..7}$ 是 $v_{0..3}$ 把 $y$ 换成 $h$。

**SDF**（精确）
$$q = \lvert p - c \rvert - h_B,\qquad f(p) = \lVert \max(q, 0) \rVert + \min(\max(q_x, q_y, q_z), 0).$$

**射线**：2.1.6.1 的裁剪，面为六个面片（法线 $\mp\hat x, \mp\hat y, \mp\hat z$，$d_f$ 分别为 $\tfrac w2, \tfrac w2, 0, h, \tfrac d2, \tfrac d2$）。

**局部包围盒** $B = S$。

**紧包围盒**：极值点集 $E = \{v_0, \dots, v_7\}$。

**表面采样**（面片顺序）：bottom $\mathrm{Rect}(v_0; v_1 - v_0, v_3 - v_0)$，top $\mathrm{Rect}(v_4; v_5 - v_4, v_7 - v_4)$，left $\mathrm{Rect}(v_0; v_3 - v_0, v_4 - v_0)$，right $\mathrm{Rect}(v_1; v_2 - v_1, v_5 - v_1)$，back $\mathrm{Rect}(v_0; v_1 - v_0, v_4 - v_0)$，front $\mathrm{Rect}(v_3; v_2 - v_3, v_7 - v_3)$。

**体积** $V = w\,h\,d$。

**凸块**：一个凸多面体，顶点 $v_0..v_7$，面片（从外面看逆时针）bottom $(0,1,2,3)$、top $(4,7,6,5)$、left $(0,3,7,4)$、right $(1,5,6,2)$、back $(0,4,5,1)$、front $(3,2,6,7)$；`approx=false`。

**三角化**：六个面片按 2.1.9 的四边形规则各分两个三角形，按上面的面片顺序，共 12 个。

**示例**
```s3d
box crate size=5,4,5 on=floor
box slab size=8,0.5,8 pin=top at=0,10,0
```

### 2.2.2 wedge

**参数表**：与 box 相同（`size`，默认 $(1,1,1)$，三个分量 $> 0$）。

**几何**：底面矩形与 box 相同，$+x$ 一侧高、$-x$ 一侧为零，斜面沿 $z$ 不变：
$$S = \{(x, y, z) : \lvert x \rvert \le \tfrac w2,\ \lvert z \rvert \le \tfrac d2,\ 0 \le y \le h\,(x + \tfrac w2)/w\}.$$
顶点：$v_0 = (-\tfrac w2, 0, -\tfrac d2)$，$v_1 = (\tfrac w2, 0, -\tfrac d2)$，$v_2 = (\tfrac w2, h, -\tfrac d2)$，$v_3 = (-\tfrac w2, 0, \tfrac d2)$，$v_4 = (\tfrac w2, 0, \tfrac d2)$，$v_5 = (\tfrac w2, h, \tfrac d2)$。高边在 $+x$ 是定义；要让高边朝别的方向用朝向属性（第 3 章），镜像副本（$\det R = -1$）的高边自然到另一侧。

**SDF**（精确）：$f(p) = \sigma\big(f_2(x, y),\ \lvert z \rvert - \tfrac d2\big)$，其中 $f_2$ 是三角形 $(-\tfrac w2, 0), (\tfrac w2, 0), (\tfrac w2, h)$ 的二维 SDF（2.1.6.2），$\sigma(a, b) = \lVert (\max(a, 0), \max(b, 0)) \rVert + \min(\max(a, b), 0)$。

**射线**：2.1.6.1 的裁剪，面片见凸块。

**局部包围盒** $B = [-\tfrac w2, \tfrac w2] \times [0, h] \times [-\tfrac d2, \tfrac d2]$（与 box 相同，斜面之上的空间也在盒里）。

**紧包围盒**：$E = \{v_0, \dots, v_5\}$。

**表面采样**：bottom $\mathrm{Rect}(v_0; v_1 - v_0, v_3 - v_0)$，right $\mathrm{Rect}(v_1; v_2 - v_1, v_4 - v_1)$，slope $\mathrm{Rect}(v_0; v_2 - v_0, v_3 - v_0)$，back $\mathrm{Tri}(v_0, v_2, v_1)$，front $\mathrm{Tri}(v_3, v_4, v_5)$。

**体积** $V = \tfrac12 w\,h\,d$。

**凸块**：一个凸多面体，面片 back $(0,2,1)$、front $(3,4,5)$、bottom $(0,1,4,3)$、right $(1,2,5,4)$、slope $(0,3,5,2)$；`approx=false`。

**三角化**：两个三角形面片原样，三个四边形各分两个，共 8 个，按面片顺序。

**示例**
```s3d
wedge ramp size=3,5,5 against=house.right
wedge fin size=3.5,6,0.4 turn=180 at=stage1.w/2,0,0 pin=bottom-right
```

### 2.2.3 cyl 与 cone

`cyl` 与 `cone` 是两种语句种类（`kind(cone)` 只选中 cone），几何上都是**圆台**：底面半径 $r_1$、顶面半径 $r_2$、高 $h$。`cyl` 的 $r_1 = r_2 = r$；`cone` 的 $r_2$ 可以为 0（圆锥）。

**参数表**

| 属性 | 可出现在 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|---|
| `r` | cyl、cone | cyl：正数；cone：非负数 | 0.5 | 见右 | 报错（2.1.10 的文案） |
| `r2` | cone | 非负数 | 0 | $r$、`r2` 不得同时为 0 | 报错"`<路径>` 的 r 与 r2 不能都是 0" |
| `h` | cyl、cone | 正数 | 1 | $> 0$ | 报错 |
| `axis` | cyl、cone | 枚举 `x` / `y` / `z` | `y` | | 报错"`<路径>` 的 axis 只能是 x / y / z" |

`cyl` 语句上写 `r2` 按第 1 章的属性表报错。记 $r_1 = r$，$R = \max(r_1, r_2)$。

**轴系**（2.1.6.4）：

| `axis` | $c_0$（底面圆心） | $a$ | $u_0$（$\theta = 0$ 的方向） | $a \times u_0$ |
|---|---|---|---|---|
| `y` | $(0, 0, 0)$ | $\hat y$ | $\hat x$ | $-\hat z$ |
| `x` | $(-\tfrac h2, R, 0)$ | $\hat x$ | $\hat y$ | $\hat z$ |
| `z` | $(0, R, -\tfrac h2)$ | $\hat z$ | $\hat y$ | $-\hat x$ |

即 `axis=y` 时底面在 $y = 0$、$\theta$ 从 $+x$ 起俯视逆时针（$+x \to -z$）；`axis=x` 时轴线在高度 $y = R$、$r_1$ 端在 $x = -\tfrac h2$，$\theta$ 从 $+y$ 起绕 $+x$ 右手增大（$+y \to +z$）；`axis=z` 时轴线在 $y = R$、$r_1$ 端在 $z = -\tfrac h2$，$\theta$ 从 $+y$ 起绕 $+z$ 右手增大（$+y \to -x$）。

**几何**：$r(t) = r_1 + (r_2 - r_1)\,t / h$，
$$S = \{\, c_0 + t\,a + \rho\,\mathrm{dir}(\theta) : 0 \le t \le h,\ 0 \le \rho \le r(t),\ \theta \in [0, 360) \,\}.$$

**SDF**（精确）：对 $p$ 记 $t = (p - c_0)\cdot a$，$\rho = \lVert (p - c_0) - t\,a \rVert$；$f(p) = f_2(\rho, t)$，$f_2$ 是四边形 $(0,0), (r_1, 0), (r_2, h), (0, h)$ 在 $(\rho, t)$ 平面里的二维 SDF（2.1.6.2），**排除**轴边 $(0, h)\text{–}(0, 0)$（$r_1 = 0$ 或 $r_2 = 0$ 时退化成三角形，仍排除轴上的边）。它等于常见的"有盖圆台"闭式距离，实现可用闭式。

**射线**：记 $t_o = (o - c_0)\cdot a$，$d_a = d \cdot a$，$q = (o - c_0) - t_o a$，$d_\perp = d - d_a a$，$t(s) = t_o + s\,d_a$，$g(s) = r(t(s))^2 - \lVert q + s\,d_\perp \rVert^2$（$s$ 的二次多项式）。$S_t = \{ s \ge 0 : 0 \le t(s) \le h \}$（一个区间，$d_a = 0$ 时为全部或空）。射线在 $S$ 内的参数集 $I = \{ s \in S_t : g(s) \ge 0 \}$，由 $g$ 的根与 $S_t$ 的端点切出至多两段；$s^* = \min I$，不存在即未命中。法线：$s^*$ 落在 $t = 0$ 的端点时 $n = -a$，落在 $t = h$ 时 $n = a$，否则 $n = \mathrm{unit}\big(h\,\mathrm{unit}(p^* - c_0 - t^* a) - (r_2 - r_1)\,a\big)$（$p^*$ 在轴上时取 $-a$ 或 $a$ 按 $d_a$ 的符号）。

**局部包围盒**：`axis=y`：$[-R, R] \times [0, h] \times [-R, R]$；`axis=x`：$[-\tfrac h2, \tfrac h2] \times [0, 2R] \times [-R, R]$；`axis=z`：$[-R, R] \times [0, 2R] \times [-\tfrac h2, \tfrac h2]$。横放时原点在盒的底面中心，轴线在 $y = R$。

**紧包围盒**：两个圆——$(c_0, r_1, a)$ 与 $(c_0 + h\,a, r_2, a)$（半径为 0 的圆就是一个点）。

**表面采样**：$\mathrm{Lat}(c_0, a, u_0; 0, 0, r_1, 0; 0, 360)$（底面）、$\mathrm{Lat}(c_0, a, u_0; r_1, 0, r_2, h; 0, 360)$（侧面）、$\mathrm{Lat}(c_0, a, u_0; r_2, h, 0, h; 0, 360)$（顶面）；半径为 0 的端面面积为 0。

**体积** $V = \tfrac13 \pi h (r_1^2 + r_1 r_2 + r_2^2)$。

**凸块**：一个凸多面体，$n_{\text{part}} = 12$ 边的内接棱台：$B_i = c_0 + r_1\,\mathrm{dir}(30 i)$、$T_i = c_0 + h\,a + r_2\,\mathrm{dir}(30 i)$，$i = 0..11$。面片：底 $(B_0, B_{11}, B_{10}, \dots, B_1)$，顶 $(T_0, T_1, \dots, T_{11})$，侧 $(B_i, B_{i+1}, T_{i+1}, T_i)$，$i = 0..11$（下标模 12）。$r_2 = 0$ 时顶环换成一个顶点 $A = c_0 + h\,a$，侧面片为三角形 $(B_i, B_{i+1}, A)$，没有顶面片；$r_1 = 0$ 对称处理。`approx=false`：12 边形是内接多边形，与圆的最大偏差是 $R(1 - \cos 15°) \approx 0.034 R$，本规范把它定义为圆柱在事实里的几何（不标近似）。

**三角化**：周向 $n_{\text{cyl}} = 48$ 段，$\theta_i = 7.5\,i$，环点 $B_i$、$T_i$ 同上式（$i = 0..47$）。顺序：底面扇形 $(c_0, B_{i+1}, B_i)$，侧面四边形 $(B_i, B_{i+1}, T_{i+1}, T_i)$ 各分两个三角形，顶面扇形 $(c_1, T_i, T_{i+1})$，$c_1 = c_0 + h\,a$。半径为 0 的一端没有端面，该端的侧面四边形退化为三角形 $(B_i, B_{i+1}, A)$。

**示例**
```s3d
cyl cup r=0.6 h=1.2 on=table
cyl spare r=2.4 h=20 axis=x inside=hangar.body align=bottom
cone tower r=1.3 r2=0.7 h=32
cone spike r=1 h=2            // r2 默认 0：圆锥
```

### 2.2.4 sphere

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `r` | 正数 | 0.5 | $> 0$ | 报错 |

**几何** $S = \{ p : \lVert p - c \rVert \le r \}$，球心 $c = (0, r, 0)$。

**SDF**（精确）$f(p) = \lVert p - c \rVert - r$。

**射线**：$b = d \cdot (o - c)$，$k = \lVert o - c \rVert^2 - r^2$，$\Delta = b^2 - k$。$\Delta < 0$ 未命中；$s_1 = -b - \sqrt\Delta$，$s_2 = -b + \sqrt\Delta$；$s_2 < 0$ 未命中；$s^* = \max(s_1, 0)$，$I = \{[s^*, s_2]\}$，法线 $n = \mathrm{unit}(p(s^*) - c)$（$s_1 < 0$ 时 $n = -d$）。

**局部包围盒** $[-r, r] \times [0, 2r] \times [-r, r]$。

**紧包围盒**：世界球心 $R\,c + t$ 的三个坐标各 $\pm r$。

**表面采样**：$\mathrm{Sph}(c, r)$。

**体积** $V = \tfrac43 \pi r^3$。

**凸块**：球 $(c, r)$，`approx=false`（精确）。

**三角化**：$n_{\text{sph}} = 48 \times 24$ 的经纬网格。纬圈 $j = 1..23$：$\phi_j = -90 + 180 j / 24$；经线 $i = 0..47$：$\theta_i = 7.5\,i$；$v_{j,i} = c + r(\cos\phi_j \cos\theta_i,\ \sin\phi_j,\ -\cos\phi_j \sin\theta_i)$；南极 $P_S = c - r\hat y$，北极 $P_N = c + r\hat y$。顺序：南极扇形 $(P_S, v_{1,i+1}, v_{1,i})$（$i = 0..47$，下标模 48）；纬带 $j = 1..22$ 的四边形 $(v_{j,i}, v_{j,i+1}, v_{j+1,i+1}, v_{j+1,i})$ 各分两个；北极扇形 $(P_N, v_{23,i}, v_{23,i+1})$。

**示例**
```s3d
sphere ball r=0.8 on=table[0.6,1,-0.5]
sphere cap r=3.5 on=tank gap=-3.5
```

### 2.2.5 plate

平板：局部 $xz$ 平面里的多边形轮廓沿 $+y$ 拉伸厚度 $h$。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `points` | 二元组列表 $(x, z)$ | 必填 | 规整（2.1.6.2）后至少 3 个点；面积不为零；不自交 | 少于 3 点或分量不是数：报错"`<路径>`：plate 的 points 写成 [(x,z); (x,z); …]，至少 3 个点"；$\lvert 2A \rvert < \varepsilon_{\text{area}}$：报错"`<路径>`：plate 的轮廓面积是 0"；自交：报错"`<路径>`：plate 的轮廓自交" |
| `h` | 正数 | 0.2 | $> 0$ | 报错"`<路径>`：plate 的厚度 h 要大于 0" |

轮廓点按书写顺序保存，绕向不限（顺时针照用）；本节用 $q_0..q_{m-1}$ 表示规整后的轮廓，$\Omega$ 表示它围成的闭区域，$A$ 是 2.1.6.2 的带符号面积（$(u, v) = (x, z)$）。

**几何** $S = \{ (x, y, z) : (x, z) \in \Omega,\ 0 \le y \le h \}$。

**SDF**（精确）$f(p) = \sigma\big(f_2(x, z),\ \lvert y - \tfrac h2 \rvert - \tfrac h2\big)$，$f_2$ 是轮廓的二维 SDF，$\sigma$ 见 2.2.2。

**射线**：先求射线与厚度层 $0 \le y \le h$ 的参数区间 $[s_a, s_b]$（$d_y = 0$ 时：$0 \le o_y \le h$ 则 $[0, +\infty)$，否则未命中）。候选命中：(a) 若 $(x, z)(s_a) \in \Omega$，候选 $s_a$，法线 $-\hat y$（$d_y > 0$）或 $+\hat y$（$d_y < 0$）；$s_a = 0$ 且起点在层内时它表示起点在 $S$ 内，$s^* = 0$。(b) 对每条轮廓边 $(q_i, q_{i+1})$：求二维射线 $(o_x, o_z) + s(d_x, d_z)$ 与线段的交点参数 $s$（平行时无），若 $s_a \le s \le s_b$ 则候选 $s$，法线为该侧壁的外法线 $\mathrm{sgn}(A)\,\mathrm{unit}(e_z, 0, -e_x)$，$e = q_{i+1} - q_i$。$s^* = $ 全部候选的最小值（并列时 (a) 优先，再按边序号），没有候选即未命中。

**局部包围盒** $[\min x_i, \max x_i] \times [0, h] \times [\min z_i, \max z_i]$。原点是轮廓坐标的原点，一般不在盒的底面中心（2.1.2）。

**紧包围盒**：$E = \{ (x_i, 0, z_i), (x_i, h, z_i) \}$。

**表面采样**：记耳切（2.1.6.2）给出的三角形 $(a_k, b_k, c_k)$（正向，$A > 0$ 的顺序）。面片顺序：底面 $\mathrm{Tri}(\bar a_k, \bar b_k, \bar c_k)$（$\bar q = (q_u, 0, q_v)$），顶面 $\mathrm{Tri}(\tilde a_k, \tilde c_k, \tilde b_k)$（$\tilde q = (q_u, h, q_v)$），侧壁按边序 $\mathrm{Rect}(\bar q_i; \bar q_{i+1} - \bar q_i, h\hat y)$。

**体积** $V = \lvert A \rvert\, h$。

**凸块**：轮廓为凸（2.1.6.2）时一个棱柱：顶点 $\bar q_0..\bar q_{m-1}, \tilde q_0..\tilde q_{m-1}$，面片底 $(\bar q)$ 按正向顺序、顶 $(\tilde q)$ 按反向顺序、侧 $(\bar q_i, \tilde q_i, \tilde q_{i+1}, \bar q_{i+1})$（轮廓先整理成正向；正向轮廓的边 $e$ 的外法线是 $(e_z, 0, -e_x)$，这个顶点序使面片从外面看逆时针）。轮廓为凹时按耳切的每个三角形各一个三棱柱（6 顶点、5 面片，构造同上）。`approx=false`。

**三角化**：底面 $(\bar a_k, \bar b_k, \bar c_k)$（正向三角形在 $(x, z)$ 里的法线是 $-\hat y$），顶面 $(\tilde a_k, \tilde c_k, \tilde b_k)$，侧壁四边形 $(\bar q_i, \tilde q_i, \tilde q_{i+1}, \bar q_{i+1})$（轮廓按正向，各分两个三角形）。

**示例**
```s3d
plate deck points=[(0,0); (6,0); (6,4); (3,6); (0,4)] h=0.3 on=ground
plate tile points=[(0,0); (2,0); (2,2); (0,2)] h=0.1 at=0,0,10 pin=origin
```

### 2.2.6 hull

凸包：一组三维点的凸包。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `points` | 三元组列表 $(x, y, z)$，只接受坐标 | 必填 | 至少 4 个点；不全共面 | 少于 4 点或分量不是数：报错"`<路径>`：hull 的 points 写成 [(x,y,z); (x,y,z); …]，至少 4 个点"；共面：报错"`<路径>`：hull 的点都在一个平面上（没有体积）" |

**共面判定**（确定性）：$e$ 是三个坐标跨度的最大值，$\varepsilon_{\text{hull}} = 10^{-9}\max(e, 10^{-9})$。取 $p_a$ = $x$ 最小的点、$p_b$ = $x$ 最大的点（并列取输入序号小的）；$\lVert p_b - p_a \rVert < \varepsilon_{\text{hull}}$ 为共面。取 $p_c$ = 离直线 $p_a p_b$ 最远的点（并列取序号小的），距离 $< \varepsilon_{\text{hull}}$ 为共面。取 $p_d$ = 离平面 $p_a p_b p_c$ 最远的点，距离 $< \varepsilon_{\text{hull}}$ 为共面。

**几何** $S = \mathrm{conv}\{P_i\}$。它的面片集 $F$ 是 2.1.6.1 意义下的极大共面面片：到面片平面距离 $\le \varepsilon_{\text{hull}}$ 的点视为在该平面上，相邻共面的三角形合并；每个面片的顶点序从面片里输入序号最小的点开始、从外面看逆时针。不是极点的输入点（在凸包内部或面上、棱上）不是顶点。实现可以用任何凸包算法，但面片集与顶点序必须符合本定义。

**SDF**：2.1.6.1 的半空间 SDF（内部精确，外部为下界）。

**射线**：2.1.6.1 的裁剪。

**局部包围盒**：全部输入点的轴向最小 / 最大。原点是点坐标的原点，可以在盒外（2.1.2）。

**紧包围盒**：$E$ = 全部输入点（极值总在凸包顶点上，包含内部点不改变结果）。

**表面采样**：按面片顺序，每个面片按 2.1.9 的扇形规则分成的三角形各一个 $\mathrm{Tri}$。

**体积**：2.1.6.1 的公式。

**凸块**：$S$ 自身（顶点、面片），`approx=false`。

**三角化**：每个面片的扇形三角化，按面片顺序。

**示例**
```s3d
hull rock points=[(0,0,0); (3,0,0); (0,0,3); (3,0,3); (1.5,2,1.5)] on=ground
hull pyramid points=[(0,-1,0); (2,-1,0); (0,-1,2); (0,1,0)] at=10,0,30 pin=origin
```

### 2.2.7 revolve

旋转体：$(r, y)$ 平面里的一条轮廓线绕局部 $y$ 轴旋转。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `profile` | 二元组列表 $(r, y)$ | 必填 | 至少 2 个点 | 报错"`<路径>`：revolve 的 profile 写成 [(半径,高); (半径,高); …]" |
| `angle` | 数 | 360 | $0 < \text{angle} \le 360$ | 报错"`<路径>` 的 angle 要大于 0、不超过 360" |
| `smooth` | 布尔 | `false` | | 报错 |

**轮廓的处理**（按此顺序，结果记作 $Q = (r_0, y_0), \dots, (r_{n-1}, y_{n-1})$）：

1. 每个 $r_i \leftarrow \lvert r_i \rvert$。
2. `smooth=true`：对序列做开放的均匀 Catmull-Rom（2.1.6.5），每段 10 个样本，得 $10(n-1)+1$ 个点。
3. 每个 $r_i \leftarrow \max(r_i, 0)$（平滑的过冲可以越过轴；超出控制点半径的过冲保留）。
4. 自动封口：若 $r_0 > \varepsilon_{\text{cap}}$ 在最前插入 $(0, y_0)$；若末点 $r > \varepsilon_{\text{cap}}$ 在最后追加 $(0, y_{\text{末}})$。
5. 规整（2.1.6.2）。
6. 闭合多边形 $\bar Q$ = $Q$ 加上从末点回到首点的**闭合边**。$\bar Q$ 里两端 $r = 0$ 的边叫**轴边**（闭合边总是轴边；轮廓自带的轴上相邻点也构成轴边）。$\bar Q$ 必须 $\lvert 2A \rvert \ge \varepsilon_{\text{area}}$（否则报错"`<路径>`：revolve 的轮廓面积是 0"）且简单（否则报错"`<路径>`：revolve 的轮廓自交"）。

记 $\Omega$ 为 $\bar Q$ 围成的闭区域，$R_{\max} = \max_i r_i$，$y_{\min} = \min_i y_i$，$y_{\max} = \max_i y_i$；$\mathrm{dir}(\theta) = (\cos\theta, 0, -\sin\theta)$（轴系 $c_0 = 0$，$a = \hat y$，$u_0 = \hat x$：$\theta = 0$ 指 $+x$，俯视逆时针，与 arc、`side[θ,v]` 一致）。

**几何** $S = \{ \rho\,\mathrm{dir}(\theta) + y\,\hat y : (\rho, y) \in \Omega,\ 0 \le \theta \le \text{angle} \}$。`angle` $= 360$ 为整圈；小于 360 时是从 $\theta = 0$ 到 $\theta = $ angle 的扇体，两个切面是实体表面的一部分（实体总是闭合的）。

**SDF**：$\rho = \sqrt{x^2 + z^2}$，$f_{\text{rev}}(p) = f_2(\rho, y)$，$f_2$ 是 $\bar Q$ 的二维 SDF，**排除轴边**（轴边只参与内外判定）；整圈时精确。`angle` $< 360$ 时 $f(p) = \max(f_{\text{rev}}(p), f_{\text{sec}}(x, z))$（下界），扇区 SDF：$\theta_p = \mathrm{atan2}(-z, x) \bmod 360 \in [0, 360)$，$u_0 = (1, 0)$、$u_1 = (\cos\text{angle}, -\sin\text{angle})$，$d_j = \lVert (x, z) - \max((x, z)\cdot u_j, 0)\,u_j \rVert$，$f_{\text{sec}} = -\min(d_0, d_1)$ 当 $\theta_p \le $ angle，否则 $+\min(d_0, d_1)$。

**射线**：距离场步进（2.1.6.6）。

**局部包围盒** $[-R_{\max}, R_{\max}] \times [y_{\min}, y_{\max}] \times [-R_{\max}, R_{\max}]$，**与 `angle` 无关**（扇体的盒按整圈算，使轴总在盒的中心；紧包围盒按真实形状）。原点在轴上 $y = 0$，$y_{\min}$ 可以不是 0。

**紧包围盒**：整圈：每个 $(r_i, y_i)$ 一个圆（圆心 $(0, y_i, 0)$、半径 $r_i$、轴 $\hat y$）；`angle` $< 360$：每个 $(r_i, y_i)$ 一段弧 $c = (0, y_i, 0)$、$u = \hat x$、$v = -\hat z$、$\theta \in [0, \text{angle}]$（2.1.6.4）。

**表面采样**：对 $\bar Q$ 的每条非轴边 $(r_i, y_i) \to (r_{i+1}, y_{i+1})$（按边序）一个母线面 $\mathrm{Lat}(0, \hat y, \hat x; r_i, y_i, r_{i+1}, y_{i+1}; 0, \text{angle})$；`angle` $< 360$ 时再加两个切面：$\bar Q$ 的耳切三角形映射到 $\theta = 0$（$(\rho, y) \mapsto (\rho, y, 0)$）与 $\theta = $ angle（$(\rho, y) \mapsto \rho\,\mathrm{dir}(\text{angle}) + y\hat y$）。

**体积** $V = 2\pi \lvert M \rvert \cdot \text{angle}/360$，$M = \tfrac16 \sum_{\text{边}} (r_i + r_{i+1})(r_i y_{i+1} - r_{i+1} y_i)$，对 $\bar Q$ 的全部边求和（轴边贡献为 0）。

**凸块**：一个凸多面体，$R_{\max}$ 为半径、$[y_{\min}, y_{\max}]$ 为高的 12 边内接棱柱（2.2.3 的构造取 $c_0 = (0, y_{\min}, 0)$、$a = \hat y$、$u_0 = \hat x$、$r_1 = r_2 = R_{\max}$、$h = y_{\max} - y_{\min}$），`angle` 不影响它；`approx=true`（外接形状）。

**三角化**：先把 $\bar Q$ 整理成正向（$A > 0$，2.1.6.2）。周向段数 $n$ 按 2.1.9（整圈 48），$\theta_j = \text{angle} \cdot j / n$，$j = 0..n$（整圈时 $j = n$ 与 $j = 0$ 重合，仍按 $j = n$ 计算）。顶点 $v_{i,j} = r_i\,\mathrm{dir}(\theta_j) + y_i\hat y$，$i$ 遍历 $\bar Q$ 的顶点。对 $\bar Q$ 的每条非轴边 $(i, i+1)$ 与每个 $j = 0..n-1$：四边形 $(v_{i,j}, v_{i,j+1}, v_{i+1,j+1}, v_{i+1,j})$ 各分两个三角形；一端 $r = 0$ 的边退化为三角形（省略面积为零的三角形）。`angle` $< 360$ 时追加切面：$\theta = 0$ 的面用耳切三角形 $(a, b, c) \mapsto (a_\rho, a_y, 0), \dots$ 原序（法线 $+\hat z$，在扇体外侧），$\theta = $ angle 的面用反序映射到 $\rho\,\mathrm{dir}(\text{angle}) + y\hat y$。

**示例**
```s3d
revolve tower profile=[(0.8,0); (0.7,9); (3.5,11); (4.2,14); (3,16.5); (0.3,17.2)] smooth=true on=ground
revolve basin profile=[(0,0); (5,0); (5.2,0.9); (4.7,0.9); (4.6,0.35); (0,0.35)]
revolve half profile=[(0,0); (2,0); (2,3); (0,3)] angle=180
```

### 2.2.8 span（beam / rod / steps）

span 连接**父坐标系**里的两点 $A$、$B$，有三种形态：梁（beam，矩形截面）、杆（rod，圆截面）、台阶（steps）。它的局部系由两点决定，语句上的定位与朝向属性不适用（第 1 章的属性表不给 span 这些属性；写了报错）。`pin` 在 span 上是几何参数（见下），不是定位点。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 / 说明 | 非法值 |
|---|---|---|---|---|
| `from` | 点或锚点（第 3 章在父框架里求值） | 必填 | | 报错"`<路径>` 的 from 需要点坐标或锚点" |
| `to` | 点或锚点 | 与 `drop` 二选一 | | 同上；`to`、`drop` 都写或都不写：报错"`<路径>`：to 与 drop 要写一个" |
| `drop` | 参照物，可带锚点 | | 见"drop" | |
| `dir` | 方向（轴名、方向词、向量或锚点的法线 / 切线，第 3 章） | $(0, -1, 0)$ | 只能与 `drop` 一起写 | 没有 `drop` 时报错"`<路径>`：dir 只能和 drop 一起写" |
| `section` | 二元组 $(w, t)$ | 梁 $(1, 0.3)$；台阶 $(1.2, 0.2)$ | $w, t > 0$ | 报错"`<路径>` 的 section 需要两个数：宽,厚" |
| `pin` | 枚举 `bottom` / `top` / `center` | `center` | 只对梁有效 | 其他值报错"`<路径>` 的 pin 只能是 bottom / top / center" |
| `r` | 正数 | 无（写了即为杆） | | 报错 |
| `steps` | 整数 $\ge 1$ | 无（写了即为台阶） | | 报错"`<路径>` 的 steps 需要正整数" |

互斥（都报错）：`steps` 与 `r`（"`<路径>`：steps 与 r 不能同时写"）；`r` 与 `section` 或 `pin`（"`<路径>`：圆杆没有 section / pin"）；`steps` 与 `pin`（"`<路径>`：台阶没有 pin"）。形态：写了 `steps` 为台阶，否则写了 `r` 为杆，否则为梁。

**drop**：$B$ 由 $A$ 沿 `dir` 与目标的一个面所在的**平面**求交得到。目标锚点缺省为 `top`；第 3 章给出该锚点在父框架里的点 $q$ 与法线 $n$（锚点法线为零时取目标的局部 $+y$ 在父框架里的方向）。$\delta = \mathrm{dir} \cdot n$；$\lvert \delta \rvert < \varepsilon_{\text{len}}$ 报错"`<路径>`：drop 的方向与 `<目标>` 的面平行，碰不到"；$s = ((q - A)\cdot n)/\delta$；$s < 0$ 报错"`<路径>`：沿 dir 方向碰不到 `<目标>`（它在反方向）"；$B = A + s\,\mathrm{dir}$。只与无限平面求交，**不检查**落点是否在面的范围内。

**公共量**：$d = B - A$，$L = \lVert d \rVert$（$L < \varepsilon_{\text{len}}$ 报错"`<路径>`：from 与 to 重合"），$H = \sqrt{d_x^2 + d_z^2}$，$\text{mid} = \tfrac12(A + B)$。

**局部系**。梁与杆：$e_x = d / L$；$e_y = \mathrm{unit}(\hat y - (\hat y \cdot e_x) e_x)$，若 $\lVert \hat y - (\hat y \cdot e_x)e_x \rVert < \varepsilon_{\text{dir}}$（竖直）则 $e_y = \mathrm{perp}(e_x)$；$e_z = e_x \times e_y$；$R = [\,e_x\ e_y\ e_z\,]$（列），$t = \text{mid}$。$\mathrm{perp}(v) = \mathrm{unit}(v \times \hat y)$ 当 $\lvert v_y \rvert < 0.9$，否则 $\mathrm{unit}(v \times \hat x)$；于是竖直向上的 span 有 $e_y = -\hat z$、$e_z = -\hat x$。局部 $x$ 沿 $A \to B$，$y$ 尽量朝上，$z$ 水平。台阶：$R = R_y(\psi)$（第 3 章的 turn 矩阵，$R_y(\psi)\hat x = (\cos\psi, 0, -\sin\psi)$），$\psi = \mathrm{atan2}(-d_z, d_x)$，$t = \text{mid}$：只绕竖直轴转，踏面水平；$H < \varepsilon_{\text{len}}$ 报错"`<路径>`：台阶的起点与终点在同一竖直线上"。

**梁（beam）**：$d_y$（截面相对连线的偏移）由 `pin` 决定：`bottom` $\to +t/2$（底面贴着连线），`top` $\to -t/2$（顶面贴着连线），`center` $\to 0$。几何 $S = [-\tfrac L2, \tfrac L2] \times [d_y - \tfrac t2, d_y + \tfrac t2] \times [-\tfrac w2, \tfrac w2]$——局部 $y$ 是厚度方向，$z$ 是宽度方向。它是盒心 $(0, d_y, 0)$、半尺寸 $(\tfrac L2, \tfrac t2, \tfrac w2)$ 的盒子：SDF、射线、局部包围盒（$= S$）、紧包围盒（8 顶点）、采样（六个矩形）、体积 $L\,w\,t$、凸块（盒，`approx=false`）、三角化（12 个三角形）全部按 2.2.1 以该盒代入。

**杆（rod）**：圆柱，轴系 $c_0 = (-\tfrac L2, 0, 0)$、$a = \hat x$、$u_0 = \hat y$，$r_1 = r_2 = r$，高 $L$。几何、SDF、射线、紧包围盒（两个圆）、采样、体积 $\pi r^2 L$、凸块（12 边棱柱，`approx=false`）、三角化全部按 2.2.3 以该轴系代入。局部包围盒 $[-\tfrac L2, \tfrac L2] \times [-r, r] \times [-r, r]$（轴线过原点，与 cyl 不同）。

**台阶（steps）**：$n = $ `steps`。第 $i$ 级（$i = 0..n-1$）的踏面顶 $\tau_i = -\tfrac{d_y}{2} + d_y\,\tfrac{i+1}{n}$，踏步盒
$$S_i = \Big[-\tfrac H2 + \tfrac{H i}{n},\ -\tfrac H2 + \tfrac{H(i+1)}{n}\Big] \times [\tau_i - t,\ \tau_i] \times \Big[-\tfrac w2, \tfrac w2\Big],$$
$S = \bigcup_i S_i$。第一级顶面比起点高 $d_y / n$，最后一级顶面与终点等高，每级底面比顶面低 $t$（可以低于起点）。踏步只在竖直面上相邻，不重叠。
- SDF（下界）：$f = \min_i f_{S_i}$。
- 射线：各 $S_i$ 的裁剪结果取最小 $s^*$（并列取 $i$ 小的）。
- 局部包围盒：$[-\tfrac H2, \tfrac H2] \times [\min(\tau_0, \tau_{n-1}) - t,\ \max(\tau_0, \tau_{n-1})] \times [-\tfrac w2, \tfrac w2]$。
- 紧包围盒：全部踏步的顶点。
- 采样：按 $i$ 顺序每级六个矩形（相邻踏步的公共竖直面也被采样，不剔除）。
- 体积 $V = H\,t\,w$。
- 凸块：每级一个盒，`approx=false`。
- 三角化：每级 12 个三角形，按 $i$ 顺序。

**示例**
```s3d
span beam from=pillarL.top-right to=pillarR.top-left section=1,0.6 pin=top
span ramp from=pillarL.bottom-right+(0,0,2) to=pillarR.top-left+(0,0,2) section=1.5,0.2 pin=bottom
span stairs from=table.bottom-left+(-6,0,0) to=table.top-left steps=8 section=2,0.2
span col from=(0,-0.7,0) drop=ground r=0.22
```

### 2.2.9 sweep

扫掠：一个二维截面沿一条路径移动扫出的实体。它是三角化路径实体：点集由本节构造的闭合三角网格定义，没有 SDF。sweep 的局部系就是父坐标系（$R = I$、$t = 0$），语句上的定位与朝向属性不适用（第 1 章不给它这些属性）；`pin` 在 sweep 上是截面参数。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 / 说明 | 非法值 |
|---|---|---|---|---|
| `path` | 曲线引用，或三元组列表（每项是坐标或锚点，在父框架里求值） | 必填 | 列表至少 2 个点 | 既不是曲线也不是列表：报错"`<路径>` 的 path 需要曲线或点列表"；少于 2 点：报错"`<路径>` 的 path 至少需要两个点，如 path=[A.top; B.top]" |
| `profile` | 枚举 `circle` / `rect`，或二元组列表 $(u, v)$ | 写了 `section` 为 `rect`，否则 `circle` | 列表至少 3 个点、面积不为零、不自交 | 少于 3 点：报错"`<路径>` 的 profile 多边形写成 [(u,v); (u,v); …]，至少 3 个点"；其他值：报错"`<路径>` 的 profile 只能是 circle / rect / 多边形列表"；退化：报错"`<路径>`：截面面积是 0"；自交：报错"`<路径>`：截面自交" |
| `r` | 正数 | 0.3 | 只对 `circle` | 与 rect / 多边形同写：报错"`<路径>`：r 只能和圆截面一起写" |
| `section` | 二元组 $(w, t)$，$> 0$ | $(1, 0.2)$ | 只对 `rect` | 与 circle / 多边形同写：报错"`<路径>`：section 与 pin 只能和 rect 截面一起写" |
| `pin` | 枚举 `bottom` / `top` / `center` | `center` | 只对 `rect` | 同上；其他值报错 |
| `smooth` | 布尔 | `false` | 只对点列表路径 | 与曲线路径同写：报错"`<路径>`：smooth 只对点列表路径有效" |

**路径点列** $P = (p_0, \dots, p_{N-1})$、闭合标志 `closed`、可选的切线列 $T_i$：

- `path` 是曲线 $C$：$P$ 是 $C$ 的采样点列（2.3.1）从 $C$ 的局部系变换到 sweep 的父框架（第 3 章的相对变换 $R_{\text{rel}}, t_{\text{rel}}$）；`closed` 随 $C$；$C$ 是 arc 或 helix 时切线 $T_i = R_{\text{rel}}\,\hat T(t_i)$（解析单位切线，2.3.2 / 2.3.3），其他曲线不给切线。
- `path` 是点列表 $c_0, \dots, c_{n-1}$（父框架）：`closed=false`，不给切线。`smooth=true`：$P$ = 开放均匀 Catmull-Rom、每段 10 个样本（2.1.6.5）。否则 $P$ = **倒圆**：倒圆半径 $\rho_c$ = 圆截面 $\max(3r, 0.6)$、rect 截面 $\max(w, 1)$、多边形截面 $1$（场景单位）；对每个内点 $b = c_i$（$i = 1..n-2$），邻点 $a = c_{i-1}$、$c = c_{i+1}$（原始控制点），$\delta = \min(\rho_c,\ 0.45\lVert b - a \rVert,\ 0.45\lVert c - b \rVert)$；$\delta < \varepsilon_{\text{dir}}$ 时保留 $b$；否则用二次 Bézier $(b + \delta\,\mathrm{unit}(a - b),\ b,\ b + \delta\,\mathrm{unit}(c - b))$ 在 $t = j/8$（$j = 0..8$）的 9 个点替换 $b$。$P$ 以 $c_0$ 开头、$c_{n-1}$ 结尾。
- $P$ 里相邻点重合（距离 $< \varepsilon_{\text{len}}$）报错"`<路径>`：path 有重合的相邻点"。

**框架** $(p_i, \text{side}_i, \text{up}_i)$ 按 2.3.7 由 $P$、`closed`、$T_i$ 算出。

**截面** $\Pi = (u_k, v_k)$，$k = 0..m-1$，在 (side, up) 平面里，正向（$A > 0$）：
- `circle`：$m = n_{\text{circ}} = 20$，$(u_k, v_k) = (r\cos 18k,\ r\sin 18k)$。
- `rect`：$\delta_v$ = `pin` 为 `bottom` $\to +t/2$、`top` $\to -t/2$、`center` $\to 0$；$\Pi = \big((\tfrac w2, -\tfrac t2 + \delta_v), (\tfrac w2, \tfrac t2 + \delta_v), (-\tfrac w2, \tfrac t2 + \delta_v), (-\tfrac w2, -\tfrac t2 + \delta_v)\big)$——$u$ 沿 side（圆弧路径上指向外侧），$v$ 沿 up；`pin=bottom` 让截面的底边贴着路径。
- 多边形：规整后若 $A < 0$ 则反转。

**环点** $\rho_{i,k} = p_i + u_k\,\text{side}_i + v_k\,\text{up}_i$。

**几何**：$S$ 是闭合三角网格 $M$ 围成的实体（2.1.6.3），$M$ 由下面的三角形按此顺序构成（段数 $N_s = N - 1$，闭合时 $N_s = N$，下标 $i + 1$ 模 $N$）：
1. 侧面：对 $i = 0..N_s - 1$、$k = 0..m-1$（$k + 1$ 模 $m$）：四边形 $(\rho_{i,k}, \rho_{i+1,k}, \rho_{i+1,k+1}, \rho_{i,k+1})$ 分成 $(\rho_{i,k}, \rho_{i+1,k}, \rho_{i+1,k+1})$ 与 $(\rho_{i,k}, \rho_{i+1,k+1}, \rho_{i,k+1})$。
2. 开放路径的封口：$\Pi$ 的耳切三角形 $(a, b, c)$（2.1.6.2），起点面用 $(\rho_0(a), \rho_0(b), \rho_0(c))$，终点面用 $(\rho_{N-1}(a), \rho_{N-1}(c), \rho_{N-1}(b))$，$\rho_i(q) = p_i + q_u\,\text{side}_i + q_v\,\text{up}_i$。

（side、up、$T$ 满足 $\text{side} \times \text{up} = -T$，所以正向截面的侧面法线朝外、起点面法线为 $-T_0$。）路径急转或自交时 $M$ 可以自交，点集仍按 2.1.6.3 的绕数定义。

**SDF**：无（三角化路径实体）。

**射线**：2.1.6.3。

**局部包围盒**：全部环点的轴向最小 / 最大（父框架）。

**紧包围盒**：$E$ = 全部环点。

**表面采样**：面片 = $M$ 的三角形，按上面的顺序。

**体积**：$M$ 的散度体积（2.1.6.3）。

**凸块**：每段一个凸多面体，$i = 0..N_s - 1$：顶点取环 $i$ 与环 $i+1$ 的点；$m > 8$ 时每环只取序号 $k_j = \lfloor j\,m / 8 \rfloor$（$j = 0..7$）的 8 个点（20 点圆截面取 $0°, 36°, 90°, 126°, 180°, 216°, 270°, 306°$），记取出的序号为 $k_0 < \dots < k_{m'-1}$。截面为凸时面片：起点环 $(\rho_{i,k_{m'-1}}, \dots, \rho_{i,k_0})$、终点环 $(\rho_{i+1,k_0}, \dots, \rho_{i+1,k_{m'-1}})$、侧面 $(\rho_{i,k_j}, \rho_{i+1,k_j}, \rho_{i+1,k_{j+1}}, \rho_{i,k_{j+1}})$，`approx=false`。截面为凹时该段的凸块是这 $2m'$ 个点的凸包（2.2.6 的构造），`approx=true`。

**三角化**：$M$ 自身。

**示例**
```s3d
arc loop r=40 angle=-60,240
sweep lane path=loop profile=rect section=8,0.1 pin=bottom
sweep rail path=railline r=0.08
sweep railL path=track profile=[(-0.75,0); (-0.55,0); (-0.55,0.18); (-0.75,0.18)]
sweep duct path=[(0,0,0); (10,0,0); (10,0,8)] section=2,0.5
```

### 2.2.10 pipe

圆管：截面固定为圆的扫掠。

**参数表**

| 属性 | 类型 | 默认值 | 说明 | 非法值 |
|---|---|---|---|---|
| `path` | 同 sweep | 必填 | | 同 sweep |
| `r` | 正数 | 0.3 | 圆截面半径 | 报错 |
| `smooth` | 布尔 | `false` | 只对点列表路径 | 同 sweep |

`profile`、`section`、`pin` 不是 pipe 的属性（第 1 章）。几何、SDF（无）、射线、局部包围盒、紧包围盒、采样、体积、凸块、三角化全部与 `sweep … profile=circle r=<r>` 相同（2.2.9），含倒圆半径 $\max(3r, 0.6)$ 与 20 点截面。

**示例**
```s3d
pipe feed path=[(0,0,70); (10,0,70); (10,0,80)] r=0.35
pipe coil path=railline r=0.05
```

### 2.2.11 mesh

网格引用：引用外部文件里的一个三角网格，并声明一个外包络盒。它是三角化路径实体。

**参数表**

| 属性 | 类型 | 默认值 | 说明 | 非法值 |
|---|---|---|---|---|
| `src` | 字符串 `"文件#节点"` | 必填 | `#` 前是文件路径（相对本文件所在目录），后是该文件里的节点名 | 空或不是字符串：报错"`<路径>`：mesh 需要 src="文件.glb#节点名"" |
| `size` | 三维向量 $(w, h, d)$，$> 0$ | 必填 | 外包络 | 缺省：报错"`<路径>` 的 size 需要三维向量 x,y,z" |
| `pivot` | 三维向量 | $(0, 0, 0)$ | 源网格里与本实体原点重合的点 | 报错 |

**几何**：两个点集。
- **外包络** $E = [-\tfrac w2, \tfrac w2] \times [0, h] \times [-\tfrac d2, \tfrac d2]$——语言层面的几何：局部包围盒、紧包围盒、方向词锚点、凸块、容器包围盒、体积（源不可用时）都按它。
- **源网格** $M_{\text{src}}$：文件里名为"节点"的节点（含其子节点）在该文件场景坐标系里的全部三角形（文件自身的节点变换已施加；坐标按场景单位原样使用，不换算），按文件里的三角形顺序；局部网格 $M = \{ v - \text{pivot} : v \in M_{\text{src}} \}$。渲染、评分、导出、布尔（2.5.3）、`top[u,w]`（2.4.1）按 $M$。文件或节点不可用时（编译不要求读文件；工具按需读），这些运算用 $E$ 的盒三角化（2.2.1，朝外）代替并标记 `placeholder`。
- 两者不一致时的规则见 2.7 节。

**SDF**：无。

**射线**：$M$（2.1.6.3），或占位盒。

**局部包围盒** $B = E$。

**紧包围盒**：$E$ 的 8 个角（按外包络算，标为近似）。

**表面采样**：$M$ 的三角形按文件顺序；或占位盒的六个矩形。

**体积**：$M$ 闭合时按 2.1.6.3；不闭合或不可用时 $V = w\,h\,d$ 并标为近似。

**凸块**：$E$ 作为盒（2.2.1 的构造），`approx=true`。

**三角化**：$M$；或占位盒。

**示例**
```s3d
mesh crate src="basics.L0.glb#crate" size=5,4,5 pivot=11,0,4.8 on=floor[0.55,1,0.4]
mesh statue src="props.glb#statue_01" size=2,3.5,2 on=plinth
```

## 2.3 曲线

### 2.3.1 共同机制：采样点列、参数化、取点

曲线不是实体：没有体积、SDF、凸块，不参与空间事实，不按实体渲染（输出为折线）。它可以被引用（sweep / pipe 的 `path=`、`repeat along=`、镜像面、锚点）、取点（`C.start / C.mid / C.end / C[t]`）、当参照物（2.3.8）。

每种曲线在**自身局部系**里定义三样东西：

1. **采样点列** $P = (p_0, \dots, p_{N-1})$，$N \ge 2$，与**闭合标志** `closed`。采样规则是语言定义的一部分：扫掠的环、`repeat along` 的实例、曲线的包围盒都由这组点决定，两份实现必须得到同一组点。
2. **解析式**（只有 arc、helix 有）：$C_a(t)$ 与单位切线 $\hat T(t)$，$t \in [0, 1]$；采样点 $p_i = C_a(t_i)$。
3. **参数化与取点**。折线序列 $\text{seq} = P$（闭合时再追加 $p_0$）；弦长 $\ell_0 = 0$，$\ell_{k+1} = \ell_k + \lVert \text{seq}_{k+1} - \text{seq}_k \rVert$（按 $k$ 顺序累加），总长 $\Lambda = \ell_{\text{末}}$；$\Lambda < \varepsilon_{\text{len}}$ 报错"`<路径>`：曲线的长度是 0"。参数 $t$ 先规整：闭合曲线 $t \leftarrow t - \lfloor t \rfloor$，开放曲线 $t \leftarrow \mathrm{clamp}(t, 0, 1)$。取点 $C[t]$：
   - arc、helix：$C_a(t)$，切线 $\hat T(t)$（参数与角度成正比，等于按弧长）。
   - 其他曲线：$\ell^* = t\,\Lambda$；$k = \max\{ k : \ell_k \le \ell^* \}$，并限制 $k \le$ 段数 $- 1$；$\lambda = (\ell^* - \ell_k)/(\ell_{k+1} - \ell_k)$（段长为 0 时 $\lambda = 0$）；点 $= \text{seq}_k + \lambda(\text{seq}_{k+1} - \text{seq}_k)$；切线 $= \mathrm{unit}(\text{seq}_{k+1} - \text{seq}_k)$（段长为 0 时取后面第一个非零段的方向，没有则前面的）。"按弧长"指按**采样折线的弦长**，不是真实曲线的弧长。
   - `C.start` $= C[0]$，`C.mid` $= C[0.5]$，`C.end` $= C[1]$（闭合曲线的 end 等于 start）。
   
   取点锚点的法线为零向量，另带切线；第 3 章定义它们在定位与 `dir=` 里的用法。

### 2.3.2 arc

圆弧（可带升高，成为一圈螺旋段）。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `r` | 正数 | 5 | | 报错 |
| `angle` | 数 $a$（表示 $0, a$）或二元组 $a_0, a_1$（度） | $0, 360$ | $a_1 \ne a_0$ | 报错"`<路径>` 的 angle 写成 起,止（度）"；$a_1 = a_0$：报错"`<路径>`：弧的起止角相同" |
| `rise` | 数 | 0 | | 报错 |
| `plane` | 枚举 `xz` / `xy` / `zy` | `xz` | | 报错"`<路径>` 的 plane 只能是 xz / xy / zy" |

**定义**：$\Delta = a_1 - a_0$，$\theta(t) = a_0 + \Delta\,t$，$\Delta' = \Delta\,\pi/180$。

| `plane` | $C_a(t)$ | $\tfrac{d}{dt}C_a$（$\hat T = \mathrm{unit}$ 之） | 含义 |
|---|---|---|---|
| `xz` | $(r\cos\theta,\ \text{rise}\cdot t,\ -r\sin\theta)$ | $(-r\Delta'\sin\theta,\ \text{rise},\ -r\Delta'\cos\theta)$ | $0°$ 指 $+x$，俯视逆时针（$+x \to -z$）；rise 沿 $+y$ |
| `xy` | $(r\cos\theta,\ r\sin\theta,\ \text{rise}\cdot t)$ | $(-r\Delta'\sin\theta,\ r\Delta'\cos\theta,\ \text{rise})$ | $0°$ 指 $+x$，$90°$ 指 $+y$；rise 沿 $+z$ |
| `zy` | $(\text{rise}\cdot t,\ r\sin\theta,\ r\cos\theta)$ | $(\text{rise},\ r\Delta'\cos\theta,\ -r\Delta'\sin\theta)$ | $0°$ 指 $+z$，$90°$ 指 $+y$；rise 沿 $+x$ |

**采样**：段数 $n_s = \max(8, \lceil \lvert\Delta\rvert / 4 \rceil)$（每段不超过 $4°$）；`closed` $\Leftrightarrow \lvert\Delta\rvert \ge 360 - \varepsilon_{\text{closed}}$ 且 $\text{rise} = 0$；$t_i = i / n_s$，$i = 0..n_s$，闭合时去掉 $i = n_s$（$N = n_s$），否则 $N = n_s + 1$。默认的整圆有 90 个点；`angle=0,90` 有 24 个点；`angle=-60,240` 有 76 个点。

**原点**：圆心（在 $y = 0$ 或起点高度所在的平面上）。局部包围盒见 2.3.8。

**示例**
```s3d
arc loop r=40 angle=-60,240
arc archline r=8 angle=0,180 plane=xy at=canal.top pin=origin
arc spiralseg r=10 angle=0,270 rise=5
```

### 2.3.3 helix

螺旋线：绕局部 $y$ 轴、沿 $+y$ 上升。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `r` | 正数 | 5 | | 报错 |
| `pitch` | 数 | 3 | $\ne 0$（每圈升高；负值下降） | $= 0$：报错"`<路径>`：helix 的 pitch 不能是 0（平的圆用 arc）" |
| `turns` | 数 | 1 | $\ne 0$（负值为顺时针） | $= 0$：报错"`<路径>`：helix 的 turns 不能是 0" |

**定义**：与 `plane=xz` 的 arc 相同，取 $a_0 = 0$、$a_1 = 360\cdot\text{turns}$、$\text{rise} = \text{pitch}\cdot\lvert\text{turns}\rvert$：$C_a(t) = (r\cos\theta,\ \text{rise}\cdot t,\ -r\sin\theta)$，$\theta = 360\,\text{turns}\,t$。起点 $(r, 0, 0)$；`turns` 为正时俯视逆时针。永不闭合。采样段数 $n_s = \max(8, \lceil 360\lvert\text{turns}\rvert / 4 \rceil)$，$N = n_s + 1$（`turns=3` 有 271 个点，`turns=1` 有 91 个）。原点是圆心（起点高度）。

**示例**
```s3d
helix rampline r=7 pitch=4.5 turns=3
helix railline r=9.4 pitch=4.5 turns=3 at=0,1,0 pin=origin
```

### 2.3.4 line

折线。line、bezier、spline 的点都在**父坐标系**里给出，它们的局部系就是父系（$R = I$、$t = 0$），语句上没有定位与朝向属性（第 1 章）。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `path` | 三元组列表，每项是坐标或锚点（锚点在父框架里求值，缺省取参照物的 center，第 3 章） | 必填 | 至少 2 个点 | 报错"`<路径>` 的 path 至少需要两个点，如 path=[A.top; B.top]" |

**定义**：$P$ = `path` 的点，顺序不变；`closed=false`。

**示例**
```s3d
line guide path=[(0,0,60); (10,0,60); (10,0,70)]
line link path=[towerA.top; towerB.top]
```

### 2.3.5 bezier

Bézier 曲线：`path` 是控制点（至少 2 个，不必在线上），参数表与 line 相同。

**定义**：$n$ 个控制点的 $n - 1$ 次 Bézier（2.1.6.5），$P$ 是 $t = j/64$（$j = 0..64$）的 65 个点；`closed=false`。

**示例**
```s3d
bezier feedline path=[tank[0,-0.7,-1]; tank[0,-0.7,-1]+(0,0,-22); pad.top-front+(-4,0.4,14); pad.top-front+(-4,0.4,0)]
```

### 2.3.6 spline

经过各控制点的样条（均匀 Catmull-Rom）。

**参数表**

| 属性 | 类型 | 默认值 | 取值范围 | 非法值 |
|---|---|---|---|---|
| `path` | 同 line | 必填 | 开放至少 2 个点，闭合至少 3 个点 | 同 line；闭合且少于 3 点：报错"`<路径>`：闭合样条至少需要 3 个点" |
| `closed` | 布尔 | `false` | | 报错 |

**定义**：2.1.6.5 的均匀 Catmull-Rom，每段 12 个样本。开放：$(n-1)\cdot 12 + 1$ 个点（$n = 2$ 时就是两个控制点）；闭合：$12n$ 个点，`closed=true`。

**示例**
```s3d
spline track closed=true path=[(0,6,0); (14,10,6); (22,16,0); (18,8,-12); (6,3,-16); (-10,12,-12); (-18,18,0); (-12,7,10); (-2,4,8)]
spline roadline path=[hangar.body.bottom-front+(0,0,10); (-28,0,12); (-4,0,9); pad.bottom-left+(-6,0,0)]
```

### 2.3.7 扫掠框架

给定点列 $P = (p_0, \dots, p_{N-1})$、`closed` 与可选的切线列 $T_i$（arc / helix 的解析切线），每个点上的框架 $(p_i, \text{side}_i, \text{up}_i)$ 定义如下（sweep、pipe 用它；`repeat along` 的实例朝向由第 3 章用同一切线定义）。

1. **切线** $T_i$：已给出时用给出的；否则由相邻点差分：开放曲线 $T_0 = \mathrm{unit}(p_1 - p_0)$、$T_{N-1} = \mathrm{unit}(p_{N-1} - p_{N-2})$、其余 $T_i = \mathrm{unit}(p_{i+1} - p_{i-1})$；闭合曲线对所有 $i$ 取 $\mathrm{unit}(p_{(i+1) \bmod N} - p_{(i-1) \bmod N})$。被单位化的向量长度 $< \varepsilon_{\text{len}}$ 时报错"`<路径>`：路径在第 i 个点退化"。
2. **side**：$\sigma_i = T_i \times \hat y = (-T_{i,z},\ 0,\ T_{i,x})$。若 $\lVert \sigma_i \rVert < \varepsilon_{\text{up}}$（竖直段）则 $\text{side}_i = \text{side}_{i-1}$，其中 $\text{side}_{-1} = (0, 0, 1)$；否则 $\text{side}_i = \mathrm{unit}(\sigma_i)$。
3. **up**：$\text{up}_i = \text{side}_i \times T_i$。

性质：side 总是水平（$y$ 分量为 0），在俯视逆时针的圆弧上指向外侧；up 在非竖直段里朝上（$y$ 分量 $> 0$）；竖直向上的段 side $= (0,0,1)$、up $= (-1, 0, 0)$，竖直向下的段 up $= (1, 0, 0)$；$\text{side}_i \times \text{up}_i = -T_i$。竖直段之后的第一个非竖直段按第 2 条重新计算 side，可能与沿用的 side 反向（截面在该段里翻转 180°）；本规范不消除这种翻转。

### 2.3.8 曲线的包围盒、原点与作为参照物

- **局部包围盒**：采样点列 $P$ 的轴向最小 / 最大（在曲线的局部系里）。它是采样折线的盒，不是解析曲线的盒：`arc r=8 angle=0,180 plane=xy` 的盒顶是 $8\sin 88° = 7.9951$ 而不是 8（46 个点，相邻点相差 $4°$），放在它上面的物体落在这个高度。
- **原点与定位**：arc、helix 有自己的局部系，像实体一样由第 3 章定位（`at=`、`pin=`、朝向属性都可用）；原点是圆心，缺省定位点是局部包围盒的底面中心（`at=P` 把弧的包围盒底面中心放到 P），`pin=origin` 才把圆心放到 P。line、bezier、spline 的局部系是父系，点就在父系里。
- **作为参照物**：方向词锚点、`B[u,v,w]` 按曲线的局部包围盒（第 3 章）；`B.top[u,w]` 取盒顶（2.4.1）；`B.start / B.mid / B.end / B[t]` 按 2.3.1。`side` / `surface` 锚点不适用。
- **容器**：曲线计入所在容器的包围盒（2.6）。
- **输出**：世界系折线 $R\,p_i + t$ 与 `closed`；没有紧包围盒。
- **镜像与副本**：镜像副本的曲线点列是原点列经反射变换的像；`turns` 为正的 helix 镜像后变成顺时针，不改参数。

## 2.4 曲面锚点

### 2.4.1 top[u,w]

曲面锚点在参照物 $B$ 的**局部系**里给出一个点 $p$ 与法线 $n$；换到别的框架、用于定位的规则见第 3 章。写法 `B.top[u,w]`、`B.side[θ,v]`、`B.surface[θ,φ]` 的方括号里必须恰好两个数，否则报错"曲面锚点写成 B.top[a,b]（两个数）"。

**`B.top[u,w]`**：盒心 $c$、半尺寸 $h_B$（2.1.2），$x = c_x + h_{B,x}\,u$，$z = c_z + h_{B,z}\,w$。$p = (x, y_{\text{top}}(x, z), z)$，$n = (0, 1, 0)$（固定朝上，斜面上也是）。$y_{\text{top}}(x, z)$ 是实体点集 $S$ 在竖直线 $\{(x, \cdot, z)\}$ 上的**最高点**，按真实表面：

| $B$ 的种类 | $y_{\text{top}}(x, z)$（"否则"= 竖直线没碰到实体） |
|---|---|
| box、span 梁 | $\lvert u \rvert \le 1 \wedge \lvert w \rvert \le 1$：$y_1$ |
| wedge | $\lvert x \rvert \le \tfrac w2 \wedge \lvert z \rvert \le \tfrac d2$：$h\,(x + \tfrac w2)/w$ |
| cyl / cone `axis=y` | $\rho = \sqrt{x^2 + z^2}$；$\rho \le r_2$：$h$；$r_2 < \rho \le r_1$：$h\,(r_1 - \rho)/(r_1 - r_2)$ |
| cyl / cone `axis=x` | $\tau = x + \tfrac h2$，$0 \le \tau \le h \wedge \lvert z \rvert \le r(\tau)$：$R + \sqrt{r(\tau)^2 - z^2}$ |
| cyl / cone `axis=z` | $\tau = z + \tfrac h2$，$0 \le \tau \le h \wedge \lvert x \rvert \le r(\tau)$：$R + \sqrt{r(\tau)^2 - x^2}$ |
| sphere | $\rho^2 = x^2 + z^2 \le r^2$：$r + \sqrt{r^2 - \rho^2}$ |
| plate | $(x, z) \in \Omega$：$h$ |
| hull | $y^+ = \min\{(d_f - n_{f,x}x - n_{f,z}z)/n_{f,y} : n_{f,y} > \varepsilon_{\text{dir}}\}$，$y^- = \max\{\cdots : n_{f,y} < -\varepsilon_{\text{dir}}\}$；$y^+ \ge y^-$ 且所有 $\lvert n_{f,y} \rvert \le \varepsilon_{\text{dir}}$ 的面满足 $n_{f,x}x + n_{f,z}z \le d_f$：$y^+$ |
| revolve | $\rho = \sqrt{x^2+z^2}$（`angle` $< 360$ 且 $\rho > 0$ 时还要求 $\theta_p \le $ angle，$\theta_p$ 见 2.2.7）；对每条非轴边：$(r_i - \rho)(r_{i+1} - \rho) \le 0$ 时给候选 $r_i = r_{i+1} ? \max(y_i, y_{i+1}) : y_i + (\rho - r_i)(y_{i+1} - y_i)/(r_{i+1} - r_i)$；有候选：最大候选 |
| span 杆 | $\lvert x \rvert \le \tfrac L2 \wedge \lvert z \rvert \le r$：$\sqrt{r^2 - z^2}$ |
| span 台阶 | 含 $(x, z)$ 的踏步存在：它们 $\tau_i$ 的最大值 |
| sweep、pipe | 竖直线与 $M$ 的三角形有交点：交点 $y$ 的最大值 |
| mesh | 源网格可用：同上；竖直线没碰到：**警告**"`<B>`.top[u,w]：竖直射线没有碰到网格，用包围盒的顶面"并取 $y_1$；源不可用：**警告**"`<B>`.top[…]：没有网格数据，用包围盒的顶面（位置是近似的）"并取 $y_1$（同一文案只警告一次） |
| 容器、曲线 | $y_1$（不检查范围） |

"否则"的情况报错"`<B>`.top[u,w]：竖直线没有碰到 `<B>` 的表面"。方向词锚点 `B.top` 与 `B[u,v,w]` 不走本节，它们按包围盒（第 3 章）：`on=wedge` 落在包围盒顶面，`on=wedge.top[u,w]` 才落在斜面上。

### 2.4.2 side[θ,v]

**`B.side[θ,v]`**：只有 cyl、cone（任意 `axis`）与 revolve 有，其他种类报错"`<B>` 没有侧面锚点 side[θ,v]（只有圆柱、圆台、旋转体有）"。$\theta$ 是度，按该实体轴系的 $\mathrm{dir}(\theta)$（2.2.3 的表、2.2.7）；$v$ 必须在 $[0, 1]$ 内，否则报错"`<B>`.side[θ,v]：v 要在 0 到 1 之间"。

- cyl / cone：$t = v\,h$，$p = c_0 + t\,a + r(t)\,\mathrm{dir}(\theta)$，$n = \mathrm{unit}\big(h\,\mathrm{dir}(\theta) - (r_2 - r_1)\,a\big)$（侧面的真实法线，含斜率）。`axis=y` 时 $\theta = 0$ 指 $+x$、$90$ 指 $-z$；`axis=x` 时 $\theta = 0$ 指 $+y$、$90$ 指 $+z$；`axis=z` 时 $\theta = 0$ 指 $+y$、$90$ 指 $-x$。
- revolve：$y = y_{\min} + v\,(y_{\max} - y_{\min})$；对每条非轴边，$(y_i - y)(y_{i+1} - y) \le 0$ 时给候选半径 $y_i = y_{i+1} ? \max(r_i, r_{i+1}) : r_i + (y - y_i)(r_{i+1} - r_i)/(y_{i+1} - y_i)$；$r(y)$ 取最大候选，并记给出它的边（并列取序号小的）$e = (\Delta r, \Delta y)$。二维法线 $n_2 = (\Delta y, -\Delta r)/\lVert e \rVert$ 当 $\Delta y > 0$，$(-\Delta y, \Delta r)/\lVert e \rVert$ 当 $\Delta y < 0$，$(1, 0)$ 当 $\Delta y = 0$（取 $\rho$ 分量非负的那个）。$p = r(y)\,\mathrm{dir}(\theta) + y\,\hat y$，$n = (n_{2,\rho}\cos\theta,\ n_{2,y},\ -n_{2,\rho}\sin\theta)$。`angle` $< 360$ 时 $\theta$ 必须满足 $0 \le \theta \bmod 360 \le $ angle，否则报错"`<B>`.side[θ,v]：θ 不在旋转体的角度范围内"。

### 2.4.3 surface[θ,φ]

**`B.surface[θ,φ]`**：只有 sphere 有，其他种类报错"`<B>` 没有球面锚点 surface[θ,φ]（只有球有）"。$\theta$（经度，$0$ 指 $+x$、$90$ 指 $-z$）与 $\phi$（纬度，$90$ 为顶）都是度：
$$d = (\cos\phi\cos\theta,\ \sin\phi,\ -\cos\phi\sin\theta),\qquad p = c + r\,d,\qquad n = d,$$
$c = (0, r, 0)$ 是球心。例：`s.surface[90,45]`（$r = 2$）$= (0,\ 2 + \sqrt2,\ -\sqrt2)$。

**示例**
```s3d
box hat size=0.5,0.2,0.5 on=dome.top[0.3,0]        // 落在球面上
box tag size=0.1,0.3,0.3 against=tank.side[90,0.5]  // 贴在圆柱侧面
box lamp size=0.2,0.2,0.2 at=orb.surface[0,45] pin=center
```

## 2.5 布尔

### 2.5.1 操作数与嵌套

`union`、`subtract`、`intersect` 是**布尔容器**：它们的子语句展开（第 4 章）后得到子节点，布尔体由其中的**操作数**按语句顺序组合而成。

- **操作数**是布尔容器的子节点里种类为实体（2.1.5 列出的 12 种）或容器（group、布尔容器、repeat、mirror / rotate 的容器式、copy、模板实例）的那些，按子节点顺序编号 $0, 1, \dots, k$。曲线不是操作数（2.5.5）。
- **容器操作数**等价于它自己的操作数（按同一规则递归取得）的 **union**：group 在布尔里就是 union；嵌套的布尔容器先按自身的运算求值，再作为一个操作数参与外层。一个没有任何实体的容器操作数**不参与**运算（被丢弃）。
- 丢弃后没有操作数：报错"`<路径>`：布尔容器没有操作数"，节点退化（零盒、不输出实体）。`subtract` 的第一个操作数被丢弃时报错"`<路径>`：subtract 的第一个操作数里没有实体"。只有一个操作数的 `subtract` 或 `intersect` 就是该操作数。
- **点集**：操作数 $i$ 的点集 $S_i$ 先按它在布尔容器局部系里的变换（第 3 章的子节点局部变换 $R_i, t_i$）换到布尔容器的局部系。$S_{\text{union}} = \bigcup_i S_i$；$S_{\text{intersect}} = \bigcap_i S_i$；$S_{\text{subtract}} = \overline{S_0 \setminus (S_1 \cup \dots \cup S_k)}$（取闭包）。
- **嵌套**：任意深度；布尔容器可以出现在 group、repeat、mirror / rotate 容器、模板里，布尔容器里可以再放这些容器。布尔容器的副本与拷贝（第 4 章）的几何与原件相同。
- **路径分类**：全部叶子操作数（递归展开到实体）都是距离场实体时，布尔体走**距离场路径**（2.5.2）；只要有一个叶子操作数是三角化路径实体（sweep、pipe、mesh），**整个最外层布尔体**（含它所有嵌套层）走**三角化路径**（2.5.3）。span 的三种形态都是距离场实体。
- 布尔容器整体是一个实体：它参与空间事实（2.5.6、第 6 章），其内部的操作数不单独参与；它可以当参照物（锚点按 2.5.4 的盒）。

### 2.5.2 距离场组合

记操作数 $i$ 的局部 SDF 为 $f_i^{\text{loc}}$，在布尔容器局部系里 $f_i(p) = f_i^{\text{loc}}(R_i^{-1}(p - t_i))$（容器操作数的 $f$ 按本节的 union 递归得到）。布尔体的 SDF：
$$f_{\text{union}} = \min_i f_i,\qquad f_{\text{intersect}} = \max_i f_i,\qquad f_{\text{subtract}} = \max(f_0,\ -f_1,\ \dots,\ -f_k).$$
组合后的符号精确（内外判断与 2.5.1 的点集一致，边界除外），大小是下界 SDF（2.1.5），因此可以继续参与外层组合与距离场步进。

**射线**：按 2.1.6.6 对 $f_{\text{bool}}$ 步进，盒用 2.5.4 的局部包围盒。若全部叶子操作数都给出了解析的参数区间集 $I$（box、wedge、cyl、cone、sphere、hull、span 梁 / 杆 / 台阶），实现**可以**改用区间运算：union 取区间并，intersect 取区间交，subtract 取 $I_0$ 减去其余区间的并，$s^*$ = 结果的最小左端点；两种方法的 $s^*$ 必须在 $\varepsilon_{\text{hit}}$ 内一致。法线：命中点落在哪个操作数的表面上就取该操作数在该点的法线，subtract 里被减操作数的法线取反；并列时取序号小的操作数。

渲染实现可以把布尔树展平成后缀序列用固定小栈求值——这是实现方式，结果必须与上面的定义一致。

### 2.5.3 三角化路径

含三角化路径实体的布尔体没有 SDF，整组按网格求值：

1. 每个叶子操作数取它的三角化（2.1.9；距离场实体用自身的三角化，mesh 用源网格或占位盒），变换到最外层布尔容器的局部系（$\det R < 0$ 时翻转绕序）。
2. 按布尔树自底向上做网格布尔，同一层从左到右折叠：$\text{acc} = ((S_0 \circ S_1) \circ S_2) \circ \cdots$，$\circ$ 是该容器的运算。
3. 结果是一个闭合三角网格 $M_{\text{bool}}$，其点集按 2.1.6.3 定义。

**语义等价要求**：$M_{\text{bool}}$ 围成的点集必须与 2.5.1 的集合运算结果一致——对任何到结果边界距离 $> \varepsilon_{\text{hit}}$ 的点，网格的内外判断等于集合运算的内外；$\partial M_{\text{bool}}$ 上每点到集合运算结果的边界距离 $\le \varepsilon_{\text{hit}}$（以最外层布尔容器的局部包围盒算 $\varepsilon_{\text{hit}}$）。操作数的内外一律按 2.1.6.3 的绕数定义，所以不闭合的源网格（mesh）也有确定的结果；网格布尔的具体算法不作规定。

三角化路径的布尔体：射线按 $M_{\text{bool}}$（2.1.6.3）；渲染不走距离场。

### 2.5.4 布尔容器的包围盒与紧包围盒

记操作数 $i$ 在布尔容器局部系里的盒 $B_i' = \mathrm{AABB}(R_i B_i + t_i)$（$B_i$ 的 8 个角变换后的轴对齐盒；容器操作数的 $B_i$ 按 2.6 节），只计入非后置的操作数（第 3 章）。

| 容器 | 局部包围盒 |
|---|---|
| `union` | $\bigcup_i B_i'$ 的轴对齐包围盒（各轴取最小 / 最大） |
| `subtract` | $B_0'$（**第一个操作数的盒**；挖掉的部分仍在盒里） |
| `intersect` | $\bigcap_i B_i'$（各轴取最大的最小值与最小的最大值）；为空时零盒 |

它们是定位、锚点、`w / h / d`、所在容器的包围盒用的盒。曲线子节点不计入（2.5.5）。

**紧包围盒**：对操作数的世界紧包围盒 $T_i$ 做同样的组合：union 取并的包围盒，subtract 取 $T_0$，intersect 取交（空则零盒）。容器操作数的 $T_i$ 取其内部实体紧包围盒的并。布尔容器是唯一有紧包围盒的容器。

### 2.5.5 布尔容器里的曲线

布尔容器里可以写曲线（例如给一个 sweep 操作数当路径）。规则：曲线不是操作数，不参与运算；不计入布尔容器的局部包围盒与紧包围盒（与 group 不同，2.6）；照常可以被引用、取点；照常进入曲线输出（世界折线）。`subtract` 的"第一个操作数"指第一个实体或容器子节点，前面的曲线不算。

### 2.5.6 布尔体的射线、采样、体积与凸块

- **射线**：2.5.2（距离场路径）或 2.5.3（三角化路径）。
- **表面采样**：距离场路径：把 $N$ 按各操作数的表面积（各自面片面积之和）用 2.1.7 的分配规则分给操作数，各操作数按 2.1.7 生成自己的点（换到布尔局部系），只保留满足 $\lvert f_{\text{bool}}(p) \rvert \le \varepsilon_{\text{hit}}$ 的点（结果点数 $\le N$，顺序按操作数、再按样本序号）。三角化路径：对 $M_{\text{bool}}$ 按 2.1.6.3 采样。
- **体积**：集合运算结果的体积。距离场路径没有闭式，按 2.5.3 的网格求 $M_{\text{bool}}$ 再用 2.1.6.3 的散度公式（这是定义；工具可以用别的方法估计，但要标为估计）。三角化路径直接用 $M_{\text{bool}}$。
- **凸块**：布尔体在空间事实里用**一个盒**代理：union 用 2.5.4 的局部包围盒，subtract 用第一个操作数的盒，intersect 用操作数盒的交；按 2.2.1 的构造，`approx=true`。挖掉的空间在事实里仍算实心（例如放在被减空腔里的物体与布尔体的关系是 inside，gap 为负），第 6 章据此判定并标 approx。
- **三角化**（导出、评分）：$M_{\text{bool}}$（2.5.3 的构造对两种路径都适用）。

**示例**
```s3d
subtract crate on=floor {
  box shell size=5,4,5
  box hollow size=4.4,4,4.4 at=shell.bottom pin=bottom offset=0,0.3,0
}
union pier { cyl base r=2 h=1; cyl shaft r=1 h=6 on=base }
intersect lens { sphere a r=3; sphere b r=3 at=2,0,0 }
subtract body {                                   // 含 cyl axis=z 的拱顶开口：整组走距离场
  box outer size=34,22,26
  box inner size=32.4,21.2,24.4
  box opening size=12,15.4,2.4 at=outer.front pin=center align=bottom
  cyl arch r=6 h=2.4 axis=z at=opening.top pin=center
}
```

## 2.6 容器的包围盒

容器（group、repeat、mirror / rotate 的容器式、copy、模板实例；布尔容器见 2.5.4）的局部包围盒只定义几何，定位在第 3 章。

- **子物体包围盒之并**：记容器的子节点 $c$（实体、曲线、容器都算；按第 3 章被后置的子节点不算）在容器局部系里的变换为 $R_c, t_c$，盒为 $B_c$；容器的盒 $= \bigcup_c \mathrm{AABB}(R_c B_c + t_c)$ 的轴对齐包围盒（各轴取最小 / 最大；与求并的顺序无关）。曲线以其采样折线的盒计入（2.3.8）。
- 没有可计入的子节点：零盒。
- **envelope**（只有 `group` 接受，其他容器写了按第 1 章报错）：`envelope=w,h,d`（三个正数）$\to [-\tfrac w2, \tfrac w2] \times [0, h] \times [-\tfrac d2, \tfrac d2]$；`envelope=x0,y0,z0,x1,y1,z1`（六个数，要求 $x_0 \le x_1$、$y_0 \le y_1$、$z_0 \le z_1$，否则报错"`<路径>` 的 envelope 的最小值要不大于最大值"）$\to [x_0, x_1] \times [y_0, y_1] \times [z_0, z_1]$；其他个数报错"`<路径>` 的 envelope 写成 w,h,d 或 x0,y0,z0,x1,y1,z1"。有 envelope 时容器的盒就是它，**与内容无关**（内容可以超出）；关系、锚点、`w / h / d` 都按它。事实与渲染仍按内容（第 6 章）。
- 副本容器（mirror / rotate 的 `of=` 形式）与 copy 的盒等于源的盒（在各自局部系里，第 4 章）。
- 容器没有紧包围盒、SDF、体积与凸块（布尔容器除外：2.5.4、2.5.6）；第 6 章的事实按其内部实体分别计算（布尔容器整体算一个实体）。

**示例**
```s3d
group rocket on=pad { cyl stage1 r=2.4 h=28; cyl stage2 r=2 h=14 on=stage1 }   // 盒 = 两级的并
group lot envelope=20,0,30 at=0,0,100 { box car size=4,1.5,2 at=3,0,5 }        // 盒 = 名义外包络
group garage { cyl core r=4 h=14; helix rampline r=7 pitch=4.5 turns=3 }       // 曲线计入
```

## 2.7 mesh 的外包络与真实网格

`mesh` 的 `size`（外包络）与 `pivot` 是语句声明的名义几何；源网格是外部数据，编译不读它。两者不一致时按下表取舍，这是定义，不是错误：

| 运算 | 以谁为准 |
|---|---|
| 局部包围盒、紧包围盒、方向词锚点、`B[u,v,w]`、`w / h / d`、所在容器的盒、定位 | 外包络 |
| 凸块与空间事实 | 外包络（`approx=true`） |
| `top[u,w]` | 源网格（可用时），否则外包络顶面并警告（2.4.1） |
| 渲染、导出、评分的残差、三角化路径的布尔 | 源网格（可用时），否则占位盒并标记 `placeholder` |
| 体积 | 源网格（可用且闭合时），否则外包络体积并标为近似 |

源网格经 $v \mapsto v - \text{pivot}$ 后应当落在外包络里；超出时语言不报错（编译阶段看不到源），读取源网格的工具应当给出警告（工具约定，不在本规范内）。要让事实按真实形状，把网格改写成基本体或凸包。
