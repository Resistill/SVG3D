//! 三角化（spec/02 2.1.9 与各实体节）：渲染、导出与评分用的闭合三角网格，顶点在局部系，法线朝外。
//! 三角形的顺序按各节给出的构造顺序；顶点坐标按公式直接计算，不焊接、不去重。

use crate::convex::hull_faces;
use crate::math::{self, Vec3};
use crate::poly2;
use crate::solid::{Axis, Solid};
use crate::transform::Transform;
use crate::Aabb;

/// 一个三角形 (a, b, c)
pub type Tri = [Vec3; 3];

/// n_cyl：圆柱、圆台、圆杆、旋转体的周向段数
pub const N_CYL: usize = 48;
/// n_sph：球的经向 × 纬向段数
pub const N_SPH_LON: usize = 48;
pub const N_SPH_LAT: usize = 24;

/// 四边形规则：从外面看逆时针的 (a, b, c, d) → (a, b, c)、(a, c, d)
fn quad(out: &mut Vec<Tri>, a: Vec3, b: Vec3, c: Vec3, d: Vec3) {
    out.push([a, b, c]);
    out.push([a, c, d]);
}

/// 扇形规则：凸多边形 (q_0..q_{k-1}) → (q_0, q_i, q_{i+1})
fn fan(out: &mut Vec<Tri>, q: &[Vec3]) {
    for i in 1..q.len().saturating_sub(1) {
        out.push([q[0], q[i], q[i + 1]]);
    }
}

/// 轴对齐盒的 12 个三角形（2.2.1 的面片顺序：底、顶、−x、+x、−z、+z）
pub fn box_tris(b: &Aabb, out: &mut Vec<Tri>) {
    let (lo, hi) = (b.min, b.max);
    let v = [
        Vec3::new(lo.x, lo.y, lo.z),
        Vec3::new(hi.x, lo.y, lo.z),
        Vec3::new(hi.x, lo.y, hi.z),
        Vec3::new(lo.x, lo.y, hi.z),
        Vec3::new(lo.x, hi.y, lo.z),
        Vec3::new(hi.x, hi.y, lo.z),
        Vec3::new(hi.x, hi.y, hi.z),
        Vec3::new(lo.x, hi.y, hi.z),
    ];
    for f in [[0, 1, 2, 3], [4, 7, 6, 5], [0, 3, 7, 4], [1, 5, 6, 2], [0, 4, 5, 1], [3, 2, 6, 7]] {
        quad(out, v[f[0]], v[f[1]], v[f[2]], v[f[3]]);
    }
}

/// 圆柱 / 圆台（2.2.3）：轴系 (c0, a, u0)，半径 r1 → r2，高 h
pub fn frustum_tris(c0: Vec3, a: Vec3, u0: Vec3, r1: f64, r2: f64, h: f64, out: &mut Vec<Tri>) {
    let w = a.cross(u0);
    let dir = |th: f64| u0 * math::cos_deg(th) + w * math::sin_deg(th);
    let c1 = c0 + a * h;
    let n = N_CYL;
    let bot: Vec<Vec3> = (0..n).map(|i| c0 + dir(7.5 * i as f64) * r1).collect();
    let top: Vec<Vec3> = (0..n).map(|i| c1 + dir(7.5 * i as f64) * r2).collect();
    if r1 > 0.0 {
        for i in 0..n {
            out.push([c0, bot[(i + 1) % n], bot[i]]);
        }
    }
    for i in 0..n {
        let j = (i + 1) % n;
        match (r1 > 0.0, r2 > 0.0) {
            (true, true) => quad(out, bot[i], bot[j], top[j], top[i]),
            (true, false) => out.push([bot[i], bot[j], c1]),
            (false, true) => out.push([c0, top[j], top[i]]),
            (false, false) => {}
        }
    }
    if r2 > 0.0 {
        for i in 0..n {
            out.push([c1, top[i], top[(i + 1) % n]]);
        }
    }
}

/// 球（2.2.4）：中心 c、半径 r
pub fn sphere_tris(c: Vec3, r: f64, out: &mut Vec<Tri>) {
    let (nlon, nlat) = (N_SPH_LON, N_SPH_LAT);
    let v = |j: usize, i: usize| {
        let phi = -90.0 + 180.0 * j as f64 / nlat as f64;
        let th = 360.0 * (i % nlon) as f64 / nlon as f64;
        let (cp, sp) = (math::cos_deg(phi), math::sin_deg(phi));
        c + Vec3::new(cp * math::cos_deg(th), sp, -cp * math::sin_deg(th)) * r
    };
    let ps = c - Vec3::Y * r;
    let pn = c + Vec3::Y * r;
    for i in 0..nlon {
        out.push([ps, v(1, i + 1), v(1, i)]);
    }
    for j in 1..nlat - 1 {
        for i in 0..nlon {
            quad(out, v(j, i), v(j, i + 1), v(j + 1, i + 1), v(j + 1, i));
        }
    }
    for i in 0..nlon {
        out.push([pn, v(nlat - 1, i), v(nlat - 1, i + 1)]);
    }
}

/// 轮廓（(x, z) 或 (ρ, y) 之类的二维点列）整理成正向；返回正向序列
fn positive(q: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if poly2::signed_area2(q) < 0.0 {
        q.iter().rev().copied().collect()
    } else {
        q.to_vec()
    }
}

/// 实体的三角化（局部系）。退化（耳切失败等）时返回空网格。
pub fn triangulate(s: &Solid) -> Vec<Tri> {
    let mut out = Vec::new();
    match s {
        Solid::Box { .. } | Solid::Beam { .. } | Solid::Mesh { .. } => box_tris(&s.local_box(), &mut out),
        Solid::Wedge { size } => {
            let (w, h, d) = (size.x, size.y, size.z);
            let v = [
                Vec3::new(-w / 2.0, 0.0, -d / 2.0),
                Vec3::new(w / 2.0, 0.0, -d / 2.0),
                Vec3::new(w / 2.0, h, -d / 2.0),
                Vec3::new(-w / 2.0, 0.0, d / 2.0),
                Vec3::new(w / 2.0, 0.0, d / 2.0),
                Vec3::new(w / 2.0, h, d / 2.0),
            ];
            out.push([v[0], v[2], v[1]]);
            out.push([v[3], v[4], v[5]]);
            quad(&mut out, v[0], v[1], v[4], v[3]);
            quad(&mut out, v[1], v[2], v[5], v[4]);
            quad(&mut out, v[0], v[3], v[5], v[2]);
        }
        Solid::Cyl { r1, r2, h, axis } => {
            let rr = r1.max(*r2);
            let (c0, a, u0) = match axis {
                Axis::Y => (Vec3::ZERO, Vec3::Y, Vec3::X),
                Axis::X => (Vec3::new(-h / 2.0, rr, 0.0), Vec3::X, Vec3::Y),
                Axis::Z => (Vec3::new(0.0, rr, -h / 2.0), Vec3::Z, Vec3::Y),
            };
            frustum_tris(c0, a, u0, *r1, *r2, *h, &mut out);
        }
        Solid::Rod { l, r } => frustum_tris(Vec3::new(-l / 2.0, 0.0, 0.0), Vec3::X, Vec3::Y, *r, *r, *l, &mut out),
        Solid::Sphere { r } => sphere_tris(Vec3::new(0.0, *r, 0.0), *r, &mut out),
        Solid::Plate { points, h } => {
            let q = positive(points);
            let tris = match poly2::earcut(&q) {
                Some(t) => t,
                None => return out,
            };
            let lo = |p: [f64; 2]| Vec3::new(p[0], 0.0, p[1]);
            let hi = |p: [f64; 2]| Vec3::new(p[0], *h, p[1]);
            for t in &tris {
                out.push([lo(q[t[0]]), lo(q[t[1]]), lo(q[t[2]])]);
            }
            for t in &tris {
                out.push([hi(q[t[0]]), hi(q[t[2]]), hi(q[t[1]])]);
            }
            let m = q.len();
            for i in 0..m {
                let j = (i + 1) % m;
                quad(&mut out, lo(q[i]), hi(q[i]), hi(q[j]), lo(q[j]));
            }
        }
        Solid::Hull { points } => {
            if let Some(faces) = hull_faces(points) {
                for f in &faces {
                    let q: Vec<Vec3> = f.iter().map(|&i| points[i]).collect();
                    fan(&mut out, &q);
                }
            }
        }
        Solid::Revolve { profile, angle, .. } => revolve_tris(profile, *angle, &mut out),
        Solid::Steps { h_span, dy, n, w, t } => {
            let nn = *n as f64;
            for i in 0..*n {
                let fi = i as f64;
                let tau = -dy / 2.0 + dy * (fi + 1.0) / nn;
                let b = Aabb::new(
                    Vec3::new(-h_span / 2.0 + h_span * fi / nn, tau - t, -w / 2.0),
                    Vec3::new(-h_span / 2.0 + h_span * (fi + 1.0) / nn, tau, w / 2.0),
                );
                box_tris(&b, &mut out);
            }
        }
        Solid::Sweep {
            rings,
            m,
            closed,
            profile,
            frames,
            ..
        } => {
            let m = *m;
            let n = frames.len();
            if m < 3 || n < 2 || rings.len() < n * m {
                return out;
            }
            let segs = if *closed { n } else { n - 1 };
            let rho = |i: usize, k: usize| rings[(i % n) * m + (k % m)];
            for i in 0..segs {
                for k in 0..m {
                    out.push([rho(i, k), rho(i + 1, k), rho(i + 1, k + 1)]);
                    out.push([rho(i, k), rho(i + 1, k + 1), rho(i, k + 1)]);
                }
            }
            if !*closed {
                if let Some(tris) = poly2::earcut(profile) {
                    let at = |f: &(Vec3, Vec3, Vec3), q: [f64; 2]| f.0 + f.1 * q[0] + f.2 * q[1];
                    let f0 = &frames[0];
                    let f1 = &frames[n - 1];
                    for t in &tris {
                        out.push([at(f0, profile[t[0]]), at(f0, profile[t[1]]), at(f0, profile[t[2]])]);
                    }
                    for t in &tris {
                        out.push([at(f1, profile[t[0]]), at(f1, profile[t[2]]), at(f1, profile[t[1]])]);
                    }
                }
            }
        }
    }
    out
}

/// 旋转体（2.2.7）：轮廓 Q（已处理）绕 y 轴转 angle 度
fn revolve_tris(profile: &[[f64; 2]], angle: f64, out: &mut Vec<Tri>) {
    let q = positive(profile);
    let full = angle >= 360.0;
    let n = if full {
        N_CYL
    } else {
        ((N_CYL as f64 * angle / 360.0).ceil() as usize).max(1)
    };
    let dir = |th: f64| Vec3::new(math::cos_deg(th), 0.0, -math::sin_deg(th));
    let theta = |j: usize| angle * j as f64 / n as f64;
    let v = |i: usize, j: usize| dir(theta(j)) * q[i][0] + Vec3::Y * q[i][1];
    let m = q.len();
    for i in 0..m {
        let i1 = (i + 1) % m;
        let (r0, r1) = (q[i][0], q[i1][0]);
        if r0 <= 0.0 && r1 <= 0.0 {
            continue; // 轴边
        }
        for j in 0..n {
            let (a, b, c, d) = (v(i, j), v(i, j + 1), v(i1, j + 1), v(i1, j));
            if r0 > 0.0 {
                out.push([a, b, c]);
            }
            if r1 > 0.0 {
                out.push([a, c, d]);
            }
        }
    }
    if !full {
        if let Some(tris) = poly2::earcut(&q) {
            let p0 = |p: [f64; 2]| Vec3::new(p[0], p[1], 0.0);
            let p1 = |p: [f64; 2]| dir(angle) * p[0] + Vec3::Y * p[1];
            for t in &tris {
                out.push([p0(q[t[0]]), p0(q[t[1]]), p0(q[t[2]])]);
            }
            for t in &tris {
                out.push([p1(q[t[0]]), p1(q[t[2]]), p1(q[t[1]])]);
            }
        }
    }
}

/// 变换到世界系：det R < 0 时交换每个三角形的第 2、3 个顶点（2.1.9）
pub fn transform_tris(tris: &[Tri], w: &Transform) -> Vec<Tri> {
    let flip = w.det() < 0.0;
    tris.iter()
        .map(|t| {
            let a = w.apply(t[0]);
            let b = w.apply(t[1]);
            let c = w.apply(t[2]);
            if flip {
                [a, c, b]
            } else {
                [a, b, c]
            }
        })
        .collect()
}

/// 散度体积（2.1.6.3，取绝对值）、表面积与包围盒
pub fn mesh_stats(tris: &[Tri]) -> (f64, f64, Aabb) {
    let mut vol = 0.0;
    let mut area = 0.0;
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for t in tris {
        vol += t[0].dot(t[1].cross(t[2])) / 6.0;
        area += math::length((t[1] - t[0]).cross(t[2] - t[0])) / 2.0;
        for p in t {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
    }
    (vol.abs(), area, Aabb::new(lo, hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed_volume(tris: &[Tri]) -> f64 {
        tris.iter().map(|t| t[0].dot(t[1].cross(t[2])) / 6.0).sum()
    }

    #[test]
    fn box_volume_and_orientation() {
        let s = Solid::Box {
            size: Vec3::new(2.0, 3.0, 4.0),
        };
        let t = triangulate(&s);
        assert_eq!(t.len(), 12);
        assert!((closed_volume(&t) - 24.0).abs() < 1e-12);
        // 镜像后体积符号不变（绕序已翻转）
        let w = Transform::reflection(Vec3::ZERO, Vec3::X);
        assert!((closed_volume(&transform_tris(&t, &w)) - 24.0).abs() < 1e-12);
    }

    #[test]
    fn wedge_cyl_sphere_volumes() {
        let w = Solid::Wedge {
            size: Vec3::new(2.0, 3.0, 4.0),
        };
        assert!((closed_volume(&triangulate(&w)) - 12.0).abs() < 1e-12);
        let c = Solid::Cyl {
            r1: 1.0,
            r2: 0.0,
            h: 3.0,
            axis: Axis::X,
        };
        let v = closed_volume(&triangulate(&c));
        assert!(v > 0.0 && (v - std::f64::consts::PI / 1.0) < 0.0 && v > 0.99 * std::f64::consts::PI);
        let s = Solid::Sphere { r: 1.0 };
        let v = closed_volume(&triangulate(&s));
        let exact = 4.0 / 3.0 * std::f64::consts::PI;
        assert!(v > 0.98 * exact && v < exact);
    }

    #[test]
    fn plate_hull_revolve_steps() {
        let p = Solid::Plate {
            points: vec![[0.0, 0.0], [0.0, 2.0], [1.0, 2.0], [1.0, 0.0]],
            h: 0.5,
        };
        assert!((closed_volume(&triangulate(&p)) - 1.0).abs() < 1e-12);
        let h = Solid::Hull {
            points: vec![
                Vec3::ZERO,
                Vec3::X,
                Vec3::Y,
                Vec3::Z,
                Vec3::new(1.0, 1.0, 1.0),
            ],
        };
        assert!(closed_volume(&triangulate(&h)) > 0.0);
        let r = Solid::Revolve {
            profile: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]],
            angle: 360.0,
            smooth: false,
        };
        let v = closed_volume(&triangulate(&r));
        assert!(v > 0.99 * 2.0 * std::f64::consts::PI && v < 2.0 * std::f64::consts::PI);
        let r2 = Solid::Revolve {
            profile: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]],
            angle: 90.0,
            smooth: false,
        };
        let v2 = closed_volume(&triangulate(&r2));
        assert!(v2 > 0.99 * std::f64::consts::PI / 2.0 && v2 < std::f64::consts::PI / 2.0 + 1e-9);
        let st = Solid::Steps {
            h_span: 4.0,
            dy: 2.0,
            n: 4,
            w: 1.0,
            t: 0.25,
        };
        assert!((closed_volume(&triangulate(&st)) - 1.0).abs() < 1e-12);
    }
}
