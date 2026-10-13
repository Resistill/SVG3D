//! 刚体（含镜像）变换 T = (R, t)：p ↦ R p + t（spec/03 记法）。

use crate::math::{Mat3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub r: Mat3,
    pub t: Vec3,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        r: Mat3::IDENTITY,
        t: Vec3::ZERO,
    };

    pub fn new(r: Mat3, t: Vec3) -> Transform {
        Transform { r, t }
    }
    pub fn translation(t: Vec3) -> Transform {
        Transform { r: Mat3::IDENTITY, t }
    }
    pub fn rotation(r: Mat3) -> Transform {
        Transform { r, t: Vec3::ZERO }
    }
    /// 把点映到 R p + t。
    #[inline]
    pub fn apply(&self, p: Vec3) -> Vec3 {
        self.r * p + self.t
    }
    /// 把方向映到 R d。
    #[inline]
    pub fn apply_dir(&self, d: Vec3) -> Vec3 {
        self.r * d
    }
    /// 复合 (A ∘ B)(p) = A(B(p))：R = R_A R_B，t = R_A t_B + t_A。
    pub fn then_inner(&self, b: &Transform) -> Transform {
        Transform {
            r: self.r * b.r,
            t: self.r * b.t + self.t,
        }
    }
    /// 逆：(Rᵀ, −Rᵀ t)。R 是正交矩阵，用转置而不是数值求逆。
    pub fn inverse(&self) -> Transform {
        let rt = self.r.transpose();
        Transform {
            r: rt,
            t: -(rt * self.t),
        }
    }
    /// det R（镜像为 −1）。
    pub fn det(&self) -> f64 {
        self.r.determinant()
    }
    /// 关于过点 q、单位法线 n 的平面的反射（spec/03 3.9.3）：R = I − 2 n nᵀ，t = 2 (q·n) n。
    pub fn reflection(q: Vec3, n: Vec3) -> Transform {
        let r = Mat3::IDENTITY - Mat3::from_cols(n * (2.0 * n.x), n * (2.0 * n.y), n * (2.0 * n.z));
        Transform {
            r,
            t: n * (2.0 * q.dot(n)),
        }
    }
    /// 绕过点 q、单位轴 a 转 θ 度（spec/03 3.9.4）：R = Rodrigues，t = q − R q。
    pub fn rotation_about(q: Vec3, axis: Vec3, angle_deg: f64) -> Transform {
        let r = crate::math::rot_axis_deg(axis, angle_deg);
        Transform { r, t: q - r * q }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_and_inverse() {
        let a = Transform::new(crate::math::rot_y_deg(35.0), Vec3::new(1.0, 2.0, 3.0));
        let b = Transform::new(crate::math::rot_x_deg(-20.0), Vec3::new(-4.0, 0.5, 9.0));
        let p = Vec3::new(0.3, -1.2, 7.7);
        let ab = a.then_inner(&b);
        assert!((ab.apply(p) - a.apply(b.apply(p))).length() < 1e-12);
        let back = ab.inverse().apply(ab.apply(p));
        assert!((back - p).length() < 1e-12);
    }

    #[test]
    fn reflection_fixes_plane_and_flips_normal() {
        let m = Transform::reflection(Vec3::new(2.0, 0.0, 0.0), Vec3::X);
        assert!((m.apply(Vec3::new(2.0, 5.0, 1.0)) - Vec3::new(2.0, 5.0, 1.0)).length() < 1e-12);
        assert!((m.apply(Vec3::new(3.0, 0.0, 0.0)) - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-12);
        assert!((m.det() + 1.0).abs() < 1e-12);
    }
}
