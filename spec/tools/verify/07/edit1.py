import sys
p='/home/user/SVG3D/spec/07-semantics.md'
s=open(p,encoding='utf-8').read()
assert '\r' not in s
def rep(old,new):
    global s
    n=s.count(old)
    if n!=1:
        print('COUNT',n,'for:',old[:80]); sys.exit(1)
    s=s.replace(old,new)

# 术语表：关系索引
rep('| 关系索引 | 由空间事实构造的七张"节点 → 节点集合"的表（7.2.4），关系谓词查它 |',
    '| 关系索引 | 由空间事实上提构造的七个有序对集合（06-facts.md 的 6.7.1；本章记作 $R_{\\text{rel}}(x)$，7.2.4），关系谓词与 pattern 的关系约束查它 |')

# 7.1.1 保证
rep('''**保证（D9 的精确表述）。** 设场景 $P$，$P^\\circ$ 是把 $P$ 里全部 `style`、`variant` 与外观时机的 `over` 删掉（其余不动）得到的场景。则 $P$ 与 $P^\\circ$ 的编译结果在下列各项上**逐位相同**：去掉 `skin: true` 条目后的 `nodes[]` 序列及每个非皮肤节点的 `path kind id tags concepts line file types meta depth late ghost of curve world box tight`；`facts[]`、`factsSkipped`、`factProviders`；`concepts[]`、`patterns[]`、`checks[]`；归属于几何层语句与只读语义语句的全部错误与警告。允许不同的只有：`nodes[]` 的 `hidden`、皮肤节点（`skin: true`）及其一切、`items[]` 与 `curves[]` 的条目集合与 `material`、`variants[]`、归属于外观层语句的诊断、以及由此变化的错误排序。这是 05-layers.md 的 5.4.4 对语义层的落实；它由本章的构造保证——阶段二读不到的量不可能被它改变——不是一条需要额外实现的规则。''',
'''**保证（D9 的精确表述）。** 设场景 $P$，$P^\\circ$ 是把 $P$ 里全部 `style`、`variant` 与外观时机的 `over` 删掉（其余不动）得到的场景。则 $P$ 与 $P^\\circ$ 的编译结果在 05-layers.md 的 5.4.4 列出的五项上**逐位相同**：(1) 去掉 `skin: true` 条目后的 `nodes[]` 序列，及每个非皮肤节点的 `path kind id tags concepts line file types meta depth late ghost of curve`；(2) 每个非皮肤节点的 `world`（$R$、$t$）、`box`、`tight`；(3) `facts[]` 的条数、顺序与每条的全部字段，以及 `factsSkipped`、`factProviders`；(4) `concepts[]`、`patterns[]`、`checks[]` 的条目、成员、匹配、通过与否与顺序；(5) 归属于几何层语句、只读语义语句、以及节点路径不在皮肤子树里的全部错误与警告。允许不同的只有 5.4.4 列出的那些：`nodes[]` 的 `hidden`；皮肤节点（`skin: true`）及其一切；`items[]` 与 `curves[]` 的**条目集合**（被隐藏的节点没有条目、皮肤有，10.6）与 `items[]` 的 `material`；`variants[]`（$P^\\circ$ 没有删任何 `def` / `material`，所以 `defs` 与 `materials` 输出也相同）；归属于外观层语句、或节点路径在皮肤子树里的诊断；由此变化的错误排序（1.1.5）。被变体隐藏的原物体仍进 `all()`、仍参与事实、概念与模式（7.8.4；5.4.3 依赖此），所以隐藏不是上面五项里任何一项的变化。这是 5.4.4 对语义层的落实；它由本章的构造保证——阶段二读不到的量不可能被它改变——不是一条需要额外实现的规则。''')

# 7.1.3 step 1
rep('''1. **挂载**：按挂载表（7.8.2）的节点顺序，对每个宿主 $N$ 执行 4.7 的挂载：标记隐藏、构造并展开皮肤语句、绑定参数、定位（7.8.3）。各宿主互相独立；一个宿主的错误不影响其他宿主（1.1.4 第 6 条）。''',
'''1. **挂载**：按挂载表（7.8.2）的节点顺序，对每个宿主 $N$ 执行 4.7 的挂载：标记隐藏（不改变任何可读的量：隐藏的节点仍进 `all()`、仍有变换、包围盒、事实与有效材质，7.8.4）、构造并展开皮肤语句、绑定参数、定位（7.8.3）。各宿主互相独立；一个宿主的错误不影响其他宿主（1.1.4 第 6 条）。''')
rep('''阶段二里没有任何选择器求值：样式与变体的选择集来自阶段一；阶段二唯一求值的表达式是皮肤模板体里的属性值、`let`、元数据与皮肤体内 over / drop 的选择器（它们的搜索起点在皮肤子树里，4.7 第 6 条）。''',
'''阶段二里没有任何选择器求值：样式与变体的选择集来自阶段一；阶段二求值的表达式只有 variant 写明的参数值（7.8.3 第 3 条）、皮肤模板体里的属性值、`let`、元数据与皮肤体内 over / drop 的选择器（它们的搜索起点在皮肤子树里，4.7 第 6 条）。"只替换外观"的精确清单就是 7.1.1 的保证（5.4.4 的五项与允许不同的项）。''')

# 7.1.4 factsSkipped (R1)
rep('''- **宿主关闭了事实推导**（第 6 章的宿主选项；输出 `factsSkipped: true`）：事实为空，关系索引为空，七个关系谓词都得到空集，pattern 的关系约束都不成立；不报错、不警告（原型 compileOnce 相同：`buildRelations([])`）。概念、检查照常求值，结果按空事实计算。宿主**应**在界面上提示事实未推导，这不属于本规范。''',
'''- **宿主关闭了事实推导**（6.6.4；输出 `factsSkipped: true`）：事实为空，关系索引（6.7.1）为空，七个关系谓词都得到空集（6.7.3），pattern 的关系约束都不成立；每条调用了关系谓词的语句（concept、pattern、check、style、variant、外观时机的 over；pattern 的关系约束视同调用，7.2.4）记一条**警告**"空间事实未推导，X() 为空集"（每条语句一条，不论调用了几次；6.6.4、6.9），语句照常求值、不报错。概念、检查照常求值，结果按空事实计算。原型 compileOnce 不警告（`buildRelations([])` 静默）；本版按 6.9 加警告。宿主**应**在界面上提示事实未推导，这不属于本规范。''')

open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
