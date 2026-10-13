//! 紧包围盒（spec/02 2.1.8）：按真实形状在世界系里算最小轴对齐盒——极值点集、圆、圆弧（2.1.6.4）。

use crate::aabb::Aabb;
use crate::math::{self, Vec3};
use crate::solid::{Axis, Solid};
use crate::transform::Transform;

/// 累积器：逐点 / 逐圆 / 逐弧扩张
struct Acc {
    min: Vec3,
    max: Vec3,
    any: bool,
}

impl Acc {
    fn new() -> Acc {
        Acc {
            min: Vec3::splat(f64::INFINITY),
            max: Vec3::splat(f64::NEG_INFINITY),
            any: false,
        }
    }
    fn point(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
        self.any = true;
    }
    fn range(&mut self, k: usize, lo: f64, hi: f64) {
        self.min[k] = self.min[k].min(lo);
        self.max[k] = self.max[k].max(hi);
        self.any = true;
    }
    /// 世界系里的圆：圆心 c、半径 r、单位轴 a
    fn circle(&mut self, c: Vec3, r: f64, a: Vec3) {
        for k in 0..3 {
            let s = (1.0 - a[k] * a[k]).max(0.0);
            let e = r * math::sqrt(s);
            self.range(k, c[k] - e, c[k] + e);
        }
    }
    /// 世界系里的圆弧 p(θ) = c + r(cosθ u + sinθ v)，θ ∈ [θ0, θ1]（度）
    fn arc(&mut self, c: Vec3, r: f64, u: Vec3, v: Vec3, th0: f64, th1: f64) {
        for k in 0..3 {
            let psi = math::deg(math::atan2(v[k], u[k]));
            let mut cands = vec![th0, th1];
            // ψ + 180 j 落在 [θ0, θ1] 内的
            let jlo = ((th0 - psi) / 180.0).ceil() as i64;
            let jhi = ((th1 - psi) / 180.0).floor() as i64;
            let mut j = jlo;
            while j <= jhi {
                cands.push(psi + 180.0 * j as f64);
                j += 1;
            }
            for th in cands {
                let p = c[k] + r * (math::cos_deg(th) * u[k] + math::sin_deg(th) * v[k]);
                self.range(k, p, p);
            }
        }
    }
    fn finish(self) -> Aabb {
        if self.any {
            Aabb::new(self.min, self.max)
        } else {
            Aabb::ZERO
        }
    }
}

/// cyl / cone / rod 的轴系 (c0, a)
fn cyl_axis(r1: f64, r2: f64, h: f64, axis: Axis) -> (Vec3, Vec3) {
    let rr = r1.max(r2);
    match axis {
        Axis::Y => (Vec3::ZERO, Vec3::Y),
        Axis::X => (Vec3::new(-h / 2.0, rr, 0.0), Vec3::X),
        Axis::Z => (Vec3::new(0.0, rr, -h / 2.0), Vec3::Z),
    }
}

/// 实体在世界变换 w 下的紧包围盒
pub fn tight_box(s: &Solid, w: &Transform) -> Aabb {
    let mut acc = Acc::new();
    let pt = |acc: &mut Acc, p: Vec3| acc.point(w.apply(p));
    match s {
        Solid::Box { size } | Solid::Mesh { size, .. } => {
            let b = Aabb::new(
                Vec3::new(-size.x / 2.0, 0.0, -size.z / 2.0),
                Vec3::new(size.x / 2.0, size.y, size.z / 2.0),
            );
            for c in b.corners() {
                pt(&mut acc, c);
            }
        }
        Solid::Wedge { size } => {
            let (hw, h, hd) = (size.x / 2.0, size.y, size.z / 2.0);
            for p in [
                Vec3::new(-hw, 0.0, -hd),
                Vec3::new(hw, 0.0, -hd),
                Vec3::new(hw, 0.0, hd),
                Vec3::new(-hw, 0.0, hd),
                Vec3::new(-hw, h, -hd),
                Vec3::new(-hw, h, hd),
            ] {
                pt(&mut acc, p);
            }
        }
        Solid::Cyl { r1, r2, h, axis } => {
            let (c0, a) = cyl_axis(*r1, *r2, *h, *axis);
            let wa = w.r * a;
            acc.circle(w.apply(c0), *r1, wa);
            acc.circle(w.apply(c0 + a * *h), *r2, wa);
        }
        Solid::Sphere { r } => {
            let c = w.apply(Vec3::new(0.0, *r, 0.0));
            for k in 0..3 {
                acc.range(k, c[k] - r, c[k] + r);
            }
        }
        Solid::Plate { points, h } => {
            for p in points {
                pt(&mut acc, Vec3::new(p[0], 0.0, p[1]));
                pt(&mut acc, Vec3::new(p[0], *h, p[1]));
            }
        }
        Solid::Hull { points } => {
            for p in points {
                pt(&mut acc, *p);
            }
        }
        Solid::Revolve { profile, angle, .. } => {
            let full = *angle >= 360.0;
            for q in profile {
                let c = w.apply(Vec3::new(0.0, q[1], 0.0));
                if full {
                    acc.circle(c, q[0], w.r * Vec3::Y);
                } else {
                    acc.arc(c, q[0], w.r * Vec3::X, w.r * Vec3::NEG_Z, 0.0, *angle);
                }
            }
        }
        Solid::Beam { l, w: bw, t, dy } => {
            let b = Aabb::new(
                Vec3::new(-l / 2.0, dy - t / 2.0, -bw / 2.0),
                Vec3::new(l / 2.0, dy + t / 2.0, bw / 2.0),
            );
            for c in b.corners() {
                pt(&mut acc, c);
            }
        }
        Solid::Rod { l, r } => {
            let a = w.r * Vec3::X;
            acc.circle(w.apply(Vec3::new(-l / 2.0, 0.0, 0.0)), *r, a);
            acc.circle(w.apply(Vec3::new(l / 2.0, 0.0, 0.0)), *r, a);
        }
        Solid::Steps {
            h_span,
            dy,
            n,
            w: sw,
            t,
        } => {
            let nn = *n as f64;
            for i in 0..*n {
                let fi = i as f64;
                let tau = -dy / 2.0 + dy * (fi + 1.0) / nn;
                let b = Aabb::new(
                    Vec3::new(-h_span / 2.0 + h_span * fi / nn, tau - t, -sw / 2.0),
                    Vec3::new(-h_span / 2.0 + h_span * (fi + 1.0) / nn, tau, sw / 2.0),
                );
                for c in b.corners() {
                    pt(&mut acc, c);
                }
            }
        }
        Solid::Sweep { rings, .. } => {
            for p in rings {
                pt(&mut acc, *p);
            }
        }
    }
    acc.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_aligned_box_matches_local() {
        let s = Solid::Box {
            size: Vec3::new(2.0, 4.0, 6.0),
        };
        let w = Transform::translation(Vec3::new(1.0, 2.0, 3.0));
        let t = tight_box(&s, &w);
        assert_eq!(t, s.local_box().transformed(&w));
    }

    #[test]
    fn tilted_cyl_uses_circles() {
        let s = Solid::Cyl {
            r1: 1.0,
            r2: 1.0,
            h: 2.0,
            axis: Axis::Y,
        };
        let w = Transform::rotation(math::rot_z_deg(90.0));
        let t = tight_box(&s, &w);
        // 轴转到 -x：x ∈ [-2, 0]，y、z ∈ [-1, 1]
        assert!((t.min.x + 2.0).abs() < 1e-12 && t.max.x.abs() < 1e-12);
        assert!((t.min.y + 1.0).abs() < 1e-12 && (t.max.y - 1.0).abs() < 1e-12);
    }

    #[test]
    fn arc_extremes() {
        let mut acc = Acc::new();
        acc.arc(Vec3::ZERO, 1.0, Vec3::X, Vec3::NEG_Z, 0.0, 90.0);
        let b = acc.finish();
        assert!((b.max.x - 1.0).abs() < 1e-12 && b.min.x.abs() < 1e-12);
        assert!((b.min.z + 1.0).abs() < 1e-12 && b.max.z.abs() < 1e-12);
    }
}
