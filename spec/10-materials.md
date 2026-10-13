# 10 材质与外观

**范围。** 本章定义材质与外观：`material` 语句（位置、属性、材质表、同名覆盖、默认材质、内置材质）；节点的外观属性 `mat=` 与 `color=`；每个节点的**有效材质**及其解析顺序（style > color > mat > 父级 > default > 内置）写成确定的算法；副本、拷贝、变体皮肤、阵列实例、模板实例、容器与布尔容器的材质来源；与语义层（style、外观时机的 over、variant）和 `where` 里 `mat` 字段的接口；输出里材质的形状；本章的诊断。

本章**不**定义：`material`、`mat`、`color` 的语法与类型（见 01-syntax.md 的 1.3.7、1.4.4、1.4.6 节）；over / drop 两个时机的判定与副本、拷贝的生成（见 04-derivation.md）；import 的展开顺序（见 05-layers.md）；style / variant 的选择器求值与两阶段求值（见 07-semantics.md）；渲染器怎样使用材质的各分量（工具约定）。材质不参与几何、定位、空间事实与语义层的求值：改变任何材质定义或外观属性，不得改变任何节点的变换、包围盒与事实。

**依赖的章。** 01-syntax.md（语法、类型、名字解析、错误恢复）、04-derivation.md（副本、拷贝、修改）、05-layers.md（展开后的语句序列）、07-semantics.md（style / variant 的选择集与两阶段求值）。被依赖：01-syntax.md 的 1.7.3（`where` 的 `mat` 字段）、07-semantics.md、09-compat.md。

## 10.1 material 语句

### 10.1.1 位置与形态

语法见 01-syntax.md 的 1.3.7 节：`material 名字 { 属性 | 元数据 }`。本节规定它的语义约束：

1. **位置**：`material` 只能写在文件的最外层（D6）——主文件的最外层，或被 import 的文件的最外层。写在任何块里（group / union / subtract / intersect / repeat / 容器式 mirror、rotate 的块、模板体、copy 块、over 块）报错"material 只能写在最外层"，该语句**无效**：不登记、不产生任何效果（1.1.4 第 4 条）。原型里块内的 `material` 被静默忽略，本版报错。
2. **名字**：必填，是一个标识符，不得是关键字（1.2.5）。缺名字（`material color=#fff`）报错"material 需要名字"，语句无效。材质名有自己的名字空间：它可以与节点名、模板名、let 名、概念名相同而互不影响（1.6.9）；引用材质的地方（`mat=`、`style … mat=`、`where mat == …`）按**材质名字面量**解析（1.4.2 第 1 条），不做对象查找。
3. **不接受**标签与块（1.3.7、1.3.15）；接受元数据（D2），元数据按 08-metadata.md 记录，不参与材质的任何求值。
4. **登记时机**：材质表（10.1.3）由展开后的全部语句序列（05-layers.md）里的 `material` 语句构成，与它们相对于节点语句的先后**无关**：`mat=steel` 可以写在 `material steel …` 之前，也可以写在另一个文件里。

```s3d
material steel  color=#7f8b96 metal=0.6 rough=0.4
material glass  color=#9cc3e0 metal=0.3 rough=0.2 opacity=0.5 ~src="lib"
material plain  color=#cfcabe default=true
```

### 10.1.2 属性与取值

属性集**固定**为下表五个（D17；类型见 1.4.6）。其他任何键报错"material 没有属性 k"；该键被忽略，语句其余部分照常（见本节末的错误规则）。原型接受任意键并原样输出，本版不允许扩展属性：材质的分量就是渲染与导出需要的分量，工具要附加的信息走元数据。

| 属性 | 类型 | 允许的取值 | 缺省时 |
|---|---|---|---|
| `color` | 颜色 | 井号十六进制字面量 `#rgb` / `#rgba` / `#rrggbb` / `#rrggbbaa`（1.2.7） | 内置值 `#c9c9c9` |
| `metal` | 数 | $0 \le v \le 1$ | 内置值 $0.05$ |
| `rough` | 数 | $0 \le v \le 1$ | 内置值 $0.78$ |
| `opacity` | 数 | $0 \le v \le 1$ | 内置值 $1$ |
| `default` | 布尔 | `true` / `false` | `false` |

- **求值时机与环境**：属性值是常量（1.4.1）：在展开时、在**全局帧**里求值（1.5.8），可以引用最外层的 `let` 与 `metre`（所有文件最外层的 `let` 都可见，与书写顺序无关），不能引用对象、模板参数、阵列索引，不能用 `where` / `all()` / 关系谓词；违反的按第 1 章报错"…必须是常量"。原型里 `let` 对 `material` 不可见，本版可见。
- **颜色的规范形式**：颜色字面量解析为四个分量 $(r, g, b, a)$，每个分量是 $0 \dots 255$ 的整数：6 / 8 位形式按两位一组读；3 / 4 位形式每位复制一次（`#abc` = `#aabbcc`，`#abcd` = `#aabbccdd`）；没有 alpha 位时 $a = 255$。大小写不敏感。材质值里的 `color` 分量是**不含 alpha** 的规范字符串 `#rrggbb`（小写）。
- **alpha 位就是 opacity**：颜色字面量带 alpha 位（4 / 8 位形式）时，它等价于同时写了 `opacity=` $a / 255$。同一条 `material` 里既写了带 alpha 的 `color` 又写了 `opacity=` 报错"material X 同时写了颜色的透明度与 opacity"：`opacity=` 生效，alpha 位忽略。
- **范围**：`metal`、`rough`、`opacity` 的值必须是 $[0, 1]$ 内的有限数；超出报错"material X 的 k 需要 0..1 之间的数"。
- `default` 只接受布尔字面量 `true` / `false`（1.4.3）：`default=1`、`default="true"` 报错"default 只能是 true / false"并当作 `false`；`default=false` 就是假——原型里 `default=false` 被当成真，本版不会。
- 同一语句里同名属性重复报错"重复的属性"（1.3.3）：该属性的两次写法都忽略（取缺省值）。
- **材质语句的错误规则**：材质语句上**属性级**的错误（未知键、类型错误、范围错误、重复、常量求值失败）各报一条，出错的属性视为没有写（取缺省值），材质**仍然登记**；只有位置错误与缺名字使整条语句无效。这是 1.1.4 第 4 条对 `material` 的细化：一个写错的分量不应让引用该材质的每个节点都报"未定义的材质"。

一条有效的 `material` 语句定义一个**材质值**：
$$T[\text{名字}] = \{\, \text{name} = \text{名字},\ \text{color},\ \text{metal},\ \text{rough},\ \text{opacity} \,\}$$
缺省的分量取上表的缺省值；`default` 不是材质值的分量，它是材质表的标记（10.1.3）。

```s3d
let base_rough = 0.8
material white  color=#ffffff rough=base_rough        // 引用最外层 let
material tint   color=#ff000080                       // 等价于 color=#ff0000 opacity=0.50196…（128/255）
material bad    color=#ff0000 shine=1                 // 报错：material 没有属性 shine；bad 仍然定义，只有 color
```

### 10.1.3 材质表：同名覆盖与 default 的唯一性

**材质表** $T$ 按下面的规则由展开后的语句序列构造（05-layers.md：import 的文件按位置原地展开，所以"跨文件的覆盖"与"同一文件里后写覆盖先写"是同一条规则）：

1. 取序列里全部**有效**的 `material` 语句（10.1.1 的位置错误与缺名字的不算），按语句顺序编号。
2. 同名的语句里，**最后**一条生效：它的材质值**整体替换**先前的定义（不逐分量合并——先前定义里写了、后来定义里没写的分量回到缺省值）；同名重定义不是错误，它是层的覆盖机制（05-layers.md）。
3. 表的顺序 = 各名字**生效的那条**语句的顺序。输出 `materials` 与错误报告都按这个顺序。
4. **默认材质的唯一性**：表里 `default=true` 的条目至多一个。恰有一个时它是默认材质；没有时没有默认材质（10.1.4）；多于一个时，按表的顺序，第一个是默认材质，其后每一个各报一条错误"多个默认材质：X（已有默认材质 Y）"（报在那条 `material` 语句上），它们不作为默认材质。这是在**整个展开序列**上、覆盖完成之后判定的：同一名字先后两次 `default=true` 只算一个条目，不报错。

**import 后的裁决**由第 2 条与第 4 条共同给出：一个层想换默认材质，必须同时重定义原来的默认材质（不写 `default=true`）与新的默认材质（写 `default=true`）；只做后者会得到两个默认材质并报错；重定义原默认材质而不写 `default` 时，它不再是默认材质（因为是整体替换）。

```s3d
// base.s3d
material plain color=#cfcabe default=true
material steel color=#7f8b96 metal=0.6 rough=0.4
```

```s3d
// style.s3d
import "base.s3d"
material plain color=#cfcabe                     // 整体替换：plain 不再是默认材质
material steel color=#9fc4ea metal=0.1 rough=0.6 default=true   // 现在 steel 是唯一的默认材质
```

### 10.1.4 内置材质与默认材质

- **内置材质** $B$ 是一个固定的材质值：
$$B = \{\, \text{name} = \text{""},\ \text{color} = \text{\#c9c9c9},\ \text{metal} = 0.05,\ \text{rough} = 0.78,\ \text{opacity} = 1 \,\}$$
  它的 name 是**空串**。它不在材质表里，不能用 `mat=` 引用（没有名字），不能被重定义。四个分量的值与原型的渲染器与导出器所用的缺省值相同（corpus/exports 里导出的材质对没写的分量显式写出 `metal=0.05 rough=0.78`）。
- **默认材质** $D$：材质表里唯一的 `default=true` 条目的材质值；没有这样的条目时 $D = B$。
- 10.3 的算法里，材质链上没有任何来源时节点得到 $D$。

```s3d
material ground color=#59604f rough=1 default=true
box ground size=200,1,200 pin=top          // 没写 mat：用默认材质 ground
box crate  size=2,2,2 on=ground mat=steel  // 用 steel
```

## 10.2 节点的外观属性

节点的外观属性是 P 组里的 `mat` 与 `color` 两个（1.4.4）。哪些语句接受它们由 1.4.4 的表规定：实体、容器（group / union / subtract / intersect / repeat / 容器式 mirror、rotate）、span / sweep / pipe、副本式 mirror / rotate、copy、模板实例都接受；`arc` / `helix` / `line` / `bezier` / `spline` 不接受（曲线没有材质，10.4.4）。

| 属性 | 类型 | 含义 |
|---|---|---|
| `mat` | 材质名（D24） | 节点的**基础材质**：材质表里名为它的材质值 |
| `color` | 颜色 | 覆盖节点原本会得到的材质的 **color 分量**（带 alpha 位时也覆盖 opacity 分量）；其他分量沿用 |

- **`mat=名`**：值是材质名字面量——裸标识符或内容相同的字符串字面量（`mat=steel`、`mat="steel"`），按字面取名，不查对象、let 或模板参数（D24；原型行为相同）。名字必须是材质表 $T$ 的键（表在全部语句展开后构成，10.1.3，所以引用的材质可以定义在引用之后或别的文件里）。名字不在表里报错"未定义的材质 X"（报在写 `mat=` 的语句上；一条语句只报一次），该节点按**没有写 `mat`** 处理——继续沿材质链向上（10.3）；节点**不**退化，几何、求解、事实、输出条目都不受影响（1.10.2 的"材质错误"行；D16：原型里一个未定义材质会清空全部输出条目，本版不允许）。
- **`color=颜色`**：值是颜色字面量（1.2.7；D5：颜色位置的井号记号一律是颜色，不查标签，所以 `color=#fff` 在场景里有 `#fff` 标签时仍是颜色）。不是 3 / 4 / 6 / 8 位十六进制的值报错"颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa"，`color=red` 这类名字报错（类型错误），节点按 1.1.4 第 2 条退化。规范形式与 alpha 位的含义同 10.1.2：带 alpha 位的 `color` 同时覆盖 opacity 分量。
- 同一节点同时写 `mat` 与 `color`：基础材质是 `mat` 指的材质值，color 分量换成 `color`（10.3.2）。
- 两者是普通属性：几何时机的 `over` 可以设置或改写它们（04-derivation.md）；`copy` 复制原语句的它们、拷贝语句自己写的覆盖之；保名换形保留它们（04-derivation.md 的表）；模板实例语句上的 `mat` / `color` 落在实例节点上；模板参数不得叫 `mat` / `color`（1.3.4），所以不存在参数遮蔽外观属性的情况（原型里 `def t(mat=…)` 会吞掉实例的 `mat=`，本版在 def 处报错）。
- 外观属性对几何、定位、包围盒、事实没有任何影响。

```s3d
group rocket on=pad mat=white {
  cyl stage1 r=2.5 h=24                               // 继承 white
  cyl band   r=2.45 h=0.8 on=stage1 gap=-0.8 mat=hazard
  box tag    size=1,1,0.1 against=stage1.front color=#ff0000   // white 的分量、红色
}
```

## 10.3 有效材质与继承顺序

每个节点 $n$ 有两个有效材质：**几何时机的有效材质** $E_g(n)$（只由 `mat` / `color` 属性与材质表决定，不含样式；`where` 里的 `mat` 字段读它，1.7.3）与**外观时机的有效材质** $E(n)$（在 $E_g(n)$ 上应用样式；输出条目带它，10.6）。两者都是 10.1.2 定义的材质值 $\{\text{name}, \text{color}, \text{metal}, \text{rough}, \text{opacity}\}$。D8 的顺序 "style > color > mat > 父级 > default > 内置" 的精确含义就是下面三小节。

术语：节点的**自有外观属性**指它在几何时机的修改全部应用之后（04-derivation.md）所带的 `mat` 与 `color` 属性；引用了未定义材质的 `mat` 视为不存在（10.2）。影子（副本里对应原件子节点的节点）通常没有自有外观属性；04-derivation.md 允许对影子单独修改时给它的 `mat` / `color` 算它的自有外观属性。

### 10.3.1 材质链

节点 $n$ 的**材质链** $\mathrm{chain}(n)$ 是一个有序的节点序列加一个**终点**：$\mathrm{chain}(n) = (x_1, x_2, \dots, x_k;\ \tau)$。$x_1 = n$，之后是依次被咨询的节点；终点 $\tau$ 是链上没有任何来源时的结果，取 $D$（默认材质，10.1.4）或 $E(N)$（皮肤子树的终点是被替换节点的外观时机有效材质，10.4.3）。链按下面的规则递归定义（$\mathrm{parent}(n)$ 是场景树里的父节点；根不是节点）：

| $n$ 是 | $\mathrm{chain}(n)$ |
|---|---|
| 根 | $(\ ;\ D)$ |
| 普通节点：实体、容器、repeat 节点、阵列实例、模板实例、copy 节点、副本组（`mirrored` / `r`$k$）、保名换形后的节点、退化节点 | $(n) \mathbin{+\!\!+} \mathrm{chain}(\mathrm{parent}(n))$ |
| 变体皮肤节点 $K$（段 `skin`，父是被替换的 $N$） | $(K;\ E(N))$ |
| 副本式节点 $R$（`mirror R of=A` / `rotate R of=A`） | $(R) \mathbin{+\!\!+} \mathrm{chain}(A)$ |
| 影子 $s$，原件 $o$，处在副本式节点 $R$（`of=A`）之下 | $(s) \mathbin{+\!\!+} \mathrm{subst}_{A \to (R, A)}(\mathrm{chain}(o))$ |
| 影子 $s$，原件 $o$，处在容器式 mirror / rotate 的副本组之下 | $(s) \mathbin{+\!\!+} \mathrm{chain}(o)$ |

$\mathrm{subst}_{A \to (R, A)}(c)$ 把序列 $c$ 里的元素 $A$（若出现）换成两个元素 $R, A$；$A$ 不在 $c$ 里时 $c$ 不变。$\mathbin{+\!\!+}$ 是序列连接，终点取右边序列的终点。原件 $o$ 自己可以是影子或副本式节点，递归适用。

要点：

- 副本式节点 $R$ 与它的影子走**原件的链**（原件的祖先），$R$ 所在位置的祖先**不**参与（原型行为，语料依赖）；$R$ 自己写的 `mat` / `color` 插在原件 $A$ 之前，即它对 $R$ 本身以及 $R$ 下面**沿链到 $A$ 之间没有自有外观属性**的影子生效，并压过 $A$ 自己写的。原型里 $R$ 的 `mat` 只对原件是实体的情况生效、对容器原件的影子被忽略；本版统一按上面的规则。
- 容器式 mirror / rotate：容器 $C$ 是原件的父节点，已经在原件的链上；副本组没有自有外观属性，所以影子与原件得到同样的结果。
- 皮肤节点 $K$ 的父节点是 $N$，但链不继续到 $N$ 的祖先，而是以 $E(N)$ 为终点（10.4.3）。
- 链里每个节点至多出现一次；链是有限的（树是有限的，副本与皮肤的定义不成环，04-derivation.md、07-semantics.md）。

### 10.3.2 几何时机的有效材质

$E_g(n)$ 由 $\mathrm{chain}(n) = (x_1, \dots, x_k;\ \tau)$ 按下面的算法得到：

```
c    ← 无                            // 记住的颜色覆盖（含可选的 alpha）
base ← 无
for i = 1 .. k:
  if x_i 有自有 color 且 c = 无:  c ← x_i 的 color
  if x_i 有自有 mat:              base ← T[x_i 的 mat]; break
if base = 无:                      base ← τ                 // D，或皮肤子树的 E(N)
E_g(n) ← base
if c ≠ 无:  E_g(n).color ← c.rgb;  若 c 带 alpha 位则 E_g(n).opacity ← c.alpha / 255
```

含义：

- 从节点自己开始向链的远端找**第一个写了 `mat` 的节点**，它的材质值是基础材质；没有就用终点。
- 在到达那个节点之前（含它自己）遇到的**第一个 `color`** 覆盖基础材质的 color 分量（与 opacity 分量，若带 alpha）；基础材质的 name、metal、rough 不变。所以同一节点上 `color` 压过 `mat` 的颜色，近的 `color` 压过远的；而一旦某一级写了 `mat`，更远一级的 `color` 不再有影响。
- 链上既没有 `mat` 也没有 `color` 的节点被跳过（父级的属性传给没写的子级）。
- $E_g$ 的 name 来自基础材质：被 `color` 改过颜色的 steel 仍然叫 steel（`where mat == steel` 为真）；基础材质是内置材质 $B$ 时 name 是空串。原型里 `color` 会把整个材质换成只有颜色的匿名材质（metal、rough 回到缺省、name 丢失），本版只覆盖 color 分量。

这个算法对链上每个节点只读它的自有外观属性，与节点的几何无关，所以退化节点（1.1.4）也有有效材质。

### 10.3.3 样式的应用：外观时机的有效材质

**样式动作**：一条 `style` 语句（或与之等价的只改外观的 `over … mat=… color=…`，07-semantics.md）在求解与语义层之后被求值，得到它的选择集 $S$（07-semantics.md）与至多两个属性 `mat`、`color`（1.3.12）。它的**作用范围**是 $S$ 的成员及其全部后代（场景树意义上的后代；皮肤子树除外——10.4.3）。一条样式语句无效时（选择器错误、`mat` 未定义、属性错误，1.10.2）不作用于任何节点。

节点 $n$（不在皮肤子树里）的**样式序列** $\mathrm{styles}(n)$：

- 普通节点：作用范围含 $n$ 的全部样式语句，按语句顺序（05-layers.md 展开后的顺序）。
- 副本式节点 $R$（`of=A`）：$\mathrm{styles}(A)$ 与"作用范围含 $R$ 的样式语句"的**并**，按语句顺序（每条语句至多一次）。
- 影子 $s$（原件 $o$）：$\mathrm{styles}(o)$ 与"作用范围含 $s$ 的样式语句"的并，按语句顺序。
- 被变体隐藏的原物体 $N$：与普通节点相同（它仍被样式选中，07-semantics.md）。

$E(n)$ 从 $E_g(n)$ 出发，按 $\mathrm{styles}(n)$ 的顺序逐条应用，每条：

```
if 该样式有 mat:    E ← T[mat]                               // 整体替换：name、color、metal、rough、opacity 全部换成该材质
if 该样式有 color:  E.color ← color.rgb;  若带 alpha 位则 E.opacity ← alpha/255
```

同一条样式里 `mat` 先于 `color` 应用，与书写顺序无关（原型里 `mat` 写在 `color` 之后会丢掉 `color`，本版不会）。所以：

- 任何作用于 $n$ 或 $n$ 的祖先的样式都压过 $n$ 自己（与祖先）的 `mat` / `color`（D8 保留的原型行为）；后写的样式压过先写的；样式里只写 `color` 时保留 $E_g(n)$ 的 name、metal、rough，只换颜色。
- 副本跟随原件的样式；直接选中副本（或它的副本祖先）的样式与原件的样式按语句顺序一起应用，后写的赢。原型里直接选中副本的样式无条件压过原件的样式（不看顺序）；本版按语句顺序，语料里两者不同时出现，结果相同。
- 没有任何样式选中 $n$ 时 $E(n) = E_g(n)$。

皮肤子树里节点的 $E$ 见 10.4.3（样式不直接作用于皮肤，皮肤经终点 $E(N)$ 继承）。

### 10.3.4 优先级小结与例子

按分量看，D8 的顺序是：

| 分量 | 决定它的来源（从高到低） |
|---|---|
| color（与 alpha 给出的 opacity） | 最后一条作用于该节点的样式的 `color`（若它之后没有再被样式的 `mat` 整体替换）> 样式的 `mat` > 链上第一个 `color`（不远于第一个 `mat`）> 链上第一个 `mat` 的材质 > 终点（默认材质 / 内置材质 / $E(N)$） |
| name、metal、rough、opacity | 最后一条作用于该节点的带 `mat` 的样式 > 链上第一个 `mat` 的材质 > 终点 |

```s3d
material steel color=#7f8b96 metal=0.6 rough=0.4
material wood  color=#8a5a3c
material plain color=#cfcabe default=true

group g mat=steel {
  box a size=1,1,1                          // E_g = steel
  box b size=1,1,1 color=#ff0000            // E_g = {name steel, #ff0000, 0.6, 0.4, 1}
  box c size=1,1,1 mat=wood color=#00ff00   // E_g = {name wood, #00ff00, 0.05, 0.78, 1}
  group h color=#0000ff { box d size=1,1,1 mat=wood }   // d：E_g = wood（d 自己的 mat 先于 h 的 color）
}
box e size=1,1,1                            // E_g = plain（默认材质）
style g.b mat=wood                          // E(b) = wood（样式压过 b 自己的 color）
style g color=#123456                       // E(a) = steel 改色 #123456；E(b) = wood 改色 #123456（后写的样式）；
                                            // E(c) = wood 改色；E(h.d) = wood 改色；e 不受影响
```

## 10.4 副本、拷贝、变体、阵列与模板实例的材质来源

本节把 10.3 的规则落到每一种派生节点上；没有新规则，只有解释与例子。

### 10.4.1 副本（mirror / rotate）

副本跟随原件的外观，就像它跟随原件的几何（04-derivation.md）：

- **容器式**（`mirror C … { 原件 }`、`rotate C n=k … { 原件 }`）：副本组 `C.mirrored` / `C.r1 … C.rk` 没有自有外观属性，影子的链等于原件的链（10.3.1），样式序列是原件的样式并上直接选中影子（或副本组、容器）的样式。所以影子与原件的 $E_g$ 相同；容器 $C$ 上的 `mat` / `color` 经原件的父子关系同时作用于两者。
- **副本式**（`mirror R of=A …`、`rotate R of=A …`）：$R$ 的链是 $(R) \mathbin{+\!\!+} \mathrm{chain}(A)$——$R$ 自己的 `mat` / `color`，然后是 $A$ 与 $A$ 的祖先；$R$ 所在位置的祖先不参与。$R$ 下面影子的链是原件的链，并在 $A$ 之前插入 $R$。效果：
  - $R$ 不写外观属性：$R$ 与它的影子与原件完全同色（语料 `spaceport.s3d` 的 `tower2`、`arm2`，`semantics.s3d` 的 `rampE`、`half2`，`curves.s3d` 的 `stepsE`、`tower2`、`gate2`，`rotation.s3d` 的 `wing2` 都是这种）。
  - `mirror R of=A mat=rust`，$A$ 是实体：$R$ 是 rust。
  - `mirror R of=A mat=rust`，$A$ 是容器：$A$ 下面没有写 `mat` / `color` 的部件的影子是 rust；自己写了 `mat` 的部件的影子保持自己的；写了 `color` 但没写 `mat` 的部件的影子是 rust 改成那个颜色。原型里这种情况忽略 $R$ 的 `mat`，本版按本规则（语料里没有带外观属性的副本式语句，不受影响）。
  - `group G mat=rust { mirror R of=A }`，$A$ 在 $G$ 之外：$G$ 的 rust 对 $R$ 与影子无效（原型行为保留）；要给副本上色，写在 $R$ 上或用样式选中 $R$。
- 副本式节点与副本组可以被样式选中（它们在 `all()`、`kind()`、`#tag` 与路径里，1.7.2）；作用范围含影子。原件被样式选中时影子跟随（10.3.3）。语料 `spaceport_blueprint.s3d` 的 `over wings.annex color=#cfe3f7` 使 `wings.annex` 与 `wings.mirrored.annex` 同色。
- 04-derivation.md 允许用几何时机的 `over` 单独改影子的 `mat` / `color`（例如 `over farm.1.t.rotor.r2.blade mat=rust`）：改后的影子有了自有外观属性，排在它链的第一位，只影响那一个影子及其链上经过它的后代影子。

```s3d
material steel color=#7f8b96 metal=0.6 rough=0.4
material rust  color=#8a4a2a rough=0.9
group tower mat=steel {
  box leg size=0.6,48,0.6
  box top size=6,1,6 on=leg color=#00ff00
}
mirror tower2 of=tower across=rocket axis=x mat=rust
// tower2.leg：E_g = rust（链：leg 影子 → leg → tower2 → tower → 根）
// tower2.top：E_g = rust 改色 #00ff00（top 的 color 在 tower2 的 mat 之前）
```

### 10.4.2 拷贝（copy）

`copy R of=A …` 生成的是独立节点（04-derivation.md）：它按**自身位置**解析——链是 $(R) \mathbin{+\!\!+} \mathrm{chain}(\mathrm{parent}(R))$，$A$ 的祖先不参与。$R$ 的自有外观属性 = 原语句（$A$ 的源语句）的 `mat` / `color` 被拷贝语句自己写的覆盖之后的结果（04-derivation.md 的属性替换规则）；$R$ 的子节点是原语句块的重新展开，各自带自己的属性。拷贝块里几何时机的 `over … color=…` 设置的是普通属性（语料 `spaceport_edit.s3d` 的 `copy hangar2 of=hangar … { over roof color=#4a5560 }`：`hangar2.roof` 的 $E_g$ 是 steel 改色 `#4a5560`——roof 的源语句写了 `mat=steel`）。

```s3d
group g mat=steel { box a size=1,1,1 }
copy b of=g.a at=5,0,0          // b 在根下：E_g(b) = 默认材质（g 的 steel 不跟过来）
copy h of=g at=0,0,10           // h.a：链 h.a → h（自有 mat=steel，来自 g 的语句）→ 根：steel
```

### 10.4.3 变体皮肤

变体（07-semantics.md）把被选中的节点 $N$ 隐藏，并在 $N$ 下挂皮肤节点 $K$（段 `skin`，模板实例），模板体在 $K$ 下展开。材质规则：

- 皮肤子树里的节点 $x$（$K$ 与它的全部后代）的链在 $K$ 处终止，终点是 $E(N)$：$\mathrm{chain}(x) = (x, \dots, K;\ E(N))$。于是：
  - $x$ 自己或 $x$ 到 $K$ 之间（含 $K$）的某个节点写了 `mat`：$x$ 用那个材质（10.3.2），$N$ 的材质与样式都不影响它——**皮肤自己写了的材质优先**。`variant … mat=… color=…`（1.3.13）写的属性落在 $K$ 上，所以它是皮肤整组的基础材质（部件自己写了的除外）。
  - 这段里只有 `color`（没有 `mat`）：基础材质是 $E(N)$，换颜色。
  - 这段里什么都没写：$x$ 的材质就是 $E(N)$——被替换物体的**外观时机**有效材质，含作用于 $N$ 及其祖先的样式。语料：`semantics.s3d` 里 `style deck mat=wood` 使 `slab.skin.pl.*.plank` 都是 wood；`spaceport_blueprint.s3d` 的 `hangar.roof.skin.base`、`hangar.roof.skin.rib.*.r` 继承 roof 的 steel；`street.h4.skin.core` 自带 `mat=brick`、`pane` 自带 `mat=glass`，不受 $N$ 影响。
- **样式不直接作用于皮肤**：样式的作用范围（10.3.3）不含皮肤子树——即使选择器选中了皮肤节点（D9 要求皮肤对选择器不可见；若 07-semantics.md 按 1.6.4 / 1.7.2 允许 `children()`、`descendants()`、通配与直接路径看到皮肤，选中了也不作用），皮肤只经终点 $E(N)$ 间接受样式影响。
- 皮肤里的 mirror / rotate 副本按 10.4.1（原件在皮肤里，链同样在 $K$ 处终止）。
- $N$ 虽被隐藏，$E_g(N)$ 与 $E(N)$ 照常计算；$N$ 的其他后代（也被隐藏）同样有有效材质，只是不输出条目（10.6）。
- 皮肤不传播到 $N$ 的副本：若 $N$ 有副本（影子或副本式节点），副本仍按 $N$ 未被替换时的链与样式计算，并照常输出条目（07-semantics.md 规定变体不传播到副本；要替换副本须另写一条变体）。

```s3d
material steel color=#7f8b96 metal=0.6 rough=0.4
material brick color=#b5654a
def ribbed(w=1, h=1, d=1) {
  box base size=w,h*0.35,d
  box cap  size=w,h*0.65,d on=base mat=brick
}
box roof size=36,0.8,28 mat=steel
over roof use=ribbed                 // roof.skin.base：E = E(roof) = steel；roof.skin.cap：brick
style roof color=#9fc4ea             // roof.skin.base 跟着变成 steel 改色；cap 不变
```

### 10.4.4 阵列实例、模板实例、容器与布尔容器、保名换形

- **阵列**：`repeat` 节点接受 `mat` / `color`（1.4.4）；阵列实例节点（段 `0`、`1`、…）没有自有外观属性；实例里的部件按树继承：部件 → 实例 → repeat 节点 → 其祖先。语料 `spaceport.s3d` 的 `repeat tanks … mat=white { cyl tank …; cyl ring … mat=hazard }`：`tanks.0.tank` 是 white，`tanks.0.ring` 是 hazard。
- **模板实例**：实例节点（种类 = 模板名）接受 `mat` / `color`；模板体里的语句按树继承（部件 → 实例节点 → 实例所在位置的祖先）。模板体内写的 `mat=` 引用的是全局材质表（材质表不分作用域）。派生模板与就地扩展不改变这一点：所有体里的语句都是实例节点的后代。语料 `spaceport.s3d` 的 `truss tower … mat=steel`：`tower.legs.0.leg`、`tower.decks.5.deck` 都是 steel；`def shed … { subtract body mat=concrete {…}; box roof … mat=steel }`：`hangar.body` 是 concrete、`hangar.roof` 是 steel。
- **容器**：group / union / subtract / intersect 与容器式 mirror / rotate 上的 `mat` / `color` 是其后代的来源；容器自己也有有效材质（输出里布尔容器的条目带它，10.6）。
- **布尔容器**：布尔容器节点与它的每个操作数各有自己的有效材质（操作数没写时继承容器的）。渲染布尔体时，结果表面上来自某个操作数表面的部分用**该操作数**的有效材质；容器自己的有效材质是条目级的材质（10.6），供不区分操作数的工具（导出、评分）使用。语料 `basics.s3d` 的 `subtract crate … mat=a { box shell …; box hollow … }`：crate、shell、hollow 都是 a。
- **保名换形**（`over X : 种类`）：换形保留 `mat` / `color`（04-derivation.md 的表），换形后按新节点的位置与属性重新解析；源语句的子语句被保留时它们的外观属性一并保留。
- **退化节点**：有有效材质（10.3.2），但没有几何，不输出条目。
- **曲线**（arc / helix / line / bezier / spline）：没有材质，不接受 `mat` / `color`（1.4.4），`where` 里它们的 `mat` 是缺失值（10.5.3）。曲线作为容器的子节点不影响任何材质链（它们不是别人的父节点）。

## 10.5 与语义层和修改的接口

### 10.5.1 style

语法见 1.3.12：`style 选择器 mat=… color=…`，只能写在最外层；选择器是任意集合表达式（可以含空白、概念名、模式角色、关系谓词与 `where` 的全部字段，1.7.4）；属性只能是 `mat`（材质名）与 `color`（颜色），各至多一次，至少一个（其他键报错"style 只支持 mat 和 color"，语句无效）。语义：

- **求值时机与顺序**由 07-semantics.md 规定（在求解、事实与概念之后；多条 style 按语句顺序）。本章只用到：每条 style 有一个选择集 $S$，以及样式按语句顺序应用（10.3.3）。
- **作用范围** = $S$ 的成员及其全部后代（10.3.3），皮肤子树除外（10.4.3）。成员可以是任何在 `all()` 里的节点：实体、容器、阵列节点与实例、模板实例、副本式节点、副本组、影子、copy 节点、被隐藏的原物体。
- **效果**只改 $E$（10.3.3）：`mat` 整体替换，`color` 只换颜色（与 alpha 给出的 opacity）。不改任何节点的属性、变换、包围盒、事实、概念——"外观层不改空间事实"由此构造保证（D12）。
- `mat` 引用未定义的材质报错"未定义的材质 X"，整条 style 无效（1.10.2）；`mat="steel"` 字符串写法合法（1.4.2；原型里 style 不接受字符串）。
- 选择集为空不是错误。

### 10.5.2 over 的两个时机与 variant

04-derivation.md 规定 `over` 的两个时机；本章只规定两种时机下 `mat` / `color` 的效果：

- **几何时机的 over**（带块、带换形、带其他属性，或写在块 / 模板体 / copy 块里的 over，即使只写了 `mat` / `color`）：`mat` / `color` 作为普通属性写到被选中的节点上（覆盖它原来的），成为该节点的自有外观属性，参与 $E_g$（10.3.2）——因此它作用于该节点及其链上经过它的后代，并且会被之后的样式压过。对影子的这种修改见 10.4.1。
- **外观时机的 over**（最外层、无块、无换形、属性只有 `mat` / `color` / `use`）：没有 `use` 时它就是一条 style（10.5.1），`over 选择器 mat=… color=…` 与 `style 选择器 mat=… color=…` 等价；有 `use` 时它是一条 variant，其中的 `mat` / `color` 落在皮肤节点 $K$ 上（10.4.3），其余键是模板参数（1.3.13）。
- **variant 语句**（1.3.13）里的 `mat` / `color` 同上，落在 $K$ 上。
- 这样，几何层的 over 用属性改外观（求解前），外观层的 over / style / variant 用样式与皮肤改外观（求解后），两条路都只影响 $E_g$ / $E$，不影响几何；而外观层改不了几何是因为它根本不写属性（D10、D12）。

### 10.5.3 where 里的 mat

`where` 谓词里的字段 `mat`（1.7.3）对成员 $m$ 取值：

- $m$ 不是曲线：$E_g(m)$ 的 name；name 是空串（基础材质是内置材质 $B$）时为**缺失值**（1.5.5：比较与 `~` 得到假）。影子与副本式节点按它们的链（10.3.1）求 $E_g$。
- $m$ 是曲线：缺失值。
- $m$ 在皮肤子树里（D9 下不会成为成员；若 07-semantics.md 按 1.6.4 / 1.7.2 允许 `descendants()`、通配或直接路径选到它）：沿 10.4.3 的链 $(m, \dots, K)$ 找第一个 `mat`，有则取它的名字；没有则取 $E_g(N)$ 的 name（样式不计），空串为缺失值。

可用的谓词：`mat == 材质名`（右侧裸标识符按材质名字面量，1.5.5）、`mat != …`、`mat ~ /正则/`。

**求值时机**：`mat` 只依赖材质表与链上节点的自有外观属性，不依赖求解，所以它是静态量，在几何时机的选择器里也可用（1.7.4、D3、D10）。几何时机的选择器求值时，$E_g$ 按**此刻**的自有外观属性计算——队列里排在前面的修改已经生效，后面的还没有（04-derivation.md 的队列顺序）；语义层与外观时机的选择器在全部几何时机的修改之后求值，读到的是最终的 $E_g$。样式永远不影响 `mat` 字段：`style all() where mat == plain mat=stone` 之后再写 `style all() where mat == plain color=#f00`，第二条仍选中原来是 plain 的那些节点。

```s3d
over kind(box) where mat == plain color=#e8f1fb          // 外观时机：所有基础材质为 plain 的盒子换色
drop all() where mat ~ /^tmp_/                            // 几何时机：mat 是静态量，可用
check "没有未上色的实体" kind(box, cyl, sphere) where not (mat ~ /./)   // 内置材质的 name 为空 → mat 缺失 → 条件为真 → 这些是违例
```

## 10.6 输出形状

编译输出里与材质有关的两个部分（其他字段由各章与输出格式定义）：

1. **`items[]`**：每个渲染条目（实体条目；布尔容器条目及其内部每个操作数条目）带 `material`，形状**统一**为
   ```
   { "name": 字符串, "color": "#rrggbb", "metal": 数, "rough": 数, "opacity": 数 }
   ```
   值是该条目节点的 $E$（10.3.3）：五个键**总是**都有；没有名字的材质（基础材质是内置材质 $B$）`name` 为空串 `""`；`color` 是 10.1.2 的规范形式（6 位小写，不含 alpha）；三个数按双精度原样输出。被隐藏的节点、退化节点、曲线不出条目（条目的集合由 02-geometry.md 与输出格式定义）；皮肤子树的节点出条目。原型只输出写了的分量（`{name, color, metal?, rough?}`），颜色覆盖时输出 `{color}`；本版统一形状。
2. **`materials`**：材质表 $T$ 的全部条目，按表的顺序（10.1.3）；每个条目是
   ```
   名字: { "color": "#rrggbb", "metal": 数, "rough": 数, "opacity": 数, "default": 布尔 }
   ```
   五个键总是都有（缺省的分量按 10.1.2 填入）。原型只输出写了的键；本版填满。

与参考输出（corpus/reference）对拍时（09-compat.md 定义规则），本章给出的对应关系：参考输出条目的 `name` 缺失对应本版的 `""`；参考输出缺失的 `metal` / `rough` / `opacity` 对应本版的内置值 $0.05$ / $0.78$ / $1$（原型渲染器与导出器正是用这些缺省值，所以语义相同）；颜色逐字相同（语料里的颜色都是 6 位小写）。因 10.3.2 的改变（`color` 只覆盖颜色分量）而**预期不同**的条目只有三个：`spaceport_blueprint` 的 `wings.annex` 与 `wings.mirrored.annex`（参考 `{color:"#cfe3f7"}`；本版 name concrete、rough 0.9、metal 0.05、color `#cfe3f7`）、`spaceport_edit` 的 `hangar2.roof`（参考 `{color:"#4a5560"}`；本版 name steel、metal 0.6、rough 0.4、color `#4a5560`）。节点的变换、包围盒与事实在所有语料里不因本章而改变。

## 10.7 诊断

本章定义的错误（文案的关键信息按 1.10.1；恢复按 1.1.4 与本章各节）：

| 文案 | 触发条件 | 恢复 |
|---|---|---|
| material 只能写在最外层 | `material` 出现在任何块里（10.1.1） | 语句无效，不登记 |
| material 需要名字 | `material` 没有名字（10.1.1） | 语句无效 |
| material 没有属性 k | 属性键不是 color / metal / rough / opacity / default（10.1.2） | 该键忽略，材质仍登记 |
| material X 的 k 需要 0..1 之间的数 | metal / rough / opacity 不是 $[0,1]$ 内的有限数（10.1.2） | 该分量取缺省值 |
| 颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa | `material` 的 `color` 不是合法颜色字面量 | 该分量取缺省值 |
| 同上 | 节点的 `color=` 不是合法颜色字面量（10.2） | 类型错误：节点退化（1.1.4 第 2 条） |
| material X 同时写了颜色的透明度与 opacity | `color` 带 alpha 位且写了 `opacity=`（10.1.2） | `opacity=` 生效 |
| …必须是常量 | 材质属性值引用了对象或用了求解期才可用的构件（10.1.2、1.4.1） | 该分量取缺省值 |
| 重复的属性 | 同一条 `material` 里同名属性两次（1.3.3） | 该分量取缺省值，材质仍登记 |
| default 只能是 true / false | `default` 的值不是布尔字面量（10.1.2） | 当作 `false` |
| 多个默认材质：X（已有默认材质 Y） | 材质表里第二个及之后的 `default=true` 条目（10.1.3） | X 不作为默认材质 |
| 未定义的材质 X | 节点的 `mat=` 引用的名字不在材质表里（10.2） | 该节点按没写 `mat` 处理；不退化 |
| 未定义的材质 X | style / 外观时机 over 的 `mat=` 引用的名字不在材质表里（10.5.1） | 整条语句无效 |
| style 只支持 mat 和 color | style 写了其他属性（10.5.1） | 整条语句无效 |
| X 没有属性 mat / color | 不接受外观属性的语句写了它们（曲线；1.4.4） | 按第 1 章：节点退化 |
| mat 需要材质名 | `mat=` 的值不是名字字面量或字符串（如 `mat=a.b`、`mat=1`；1.4.2） | 类型错误：节点退化 |

本章不定义警告。材质解析不会因任何错误而中止：每个条目独立计算自己的材质（1.1.4 第 6 条）。

## 10.8 语料对照

下表列出 corpus/examples 与 corpus/exports 里与本章有关的语句形态，以及按本章得到的结果（节点的变换与事实不受影响）。"参考一致"指 items 的材质与参考输出按 10.6 的对应关系一致。

| 语料 | 语句形态 | 本章规则 | 结果 |
|---|---|---|---|
| basics.s3d:3–6 | 四条 `material a/b/c/d color=#…`，没有 default | 10.1.2–10.1.4：没有默认材质，$D = B$ | 表有四项；全部节点都写了 `mat`，参考一致 |
| basics.s3d:21–24 | `subtract crate … mat=a { box shell …; box hollow … }` | 10.4.4 布尔容器 | crate、shell、hollow 都是 a |
| spaceport.s3d:17–19 | `group rocket … mat=white { …; cyl band … mat=hazard }` | 10.3.2 链 | stage1 white、band hazard |
| spaceport.s3d:24–26 | `rotate fins n=3 … { wedge fin … mat=hazard }` | 10.4.1 容器式 | `fins.fin`、`fins.r1..r3.fin` 都是 hazard |
| spaceport.s3d:41、47–48 | `truss tower … mat=steel`；`mirror tower2 of=tower …`、`mirror arm2 of=arm …`（不写外观） | 10.4.4 模板实例；10.4.1 副本式 | `tower.legs.0.leg`、`tower2.legs.0.leg`、`arm2` 都是 steel |
| spaceport.s3d:51–53 | `repeat lights … mat=steel { cyl pole …; box lamp … mat=white }` | 10.4.4 阵列 | pole steel、lamp white |
| spaceport.s3d:57–68 | `def shed … { subtract body mat=concrete {…}; box roof … mat=steel }`、`shed hangar …` | 10.4.4 模板体 | `hangar.body` 与操作数 concrete，`hangar.roof` steel |
| spaceport.s3d:74–78 | `mirror wings … { box annex … mat=concrete; span stair1 … mat=steel; … }` | 10.4.1 容器式 | annex 与 `mirrored.annex` concrete，stair 都 steel |
| spaceport_edit.s3d:13–14 | `copy hangar2 of=hangar … { over roof color=#4a5560 }` | 10.4.2、10.5.2 几何时机 over | `hangar2.roof` = steel 改色 `#4a5560`（预期与参考不同，10.6）；`hangar2.body` concrete |
| spaceport_blueprint.s3d:6–12 | 7 条 `material` 重定义（import 之后） | 10.1.3 整体替换 | 全场 steel 等用新值（如 steel = `#9fc4ea` 0.1 0.6）；没有 default，参考一致 |
| spaceport_blueprint.s3d:19 | `over hangar.roof use=ribbed`（变体） | 10.4.3 | `hangar.roof.skin.base`、`skin.rib.*.r` 继承 roof 的 steel |
| spaceport_blueprint.s3d:20 | `over wings.annex color=#cfe3f7`（外观时机） | 10.3.3、10.4.1 | `wings.annex` 与 `wings.mirrored.annex` = concrete 改色（预期与参考不同，10.6） |
| rotation.s3d:3–8、47 | 六条材质；`group pisa … mat=white {…}`；`rotate rotor n=2 { box blade … mat=white }`；`mirror wing2 of=wing …` | 10.3.2、10.4.1 | pisa 下未写的部件 white；`rotor.r1.blade` white；`sat.wing2` panel |
| semantics.s3d:28、91 | `repeat piers … mat=plain {…}` + `style pier mat=stone` | 10.3.3 样式压过链 | `piers.*.p` stone |
| semantics.s3d:31、92、130 | `box slab … mat=plain` + `style deck mat=wood` + `variant deck use=planks` | 10.4.3 终点 $E(N)$ | slab 隐藏；`slab.skin.pl.*.plank` wood |
| semantics.s3d:32–33、93 | `span rampW … mat=plain`、`mirror rampE of=rampW …` + `style bridge-pier-deck mat=stone` | 10.3.3 副本的样式序列 | rampW、rampE 都 stone |
| semantics.s3d:34–41、94 | rails / lamps `mat=plain` + `style railing\|lamp mat=metal` | 10.3.3 | post、pole、bulb 都 metal |
| semantics.s3d:45–47、95 | house 的 `wedge half … mat=plain`、`mirror half2 of=half …` + `style roof mat=tile` | 10.3.3、10.4.1 | `street.h*.half`、`half2` tile；door、body plain |
| semantics.s3d:62–65、96、143 | `group watch … mat=plain { … box cabin … }`、`copy watch2 of=watch …` + `style cabin mat=wood` | 10.4.2 拷贝按自身位置 | `watch.cabin`、`watch2.cabin` wood；legs plain |
| semantics.s3d:68–72、97–98 | `repeat trees … mat=plain {…}` + `style tree mat=leaf` + `style kind(cyl)&tree mat=wood` | 10.3.3 后写的样式赢 | trunk wood、crown leaf |
| semantics.s3d:118–128、131 | `def townhouse … { box core … mat=brick; box parapet … mat=plain; … box pane … mat=glass }` + `variant tall_building use=townhouse` | 10.4.3 皮肤自己写的优先 | `street.h4.skin.core` brick、parapet plain、pane glass |
| curves.s3d:18–21、52–54、60–64 | `repeat lamps … mat=steel { cyl pole; sphere bulb … mat=white }`；`rotate benches n=5 { … mat=accent }`；`mirror halves across=x { box house … mat=stone; … box flag … mat=accent }` | 10.4.4、10.4.1 | pole steel、bulb white；`benches.r*.seat` accent；`halves.mirrored.house` stone、`.flag` accent |
| curves.s3d:39、45、71 | `mirror stepsE of=stepsW …`、`mirror tower2 of=tower …`、`rotate gate2 of=gate …` | 10.4.1 副本式 | 与原件同：stone、white、各影子同原件 |
| exports/*.s3d | `material … color=#… metal=0.05 rough=0.78`（显式写出内置缺省）；`mesh … mat=…`；`mirror X of=Y`（不写外观） | 10.1.2、10.4.1 | 与原型一致；没有 default、style、variant、`color=` |

语料里没有出现的本章构件：`opacity`、`default=true`、带 alpha 位的颜色、`material` 里引用 `let`、副本式节点自带 `mat` / `color`、`where mat`。
