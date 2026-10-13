//! SVG3D 几何内核（spec/02、spec/06 的几何部分）。
//!
//! 全部数值是 f64，超越函数走 libm（`math` 模块），原则 18 的确定性约束由 clippy.toml 守住。

pub mod aabb;
pub mod convex;
pub mod curve;
pub mod math;
pub mod mesh;
pub mod poly2;
pub mod solid;
pub mod sweep;
pub mod tight;
pub mod transform;

pub use aabb::Aabb;
pub use curve::Curve;
pub use math::{Mat3, Vec2, Vec3};
pub use solid::{Axis, Solid, SpanPin};
pub use transform::Transform;
