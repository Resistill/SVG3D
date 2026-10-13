//! 实体的参数与局部几何（spec/02 2.2）。M1 第一步只给局部包围盒；SDF / 射线 / 采样 / 凸块逐步补。

use crate::aabb::Aabb;
use crate::math::{self, Mat3, Vec3};
use crate::transform::Transform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn parse(s: &str) -> Option<Axis> {
        match s {
            "x" => Some(Axis::X),
            "y" => Some(Axis::Y),
            "z" => Some(Axis::Z),
            _ => None,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        }
    }
}

/// span 的截面在连线上的位置（2.2.8）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanPin {
    Bottom,
    Top,
    Center,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Solid {
    /// 2.2.1：S = [-w/2, w/2] × [0, h] × [-d/2, d/2]
    Box { size: Vec3 },
    /// 2.2.2：盒同 box，斜面高边在 -x 侧
    Wedge { size: Vec3 },
    /// 2.2.3：圆柱 / 圆台；r1 = 底半径，r2 = 顶半径（cyl 时 r2 = r1）
    Cyl { r1: f64, r2: f64, h: f64, axis: Axis },
    /// 2.2.4
    Sphere { r: f64 },
    /// 2.2.5：轮廓 (x, z) 与厚度 h
    Plate { points: Vec<[f64; 2]>, h: f64 },
    /// 2.2.6：点集的凸包
    Hull { points: Vec<Vec3> },
    /// 2.2.7：轮廓 (ρ, y) 绕 y 轴
    Revolve {
        profile: Vec<[f64; 2]>,
        angle: f64,
        smooth: bool,
    },
    /// 2.2.8 梁：长 L、截面 (w, t)、截面偏移 dy
    Beam { l: f64, w: f64, t: f64, dy: f64 },
    /// 2.2.8 杆
    Rod { l: f64, r: f64 },
    /// 2.2.8 台阶：水平跨度 H、总升 dy、级数 n、截面 (w, t)
    Steps {
        h_span: f64,
        dy: f64,
        n: u32,
        w: f64,
        t: f64,
    },
    /// 2.2.11：外部网格的外包络（size）与 pivot
    Mesh { size: Vec3, pivot: Vec3, src: String },
    /// 2.2.9 / 2.2.10：扫掠（点集由路径与截面决定；盒由全部环点给出）
    Sweep { points: Vec<Vec3> },
}

impl Solid {
    /// 局部包围盒（2.1.2、2.2 各节）
    pub fn local_box(&self) -> Aabb {
        match self {
            Solid::Box { size } | Solid::Wedge { size } | Solid::Mesh { size, .. } => Aabb::new(
                Vec3::new(-size.x / 2.0, 0.0, -size.z / 2.0),
                Vec3::new(size.x / 2.0, size.y, size.z / 2.0),
            ),
            Solid::Cyl { r1, r2, h, axis } => {
                let r = r1.max(*r2);
                match axis {
                    Axis::Y => Aabb::new(Vec3::new(-r, 0.0, -r), Vec3::new(r, *h, r)),
                    Axis::X => Aabb::new(Vec3::new(-h / 2.0, 0.0, -r), Vec3::new(h / 2.0, 2.0 * r, r)),
                    Axis::Z => Aabb::new(Vec3::new(-r, 0.0, -h / 2.0), Vec3::new(r, 2.0 * r, h / 2.0)),
                }
            }
            Solid::Sphere { r } => Aabb::new(Vec3::new(-r, 0.0, -r), Vec3::new(*r, 2.0 * r, *r)),
            Solid::Plate { points, h } => {
                let (mut x0, mut x1, mut z0, mut z1) =
                    (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
                for p in points {
                    x0 = x0.min(p[0]);
                    x1 = x1.max(p[0]);
                    z0 = z0.min(p[1]);
                    z1 = z1.max(p[1]);
                }
                Aabb::new(Vec3::new(x0, 0.0, z0), Vec3::new(x1, *h, z1))
            }
            Solid::Hull { points } => Aabb::of_points(points.iter().copied()).unwrap_or(Aabb::ZERO),
            Solid::Revolve { profile, .. } => {
                // 2.2.7：盒按整圈算：ρ 的最大值为 R，y 的范围
                let mut r = 0.0f64;
                let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
                for p in profile {
                    r = r.max(p[0].abs());
                    y0 = y0.min(p[1]);
                    y1 = y1.max(p[1]);
                }
                Aabb::new(Vec3::new(-r, y0, -r), Vec3::new(r, y1, r))
            }
            Solid::Beam { l, w, t, dy } => Aabb::new(
                Vec3::new(-l / 2.0, dy - t / 2.0, -w / 2.0),
                Vec3::new(l / 2.0, dy + t / 2.0, w / 2.0),
            ),
            Solid::Rod { l, r } => Aabb::new(Vec3::new(-l / 2.0, -r, -r), Vec3::new(l / 2.0, *r, *r)),
            Solid::Steps { h_span, dy, n, w, t } => {
                let n = *n as f64;
                let tau0 = -dy / 2.0 + dy * (1.0 / n);
                let tau_last = -dy / 2.0 + dy;
                Aabb::new(
                    Vec3::new(-h_span / 2.0, tau0.min(tau_last) - t, -w / 2.0),
                    Vec3::new(h_span / 2.0, tau0.max(tau_last), w / 2.0),
                )
            }
            Solid::Sweep { points } => Aabb::of_points(points.iter().copied()).unwrap_or(Aabb::ZERO),
        }
    }
}

/// perp(v)（3.5.4、2.2.8）：|v_y| < 0.9 时 unit(v × ŷ)，否则 unit(v × x̂)
pub fn perp(v: Vec3) -> Vec3 {
    let c = if v.y.abs() < 0.9 {
        v.cross(Vec3::Y)
    } else {
        v.cross(Vec3::X)
    };
    math::unit(c, 0.0).unwrap_or(Vec3::Z)
}

/// span 的局部系（2.2.8）：两端点 A、B 在父框架里。
/// 梁 / 杆：e_x = d/L，e_y = unit(ŷ − (ŷ·e_x) e_x)（竖直时 perp(e_x)），e_z = e_x × e_y，t = mid。
/// 台阶：R = R_y(ψ)，ψ 使局部 +x 指向 (d_x, 0, d_z)。
/// 返回 (变换, L, H)；L < ε_len 时 None。
pub fn span_frame(a: Vec3, b: Vec3, steps: bool) -> Option<(Transform, f64, f64)> {
    let d = b - a;
    let l = math::length(d);
    if l < 1e-9 {
        return None;
    }
    let h = math::sqrt(d.x * d.x + d.z * d.z);
    let mid = (a + b) * 0.5;
    if steps {
        // R_y(ψ)：e_x = unit(d_x, 0, d_z)
        let (c, s) = if h < 1e-9 { (1.0, 0.0) } else { (d.x / h, -d.z / h) };
        // R_y(θ) 把 x̂ 映到 (cos θ, 0, −sin θ)，所以 cos θ = d_x/H，sin θ = −d_z/H
        let r = Mat3::from_cols(Vec3::new(c, 0.0, -s), Vec3::Y, Vec3::new(s, 0.0, c));
        return Some((Transform::new(r, mid), l, h));
    }
    let ex = d / l;
    let yp = Vec3::Y - ex * Vec3::Y.dot(ex);
    let ey = match math::unit(yp, 1e-6) {
        Some(u) => u,
        None => perp(ex),
    };
    let ez = ex.cross(ey);
    Some((Transform::new(Mat3::from_cols(ex, ey, ez), mid), l, h))
}
