//! 扫掠（spec/02 2.2.9 / 2.2.10）：路径点列（倒圆 / Catmull-Rom）、扫掠框架（2.3.7）、截面与环点。

use crate::math::{self, Vec3};
use crate::poly2;
use crate::solid::SpanPin;

/// ε_len（2.1.4）
pub const EPS_LEN: f64 = 1e-9;
/// ε_dir
pub const EPS_DIR: f64 = 1e-6;
/// ε_up
pub const EPS_UP: f64 = 1e-4;
/// n_circ：圆截面点数
pub const N_CIRC: usize = 20;

/// 截面
#[derive(Clone, Debug, PartialEq)]
pub enum Profile {
    Circle { r: f64 },
    Rect { w: f64, t: f64, pin: SpanPin },
    Polygon(Vec<[f64; 2]>),
}

/// 构造截面点列 Π（正向）。Err 给出不含路径名的文案；Ok 的 bool 表示规整时改动过（警告）。
pub fn profile_ring(p: &Profile) -> Result<(Vec<[f64; 2]>, bool), &'static str> {
    match p {
        Profile::Circle { r } => {
            let mut out = Vec::with_capacity(N_CIRC);
            for k in 0..N_CIRC {
                let a = 18.0 * k as f64;
                out.push([r * math::cos_deg(a), r * math::sin_deg(a)]);
            }
            Ok((out, false))
        }
        Profile::Rect { w, t, pin } => {
            let dv = match pin {
                SpanPin::Bottom => t / 2.0,
                SpanPin::Top => -t / 2.0,
                SpanPin::Center => 0.0,
            };
            Ok((
                vec![
                    [w / 2.0, -t / 2.0 + dv],
                    [w / 2.0, t / 2.0 + dv],
                    [-w / 2.0, t / 2.0 + dv],
                    [-w / 2.0, -t / 2.0 + dv],
                ],
                false,
            ))
        }
        Profile::Polygon(pts) => {
            let (mut q, changed) = poly2::normalize(pts);
            if q.len() < 3 {
                return Err("profile 多边形写成 [(u,v); (u,v); …]，至少 3 个点");
            }
            let a2 = poly2::signed_area2(&q);
            if a2.abs() < poly2::EPS_AREA {
                return Err("截面面积是 0");
            }
            if !poly2::is_simple(&q) {
                return Err("截面自交");
            }
            if a2 < 0.0 {
                q.reverse();
            }
            Ok((q, changed))
        }
    }
}

/// 倒圆半径 ρ_c
pub fn fillet_radius(p: &Profile) -> f64 {
    match p {
        Profile::Circle { r } => (3.0 * r).max(0.6),
        Profile::Rect { w, .. } => w.max(1.0),
        Profile::Polygon(_) => 1.0,
    }
}

/// 倒圆路径：每个内点用二次 Bézier 的 9 个点替换
pub fn fillet_path(ctrl: &[Vec3], rho: f64) -> Vec<Vec3> {
    let n = ctrl.len();
    let mut out = Vec::with_capacity(n * 9);
    if n == 0 {
        return out;
    }
    out.push(ctrl[0]);
    for i in 1..n.saturating_sub(1) {
        let (a, b, c) = (ctrl[i - 1], ctrl[i], ctrl[i + 1]);
        let la = math::length(b - a);
        let lc = math::length(c - b);
        let delta = rho.min(0.45 * la).min(0.45 * lc);
        if delta < EPS_DIR {
            out.push(b);
            continue;
        }
        let p0 = b + (a - b) / la * delta;
        let p2 = b + (c - b) / lc * delta;
        for j in 0..=8 {
            let t = j as f64 / 8.0;
            let s = 1.0 - t;
            out.push(p0 * (s * s) + b * (2.0 * s * t) + p2 * (t * t));
        }
    }
    if n > 1 {
        out.push(ctrl[n - 1]);
    }
    out
}

/// 均匀 Catmull-Rom（2.1.6.5），每段 m 个样本
pub fn catmull_rom(ctrl: &[Vec3], closed: bool, m: usize) -> Vec<Vec3> {
    let n = ctrl.len();
    if !closed && n == 2 {
        return ctrl.to_vec();
    }
    let cr = |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f64| -> Vec3 {
        (p1 * 2.0
            + (p2 - p0) * t
            + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * (t * t)
            + (-p0 + p1 * 3.0 - p2 * 3.0 + p3) * (t * t * t))
            * 0.5
    };
    let mut pts = Vec::new();
    if closed {
        for i in 0..n {
            let (p0, p1, p2, p3) = (
                ctrl[(i + n - 1) % n],
                ctrl[i],
                ctrl[(i + 1) % n],
                ctrl[(i + 2) % n],
            );
            for j in 0..m {
                pts.push(cr(p0, p1, p2, p3, j as f64 / m as f64));
            }
        }
    } else {
        for i in 0..n - 1 {
            let p0 = if i == 0 { ctrl[0] } else { ctrl[i - 1] };
            let p3 = if i + 2 >= n { ctrl[n - 1] } else { ctrl[i + 2] };
            let (p1, p2) = (ctrl[i], ctrl[i + 1]);
            for j in 0..m {
                pts.push(cr(p0, p1, p2, p3, j as f64 / m as f64));
            }
        }
        pts.push(ctrl[n - 1]);
    }
    pts
}

/// 扫掠框架（2.3.7）：每个点的 (T, side, up)。Err(i) 表示路径在第 i 个点退化。
pub fn frames(
    pts: &[Vec3],
    closed: bool,
    tangents: Option<&[Vec3]>,
) -> Result<Vec<(Vec3, Vec3, Vec3)>, usize> {
    let n = pts.len();
    let mut out = Vec::with_capacity(n);
    let mut side_prev = Vec3::Z;
    for i in 0..n {
        let t = match tangents {
            Some(ts) => ts[i],
            None => {
                let d = if closed {
                    pts[(i + 1) % n] - pts[(i + n - 1) % n]
                } else if i == 0 {
                    pts[1] - pts[0]
                } else if i == n - 1 {
                    pts[n - 1] - pts[n - 2]
                } else {
                    pts[i + 1] - pts[i - 1]
                };
                match math::unit(d, EPS_LEN) {
                    Some(u) => u,
                    None => return Err(i),
                }
            }
        };
        let sigma = Vec3::new(-t.z, 0.0, t.x);
        let side = if math::length(sigma) < EPS_UP {
            side_prev
        } else {
            math::unit(sigma, 0.0).unwrap()
        };
        let up = math::unit(side.cross(t), 0.0).unwrap_or(Vec3::Y);
        side_prev = side;
        out.push((t, side, up));
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SweepErr {
    /// 相邻点重合
    Coincident,
    /// 路径在第 i 个点退化
    Degenerate(usize),
}

/// 扫掠框架 (p, side, up)
pub type Frame = (Vec3, Vec3, Vec3);

/// 环点 ρ_{i,k}（按 i 外层、k 内层排列）与框架 (p, side, up)
pub fn rings(
    pts: &[Vec3],
    closed: bool,
    tangents: Option<&[Vec3]>,
    ring: &[[f64; 2]],
) -> Result<(Vec<Vec3>, Vec<Frame>), SweepErr> {
    let n = pts.len();
    let segs = if closed { n } else { n.saturating_sub(1) };
    for i in 0..segs {
        if math::length(pts[(i + 1) % n] - pts[i]) < EPS_LEN {
            return Err(SweepErr::Coincident);
        }
    }
    let fr = frames(pts, closed, tangents).map_err(SweepErr::Degenerate)?;
    let mut out = Vec::with_capacity(n * ring.len());
    let mut frames_out = Vec::with_capacity(n);
    for (i, (_, side, up)) in fr.iter().enumerate() {
        for q in ring {
            out.push(pts[i] + *side * q[0] + *up * q[1]);
        }
        frames_out.push((pts[i], *side, *up));
    }
    Ok((out, frames_out))
}

/// revolve 的轮廓处理（2.2.7 第 1–6 步）：|r|、可选平滑、截断、自动封口、规整、面积与简单性检查。
/// 返回 (Q, 规整时是否改动过)。
pub fn revolve_profile(profile: &[[f64; 2]], smooth: bool) -> Result<(Vec<[f64; 2]>, bool), &'static str> {
    const EPS_CAP: f64 = 1e-6;
    let mut q: Vec<[f64; 2]> = profile.iter().map(|p| [p[0].abs(), p[1]]).collect();
    if smooth {
        let ctrl: Vec<Vec3> = q.iter().map(|p| Vec3::new(p[0], p[1], 0.0)).collect();
        q = catmull_rom(&ctrl, false, 10).iter().map(|v| [v.x, v.y]).collect();
    }
    for p in q.iter_mut() {
        p[0] = p[0].max(0.0);
    }
    if q[0][0] > EPS_CAP {
        q.insert(0, [0.0, q[0][1]]);
    }
    let last = q[q.len() - 1];
    if last[0] > EPS_CAP {
        q.push([0.0, last[1]]);
    }
    let (q, changed) = poly2::normalize(&q);
    if poly2::signed_area2(&q).abs() < poly2::EPS_AREA {
        return Err("revolve 的轮廓面积是 0");
    }
    if !poly2::is_simple(&q) {
        return Err("revolve 的轮廓自交");
    }
    Ok((q, changed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fillet_keeps_ends() {
        let c = [Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 8.0)];
        let p = fillet_path(&c, 1.0);
        assert_eq!(p.len(), 11);
        assert_eq!(p[0], c[0]);
        assert_eq!(p[10], c[2]);
        assert!((p[1] - Vec3::new(9.0, 0.0, 0.0)).length() < 1e-12);
        assert!((p[9] - Vec3::new(10.0, 0.0, 1.0)).length() < 1e-12);
    }

    #[test]
    fn frames_horizontal() {
        let pts = [Vec3::ZERO, Vec3::X];
        let f = frames(&pts, false, None).unwrap();
        // side = T × ŷ = (−Tz, 0, Tx) = (0,0,1)；up = side × T = (0,1,0)
        assert!((f[0].1 - Vec3::Z).length() < 1e-12);
        assert!((f[0].2 - Vec3::Y).length() < 1e-12);
    }

    #[test]
    fn circle_ring() {
        let (r, _) = profile_ring(&Profile::Circle { r: 2.0 }).unwrap();
        assert_eq!(r.len(), 20);
        assert!((r[5][1] - 2.0).abs() < 1e-12);
    }
}
