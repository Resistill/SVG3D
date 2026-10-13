//! 三角形集合的静态 BVH：点到表面的最近点（带封顶）与射线穿越计数（评分的残差与体积误差用）。
//! 建树按质心包围盒最长轴的中位数切分，叶子最多 LEAF 个三角形；遍历顺序只取决于输入顺序与坐标，
//! 单线程、不用 HashMap，结果逐位确定。

use crate::math::{self, Vec3};
use crate::tri::Tri;

const LEAF: usize = 4;

#[derive(Clone, Copy)]
struct Node {
    lo: Vec3,
    hi: Vec3,
    /// 内部节点：左孩子序号（右孩子 = left + 1 的兄弟存在 right）；叶子：三角形起点
    first: u32,
    /// 叶子：三角形数（> 0）；内部节点：0
    count: u32,
    right: u32,
}

/// 三角形集合的 BVH
pub struct Bvh {
    nodes: Vec<Node>,
    /// 按树序重排后的三角形
    tris: Vec<Tri>,
    /// 重排后第 i 个三角形在输入里的序号
    order: Vec<u32>,
}

fn box_of(tris: &[Tri], idx: &[u32]) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for &i in idx {
        for p in &tris[i as usize] {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
    }
    (lo, hi)
}

fn centroid(t: &Tri) -> Vec3 {
    (t[0] + t[1] + t[2]) / 3.0
}

impl Bvh {
    pub fn new(tris: &[Tri]) -> Bvh {
        let mut b = Bvh {
            nodes: Vec::new(),
            tris: Vec::with_capacity(tris.len()),
            order: Vec::with_capacity(tris.len()),
        };
        if tris.is_empty() {
            return b;
        }
        let mut idx: Vec<u32> = (0..tris.len() as u32).collect();
        b.build(tris, &mut idx);
        b
    }

    pub fn is_empty(&self) -> bool {
        self.tris.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tris.len()
    }

    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        self.nodes.first().map(|n| (n.lo, n.hi))
    }

    fn build(&mut self, tris: &[Tri], idx: &mut [u32]) -> u32 {
        let (lo, hi) = box_of(tris, idx);
        let me = self.nodes.len() as u32;
        self.nodes.push(Node {
            lo,
            hi,
            first: 0,
            count: 0,
            right: 0,
        });
        if idx.len() <= LEAF {
            let first = self.tris.len() as u32;
            for &i in idx.iter() {
                self.tris.push(tris[i as usize]);
                self.order.push(i);
            }
            self.nodes[me as usize].first = first;
            self.nodes[me as usize].count = idx.len() as u32;
            return me;
        }
        // 质心包围盒最长轴
        let mut clo = Vec3::splat(f64::INFINITY);
        let mut chi = Vec3::splat(f64::NEG_INFINITY);
        for &i in idx.iter() {
            let c = centroid(&tris[i as usize]);
            clo = clo.min(c);
            chi = chi.max(c);
        }
        let ext = chi - clo;
        let axis = if ext.x >= ext.y && ext.x >= ext.z {
            0
        } else if ext.y >= ext.z {
            1
        } else {
            2
        };
        // 按质心坐标排序（并列按输入序号），取中位数切分；全部重合时也能对半分
        idx.sort_by(|&a, &b| {
            let ca = centroid(&tris[a as usize])[axis];
            let cb = centroid(&tris[b as usize])[axis];
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b))
        });
        let mid = idx.len() / 2;
        let (l, r) = idx.split_at_mut(mid);
        let left = self.build(tris, l);
        let right = self.build(tris, r);
        self.nodes[me as usize].first = left;
        self.nodes[me as usize].right = right;
        me
    }

    /// 最近点查询：返回 (输入三角形序号, 距离, 最近点)；封顶 max 之内没有三角形时 None。
    /// 遍历时先看离点近的孩子；并列距离取先遍历到的三角形。
    pub fn closest(&self, p: Vec3, max: f64) -> Option<(usize, f64, Vec3)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best = max * max;
        let mut best_tri: Option<(usize, Vec3)> = None;
        let mut stack: Vec<(u32, f64)> = vec![(0, -1.0)];
        while let Some((ni, s)) = stack.pop() {
            if s >= 0.0 && !(s < best) {
                continue;
            }
            let n = self.nodes[ni as usize];
            if n.count > 0 {
                for i in n.first..n.first + n.count {
                    let c = closest_on_tri(p, &self.tris[i as usize]);
                    let d = (p - c).length_squared();
                    if d < best {
                        best = d;
                        best_tri = Some((i as usize, c));
                    }
                }
                continue;
            }
            let (c1, c2) = (n.first, n.right);
            let s1 = box_dist2(&self.nodes[c1 as usize], p);
            let s2 = box_dist2(&self.nodes[c2 as usize], p);
            // 先压远的，后压近的（近的先出栈）
            if s2 < s1 {
                stack.push((c1, s1));
                stack.push((c2, s2));
            } else {
                stack.push((c2, s2));
                stack.push((c1, s1));
            }
        }
        best_tri.map(|(i, c)| (self.order[i] as usize, math::sqrt(best), c))
    }

    /// 射线穿越：打到背面（方向与法线同向，从封闭网格里出去）+1，正面 −1；和 > 0 表示起点在某个封闭网格里
    pub fn crossings(&self, o: Vec3, d: Vec3) -> i32 {
        if self.nodes.is_empty() {
            return 0;
        }
        let inv = Vec3::new(1.0 / d.x, 1.0 / d.y, 1.0 / d.z);
        let mut sum = 0;
        let mut stack: Vec<u32> = vec![0];
        while let Some(ni) = stack.pop() {
            let n = self.nodes[ni as usize];
            if !ray_hits_box(&n, o, inv) {
                continue;
            }
            if n.count > 0 {
                for i in n.first..n.first + n.count {
                    sum += ray_tri_sign(o, d, &self.tris[i as usize]);
                }
                continue;
            }
            stack.push(n.right);
            stack.push(n.first);
        }
        sum
    }
}

fn box_dist2(n: &Node, p: Vec3) -> f64 {
    let q = p.clamp(n.lo, n.hi) - p;
    q.length_squared()
}

fn ray_hits_box(n: &Node, o: Vec3, inv: Vec3) -> bool {
    let mut t0 = 0.0f64;
    let mut t1 = f64::INFINITY;
    for k in 0..3 {
        let mut a = (n.lo[k] - o[k]) * inv[k];
        let mut b = (n.hi[k] - o[k]) * inv[k];
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        t0 = t0.max(a);
        t1 = t1.min(b);
    }
    t0 <= t1
}

/// Möller–Trumbore：命中返回 ±1（det < 0 为背面），否则 0
fn ray_tri_sign(o: Vec3, d: Vec3, t: &Tri) -> i32 {
    let e1 = t[1] - t[0];
    let e2 = t[2] - t[0];
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return 0;
    }
    let inv = 1.0 / det;
    let s = o - t[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return 0;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return 0;
    }
    if e2.dot(q) * inv > 1e-9 {
        if det < 0.0 {
            1
        } else {
            -1
        }
    } else {
        0
    }
}

/// 点到三角形的最近点（Ericson，Voronoi 区域法）
pub fn closest_on_tri(p: Vec3, t: &Tri) -> Vec3 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return a + ab * v;
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return a + ac * w;
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * w;
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a + ab * v + ac * w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solid::Solid;
    use crate::tri::triangulate;

    #[test]
    fn closest_matches_brute_force() {
        let s = Solid::Sphere { r: 1.0 };
        let tris = triangulate(&s);
        let bvh = Bvh::new(&tris);
        let mut seed = 12345u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f64 / (1u32 << 24) as f64
        };
        for _ in 0..200 {
            let p = Vec3::new(rnd() * 4.0 - 2.0, rnd() * 4.0 - 1.0, rnd() * 4.0 - 2.0);
            let brute = tris
                .iter()
                .map(|t| math::length(p - closest_on_tri(p, t)))
                .fold(f64::INFINITY, f64::min);
            let (_, d, _) = bvh.closest(p, f64::INFINITY).unwrap();
            assert!((d - brute).abs() < 1e-12, "{} vs {}", d, brute);
            assert_eq!(bvh.closest(p, brute * 0.5).is_some(), false);
        }
    }

    #[test]
    fn crossings_inside_outside() {
        let tris = triangulate(&Solid::Box {
            size: Vec3::new(2.0, 2.0, 2.0),
        });
        let bvh = Bvh::new(&tris);
        let d = Vec3::new(1.0, 1.3e-4, 2.7e-4);
        assert_eq!(bvh.crossings(Vec3::new(0.1, 1.0, 0.2), d), 1);
        assert_eq!(bvh.crossings(Vec3::new(5.0, 1.0, 0.2), d), 0);
        assert_eq!(bvh.crossings(Vec3::new(-5.0, 1.0, 0.2), d), 0);
    }
}
