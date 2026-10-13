//! 网格布尔（spec/02 2.5.3）：含三角化路径实体的布尔体按三角网格求值。
//! 算法：BSP 树上的多边形裁剪（csg.js 的做法）——确定性、只用加减乘除与开方；结果按 2.5.3 的语义等价要求
//! 围成同一个点集，三角形数量与切分方式不作规定。输入输出都是三角形列表（局部系，法线朝外）。

use crate::math::{self, Vec3};
use crate::tri::Tri;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Union,
    Subtract,
    Intersect,
}

#[derive(Clone, Copy)]
struct Plane {
    n: Vec3,
    w: f64,
}

#[derive(Clone)]
struct Polygon {
    verts: Vec<Vec3>,
    plane: Plane,
}

const COPLANAR: u8 = 0;
const FRONT: u8 = 1;
const BACK: u8 = 2;
const SPANNING: u8 = 3;

impl Plane {
    fn of(a: Vec3, b: Vec3, c: Vec3) -> Option<Plane> {
        let n = math::unit((b - a).cross(c - a), 0.0)?;
        Some(Plane { n, w: n.dot(a) })
    }
    fn flip(&mut self) {
        self.n = -self.n;
        self.w = -self.w;
    }
    fn split(
        &self,
        p: &Polygon,
        eps: f64,
        coplanar_front: &mut Vec<Polygon>,
        coplanar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        let mut ptype = 0u8;
        let mut types = Vec::with_capacity(p.verts.len());
        for v in &p.verts {
            let t = self.n.dot(*v) - self.w;
            let ty = if t < -eps {
                BACK
            } else if t > eps {
                FRONT
            } else {
                COPLANAR
            };
            ptype |= ty;
            types.push(ty);
        }
        match ptype {
            COPLANAR => {
                if self.n.dot(p.plane.n) > 0.0 {
                    coplanar_front.push(p.clone());
                } else {
                    coplanar_back.push(p.clone());
                }
            }
            FRONT => front.push(p.clone()),
            BACK => back.push(p.clone()),
            _ => {
                let mut f = Vec::new();
                let mut b = Vec::new();
                let m = p.verts.len();
                for i in 0..m {
                    let j = (i + 1) % m;
                    let (ti, tj) = (types[i], types[j]);
                    let (vi, vj) = (p.verts[i], p.verts[j]);
                    if ti != BACK {
                        f.push(vi);
                    }
                    if ti != FRONT {
                        b.push(vi);
                    }
                    if (ti | tj) == SPANNING {
                        let t = (self.w - self.n.dot(vi)) / self.n.dot(vj - vi);
                        let v = vi + (vj - vi) * t;
                        f.push(v);
                        b.push(v);
                    }
                }
                if f.len() >= 3 {
                    front.push(Polygon {
                        verts: f,
                        plane: p.plane,
                    });
                }
                if b.len() >= 3 {
                    back.push(Polygon {
                        verts: b,
                        plane: p.plane,
                    });
                }
            }
        }
    }
}

impl Polygon {
    fn flip(&mut self) {
        self.verts.reverse();
        self.plane.flip();
    }
}

#[derive(Default)]
struct Node {
    plane: Option<Plane>,
    front: Option<Box<Node>>,
    back: Option<Box<Node>>,
    polygons: Vec<Polygon>,
}

impl Node {
    fn new(polys: Vec<Polygon>, eps: f64) -> Node {
        let mut n = Node::default();
        n.build(polys, eps);
        n
    }
    fn invert(&mut self) {
        for p in &mut self.polygons {
            p.flip();
        }
        if let Some(pl) = &mut self.plane {
            pl.flip();
        }
        if let Some(f) = &mut self.front {
            f.invert();
        }
        if let Some(b) = &mut self.back {
            b.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }
    fn clip_polygons(&self, polys: Vec<Polygon>, eps: f64) -> Vec<Polygon> {
        let plane = match &self.plane {
            Some(p) => *p,
            None => return polys,
        };
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut cop_f = Vec::new();
        let mut cop_b = Vec::new();
        for p in &polys {
            plane.split(p, eps, &mut cop_f, &mut cop_b, &mut front, &mut back);
            front.append(&mut cop_f);
            back.append(&mut cop_b);
        }
        let front = match &self.front {
            Some(f) => f.clip_polygons(front, eps),
            None => front,
        };
        let back = match &self.back {
            Some(b) => b.clip_polygons(back, eps),
            None => Vec::new(),
        };
        let mut out = front;
        out.extend(back);
        out
    }
    fn clip_to(&mut self, bsp: &Node, eps: f64) {
        let polys = std::mem::take(&mut self.polygons);
        self.polygons = bsp.clip_polygons(polys, eps);
        if let Some(f) = &mut self.front {
            f.clip_to(bsp, eps);
        }
        if let Some(b) = &mut self.back {
            b.clip_to(bsp, eps);
        }
    }
    fn all_polygons(&self, out: &mut Vec<Polygon>) {
        out.extend(self.polygons.iter().cloned());
        if let Some(f) = &self.front {
            f.all_polygons(out);
        }
        if let Some(b) = &self.back {
            b.all_polygons(out);
        }
    }
    fn build(&mut self, polys: Vec<Polygon>, eps: f64) {
        if polys.is_empty() {
            return;
        }
        if self.plane.is_none() {
            self.plane = Some(polys[0].plane);
        }
        let plane = self.plane.unwrap();
        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut cop_f = Vec::new();
        let mut cop_b = Vec::new();
        for p in &polys {
            plane.split(p, eps, &mut cop_f, &mut cop_b, &mut front, &mut back);
            self.polygons.append(&mut cop_f);
            self.polygons.append(&mut cop_b);
        }
        if !front.is_empty() {
            let f = self.front.get_or_insert_with(|| Box::new(Node::default()));
            f.build(front, eps);
        }
        if !back.is_empty() {
            let b = self.back.get_or_insert_with(|| Box::new(Node::default()));
            b.build(back, eps);
        }
    }
}

fn to_polys(tris: &[Tri]) -> Vec<Polygon> {
    tris.iter()
        .filter_map(|t| {
            Plane::of(t[0], t[1], t[2]).map(|plane| Polygon {
                verts: t.to_vec(),
                plane,
            })
        })
        .collect()
}

fn to_tris(polys: &[Polygon]) -> Vec<Tri> {
    let mut out = Vec::new();
    for p in polys {
        for i in 1..p.verts.len().saturating_sub(1) {
            out.push([p.verts[0], p.verts[i], p.verts[i + 1]]);
        }
    }
    out
}

/// 平面判定的容差：两组网格包围盒最大边长的 1e-9（下限 1e-12）
fn eps_of(a: &[Tri], b: &[Tri]) -> f64 {
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for t in a.iter().chain(b.iter()) {
        for p in t {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
    }
    let e = (hi - lo).max_element();
    if e.is_finite() {
        (1e-9 * e).max(1e-12)
    } else {
        1e-12
    }
}

/// 两个闭合三角网格的布尔运算（csg.js 的 union / subtract / intersect）
pub fn boolean(a: &[Tri], b: &[Tri], op: Op) -> Vec<Tri> {
    if a.is_empty() {
        return match op {
            Op::Union => b.to_vec(),
            _ => Vec::new(),
        };
    }
    if b.is_empty() {
        return match op {
            Op::Intersect => Vec::new(),
            _ => a.to_vec(),
        };
    }
    let eps = eps_of(a, b);
    let mut na = Node::new(to_polys(a), eps);
    let mut nb = Node::new(to_polys(b), eps);
    match op {
        Op::Union => {
            na.clip_to(&nb, eps);
            nb.clip_to(&na, eps);
            nb.invert();
            nb.clip_to(&na, eps);
            nb.invert();
            let mut polys = Vec::new();
            nb.all_polygons(&mut polys);
            na.build(polys, eps);
        }
        Op::Subtract => {
            na.invert();
            na.clip_to(&nb, eps);
            nb.clip_to(&na, eps);
            nb.invert();
            nb.clip_to(&na, eps);
            nb.invert();
            let mut polys = Vec::new();
            nb.all_polygons(&mut polys);
            na.build(polys, eps);
            na.invert();
        }
        Op::Intersect => {
            na.invert();
            nb.clip_to(&na, eps);
            nb.invert();
            na.clip_to(&nb, eps);
            nb.clip_to(&na, eps);
            let mut polys = Vec::new();
            nb.all_polygons(&mut polys);
            na.build(polys, eps);
            na.invert();
        }
    }
    let mut polys = Vec::new();
    na.all_polygons(&mut polys);
    to_tris(&polys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solid::Solid;
    use crate::transform::Transform;
    use crate::tri::{transform_tris, triangulate};

    fn vol(tris: &[Tri]) -> f64 {
        tris.iter().map(|t| t[0].dot(t[1].cross(t[2])) / 6.0).sum()
    }

    #[test]
    fn box_minus_box() {
        let a = triangulate(&Solid::Box {
            size: Vec3::new(2.0, 2.0, 2.0),
        });
        let b = transform_tris(
            &triangulate(&Solid::Box {
                size: Vec3::new(1.0, 1.0, 1.0),
            }),
            &Transform::translation(Vec3::new(0.0, 0.5, 0.0)),
        );
        let u = boolean(&a, &b, Op::Union);
        assert!((vol(&u) - 8.0).abs() < 1e-9);
        let s = boolean(&a, &b, Op::Subtract);
        assert!((vol(&s) - 7.0).abs() < 1e-9);
        let i = boolean(&a, &b, Op::Intersect);
        assert!((vol(&i) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn overlapping_boxes() {
        let a = triangulate(&Solid::Box {
            size: Vec3::new(2.0, 1.0, 1.0),
        });
        let b = transform_tris(&a, &Transform::translation(Vec3::new(1.0, 0.0, 0.0)));
        assert!((vol(&boolean(&a, &b, Op::Union)) - 3.0).abs() < 1e-9);
        assert!((vol(&boolean(&a, &b, Op::Subtract)) - 1.0).abs() < 1e-9);
        assert!((vol(&boolean(&a, &b, Op::Intersect)) - 1.0).abs() < 1e-9);
    }
}
