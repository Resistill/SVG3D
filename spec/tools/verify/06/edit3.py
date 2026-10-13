p='/home/user/SVG3D/spec/06-facts.md'
s=open(p,encoding='utf-8').read()
def rep(old,new):
    global s
    assert s.count(old)==1, old[:60]
    s=s.replace(old,new)
rep("但 `corpus/exports/` 的 11 个文件受影响很大：", "但 `corpus/exports/` 的 10 个文件里有 8 个受影响很大：")
rep("数量集中在 rotation.L0（709 条事实里 249 条 `intersects` → `inside`：过山车的枕木与立柱落在三条轨道 mesh 的外包络盒里）。",
    "数量集中在 rotation.L0（709 条事实里 249 条 `intersects` → `inside`：247 条是过山车的枕木 `coaster_ties.*.tie` 对三条轨道 `coaster_railL` / `coaster_railR` / `coaster_spine`——逆向把扫掠轨道写成 mesh，细长的外包络盒把途经的枕木全部\"包住\"；另 2 条是 `radar_horn` / `radar_feed` 对 `radar_dish`）。")
rep("记录按节点顺序给出。", "记录按节点顺序给出。`contact` 提供者只读其中 6.2.2 表里的量与两个序号（6.1.2）；节点字段给其他提供者读标签、元数据、种类用。")
open(p,'w',encoding='utf-8',newline='\n').write(s)
print('ok')
