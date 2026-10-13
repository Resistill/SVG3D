# 6 空间事实

**范围。** 本章定义求解之后从几何推导出的**空间事实**（fact）：哪些节点参与（对象）、怎样枚举候选对、每个对象提供给事实层的几何（世界包围盒、局部包围盒、凸块、近似标记）、两个凸块之间的**分离量**（面法线 ∪ 棱叉积 ∪ 球心方向上的最大间隙，可为负）、两个对象之间的间隙与接触块对、五种接触关系 on / against / inside / touching / intersects 的精确判定（谓词、容差、$a$ / $b$ 的顺序、对称性）、事实的输出形状与确定性要求、"不推导"的输出形状、事实提供者的接口，以及语义层读事实用的**关系索引**（关系向祖先上提、谓词 `on(S, k)` 等的定义）。事实只看几何，不看语句写了什么关系（6.5.7）。

本章**不**定义：每种实体的凸块怎样构造（02-geometry.md 的 2.2 节各实体的"凸块"、2.5.6、2.6、2.7；本章只用它们的结果）；节点的世界变换与世界包围盒（03-solving.md）；哪些节点是影子、皮肤、拷贝（04-derivation.md）；关系谓词在 concept / pattern / check / style / variant 里怎样求值、集合运算、`where`（07-semantics.md）；工具怎样使用事实（坐标依赖、报告、界面）——那些是工具约定。

**依赖的章。** 01-syntax.md（术语、节点顺序 1.6.1、`all()` 的范围 1.7.2、关系谓词的语法 1.5.6、错误恢复 1.1.4）、02-geometry.md（局部包围盒、凸块与 `approx`、容差常数 2.1.4、布尔容器 2.5、容器 2.6、mesh 2.7）、03-solving.md（世界变换 $W_n$、世界包围盒、容器的包络 3.7）、04-derivation.md（阶段表 4.1.1、影子 4.4.5、变体皮肤的挂载 4.7）。被依赖：05-layers.md 的 5.4.4（外观层不改变事实，依据本章的 6.1.2 与 6.2.1）、07-semantics.md 的 7.1.4（事实未推导）、7.2.4（关系索引只读事实的 `rel`、`a`、`b`）、7.8.4（皮肤与隐藏的不可见性）、09-compat.md 的 9.4.3（事实的比较规则）与 9.5.3（exports 的预期改变）。

**记法。** 沿用 02-geometry.md：向量是列向量，$\hat x, \hat y, \hat z$ 是坐标轴单位向量，$\lVert v \rVert$ 是欧氏长度，$\mathrm{unit}(v) = v / \lVert v \rVert$。节点 $n$ 的世界变换 $W_n = (R_n, t_n)$，$p \mapsto R_n p + t_n$（03-solving.md 的 3.1.1）。轴对齐盒 $b = [b^{\min}, b^{\max}]$ 的分量写 $b^{\min}_x$ 等。区间 $I = [I_0, I_1]$。两个区间的**重叠量** $\mathrm{ov}(I, J) = \min(I_1, J_1) - \max(I_0, J_0)$（分离时为负，等于间隙的相反数）；两个盒沿轴 $k$ 的重叠量 $\mathrm{ov}_k(a, b) = \mathrm{ov}([a^{\min}_k, a^{\max}_k], [b^{\min}_k, b^{\max}_k])$。

## 6.1 范围、依赖、记法

### 6.1.1 事实是什么

一条空间事实是一个三元组 (关系, $a$, $b$) 加上度量：关系是本章定义的五个词之一，$a$、$b$ 是两个**对象**（6.2）的路径，度量是它们之间的间隙 `gap`、判定用的容差 `eps`、是否基于近似几何 `approx`。事实在求解（第 3 章）之后、语义层（第 7 章）之前推导一次，只依赖对象的世界几何；定位属性（`on=`、`against=` ……）不产生事实，也不阻止事实（6.5.7）。事实是语义层关系谓词（6.7）与工具的输入；本章只给事实，不做任何"这是什么"的判断。

### 6.1.2 阶段

事实在下面的时点推导：全部几何时机的修改已施加、模板与阵列已展开、副本树已生成、求解已完成（含后置节点，3.6.4）、变体的选择集已确定但皮肤不参与（皮肤子树被排除，6.2.1）。这是 04-derivation.md 4.1.1 阶段表的阶段 7；阶段 8（外观时机）的任何操作——样式、只改外观的 over、变体皮肤的挂载——不改变事实（07-semantics.md 的 7.1.1 两阶段保证；05-layers.md 的 5.4.4）。

**事实只由阶段 7 之前的量决定。** 对象的集合、每条事实的判定与输出顺序都只是 6.2.2 那张表里的量（世界变换、局部包围盒、凸块、`approx`）与节点顺序的函数：`contact` 提供者**不读** `hidden` 标记、样式、材质属性（`mat` / `color` 与材质表）、元数据、标签、概念成员；也不读节点是否被变体选中。因此同一场景有没有变体、有没有样式，事实列表逐条相同（6.8：spaceport 与 spaceport_blueprint 的 203 条事实相同）；05-layers.md 的 5.4.4 第 3 项以本条与 6.2.1 第 3 条为依据。

### 6.1.3 本章的常量

| 名字 | 值 | 单位 | 用途 |
|---|---|---|---|
| $\varepsilon$（接触容差） | $0.02 \cdot \mathtt{metre}$ | 场景单位 | 一切"贴着"的判定；输出里的 `eps` 字段。它是 2 厘米换算成场景单位，是本规范里**唯一随 `metre` 换算**的容差（`metre` 是宿主给出的全局常量，默认 1，01-syntax.md 的 1.5.8） |
| $\varepsilon_{\text{dir}}$ | $10^{-6}$ | 无 | 竖直轴平行、相对旋转轴对齐的判定（与 02-geometry.md 的 2.1.4 同名同值） |
| $\varepsilon_{\text{len}}$ | $10^{-9}$ | 场景单位 | 零长度分离轴的丢弃；零尺寸对象的排除（与 2.1.4 同名同值） |
| $\kappa_{\text{in}}$ | $0.9$ | 无 | `inside` 的体积比上限（6.5.4） |

判定一律写成明确的不等式，边界（恰好相等）按不等式的写法归属。参考输出包里 `metre = 1`，所以每条事实的 `eps` 都是 0.02；实战工程以 stud 为单位（$\mathtt{metre} = 1/0.28$）时 $\varepsilon = 0.0714\ldots$ stud，仍是 2 厘米。

## 6.2 事实的对象与候选对

### 6.2.1 参与的节点

**对象**（fact object）是满足下列全部条件的节点，按节点顺序（01-syntax.md 的 1.6.1）编号 $n_0, n_1, \dots$：

1. 它的有效种类（保名换形后的种类，04-derivation.md 的 4.6）是**实体**（1.1.2 列出的 12 种：box、wedge、plate、hull、cyl、cone、sphere、revolve、span、sweep、pipe、mesh）或**布尔容器**（union、subtract、intersect）。布尔容器整体是一个对象（2.5.1）。
2. 它的任何祖先都不是布尔容器：布尔容器内部的操作数、嵌套的布尔容器、布尔容器里的 group 及其后代都不单独参与（2.5.1"其内部的操作数不单独参与"）。
3. 它不在任何变体皮肤子树里（段为 `skin` 的节点及其全部后代不参与，04-derivation.md 的 4.7"不改变阶段 7 的量"；皮肤对各种机制可见与不可见的精确清单在 07-semantics.md 的 7.8.4）。被变体**选中**的宿主与被**隐藏**的原物体（`hidden`）照常参与：事实按原来的几何算，隐藏只影响输出条目（7.8.4）。
4. 它不是退化节点（1.1.4 第 2 条：零盒、没有凸块），且局部包围盒 $B_n$ 的三边之和 $w + h + d > \varepsilon_{\text{len}}$。合法的实体三边都 $> 0$，所以这一条只排除退化节点与盒为零盒的布尔容器（2.5.1 没有操作数、2.5.4 的 `intersect` 为空）。退化节点因此不出现在任何事实里，也就不在任何关系谓词的结果里（7.2.4）。

不参与的节点：曲线（2.3：没有凸块）；容器 group、repeat 节点与阵列实例、容器式 mirror / rotate 节点、副本组、模板实例、copy 节点（它们没有自己的几何；事实按其内部的实体分别计算，2.6）；皮肤子树；退化节点。**参与**的节点里包括：阵列实例里的实体、模板实例里的实体、copy 出来的实体、影子（04-derivation.md 的 4.4.5：按自己的世界变换，镜像翻转手性不影响事实的定义）、副本式节点 `mirror R of=A` 当 $A$ 是实体时的 R、后置节点（3.6.4：后置只改变求解的先后与包络的计算，它们有世界变换，照常参与事实）、隐藏的原物体。容器从不是对象，容器的事实就是它内部实体的事实：有 `envelope` 的 group 也不是对象，它里面的实体照常按真实几何参与（2.6"事实与渲染仍按内容"、3.7.1"空间事实与渲染按内容"）——包络只用于定位与锚点，不进入本章。

对象之间不存在祖先关系：实体不接受块（1.3.15），布尔容器内部不参与。实现仍**必须**保证互为祖先 / 后代的两个节点不构成候选对（6.2.3）——若某个版本允许实体带块，这条规则直接生效。

### 6.2.2 对象提供的几何

每个对象 $n$ 向事实层提供下列量，全部由第 2、3 章决定：

| 量 | 定义 | 用途 |
|---|---|---|
| $W_n = (R_n, t_n)$ | 世界变换（3.1.1） | 把局部量换到世界；`on` / `against` 的参照系 |
| $B_n$ | 局部包围盒（2.1.2；布尔容器按 2.5.4；mesh 按外包络 2.7） | 体积比较、`against` 的面、零尺寸判定 |
| $V_n = \mathrm{vol}(B_n)$ | 局部包围盒的体积 $= w\,h\,d$ | 谁是 $a$、`inside` 的体积比 |
| 凸块 $K_n = (K_{n,0}, \dots, K_{n,m-1})$ | 第 2 章该实体的"凸块"（局部系；布尔容器按 2.5.6 是一个盒），每块经 $W_n$ 变换到**世界系**：多面体的顶点 $R_n v + t_n$、法线集与棱集里的方向 $R_n d$；球的球心 $R_n c + t_n$、半径不变 | 分离量、接触、`on` / `inside` 的投影 |
| $\mathrm{approx}(n)$ | 该实体凸块的 `approx` 标记（2.1.5 第 9 条）；多块实体取各块标记之**或**（sweep 的凹截面段，2.2.9） | 事实的 `approx` 字段 |
| 块盒 $\beta_{n,i}$ | 第 $i$ 块的世界轴对齐包围盒：多面体取顶点的最小 / 最大；球取 $c \pm r$ | 块对的粗筛 |
| 对象盒 $\Omega_n$ | 全部块盒之并的轴对齐盒（各轴取最小 / 最大） | 候选对的粗筛、扫描序 |
| 盒类 $\mathrm{boxy}(n)$ | 恰有一块，且该块是按 2.2.1 的盒构造或 2.2.2 的楔形构造得到的：有效种类为 box、wedge、mesh（外包络）、span 的梁形态（2.2.8）、布尔容器（2.5.6 的代理盒）。cyl、sphere、plate、hull、revolve、杆、台阶、sweep、pipe 不是盒类 | `against`（6.5.3） |

凸块按 2.1.6.1 的表示：顶点列表、**法线集**（面法线按方向去重，正负不分）、**棱集**（边方向按方向去重）；球是 $(c, r)$。分离量（6.4）只用顶点、法线集、棱集与球。对精确凸块（`approx=false`），$\Omega_n$ 是凸块顶点（球则 $c \pm r$）的盒：多面体实体（box、wedge、plate、hull、梁、台阶）与 sphere 的 $\Omega_n$ 等于该对象的紧包围盒（2.1.8）；圆类实体的凸块用内接多边形代替圆（cyl、cone、杆：12 边棱柱 / 棱台，2.2.3、2.2.8；圆截面的 sweep / pipe：每环取 8 点，2.2.9），顶点落在圆上，cyl / cone / 杆轴对齐时极值与紧包围盒相同，否则 $\Omega_n$ 比按真实形状算的紧包围盒**小**（cyl / cone / 杆每面至多 $R(1 - \cos 15°)$）。2.1.8 的 `tight` 按真实几何、本章的 $\Omega_n$ 按凸块，两者不要求相等；$\Omega_n$ 只用于粗筛与扫描序，不输出（参考输出的 `tight` 恰是原型按凸块顶点算的盒，对它比较时的额外容许量由 09-compat.md 的 9.4.2（$\delta_{\text{tight}}$，与 02-geometry.md 同一处定义）规定）。对近似凸块，$\Omega_n$ 是代理的盒。

**近似的含义。** `approx=true` 的凸块是包住实体的外接形状（revolve 的外接棱柱、mesh 的外包络、布尔容器的代理盒、凹截面扫掠段的凸包）。它们参与全部判定，与精确块没有区别；只是由它们得出的事实带 `approx: true`（6.6.1），由读事实的一方决定信不信。特别地，布尔容器挖掉的空间在事实里算实心：放在 `subtract` 空腔里的物体与它的关系是 `inside`、`gap` 为负、`approx: true`（2.5.6）。

### 6.2.3 扫描序与候选对

**扫描序。** 对象按键 $(\Omega^{\min}_{n,x},\ \text{节点序号})$ 升序排列（先按对象盒的 $x$ 最小值，相等时按节点顺序），得到扫描序号 $\sigma(n) = 0, 1, \dots$。扫描序只用来固定枚举顺序、输出顺序与两个平局（6.5.1、6.5.6），不影响任何几何判定。

**候选对。** 有序对 $(P, Q)$，$\sigma(P) > \sigma(Q)$，是候选对当且仅当
$$\mathrm{ov}_x(\Omega_P, \Omega_Q) \ge -\varepsilon,\qquad \mathrm{ov}_y(\Omega_P, \Omega_Q) \ge -\varepsilon,\qquad \mathrm{ov}_z(\Omega_P, \Omega_Q) \ge -\varepsilon,$$
且 $P$、$Q$ 互不为祖先 / 后代。候选对的枚举顺序是 $(\sigma(P), \sigma(Q))$ 的字典序（先按 $P$ 升序，再按 $Q$ 升序）。

这三个不等式是**定义的一部分**，不只是加速：分离量（6.4.2）是真实距离的下界，有些对的分离量 $\le \varepsilon$ 而真实距离 $> \varepsilon$（6.4.4），粗筛把其中沿坐标轴已经分开超过 $\varepsilon$ 的对排除在外；两份实现必须按同一组不等式取同一组候选对。实现可以按 $x$ 扫描：按扫描序推进 $P$，活动集里 $\Omega^{\max}_{Q,x} < \Omega^{\min}_{P,x} - \varepsilon$ 的 $Q$ 永远不再是候选，可以剔除——这与上面的定义等价。

**示例**
```s3d
box floor size=40,1,24 pin=top
box table size=8,3,5 on=floor[-0.5,1,0]
sphere ball r=0.8 on=table[0.6,1,-0.5]
cyl post r=0.3 h=5 at=30,0,0
```
对象 4 个（都是实体）。`post` 的对象盒 $x \in [29.7, 30.3]$ 与 `floor` 的 $[-20, 20]$ 沿 $x$ 分开 9.7（与 `table`、`ball` 也沿 $x$ 分开），不是候选对；`ball` 的对象盒 $y \in [3, 4.6]$ 与 `floor` 的 $[-1, 0]$ 沿 $y$ 分开 3（$\mathrm{ov}_y = -3 < -\varepsilon$），也不是候选对。候选对只有 (`table`, `floor`) 与 (`ball`, `table`)，推导得到 `table on floor`、`ball on table`（6.5.2）；原型对这个例子恰输出这两条事实（6.8.4）。

## 6.3 几何提供者接口

### 6.3.1 分层

事实层与几何层之间只有 6.2.2 那张表：几何内核对每个对象给出 $W_n$、$B_n$、凸块、`approx`，事实层据此做全部判定。事实层不读 SDF、三角网格、曲线、材质、元数据；几何层不知道关系的名字。这保证两份实现只要凸块相同（第 2 章）、变换相同（第 3 章），事实就按本章逐条相同。

### 6.3.2 事实提供者

事实由一组**提供者**（provider）推导。提供者有名字、它产出的关系词列表，和一个推导函数：输入全部对象（6.2.1 的列表，带 6.2.2 的几何）与上下文，输出一组事实。上下文包含：$\varepsilon$、`metre`、候选对的枚举（6.2.3，可带别的边距）、两个对象的间隙与接触块对（6.4.3）、凸块的几何工具（投影区间 6.4.1、分离量 6.4.2、点在凸块内、线段与凸块的间隙）。提供者只读几何，不读语义；它不得修改对象或场景树。

- 本规范**只定义一个提供者**：`contact`，产出 `on`、`against`、`inside`、`touching`、`intersects` 五种关系（6.5）。它是缺省启用的提供者；它的事实集合由本章完全确定，任何实现、任何提供者组合下都相同。
- 实现**可以**提供别的提供者（例如可行走面、可达、可见），它们是工具约定，不在本规范内；它们的名字不得是 `contact`，产出的关系词不得与 6.5 的五个重名（二元事实用自己的关系词；关系索引只认五个词，所以这些事实不进索引，7.2.4 第 1 条）；每条事实至少有 `rel`、`a`、`text`、`provider`，二元事实有 `b`，一元事实（"X 怎样"）的 `b` 为 `null`；`a`、`b` 是节点的完整路径（6.6.1 的形式，不带锚点后缀）；度量字段由各提供者自定，`gap` / `eps` / `approx` 若出现，含义必须与 6.6.1 相同。
- 启用哪些提供者由宿主决定（编译选项）；输出的 `factProviders` 按启用顺序列出名字（6.6.2）；`facts` 里各提供者的事实按启用顺序分组，组内按该提供者自己的确定性顺序（`contact` 的顺序见 6.6.1）。
- 一个提供者出错（抛出）：记一条错误"事实提供者 X 出错：…"，它的事实全部丢弃，其他提供者照常（1.1.4 第 6 条的局部化）。请求了不存在的提供者名：报错"没有名为 X 的事实提供者（已注册：…）"，并按缺省的提供者列表继续。这两种错误不对应任何语句：归根文件（文件序号 0）、行 0、不带节点路径（05-layers.md 的 5.2.5）。
- 语义层的七个关系谓词（6.7）只读 `contact` 的事实；其他提供者的事实（一元的，以及用了别的关系词的二元的）只进输出，给工具读。

### 6.3.3 对象记录

提供者收到的每个对象记录含：路径、有效种类、节点（供读标签、元数据、种类）、$W_n$、$B_n$、世界凸块列表、各块的块盒、对象盒、`approx`、`boxy`（6.2.2），以及它在节点顺序与扫描序里的序号。记录按节点顺序给出。`contact` 提供者只读其中 6.2.2 表里的量与两个序号（6.1.2）；节点字段给其他提供者读标签、元数据、种类用。

## 6.4 分离量

### 6.4.1 投影区间

凸块 $K$ 在单位方向 $L$ 上的投影区间 $\mathrm{proj}(K, L)$：

- 多面体（顶点 $v_0, \dots$）：$[\min_i v_i \cdot L,\ \max_i v_i \cdot L]$；
- 球 $(c, r)$：$[c \cdot L - r,\ c \cdot L + r]$。

### 6.4.2 两个凸块的分离量

对凸块 $A$、$B$（世界系），**候选分离轴**集合按下面的顺序构成（顺序只决定浮点运算的发生次序，结果是最大值，与顺序无关）：

1. $A$ 的法线集（$A$ 是多面体时）；
2. $B$ 的法线集（$B$ 是多面体时）；
3. $A$、$B$ 都是多面体时，所有棱对的叉积 $e_A \times e_B$（$e_A$ 遍历 $A$ 的棱集，内层 $e_B$ 遍历 $B$ 的棱集）；
4. 两者都是球：$c_B - c_A$；一个是球 $S$、另一个是多面体 $P$：对 $P$ 的每个顶点 $v$ 加入 $v - c_S$。

对每个候选轴 $L'$：若 $\lVert L' \rVert < \varepsilon_{\text{len}}$ 则丢弃（平行棱的叉积、重合的球心），否则 $L = \mathrm{unit}(L')$，$[a_0, a_1] = \mathrm{proj}(A, L)$，$[b_0, b_1] = \mathrm{proj}(B, L)$，该轴的间隙 $g_L = \max(b_0 - a_1,\ a_0 - b_1)$。**分离量**
$$\mathrm{gap}(A, B) = \max_L g_L .$$
多面体至少有一个法线，所以只要有一方是多面体就至少有一个候选轴。两个球的球心重合（$\lVert c_B - c_A \rVert < \varepsilon_{\text{len}}$，同心球）时唯一的候选轴被丢弃，没有候选轴，此时**定义** $\mathrm{gap}(A, B) = -(r_A + r_B)$——即下面两球公式 $\lVert c_B - c_A \rVert - r_A - r_B$ 在同心时的值；原型 `partGap` 在这种情形下得到 $-\infty$，输出的 `gap` 成为 JSON 的 `null`（6.8.5）。

性质：$\mathrm{gap} > 0$ 表示两块分离，$\mathrm{gap}$ 是它们之间距离的**下界**；$\mathrm{gap} \le 0$ 表示相交或相切，$-\mathrm{gap}$ 是把两块沿某个候选轴分开所需平移的下界（穿透深度的下界）。$\mathrm{gap}(A, B) = \mathrm{gap}(B, A)$（候选轴集合相同，$g_L$ 对称）。法线集与棱集按方向去重且正负不分（2.1.6.1），$g_L$ 对 $L$ 取反不变，所以去重不改变结果。

**精确的情形。** 两个多面体的最近特征是"顶点—面"或"棱—棱"时，分离轴在候选集里，$\mathrm{gap}$ 等于真实距离；最近特征是"顶点—顶点"或"顶点—棱"时 $\mathrm{gap}$ 小于真实距离。两个球：$\mathrm{gap} = \lVert c_B - c_A \rVert - r_A - r_B$，精确。球与多面体：最近点在多面体的面或顶点上时精确，在棱上时偏小。相交时 $-\mathrm{gap}$ 等于沿候选轴的最小穿透深度，在候选集完备（多面体对、球对）时就是最小穿透深度。

### 6.4.3 两个对象的间隙与接触块对

对对象 $P$、$Q$，按 $P$ 的块序号 $i$ 为外层、$Q$ 的块序号 $j$ 为内层遍历块对 $(i, j)$：

1. 块盒粗筛：若 $\mathrm{ov}_x(\beta_{P,i}, \beta_{Q,j}) < -\varepsilon$ 或 $\mathrm{ov}_y < -\varepsilon$ 或 $\mathrm{ov}_z < -\varepsilon$，跳过该块对（与 6.2.3 相同，也是定义的一部分）；
2. 否则 $g_{ij} = \mathrm{gap}(K_{P,i}, K_{Q,j})$。

对象间隙 $\mathrm{gap}(P, Q) = \min_{(i,j)\ \text{未跳过}} g_{ij}$（没有未跳过的块对时为 $+\infty$）；**接触块对**列表 $T(P, Q) = \big((i, j) : g_{ij} \le \varepsilon\big)$，按遍历顺序。$\mathrm{gap}(P, Q) = \mathrm{gap}(Q, P)$；$T(Q, P)$ 是 $T(P, Q)$ 把每对的两个序号交换、**顺序不变**。

### 6.4.4 为什么粗筛是定义

分离量是距离的下界（6.4.2），所以存在"真实距离 $> \varepsilon$ 但分离量 $\le \varepsilon$"的块对（例如两个盒子顶点对顶点斜着相距 3 厘米：候选轴里没有连接两顶点的方向，每个候选轴上的间隙都更小，可能 $\le 2$ 厘米）。6.2.3 与 6.4.3 的轴向粗筛把沿坐标轴已经分开超过 $\varepsilon$ 的情形排除；剩下的情形（沿三个坐标轴都重叠或几乎重叠、斜向分开）本规范接受分离量的判定。两份实现按同一套不等式得到同一组事实；不按这套粗筛的实现会多出事实，不符合本规范。

## 6.5 关系

### 6.5.1 判定顺序

对每个候选对 $(P, Q)$（$\sigma(P) > \sigma(Q)$，6.2.3）：

1. 求 $\mathrm{gap}(P, Q)$ 与 $T(P, Q)$（6.4.3）。若 $\mathrm{gap}(P, Q) > \varepsilon$，这一对**没有事实**，结束。
2. 否则这一对**恰有一条事实**，按下面的顺序取第一个成立的：
   1. $P$ on $Q$（6.5.2，$A = P$、$B = Q$，接触块对按 $T(P, Q)$ 的顺序）；
   2. $Q$ on $P$（$A = Q$、$B = P$，接触块对按 $T(Q, P)$ 的顺序，即与上一步相同的次序）；
   3. 记 $A$ 为两者中局部包围盒体积较小者、$B$ 为另一个：$V_P \le V_Q$ 时 $A = P$，否则 $A = Q$（体积相等时 $A = P$，即扫描序靠后者）。$A$ against $B$（6.5.3）；
   4. $A$ inside $B$（6.5.4）；
   5. $\mathrm{gap}(P, Q) \ge -\varepsilon$ 时 $A$ touching $B$，否则 $A$ intersects $B$（6.5.5）。

事实的 `approx` 字段 $= \mathrm{approx}(P) \lor \mathrm{approx}(Q)$（对五种关系都一样）。一个对象可以与多个对象各有一条事实（一块板同时 `on` 两根柱子；一个盒子既 `on` 地面又 `against` 墙时，与地面是 `on`、与墙是 `against`——两条事实来自两个不同的对）。

### 6.5.2 on

"$A$ on $B$"：$A$ 立在 $B$ 的上方向上，在 **$B$ 的局部系**里判定。记 $R_{A \to B} = R_B^{\mathsf T} R_A$（$A$ 的朝向在 $B$ 的局部系里），$u = R_B\,\hat y$、$e_x = R_B\,\hat x$、$e_z = R_B\,\hat z$（$B$ 的三个局部轴在世界里的方向）。条件：

1. **竖直轴平行**：$\big\lvert\, \lvert (R_{A \to B})_{yy} \rvert - 1 \,\big\rvert \le \varepsilon_{\text{dir}}$，即 $A$ 的局部 $+y$ 与 $B$ 的局部 $+y$ 同向或反向（$(R_{A \to B})_{yy} = (R_A \hat y) \cdot (R_B \hat y)$）。倾斜的 $A$（`tilt=10`）不满足；倒扣的 $A$（`roll=180`）满足。
2. **底贴顶**：$A$ 的最低处 $\beta_A = \min_i\, \mathrm{proj}(K_{A,i}, u)_0$（在 $u$ 方向上，$A$ 全部块的下端的最小值）。按接触块对的顺序取第一个满足下面两条的 $(i, j)$（$i$ 是 $A$ 的块、$j$ 是 $B$ 的块）：
   - $\lvert \beta_A - \tau_j \rvert < \varepsilon$，其中 $\tau_j = \mathrm{proj}(K_{B,j}, u)_1$（$B$ 的第 $j$ 块的上端）；
   - **水平重叠**：$\mathrm{ov}\big(\mathrm{proj}(K_{A,i}, e_x),\ \mathrm{proj}(K_{B,j}, e_x)\big) > \varepsilon$ 且 $\mathrm{ov}\big(\mathrm{proj}(K_{A,i}, e_z),\ \mathrm{proj}(K_{B,j}, e_z)\big) > \varepsilon$。
3. 成立时事实为 `on`，$a = A$、$b = B$，`gap` $= \beta_A - \tau_j$（用第一个满足的 $j$；正为悬空、负为沉入，绝对值 $< \varepsilon$）。

说明：

- 判定在 $B$ 的局部系里，所以转过、倾斜的 $B$ 的"上"是它自己的 $+y$：`turn=35` 的 `turned`（basics）上的东西按 `turned` 的轴判 `on`（贴在它侧面的 `sticker` 得到的是 `sticker against turned.right`——6.5.3 同样在 `turned` 的局部系里判定）；`tilt=20` 的组里立在 `body` 顶面上的东西是 `on body`。倒扣的 $B$（局部 $+y$ 朝下）的"顶"在世界的下方，立在它世界上方的东西**不是** `on` 它（它们与 $B$ 的关系落到 6.5.3–6.5.5）。
- 最低处取 $A$ **全部**块的最小值：多块的 $A$（台阶、扫掠）要整体的最低点贴着 $B$ 的某块的顶才算；basics 的 `stairs`（8 级踏步，最高一级与桌面齐平、贴着桌子的侧面）整体的最低点在地面附近、不在桌顶，所以不是 `on`，是 `stairs touching table`（gap 0）。
- 水平重叠要**严格大于** $\varepsilon$：只在一个角或一条棱上碰到不算 `on`。
- 球与圆柱按它们的凸块投影：球在桌上的最低点是球心减半径，水平重叠是球在 $e_x$、$e_z$ 上的直径区间（`ball on table`）；12 边棱柱的顶是顶环的最高点。
- `on` 的两个方向都试（6.5.1 的第 1、2 步），所以 `$B$ 立在 $A$ 上`也能得到；两者同时成立（要求两者沿 $u$ 的厚度之和小于 $2\varepsilon$，例如两块薄板互相在容差内叠放）时取 $P$ on $Q$——$P$ 是扫描序靠后者，即使它在下面也如此（6.8.4 的 `low2 on up1`）。
- 沉入超过 $\varepsilon$ 的物体（`on=floor gap=-0.1`）不是 `on`，是 `intersects`（gap $-0.1$）；悬空不超过 $\varepsilon$ 的物体（`gap=0.015`）是 `on`，`gap` 为 0.015。

### 6.5.3 against

"$A$ against $B$.面"：两个盒类对象（6.2.2）贴面，$A$ 是较小者，在 **$B$ 的局部系**里判定，面是 $B$ 局部包围盒的 left / right / back / front 之一（2.1.1）。条件：

1. $\mathrm{boxy}(A)$ 且 $\mathrm{boxy}(B)$；
2. **轴对齐**：$R_{A \to B} = R_B^{\mathsf T} R_A$ 的每个元素 $r$ 满足 $\lvert r \rvert \le \varepsilon_{\text{dir}}$ 或 $\big\lvert\, \lvert r \rvert - 1 \,\big\rvert \le \varepsilon_{\text{dir}}$（$A$ 的轴与 $B$ 的轴平行或反平行，允许镜像）；
3. 记 $a$ = $A$ 的局部包围盒 $B_A$ 的 8 个角经 $T_{A \to B} = W_B^{-1} \circ W_A$ 变换后的轴对齐盒（在 $B$ 的局部系里），$b = B_B$；$o_k = \mathrm{ov}_k(a, b)$，$k \in \{x, y, z\}$；
4. $o_y > \varepsilon$（竖直方向有实在的重叠：贴面不是叠放），且按下面的顺序取第一个成立的面：

| 面 | 条件 |
|---|---|
| `left` | $\lvert a^{\max}_x - b^{\min}_x \rvert < \varepsilon$ 且 $o_z > \varepsilon$（$A$ 在 $B$ 的 $-x$ 一侧，贴着 $B$ 的左面） |
| `right` | $\lvert a^{\min}_x - b^{\max}_x \rvert < \varepsilon$ 且 $o_z > \varepsilon$ |
| `back` | $\lvert a^{\max}_z - b^{\min}_z \rvert < \varepsilon$ 且 $o_x > \varepsilon$ |
| `front` | $\lvert a^{\min}_z - b^{\max}_z \rvert < \varepsilon$ 且 $o_x > \varepsilon$ |

5. 成立时事实为 `against`，$a = A$、$b = B$，`text` 是 `<A> against <B>.<面>`，`gap` $= \mathrm{gap}(P, Q)$（6.4.3 的对象间隙，由真实凸块算出，可以是 $[-\varepsilon, \varepsilon]$ 里的任何值）。

说明：

- `top` / `bottom` 永远不是 `against` 的面：叠放由 `on` 覆盖（6.5.1 先试 `on`）。
- 盒类对象的面按**局部包围盒**取，楔形的斜面不是面：盒子贴着楔形的竖直高面（$+x$ 面）是 `against ramp.right`；搭在斜面上的盒子与楔形之间没有满足第 4 条的面，落到 `touching` / `intersects`。
- 两个盒子的面相贴但错开，使 $o_z \le \varepsilon$（只在一条竖棱上碰到）：不是 `against`，是 `touching`。
- 非盒类对象（圆柱贴墙、球靠箱）不产生 `against`，它们的接触是 `touching`。
- `approx`：mesh 的外包络与布尔容器的代理盒是盒类，它们的 `against` 事实带 `approx: true`。原型 `contactProvider` 对 `against` 一律不标 approx，且不把 mesh 当盒类（它要求凸块精确）；本版按 6.2.2 的盒类定义，理由是贴面是外包络层面的陈述，与 `on` 一样可以在近似几何上成立、由 `approx` 标出。

### 6.5.4 inside

"$A$ inside $B$"：$A$（较小者）的全部凸块都在 $B$ 的**唯一**凸块里（允许 $\varepsilon$ 的外溢）。条件：

1. $B$ 恰有一个凸块 $K_B$（box、wedge、cyl、cone、sphere、hull、凸轮廓的 plate、revolve、梁、杆、mesh、布尔容器、单段扫掠；多块的台阶、多段扫掠、凹轮廓的 plate 不能作 $B$）；
2. $V_A < \kappa_{\text{in}}\, V_B$（局部包围盒体积，$\kappa_{\text{in}} = 0.9$：体积相近的两个物体即使一个套着另一个也不算 `inside`，而是 `intersects`）；
3. 包含判定：
   - $K_B$ 是球 $(c, r)$：$A$ 的每一块 $K$，球 $(c', r')$ 要 $\lVert c' - c \rVert + r' \le r + \varepsilon$，多面体要 $\max_v \lVert v - c \rVert \le r + \varepsilon$；
   - $K_B$ 是多面体：对 $K_B$ 法线集里的每个方向 $\nu$，记 $[l, h] = \mathrm{proj}(K_B, \nu)$，$A$ 的每一块 $K$ 的 $[l', h'] = \mathrm{proj}(K, \nu)$ 要满足 $h' \le h + \varepsilon$ 且 $l' \ge l - \varepsilon$。凸多面体的面法线是完备的分离轴集合，所以这是"$A$ 的每块都在 $K_B$ 向外加厚 $\varepsilon$ 的像里"（对非轴对齐的 $A$ 也精确）。
4. 成立时事实为 `inside`，$a = A$、$b = B$，`gap` $= \mathrm{gap}(P, Q)$（通常为负，是沿 $K_B$ 某个候选轴的穿透深度的下界；贴在内壁上的物体可以得到 $-\varepsilon$ 到 $\varepsilon$ 之间的值）。

说明：

- `inside` 看的是 $B$ 的凸块，不是 $B$ 的真实形状：$B$ 是 `subtract` 时 $K_B$ 是第一个操作数的盒（2.5.6），空腔里的东西是 `inside`（basics 的 `orb inside crate`、spaceport 的 `spare inside hangar.body`，`approx: true`）；$B$ 是 revolve 时 $K_B$ 是外接棱柱，花瓶口里的东西是 `inside`（`approx: true`）。原型 `insideTest` 要求 $B$ 的凸块**精确**（它的布尔容器凸块没有 approx 标记，所以空腔仍得到 inside；revolve 得到 intersects）；本版允许近似的 $B$ 并标 approx，理由是 2.5.6 的要求与"近似几何参与全部判定、由 approx 标出"的统一规则（6.2.2）。
- $\varepsilon$ 的外溢使"薄片贴在盒子某个面的外侧、整体在该面的投影范围里"也满足第 3 条（例如 5 毫米厚的板平贴在盒子顶面上：先试的 `on` 成立，所以是 `on`；平贴在倒扣盒子的世界上方时 `on` 不成立，得到 `inside`、gap $\approx 0$）。这是容差的后果，不是错误。
- 第 2 条用局部包围盒体积，不是真实体积；球 inside 盒、盒 inside 球都按此判。

### 6.5.5 touching 与 intersects

两者都不成立时，由间隙决定：

- $-\varepsilon \le \mathrm{gap}(P, Q) \le \varepsilon$：`touching`——相切或在容差内相贴 / 相嵌（斜着搭的梁与地面、两个相切的球、盒子只在一条棱上碰到另一个盒子、台阶靠着桌边）；
- $\mathrm{gap}(P, Q) < -\varepsilon$：`intersects`——穿透超过容差（沉入地面的物体、互相穿过的风车叶片、从圆盘里伸出的馈源）。

$a = A$（较小者）、$b = B$，`gap` $= \mathrm{gap}(P, Q)$。

### 6.5.6 $a$ / $b$ 的顺序与对称性

| 关系 | $a$ | $b$ | 几何上对称？ | 一对对象最多几条 |
|---|---|---|---|---|
| `on` | 立在上面的 | 托着的 | 否（6.5.1 两个方向都试） | 1 |
| `against` | 局部包围盒体积较小者 | 较大者；面是 $b$ 的 | 否 | 1 |
| `inside` | 较小者 | 较大者 | 否 | 1 |
| `touching` | 较小者 | 较大者 | 是（事实的 $a$ / $b$ 仍按体积排） | 1 |
| `intersects` | 较小者 | 较大者 | 是 | 1 |

体积相等时 $a = P$（扫描序靠后者：对象盒 $x$ 最小值较大者，仍相等时节点顺序靠后者）。镜像副本与原件、同一阵列里相邻的实例体积相等，它们之间的 `touching` / `intersects` / `against` 由这条规则定 $a$（semantics 的 `street.h1.half2 against street.h1.half.right`：`half2` 是镜像副本，对象盒更靠 $+x$，所以是 $a$）。一对对象至多一条事实：6.5.1 对每个候选对只产出一条。

### 6.5.7 与定位关系的关系

事实**只看几何**。写了 `on=B` 的物体与 $B$ 之间不自动有 `on` 事实：它可能因为 `gap`、`offset`、`align`、继承的朝向而离开了 $B$ 的顶面，或者 $B$ 是容器（容器不是对象，事实落在容器里的实体上）；反过来，没有写任何关系、用 `at=` 放到位的物体照常得到事实。第 3 章的 `on` / `under` / `against` / `inside` / `at` 决定**位置**，本章的 `on` / `against` / `inside` / `touching` / `intersects` 描述**结果**；两套词同名是为了让"按关系写出来的场景，事实里就是那个关系"在常见情形下成立：

| 写法 | 事实 |
|---|---|
| `box a on=B` | `a on B`（$B$ 是实体；默认 gap 0，底贴顶） |
| `box a on=B gap=0.015` | `a on B`，gap 0.015 |
| `box a on=B gap=-0.1` | `a intersects B`，gap $-0.1$ |
| `box a on=B tilt=10` | `a intersects B`（一个角沉入；6.5.2 第 1 条不成立） |
| `box a under=B` | 没有 `under` 事实；`B on a`（$B$ 立在 $a$ 上）当 $a$ 的顶与 $B$ 的底相贴 |
| `box a against=B.left align=bottom` | `a against B.left`（都是盒类、轴对齐）；$B$ 是圆柱时 `a touching B` |
| `sphere a inside=B` | `a inside B`（$B$ 单块、体积够大）；$B$ 是 group 时事实落在 group 里的实体上 |
| `box a at=10,0,0` | 按几何：与它碰到的东西有事实，与写法无关 |
| `box a on=G`（$G$ 是 group） | `a on <G 里的某个实体>`（若贴着）；$G$ 本身不是对象 |

工具用事实判断"坐标依赖"时读的就是这些结果，所以"全锚到地面"的场景只有与地面的接触事实。

## 6.6 输出形状与确定性

### 6.6.1 事实

每条 `contact` 事实是一个对象，字段固定、顺序固定：

| 字段 | 类型 | 定义 |
|---|---|---|
| `rel` | 字符串 | `on` / `against` / `inside` / `touching` / `intersects` |
| `a` | 字符串 | 对象 $a$ 的**完整路径**（1.6.10 的输出形式：段用 `.` 连接），不带 `.left`、`.top` 之类的锚点后缀 |
| `b` | 字符串 | 对象 $b$ 的完整路径，同样不带锚点后缀（`against` 的面只进 `text`）；`contact` 的事实都是二元的，其他提供者的一元事实为 `null` |
| `text` | 字符串 | `<a> <rel> <b>`，`against` 为 `<a> against <b>.<面>`；三部分用一个空格分开 |
| `gap` | 数 | 6.5 各节给出的间隙（场景单位），双精度，**不**四舍五入 |
| `eps` | 数 | $\varepsilon$（场景单位，6.1.3） |
| `approx` | 布尔 | 6.5.1：两对象 `approx` 之或；总是输出 |
| `provider` | 字符串 | `"contact"` |

原型把 `gap` 四舍五入到 4 位小数（`+gap.toFixed(4)`）并只在为真时输出 `approx`；本版输出完整的双精度与总是存在的布尔。验收比对 `gap` 用容差（6.8.2；09-compat.md 的 9.4.3）。语义层的关系索引（6.7.1、7.2.4）只读每条事实的 `rel`、`a`、`b`，所以 `a`、`b` 必须是能在场景树里解析到节点的路径；`text`、`gap` 等其他字段只给工具与验收读。

**顺序。** `contact` 的事实按候选对的枚举顺序（6.2.3：$(\sigma(P), \sigma(Q))$ 的字典序）输出，没有事实的候选对不占位。参考输出的 1414 条事实恰是这个顺序（6.8.3）。

### 6.6.2 顶层字段

| 字段 | 类型 | 定义 |
|---|---|---|
| `facts` | 列表 | 全部启用的提供者的事实，按提供者启用顺序分组 |
| `factsSkipped` | 布尔 | 是否没有推导事实（6.6.4） |
| `factProviders` | 字符串列表 | 启用的提供者名，按启用顺序（缺省 `["contact"]`） |

### 6.6.3 确定性

事实的推导是对象列表的纯函数，必须在任何平台、任何线程数下逐位相同（1.1.3）。本章为此固定了：对象顺序（节点顺序）、扫描序、候选对与块对的枚举顺序、候选轴的构成顺序、每条判定的不等式与比较顺序、$a$ / $b$ 的平局规则、输出顺序。实现上的要求：

- 所用的运算只有加减乘除、比较、`sqrt`（归一化）、`max` / `min`；`max` / `min` 与顺序无关，所以并行计算不同候选对的结果再按 $(\sigma(P), \sigma(Q))$ 归并与串行结果逐位相同。并行只允许按 $P$ 的扫描序号静态分块、按块号顺序拼接（不得用依赖线程数的归约）。
- 不得用散列顺序决定任何枚举；不得用"先算出一个近似再细化"的策略改变比较的结果。
- 凸块、法线集、棱集按第 2 章给出的顶点与面片顺序构造（去重保留先出现的方向），变换到世界系时按 $R_n v + t_n$ 的分量顺序计算；这些是第 2、3 章的责任，本章只消费。
- 事实不是"在预算内尽力算"的量：本章没有任何上限、步数、时间的概念；要么全部推导（6.5），要么整体不推导（6.6.4）。

### 6.6.4 不推导

宿主可以要求编译**不推导事实**（例如只为看图；也可以按场景规模自动决定，阈值是工具约定，不在本规范内）。此时：

- `facts` 为空列表，`factsSkipped` 为 `true`，`factProviders` 仍列出本应启用的提供者；
- 关系索引（6.7）为空；语义层里每条调用了关系谓词的语句得到空集，并记一条警告"空间事实未推导，X() 为空集"（每条语句一条，6.9）；pattern 的关系约束同样匹配不到；
- 节点、条目、曲线、材质、错误等其他输出不受影响；
- 实现**不得**部分推导（只算一部分对象或一部分对、或在某个预算内停下）后把 `factsSkipped` 写成 `false`。

不跳过时 `factsSkipped` 为 `false`，`facts` 是 6.5 定义的完整列表。

## 6.7 关系索引与语义层的接口

### 6.7.1 上提

语义层的关系谓词不直接读事实列表，读由它派生出的**关系索引** $I_r$（$r \in$ {`on`, `supports`, `against`, `inside`, `contains`, `touching`, `intersects`}），每个 $I_r$ 是有序对 $(x, y)$ 的集合，$x$、$y$ 是任意节点（不限于对象）。构造：对每条 `contact` 事实 $(r, A, B)$，记 $\mathrm{chain}(n)$ 为从 $n$ 到最外层节点的祖先链（含 $n$，不含根），$C = \mathrm{chain}(A) \cap \mathrm{chain}(B)$（公共祖先）。对每个 $x \in \mathrm{chain}(A) \setminus C$、$y \in \mathrm{chain}(B) \setminus C$，按表加入：

| 事实的 $r$ | 加入 $I$ 的对 |
|---|---|
| `on` | $(x, y) \in I_{\text{on}}$，$(y, x) \in I_{\text{supports}}$，$(x, y), (y, x) \in I_{\text{touching}}$ |
| `against` | $(x, y), (y, x) \in I_{\text{against}}$，$(x, y), (y, x) \in I_{\text{touching}}$ |
| `inside` | $(x, y) \in I_{\text{inside}}$，$(y, x) \in I_{\text{contains}}$ |
| `touching` | $(x, y), (y, x) \in I_{\text{touching}}$ |
| `intersects` | $(x, y), (y, x) \in I_{\text{intersects}}$，$(x, y), (y, x) \in I_{\text{touching}}$ |

即关系从对象**上提**到它们的祖先容器（阵列、组、模板实例、副本组、copy 节点、隐藏的原物体），但不上提到两者的公共祖先：`rocket.stage1 on ground` 使 `rocket on ground`、`ground supports rocket`；`rocket.stage2 on rocket.stage1` 不使 `rocket on rocket`（`rocket` 是公共祖先）。`touching` 在索引里是"任何接触"：`on`、`against`、`intersects` 都加入 $I_{\text{touching}}$，`inside` 不加入（6.8.4 的小例子 `touching(all(), 2)` 包含 `ground`，`contains(all())` 只有 `flip`）。索引只读每条事实的 `rel`、`a`、`b`（7.2.4）：`rel` 不是五个词之一、或 `b` 为 `null` 的事实（其他提供者的，6.3.2）不进索引。索引里的节点都在 `all()` 的范围内（1.7.2：对象与其祖先都不是皮肤、不是根；皮肤的不可见性见 7.8.4）；被变体隐藏的宿主与被选中的宿主照常在索引里（6.2.1 第 3 条）；退化节点不在（6.2.1 第 4 条）。

### 6.7.2 关系谓词

1.5.6 的七个函数 `on supports against inside contains touching intersects` 的值：
$$\mathrm{rel}(S, k) = \{\, x : \lvert \{ y \in S : (x, y) \in I_{\text{rel}} \} \rvert \ge k \,\},$$
$S$ 是集合类的值（任何节点的集合，含容器），$k$ 是与某个整数 $m \ge 1$ 相差不超过 $\varepsilon_{\text{int}}$（03-solving.md 的 3.1.7）的数，取 $m$（缺省 1）；$S$ 不是集合类、$k$ 不满足这一条的报错文案在 6.9（与 1.5.6、7.2.4 相同）。结果按节点顺序排列（1.6.1：集合顺序用节点顺序）。例如 `on(ground)` 是直接立在 `ground` 上的实体及其非公共的祖先容器；`on(pier, 2)` 是至少架在两根 `pier` 上的节点；`supports(kind(box))` 是托着至少一个盒子的节点；`touching(all(), 2)` 是与至少两个节点有接触的节点。$S$ 为空集时结果为空集；$x$ 与 $S$ 的成员是同一节点时不计（$I$ 里没有 $(x, x)$）。

关系谓词只能在求解之后使用（外观时机的 over、style、variant、concept、pattern、check）；几何时机的选择器里调用报错（1.7.4 的文案）。它们的详细求值次序与集合语义在 07-semantics.md。

### 6.7.3 事实未推导时

`factsSkipped` 为 `true` 时索引为空，谓词按 6.7.2 得到空集并给 6.6.4 的警告。实现不得在这种情况下按定位属性（`on=`）伪造关系。

## 6.8 用参考输出验证

### 6.8.1 参考输出的统计

脚本（scratchpad 的 `stats.mjs`、`shape.mjs`、`csgfacts.mjs`）对 7 个示例的 `corpus/reference/examples/<名>.compile.json` 统计：

| 文件 | 节点 | 对象 | 事实 | on | against | inside | touching | intersects |
|---|---|---|---|---|---|---|---|---|
| basics | 18 | 16 | 20 | 11 | 6 | 1 | 2 | 0 |
| curves | 94 | 62 | 69 | 56 | 4 | 0 | 2 | 7 |
| rotation | 615 | 321 | 562 | 237 | 2 | 0 | 229 | 94 |
| semantics | 267 | 99 | 118 | 104 | 8 | 0 | 0 | 6 |
| spaceport | 193 | 111 | 203 | 40 | 3 | 1 | 12 | 147 |
| spaceport_blueprint | 240 | 111 | 203 | 40 | 3 | 1 | 12 | 147 |
| spaceport_edit | 217 | 125 | 239 | 42 | 3 | 1 | 12 | 181 |
| 合计 | 1644 | 845 | 1414 | 530 | 29 | 4 | 269 | 582 |

- `rel` 只取五个值；`provider` 全是 `contact`；`eps` 全是 0.02（`metre = 1`）；`factsSkipped` 全为 `false`，`factProviders` 全为 `["contact"]`。
- 字段形状：1404 条是 `rel,a,b,text,gap,eps,provider`，10 条多一个 `approx: true`（curves 的 4 条 `on`：两座 revolve 塔与 `plaza.basin` / `plaza.spout`；rotation 的 3 条 `intersects`：`radar.horn / feed / mast intersects radar.dish`，dish 是 revolve；spaceport 三个文件各 1 条 `rocket.nose on rocket.stage2`，nose 是 revolve）。
- `text` 格式：`<a> <rel> <b>`，`against` 带 `.<面>`；面只出现 `front`（6）、`right`（15）、`left`（8），没有 `back`。
- `gap`：`on` 530 条全为 0；`against` 29 条全为 0；`inside` 4 条在 $[-4.8, -3.1]$；`touching` 269 条在 $[-0.0197, 0.0177]$（48 条为 0、141 条负、80 条正）；`intersects` 582 条在 $[-8.5001, -0.0202]$。与 6.5 的区间一致（`touching` 落在 $[-\varepsilon, \varepsilon]$，`intersects` $< -\varepsilon$）。
- 参与者的种类：实体（box、cyl、cone、sphere、wedge、revolve、span 的三种形态、sweep）、影子（rotation 216 条事实涉及影子，curves 28、spaceport 71）、隐藏的原物体（semantics 40 条、spaceport_blueprint 3 条）、布尔容器（basics 的 `crate`、spaceport 的 `hangar.body`、spaceport_edit 再加 `hangar2.body`）；没有曲线、容器、皮肤。spaceport 与 spaceport_blueprint 的 203 条事实逐条相同（变体不改事实）。

### 6.8.2 方法

脚本 `verify6.mjs` 从原型**只取**第 2 章与第 3 章的结果——每个对象的世界凸块（原型 `localParts` + `worldPart`）、局部包围盒与世界变换（经原型的提供者接口 `registerFactProvider` 捕获 `objs`，不运行原型的 `contactProvider`）——然后按本章独立实现 6.2.3 的扫描序与候选对、6.4 的投影区间 / 分离量 / 对象间隙、6.5.1 的判定顺序与五个谓词、6.5.6 的平局规则，再与参考输出的 `facts[]` 比：以 (rel, a, b) 为键比成员；`gap` 容差 $10^{-4}$（参考输出四舍五入到 4 位）；比 `approx`；比 `against` 的面；比输出顺序。按 2.5.6 把布尔容器的 `approx` 置为 `true`（原型的布尔容器没有 approx 标记）。

### 6.8.3 结果

| 文件 | 候选对 | 间隙 $\le \varepsilon$ 的对 | 本章得到 | 参考 | (rel, a, b) 一致 | gap 偏差 $> 10^{-4}$ | 只有 approx 不同 | 缺少 | 多出 |
|---|---|---|---|---|---|---|---|---|---|
| basics | 20 | 20 | 20 | 20 | 20 | 0 | 2 | 0 | 0 |
| curves | 87 | 69 | 69 | 69 | 69 | 0 | 0 | 0 | 0 |
| rotation | 784 | 562 | 562 | 562 | 561 | 0 | 0 | 1 | 1 |
| semantics | 118 | 118 | 118 | 118 | 118 | 0 | 0 | 0 | 0 |
| spaceport | 231 | 203 | 203 | 203 | 203 | 0 | 5 | 0 | 0 |
| spaceport_blueprint | 231 | 203 | 203 | 203 | 203 | 0 | 5 | 0 | 0 |
| spaceport_edit | 275 | 239 | 239 | 239 | 239 | 0 | 7 | 0 | 0 |
| 合计 | 1746 | 1414 | 1414 | 1414 | 1413 | 0 | 19 | 1 | 1 |

- 1413 条的 (rel, a, b)、`gap`（偏差 $< 10^{-4}$）、`against` 的面全部一致，含 13 对体积相等的 $a$ / $b$ 平局（rotation 的 9 对风车叶片 `intersects`、semantics 的 4 对镜像半屋 `against`）——都按 6.5.6 的规则得到与参考相同的 $a$（脚本 `06/ties.mjs`：13 对都由对象盒的 $x$ 最小值分出扫描序，没有一对需要用节点顺序；884 条非 `on` 事实的 $a$ 体积都不大于 $b$）。
- **输出顺序**：7 个文件的 1414 条事实按 6.6.1 的顺序（候选对 $(\sigma(P), \sigma(Q))$ 的字典序）零处违反（脚本 `order.mjs`）。
- 19 条只有 `approx` 不同：全部是一方为布尔容器的事实（basics 的 `crate on floor`、`orb inside crate`；spaceport 三个文件的 `hangar.body on ground`、`hangar.roof on hangar.body`、`spare inside hangar.body`、`wings.annex against hangar.body.right`、`wings.mirrored.annex against hangar.body.left`，spaceport_edit 再加 `hangar2.body on ground`、`hangar2.roof on hangar2.body`）。参考输出不标 approx，本版标 `true`——2.5.6 规定布尔容器的代理盒 `approx=true`。
- 1 条关系不同：rotation 的 `radar.feed` 与 `radar.dish`（revolve，单块外接棱柱）：参考为 `intersects`（gap $-2.8$，approx），本版为 `inside`（gap $-2.8$，approx）。原因是 6.5.4 允许近似的 $B$（原型 `insideTest` 要求 `convex` 即精确单块）。feed 的凸块（12 边杆）完全落在 dish 的外接棱柱里，所以按 6.5.4 是 `inside`；`radar.horn`、`radar.mast` 伸出了棱柱，仍是 `intersects`，与参考一致。
- 没有其他差异：每个间隙 $\le \varepsilon$ 的候选对恰得到一条事实，数量与参考相同。

验证里凸块取自原型；第 2 章与原型凸块的已知差别（2.2.3、2.2.9、2.1.6.2 以"原型 …；本版 …"标出，2.8.3 列表）对本节的影响：圆锥顶点（2.2.3 用一个顶点，原型用半径 $10^{-4}$ 的小环；gap 的差 $\le 10^{-4}$，rotation 有 6 条涉及 cone 的事实）、凹截面扫掠段（2.2.9 取凸包并标 approx，原型直接用截面环；语料的扫掠截面都是凸的）、凹轮廓 plate 的三角化（2.1.6.2 的耳切与原型 `triangulate2` 可能切出不同的三角形；语料的事实不涉及 plate）、12 边形的起始角与绕向（顶点集合相同，去重后的法线集与棱集相同）。

### 6.8.4 小例子

用原型编译小例子（scratchpad 的 `edge.s3d`、`edge2.s3d`，以及 `06/ex623.s3d`、`06/edge3.s3d`–`06/edge6.s3d`、`06/prov.mjs`）确认未被语料覆盖的情形；"本版"一列是按本章的判定：

| 情形 | 原型 | 本版 |
|---|---|---|
| `box flip size=2,2,2 roll=180 pin=top on=ground`（倒扣） | `flip on ground`（$\lvert R_{yy} \rvert = 1$） | 相同（6.5.2 第 1 条允许反向） |
| `box tilted size=2,2,2 on=floor tilt=10` | `tilted intersects floor`，gap $-0.1736$ | 相同 |
| `on=floor gap=-0.1` / `gap=0.015` | `intersects` $-0.1$ / `on` 0.015 | 相同 |
| 5 毫米薄板 `thin1 on=floor`，`thin2 on=thin1` | `thin2 on floor`（gap 0.005）与 `thin2 on thin1` 都有 | 相同（一个物体可以 `on` 多个） |
| 薄板平贴在倒扣盒子的世界上方 | `thin1 inside upside`，gap 0 | 相同（6.5.4 的说明） |
| `box onramp at=ramp.right pin=left`（盒贴楔形高面） | `onramp against ramp.right` | 相同 |
| `sphere small r=0.5 on=big`（球叠球）；两球相切 | `small on big`；`s2 touching s1` | 相同 |
| `sphere inner inside=bigbox`；`box innerbox inside=bigsph` | `inside` $-4$ / `inside` $-4.5$ | 相同 |
| `box incrate` 放在 `subtract crate` 的空腔里 | `incrate inside crate`（不标 approx） | `inside`，`approx: true`（2.5.6） |
| `box tiny` 放在 `revolve vase` 的口里 | `tiny intersects vase`，approx | `tiny inside vase`，approx（6.5.4） |
| `mesh m … against=wall.front` | `m touching wall`，approx（mesh 不是 boxy） | `m against wall.front`，approx（6.5.3） |
| `box exact size=1,1,1 at=0,0.02,0`（与 `ground` 顶面恰相距 $\varepsilon$；`at=0,0.019,0` 作对照） | 没有事实（`broadPairs` 对 $y$、$z$ 用 `> -margin`，恰好相等被排除；0.019 时 `exact on ground` 0.019）。沿 $x$ 恰相距 $\varepsilon$ 的 `box ax2 at=21.02,0,0` 与 `ax1 at=20,0,0` 则保留（$x$ 用 `>=`）：`ax2 against ax1.right` 0.02 | `exact touching ground`，gap 0.02（6.2.3 三个方向统一用 $\ge -\varepsilon$；只在恰好相等时不同）；`ax2 against ax1.right` 相同 |
| `box up1 size=1,0.013,1 at=0,0.005,0`、`box low2 size=1,0.005,1 at=0,0,0`（两块薄板，两个方向的 `on` 都成立） | `low2 on up1`，gap $-0.018$（$P$ = 节点顺序靠后的 `low2`，先试 $P$ on $Q$） | 相同（6.5.2 说明：取 $P$ on $Q$） |
| `sphere ca r=1`、`sphere cb r=2` 同心（`at=20,0,0 pin=center`） | `ca inside cb`，`gap` 输出为 `null`（`partGap` 没有候选轴，$-\infty$） | `ca inside cb`，gap $-3 = -(r_A + r_B)$（6.4.2） |
| 6.2.3 的示例（`floor`、`table`、`ball`、`post`） | `table on floor`、`ball on table` 两条 | 相同（`ball` 与 `floor` 沿 $y$ 分开 3，不是候选对） |
| `concept lifted = on(ground)`，`rocket` 组里 `stage1 on=ground` | 成员含 `rocket.stage1` 与 `rocket` | 相同（6.7.1 上提） |
| `concept twoleg = on(kind(box), 2)`，桌板架在两条腿上 | 成员 `table` | 相同（6.7.2 的 $k$） |
| `touching(all(), 2)`；`contains(all())` | 含 `ground`（supports 也算 touching）；只含 `flip` | 相同（6.7.1 的表） |

### 6.8.6 实战工程核对（2026-10-13 补）

三个实战工程（3.11.6 的同一批 compile JSON；$\varepsilon = 0.02 \cdot \mathtt{metre} = 0.02 / 0.28 = 0.0714$ stud，与参考输出每条事实的 `eps` 相同——6.1.3 的单位换算由此得到验证）用 `spec/tools/verify/verify6-work.mjs` 按本章定义重算全部事实，候选对按 6.2.3 的扫描序用扫掠线取得：

| 工程 | 对象 | 候选对 | 事实（参考 = 本版） | (rel, a, b)、gap、approx 全同 | rel 不同 | gap 不同 | approx 不同 | $a$ / $b$ 平局对 | $a$ / $b$ 顺序不同 |
|---|---|---|---|---|---|---|---|---|---|
| dorm | 2182 | 9987 | 9683 | 8414 | 1269 | 0 | 0 | 204 | 0 |
| h1 | 2065 | 15469 | 15232 | 10212 | 5020 | 19 | 0 | — | 0 |
| sc | 102843 | 411947 | 275343 | 274604 | 739 | 350 | 13820 | 18880 | 0 |

不一致的每一条都落在 6.8.5 已列的三类里，没有新类：

- **rel 不同**：参考 `intersects` / `touching` → 本版 `inside`（dorm 1231、h1 4959、sc 739），全部是 $B$ 为 `mesh` 外包络（道路贴花、楼体 mesh）的近似容器，6.5.4 允许近似 $B$；参考 `touching` → 本版 `against`（dorm 38、h1 61），全部是 mesh 外包络的贴面，6.5.3。两类事实都带 `approx: true`。
- **approx 不同**：sc 的 13 820 条全部有一方是布尔容器（`subtract` 做的楼板 `group2.floor` 等 472 个），6.5.1 按 2.5.6 标 `approx`，原型不标。
- **gap 不同**：h1 的 19 条与 sc 的 350 条参考 `gap` 为 `null`——不是同心球，而是两个**零尺寸**的盒（`box body size=1,0,0` 一类，原型逆向对退化部件的输出）重合：零长度的棱使全部候选轴为零向量，原型 `partGap` 得 $-\infty$。本版按 2.2 这些实体是报错的退化节点，不参与事实（6.2.1），所以这些对在本版里不存在；是否改为允许零尺寸见 09-compat.md 9.5.5。
- **$a$ / $b$ 的平局**（体积相等，6.5.1 按扫描序取靠后者为 $a$）：dorm 204 对、sc 18 880 对全部与参考一致；本节脚本最初按节点顺序取对时有 125 对不同，改按 6.2.3 的扫描序后归零——扫描序是定义的一部分，不是实现细节。

### 6.8.5 因本章定义而预期不同的项

M1 的编译器按本章实现时，与参考输出的差别只有下列情形（第 9 章汇总）：

| 节 | 情形 | 原型 | 本版 | 理由 |
|---|---|---|---|---|
| 6.2.2、6.5.1 | 一方是布尔容器的事实 | 不标 approx（`localParts` 对没有 `geom` 的节点返回 `approx: false`） | `approx: true` | 2.5.6：代理盒是近似；语料 19 条 |
| 6.5.4 | $B$ 的唯一凸块是近似的（revolve、mesh、布尔容器、凹截面扫掠段） | 不判 `inside`（`insideTest` 要求 `convex`），得 `intersects` / `touching` | 判 `inside` 并标 approx | 与 2.5.6 对布尔容器的要求一致；近似几何的统一规则；示例 1 条（`radar.feed`）；exports 预计 308 条（下文） |
| 6.5.3 | mesh 外包络贴面 | `touching`（`boxy` 要求精确） | `against`，approx | 同上；示例无；exports 预计 37 条（下文） |
| 6.6.1 | `gap` 的精度 | 四舍五入到 4 位小数（`toFixed(4)`） | 完整双精度 | 小单位场景（stud）下 4 位不够；确定性按逐位 |
| 6.6.1 | `approx` 字段 | 只在为真时出现 | 总是输出布尔 | 形状固定 |
| 6.2.3、6.4.3、6.5.3 | 对象盒 / 块盒恰好相距 $\varepsilon$ 的对；`against` 轴对齐判定的元素恰差 $\varepsilon_{\text{dir}}$ | `broadPairs` 对 $y$ / $z$ 用 `> -margin`（排除恰好相等）、对 $x$ 用 `>=`（保留），`gapBetween` 用 `< -eps` 跳过（保留恰好相等），三处不一致；`isAxisAligned` 用严格的 `< 1e-6` | 三处统一为重叠量 $\ge -\varepsilon$ 保留；轴对齐用 $\le \varepsilon_{\text{dir}}$（与 6.5.2 第 1 条、2.1.4 一致） | 边界只按一条不等式；语料无（6.8.4 的 `exact` / `ax2`） |
| 6.4.2 | 同心球（两个球的球心重合） | 没有候选轴，`gap` 为 $-\infty$，JSON 输出 `null` | `gap` $= -(r_A + r_B)$，是数 | `gap` 必须是数（6.6.1）；语料无（6.8.4） |
| 6.6.4、6.7.3 | 不推导事实时调用关系谓词 | 静默得到空集 | 空集并警告 | 1.1.1：不得静默 |
| 6.3.2 | 请求了不存在的提供者 | 报错并用缺省（`compileOnce`） | 相同 | — |
| 6.3.2 | 提供者出错 | 只丢该提供者的事实（`deriveFacts` 的 `onError`） | 相同 | — |

**对 exports 的影响量级。** 7 个示例里上面两行只命中 1 条，但 `corpus/exports/` 的 10 个文件里有 8 个受影响很大：逆向把 revolve、sweep 与布尔体写成了引用源网格的 `mesh`，其外包络（2.7）是近似的单个盒形凸块，于是按 6.5.4 任何落在该盒里的较小物体都得到 `inside`，按 6.5.3 轴对齐贴着该盒侧面的盒类得到 `against`。09-compat.md 的 9.5.3 用参考输出的盒按本章推算（脚本 `c9/predict.py`，是预测，M1 用实现跑出后逐条核对）：全部 exports 共 345 条事实的 `rel` 预期改变——`intersects` → `inside` 308 条、`touching` → `against` 37 条（`against` 的 `text` 随之加面名）；数量集中在 rotation.L0（709 条事实里 249 条 `intersects` → `inside`：247 条是过山车的枕木 `coaster_ties.*.tie` 对三条轨道 `coaster_railL` / `coaster_railR` / `coaster_spine`——逆向把扫掠轨道写成 mesh，细长的外包络盒把途经的枕木全部"包住"；另 2 条是 `radar_horn` / `radar_feed` 对 `radar_dish`）。这些事实的 `gap` 不变、全部带 `approx: true`，由读事实的一方（check、concept、工具）据此取舍；本章只记录影响，不重开 6.5.4 的定义。依赖"mesh 只会 `touching` / `intersects`"的 check / concept 要复查（9.6.2）。

语料里不涉及、只按原型源码与本章定义写下的规则：`under=` 的事实（6.5.7：没有 `under` 事实）、球与多面体的候选轴（6.4.2 第 4 条；语料的 `ball on table`、`orb inside crate`、`small on big` 覆盖了球—多面体与球—球）、多段扫掠作 $B$ 不判 `inside`（6.5.4 第 1 条）、`metre ≠ 1` 时的 $\varepsilon$（6.1.3；实战工程以 stud 编译时原型用 `factEps = 0.02 / 0.28`，与 $0.02 \cdot \mathtt{metre}$ 相同）、一元事实的输出形状（6.3.2；原型 `deriveFacts` 给没有 `b` 的事实补 `b: null`，小例子 `06/prov.mjs` 确认）。两个方向的 `on` 同时成立与同心球已由 6.8.4 的小例子在原型上确认。

## 6.9 错误与警告

按 01-syntax.md 的 1.10 的格式。本章的判定本身不产生错误：几何不合法的节点在第 2、3 章已退化并被 6.2.1 排除，容差之外的任何构型都有定义（没有事实或某一种事实）。

| 种类 | 触发条件 | 文案（引号里是必须包含的信息） | 恢复 |
|---|---|---|---|
| 提供者错误 | 请求的提供者名不存在（6.3.2） | "没有名为 `X` 的事实提供者（已注册：…）" | 按缺省提供者列表推导 |
| 提供者错误 | 某个提供者推导时抛出（6.3.2） | "事实提供者 `X` 出错：…" | 该提供者的事实全部丢弃，其他提供者照常；错误没有对应语句，归根文件（文件序号 0）、行 0、不带节点路径（05-layers.md 5.2.5） |
| 时机错误 | 几何时机的选择器调用关系谓词（1.7.4、6.7.2） | "`X()` 依赖求解后的空间事实，不能用来决定位置 / 几何" | 该 over / drop 语句无效（1.1.4 第 5 条） |
| 类型错误 | 关系谓词的第一个参数不是集合类（1.5.6、7.2.4） | "`X()` 需要参照集合，如 `X(ground)`" | 该语义语句无效（1.1.4 第 4 条；外观时机的 over 按第 5 条） |
| 类型错误 | 关系谓词的第二个参数不是与某个整数 $m \ge 1$ 相差不超过 $\varepsilon_{\text{int}}$ 的数（1.5.6、6.7.2） | "`X()` 的第二个参数需要不小于 1 的整数" | 同上 |

**警告。** `factsSkipped` 为 `true` 时，每条调用了关系谓词的语句（concept、pattern、check、style、variant、外观时机的 over；pattern 的关系约束算调用）记一条警告"空间事实未推导，`X()` 为空集"（每条语句一条，不论调用了几次；归属该语句），语句照常求值（6.6.4、6.7.3；07-semantics.md 的 7.1.4 与 01-syntax.md 的 1.10.3 引用本条）。本章不定义其他警告；`approx` 不是警告，是事实的字段。
