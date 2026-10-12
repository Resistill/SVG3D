# corpus/reference/

原型冻结前导出的参考输出包，另一个任务会填进来。M1 到 M3 的验收都对着它比（按定义与容差，不逐字节）。

结构：

- `examples/`：每个示例的编译 JSON（逐物体变换、包围盒、事实）、eval 结果
- `exports/`：`out/*.s3d` 的 eval 结果、`compare` 结果与渲染图
- `work/`：三个实战工程（dorm、h1、sc）的 `score.json` 等；**被 gitignore**（体积可能很大，之后视大小决定是否入库）
