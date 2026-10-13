//! SVG3D 评分 crate。M1：真值评测 `eval`（闭环：原文与逆向描述各自编译后按 IoU 匹配，比形状、事实、压缩、语义、描述质量）。
//! M2：自评分数 `score`（S = L_stmt + L_mesh + R_src + R_recon + R_vol，另报 E_edit）及其几何支撑
//! （`objects` 世界三角形、`sample` 采样、`surfaces` 表面树、`volume` 体积误差、`roles` 角色）。

pub mod edit;
pub mod eval;
pub mod objects;
pub mod roles;
pub mod sample;
pub mod score;
pub mod surfaces;
pub mod volume;
