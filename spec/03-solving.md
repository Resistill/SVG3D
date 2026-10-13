# 3 定位与朝向的求解语义

**范围。** 本章定义场景树里每个节点的**局部变换**（相对父节点的旋转与平移）怎样由语句上的定位与朝向属性求出：锚点与度量的值、五种定位关系（on / under / against / inside / at）的落点、pin / gap / align / alignto / offset 的效果、朝向的继承与 turn / tilt / roll / rot / aim 的矩阵、求解顺序（依赖图、循环、后置）、容器的定位、repeat 的展开与实例变换、mirror / rotate 的副本变换、span / sweep / pipe 由端点或路径决定的局部系；最后用参考输出的数值验证这些规则。本章的规则是实现求解器的依据：两份独立实现按本章得到的 $R$、$t$ 与包围盒必须在 09-compat.md 的 9.4.2 规定的容差内一致（`world.R` 的九个元素各在 $\tau_{\text{rot}}$ 内，`world.t` 与 `box` 的分量各在 $\tau_{\text{len}}$ 内）；由这些量经阈值判定得到的离散量只允许在 9.4.4 的边界对上不同（`late` 没有阈值，必须相同）。

本章**不**定义：属性的语法与类型（见 01-syntax.md 的 1.4、1.5 节；本章假定值已按类型解析）；实体与曲线的局部几何——局部包围盒、曲面锚点的点与法线、曲线的采样与取点、span / sweep / pipe 的形状、容器包围盒的求并（见 02-geometry.md）；模板展开、copy、over / drop 对属性的改写、副本与影子的生成、孪生体（见 04-derivation.md）；空间事实（见 06-facts.md）；变体皮肤的挂载（见 07-semantics.md）。

**依赖的章。** 01-syntax.md（术语、属性表、表达式与求值环境、名字解析、节点顺序、错误恢复与退化节点）、02-geometry.md（局部包围盒、盒心与半尺寸、曲面锚点、曲线取点、扫掠框架、容器包围盒）。被依赖：02（旋转矩阵、定位点、后置）、04、06、07、09。

**记法。** 向量是列向量，$\hat x = (1,0,0)$、$\hat y = (0,1,0)$、$\hat z = (0,0,1)$；$\lVert v \rVert$ 是欧氏长度，$\mathrm{unit}(v) = v / \lVert v \rVert$；$u \circ v$ 是逐分量乘积；$u \cdot v$ 是点积，$u \times v$ 是叉积；$\mathrm{diag}(a,b,c)$ 是对角矩阵；$I$ 是单位矩阵。角度一律用度。

- **变换** $T = (R, t)$ 把点 $p$ 映到 $R\,p + t$，把方向 $d$ 映到 $R\,d$；$R$ 是 3×3 正交矩阵（$\det R = \pm 1$，$-1$ 只出现在镜像副本与以它们为祖先的节点上）。复合 $(A \circ B)(p) = A(B(p))$，即 $A \circ B = (R_A R_B,\ R_A t_B + t_A)$；逆 $T^{-1} = (R^{\mathsf T},\ -R^{\mathsf T} t)$。恒等变换记 $\mathbb 1 = (I, 0)$。
- **方向词向量**：`top`$=\hat y$、`bottom`$=-\hat y$、`right`$=\hat x$、`left`$=-\hat x$、`front`$=\hat z$、`back`$=-\hat z$、`center`$=0$（`up` `down` `east` `west` `south` `north` 是同义词，01-syntax.md 的 1.6.8）；组合词是各词向量之和（1.6.8）。六个**面**按 02-geometry.md 的 2.1.1：面名与其外法线方向词相同。$\mathrm{FACES} = (\text{left}, \text{right}, \text{front}, \text{back}, \text{top}, \text{bottom})$ 是本章在"取第一个最值"时使用的固定顺序。
- 节点 $n$ 的**局部包围盒** $B_n = [\min, \max]$（02-geometry.md 的 2.1.2），盒心 $c_n = \tfrac12(\min + \max)$，半尺寸 $h_n = \tfrac12(\max - \min)$，底面中心 $b_n = (c_{n,x}, \min_y, c_{n,z})$；零盒 $Z$ 的 $c = h = b = 0$。
- $P$ 记被定位节点的父节点（根也算一个框架）；"在 $P$ 里"指在 $P$ 的局部坐标系里表达。
- 示例里的数值保留 4 位小数。

## 3.1 求解模型

### 3.1.1 局部变换与世界变换

- 每个节点 $n$ 有一个**局部变换** $T_n = (R_n, t_n)$，把 $n$ 的局部坐标映到父节点 $P$ 的局部坐标。根（场景）没有变换；最外层节点的父框架就是世界框架。
- **世界变换** $W_n = W_P \circ T_n$，$W_{\text{根}} = \mathbb 1$。实现**必须**按这个顺序复合（先父后子），不得改变结合顺序。
- 两个节点之间的**相对变换** $T_{B \to F} = W_F^{-1} \circ W_B$（$B$ 的局部坐标到 $F$ 的局部坐标）。记 $A$ 为 $B$ 与 $F$ 的最近公共祖先（$B$、$F$ 本身可以是 $A$；根是任何两个节点的公共祖先），$b_1, \dots, b_m$ 是从 $A$ 的子节点到 $B$ 的链，$f_1, \dots, f_k$ 是从 $A$ 的子节点到 $F$ 的链，则
$$T_{B \to F} = \big(T_{f_1} \circ \cdots \circ T_{f_k}\big)^{-1} \circ \big(T_{b_1} \circ \cdots \circ T_{b_m}\big),$$
实现**必须**按这个公式只用两条链上的局部变换计算（链为空时取 $\mathbb 1$；复合从左到右结合），不得经过 $A$ 以上的节点；这既决定了依赖关系（3.6.1），也决定了浮点结果。
- 本章的全部定位量（锚点、法线、gap、align、offset、aim 的目标、span 的端点、曲线路径点、镜像面、旋转轴）都先换算到被定位节点的父框架 $P$ 里，再求 $T_n$。

### 3.1.2 每种节点的局部变换由什么决定

按下表第一个命中的行：

| 节点 | $T_n$ | 节 |
|---|---|---|
| 影子（副本里对应原件子节点的节点，01-syntax.md 的 1.6.3） | $= T$（原件，或有影子覆盖时原件的孪生体，04-derivation.md 的 4.4.4） | 3.9.7 |
| 副本式 `mirror R of=A` / `rotate R of=A` 的节点 R | $M \circ T_{A \to P}$ | 3.9.1 |
| 容器式 mirror / rotate 生成的副本组 `mirrored` / `r`$k$ | 容器自身框架里的镜像 / 旋转 $M_k$ | 3.9.2 |
| span、sweep、pipe | 由端点 / 路径决定 | 3.10 |
| line、bezier、spline | $\mathbb 1$（点在父框架里） | 3.10.5 |
| 阵列实例（repeat 的子节点） | 由阵列的 step / center / turn / along 决定 | 3.8 |
| 变体皮肤 `skin` | $(I,\ b_N - \pi_K)$：皮肤盒的底面中心放到宿主包络的底面中心（04-derivation.md 的 4.7 第 5 条） | 3.7.6 |
| 退化节点（01-syntax.md 的 1.1.4） | $\mathbb 1$ | — |
| 其他：实体、arc、helix、group / union / subtract / intersect、repeat 节点本身、容器式 mirror / rotate 节点本身、模板实例、copy 节点 | **一般规则**：$R_n$ 按 3.5，$t_n$ 按 3.3、3.4 | 3.1.3 |

### 3.1.3 一般规则的总览

对表中"其他"一行的节点（下文叫**一般节点**），求解分两步，第二步的各项按书写的顺序累加：

1. **朝向** $R_n = R_{\text{base}} \cdot R_y(\text{turn}) \cdot [\text{aim}] \cdot R_x(\text{tilt}) \cdot R_{\text{roll}}(\text{roll}) \cdot R_{\text{axis}}(\text{rot})$（3.5）。
2. **平移**：
   - 定位关系（3.3）给出参照物上的点 $q$、法线 $\nu$（都在 $P$ 里）与缺省定位点；定位点 $\pi$（3.4.1，在 $n$ 的局部系里）由 `pin` 或缺省决定；
   - $t \leftarrow q - R_n\,\pi$（把自己的定位点放到 $q$，3.3.9）；
   - $t \leftarrow t + g\,\nu$（`gap`，3.4.2）；
   - 对 `align` 的每个面名 / 轴名按书写顺序调整 $t$（3.4.3）；
   - $t \leftarrow t + o$（`offset`，3.4.4）；
   - $t_n = t$。

没有定位关系时 $q = 0$、$\nu = 0$（3.3.7）。所有量都是双精度；每一步的运算顺序如上，实现不得重排（例如不得先把 gap 与 offset 合并）。

### 3.1.4 主定位关系

- 一个节点**至多写一个**定位关系（`on` `under` `against` `inside` `at` 五者之一；多写报错"定位关系只能写一个"，01-syntax.md 的 1.4.4，节点退化）。写了的那一个叫它的**主定位关系**，关系所引用的对象叫**参照物**。
- "每个物体由一条主定位关系定位"的精确含义：$t_n$ 只由这一条关系（给出 $q$、$\nu$）、它自己的 pin / gap / align / offset 与自己的 $R_n$ 决定；没有联立、没有多关系的折中；align 只是在关系落点之后沿某个轴或法线的再平移（3.4.3）。
- 参照物可以是任何已求解的节点（实体、曲线、容器、阵列实例、副本、影子、copy、模板实例）或非空集合（3.2.8），但不能是自己、自己的祖先或后代（3.6.2）。
- 所有关系都在 $P$ 里计算：参照物 $B$ 的锚点经 $T_{B \to P}$ 换到 $P$（3.2.4）。因此 $n$ 与 $B$ 可以在树的任何位置，父框架不同也可以（例如 `box d on=h.b`，$h$ 是转过的组）。

### 3.1.5 求解顺序

- 求解顺序由**依赖图**决定（3.6.1）：一个节点的局部变换在它依赖的全部量求出之后计算；无依赖关系的节点之间的顺序对结果没有影响。源码的书写顺序**不**影响结果，除非本章明确用"节点顺序"（01-syntax.md 的 1.6.1）打破平局（3.6.3、3.6.5）。前向引用（引用后面才写的对象）合法。
- 依赖图有环时，按 3.6.3 判定后置、按 3.6.5 报循环依赖。

### 3.1.6 输出

每个节点输出：世界变换 $W_n$（`world.R`、`world.t`）；**世界包围盒** `box` $= \mathrm{AABB}(W_n(B_n))$——局部包围盒的 8 个角经 $W_n$ 变换后各轴取最小 / 最大（转过的物体的 `box` 因此比实体本身大；按真实形状的紧包围盒见 02-geometry.md 的 2.1.8）；后置标记 `late`（3.6.4）。退化节点输出 $W_n = W_P$、零盒。输出格式的其他字段不属于本章。

### 3.1.7 本章的常量

| 名字 | 值 | 单位 | 用途 |
|---|---|---|---|
| $\varepsilon_{\text{len}}$ | $10^{-9}$ | 场景单位 | 零长度：aim 的目标与起点重合、镜像面法线为零、方向规格为零、主法线的零判定（3.2.3，无量纲时同值）（与 02-geometry.md 的 2.1.4 同名同值） |
| $\varepsilon_{\text{dir}}$ | $10^{-6}$ | 无 | 单位向量的退化判定：aim 的次轴投影、`orient=full` 的 side |
| $\varepsilon_{\text{yaw}}$ | $10^{-9}$ | 无 | `orient=yaw`：切线水平分量的长度小于它时不转 |
| $\kappa_{\text{up}}$ | $0.99$ | 无 | aim：$\lvert a_y \rvert < \kappa_{\text{up}}$ 时次轴取 $\hat y$，否则取 $\hat z$（3.5.4） |
| $\kappa_{\text{perp}}$ | $0.9$ | 无 | $\mathrm{perp}(d)$：$\lvert d_y \rvert < \kappa_{\text{perp}}$ 时用 $d \times \hat y$，否则 $d \times \hat x$（3.5.4；与 02-geometry.md 的 2.2.8 相同） |
| $\varepsilon_{\text{int}}$ | $10^{-9}$ | 无 | repeat 的 `n` 与整数的允许偏差（3.8.1） |
| $N_{\max}$ | 5000 | 个 | 一个 repeat 的实例总数上限（3.8.6） |

这些常数不随 `metre` 换算。判定一律写成明确的不等式，边界按不等式的写法归属。

## 3.2 锚点与度量

锚点的**语法**在 01-syntax.md 的 1.5.4；本节定义每种写法的**值**。一个锚点求值得到参照物 $B$ 局部系里的三元组 $(p, \nu, \tau)$：点 $p$、法线 $\nu$（可以是零向量）、切线 $\tau$（只有曲线取点有，其余为"无"）。度量与坐标分量求值得到数。

### 3.2.1 参照物的局部系与盒

- 参照物 $B$ 是一个节点（或集合，3.2.8）。锚点在 $B$ 的局部系里按 $B$ 的局部包围盒 $B_B$（02-geometry.md）计算；有 `envelope` 的 group 用 envelope（02-geometry.md 的 2.6）；退化节点用零盒；曲线用采样折线的盒（02-geometry.md 的 2.3.8）。记 $c$、$h$ 为 $B_B$ 的盒心与半尺寸。
- 锚点**不看形状**：方向词锚点与 `B[u,v,w]` 永远取盒上的点（`on=wedge` 落在包围盒顶面，不在斜面上；`on=sphere` 落在球顶上方的盒顶，与球只有一点相切）。要取真实表面的点用曲面锚点（3.2.5）。

### 3.2.2 锚点的值

| 写法 | $p$（$B$ 局部系） | $\nu$ | $\tau$ |
|---|---|---|---|
| `B`（裸引用，用作点或 `at=` / `inside=` 的缺省锚点） | $c$ | $0$ | 无 |
| `B.origin` | $(0,0,0)$ | $0$ | 无 |
| `B.`方向词 / 组合词，向量 $a \in \{-1,0,1\}^3$（`center` 是 $a = 0$） | $c + h \circ a$ | $\mathrm{dominant}(a)$ | 无 |
| `B[u,v,w]`（三个数，不限于 $[-1,1]$） | $c + h \circ (u,v,w)$ | $\mathrm{dominant}((u,v,w))$ | 无 |
| `C[t]`、`C.start`、`C.mid`、`C.end`（$C$ 是曲线；$t$ 任意数，按 02-geometry.md 的 2.3.1 规整） | 曲线上的点 $C[t]$ | $0$ | 该点的单位切线 |
| `B.top[u,w]`、`B.side[θ,v]`、`B.surface[θ,φ]` | 02-geometry.md 的 2.4 | 同左 | 无 |

- 组合词的向量之和可以有多个非零分量；`top-left-front` $= (-1,1,1)$ 是盒的一个角，`top-left` $= (-1,1,0)$ 是一条棱的中点。
- `B[u,v,w]` 的分量超出 $[-1,1]$ 时点在盒外，合法。
- 对不是曲线的 $B$ 写 `B[t]` 报错"X 不是曲线，不能用 [t] 取点"；写 `B.start` / `B.mid` / `B.end` 时这一段不是曲线点，按后缀规则报错"X 中没有子对象或锚点 start"（两条都见 01-syntax.md 的 1.5.4）；对曲线写 `side` / `surface` 报错（02-geometry.md 的 2.4）。
- 退化节点的任何锚点都是它的原点（零盒），$\nu$ 按上表（例如 `.top` 仍是 $\hat y$），不再额外报错（01-syntax.md 的 1.1.4）。

### 3.2.3 主法线 dominant

向量 $a = (a_x, a_y, a_z)$ 的主法线 $\mathrm{dominant}(a)$：

1. 令 $m = \max(\lvert a_x \rvert, \lvert a_y \rvert, \lvert a_z \rvert)$；$m < \varepsilon_{\text{len}}$（3.1.7；含 $a = 0$）：$0$（原型 `dominant` 的同一判定，`B[1e-10,0,0]` 的法线是零）。
2. 否则：若 $\lvert a_y \rvert = m$ 取 $\mathrm{sign}(a_y)\,\hat y$；否则若 $\lvert a_x \rvert = m$ 取 $\mathrm{sign}(a_x)\,\hat x$；否则取 $\mathrm{sign}(a_z)\,\hat z$。

并列时 $y$ 优先、其次 $x$、最后 $z$：`B.top-right` 与 `B[1,1,0]` 的法线是 $\hat y$（当作顶面上的点），`B.right-front` 的法线是 $\hat x$。这是定义，不是近似。

### 3.2.4 换到框架 F 与偏移

把 $B$ 的锚点 $(p, \nu, \tau)$ 换到框架 $F$（通常是被定位节点的父节点 $P$）：
$$q = R_{B \to F}\,p + t_{B \to F} + o_{\text{ref}},\qquad \nu_F = R_{B \to F}\,\nu,\qquad \tau_F = R_{B \to F}\,\tau,$$
其中 $(R_{B \to F}, t_{B \to F}) = T_{B \to F}$（3.1.1），$o_{\text{ref}}$ 是引用上累加的偏移向量（`B.top+(dx,dy,dz)`，01-syntax.md 的 1.5.2；没有偏移时为 $0$）。

- **偏移在框架 $F$ 里加**，即在使用这个点的地方的坐标系里（定位属性：被定位节点的父框架；span 的端点、曲线路径点：该语句的父框架；坐标分量 `B.top.y` 的锚点：表达式的求值框架 $P$，01-syntax.md 的 1.5.8——分量后缀只能跟在不带偏移的锚点后面，1.5.1 的 `path_ref`，所以分量本身没有偏移）；它**不随 $B$ 旋转**。`sphere s at=B.top+(1,0,0)`，$B$ 有 `turn=90`：球心在 $B$ 顶面中心沿 $P$ 的 $+x$ 偏 1，而不是沿 $B$ 的 $+x$。要沿 $B$ 自己的轴偏移，用 `B[u,v,w]` 或 `B.top[u,w]`。
- 法线与切线不受偏移影响。
- 以下每处用到"锚点在 $P$ 里的点"都指本节的 $q$，"法线"指 $\nu_F$。

### 3.2.5 曲面锚点与曲线取点作为锚点

- 曲面锚点 `B.top[u,w]`、`B.side[θ,v]`、`B.surface[θ,φ]` 的 $p$、$\nu$ 由 02-geometry.md 的 2.4 给出（`top[u,w]` 的法线固定为 $\hat y$，`side` 与 `surface` 的法线是该处的真实外法线）。它们在定位里的用法：
  - `at=B.side[θ,v]`：$q = p$，$\nu$ 参与 gap（3.3.6）；
  - `against=B.side[θ,v]`：$\nu$ 既是 gap 的方向也是自动选面的依据（3.3.4）；
  - `on=B.top[u,w]`：$q = p$，但法线**不用**锚点的 $\nu$，一律是 $B$ 的 $+y$（3.3.2）；`on=B.side[…]` 同理（物体底面放到侧面上的点，沿 $B$ 的 $+y$ 抬 gap）。
- 曲线取点 `C[t]` 等：$\nu = 0$，所以 `at=C[0.3] gap=1` 报错（3.4.2）；切线 $\tau$ 只在方向规格（`axis=C[t]`、`dir=C[t]`，3.9.5）里用。
- 曲面锚点与曲线取点都不能做 `pin` 的值（01-syntax.md 的定位点类型）。

### 3.2.6 度量 B.w / B.h / B.d

`B.w`、`B.h`、`B.d` 是 $B$ 的**局部包围盒**沿 $B$ 自己的 $x$、$y$、$z$ 的边长（02-geometry.md 的 2.1.3；有 envelope 的 group 用 envelope；退化节点为 0；集合见 3.2.8）。它们与 $B$ 的朝向、与表达式所在的框架无关：$B$ 转了 90° 之后 `B.w` 仍是它自己 $x$ 向的边长。

### 3.2.7 坐标分量

- `锚点.x`、`锚点.y`、`锚点.z` 是该锚点（3.2.2 的任一写法；不能带偏移——01-syntax.md 的 1.5.1 的 `path_ref` 不接受 `(B.top+(1,0,0)).y`，要偏移就在分量上加数）换到表达式的**求值框架** $P$（01-syntax.md 的 1.5.8：节点属性里是该节点的父节点，`let` 里是声明它的块的节点，最外层即世界）之后的点 $q$ 的分量。`B.x`、`B.y`、`B.z` 是 $B$ 的盒心（裸引用的锚点）的分量。
- 例：`cyl post h=deck.bottom.y`、`span r from=slab.top-left to=(slab.left.x-12, 0, slab.z)`。
- 分量只是一个数：`B.top.y` 不带法线，也不记得来自 $B$（`at=B.top.y,0,0` 是一个纯点）。

### 3.2.8 集合作为参照物

参照物类型的属性（`on` `under` `against` `inside` `alignto`）、点类型的属性（`at` `aim` `from` `to` `about`、路径点）与经 `let` 绑定的集合名后面的锚点 / 度量 / 分量（01-syntax.md 的 1.6.5）都可以引用一个**集合** $S$（`#tag`、`a.*.b`、`[a; b]`、`kind()`……）。规则：

- $S$ 为空报错"集合是空的，不能当参照物"（01-syntax.md）。
- 集合的**框架是世界框架**：$T_{S \to F} = W_F^{-1}$。
- 集合的**盒** $B_S$ 是其成员（按节点顺序，含退化成员，不排除后置成员）的世界包围盒（3.1.6）之并的轴对齐盒；盒心、半尺寸、底面中心由它算。锚点按 3.2.2 在 $B_S$ 上取，法线是世界轴（经 $W_F^{-1}$ 换到 $F$）；`S.w / S.h / S.d` 是 $B_S$ 的世界轴向边长；`S.origin` 是世界原点；曲线取点与曲面锚点对集合不适用（`S[t]` 报错"{…} 不是曲线，不能用 [t] 取点"，`S.start` 等报错"{…} 中没有子对象或锚点 X"，按 01-syntax.md 的 1.5.4）。
- 以集合为参照物时继承的朝向是世界朝向（3.5.1）：在转过的容器里写 `on=#wall`，物体会被转回世界轴向。集合的成员在 $n$ 所在容器之外时，$n$ 按 3.6.3 可能后置。
- 集合里含 $n$ 自己、$n$ 的祖先或后代：报错"`<路径>` 的 `<属性>`：集合里包含它自己或它的上下级"（3.6.2）。

```s3d
box slab size=36,0.6,6 on=piers.*.p          // 放在全部桥墩的总包围盒顶面中心
let walls = #wall
box roof size=12,0.5,8 at=walls.top pin=center
```

### 3.2.9 点

类型为"点"的属性值（`at` 的非引用形式、`aim`、`from`、`to`、`about`、曲线与扫掠路径里的项）按下面的规则求成框架 $F$ 里的一个点：

| 值 | 点 |
|---|---|
| 向量₃ $(x,y,z)$ | 就是 $F$ 里的这个点 |
| 引用（可带锚点、偏移） | 3.2.4 的 $q$；没有锚点时取盒心 |
| 集合 | 集合盒（3.2.8）的中心经 $W_F^{-1}$ 换到 $F$ |
| 其他 | 报错"`<路径>` 的 `<属性>` 需要点坐标或锚点"（01-syntax.md） |

引用形式的点另带法线 $\nu_F$（`at=` 用它，3.3.6）；向量与集合形式的法线为 $0$。

## 3.3 定位关系

### 3.3.1 关系给出的量

主定位关系（3.1.4）求值得到四样东西，都在父框架 $P$ 里：

| | 含义 |
|---|---|
| $q$ | 参照物上的点：$n$ 的定位点将落在这里 |
| $\nu$ | 关系的法线（单位向量或零向量）：gap 沿它走（3.4.2） |
| 缺省定位点 | `pin` 没写时 $\pi$ 用哪个（3.4.1） |
| 是否继承朝向 | $R_{\text{base}}$ 是否取参照物的朝向（3.5.1） |

下表是五种关系的定义；$B$ 是参照物，"锚点"指写在引用上的锚点（`on=B.top[0.3,0]`、`at=B[u,v,w]`），"缺省锚点"是没写时用的。$R_{B \to P}\hat y$ 等记"B 的 $+y$ 在 $P$ 里的方向"。

| 关系 | 缺省锚点 | $q$ | $\nu$ | 缺省定位点 | 继承 |
|---|---|---|---|---|---|
| `on=B` | `top` | 锚点在 $P$ 里的点 | $R_{B \to P}\,\hat y$（**与锚点无关**） | `bottom` | 是 |
| `under=B` | `bottom` | 同上 | $-R_{B \to P}\,\hat y$（与锚点无关） | `top` | 是 |
| `against=B.面` | 无（必须写） | 锚点的点 | 锚点的法线 $\nu_F$ | 自动选面（3.3.4） | 是 |
| `inside=B` | `center` | 锚点的点 | $0$ | `center` | 是 |
| `at=B`（引用，可带锚点与偏移） | `center` | 锚点的点 | 锚点的法线 $\nu_F$ | `bottom` | **否** |
| `at=x,y,z` / `at=集合`（非引用的点） | — | 该点（3.2.9） | $0$ | `bottom` | 否 |

参照物是集合时，$R_{B \to P}$ 取 $W_P^{-1}$ 的旋转部分（3.2.8）。

### 3.3.2 on

- $q$ 是锚点的点：缺省顶面中心 $c + h \circ \hat y$；写了锚点用写的（`on=floor[-0.5,1,0]` 是顶面上偏左的点；`on=B[1,0,0]` 是右侧面的中心；`on=B.top[u,w]` 是真实表面上的点）。
- $\nu$ **总是** $B$ 的 $+y$ 在 $P$ 里的方向，不管锚点在哪个面上、法线是什么。所以 `on=B[1,0,0] gap=1` 把物体放到 $B$ 右侧面的中心、再沿 $B$ 的上方抬 1——`on` 的意思是"立在 $B$ 的上方向上"。要贴到侧面用 `against`，要沿锚点法线留空用 `at=B.right gap=1`。
- 缺省定位点 `bottom`：自己的底面中心落在 $q$。
- 继承 $B$ 的朝向：$B$ 倾斜，自己跟着倾斜，底面贴在 $B$ 的顶面上（3.5.5）。
- `on=B` 只决定**位置**，不产生空间事实；事实只看求解后的几何（06-facts.md 的 6.5.7）：`box a on=B` 在缺省 gap 下得到事实 `a on B`，`on=B gap=-0.1` 得到的是 `a intersects B`，`B` 是容器时事实落在容器里的实体上。

```s3d
box table size=8,3,5 on=floor[-0.5,1,0]
cyl cup r=0.6 h=1.2 on=table
sphere ball r=0.8 on=table[0.6,1,-0.5]
cyl band r=2.45 h=0.8 on=stage1 gap=-0.8      // 负 gap 沉进参照物
box deck size=18,0.5,3.6 on=archline           // 曲线也可以当参照物：按它的采样折线盒
```

### 3.3.3 under

与 `on` 上下对称：缺省锚点 `bottom`，$\nu = -R_{B \to P}\hat y$，缺省定位点 `top`（自己的顶面中心贴到 $B$ 的底面中心），继承朝向。`gap` 为正时向下离开 $B$。

### 3.3.4 against

- 必须写一个**带非零法线的锚点**：方向词 / 组合词锚点（`B.left`、`B.top-right`）、`B[u,v,w]`（$\mathrm{dominant} \ne 0$）或曲面锚点（`B.side[90,0.5]`、`B.surface[0,45]`、`B.top[u,w]`）。裸 `B`、`B.origin`、`B.center`、`B[0,0,0]`、曲线取点（法线为零）报错"`<路径>`：against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]"，节点退化。原型（`relation`）只拒绝裸引用与 `origin` / `center` 一类非向量锚点，对 `B[0,0,0]` 这样主法线为零的向量锚点不报错（$\nu = 0$，自动选面时全部得分为 0 而取 `left`，gap 无效）；本版报错（3.11.5）。
- $q$ 是锚点的点，$\nu$ 是锚点的法线（在 $P$ 里）。
- **自动选面**（没写 `pin` 时）：在 $\mathrm{FACES}$ 的顺序里取使 $(R_n\,f) \cdot \nu$ **最小**的面 $f$（自己外法线在 $P$ 里与 $\nu$ 最反向的面；并列取顺序靠前的）；定位点 $\pi$ 是该面的中心 $c_n + h_n \circ f$。$R_n$ 是 3.5 求出的完整朝向（含继承、turn、tilt……），所以 `against=A.right tilt=90` 会选到翻转后朝向 $A$ 的那个面。
- 写了 `pin` 时不选面：`pin` 指定的点直接落到 $q$（`against=A.right pin=top` 把自己的顶面中心贴到 $A$ 的右面中心）。
- 继承 $B$ 的朝向；$B$ 是镜像时按 3.5.7。
- `gap` 为正时沿 $\nu$ 离开面。
- `against` 只决定位置，不产生事实；事实按几何（06-facts.md 的 6.5.7）：两个轴对齐的盒类贴面时得到 `a against B.<面>`，$B$ 是圆柱时得到 `a touching B`。

```s3d
box book1 size=0.6,3,1.6 against=shelf.front align=bottom,left
box book2 size=0.6,2.6,1.6 against=book1.right align=bottom,back
wedge berm size=7,5,10 against=bunker.left align=back,bottom   // bunker turn=30：berm 继承 30°
box tag size=0.1,0.3,0.3 against=tank.side[90,0.5]             // 贴在圆柱侧面，法线径向
```

### 3.3.5 inside

- 缺省锚点 `center`，$q$ 是 $B$ 的盒心（或写的锚点）；$\nu = 0$（`gap` 报错，3.4.2）；缺省定位点 `center`（自己的盒心落到 $q$）；继承朝向。
- `inside` 只按盒定位，不检查是否真的在 $B$ 内，也不看 $B$ 的形状（`inside=crate.hollow` 把球心放到被减体的盒心；`align=bottom` 再把它落到底）。是否真的得到事实 `a inside B` 由 06-facts.md 的 6.5.7 按几何判定（$B$ 单块、体积够大）；$B$ 是 group 时事实落在 group 里的实体上。

```s3d
sphere orb r=1.4 inside=crate.hollow align=bottom
cyl spare r=2.4 h=20 axis=x inside=hangar.body align=bottom
```

### 3.3.6 at

`at` 的值是点类型（01-syntax.md），四种形式：

| 形式 | $q$ | $\nu$ | 例 |
|---|---|---|---|
| 向量₃ | $P$ 里的坐标 | $0$ | `at=-7,0,-8`、`at=stage1.w/2,0,0`（分量是表达式，整体是向量） |
| 引用，无锚点 | $B$ 的盒心 | $0$ | `at=sat`、`at=hub.center` |
| 引用，带锚点（可带偏移） | 锚点的点 | 锚点的法线：方向词 / `B[u,v,w]` 的 dominant、曲面法线、曲线取点为 $0$ | `at=shell.bottom`、`at=B.top+(1,0,0)`、`at=C[0.3]`、`at=core.front+(0,0.2,0)` |
| 集合 | 集合盒心 | $0$ | `at=#wall` |

- 缺省定位点 `bottom`；**不继承**朝向（$R_{\text{base}} = I$）：`at=B.top` 把自己立在 $B$ 顶面中心但保持父框架的轴向；要跟着 $B$ 转用 `on=B`。
- 带法线的形式让 `gap` 有效：`at=B.right gap=1` 沿 $B$ 的 $+x$ 推 1（与 `on=B.right gap=1` 沿 $B$ 的 $+y$ 抬 1 不同）。
- `at` 没有写锚点时 $q$ 是盒心，所以 `box a at=B` 把 $a$ 的底面中心放到 $B$ 的中心。

### 3.3.7 没有关系

- 没写任何关系、也没写 `pin`：$q = 0$、$\nu = 0$、$\pi = $ **原点** $(0,0,0)$，于是 $t = 0 + o$：节点的**原点**落在父原点（再加 offset）。对原点在底面中心的实体这与"底面中心落在父原点"相同；对 plate、hull、revolve、arc、helix（原点不在盒底面中心，02-geometry.md 的 2.1.2）这意味着它们的**坐标原点**落在父原点——`arc loop r=40 angle=-60,240` 的圆心在父原点，不是弧的盒底面中心。
- 没写关系但写了 `pin`：$q = 0$、$\nu = 0$、$\pi$ 按 `pin`：`box floor size=40,1,24 pin=top` 把顶面中心钉在父原点（$t = (0,-1,0)$）。
- 没有关系时 `align` 必须配 `alignto`（3.4.3），`gap` 报错（3.4.2），`offset` 照常生效（3.4.4），`aim` 的起点按 3.5.4。

### 3.3.8 互斥

五个关系键至多一个（3.1.4）。本版不按优先级取其一（原型里 `on` 优先于其他、多写的被静默忽略；本版报错）。

### 3.3.9 落点公式

关系求出 $q$、$\nu$，定位点 $\pi$（3.4.1）在 $n$ 的局部系里，则
$$t = q - R_n\,\pi .$$
含义：先把节点绕**自己的原点**转到 $R_n$，再平移使它的定位点与 $q$ 重合；定位点既是落点也是旋转支点——`pin=center` 的物体 `tilt=30` 时绕盒心倾斜，`pin=bottom`（缺省）的物体绕底面中心倾斜。之后再加 gap、align、offset（3.4.5）。

## 3.4 pin、gap、offset、align、alignto

本节的量都承接 3.3：关系给出 $q$、$\nu$（在 $P$ 里）与缺省定位点；$R_n$ 由 3.5 求出。五个属性的类型与缺省见 01-syntax.md 的 1.4.4（`pin` 定位点、`gap` 数、`offset` 向量₃、`align` 面名列表、`alignto` 参照物）。

### 3.4.1 定位点 pin

定位点 $\pi$ 是 $n$ **局部系**里的一个点，由 `pin` 的值按下表求出（$c_n$、$h_n$ 是 $n$ 的局部包围盒的盒心与半尺寸，3 章记法；容器用容器的盒，有 `envelope` 的 group 用 envelope，退化节点与空容器用零盒）：

| `pin` 的值 | $\pi$ | 例 |
|---|---|---|
| 没写 | 关系的缺省定位点（3.3.1 的表：`on` / `at` → `bottom`，`under` → `top`，`inside` → `center`，`against` → 自动选面 3.3.4，没有关系 → `origin`，3.3.7） | |
| `origin` | $(0,0,0)$ | `arc loop … pin=origin`：圆心落到 $q$ |
| 方向词 / 组合词，向量 $a \in \{-1,0,1\}^3$（`center` 是 $0$） | $c_n + h_n \circ a$ | `pin=top`、`pin=bottom-right`（$a = (1,-1,0)$：底面右棱的中点） |
| 向量₃ $(u,v,w)$（任意数，不限于 $[-1,1]$） | $c_n + h_n \circ (u,v,w)$ | `pin=0,1,0` 与 `pin=top` 相同；`pin=0,-1,0` 把盒底面之下一个半高处钉到 $q$ |

- 定位点**只按局部包围盒**算（与锚点一样不看形状，3.2.1）：`sphere … pin=top` 是球顶，`wedge … pin=top` 是包围盒顶面中心（在斜面之上）。
- 零盒（退化节点、空容器）的任何定位点都是原点。
- `pin` 对 span、sweep 是几何参数（02-geometry.md 的 2.2.8、2.2.9，枚举 `bottom` / `top` / `center`），不是定位点；pipe、line、bezier、spline 没有 `pin`（01-syntax.md 的 1.4.4）。
- 定位点同时是旋转支点（3.3.9）：`box fin … turn=180 pin=bottom-right` 绕底面右棱中点转。
- `pin` 的值不是 `origin`、方向词 / 组合词或三个数时报错"pin 写成 origin、方向词或 u,v,w"（01-syntax.md 的 1.4.1），节点退化。

### 3.4.2 gap

$$t \leftarrow t + g\,\nu,$$
$g$ 是 `gap` 的值（数，缺省 0），$\nu$ 是关系的法线（3.3.1，单位向量，在 $P$ 里）。正值沿法线离开参照物，负值沉进去（`cyl band … on=stage1 gap=-0.8`：底面比 stage1 顶面低 0.8）。

- **$\nu = 0$ 时写了 `gap`**（不论值是多少）报错"`<路径>`：gap 需要有法线的定位关系（on / under / against，或 at= 带面锚点）"，节点退化。$\nu = 0$ 的情形：`inside`（3.3.5）、`at=` 向量 / 集合 / 无锚点引用 / 曲线取点（3.3.6）、没有关系（3.3.7）。原型（`local`）在 $\nu = 0$ 时静默忽略 gap；本版报错。
- `gap` 与 `offset` 不合并：先加 gap，再做 align，再加 offset（3.4.5）。
- gap 的方向由**关系**决定，不由锚点决定：`on=B[1,0,0] gap=1` 沿 $B$ 的 $+y$（3.3.2），`at=B.right gap=1` 沿 $B$ 的 $+x$（3.3.6）。

### 3.4.3 align 与 alignto

`align` 的值是面名列表（01-syntax.md 的 1.4.1：每项 ∈ {`left` `right` `front` `back` `top` `bottom` `x` `y` `z`}，组合词与 `center` 不合法，报错"align 只接受面名或 x / y / z，如 left,back"）。它在关系落点之后、offset 之前，按**书写顺序**逐项平移 $t$，不改 $R_n$。

**对齐目标** $A$：写了 `alignto` 用它（参照物类型，可以是集合）；否则用主定位关系的参照物（`on` `under` `against` `inside` 的参照物，或 `at=` 引用形式的参照物）。没有目标（没有关系；`at=` 是向量或集合；只有 `pin`）报错"`<路径>`：align 需要参照（on / under / against / inside / at=对象 或 alignto=对象）"，节点退化。`alignto` 没有 `align` 报错"alignto 需要 align"（01-syntax.md 的 1.4.4）。目标上的锚点与偏移**不参与** align：下面两种锚点都在 $A$ 的盒上重新取、偏移取 $0$。

对列表里的每一项 $f$，记当前的 $T_0 = (R_n, t)$：

1. **轴名** $f \in \{x, y, z\}$（轴序号 $k$）：**盒心对齐**——$q_c$ = $A$ 的盒心换到 $P$（3.2.4，锚点 `center`，偏移 0），$p_c = T_0(c_n)$ 是自己盒心在 $P$ 里的位置；$t_k \leftarrow t_k + (q_{c,k} - p_{c,k})$，只改 $P$ 的第 $k$ 个坐标。
2. **面名** $f$（向量 $a_f$）：**面对齐**——
   - 目标面：$q_f$、$\nu_f$ = $A$ 的锚点 `f`（$c_A + h_A \circ a_f$，法线 $\mathrm{dominant}(a_f) = a_f$）换到 $P$（3.2.4）；
   - 自己的面：在 $\mathrm{FACES}$ 的顺序里取使 $(R_n\,a_g) \cdot \nu_f$ **最大**的面 $g$（自己外法线在 $P$ 里与目标面法线最同向的面；并列取顺序靠前的；$R_n$ 是 3.5 的完整朝向）；$p_g = T_0(c_n + h_n \circ a_g)$；
   - $t \leftarrow t + \big((q_f - p_g)\cdot \nu_f\big)\,\nu_f$：沿目标面的法线平移，使自己那个面的中心落到目标面所在的**平面**上（不是落到面的范围内；也不改变沿其他方向的位置）。

- 轴名对齐在 $P$ 的坐标轴上做，面名对齐在**目标自己的面法线**上做；参照物转过时两者不同：`on=floor turn=30 align=x` 把盒心的 $x$ 对齐到 floor 盒心的 $x$；`align=back` 把自己朝 $-z_{\text{floor}}$ 的那个面（选面用的是转过之后的外法线）贴到 floor 的后面所在的平面。
- 同一项写两次（`align=left,left`）做两次，第二次位移为 0。先 `align=x` 再 `align=left`（即 `align=x,left`）以后者为准，因为 left 的平移改掉了 x 对齐的结果。
- 目标是集合时用集合的盒（3.2.8），法线是世界轴换到 $P$。
- 自己是零盒时 $c_n = h_n = 0$：面对齐把原点放到目标面的平面上，轴对齐把原点对到目标盒心的该坐标。

```s3d
box shelf size=10,9,2 on=floor align=back                      // 后面贴到 floor 的后面平面
box book1 size=0.6,3,1.6 against=shelf.front align=bottom,left   // 先底对底，再左对左
cyl spare r=2.4 h=20 axis=x inside=hangar.body align=bottom     // 落到被减体的底面
box sign size=3,1,0.1 on=plinth align=x,z alignto=gate          // 盒心的 x、z 对齐到 gate，仍立在 plinth 上
```

### 3.4.4 offset

$$t \leftarrow t + o,$$
$o$ 是 `offset` 的值（向量₃，缺省 0），**在父框架 $P$ 里**加，不随 $R_n$ 转、不随参照物转（原型 `local` 的最后一步；用参考输出核对见 3.11，并经小例子确认：`box a turn=90 offset=1,0,0` 得 $t = (1,0,0)$；转过 90° 的组里 `box b offset=1,0,0` 的 $b$ 沿组的 $+x$ 偏 1）。要沿自己的轴偏移，用 `pin` 的向量形式或在锚点里写 `B[u,v,w]`；要沿参照物的轴偏移，用 `B[u,v,w]` / `B.top[u,w]`。

- offset 对所有一般节点有效，包括没有关系的（3.3.7：$t = o$）、容器与阵列节点。
- 引用上的偏移（`at=B.top+(0,0.2,0)`，3.2.4）与 `offset=0,0.2,0` 对定位的效果相同；区别是前者影响锚点 $q$（因此也影响 `aim` 的起点 3.5.4），后者只在最后加。

### 3.4.5 合成顺序与示例

一般节点的平移按 3.1.3 的顺序，不得重排：

$$t = \underbrace{q - R_n\,\pi}_{\text{3.3.9}} \;+\; \underbrace{g\,\nu}_{\text{3.4.2}} \;\xrightarrow{\ \text{align 逐项（3.4.3）}\ }\; t' \;+\; \underbrace{o}_{\text{3.4.4}}.$$

align 要用到"当前的 $t$"（求 $p_c$、$p_g$），所以它放在 gap 之后、offset 之前这件事有观察后果：`on=B gap=1 align=bottom` 的 bottom 对齐把 gap 抬起的量又消掉（底面贴回 $B$ 的底面平面），而 `on=B align=bottom offset=0,1,0` 最后仍抬 1。

**示例**（basics.s3d；floor 是 $40 \times 1 \times 24$ 的盒、`pin=top`，顶面在 $y = 0$）：

```s3d
box turned size=6,3,3 on=floor[-0.1,1,0.55] turn=35
box sticker size=1,1.5,1 against=turned.right align=bottom
```

turned：$q = c_{\text{floor}} + h \circ (-0.1, 1, 0.55) = (-2, 0, 6.6)$，$R = R_y(35°)$，$\pi$ = 底面中心 $= 0$，$t = (-2, 0, 6.6)$。
sticker：继承 $R_{\text{base}} = R_y(35°)$（3.5.1），$R_n = R_y(35°)$；$q$ = turned 的右面中心 $(3, 1.5, 0)$ 换到 $P$：$(0.4575, 1.5, 4.8793)$，$\nu = R_y(35°)\hat x = (0.8192, 0, -0.5736)$；自动选面：$(R_n a_g)\cdot\nu = a_g \cdot \hat x$ 最小的是 `left`，$\pi = (-0.5, 0.75, 0)$；$t = q - R_n\pi = (0.8671, 0.75, 4.5925)$；`align=bottom`：$q_f = (-2, 0, 6.6)$、$\nu_f = -\hat y$，自己最朝下的面是 bottom，$p_g = t = (0.8671, 0.75, 4.5925)$，位移 $((q_f - p_g)\cdot\nu_f)\nu_f = (0, -0.75, 0)$；最终 $t = (0.8671, 0, 4.5925)$——与参考输出一致（3.11）。

## 3.5 朝向

$R_n$ 是 $n$ 的局部轴在 $P$ 里的方向（列向量是局部 $\hat x, \hat y, \hat z$ 的像）。本节定义它的每个因子；对 span / sweep / pipe、曲线 line / bezier / spline、阵列实例、副本与影子，$R_n$ 由 3.8–3.10 另定，本节只适用于一般节点（3.1.2）。

### 3.5.1 缺省朝向：继承

$$R_{\text{base}} = \begin{cases} R_{B \to P} & \text{主定位关系是 } \texttt{on} / \texttt{under} / \texttt{against} / \texttt{inside}\text{，参照物 } B \\ I & \text{主定位关系是 } \texttt{at}\text{（任何形式），或没有关系} \end{cases}$$

$R_{B \to P}$ 是 $T_{B \to P}$ 的旋转部分（3.1.1；$B$ 是集合时取 $W_P^{-1}$ 的旋转部分，即继承世界朝向，3.2.8）。含义：

- 贴在、放在、装在一个物体上（下）的东西跟它一起转——`bunker turn=30` 上的 `mast`、`berm` 都是 30°，`turned turn=35` 右面上的 `sticker` 是 35°；参照物在转过的容器里时，容器的旋转也在 $R_{B \to P}$ 里。
- `at=` **不**继承：`at=B.top` 把物体立在 $B$ 顶面中心，但轴向与父框架相同（3.3.6）。
- 没有关系的节点取父框架的轴向（$I$）——在转过的组里就是组的轴向。
- $\det R_{B \to P} = -1$（参照物是镜像副本或其后代）时按 3.5.7 修正。
- `repeat` 节点：$R_n = R_{\text{base}}$，不再乘任何因子（它的 `turn` 是实例的递增角，3.8.3；`tilt` `roll` `rot` `aim` 对 repeat 不合法，01-syntax.md 的 1.4.4）。

### 3.5.2 turn、tilt、roll

三个角都是度（数，缺省 0）。矩阵（$c = \cos\theta$，$s = \sin\theta$）：

| 属性 | 矩阵 | 轴与正方向 | 效果 |
|---|---|---|---|
| `turn` | $R_y(\theta) = \begin{pmatrix} c & 0 & s \\ 0 & 1 & 0 \\ -s & 0 & c \end{pmatrix}$ | 绕局部 $+y$，右手（俯视逆时针） | $\hat x \mapsto (c, 0, -s)$：右转向后（北）；$\hat z \mapsto (s, 0, c)$：前转向右（东） |
| `tilt` | $R_x(\theta) = \begin{pmatrix} 1 & 0 & 0 \\ 0 & c & -s \\ 0 & s & c \end{pmatrix}$ | 绕局部 $+x$，右手 | $\hat y \mapsto (0, c, s)$：顶部倒向前（$+z$） |
| `roll` | $R_{\text{roll}}(\theta) = \begin{pmatrix} c & s & 0 \\ -s & c & 0 \\ 0 & 0 & 1 \end{pmatrix}$ | 绕局部 $+z$，**左手**（$= R_z(-\theta)$） | $\hat y \mapsto (s, c, 0)$：顶部倒向右（$+x$） |

- roll 的正方向是"顶部倒向右"，所以它等于绕 $+z$ 的右手旋转 $-\theta$：`roll=5` 与 `rot=(0,0,1,-5)` 相同。这是定义（语料 `pisa roll=5` 向东倾）。
- 三者都是**内旋**（绕自己当前的轴）：$R_{\text{base}} R_y(\text{turn}) R_x(\text{tilt}) R_{\text{roll}}(\text{roll})$ 从右向左读是"先 roll、再 tilt、再 turn"作用在局部点上，从左向右读是"先 turn 转父系的 $y$、再 tilt 转转过之后的 $x$、再 roll 转再转过之后的 $z$"。`turn=30 tilt=20`（rotation.s3d 的 `sat`）得 $R = R_y(30°) R_x(20°)$，第一列 $(0.8660, 0, -0.5)$、第二列 $(0.1710, 0.9397, 0.2962)$、第三列 $(0.4698, -0.3420, 0.8138)$。
- 角度不规整（不取模、不限范围）；$\sin$ / $\cos$ 用 01-syntax.md 的 1.1.3 要求的确定性实现。

### 3.5.3 rot

`rot=(a_x, a_y, a_z, θ)`（向量₄；其他长度报错"`<路径>` 的 rot 写成 (轴x,轴y,轴z,角度)"）：绕局部轴 $\hat a = \mathrm{unit}(a)$ 右手旋转 $\theta$ 度（Rodrigues）：
$$R_{\text{axis}}(a, \theta) = c\,I + s\,[\hat a]_\times + (1 - c)\,\hat a \hat a^{\mathsf T},\qquad [\hat a]_\times = \begin{pmatrix} 0 & -a_z & a_y \\ a_z & 0 & -a_x \\ -a_y & a_x & 0 \end{pmatrix}.$$

- $\lVert a \rVert < \varepsilon_{\text{len}}$ 报错"`<路径>` 的 rot 的轴不能是零向量"，节点退化（原型对零轴算出 $cI$，不是正交矩阵；本版报错）。
- `rot` 在 turn / tilt / roll 之后乘（3.5.6）；`rot=(0,1,0,θ)` 与 `turn=θ` 等价，`rot=(1,0,0,θ)` 与 `tilt=θ` 等价，`rot=(0,0,1,θ)` 与 `roll=-θ` 等价。
- 轴向量在**节点自己的局部系**里（内旋）；要绕父系的轴转，把 rot 写在还没转过的因子前面没有语法，用等价的 turn / tilt / roll 或改写轴向量。

```s3d
rotate rotor n=2 axis=z at=hub.center pin=origin rot=(0,0,1,phase)   // 整体先转一个相位
box ramp size=12.2,0.2,1.5 at=floor[0.575,8,-0.43] rot=(0,0,1,35)
```

### 3.5.4 aim 与 aimwith

`aim` 的值是点（3.2.9，在 $P$ 里求成 $g$）；`aimwith` 是方向词或组合词（缺省 `front`；不是方向词报错"`<路径>` 的 aimwith 需要方向词，如 front / top / right"，没有 `aim` 时写 `aimwith` 报错"aimwith 需要 aim"，01-syntax.md 的 1.4.4）。效果：让自己的 `aimwith` 方向指向目标 $g$，并尽量保持"上"。

记 $R_1 = R_{\text{base}} R_y(\text{turn})$（3.5.1、3.5.2）。

1. **起点** $s$：有主定位关系时 $s = q$（关系的落点，3.3.1；含引用上的偏移，不含 gap / align / offset）；没有关系但写了 `pin` 时 $s = q = 0$（3.3.7，同样不含 offset）；既没有关系也没有 `pin` 时 $s = o$（`offset` 的值；没写则 $0$）。三种情形都与原型 `orientation` 相同（`relation` 对只有 `pin` 的节点也返回 $q = 0$；小例子见 3.11.4）。
2. $d = g - s$；$\lVert d \rVert < \varepsilon_{\text{len}}$ 报错"`<路径>`：aim 的目标与自身重合"，节点退化。$\hat d = \mathrm{unit}(d)$。
3. 主轴 $\hat a = \mathrm{unit}(a)$，$a$ 是 `aimwith` 的方向向量（组合词取各词向量之和再单位化，`top-left` $\to \mathrm{unit}(-1, 1, 0)$）。
4. 次轴 $b$：$\lvert \hat a_y \rvert < \kappa_{\text{up}} = 0.99$ 时 $b = \hat y$（保持"上"），否则 $b = \hat z$（主轴本身是上下时保持"前"）；$\hat b = \mathrm{unit}(b - (b \cdot \hat a)\hat a)$。
5. 次轴在 $P$ 里的期望方向：$e_0 = R_1\,b$；去掉沿 $\hat d$ 的分量 $e_1 = e_0 - (e_0 \cdot \hat d)\hat d$；$\lVert e_1 \rVert < \varepsilon_{\text{dir}} = 10^{-6}$ 时 $\hat e = \mathrm{perp}(\hat d)$，否则 $\hat e = \mathrm{unit}(e_1)$。其中 $\mathrm{perp}(v) = \mathrm{unit}(v \times \hat y)$ 当 $\lvert v_y \rvert < \kappa_{\text{perp}} = 0.9$，否则 $\mathrm{unit}(v \times \hat x)$（与 02-geometry.md 的 2.2.8 相同）。
6. $$A = \big[\,\hat d\ \ \hat e\ \ \hat d \times \hat e\,\big]\;\big[\,\hat a\ \ \hat b\ \ \hat a \times \hat b\,\big]^{\mathsf T}$$（方括号里是列向量拼成的矩阵）。$A$ 是正交矩阵，$A\hat a = \hat d$，$A\hat b = \hat e$。

aim 的结果 $A$ **取代** $R_1$（继承与 turn 只通过第 5 步的 $e_0 = R_1 b$ 影响结果，即只决定绕视线的滚转）；之后再乘 tilt、roll、rot（3.5.6）。

- 起点取关系落点而不是自己的盒心，是为了让"放在 B 上、对准 C"的物体从它站立的那一点瞄准；要从别处瞄准，用 `pin` 改落点、用引用上的偏移改 $q$，或（既没有关系也没有 `pin` 时）用 `offset`。
- 目标可以是引用（取锚点或盒心）、集合（盒心）或向量。目标在另一个容器里时，$n$ 可能后置（3.6.3，rotation.s3d 的 `radar.dish aim=sat`）。

**示例**（rotation.s3d）：`revolve dish … on=mast aimwith=top aim=sat`，radar 组在世界 $(-44, 0, -36)$，mast 顶面中心在组里是 $(0, 9, 0)$，sat 组的盒心在世界 $(-59.8290, 58.9397, -71.7038)$。$s = q = (0, 9, 0)$，目标换到 radar 组里 $g = (-15.8290, 58.9397, -35.7038)$，$\hat d = (-0.2497, 0.7877, -0.5632)$；$\hat a = \hat y$（$\lvert \hat a_y \rvert \ge 0.99$）所以 $b = \hat z$、$\hat b = \hat z$；$R_1 = I$，$e_0 = \hat z$，$\hat e = \mathrm{unit}(\hat z - \hat z_{\parallel \hat d})$；$A$ 的第二列（$\hat y$ 的像）$= \hat d$——与参考输出 `radar.dish` 的 $R$ 第二列 $(-0.2497, 0.7877, -0.5632)$ 一致，$t = q - A\,\pi = (0, 9, 0)$（dish 的盒底面中心是原点）。

### 3.5.5 继承的后果

- 继承让关系"以参照物自身的面为准"：`against=turned.right` 贴的是 turned 转过之后的右面，法线 $\nu = R_{B \to P}\hat x$，物体自己也转同样的角，于是面与面平行相贴。
- 倾斜的参照物：`on=B`，$B$ 有 `tilt=25`，则 $n$ 也 tilt 25°，底面贴在 $B$ 倾斜的顶面上（$\nu = R_{B \to P}\hat y$ 是倾斜的顶面法线，gap 沿它走）。
- 继承只取 $R_{B \to P}$，不取 $B$ 自己的 pin / gap / offset，也不层层累加：$n$ 的 `turn` 是相对 $R_{B \to P}$ 再转。
- 参照物是集合时继承世界朝向：转过的容器里 `on=#wall` 的物体会被转回世界轴向（3.2.8）。

### 3.5.6 合成顺序

$$R_n = \big(\text{aim 存在 ? } A : R_{\text{base}}\,R_y(\text{turn})\big)\ \cdot\ R_x(\text{tilt})\ \cdot\ R_{\text{roll}}(\text{roll})\ \cdot\ R_{\text{axis}}(\text{rot}),$$

缺省的因子是 $I$（没写的角为 0、没写 `rot` 为 $I$）。这就是 3.1.3 第 1 步的 $R_{\text{base}} \cdot R_y(\text{turn}) \cdot [\text{aim}] \cdot R_x(\text{tilt}) \cdot R_{\text{roll}}(\text{roll}) \cdot R_{\text{axis}}(\text{rot})$：$[\text{aim}]$ 表示 aim 存在时它左边的乘积整体换成 $A$。矩阵乘法按此顺序从左到右做，实现不得改变结合顺序（浮点结果要逐位一致）。

| 情形 | $R_n$ |
|---|---|
| 没有朝向属性、没有关系或 `at` | $I$ |
| 没有朝向属性、`on` / `under` / `against` / `inside` | $R_{B \to P}$（3.5.7 修正后） |
| 只有 `turn` | $R_{\text{base}} R_y(\text{turn})$ |
| `turn` + `tilt` | $R_{\text{base}} R_y(\text{turn}) R_x(\text{tilt})$ |
| `aim`（可带 `aimwith`） | $A\,R_x(\text{tilt})\,R_{\text{roll}}(\text{roll})\,R_{\text{axis}}(\text{rot})$ |
| `repeat` 节点 | $R_{\text{base}}$ |

### 3.5.7 参照物是镜像时

$\det R_{B \to P} = -1$（$B$ 是镜像副本、镜像副本组或它们的后代，3.9.6）时
$$R_{\text{base}} = R_{B \to P}\,\mathrm{diag}(-1, 1, 1),$$
即把 $B$ 的局部 $x$ 轴反向之后再继承：$R_{\text{base}}$ 仍是旋转（$\det = +1$），$n$ 的 $y$、$z$ 轴与 $B$ 的一致，$x$ 轴相反。含义："只继承朝向，不继承手性"——放在镜像物体上的东西不会自己变成镜像（楔形的高边不会翻）。对一个非镜像的 $B$ 这条不起作用。

小例子（用原型 `orientation` 确认）：`box a size=2,1,2 turn=30 at=5,0,0`，`mirror a2 of=a across=z`，`box b on=a2`：$R_{a2}$ 的列为 $\hat x \mapsto (0.866, 0, 0.5)$、$\hat z \mapsto (0.5, 0, -0.866)$，$\det = -1$；$b$ 的 $R$ 的列为 $\hat x \mapsto (-0.866, 0, -0.5)$、$\hat z \mapsto (0.5, 0, -0.866)$，$\det = +1$。

## 3.6 依赖、循环与后置

### 3.6.1 依赖图

求解的对象是两类量：每个节点的**局部变换** $L(n) = T_n$ 与**局部包围盒** $X(n) = B_n$。依赖图 $G$ 的顶点是全部节点的 $L(\cdot)$ 与 $X(\cdot)$；"$u$ 依赖 $v$"记 $u \to v$，边按下面的规则**精确地**由计算过程读到的量给出（读到才有边；不读不加边）：

| 量 | 依赖 |
|---|---|
| $T_{B \to F}$（3.1.1） | 两条链上每个节点的 $L$：$L(b_1), \dots, L(b_m)$、$L(f_1), \dots, L(f_k)$；$B$ 是集合时 $= W_F^{-1}$，依赖 $F$ 到根的链上每个节点的 $L$ |
| 锚点在 $F$ 里的点与法线（3.2.4） | $T_{B \to F}$；方向词锚点与 `B[u,v,w]` 还依赖 $X(B)$；曲面锚点依赖 $X(B)$ 与 $B$ 的几何参数；曲线取点依赖 $B$ 的采样点列（$B$ 是 line / bezier / spline 时即它 `path` 里各点的依赖） |
| 集合的盒（3.2.8） | 每个成员 $m$ 的 $X(m)$ 与 $W_m$（$m$ 到根的链上每个 $L$） |
| 度量 `B.w/h/d`（3.2.6） | $X(B)$ |
| 坐标分量 `锚点.x`（3.2.7） | 该锚点在求值框架里的点的依赖 |
| `let`、模板参数的值 | 它们的表达式按自己的上下文（01-syntax.md 的 1.5.8）求值时读到的量 |
| $L(n)$，一般节点（3.1.3） | 主定位关系的锚点（$q$、$\nu$）与 $R_{B \to P}$；$\pi$ 不是 `origin` 时 $X(n)$（缺省定位点在有关系或有 `pin` 时都不是 origin，3.4.1）；`against` 没有 `pin` 时 $X(n)$；`align` 时 $X(n)$、目标的盒心 / 面锚点；`aim` 的目标点；属性值里引用的量 |
| $L(n)$，阵列实例（3.8） | 阵列节点 $R$ 的 `step` `center` `turn` `n`；`along` 时曲线 $C$ 的取点与 $T_{C \to R}$ |
| $L(n)$，副本式（3.9.1） | $T_{A \to P}$（$A$ 是原件）；镜像面 / 旋转轴的量（3.9.3、3.9.4） |
| $L(n)$，副本组（3.9.2） | 容器 $C$ 的镜像面 / 旋转轴的量（在 $C$ 里求） |
| $L(n)$，影子（3.9.7） | 原件（或其孪生体，04-derivation.md 的 4.4.4）的 $L$ |
| $L(n)$，变体皮肤（3.7.6） | 宿主 $N$ 的 $X(N)$ 与皮肤自己的 $X(K)$ |
| $L(n)$，span（3.10.1） | `from`、`to` / `drop` 的点与法线、`dir` |
| $L(n)$，sweep / pipe / line / bezier / spline | 无（$\mathbb 1$） |
| $X(n)$，实体 | 自己的几何参数（它们的表达式里引用的量）；sweep / pipe 还依赖路径点或曲线 $C$ 的采样点列与 $T_{C \to P}$；span 依赖 $L(n)$ 用到的端点 |
| $X(n)$，曲线 | arc / helix：参数；line / bezier / spline：`path` 各点 |
| $X(n)$，容器（02-geometry.md 的 2.6、2.5.4） | 每个计入的子节点 $c$ 的 $L(c)$ 与 $X(c)$（group / union / repeat 节点 / 容器式 mirror、rotate / 模板实例 / copy / 阵列实例：全部子节点，曲线在布尔容器里不计；subtract：第一个操作数；intersect：全部操作数）；段为 `skin` 的子节点一律不计（3.7.6）；有 `envelope` 的 group：无 |
| $X(n)$，副本式节点、copy | 源的 $X$ |
| $X(n)$，退化节点 | 无（零盒） |

$W_n$ 不是顶点：它由根到 $n$ 的 $L$ 复合而成（3.1.1），"依赖 $W_n$"即依赖链上每个 $L$。

### 3.6.2 不能引用自己、祖先或后代

参照物类型与点类型的属性（`on` `under` `against` `inside` `at` `alignto` `aim` `from` `to` `drop` `about` `across`、`along`、路径点）引用的节点不得是 $n$ 自己、$n$ 的祖先或 $n$ 的后代；集合里含这三类成员同样不行。报错"`<路径>` 不能相对自己定位"（自己或后代）、"`<路径>` 不能相对包含自己的对象定位"（祖先；原型（`ref`）只检查自己与后代，祖先经 3.6.3 成为后置，本版直接报错）、"`<路径>` 的 `<属性>`：集合里包含它自己或它的上下级"（集合），节点退化。原型的 `ref` 只对参照物类型的属性与 `drop` / `along` 检查自己与后代；`aim` `from` `to` `about` `across` 与路径点引用自己或后代时原型不检查，经 3.6.5 的环报"循环依赖"（结果同样是该节点退化），本版统一报本节的文案。

- 引用祖先的锚点（`box a on=g.top`，$g$ 是 $a$ 的父节点）看似"把自己放在容器顶上"，但容器的盒由 $a$ 决定（02-geometry.md 的 2.6），没有不循环的解释；写 `at=g.origin` 一类不读盒的锚点也一律报错，不做特例。
- 引用祖先的**度量或分量**（`size=g.w,1,1`）不属于本节（不是定位引用），按 3.6.3 处理：$X(a) \to X(g) \to X(a)$ 使 $a$ 后置，$g$ 的盒不含 $a$，$a$ 读到的 `g.w` 是不含它自己的盒宽（可以为 0）。
- `mirror R of=A` / `rotate R of=A` / `copy R of=A` 的 $A$ 不得是 $R$ 自己或 $R$ 的祖先（04-derivation.md 报错）。

### 3.6.3 后置的判定

容器 $A$ 的盒 $X(A)$ 由子节点算出（3.6.1 最后几行），而子节点的位置又可能依赖 $A$ 自己的位置或盒（例如子节点 `on=` 容器外的物体、`on=` 集合、`aim=` 容器外的点、读 `A.w`）。这种子节点叫**后置**（late）：它不计入 $A$ 的盒，在 $A$ 定位之后再求解。判定规则：

1. 对每个由子节点算盒的容器 $A$（有 `envelope` 的 group 不算；退化节点不算），按**节点顺序**（01-syntax.md 的 1.6.1：父容器先于子容器处理）依次处理；对 $A$ 的每个计入盒的子节点 $c$ 按节点顺序依次判定：
2. 若在**当前的** $G$ 里从 $\{L(c), X(c)\}$ 出发沿边可达 $X(A)$，则 $c$ 是 $A$ 的后置子节点：标记 `late`，并从 $G$ 删去边 $X(A) \to L(c)$ 与 $X(A) \to X(c)$。
3. 全部容器处理完后得到最终的 $G'$ 与后置集合；3.6.5 在 $G'$ 上判循环。

性质与例子（三条都用原型编译的小例子确认与原型一致）：

- `group radar on=ground[…] { cyl base; cyl mast on=base; revolve dish on=mast aim=sat; span feed from=dish.bottom to=dish.top; sphere horn at=dish.top }`，sat 在 radar 之外：$L(\text{dish}) \to T_{\text{sat} \to \text{radar}} \to L(\text{radar}) \to X(\text{radar})$（radar 有关系，读自己的盒做定位点），所以 dish 后置；feed、horn 经 $L(\text{dish})$ 同样可达，也后置；base、mast 不后置。radar 的盒 = base ∪ mast（rotation.s3d 的参考输出：这三个节点 `late`，radar 的 `box` 高 9）。
- 嵌套：`group g on=floor { group h { box a on=#wall } box b }`。先处理 $g$：$X(h) \to L(a) \to L(h), L(g)$（集合的框架是世界，$W_h^{-1}$ 含两者），$L(g) \to X(g)$（$g$ 有关系，读自己的盒做定位点），所以 $h$ 后置，删去 $X(g) \to L(h)$ 与 $X(g) \to X(h)$。再处理 $h$：从 $L(a)$ 出发经 $L(g) \to X(g)$ 只到得了 $L(b)$、$X(b)$（到 $h$ 的两条边已删），$L(h)$ 没有关系与 pin、不读 $X(h)$，所以到不了 $X(h)$：$a$ **不**后置（$g$ 的盒只含 $b$，$h$ 的盒含 $a$）。若按相反的顺序先处理 $h$，就会经 $L(a) \to L(g) \to X(g) \to X(h)$ 把 $a$ 判成后置而 $h$ 不后置，与原型不同；本版不这样做。若把 $h$ 改成 `group h on=floor { … }`，则 $h$ 因 $L(h) \to T_{\text{floor} \to g} \to L(g) \to X(g)$ 后置，之后 $L(a) \to L(h) \to X(h)$，$a$ 也后置（$h$ 的盒为零盒）；若反过来只给 $h$ 写关系（`group g { group h on=floor { … } box b }`），$L(g)$ 不依赖任何量，$h$ 不后置、$a$ 后置。
- 父容器先判：一条经过多层容器的环（上例的 $X(g) \to X(h) \to L(a) \to L(g) \to X(g)$）在最外层被打断——外层容器的那个子节点后置、边被删，内层容器的判定在删边之后做，所以内层不再因同一条环把自己的子节点也判成后置。
- 后置不是错误，不产生诊断；但它改变容器的盒（因此改变容器的定位点、锚点、`w/h/d`、事实里容器的范围，02-geometry.md 的 2.6），输出里用 `late` 标出（3.6.4）。
- 阵列实例是阵列节点的子节点，同样可以后置：`repeat r n=4 along=c on=floor { … }` 里 $L(\text{实例}) \to T_{c \to r} \to L(r) \to X(r)$，全部实例后置，阵列的盒是零盒，`on=floor` 把阵列的原点放到 floor 顶面中心。
- 变体皮肤 `skin` 不经本节判定：它不计入宿主的包络（3.6.1 的 $X$ 行没有到它的边），`late` 由 3.7.6 直接规定。

原型（`Solver.box` 的 `Cycle` 捕获与 `localBusy` 标记）用惰性求值的"忙"标记在求值时顺带判定后置：环在哪个容器的盒的求值里闭合，就标记该容器的那个子节点，结果依赖求值顺序——它按节点顺序求世界变换，通常外层容器先开始求值，与本节一致；但当外层容器之前的某条语句先读到了内层容器的盒（上例前面加一句 `box z size=g.h.w,1,1`），环在 $h$ 的盒的求值里闭合，原型标记 $a$ 而不是 $h$（小例子见 3.11.4）。本版只按本节的图规则，不看谁先被读到。两者在语料的全部节点上一致（3.11.3）。

### 3.6.4 后置的输出与求解时机

- 后置子节点 $c$ 的 $L(c)$、$X(c)$ 照常求解，只是排在 $L(A)$（与 $X(A)$）之后；$W_c = W_A \circ L(c)$ 不变。它照常参与空间事实、选择器、语义层，照常渲染。
- 输出里每个节点带 `late`（布尔）：3.6.3 判为后置的为真，其余为假；变体皮肤按 3.7.6，影子按 3.9.7（取原件的）。后置是子节点相对**其父容器**的属性，不向上传播（孙节点后置不使父节点 `late`）。
- 后置节点的子树照常：它自己的子节点按 3.6.3 对**它**判定。

### 3.6.5 循环依赖

3.6.3 之后的图 $G'$ 必须无环。有环时，环上（任何一个非平凡强连通分量里）的每个 $L(\cdot)$ 或 $X(\cdot)$ 所属的节点都报错"循环依赖：`<路径>` 的位置最终依赖于它自己"，并成为退化节点（01-syntax.md 的 1.1.4：$L = \mathbb 1$、零盒）；依赖这些节点而自身不在环上的节点照常求解（用退化值）。错误按 1.1.5 排序；同一个节点只报一条。

- 例：`box x on=y`、`box y on=x`：$L(x) \to L(y) \to L(x)$，x、y 都退化。原型（`compileOnce` 的 `fail`）只把先碰到"忙"标记的那个节点（x）退化、报一条错，y 放在退化的 x 上；本版两者都退化。
- 例：`let a = b.w`、`box b size=a,1,1`：$X(b) \to a \to X(b)$，b 退化（`let` 的循环见 01-syntax.md 的 1.5.8，这里的环经过了节点，按本节报）。
- 自环（$L(n) \to L(n)$ 经一条边，如 `at=` 引用自己）在 3.6.2 已经报错，不到本节。
- 退化后 3.6.3 的后置判定不重做；输出以第一次判定为准。

### 3.6.6 求解顺序与确定性

- 实现按 $G'$ 的任一拓扑序求解即可：每个量的值是其依赖的值的确定函数（本章的公式），与拓扑序的选择无关；因此并行求解合法，只要每个量按本章的运算顺序计算。
- 源码顺序只在两处起作用：节点顺序决定 3.6.3 的处理顺序（因而决定罕见情形下谁后置），以及错误的排序（01-syntax.md 的 1.1.5）。前向引用、跨文件引用、引用后面才展开的 copy 或副本都合法。
- $T_{B \to F}$ 必须按 3.1.1 的两条链计算，不得用 $W_F^{-1} \circ W_B$ 代替：两种算法的结果在 09-compat.md 的 9.4.2 的容差内相同（语料里最大偏差 $10^{-12}$ 量级，3.11.1），但不逐位相同（原型 `toAncestor` 从节点向上逐个左乘，即链从右到左结合；本版规定从左到右结合，两者也只在逐位上不同）。09-compat.md 的 9.4 区分三种比较：两份独立实现互比、实现对参考输出（规范化后）比都用 9.4.2 的容差，而**同一实现**在不同平台上必须逐位一致（1.1.3）——所以同一实现不得在两种算法之间按平台或并行度切换。

## 3.7 容器

容器指 group、union、subtract、intersect、repeat 节点、容器式 mirror / rotate 节点、模板实例、copy 节点、阵列实例节点；副本组 `mirrored` / `r`$k$ 的定位见 3.9.2，皮肤见 3.7.6。

### 3.7.1 容器的盒与包络

容器的局部包围盒 $B_A$ 按 02-geometry.md 的 2.6（子节点盒之并，后置子节点不计，段为 `skin` 的皮肤不计，曲线计入）、2.5.4（布尔容器）与 2.6 的 `envelope`（只有 group 接受）。本章把它叫容器的**包络**：

- 没有 `envelope`：包络 = 子节点在容器局部系里的盒之并的轴对齐盒，即**内容决定包络**；
- 有 `envelope`：包络 = 声明的盒（01-syntax.md 的 1.1.2 的"外包络"），**与内容无关**，内容可以超出或空着。

关系、锚点、`pin`、`align`、`w/h/d`、作为参照物时的面都按包络；空间事实与渲染按内容（06-facts.md）。

### 3.7.2 容器的定位

容器是一般节点（3.1.2），定位与朝向完全按 3.3–3.5，把"局部包围盒"换成包络：`on=B` 把包络的底面中心放到 $B$ 的顶面中心；`pin=origin` 把容器的原点放到落点；`inside=B` 把包络的盒心放到 $B$ 的盒心；`turn` 绕落点转整个容器。

- `group rocket on=pad { cyl stage1 …; cyl stage2 on=stage1 … }`：rocket 的包络是两级之并（底面中心在 stage1 底面中心），rocket 的原点正好在那里，所以 rocket 的 $t$ = pad 顶面中心。
- `group sat at=-60,58,-72 turn=30 tilt=20 { box body size=2,2,3 pin=center; box wing … against=body.right; mirror wing2 of=wing across=body axis=x }`：包络 $[-7,7] \times [-1,1] \times [-1.5,1.5]$，$\pi = (0,-1,0)$，$R = R_y(30°)R_x(20°)$，$t = (-60,58,-72) - R\pi = (-59.8290, 58.9397, -71.7038)$（参考输出一致）。写 `pin=origin` 才让 $t = (-60,58,-72)$。
- `repeat lights n=(2,1,2) step=40,0,40 center=true at=pad.bottom pin=origin`：阵列节点的原点钉在 pad 底面中心；不写 `pin=origin` 则是阵列包络（四根灯塔之并）的底面中心。

### 3.7.3 子节点与容器的先后

- 子节点的局部变换 $L(c)$ 在**容器的局部系**里求，不需要知道容器放在哪（除非它引用了容器外的量，3.6.3）；容器的包络由这些 $L(c)$、$X(c)$ 算出；容器再按包络定位。所以顺序是：子节点 → 包络 → 容器的 $L(A)$ → 子节点的 $W_c = W_A \circ L(c)$。
- 容器的 `turn` / `tilt` / `roll` / `rot` / `aim` 转的是整个容器（连同全部子节点），绕容器的定位点（3.3.9）；子节点自己的朝向属性是相对容器局部系的。
- 容器的 `mat` / `color` 向子节点继承（10-materials.md）；定位属性不继承。
- 修改容器的包络（加减子节点、改子节点尺寸）会移动容器的落点（因为定位点随包络变），这正是 `envelope` 存在的理由：声明包络后，细化容器内部不会移动挂在它上面的东西。

### 3.7.4 空容器

没有任何计入的子节点（没有子节点，或子节点全部后置）的容器：包络是零盒 $Z$（02-geometry.md 的 2.6）。退化的子节点以零盒计入——它的 $T = \mathbb 1$（01-syntax.md 的 1.1.4），零盒落在容器的原点，所以只含退化子节点的容器的包络也是 $Z$。包络是 $Z$ 时所有定位点都是原点，`on=B` 把原点放到 $B$ 的顶面中心，`inside=B` 放到 $B$ 的盒心，锚点全在原点，`w = h = d = 0`。空容器不报错（空块合法，01-syntax.md 的 1.3.15）；布尔容器没有操作数时按 02-geometry.md 的 2.5.1 报错并退化。

### 3.7.5 各种容器的包络

| 容器 | 包络 | 说明 |
|---|---|---|
| group | 子节点之并；有 `envelope` 则为声明值 | 2.6 |
| union | 全部操作数的盒之并 | 2.5.4；曲线不计 |
| subtract | 第一个操作数的盒 | 2.5.4；被挖的部分仍在包络里 |
| intersect | 操作数盒之交（空则零盒） | 2.5.4 |
| repeat 节点 | 全部（非后置）实例的盒之并 | 3.8.5 |
| 阵列实例 | 其子节点之并 | 它是一个容器（1.6.2） |
| 容器式 mirror / rotate 节点 | 原件与全部副本组的盒之并 | 3.9.2 |
| 副本组 `mirrored` / `r`$k$ | 子节点之并：影子（有覆盖的影子按孪生体的盒，3.9.7）与经 `over` 块加入的真实子节点（04-derivation.md 的 4.4.4）；没有覆盖、没有加入时等于原件之并（在副本组自己的局部系里） | 3.9.2；02-geometry.md 的 2.6 |
| 副本式 `mirror R of=A` | 等于 $A$ 的盒（在 $R$ 的局部系里） | 2.6；$R$ 的局部系是 $A$ 的局部系经 $M$ 映射 |
| 模板实例 | 体展开出的子节点之并 | 04-derivation.md 的 4.2.3 |
| copy 节点 | 复制出的内容之并（与源语句重新展开的结果一致） | 04-derivation.md 的 4.3 |

### 3.7.6 变体皮肤

变体（07-semantics.md 的 7.8）给被选中的宿主节点 $N$ 挂一个段为 `skin` 的子节点 $K$（模板实例，04-derivation.md 的 4.7 第 2 条）。$K$ 的局部变换由本节固定，不走 3.1.3 的一般规则，也不接受定位属性（`at`、`pin` 在 variant 上报错，01-syntax.md 的 1.3.13）：

$$R_K = I,\qquad t_K = b_N - \pi_K,\qquad b_N = (c_{N,x},\ \min_{N,y},\ c_{N,z}),$$

$b_N$ 是宿主 $N$ 的**包络**（3.7.1：实体的局部包围盒、容器的子节点之并、有 `envelope` 的 group 用 envelope；退化节点或空容器为零盒，$b_N = 0$）的底面中心，$\pi_K$ 是 $K$ 自己的局部包围盒（体展开出的子节点之并，02-geometry.md 的 2.6）的底面中心。即皮肤的**局部包围盒底面中心**放到宿主包络的底面中心，轴向与 $N$ 相同（$K$ 是 $N$ 的子节点，所以跟随 $N$ 的世界朝向）；皮肤的尺寸由模板按自动传入的 `w` `h` `d`（$N$ 包络的边长，07-semantics.md 的 7.8.3 第 3 条）决定，不强制等于宿主。这等价于给皮肤语句写 `at=N.bottom pin=bottom`（原型 compileOnce 正是这样构造）；这条对祖先 $N$ 的引用由本节规定，不受 3.6.2 的限制。04-derivation.md 的 4.7 第 5 条是同一条公式。

**依赖。** $L(K)$ 依赖 $X(N)$ 与 $X(K)$（3.6.1）；但 $K$ **不计入** $N$ 与 $N$ 任何祖先的包络（02-geometry.md 的 2.6；04-derivation.md 的 4.7），所以 3.6.1 里没有 $X(N) \to L(K)$ 的边，皮肤不会使依赖图成环，也不经 3.6.3 判定——变体不改变任何非皮肤节点的变换、包围盒与事实（07-semantics.md 的 7.8.4）。

**后置标记。** 皮肤节点 $K$ 的 `late` 由下面的可判定规则直接给出：**$N$ 的包络按 3.6.1 的 $X(N)$ 行要读到它的全部子节点时 $K$ 为 `late`，否则不为 `late`**。即：

| 宿主 $N$ | $K$ 的 `late` |
|---|---|
| 没有 `envelope` 的 group、union、intersect、repeat 节点、阵列实例、容器式 mirror / rotate 节点、模板实例、copy 节点 | 真 |
| 实体、曲线、有 `envelope` 的 group、subtract（只读第一个操作数）、退化节点 | 假 |

这个标记只是输出字段，不改变任何量：$K$ 的子树照常求解，$K$ 自己的子节点按 3.6.3 对 $K$ 判定。规则的来源：原型在求容器盒时遍历全部子节点、遇到读自己盒的子节点就标 `late`，皮肤的 `at=N.bottom` 正是这样的子节点，而实体、有 envelope 的 group 与 subtract（只遍历第一个操作数）求盒时不遍历到皮肤，于是不标。参考输出与此一致：semantics.s3d 的 `slab.skin`（宿主是 box）与 spaceport_blueprint.s3d 的 `hangar.roof.skin`（宿主是 box）`late` 为假，semantics.s3d 的 `street.h4.skin`（宿主是模板实例）`late` 为真；用原型编译的小例子（3.11.4）确认了表里其余各行。

参考输出的数值：`street.h4.skin` 的世界 $t = (43, 0, 8)$ = h4 的包络底面中心（h4 的门向前凸 0.2，使盒心 $z$ 比 h4 的原点多 0.1）；`slab.skin` 的 $t = (0, 5.05, 0)$ = slab 的底面中心；`hangar.roof.skin` 的 $t = (-45, 22, -20)$ = roof 的底面中心。

## 3.8 repeat

`repeat` 语句生成一个**阵列节点** $R$（一般节点，按 3.7.2 定位，$R_R = R_{\text{base}}$，3.5.1）和它的子节点——**阵列实例**（段是扁平下标，01-syntax.md 的 1.6.2；种类 `instance`）；块里的语句在每个实例下各展开一份（04-derivation.md）。本节定义实例的个数、下标与局部变换。专有属性见 01-syntax.md 的 1.4.5：`n`（常量，必填）、`step`（向量₃，缺省 0）、`turn`（数，缺省 0）、`center`（布尔，缺省 false）、`along`（曲线）、`orient`（枚举 `yaw` `full` `none`，缺省 `yaw`）。

### 3.8.1 n 与实例数

`n` 是常量（展开时求值，可以引用 let、模板参数、外层阵列的 `i j k idx`、`metre`，不能引用对象）：

- 一个数 $\nu$：**线性阵列**，$n_i = \nu$，$n_j = n_k = 1$。
- 向量₂ $(\nu_i, \nu_j)$ 或向量₃ $(\nu_i, \nu_j, \nu_k)$：**网格阵列**，缺省的 $n_k = 1$。
- 每个分量必须与某个整数 $m \ge 0$ 的差 $< \varepsilon_{\text{int}} = 10^{-9}$，取 $m$；否则报错"`<路径>` 的 n 需要非负整数（得到 X）"，节点退化、不生成实例（原型（`elaborate`）对小数四舍五入、负数取 0；本版报错）。
- 实例总数 $N = n_i n_j n_k$。$N = 0$ 合法：没有实例，阵列包络为零盒，不报错。$N > N_{\max} = 5000$ 报错"repeat 数量过多（N）"，节点退化、不生成实例。
- 其他类型（字符串、引用、4 个分量……）报错"`<路径>` 的 n 需要数或 n_i,n_j,n_k"。

线性与网格是两种不同的变换规则（3.8.3）：`n=3` 与 `n=(3,1,1)` 的实例位置**不同**，前者沿 `step` 向量排列，后者只用 `step` 的 $x$ 分量。

### 3.8.2 下标与实例段

实例按 $k$ 外层、$j$ 中层、$i$ 内层的顺序生成：
$$idx = i + n_i\,(j + n_j\,k),\qquad 0 \le i < n_i,\ 0 \le j < n_j,\ 0 \le k < n_k,$$
实例的段是 $idx$ 的十进制数字串；节点顺序按 $idx$ 递增（01-syntax.md 的 1.6.1）。线性阵列的 $(i, j, k) = (idx, 0, 0)$。

实例体内的词法环境（阵列实例帧，01-syntax.md 的 1.5.8）绑定 `i` `j` `k` `idx` 四个数；嵌套阵列的内层帧遮蔽外层的同名变量。被 `drop` 的实例留下空洞，其余实例的下标与位置不变（1.6.2）。

### 3.8.3 网格阵列与线性阵列的实例变换

没有 `along` 时，实例 $(i,j,k)$ / $idx$ 的局部变换（在 $R$ 的局部系里）：

$$R_{\text{inst}} = R_y(\text{turn} \cdot idx),\qquad t_{\text{inst}} = \begin{cases} \big((i - \gamma_i)\,s_x,\ (j - \gamma_j)\,s_y,\ (k - \gamma_k)\,s_z\big) & \text{网格} \\ (idx - \gamma_i)\,s & \text{线性} \end{cases}$$

$s = $ `step`，$\gamma_a = (n_a - 1)/2$ 当 `center=true`，否则 $0$（$a \in \{i,j,k\}$；网格阵列没写的 $n_k$ 按 1 算，$\gamma_k = 0$）。

- **网格**：三个下标各走 `step` 的一个分量；`step=w,0,w`、`n=(2,1,2)` 是 $x$、$z$ 各两列。
- **线性**：下标走整个 `step` 向量，所以可以沿斜线排（`repeat farm n=3 step=26,0,-8`）。
- `center=true` 把阵列对称到原点两侧（下标中心 $\gamma$ 落在原点）；`turn` 给第 $idx$ 个实例绕它自己原点的 $+y$ 转 $\text{turn} \cdot idx$ 度（螺旋楼梯：`step=0,h,0 turn=15`）。
- 实例自己没有属性（1.6.2），不能写定位；要让每个实例的内容按下标变化，在体内用 `i j k idx`（`let s = 0.8 + 0.12*i`、`turbine t phase=idx*40`）。
- 阵列节点的 `turn` 不转阵列本身（3.5.1）；要整体转，把 repeat 放进带 `turn` 的 group。

### 3.8.4 沿曲线：along 与 orient

`along=C`（$C$ 是曲线，01-syntax.md 的 1.4.1；写了 `along` 时 `step` / `center` 不得写，报错"`<路径>`：along 与 step / center 不能同时写"；`orient` 只能与 `along` 一起写，否则报错"`<路径>`：orient 只能和 along 一起写"。原型忽略这些组合；本版报错）：

1. 曲线参数：$N$ 是实例总数（3.8.1，`n` 通常是一个数；网格也允许，按 $idx$ 排）；闭合曲线 $s = idx / N$（首尾不重合），开放曲线 $s = idx / (N - 1)$（$N = 1$ 时 $s = 0$；两端各一个实例）。
2. 取点 $C[s]$（02-geometry.md 的 2.3.1）换到 $R$ 的局部系：$p = T_{C \to R}(C[s])$，切线 $\tau = R_{C \to R}\,\hat T(s)$（单位向量）。$C$ 可以在树的任何位置。
3. 朝向 $M$ 按 `orient`：
   - `yaw`（缺省）：$h = \sqrt{\tau_x^2 + \tau_z^2}$；$h > \varepsilon_{\text{yaw}} = 10^{-9}$ 时 $M = R_y(\psi)$，$\psi = \mathrm{atan2}(-\tau_z, \tau_x)$（度），使局部 $+x$ 的水平投影沿切线的水平投影；否则（竖直段）$M = I$。实例保持竖直。
   - `full`：$\sigma = \tau \times \hat y$；$\lVert \sigma \rVert > \varepsilon_{\text{dir}} = 10^{-6}$ 时 $\text{side} = \mathrm{unit}(\sigma)$，否则 $\text{side} = \hat z$；$M = [\,\tau\ \ \text{side} \times \tau\ \ \text{side}\,]$（列）：局部 $x$ 沿切线、$z$ 水平朝曲线外侧（俯视逆时针的弧的外侧）、$y$ 是两者的法向，与扫掠框架（02-geometry.md 的 2.3.7）的 $(T, \text{up}, \text{side})$ 一致，但竖直段的 side 取固定的 $\hat z$（扫掠框架沿用前一段的 side）。
   - `none`：$M = I$。
4. $$R_{\text{inst}} = M\,R_y(\text{turn} \cdot idx),\qquad t_{\text{inst}} = p .$$

- 实例原点落在曲线上，所以体内物体用自己的定位决定相对曲线的位置：`cyl pole … at=0,0,6` 在 `yaw` 下沿局部 $+z$（曲线外侧）偏 6；`box tread pin=top` 把踏面顶放到螺旋线上。
- 曲线自身的朝向（arc 的 `turn` 等）在 $T_{C \to R}$ 里；曲线是镜像副本时 $\tau$ 随之反射。
- 参数按**采样折线的弦长**（arc / helix 用解析式）均分，不是按真实弧长。

**示例**（curves.s3d）：`arc loop r=40 angle=-60,240`，`repeat lamps n=11 along=loop`。开放弧，$N = 11$；$idx = 3$：$s = 0.3$，$\theta = -60 + 300 \cdot 0.3 = 30°$，$p = (40\cos 30°, 0, -40\sin 30°) = (34.6410, 0, -20)$，$\tau \propto (-\sin 30°, 0, -\cos 30°)$，$\psi = \mathrm{atan2}(\cos 30°, -\sin 30°) = 120°$，$M = R_y(120°)$——参考输出 `lamps.3` 的 $t$、$R$ 一致。rotation.s3d 的 `repeat ties n=110 along=track orient=full`（闭合样条，$s = idx/110$）与 `repeat legs n=20 along=track`（yaw）全部 130 个实例与参考输出一致（3.11）。

### 3.8.5 阵列节点与实例的盒和定位

- 实例节点是容器：盒 = 其子节点之并（02-geometry.md 的 2.6），后置子节点不计。
- 阵列节点 $R$ 的包络 = 全部非后置实例的盒经 $T_{\text{inst}}$ 变换后的并（3.7.5）；$R$ 按 3.7.2 定位（`on=river` 把阵列包络的底面中心放到 river 顶面；`at=slab.top pin=origin` 把阵列原点放到 slab 顶面中心）。
- 实例的 $W = W_R \circ T_{\text{inst}}$；体内节点相对实例定位。
- `along` 的曲线在 $R$ 的父框架或更外层时，$T_{C \to R}$ 含 $L(R)$；若 $R$ 又有定位关系或 `pin`（读自己的包络），全部实例后置、$R$ 的包络为零盒（3.6.3）。要避免，让曲线成为阵列体外、阵列**不**写关系（语料的写法），或给阵列 `pin=origin at=…`。

### 3.8.6 上限与错误

| 情形 | 处理 |
|---|---|
| `n` 缺省 | 报错"`<路径>` 缺少 n"（01-syntax.md 的 1.4.5），节点退化（原型 `elaborate` 缺省 1 个实例；3.11.5） |
| `n` 不是常量（引用对象、用了 where） | 报错"`<路径>` 的 n 必须是常量"，节点退化 |
| 分量不是非负整数（容差 $\varepsilon_{\text{int}}$） | 报错，节点退化 |
| $N > N_{\max} = 5000$ | 报错"repeat 数量过多（N）"，节点退化 |
| `along` 不是曲线 | 报错"`<路径>` 的 along 需要一条曲线（line / bezier / spline / arc / helix）" |
| `along` 与 `step` / `center` 同写；`orient` 没有 `along` | 报错（3.8.4），节点退化 |
| `orient` 不是 `yaw` / `full` / `none` | 报错"`<路径>` 的 orient 只能是 yaw / full / none"（原型还接受 `true` / `false`；本版按 01-syntax.md 的 1.4.3 只接受三个枚举值） |
| `center` 不是 `true` / `false` | 报错（01-syntax.md 的 1.4.3） |
| 阵列节点上写 `tilt` `roll` `rot` `aim` `aimwith` | 报错"repeat 没有属性 X"（01-syntax.md 的 1.4.4） |

节点退化时不生成任何实例（体不展开），路径 `r.0` 等不存在，引用它们按"找不到对象"报错。

## 3.9 mirror / rotate

两种形式（01-syntax.md 的 1.3.3、1.6.3）：**副本式** `mirror R of=A …` / `rotate R of=A …` 生成一个跟随 $A$ 的节点 $R$；**容器式** `mirror C … { 原件 }` / `rotate C n=k … { 原件 }` 在容器 $C$ 下生成副本组 `mirrored` / `r1 … rk`。副本树的生成、影子的属性与修改见 04-derivation.md 的 4.4；本节只定义变换。记 $M$ 为**副本变换**（镜像：反射；旋转：绕轴旋转），在框架 $F$ 里表达。

### 3.9.1 副本式：of=

$R$ 的父节点是 $P$（写 `mirror R` 的块的节点），原件 $A$ 可以在树的任何位置（不得是 $R$ 或 $R$ 的祖先）。
$$T_R = M \circ T_{A \to P},$$
$M$ 在 $P$ 里求（镜像面 3.9.3、旋转轴 3.9.4，$F = P$）。含义：$R$ 的局部系是 $A$ 的局部系先换到 $P$、再经 $M$ 映射；$R$ 的局部几何与局部盒等于 $A$ 的（02-geometry.md 的 2.6），所以 $R$ 的世界几何 = $A$ 的世界几何经 $W_P \circ M \circ W_P^{-1}$（$P$ 里的那个反射 / 旋转）映射。

- $R$ 的子节点是 $A$ 的子节点的影子，$T = $ 原件的 $T$（3.9.7）；于是整棵副本树随 $A$ 一起被映射。
- $R$ 不接受定位与朝向属性（`at` `on` `pin` `turn` …，报错"副本跟随原物体，不接受定位属性"，01-syntax.md 的 1.4.4；原型（`local` 的 `n.src` 分支）静默忽略；本版报错，节点退化）。只接受 `mat` / `color` 与自己的专有属性（`across` `axis`；`about` `axis` `angle`）。
- rotate 副本式：$k = 1$，`angle` 缺省 180；`n` 不合法（1.4.5）。
- $A$ 是 span / sweep / pipe / 曲线 / 容器 / 阵列 / 模板实例 / 另一个副本都可以；$A$ 是集合或带锚点的引用报错"`<路径>`：of 需要写一个对象名"。

**示例**（spaceport.s3d）：`mirror tower2 of=tower across=rocket axis=x`，rocket 是没转过的组、盒心 $x = 25$（在根里）。$M$：过 $(25, c_y, c_z)$、法线 $\hat x$ 的反射；tower 的 $t = (15.9, 2, -9.1)$ → tower2 的 $t = (34.1, 2, -9.1)$，$R = \mathrm{diag}(-1, 1, 1)$（参考输出一致）。

### 3.9.2 容器式：副本组

容器 $C$ 是一般节点（按 3.7.2 定位，包络 = 原件与副本组之并）。第 $k$ 个副本组 $G_k$（mirror 只有 $k = 1$，段 `mirrored`；rotate 有 $k = 1..n$，段 `r`$k$）是 $C$ 的子节点：
$$T_{G_k} = M_k,$$
$M_k$ 在 **$C$ 自己的局部系**里求（$F = C$；旋转的角是 $k \cdot$ `angle`，3.9.4）。$G_k$ 的子节点是原件的影子，$T$ = 原件的 $T$（在 $C$ 里）；所以副本组里每个影子的世界几何 = 原件的世界几何经 $W_C \circ M_k \circ W_C^{-1}$ 映射。

- 原件留在 $C$ 下原位（它们的 $T$ 在 $C$ 里按一般规则求，可以互相引用：`wedge ramp against=house.right`）。
- 副本组自己没有属性、不能被定位（1.6.3）。
- `rotate C n=k`：$n$ 必须是 1 到 360 的整数（容差 $\varepsilon_{\text{int}}$），否则报错"`<路径>`：rotate 的 n 需要 1..360 的整数"（原型四舍五入后检查范围；本版非整数报错），节点退化、不生成副本组。缺省 $n = 1$。
- 镜像面 / 旋转轴的量引用容器外的节点时（`mirror wings across=hangar.body axis=x`），$T_{B \to C}$ 含 $L(C)$；$C$ 自己没有关系与 pin 时没有环（语料的写法）；$C$ 有关系时副本组后置、不计入 $C$ 的包络（3.6.3）。

**示例**（spaceport.s3d）：`rotate fins n=3 about=stage1 { wedge fin … }`：`angle` 缺省 $360/4 = 90$，$M_1$ = 绕过 stage1 盒心 $(0, 14, 0)$、方向 $\hat y$ 的轴转 90°：$R = R_y(90°)$，$t = q - Rq = 0$（$q$ 在轴上）；参考输出 `rocket.fins.r1` 的 $T$ 一致。rotation.s3d 的 `rotate rotor n=2 axis=z { box blade … }`：`angle` $= 120$，$M_1 = R_{\text{axis}}(\hat z, 120°)$，$t = 0$。

### 3.9.3 镜像面：across 与 axis

镜像面由一点 $q$ 与单位法线 $\hat m$（都在 $F$ 里）确定，反射
$$M = \big(I - 2\,\hat m \hat m^{\mathsf T},\ \ 2\,(q \cdot \hat m)\,\hat m\big).$$

`across` 的类型是镜像面规格（01-syntax.md 的 1.4.1），缺省 `x`；`axis` 是方向规格（3.9.5），缺省 $\hat x$：

| `across` | $q$ | $\hat m$ | 说明 |
|---|---|---|---|
| `x` / `y` / `z` | $F$ 的原点 $0$ | $F$ 的该轴 | 过框架原点的坐标面；写了 `axis` 报错"`<路径>`：across 是轴名时不能再写 axis"（原型忽略 axis；本版报错） |
| 向量₃ | 该点（在 $F$ 里） | `axis` 在 $F$ 里的方向（3.9.5；缺省 $\hat x$） | 过给定点 |
| 引用，带**向量非零的盒锚点**（方向词 / 组合词 / `B[u,v,w]`），且**没写** `axis` | 锚点的点 $q$（3.2.4） | 锚点的法线 $\nu_F$（$\mathrm{dominant}$） | "过某个面"：`across=B.right` 是 $B$ 的右面所在平面；`across=B.top-right` 法线取 $\hat y$（3.2.3）；`across=B[0,1,0.5]` 过顶面上的点、法线 $\hat y$ |
| 引用，其他情形（无锚点、`center` / `origin` / 曲线取点 / 曲面锚点，或写了 `axis`） | 锚点的点（无锚点取盒心） | `axis` 在**该引用节点的局部系**里的方向换到 $F$（缺省该节点的 $\hat x$） | "过某物中心、垂直于它的某根轴"：`across=rocket axis=x`；`across=B.right axis=z` 是过 $B$ 右面中心、法线为 $B$ 的 $z$ 轴的面 |
| 其他值 | 报错"across 写成 x / y / z、点坐标，或对象 / 锚点（如 B、B.right、B.top）" | | |

- $\lVert m \rVert < \varepsilon_{\text{len}}$（例如 `axis=(0,0,0)`）报错"`<路径>`：镜像面的法线为零"，节点退化。
- `axis` 是向量₃时直接在 $F$ 里（不随引用节点转）；是方向词 / 轴名时在引用节点的局部系里（没有引用时在 $F$ 里）；是锚点引用时取该锚点的法线或切线（3.9.5）。
- 引用是集合时：盒心是集合盒心，轴是世界轴（3.2.8）。
- 曲线可以作镜像参照（`across=archline axis=x`：过弧的采样折线盒的盒心、法线为弧局部 $x$ 的面）。

小例子（原型 `mirrorIn` 确认）：floor 是 $40\times1\times24$、`pin=top` 的盒，`box q size=2,2,2 at=20,0,20`：`mirror q2 of=q across=floor.right axis=z` 得 $t = (20, 0, -20)$（过右面中心、法线 $\hat z$）；`mirror q3 of=q across=floor.right` 得 $t = (20, 0, 20)$、$R = \mathrm{diag}(-1,1,1)$（过 $x = 20$ 的面）；`mirror q4 of=q across=floor` 得 $t = (-20, 0, 20)$（过盒心、法线 $\hat x$）。

### 3.9.4 旋转：about、axis、angle、n

绕过点 $q$（`about`，点类型，缺省 $F$ 的原点 $0$；引用取锚点或盒心，3.2.9）、方向 $\hat a$（`axis`，方向规格，缺省 $\hat y$，**在 $F$ 里**——轴名与方向词按 $F$ 的轴，不按 `about` 所引用物体的轴）、角 $\theta$（`angle`，数，度）的旋转：
$$M_k = \big(R_{\text{axis}}(\hat a, k\theta),\ \ q - R_{\text{axis}}(\hat a, k\theta)\,q\big),$$
$R_{\text{axis}}$ 见 3.5.3（右手）。`angle` 缺省 $360 / (n + 1)$（$n$ 是容器式的 `n`；副本式 $n = 1$，即 180）：$n$ 个副本加原件均分一圈。

- `angle` 可以为 0 或负、不限范围；`angle=0` 的副本与原件重合，合法。
- $\lVert \hat a \rVert < \varepsilon_{\text{len}}$（`axis=(0,0,0)`）报错"`<路径>` 的 axis 不能是零向量"，节点退化。
- `about` 引用容器外的物体时的后置规则同 3.9.2。

**示例**：curves.s3d 的 `rotate benches n=5 about=basin { … }`：$\theta = 60°$，$q$ = basin 的盒心（在 plaza 里），$\hat a = \hat y$；`rotate gate2 of=gate about=garage angle=180`：$F$ = 根，$q$ = garage 包络的盒心。两者与参考输出一致（3.11）。

### 3.9.5 方向规格

类型"方向规格"（01-syntax.md 的 1.4.1）用于 mirror / rotate 的 `axis` 与 span 的 `dir`。给定使用处的框架 $F$ 与可选的**参照系节点** $Q$（3.9.3 第四行的引用节点；其他情形没有），值 $v$ 求成 $F$ 里的单位向量：

| $v$ | 方向 |
|---|---|
| `x` / `y` / `z` | $\hat x$ / $\hat y$ / $\hat z$，有 $Q$ 时经 $R_{Q \to F}$ 换到 $F$ |
| 方向词 / 组合词（向量 $a$） | $\mathrm{unit}(a)$，有 $Q$ 时经 $R_{Q \to F}$ |
| 向量₃ | $\mathrm{unit}(v)$，直接在 $F$ 里（不经 $Q$） |
| 盒锚点引用 `B.面` / `B.组合词` / `B[u,v,w]`（向量 $a \ne 0$） | $\mathrm{unit}(R_{B \to F}\,a)$——注意用的是锚点的**方向向量**，不是 $\mathrm{dominant}$：`axis=B.top-right` 是 $B$ 里 $(1,1,0)$ 方向 |
| 曲线取点 `C[t]` / `C.start` … | 切线 $\tau_F = R_{C \to F}\,\hat T$（3.2.2） |
| 裸引用、`center` / `origin`、曲面锚点、集合、其他 | 报错"`<路径>` 的 `<属性>` 需要方向：x / y / z、方向词、向量，或锚点如 B.front" |

结果的长度 $< \varepsilon_{\text{len}}$（零向量）报错"`<路径>` 的 `<属性>` 不能是零向量"。缺省值（没写）按各属性：mirror 的 `axis` 是 $\hat x$（有 $Q$ 时是 $Q$ 的 $\hat x$），rotate 的 `axis` 是 $F$ 的 $\hat y$，span 的 `dir` 是 $F$ 的 $(0,-1,0)$。

### 3.9.6 det R = −1

镜像副本（副本式 $R$、副本组 `mirrored`）的 $T$ 有 $\det = -1$，它们的后代经复合也有 $\det W = -1$（偶数次镜像又回到 $+1$）。几何上的含义全部由 02-geometry.md 定义（2.1.1：点集就是局部点集经反射的像，楔形的高边换到另一侧、helix 的旋向反转；2.1.9：三角化按 $\det$ 调整绕序；2.3.8：曲线点列反射）。本章只规定：

- $\det = -1$ 的 $T$ 在复合、求逆、$T_{B \to F}$ 里与旋转同样处理（正交矩阵的一般公式，不特判）；
- 继承镜像参照物的朝向时按 3.5.7 去掉手性；
- 镜像副本自己的 `turn` 等属性不存在（3.9.1），所以不存在"在镜像系里转"的歧义；影子的 $T$ 直接取原件的（3.9.7），在镜像系里解释。

### 3.9.7 影子

副本树里的每个影子 $g$ 对应原件子树里的一个节点 $o$（04-derivation.md 的 4.4.2）。影子不接受定位属性；影子的子节点是 $o$ 的子节点的影子。

**局部变换。**
$$T_g = T_{\tilde o},$$
$\tilde o$ 是 $o$ 的**孪生体**（04-derivation.md 的 4.4.4）：父节点 = $o$ 的父节点，种类、源语句、参数绑定、子节点 = $o$ 的，有效属性 = $o$ 的有效属性被 $g$ 上的影子覆盖（路径经过副本段的几何时机 `over` 写的属性）逐键替换；$g$ 没有覆盖时 $\tilde o = o$，于是 $T_g = T_o$。$T_{\tilde o}$ 在 **$o$ 的父框架**里按 3.1.2 的表求（$o$ 是一般节点时即 3.1.3：关系、pin、gap、align、offset、朝向都按 $\tilde o$ 的有效属性，参照物从 $o$ 所在处解析），$\tilde o$ 不进场景树、不输出、不进任何集合。影子的局部盒同理取 $\tilde o$ 的盒（覆盖了 `size` 的影子有自己的盒；$\tilde o$ 是容器时按 $o$ 的子节点算）。

- 影子的世界变换 $W_g = W_{\text{副本根}} \circ T_{\tilde o_1} \circ \cdots \circ T_{\tilde o_m}$（从副本根到 $g$ 的影子链各取其孪生体的 $T$），即原件的世界几何经副本变换映射（3.9.1、3.9.2）。
- 影子里不再求解关系：原件 `against=house.right` 在镜像里仍是"贴在镜像后的 house 的（镜像后的）右面"，因为整体一起映射；镜像后的楔形坡向翻转就是这个意思（curves.s3d 的 `gate`）。有覆盖时同理：`over tower2.legs.0.leg on=other` 里的 `other` 从 tower 所在处解析，孪生体在 tower 的框架里落到 other 上，再被镜像带到 tower2 那边。
- 依赖：$L(g)$ 依赖 $L(\tilde o)$（3.6.1），$L(\tilde o)$ 的依赖按 $\tilde o$ 的属性与 3.6.1 的一般规则；孪生体不是任何容器的子节点，不参与 3.6.3 的后置判定。
- 影子的后置标记 `late` 取原件 $o$ 的（输出照抄）：副本的包络等于原件的包络（3.7.5），原件里不计入包络的子节点在副本里同样不计入。原型不复制这个标记（影子一律为假）；语料里没有后置节点被复制，参考输出不受影响（3.11.5）。

## 3.10 span / sweep / pipe 的定位

这三种实体的局部系不由定位属性决定（01-syntax.md 的 1.4.4 不给它们定位 / 朝向属性），而由端点或路径决定；几何、局部系的公式在 02-geometry.md 的 2.2.8–2.2.10。本节定义接口：各个点与方向怎样在父框架里求出，以及由此得到的 $T_n$。

### 3.10.1 span 的端点

`from`、`to` 是点（3.2.9），在 span 的**父框架 $P$** 里求值：向量₃直接用；引用取锚点（无锚点取盒心），引用上的偏移在 $P$ 里加（3.2.4）；集合取集合盒心。得到 $A$ 与 $B$，交给 02-geometry.md 的 2.2.8：梁 / 杆 $R = [\,e_x\ e_y\ e_z\,]$、台阶 $R = R_y(\psi)$，$t = \tfrac12(A + B)$。

- `span beam from=pillarL.top-right to=pillarR.top-left`：两个锚点在 pillar 的盒上（经各自的 $T_{\cdot \to P}$ 换到 $P$）。
- 端点可以在不同容器里，也可以引用后置、副本、影子、阵列实例（`tower.decks.5.deck.top-right-front`）。
- 2.2.8 的 $\mathrm{perp}$ 与本章 3.5.4 的相同；$e_y$ 的退化阈值 $\varepsilon_{\text{dir}}$。

### 3.10.2 drop

`drop=B`（参照物，可带锚点；缺省锚点 `top`）与 `dir`（方向规格，3.9.5，在 $P$ 里，没有参照系节点，缺省 $(0,-1,0)$）：锚点的点 $q$ 与法线 $\nu$ 按 3.2.4 换到 $P$；$\nu = 0$ 时（裸引用、`center`、曲线取点）取 $B$ 的 $+y$ 在 $P$ 里的方向 $R_{B \to P}\hat y$。然后按 2.2.8 求 $B_{\text{点}} = A + s\,\mathrm{dir}$（与无限平面求交，不检查落点在面内）。

- rotation.s3d 的 `repeat legs n=20 along=track { span col from=(0,-0.7,0) drop=ground r=0.22 }`：每个实例里 $A = (0, -0.7, 0)$（实例的局部系，原点在轨道上），`dir` 缺省竖直向下（在实例框架里；`yaw` 朝向保持 $y$ 竖直，所以就是世界的下），ground 的顶面经 $T_{\text{ground} \to \text{实例}}$ 换到实例框架；20 根立柱与参考输出一致。
- 法线与 `dir` 平行、目标在反方向的报错见 2.2.8。

### 3.10.3 span 的局部系与盒

$T_n$ 由 2.2.8 给出；局部盒也由它给出（梁 / 杆 / 台阶各自的公式）。span 的 `pin` 是截面参数（2.2.8），不参与定位；`turn` 等不合法。span 的 $T$ 依赖的量见 3.6.1。span 可以当参照物（`on=deck`，锚点按它的局部盒：梁的顶面是梁真正的顶面，杆的盒是 $[-\tfrac L2,\tfrac L2]\times[-r,r]^2$）。

### 3.10.4 sweep / pipe 的路径点与曲线路径的变换

$T_n = \mathbb 1$：sweep、pipe 的局部系就是父框架，几何直接在 $P$ 里构造（2.2.9）。

- `path` 是点列表：每项是点（3.2.9），在 $P$ 里求值（坐标、锚点、带偏移的锚点、集合盒心）。
- `path` 是曲线 $C$：$C$ 的采样点列 $p_i$（在 $C$ 的局部系里）经 $T_{C \to P}$ 换到 $P$：$p_i' = R_{C \to P}\,p_i + t_{C \to P}$；arc / helix 的解析切线经 $R_{C \to P}$。这就是 2.2.9 里的 $(R_{\text{rel}}, t_{\text{rel}})$。$C$ 可以在任何位置（`sweep ramp path=rampline` 与 helix 同在 garage 组里；curves.s3d 的 `sweep lane path=loop` 在根）。
- 局部盒 = 全部环点的盒（2.2.9），世界盒 = 同一盒（$W_n = W_P$）。
- 依赖：$X(n)$ 依赖路径各点的依赖、$T_{C \to P}$ 与 $C$ 的采样点列（3.6.1）。sweep 的几何由 $X(n)$ 的计算一并得到。

### 3.10.5 line / bezier / spline

$T_n = \mathbb 1$；`path` 的每一项在 $P$ 里求值（3.2.9；引用无锚点时取盒心，02-geometry.md 的 2.3.4）；采样点列在 $P$ 里。它们不接受任何定位 / 朝向属性（写了报错，01-syntax.md 的 1.4.4）。作为参照物时局部盒是采样点列的盒（2.3.8）；它们的局部系就是 $P$，所以 `C[t]` 等取点直接是 $P$ 里的点，换到别的框架按 3.2.4（$T_C = \mathbb 1$ 只是链上的一个恒等因子）。

### 3.10.6 arc / helix

arc、helix 是一般节点（3.1.2）：有自己的局部系，原点是圆心，按 3.3–3.5 定位（`at=canal.top pin=origin`、`at=0,1,0 pin=origin`），缺省定位点是采样折线盒的底面中心（2.3.8）。它们的采样点列在自己的局部系里，被 sweep / repeat / 锚点使用时经 $T_{C \to F}$ 换框架（3.10.4、3.8.4）。

## 3.11 用参考输出验证

### 3.11.1 方法

脚本（scratchpad 的 `verify3.mjs`）对 7 个示例的参考输出 `corpus/reference/examples/<名>.compile.json` 逐节点核对本章的规则。为了只检验本章，脚本从原型只取第 1 章（属性值求值、名字解析）与第 2 章（局部包围盒、曲面锚点、曲线采样）的结果，**不**使用原型的求解结果；按 3.1.2 的表对每个节点独立计算 $T_n$（3.3–3.10 的公式照本章实现），再与参考输出比较：

- 变换：$W_P^{\text{ref}} \circ T_n$ 对 `nodes[].world`（$P$ 的世界变换取自参考输出，所以每个节点的核对只检验**它自己**那条规则，不受上游误差影响）；相对变换 $T_{B \to P}$ 用参考输出的 $W_P^{-1} \circ W_B$。
- 包围盒：$\mathrm{AABB}(W_n^{\text{ref}}(B_n))$ 对 `nodes[].box`（3.1.6）。
- 后置：`nodes[].late` 对 3.6.3 的判定（人工按依赖图推导）与 3.7.6 的皮肤规则。

容差 $10^{-6}$（场景单位 / 无量纲；即 09-compat.md 的 9.4.2 的 $\tau_{\text{rot}}$ 与场景尺度 $L \le 10^3$ 时的 $\tau_{\text{len}}$；实际最大偏差在 $10^{-12}$ 量级，来自 $T_{B \to P}$ 的两种计算顺序）。

### 3.11.2 结果

全部 7 个文件、1644 个节点（basics 18、curves 94、rotation 615、semantics 267、spaceport 193、spaceport_blueprint 240、spaceport_edit 217）的 `world` 与 `box` **全部通过**，参考输出的 `errors` / `warnings` 为空。按规则分类（一个节点按它用到的属性归一类；数字是节点数）：

| 规则 | 节点数 | 出现的文件 |
|---|---|---|
| `on`（3.3.2；含与 gap / turn / tilt / roll / aim / align 的组合） | 162 | 全部 7 个 |
| `against`（3.3.4，自动选面；含 align） | 15 | basics, curves, rotation, semantics, spaceport × 3 |
| `inside` + `align`（3.3.5、3.4.3） | 4 | basics, spaceport × 3 |
| `at=` 向量（3.3.6；含 pin / turn / tilt） | 70 | curves, rotation, semantics, spaceport × 3 |
| `at=` 引用（3.3.6；含 pin / offset / rot / align） | 25 | basics, curves, rotation, semantics, spaceport × 3 |
| 没有关系（3.3.7；含 turn） | 176 | 全部 7 个 |
| 只有 `pin`（3.3.7；含 turn） | 193 | 全部 7 个 |
| 变体皮肤及皮肤子树里的节点（3.7.6） | 77 | semantics, spaceport_blueprint |
| 网格阵列实例（3.8.3） | 97 | rotation, semantics, spaceport × 3 |
| 线性阵列实例（3.8.3，含 center / turn） | 117 | rotation, semantics, spaceport × 3 |
| 沿曲线 `yaw`（3.8.4） | 103 | curves, rotation, spaceport × 3 |
| 沿曲线 `full`（3.8.4） | 110 | rotation |
| `mirror … of=`（3.9.1） | 14 | curves, rotation, semantics, spaceport × 3 |
| `mirrored` 副本组（3.9.2） | 4 | curves, spaceport × 3 |
| `rotate … of=`（3.9.1） | 1 | curves |
| `r`$k$ 副本组（3.9.2） | 125 | curves, rotation, spaceport × 3 |
| 影子（3.9.7） | 260 | curves, rotation, spaceport × 3 |
| span 梁 / 杆 / 台阶（3.10.1、3.10.3） | 41 | 全部 7 个 |
| span `drop`（3.10.2） | 20 | rotation |
| sweep（3.10.4） | 20 | curves, rotation, spaceport × 3 |
| line / bezier / spline（3.10.5） | 10 | rotation, spaceport × 3 |

覆盖到的具体规则：五种关系里的四种（`under` 语料没有，见 3.11.4）；pin 的 `origin`、方向词、组合词（`bottom-right`）；负 gap；align 的面名（`bottom` `left` `back` `front`）；offset；turn、tilt、roll、rot、aim+aimwith（`top`）；继承（转过的 bunker / turned 上的物体、倾斜的 sat 组里的物体）；网格与线性阵列、`center`、`turn`、嵌套阵列（pisa 的 galleries × cols）；闭合与开放曲线上的 `along`；`across` 的引用 + `axis`、`across=x`；`about` 的引用、缺省 `angle`、`axis=z`、`rot` 与容器式 rotate 的组合；span 的三种形态与 drop；sweep 的曲线路径与点列表路径。

语料**没有**覆盖、只按原型源码与本章定义写下的规则：`under`（3.3.3；`relation` 的 `under` 分支，小例子见 3.11.4）、`alignto` 与 align 的轴名 `x` `y` `z`（3.4.3；`local` 的 align 循环，小例子见 3.11.4）、`aimwith` 取 `front` 等非竖直方向时的次轴选择（3.5.4；`orientation`，小例子 3.11.4 的 `aim=0,10,0`）、`orient=none`（3.8.4）、`across` 为向量₃或 `B[u,v,w]` 锚点（3.9.3）、方向规格的锚点与曲线切线形式（3.9.5）、副本式 `rotate … of=` 带 `about`（curves.s3d 的 `gate2` 只用了缺省轴）。

### 3.11.3 后置

参考输出里的 `late` 节点恰好是：rotation.s3d 的 `radar.dish`、`radar.feed`、`radar.horn`（3.6.3 第一个例子）与 semantics.s3d 的 `street.h4.skin`（3.7.6）；其他 1640 个节点都不是，其中包括另外两个皮肤 `slab.skin`（semantics）与 `hangar.roof.skin`（spaceport_blueprint）。按 3.6.3 的图规则推导得到前三个（dish 经 $T_{\text{sat} \to \text{radar}}$ 可达 $L(\text{radar}) \to X(\text{radar})$；feed、horn 经 $L(\text{dish})$ 可达）；三个皮肤按 3.7.6 的规则：h4 是模板实例（包络读全部子节点）→ 真，slab 与 roof 是 box → 假。

### 3.11.4 小例子

语料没有覆盖、用原型编译小例子确认原型行为（结果都已写进相应小节）：

| 情形 | 原型结果 | 本版 |
|---|---|---|
| `box a turn=90 offset=1,0,0`；转过的组里 `offset=1,0,0` | offset 在父框架里加（3.4.4） | 相同 |
| `box c under=floor`；`under=floor pin=center` | 顶面 / 盒心贴到底面中心（3.3.3） | 相同 |
| `on=floor turn=30 align=x`、`align=z,back` | 轴名对盒心、面名沿目标面法线（3.4.3） | 相同 |
| `pin=0,1,0` | 与 `pin=top` 相同（3.4.1） | 相同 |
| `against=floor.right pin=top` | 不自动选面，顶面中心落到右面中心（3.3.4） | 相同 |
| `inside=g gap=5` | gap 被静默忽略 | 报错（3.4.2） |
| `aim=0,10,0 offset=3,0,0`（无关系、无 pin）；`pin=center aim=0,10,0 offset=3,0,0`（只有 pin） | 前者起点取 offset、$\hat d = \mathrm{unit}(-3,10,0)$；后者起点是原点、offset 不计、$\hat d = \hat y$（3.5.4） | 相同 |
| `against=s[0,0,0]` | 不报错：$\nu = 0$，自动选面取 `left` 贴到盒心 | 报错（3.3.4） |
| `repeat r step=2,0,0 { … }`（没写 `n`） | 1 个实例 | 报错"缺少 n"（3.8.1） |
| `rot=(0,0,0,30)` | $R = 0.866\,I$（非正交） | 报错（3.5.3） |
| `mirror a2 of=a across=x axis=z` | axis 被忽略 | 报错（3.9.3） |
| `across=floor.right axis=z` / `across=floor.right` / `across=floor` | 3.9.3 表的第四 / 三 / 四行 | 相同 |
| `on=` 镜像副本 | $R_{\text{base}} = R_{B \to P}\,\mathrm{diag}(-1,1,1)$（3.5.7） | 相同 |
| `mirror … of=a turn=90 at=…`；`rotate … of=a n=3` | 定位属性与 `n` 被忽略 | 报错（3.9.1） |
| `repeat n=2.4` / `n=-1` / `n=0` | 2 个 / 0 个 / 0 个实例 | 报错 / 报错 / 0 个（3.8.1） |
| `repeat n=(2,2) step=1,2,3 center=true`；`n=3 step=1,0,0 turn=10` | 3.8.3 的两个公式 | 相同 |
| `repeat n=4 along=闭合弧`、`n=3 along=弧 orient=none turn=15` | 3.8.4 | 相同 |
| `repeat … along=c step=1,0,0 center=true orient=true` | step / center 被忽略，`true` 当 `yaw` | 报错（3.8.4、3.8.6） |
| `repeat r n=4 along=c on=floor` | 全部实例后置、阵列盒为零盒（3.8.5） | 相同 |
| `rotate rr n=2.6 { … }` | 3 个副本组 | 报错（3.9.2） |
| `rotate rr n=2 about=floor.right` | 绕锚点、120°（3.9.4） | 相同 |
| `group g on=floor { group h { box a on=#wall } box b }`；只给 h 写 `on=floor`；g、h 都写 | h 后置 / a 后置 / 都后置（3.6.3 第二个例子） | 相同 |
| 第一例前面加 `box z size=g.h.w,1,1` | 环在 h 的盒的求值里闭合：a 后置、h 不后置 | h 后置、a 不后置（3.6.3 不依赖求值顺序） |
| `group g4 { box x on=y; box y on=x }` | 只有 x 报错并退化 | x、y 都报错并退化（3.6.5） |
| `group g pin=origin at=… { box p at=g.origin }` | 不后置、不报错 | 报错（3.6.2：引用祖先） |
| `variant N use=cap`，$N$ 依次为 box、无 envelope 的 group、有 envelope 的 group、subtract、union、intersect、repeat 节点 | 皮肤 `late` 依次为假、真、假、假、真、真、真 | 相同（3.7.6 的表） |
| `mirror radar2 of=radar …`，radar 里的 dish 后置 | `radar2.dish` 的 `late` 为假（标记不复制） | 真（3.9.7：取原件的） |

### 3.11.5 因本章定义而预期不同的项

M1 的编译器按本章实现时，与原型输出的差别只会出现在下列情形；语料（7 个示例与 11 个 exports）里没有任何一处触发，所以参考输出仍然逐节点适用：

| 节 | 情形 | 原型 | 本版 | 理由 |
|---|---|---|---|---|
| 3.4.2 | `gap` 写在 $\nu = 0$ 的关系上 | 静默忽略（`local`） | 报错，节点退化 | 1.1.1：不得静默接受无效写法 |
| 3.5.3 | `rot` 的轴是零向量 | 非正交矩阵（`axisAngle`） | 报错 | 结果无定义 |
| 3.5.4 | `aimwith` 的值不是方向词 | 值不是名字（`aimwith=5`）时退回 `front`（`sym`）；是名字但不是方向词（`aimwith=foo`）时原型也报错 | 一律报错 | 类型规则 1.4.1 |
| 3.3.4 | `against=B[u,v,w]` 的主法线为零（`B[0,0,0]`） | 不报错：$\nu = 0$，自动选面取 `left`，gap 无效（`relation`） | 报错，节点退化 | 1.4.4：against 必须带面锚点 |
| 3.8.1 | `repeat` 没写 `n` | 缺省 1 个实例（`elaborate`） | 报错"缺少 n"，节点退化 | 1.4.5：必填（01-syntax.md 的 1.11 已列） |
| 3.6.2 | 定位引用自己的祖先 | 经后置得到一个解（`ref` 只查自己与后代） | 报错 | 容器的盒由子节点决定，引用没有不循环的含义 |
| 3.6.3 | 后置的判定 | 惰性求值的忙标记，依赖求值顺序 | 依赖图上的可达性，容器按节点顺序（父先于子） | 确定性（1.1.3）；语料结果相同 |
| 3.6.5 | 循环依赖 | 只退化先碰到的节点、报一条错 | 环上全部节点退化，各报一条 | 错误局部化与确定性 |
| 3.8.1 | `n` 非整数 / 负数 | 四舍五入 / 取 0（`elaborate`） | 报错 | 不猜 |
| 3.8.4、3.8.6 | `along` 与 `step` / `center` 同写；`orient` 没有 `along`；`orient=true/false` | 忽略 / 当 yaw、none（`instLocal`） | 报错 | 互斥与枚举规则 |
| 3.9.1 | 副本式节点写定位属性；`rotate … of=` 写 `n` | 忽略（`local` 的 `n.src` 分支、`rotationIn`） | 报错 | 1.4.4、1.4.5 |
| 3.9.2 | `rotate` 容器式的 `n` 非整数 | 四舍五入（`buildCopies`） | 报错 | 不猜 |
| 3.9.3 | `across=x/y/z` 与 `axis` 同写 | 忽略 axis（`mirrorIn`） | 报错 | 互斥 |
| 3.9.5 | 方向规格为零向量 | `unit` 得零向量，继续算 | 报错 | 结果无定义 |
| 3.3.8 | 多个定位关系 | `on` 优先、其余忽略（`relation`） | 报错（已在 3.3.8 列出） | 1.4.4 |
| 3.9.7 | 后置节点被 mirror / rotate 复制时影子的 `late` | 假（标记不复制） | 取原件的（真） | 副本的包络等于原件的包络，影子同样不计入；只是输出字段，不改变任何量 |

## 3.12 本章的错误与警告

按 01-syntax.md 的 1.10 的格式；"恢复"按 1.1.4（节点退化 = $T = \mathbb 1$、零盒、不参与事实、不渲染，子节点照常）。错误携带文件、行、文案与节点路径（1.10.1）。

| 种类 | 触发条件 | 文案（引号里是必须包含的信息） | 恢复 |
|---|---|---|---|
| 互斥 | 写了多个定位关系（3.1.4） | "定位关系只能写一个" | 节点退化 |
| 类型错误 | 参照物属性的值不是引用也不是集合（3.2.8、3.3） | "`<路径>` 的 `<属性>` 需要引用一个对象，得到 X" | 节点退化 |
| 类型错误 | 集合为空（3.2.8） | "集合是空的，不能当参照物" | 节点退化 |
| 类型错误 | 点属性的值不是向量₃ / 引用 / 集合（3.2.9） | "`<路径>` 的 `<属性>` 需要点坐标或锚点" | 节点退化 |
| 找不到的名字 | 对非曲线写 `[t]`；对非曲线写 `start` / `mid` / `end`（3.2.2） | "X 不是曲线，不能用 [t] 取点"；"X 中没有子对象或锚点 start" | 节点退化 |
| 找不到的名字 | 对集合取曲线点、曲面锚点或子对象（3.2.8） | "{…} 中没有子对象或锚点 X" | 节点退化 |
| 互斥 / 类型错误 | `against` 的锚点没有非零法线（3.3.4） | "`<路径>`：against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]" | 节点退化 |
| 类型错误 | `pin` 不是 origin / 方向 / 三个数（3.4.1） | "pin 写成 origin、方向词或 u,v,w" | 节点退化 |
| 互斥 | `gap` 写在法线为零的关系上（3.4.2） | "`<路径>`：gap 需要有法线的定位关系（on / under / against，或 at= 带面锚点）" | 节点退化 |
| 类型错误 | `align` 的项不是面名或 x / y / z（3.4.3） | "align 只接受面名或 x / y / z，如 left,back" | 节点退化 |
| 互斥 | `align` 没有对齐目标（3.4.3） | "`<路径>`：align 需要参照（on / under / against / inside / at=对象 或 alignto=对象）" | 节点退化 |
| 互斥 | `alignto` 没有 `align`；`aimwith` 没有 `aim`（1.4.4） | "alignto 需要 align"；"aimwith 需要 aim" | 节点退化 |
| 类型错误 | `rot` 不是四个数（3.5.3） | "`<路径>` 的 rot 写成 (轴x,轴y,轴z,角度)" | 节点退化 |
| 类型错误 | `rot` 的轴长度 $< \varepsilon_{\text{len}}$（3.5.3） | "`<路径>` 的 rot 的轴不能是零向量" | 节点退化 |
| 类型错误 | `aimwith` 不是方向词 / 组合词（3.5.4） | "`<路径>` 的 aimwith 需要方向词，如 front / top / right" | 节点退化 |
| 求解错误 | `aim` 的目标与起点距离 $< \varepsilon_{\text{len}}$（3.5.4） | "`<路径>`：aim 的目标与自身重合" | 节点退化 |
| 求解错误 | 定位 / 点 / 曲线属性引用自己或后代（3.6.2） | "`<路径>` 不能相对自己定位" | 节点退化 |
| 求解错误 | 引用祖先（3.6.2） | "`<路径>` 不能相对包含自己的对象定位" | 节点退化 |
| 求解错误 | 集合里含自己或上下级（3.6.2） | "`<路径>` 的 `<属性>`：集合里包含它自己或它的上下级" | 节点退化 |
| 循环 | 依赖图去掉后置边后仍有环（3.6.5） | "循环依赖：`<路径>` 的位置最终依赖于它自己" | 环上每个节点退化，各报一条 |
| 缺少必填 | repeat 没有 `n`（3.8.1） | "`<路径>` 缺少 n" | 节点退化，不生成实例 |
| 时机错误 | `n` 不是常量（3.8.1） | "`<路径>` 的 n 必须是常量" | 同上 |
| 类型错误 | `n` 的分量不是非负整数（容差 $\varepsilon_{\text{int}}$）或不是数 / 向量（3.8.1） | "`<路径>` 的 n 需要非负整数（得到 X）"；"`<路径>` 的 n 需要数或 n_i,n_j,n_k" | 同上 |
| 几何参数不合法 | 实例总数 $> N_{\max}$（3.8.1） | "repeat 数量过多（N）" | 同上 |
| 类型错误 | `along` 不是曲线（3.8.4） | "`<路径>` 的 along 需要一条曲线（line / bezier / spline / arc / helix）" | 同上 |
| 互斥 | `along` 与 `step` / `center` 同写；`orient` 没有 `along`（3.8.4） | "`<路径>`：along 与 step / center 不能同时写"；"`<路径>`：orient 只能和 along 一起写" | 同上 |
| 类型错误 | `orient` 不是 yaw / full / none；`center` 不是布尔（3.8.6） | "`<路径>` 的 orient 只能是 yaw / full / none"；"`<路径>` 的 center 只能是 true / false" | 同上 |
| 副本限制 | 副本式 mirror / rotate 写定位 / 朝向属性，或 `rotate … of=` 写 `n`（3.9.1） | "副本跟随原物体，不接受定位属性"；"rotate 副本不接受 n"（04-derivation.md 的 4.4.6） | 节点退化 |
| 类型错误 | `of` 不是单个对象路径（3.9.1） | "`<路径>`：of 需要写一个对象名" | 节点退化 |
| 类型错误 | `rotate` 容器式的 `n` 不是 1..360 的整数（3.9.2） | "`<路径>`：rotate 的 n 需要 1..360 的整数" | 节点退化，不生成副本组 |
| 类型错误 | `across` 不合法（3.9.3） | "across 写成 x / y / z、点坐标，或对象 / 锚点（如 B、B.right、B.top）" | 节点退化 |
| 互斥 | `across=x/y/z` 与 `axis` 同写（3.9.3） | "`<路径>`：across 是轴名时不能再写 axis" | 节点退化 |
| 几何参数不合法 | 镜像面法线为零（3.9.3） | "`<路径>`：镜像面的法线为零" | 节点退化 |
| 类型错误 | 方向规格不合法（3.9.5） | "`<路径>` 的 `<属性>` 需要方向：x / y / z、方向词、向量，或锚点如 B.front" | 节点退化 |
| 几何参数不合法 | 方向规格为零向量（3.9.5） | "`<路径>` 的 `<属性>` 不能是零向量" | 节点退化 |
| 几何参数不合法 | span 的端点、drop、台阶的退化（3.10.1、3.10.2） | 02-geometry.md 的 2.2.8 的文案 | 节点退化 |

副本式节点 $R$ 退化时它的影子子树照常生成但随 $R$ 一起放在 $\mathbb 1$ 处（1.1.4 第 6 条：只影响该副本）；阵列节点退化时不生成实例（3.8.6）。后置（3.6.3）不是错误，不产生诊断。

**警告。** 本章不定义警告。曲面锚点的近似（`mesh.top[u,w]`）的警告在 02-geometry.md 的 2.4.1。
