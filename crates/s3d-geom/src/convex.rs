//! 凸块（spec/02 各实体的"凸块"一项、2.1.6.1 的表示）与分离量（spec/06 6.4）。

use crate::aabb::Aabb;
use crate::math::{self, Vec3};
use crate::poly2;
use crate::solid::{Axis, Solid};
use crate::transform::Transform;

const EPS_DIR: f64 = 1e-6;
const EPS_LEN: f64 = 1e-9;
/// n_part：圆类凸块的边数
pub const N_PART: usize = 12;

/// 一个凸块的形状
#[derive(Clone, Debug)]
pub enum Shape {
    /// 凸多面体：顶点、法线集（按方向去重）、棱集（按方向去重）
    Poly {
        verts: Vec<Vec3>,
        normals: Vec<Vec3>,
        edges: Vec<Vec3>,
    },
    Sphere {
        c: Vec3,
        r: f64,
    },
}

/// 凸块：形状 + 近似标记
#[derive(Clone, Debug)]
pub struct Part {
    pub shape: Shape,
    pub approx: bool,
}

fn push_dir_dedup(set: &mut Vec<Vec3>, d: Vec3) {
    if set.iter().any(|m| m.dot(d).abs() >= 1.0 - EPS_DIR) {
        return;
    }
    set.push(d);
}

/// 由顶点与面片（顶点序号列表）构造凸多面体：Newell 法线，法线集与棱集按面片序、顶点序去重
pub fn poly_from_faces(verts: Vec<Vec3>, faces: &[Vec<usize>]) -> Shape {
    let mut normals = Vec::new();
    let mut edges = Vec::new();
    for f in faces {
        let k = f.len();
        if k < 3 {
            continue;
        }
        let mut n = Vec3::ZERO;
        for i in 0..k {
            let a = verts[f[i]];
            let b = verts[f[(i + 1) % k]];
            n += (a - b).cross(a + b);
        }
        if let Some(u) = math::unit(n, EPS_LEN) {
            push_dir_dedup(&mut normals, u);
        }
        for i in 0..k {
            let a = verts[f[i]];
            let b = verts[f[(i + 1) % k]];
            if let Some(u) = math::unit(b - a, EPS_LEN) {
                push_dir_dedup(&mut edges, u);
            }
        }
    }
    Shape::Poly {
        verts,
        normals,
        edges,
    }
}

/// 轴对齐盒的凸块（2.2.1 的顶点编号与面片）
pub fn box_shape(b: &Aabb) -> Shape {
    let (lo, hi) = (b.min, b.max);
    let verts = vec![
        Vec3::new(lo.x, lo.y, lo.z),
        Vec3::new(hi.x, lo.y, lo.z),
        Vec3::new(hi.x, lo.y, hi.z),
        Vec3::new(lo.x, lo.y, hi.z),
        Vec3::new(lo.x, hi.y, lo.z),
        Vec3::new(hi.x, hi.y, lo.z),
        Vec3::new(hi.x, hi.y, hi.z),
        Vec3::new(lo.x, hi.y, hi.z),
    ];
    let faces: Vec<Vec<usize>> = vec![
        vec![0, 1, 2, 3],
        vec![4, 7, 6, 5],
        vec![0, 3, 7, 4],
        vec![1, 5, 6, 2],
        vec![0, 4, 5, 1],
        vec![3, 2, 6, 7],
    ];
    poly_from_faces(verts, &faces)
}

/// 圆台的 12 边内接棱台（2.2.3）：轴系 (c0, a, u0)，半径 r1 / r2，高 h
pub fn frustum_shape(c0: Vec3, a: Vec3, u0: Vec3, r1: f64, r2: f64, h: f64) -> Shape {
    let w = a.cross(u0);
    let dir = |th: f64| u0 * math::cos_deg(th) + w * math::sin_deg(th);
    let c1 = c0 + a * h;
    let mut verts = Vec::new();
    let mut faces: Vec<Vec<usize>> = Vec::new();
    let n = N_PART;
    match (r1 > 0.0, r2 > 0.0) {
        (true, true) => {
            for i in 0..n {
                verts.push(c0 + dir(30.0 * i as f64) * r1);
            }
            for i in 0..n {
                verts.push(c1 + dir(30.0 * i as f64) * r2);
            }
            faces.push((0..n).rev().map(|i| (i + 1) % n).collect()); // 底 (B0, B11, …, B1)
            faces.push((0..n).map(|i| n + i).collect());
            for i in 0..n {
                faces.push(vec![i, (i + 1) % n, n + (i + 1) % n, n + i]);
            }
        }
        (true, false) => {
            for i in 0..n {
                verts.push(c0 + dir(30.0 * i as f64) * r1);
            }
            verts.push(c1);
            faces.push((0..n).rev().map(|i| (i + 1) % n).collect());
            for i in 0..n {
                faces.push(vec![i, (i + 1) % n, n]);
            }
        }
        (false, true) => {
            verts.push(c0);
            for i in 0..n {
                verts.push(c1 + dir(30.0 * i as f64) * r2);
            }
            faces.push((0..n).map(|i| 1 + i).collect());
            for i in 0..n {
                faces.push(vec![0, 1 + (i + 1) % n, 1 + i]);
            }
        }
        (false, false) => {
            verts.push(c0);
            verts.push(c1);
        }
    }
    poly_from_faces(verts, &faces)
}

/// 点集的凸包（小规模、确定性的平面枚举）；共面点集返回 None
pub fn hull_shape(pts: &[Vec3]) -> Option<Shape> {
    let faces = hull_faces(pts)?;
    Some(poly_from_faces(pts.to_vec(), &faces))
}

/// 凸包的面片（2.2.6）：每个面片是输入点序号的列表，从外面看逆时针，按平面枚举的顺序；共面点集返回 None
pub fn hull_faces(pts: &[Vec3]) -> Option<Vec<Vec<usize>>> {
    let n = pts.len();
    if n < 4 {
        return None;
    }
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for p in pts {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let e = (hi - lo).max_element();
    let eps = 1e-9 * e.max(1e-9);
    let mut faces: Vec<Vec<usize>> = Vec::new();
    let mut planes: Vec<(Vec3, f64)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let nrm = match math::unit((pts[j] - pts[i]).cross(pts[k] - pts[i]), eps) {
                    Some(u) => u,
                    None => continue,
                };
                let d = nrm.dot(pts[i]);
                let mut pos = false;
                let mut neg = false;
                for p in pts {
                    let s = nrm.dot(*p) - d;
                    if s > eps {
                        pos = true;
                    } else if s < -eps {
                        neg = true;
                    }
                    if pos && neg {
                        break;
                    }
                }
                if pos && neg {
                    continue;
                }
                let (nrm, d) = if pos { (-nrm, -d) } else { (nrm, d) };
                if planes
                    .iter()
                    .any(|(m, dd)| m.dot(nrm) >= 1.0 - EPS_DIR && (dd - d).abs() <= eps)
                {
                    continue;
                }
                // 面片：平面上的全部点，按绕法线的角序（从输入序号最小的点起）
                let on: Vec<usize> = (0..n).filter(|&q| (nrm.dot(pts[q]) - d).abs() <= eps).collect();
                let c = on.iter().fold(Vec3::ZERO, |acc, &q| acc + pts[q]) / on.len() as f64;
                let u = math::unit(pts[on[0]] - c, eps).unwrap_or(crate::solid::perp(nrm));
                let v = nrm.cross(u);
                let mut ord: Vec<(f64, usize)> = on
                    .iter()
                    .map(|&q| {
                        let r = pts[q] - c;
                        (math::atan2(r.dot(v), r.dot(u)), q)
                    })
                    .collect();
                ord.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.cmp(&b.1)));
                planes.push((nrm, d));
                faces.push(ord.into_iter().map(|(_, q)| q).collect());
            }
        }
    }
    if faces.len() < 4 {
        return None;
    }
    Some(faces)
}

/// 一组 (x, z) 轮廓沿 y 拉伸的棱柱（plate 的凸块；轮廓须为凸且已规整）
fn prism_xz(q: &[[f64; 2]], y0: f64, y1: f64) -> Shape {
    let m = q.len();
    let mut verts = Vec::with_capacity(2 * m);
    for p in q {
        verts.push(Vec3::new(p[0], y0, p[1]));
    }
    for p in q {
        verts.push(Vec3::new(p[0], y1, p[1]));
    }
    let mut faces: Vec<Vec<usize>> = vec![(0..m).collect(), (0..m).rev().map(|i| m + i).collect()];
    for i in 0..m {
        faces.push(vec![i, m + i, m + (i + 1) % m, (i + 1) % m]);
    }
    poly_from_faces(verts, &faces)
}

/// 实体的凸块列表（局部系）
pub fn parts_of(s: &Solid) -> Vec<Part> {
    let exact = |shape: Shape| Part { shape, approx: false };
    match s {
        Solid::Box { .. } | Solid::Beam { .. } | Solid::Rod { .. } if false => unreachable!(),
        Solid::Box { .. } => vec![exact(box_shape(&s.local_box()))],
        Solid::Mesh { .. } => vec![Part {
            shape: box_shape(&s.local_box()),
            approx: true,
        }],
        Solid::Wedge { size } => {
            let (hw, h, hd) = (size.x / 2.0, size.y, size.z / 2.0);
            let verts = vec![
                Vec3::new(-hw, 0.0, -hd),
                Vec3::new(hw, 0.0, -hd),
                Vec3::new(hw, h, -hd),
                Vec3::new(-hw, 0.0, hd),
                Vec3::new(hw, 0.0, hd),
                Vec3::new(hw, h, hd),
            ];
            let faces: Vec<Vec<usize>> = vec![
                vec![0, 2, 1],
                vec![3, 4, 5],
                vec![0, 1, 4, 3],
                vec![1, 2, 5, 4],
                vec![0, 3, 5, 2],
            ];
            vec![exact(poly_from_faces(verts, &faces))]
        }
        Solid::Cyl { r1, r2, h, axis } => {
            let rr = r1.max(*r2);
            let (c0, a, u0) = match axis {
                Axis::Y => (Vec3::ZERO, Vec3::Y, Vec3::X),
                Axis::X => (Vec3::new(-h / 2.0, rr, 0.0), Vec3::X, Vec3::Y),
                Axis::Z => (Vec3::new(0.0, rr, -h / 2.0), Vec3::Z, Vec3::Y),
            };
            vec![exact(frustum_shape(c0, a, u0, *r1, *r2, *h))]
        }
        Solid::Sphere { r } => vec![exact(Shape::Sphere {
            c: Vec3::new(0.0, *r, 0.0),
            r: *r,
        })],
        Solid::Plate { points, h } => {
            let (mut q, _) = poly2::normalize(points);
            if q.len() < 3 {
                return vec![];
            }
            if poly2::signed_area2(&q) < 0.0 {
                q.reverse();
            }
            if poly2::is_convex(&q) {
                vec![exact(prism_xz(&q, 0.0, *h))]
            } else {
                match poly2::earcut(&q) {
                    Some(tris) => tris
                        .iter()
                        .map(|t| exact(prism_xz(&[q[t[0]], q[t[1]], q[t[2]]], 0.0, *h)))
                        .collect(),
                    None => vec![],
                }
            }
        }
        Solid::Hull { points } => match hull_shape(points) {
            Some(sh) => vec![exact(sh)],
            None => vec![],
        },
        Solid::Revolve { profile, .. } => {
            let mut rmax = 0.0f64;
            let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
            for p in profile {
                rmax = rmax.max(p[0]);
                y0 = y0.min(p[1]);
                y1 = y1.max(p[1]);
            }
            vec![Part {
                shape: frustum_shape(Vec3::new(0.0, y0, 0.0), Vec3::Y, Vec3::X, rmax, rmax, y1 - y0),
                approx: true,
            }]
        }
        Solid::Beam { .. } => vec![exact(box_shape(&s.local_box()))],
        Solid::Rod { l, r } => vec![exact(frustum_shape(
            Vec3::new(-l / 2.0, 0.0, 0.0),
            Vec3::X,
            Vec3::Y,
            *r,
            *r,
            *l,
        ))],
        Solid::Steps { h_span, dy, n, w, t } => {
            let nn = *n as f64;
            (0..*n)
                .map(|i| {
                    let fi = i as f64;
                    let tau = -dy / 2.0 + dy * (fi + 1.0) / nn;
                    exact(box_shape(&Aabb::new(
                        Vec3::new(-h_span / 2.0 + h_span * fi / nn, tau - t, -w / 2.0),
                        Vec3::new(-h_span / 2.0 + h_span * (fi + 1.0) / nn, tau, w / 2.0),
                    )))
                })
                .collect()
        }
        Solid::Sweep {
            rings,
            m,
            closed,
            profile,
            ..
        } => {
            let m = *m;
            let n = rings.len() / m.max(1);
            let segs = if *closed { n } else { n.saturating_sub(1) };
            let sel: Vec<usize> = if m > 8 {
                (0..8).map(|j| j * m / 8).collect()
            } else {
                (0..m).collect()
            };
            let mp = sel.len();
            let convex = poly2::is_convex(profile);
            let mut out = Vec::with_capacity(segs);
            for i in 0..segs {
                let i1 = (i + 1) % n;
                let mut verts = Vec::with_capacity(2 * mp);
                for &k in &sel {
                    verts.push(rings[i * m + k]);
                }
                for &k in &sel {
                    verts.push(rings[i1 * m + k]);
                }
                if convex {
                    // 面片顺序与顶点序是定义的一部分（2.1.6.1 的棱集按先出现去重；相邻两环的沿路径棱只差
                    // 微小角度，保留的是第一条）：起点环按 k 递减、终点环按 k 递增、侧面 (s_j, s_{j+1}, e_{j+1}, e_j)
                    let mut faces: Vec<Vec<usize>> =
                        vec![(0..mp).rev().collect(), (0..mp).map(|j| mp + j).collect()];
                    for j in 0..mp {
                        faces.push(vec![j, (j + 1) % mp, mp + (j + 1) % mp, mp + j]);
                    }
                    out.push(exact(poly_from_faces(verts, &faces)));
                } else if let Some(sh) = hull_shape(&verts) {
                    out.push(Part {
                        shape: sh,
                        approx: true,
                    });
                }
            }
            out
        }
    }
}

impl Shape {
    pub fn transformed(&self, w: &Transform) -> Shape {
        match self {
            Shape::Poly {
                verts,
                normals,
                edges,
            } => Shape::Poly {
                verts: verts.iter().map(|v| w.apply(*v)).collect(),
                normals: normals.iter().map(|d| w.r * *d).collect(),
                edges: edges.iter().map(|d| w.r * *d).collect(),
            },
            Shape::Sphere { c, r } => Shape::Sphere {
                c: w.apply(*c),
                r: *r,
            },
        }
    }

    pub fn bbox(&self) -> Aabb {
        match self {
            Shape::Poly { verts, .. } => Aabb::of_points(verts.iter().copied()).unwrap_or(Aabb::ZERO),
            Shape::Sphere { c, r } => Aabb::new(*c - Vec3::splat(*r), *c + Vec3::splat(*r)),
        }
    }

    /// 投影区间（6.4.1）
    pub fn proj(&self, l: Vec3) -> (f64, f64) {
        match self {
            Shape::Poly { verts, .. } => {
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for v in verts {
                    let d = v.dot(l);
                    lo = lo.min(d);
                    hi = hi.max(d);
                }
                (lo, hi)
            }
            Shape::Sphere { c, r } => {
                let d = c.dot(l);
                (d - r, d + r)
            }
        }
    }
}

/// 两个凸块的分离量（6.4.2）
pub fn gap(a: &Shape, b: &Shape) -> f64 {
    let mut axes: Vec<Vec3> = Vec::new();
    if let Shape::Poly { normals, .. } = a {
        axes.extend(normals.iter().copied());
    }
    if let Shape::Poly { normals, .. } = b {
        axes.extend(normals.iter().copied());
    }
    match (a, b) {
        (Shape::Poly { edges: ea, .. }, Shape::Poly { edges: eb, .. }) => {
            for x in ea {
                for y in eb {
                    axes.push(x.cross(*y));
                }
            }
        }
        (Shape::Sphere { c: ca, .. }, Shape::Sphere { c: cb, .. }) => axes.push(*cb - *ca),
        (Shape::Sphere { c, .. }, Shape::Poly { verts, .. })
        | (Shape::Poly { verts, .. }, Shape::Sphere { c, .. }) => {
            for v in verts {
                axes.push(*v - *c);
            }
        }
    }
    let mut best = f64::NEG_INFINITY;
    let mut any = false;
    for ax in axes {
        let l = match math::unit(ax, EPS_LEN) {
            Some(u) => u,
            None => continue,
        };
        let (a0, a1) = a.proj(l);
        let (b0, b1) = b.proj(l);
        let g = (b0 - a1).max(a0 - b1);
        if g > best {
            best = g;
        }
        any = true;
    }
    if !any {
        if let (Shape::Sphere { r: ra, .. }, Shape::Sphere { r: rb, .. }) = (a, b) {
            return -(ra + rb);
        }
        return f64::NEG_INFINITY;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_gap() {
        let a = box_shape(&Aabb::new(Vec3::ZERO, Vec3::ONE));
        let b = box_shape(&Aabb::new(Vec3::new(1.5, 0.0, 0.0), Vec3::new(2.5, 1.0, 1.0)));
        assert!((gap(&a, &b) - 0.5).abs() < 1e-12);
        if let Shape::Poly { normals, edges, .. } = &a {
            assert_eq!(normals.len(), 3);
            assert_eq!(edges.len(), 3);
        }
    }

    #[test]
    fn sphere_on_box() {
        let a = box_shape(&Aabb::new(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 1.0, 1.0)));
        let s = Shape::Sphere {
            c: Vec3::new(0.0, 1.5, 0.0),
            r: 0.5,
        };
        assert!(gap(&a, &s).abs() < 1e-12);
    }

    #[test]
    fn hull_tetra() {
        let pts = [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z];
        let h = hull_shape(&pts).unwrap();
        if let Shape::Poly { normals, .. } = h {
            assert_eq!(normals.len(), 4);
        }
    }
}
