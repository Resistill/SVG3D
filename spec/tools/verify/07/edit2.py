import sys,re
p='/home/user/SVG3D/spec/07-semantics.md'
s=open(p,encoding='utf-8').read()
def rep(old,new):
    global s
    n=s.count(old)
    if n!=1:
        print('COUNT',n,'for:',old[:80]); sys.exit(1)
    s=s.replace(old,new)

# 7.2.4 整节替换：从标题到 7.2.5 标题前的代码块之前
start=s.index('### 7.2.4 关系谓词')
end=s.index('```s3d\nconcept pier = kind(cyl) & on(river) & supports(kind(box))   // 立在水里')
new='''### 7.2.4 关系谓词

七个关系谓词 `on supports against inside contains touching intersects`（1.5.6）读 06-facts.md 的 6.7 定义的**关系索引**，不直接读事实列表：

**索引（6.7.1）。** 由 `facts[]` 里每条 `contact` 事实 $(r, A, B)$ 构造七个有序对集合 $I_{\\text{on}}, I_{\\text{supports}}, I_{\\text{against}}, I_{\\text{inside}}, I_{\\text{contains}}, I_{\\text{touching}}, I_{\\text{intersects}}$：关系从对象**上提**到 $A$、$B$ 各自的祖先链（含自身，不含根），但不上提到两者的公共祖先；`on` 同时加入 $I_{\\text{supports}}$（反向）与 $I_{\\text{touching}}$（双向），`against` 与 `intersects` 各自双向加入并加入 $I_{\\text{touching}}$（双向），`inside` 只加入 $I_{\\text{contains}}$（反向）、不派生 `touching`；其他提供者的事实不进索引。定义与表都在 6.7.1，本章不重复；本章记

$$R_{\\text{rel}}(x) = \\{\\, y : (x, y) \\in I_{\\text{rel}} \\,\\}$$

（节点 $x$ 在表 rel 里的对方集合），7.5.3 的关系约束用同一记法。上提的后果：**容器因其任一部件满足关系而满足关系**——语料 semantics.s3d 的 `street` 与 `watch` 因此 `on(ground)`（参考大纲里它们带 "· on ground"），但 `watch.legs.0.leg on watch.cabin` 不使 `watch` on 自己（`watch` 是公共祖先）；`touching(S)` 是"与 S 有任何接触类事实"（语料里 `rampW intersects slab` 使 `rampW ∈ touching(deck)`，于是 `rampW ∈ bridge`，7.9.1）。原型 Solver.buildRelations 与 6.7.1 逐条相同。

**谓词的值（6.7.2）。** `rel(S, k)`（$k$ 缺省为 1）：

$$\\mathrm{rel}(S, k) = \\{\\, x \\in \\mathrm{all}() \;:\; \\lvert R_{\\text{rel}}(x) \\cap S \\rvert \\ge k \\,\\}$$

按节点顺序排列。$S$ 必须是集合类（否则报错"X() 需要参照集合，如 X(ground)"）；$k$ 必须是与某个整数 $m \\ge 1$ 相差不超过 $\\varepsilon_{\\text{int}}$（3.1.7）的数，取 $m$，否则报错"X() 的第二个参数需要不小于 1 的整数"（1.5.6），该语句无效。原型 relSet 接受任何数（`c >= k`：`on(ground, 0)` 选中索引里有任何 on 项的全部节点，`on(ground, 1.5)` 等价于 2）；本版按 1.5.6 要求整数 $\\ge 1$。$S$ 为空时结果为空，不报错。$x$ 与 $S$ 的成员是同一节点时不计（索引里没有 $(x, x)$，6.7.2）。

要点：

- 谓词的结果只含非皮肤节点（阶段一没有皮肤；索引里的节点都在 `all()` 里，6.7.1）；退化节点不是事实的对象（6.2.1），所以不在任何 $R$ 里，也就不在任何谓词的结果里；影子、副本式节点、副本组、被变体选中的节点都按它们的事实进入索引（事实对它们的定义见 6.2.1）。
- `supports` 与 `contains` 是 `on` / `inside` 的反向关系：由 6.7.1 的表，$y \\in R_{\\text{supports}}(x) \\iff x \\in R_{\\text{on}}(y)$，$y \\in R_{\\text{contains}}(x) \\iff x \\in R_{\\text{inside}}(y)$；所以 `supports(S)` 是"托着 S 的某个成员（或其后代）"的节点及其祖先（不跨公共祖先）。
- pattern 的关系约束 `L rel M`（7.5.1 第 2 种、7.5.3）读**同一索引**：`a on b`（两个标量角色）成立 ⟺ $b \\in R_{\\text{on}}(a)$ ⟺ $a \\in \\mathrm{on}(\\{b\\})$。
- 谓词**只在语义层与外观时机可用**（1.5.6、5.4.2）；几何时机的选择器里报错"X() 依赖求解后的空间事实，不能用来决定位置 / 几何"（7.2.5 第 5 条）。
- **事实未推导**（`factsSkipped: true`，6.6.4）：索引为空，每个谓词得到空集，每条调用了谓词的语句（含写了关系约束的 pattern）记一条警告"空间事实未推导，X() 为空集"（7.1.4；6.9）。
- 关系索引是事实的函数：两份实现的事实相同（第 6 章的定义与容差）则谓词的结果相同；事实在容差边界上不同时，谓词的差异限于那几对节点及其链。

'''
s=s[:start]+new+s[end:]

# 7.2.5 第 3、5 条
rep('''3. **皮肤不可选**：本规范里没有任何选择器在皮肤存在之后求值（7.1.3）；因此 concept、pattern、check、style、variant、外观时机 over 的选择集**不可能**含皮肤节点，`descendants()`、通配与直接路径对皮肤的可见性（1.6.4）只在皮肤模板体内的表达式里（over / drop 的选择器、`on=` 等属性，4.7 第 6 条）与工具事后的查询里有意义。10-materials.md 的 10.4.3、10.5.3 里"若选中了皮肤"的分支在编译里不会发生。''',
'''3. **皮肤不可选**：本规范里没有任何选择器在皮肤存在之后求值（7.1.3）；因此 concept、pattern、check、style、variant、外观时机 over 的选择集**不可能**含皮肤节点。`descendants()`、通配与直接路径对皮肤的可见性（1.6.4）只在皮肤模板体内的表达式里（over / drop 的选择器、`on=` 等属性，4.7 第 6 条）与工具事后的查询里有意义；皮肤对各种机制可见与否的精确清单在 7.8.4。10-materials.md 的 10.4.3、10.5.3 据此把"样式选中了皮肤"、"`where mat` 的成员在皮肤子树里"记为编译里不会发生的情形（只在皮肤体内 over / drop 的选择器与工具查询里出现）。''')
rep('''5. **几何时机**的选择器里出现概念名、模式名、关系谓词、`w h d x y z top bottom` 按 5.4.2 报时机错误；本章不再重复。''',
'''5. **几何时机**的选择器按 5.4.2 的构件表，本章只索引、不另定义。可用：路径、通配、`#tag`、`kind()`、`all()`、`children()`、`descendants()`、列表、集合运算、`where` 里的 `kind` / `name` / `mat` / 元数据键、只作引用比较（`it == a`）或作集合的 `it`、`count(S)`（$S$ 由这些静态构件组成）。不可用：`where` 里的 `w h d x y z top bottom`，任何对象或 `it` 的锚点 / 度量 / 分量 / 坐标后缀（`it.top.y`、`it.w`、`a.top`），关系谓词，概念名 / 模式名 / `模式名.角色`，以及读到了这些的 `let`。文案两种：字段、锚点、概念 / 模式与读了它们的 `let` 报 1.7.4 的"几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）"；关系谓词报 1.5.6 的"X() 依赖求解后的空间事实，不能用来决定位置 / 几何"。判定是静态的：求值前对表达式树检查，不依赖成员集合，每条语句至多一条（5.4.2 第 1 条）；裸标识符在几何时机的环境里解析失败时，若它是 $P$ 里某个 concept / pattern 的名字则报时机错误，否则报"找不到对象 X"（5.4.2 第 2 条）；被几何时机选择器读到的 `let` 按 1.5.8 归属该 `let` 语句（5.4.2 第 6 条）。''')

open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
