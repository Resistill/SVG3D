import re,sys
p='/home/user/SVG3D/spec/crosswalk.md'
s=open(p,encoding='utf-8').read()
def rep(old,new,count=1):
    global s
    n=s.count(old)
    if n!=count:
        print('MISMATCH',n,repr(old[:80])); sys.exit(1)
    s=s.replace(old,new)

# 1 用途 bullet
rep('本表逐条语句的解释没有缺口，三处**输出形状**的缺口列在第 11 节并进 cross_chapter_requests。',
    '本表逐条语句的解释没有缺口；初稿核对第 5 列时发现的三处**输出形状**缺口已由 04-derivation.md 的 4.2.7、08-metadata.md 的 8.4.3、02-geometry.md 的 2.1.11 补齐，第 11 节记录缺口与补齐它们的条款。')
# 2 读法：字段名的出处
rep('坐标保留 4 位小数；"N" 代表任一阵列下标，"K" 代表任一副本组序号。\n',
    '坐标保留 4 位小数；"N" 代表任一阵列下标，"K" 代表任一副本组序号。\n- 第 5 列里输出字段的名字与形状按定义它们的章：`items[]` 与 `curves[]` 条目的字段按 02-geometry.md 的 2.1.11（字段名沿用原型，所以能直接对参考输出），条目的 `material` 按 10-materials.md 的 10.6，`defs[]` 条目按 04-derivation.md 的 4.2.7，顶层 `meta` 按 08-metadata.md 的 8.4.3，`nodes[]` 条目按 03-solving.md 的 3.1.6。这几节只定义值的含义与输出形状，不改变任何语句的解释，所以不计入第 4 列与第 9 节的计数。\n')
# 3 统计
rep('| 输出形状缺口 | 3（第 11 节：`defs` 条目形状、顶层 `meta` 与原型的差别、`curves[]` / `items[]` 几何字段） |',
    '| 输出形状缺口 | 0（初稿发现的 3 处——`defs[]` 条目形状、顶层 `meta`、`curves[]` / `items[]` 的几何字段——已分别由 4.2.7、8.4.3、2.1.11 补齐；第 11 节记录） |')
# 4 basics row 8
rep('| 8 | `box floor size=40,1,24 pin=top mat=a` | 实体 box、`size`、没有关系只有 `pin`、`mat` | 1.3.3、1.4.4、1.4.5；2.2.1；3.3.7（没有关系但写了 pin）、3.4.1；10.2 | 节点 `floor`[box]，`world.t` = (0, −1, 0)，`box` = [−20, −1, −12]…[20, 0, 12]；大纲 `floor 〔slab platform〕` |',
    '| 8 | `box floor size=40,1,24 pin=top mat=a` | 实体 box、`size`、没有关系只有 `pin`、`mat`；节点名 `floor` 与内置函数同名（软保留名：放行并警告，作路径首段 `on=floor…` 照常引用） | 1.3.3、1.4.4、1.4.5；2.2.1；3.3.7（没有关系但写了 pin）、3.4.1；10.2；1.6.9（语料命中处之一） | 节点 `floor`[box]，`world.t` = (0, −1, 0)，`box` = [−20, −1, −12]…[20, 0, 12]；大纲 `floor 〔slab platform〕`；`warnings` ※（参考 `[]`，本版一条"节点名 floor 与保留名相同…"） |')
# 5 curves row 48–56 col 5
rep('`plaza.benches.rK.back on plaza.benches.rK.seat`（6 条）；`plaza.benches.r2.seat intersects lane` −0.1 |',
    '`plaza.benches.rK.back on plaza.benches.rK.seat`（6 条）；`plaza.benches.r2.seat intersects lane` −0.1；`warnings` ※（参考 `[]`，本版一条"节点名 back 与保留名相同…"） |')
# 6 rotation 13–21
rep('`defs: ["turbine"]`（条目形状见第 11 节 G1）',
    '`defs` 的 `name` 序列 `["turbine"]`（条目形状 ※：参考是模板名数组，本版按 4.2.7 输出 `{name, line, file, params, base, meta}`）')
# 7 spaceport 30–39
rep('`defs` 含 `truss`（条目形状见 G1） |',
    '`defs` 的 `name` 序列含 `truss`（条目形状 ※，4.2.7） |')
# 8 blueprint 15–18
rep('| `defs: ["truss","shed","ribbed"]`；`hangar.roof.skin.base`',
    '| `defs` 的 `name` 序列 `["truss","shed","ribbed"]`（条目形状 ※，4.2.7）；`hangar.roof.skin.base`')
# 9 semantics 51–53
rep('子节点顺序 `body half half2 door chimney`，body 高 6；`concept building` 含它们 |',
    '子节点顺序 `body half half2 door chimney`，body 高 6；`concept building` 含它们；`defs` 的 `name` 序列 `["house","chimney_house","planks","townhouse"]`（条目形状 ※，4.2.7；`chimney_house` 条目的 `base` 为 `"house"`） |')
# 13 semantics rows 130/131
rep('| 130 | `variant deck use=planks` | variant；自动传参 w h d；皮肤定位到宿主包络底面中心 | 1.3.13；7.8.1（示例原句）、7.8.2、7.8.3、7.8.4；4.7；3.7.6 | `variants[0]` {line 130, use planks, members ["slab"]}；`slab` `hidden: true`；`slab.skin`[planks] `skin: true`，t = (0, 5.05, 0) = slab 的底面中心 |',
    '| 130 | `variant deck use=planks` | variant；自动传参 w h d；皮肤定位到宿主包络底面中心；宿主是实体（包络不读子节点）→ 皮肤不后置 | 1.3.13；7.8.1（示例原句）、7.8.2、7.8.3、7.8.4；4.7；3.7.6（后置的可判定规则） | `variants[0]` {line 130, use planks, members ["slab"]}；`slab` `hidden: true`；`slab.skin`[planks] `skin: true`、`late: false`，t = (0, 5.05, 0) = slab 的底面中心 |')
rep('| 131 | `variant tall_building use=townhouse` | 同上；宿主是模板实例（容器）→ 皮肤后置 | 7.8.3；3.7.6（参考输出 `street.h4.skin` 为 late 的说明）；4.4.5（h4 下的副本式节点 half2 随宿主隐藏） |',
    '| 131 | `variant tall_building use=townhouse` | 同上；宿主是模板实例（包络由全部子节点算出）→ 皮肤后置 | 7.8.3；3.7.6（后置的可判定规则；`street.h4.skin` 是语料里唯一后置的皮肤）；4.4.5（h4 下的副本式节点 half2 随宿主隐藏） |')
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
