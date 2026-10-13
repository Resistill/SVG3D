//! SVG3D 几何内核（spec/02、spec/06 的几何部分）。
//!
//! 全部数值是 f64，超越函数走 libm（`math` 模块），原则 18 的确定性约束由 clippy.toml 守住。

pub mod aabb;
pub mod math;
pub mod transform;

pub use aabb::Aabb;
pub use math::{Mat3, Vec2, Vec3};
pub use transform::Transform;
