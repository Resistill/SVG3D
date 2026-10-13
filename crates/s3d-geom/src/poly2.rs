//! 二维多边形（spec/02 2.1.6.2）：带符号面积、规整、简单性、凸性、耳切三角化。

use crate::math::{self, Vec2};

/// ε_area（2.1.4）
pub const EPS_AREA: f64 = 1e-12;
/// ε_len（2.1.4）
pub const EPS_LEN: f64 = 1e-9;

fn v(p: [f64; 2]) -> Vec2 {
    Vec2::new(p[0], p[1])
}

fn cross(a: Vec2, b: Vec2) -> f64 {
    a.x * b.y - a.y * b.x
}

/// 带符号面积的两倍 2A = Σ (u_i v_{i+1} − u_{i+1} v_i)
pub fn signed_area2(pts: &[[f64; 2]]) -> f64 {
    let m = pts.len();
    let mut s = 0.0;
    for i in 0..m {
        let a = pts[i];
        let b = pts[(i + 1) % m];
        s += a[0] * b[1] - b[0] * a[1];
    }
    s
}

/// 规整：去掉与首点重合的末点、相邻重合点（都给警告）。返回 (点列, 是否改动过)
pub fn normalize(pts: &[[f64; 2]]) -> (Vec<[f64; 2]>, bool) {
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(pts.len());
    let mut changed = false;
    for p in pts {
        if let Some(last) = out.last() {
            if math::length2(v(*p) - v(*last)) < EPS_LEN {
                changed = true;
                continue;
            }
        }
        out.push(*p);
    }
    while out.len() > 1 && math::length2(v(out[out.len() - 1]) - v(out[0])) < EPS_LEN {
        out.pop();
        changed = true;
    }
    (out, changed)
}

/// 线段 (a,b) 与 (c,d) 是否相交（含端点相碰；共线时按区间重叠）
fn segments_intersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let sgn = |x: f64| -> i32 {
        if x.abs() < EPS_AREA {
            0
        } else if x > 0.0 {
            1
        } else {
            -1
        }
    };
    let d1 = sgn(cross(b - a, c - a));
    let d2 = sgn(cross(b - a, d - a));
    let d3 = sgn(cross(d - c, a - c));
    let d4 = sgn(cross(d - c, b - c));
    if d1 * d2 < 0 && d3 * d4 < 0 {
        return true;
    }
    // 共线 / 端点落在另一条线段上
    let on = |p: Vec2, q: Vec2, r: Vec2| -> bool {
        r.x >= p.x.min(q.x) && r.x <= p.x.max(q.x) && r.y >= p.y.min(q.y) && r.y <= p.y.max(q.y)
    };
    (d1 == 0 && on(a, b, c))
        || (d2 == 0 && on(a, b, d))
        || (d3 == 0 && on(c, d, a))
        || (d4 == 0 && on(c, d, b))
}

/// 简单多边形判定：不相邻的边不相交
pub fn is_simple(pts: &[[f64; 2]]) -> bool {
    let m = pts.len();
    if m < 3 {
        return false;
    }
    for i in 0..m {
        for j in (i + 1)..m {
            // 相邻边（含首尾）跳过
            if j == i + 1 || (i == 0 && j == m - 1) {
                continue;
            }
            let (a, b) = (v(pts[i]), v(pts[(i + 1) % m]));
            let (c, d) = (v(pts[j]), v(pts[(j + 1) % m]));
            if segments_intersect(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

/// 凸性：相邻边叉积同号（容差 ε_area）
pub fn is_convex(pts: &[[f64; 2]]) -> bool {
    let m = pts.len();
    let mut pos = false;
    let mut neg = false;
    for i in 0..m {
        let a = v(pts[i]);
        let b = v(pts[(i + 1) % m]);
        let c = v(pts[(i + 2) % m]);
        let x = cross(b - a, c - b);
        if x > EPS_AREA {
            pos = true;
        } else if x < -EPS_AREA {
            neg = true;
        }
    }
    !(pos && neg)
}

/// 点是否在三角形内或边界上（三个叉积都 ≥ −ε_area；三角形须为正向）
fn in_tri_or_edge(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    cross(b - a, p - a) >= -EPS_AREA && cross(c - b, p - b) >= -EPS_AREA && cross(a - c, p - c) >= -EPS_AREA
}

/// 耳切三角化（2.1.6.2）。输入须已规整；本函数把反向序列反转成正向后再切。
/// 返回的三角形是**输入序列的下标**三元组（按正向后的顶点序 (q_{i-1}, q_i, q_{i+1})）。
/// 失败（数值退化、找不到耳）返回 None。
pub fn earcut(pts: &[[f64; 2]]) -> Option<Vec<[usize; 3]>> {
    let m = pts.len();
    if m < 3 {
        return None;
    }
    let mut idx: Vec<usize> = (0..m).collect();
    if signed_area2(pts) < 0.0 {
        idx.reverse();
    }
    // 去掉直顶点
    loop {
        let k = idx.len();
        if k < 3 {
            return None;
        }
        let mut removed = false;
        for i in 0..k {
            let a = v(pts[idx[(i + k - 1) % k]]);
            let b = v(pts[idx[i]]);
            let c = v(pts[idx[(i + 1) % k]]);
            if cross(b - a, c - b).abs() < EPS_AREA {
                idx.remove(i);
                removed = true;
                break;
            }
        }
        if !removed {
            break;
        }
    }
    let mut out = Vec::with_capacity(idx.len() - 2);
    while idx.len() > 3 {
        let k = idx.len();
        let mut ear: Option<usize> = None;
        let mut first_convex: Option<usize> = None;
        for i in 0..k {
            let ia = idx[(i + k - 1) % k];
            let ib = idx[i];
            let ic = idx[(i + 1) % k];
            let (a, b, c) = (v(pts[ia]), v(pts[ib]), v(pts[ic]));
            if cross(b - a, c - a) <= 0.0 {
                continue;
            }
            if first_convex.is_none() {
                first_convex = Some(i);
            }
            let mut ok = true;
            for (j, &iq) in idx.iter().enumerate() {
                if j == (i + k - 1) % k || j == i || j == (i + 1) % k {
                    continue;
                }
                if in_tri_or_edge(v(pts[iq]), a, b, c) {
                    ok = false;
                    break;
                }
            }
            if ok {
                ear = Some(i);
                break;
            }
        }
        let i = ear.or(first_convex)?;
        let k = idx.len();
        out.push([idx[(i + k - 1) % k], idx[i], idx[(i + 1) % k]]);
        idx.remove(i);
    }
    out.push([idx[0], idx[1], idx[2]]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square() {
        let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!((signed_area2(&sq) - 2.0).abs() < 1e-12);
        assert!(is_simple(&sq));
        assert!(is_convex(&sq));
        let tris = earcut(&sq).unwrap();
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn bowtie_not_simple() {
        let bt = [[0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [0.0, 1.0]];
        assert!(!is_simple(&bt));
    }

    #[test]
    fn concave_earcut() {
        let c = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [1.0, 0.5], [0.0, 2.0]];
        assert!(!is_convex(&c));
        let tris = earcut(&c).unwrap();
        assert_eq!(tris.len(), 3);
    }
}
