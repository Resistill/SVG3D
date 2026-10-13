import io
p = '/home/user/SVG3D/spec/04-derivation.md'
s = io.open(p, encoding='utf-8', newline='').read()

def rep(old, new, count=1):
    global s
    n = s.count(old)
    assert n == count, (n, old[:80])
    s = s.replace(old, new)

rep('被依赖：05-layers.md、07-semantics.md、08-metadata.md、09-compat.md、10-materials.md。',
    '被依赖：05-layers.md、07-semantics.md、08-metadata.md、09-compat.md、10-materials.md。本章反过来只**引用**这些章里的三处规则而不重复定义：几何时机选择器的逐构件表与检查顺序（05-layers.md 的 5.4.2）、变体语句的静态检查（07-semantics.md 的 7.8.1、7.8.8）、皮肤的可见与不可见清单（7.8.4）。')

rep('一条修改的选择器在阶段 5 求值失败（"找不到对象"、"名字不唯一"、"X 中没有子对象或锚点 Y"）或求得空集时，这条修改**推迟**到阶段 6：',
    '一条修改的选择器在阶段 5 求值失败（"找不到对象"、"名字不唯一"、"X 中没有子对象或锚点 Y"）或求得空集时，这条修改**推迟**到阶段 6（时机错误在求值前静态判定、不推迟，4.5.2）：')

rep('| 找不到对象 X / 名字 X 不唯一 / X 中没有子对象或锚点 Y | 选择器里的路径解析失败（阶段 5 推迟、阶段 6 报，4.4.4） | 整条语句无效 |',
    '| 找不到对象 X / 名字 X 不唯一 / X 中没有子对象或锚点 Y | 选择器里的路径解析失败（阶段 5 推迟、阶段 6 报，4.4.4）；解析失败的裸标识符是 concept / pattern 名时改报时机错误（4.5.2） | 整条语句无效 |')

rep('rotate 的 `n` 的常量要求与整数容差（4.4.1）。',
    'rotate 的 `n` 的常量要求与整数容差（4.4.1）；`defs[]` 条目的 `params` / `base` / `meta`（4.2.7：参考输出只有名字）；影子的 `late` 照抄原件（4.4.5）。')

io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ok')
