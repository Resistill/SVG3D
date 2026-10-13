//! 曲线（spec/02 2.3）：采样点列、参数化与取点、扫掠框架、包围盒。

use crate::aabb::Aabb;
use crate::math::{self, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArcPlane {
    Xz,
    Xy,
    Zy,
}

impl ArcPlane {
    pub fn parse(s: &str) -> Option<ArcPlane> {
        match s {
            "xz" => Some(ArcPlane::Xz),
            "xy" => Some(ArcPlane::Xy),
            "zy" => Some(ArcPlane::Zy),
            _ => None,
        }
    }
}

/// 解析曲线（arc / helix）的参数：C_a(t) 与切线
#[derive(Clone, Debug, PartialEq)]
pub struct Analytic {
    pub r: f64,
    pub a0: f64,
    pub a1: f64,
    pub rise: f64,
    pub plane: ArcPlane,
}

impl Analytic {
    pub fn point(&self, t: f64) -> Vec3 {
        let th = math::rad(self.a0 + (self.a1 - self.a0) * t);
        let (c, s) = (math::cos(th), math::sin(th));
        let r = self.r;
        match self.plane {
            ArcPlane::Xz => Vec3::new(r * c, self.rise * t, -r * s),
            ArcPlane::Xy => Vec3::new(r * c, r * s, self.rise * t),
            ArcPlane::Zy => Vec3::new(self.rise * t, r * s, r * c),
        }
    }
    pub fn tangent(&self, t: f64) -> Vec3 {
        let th = math::rad(self.a0 + (self.a1 - self.a0) * t);
        let dp = math::rad(self.a1 - self.a0);
        let (c, s) = (math::cos(th), math::sin(th));
        let r = self.r;
        let d = match self.plane {
            ArcPlane::Xz => Vec3::new(-r * dp * s, self.rise, -r * dp * c),
            ArcPlane::Xy => Vec3::new(-r * dp * s, r * dp * c, self.rise),
            ArcPlane::Zy => Vec3::new(self.rise, r * dp * c, -r * dp * s),
        };
        math::unit(d, 0.0).unwrap_or(Vec3::X)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub pts: Vec<Vec3>,
    pub closed: bool,
    pub analytic: Option<Analytic>,
    /// 弦长累积 ℓ_k（seq 含闭合时追加的首点）
    pub cum: Vec<f64>,
}

impl Curve {
    pub fn new(pts: Vec<Vec3>, closed: bool, analytic: Option<Analytic>) -> Curve {
        let mut cum = vec![0.0];
        let n = pts.len();
        let segs = if closed { n } else { n.saturating_sub(1) };
        for k in 0..segs {
            let a = pts[k];
            let b = pts[(k + 1) % n];
            let last = *cum.last().unwrap();
            cum.push(last + math::length(b - a));
        }
        Curve {
            pts,
            closed,
            analytic,
            cum,
        }
    }

    /// 总弦长 Λ
    pub fn length(&self) -> f64 {
        *self.cum.last().unwrap_or(&0.0)
    }

    pub fn local_box(&self) -> Aabb {
        Aabb::of_points(self.pts.iter().copied()).unwrap_or(Aabb::ZERO)
    }

    /// 取点 C[t]（2.3.1）：返回 (点, 单位切线)
    pub fn point_at(&self, t: f64) -> (Vec3, Vec3) {
        let t = if self.closed {
            t - t.floor()
        } else {
            t.clamp(0.0, 1.0)
        };
        if let Some(a) = &self.analytic {
            return (a.point(t), a.tangent(t));
        }
        let n = self.pts.len();
        let segs = if self.closed { n } else { n - 1 };
        let seq = |i: usize| self.pts[i % n];
        let lam = self.length();
        let lstar = t * lam;
        let mut k = 0;
        for kk in 0..segs {
            if self.cum[kk] <= lstar {
                k = kk;
            }
        }
        let k = k.min(segs - 1);
        let seg_len = self.cum[k + 1] - self.cum[k];
        let l = if seg_len > 0.0 {
            (lstar - self.cum[k]) / seg_len
        } else {
            0.0
        };
        let p = seq(k) + (seq(k + 1) - seq(k)) * l;
        let mut tan = math::unit(seq(k + 1) - seq(k), 1e-9);
        if tan.is_none() {
            // 零长段：取后面第一个非零段的方向，没有则前面的
            for kk in (k + 1)..segs {
                if let Some(u) = math::unit(seq(kk + 1) - seq(kk), 1e-9) {
                    tan = Some(u);
                    break;
                }
            }
            if tan.is_none() {
                for kk in (0..k).rev() {
                    if let Some(u) = math::unit(seq(kk + 1) - seq(kk), 1e-9) {
                        tan = Some(u);
                        break;
                    }
                }
            }
        }
        (p, tan.unwrap_or(Vec3::X))
    }

    /// 解析曲线（arc / helix）在每个采样点的单位切线 R̂T(t_i)；其他曲线没有
    pub fn analytic_tangents(&self) -> Option<Vec<Vec3>> {
        let a = self.analytic.as_ref()?;
        let n = self.pts.len();
        Some(
            (0..n)
                .map(|i| {
                    let ti = if self.closed {
                        i as f64 / n as f64
                    } else {
                        i as f64 / (n as f64 - 1.0)
                    };
                    a.tangent(ti)
                })
                .collect(),
        )
    }

    /// 2.3.7 扫掠框架：每个点的 (切线, side, up)
    pub fn frames(&self) -> Result<Vec<(Vec3, Vec3, Vec3)>, usize> {
        let ts = self.analytic_tangents();
        crate::sweep::frames(&self.pts, self.closed, ts.as_deref())
    }
}

/// arc（2.3.2）：返回 Err(文案) 表示参数非法
pub fn arc(r: f64, a0: f64, a1: f64, rise: f64, plane: ArcPlane) -> Result<Curve, &'static str> {
    if r <= 0.0 {
        return Err("r 要大于 0");
    }
    if a1 == a0 {
        return Err("弧的起止角相同");
    }
    let delta = a1 - a0;
    let ns = ((delta.abs() / 4.0).ceil() as usize).max(8);
    let closed = delta.abs() >= 360.0 - 1e-9 && rise == 0.0;
    let an = Analytic {
        r,
        a0,
        a1,
        rise,
        plane,
    };
    let count = if closed { ns } else { ns + 1 };
    let pts: Vec<Vec3> = (0..count).map(|i| an.point(i as f64 / ns as f64)).collect();
    Ok(Curve::new(pts, closed, Some(an)))
}

/// helix（2.3.3）
pub fn helix(r: f64, pitch: f64, turns: f64) -> Result<Curve, &'static str> {
    if r <= 0.0 {
        return Err("r 要大于 0");
    }
    if pitch == 0.0 {
        return Err("helix 的 pitch 不能是 0（平的圆用 arc）");
    }
    if turns == 0.0 {
        return Err("helix 的 turns 不能是 0");
    }
    let an = Analytic {
        r,
        a0: 0.0,
        a1: 360.0 * turns,
        rise: pitch * turns.abs(),
        plane: ArcPlane::Xz,
    };
    let ns = ((360.0 * turns.abs() / 4.0).ceil() as usize).max(8);
    let pts: Vec<Vec3> = (0..=ns).map(|i| an.point(i as f64 / ns as f64)).collect();
    Ok(Curve::new(pts, false, Some(an)))
}

/// line（2.3.4）
pub fn line(path: Vec<Vec3>) -> Curve {
    Curve::new(path, false, None)
}

/// bezier（2.3.5、2.1.6.5）：de Casteljau，65 个样本
pub fn bezier(ctrl: &[Vec3]) -> Curve {
    let m = 64;
    let mut pts = Vec::with_capacity(m + 1);
    for j in 0..=m {
        let t = j as f64 / m as f64;
        let mut work: Vec<Vec3> = ctrl.to_vec();
        while work.len() > 1 {
            for i in 0..work.len() - 1 {
                work[i] = work[i] + (work[i + 1] - work[i]) * t;
            }
            work.pop();
        }
        pts.push(work[0]);
    }
    Curve::new(pts, false, None)
}

/// spline（2.3.6、2.1.6.5）：均匀 Catmull-Rom，每段 12 个样本
pub fn spline(ctrl: &[Vec3], closed: bool) -> Curve {
    Curve::new(crate::sweep::catmull_rom(ctrl, closed, 12), closed, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_counts() {
        assert_eq!(arc(5.0, 0.0, 360.0, 0.0, ArcPlane::Xz).unwrap().pts.len(), 90);
        assert_eq!(arc(5.0, 0.0, 90.0, 0.0, ArcPlane::Xz).unwrap().pts.len(), 24);
        assert_eq!(arc(40.0, -60.0, 240.0, 0.0, ArcPlane::Xz).unwrap().pts.len(), 76);
        assert_eq!(helix(7.0, 4.5, 3.0).unwrap().pts.len(), 271);
        assert_eq!(spline(&[Vec3::ZERO, Vec3::X, Vec3::Y], true).pts.len(), 36);
        assert_eq!(spline(&[Vec3::ZERO, Vec3::X, Vec3::Y], false).pts.len(), 25);
        assert_eq!(bezier(&[Vec3::ZERO, Vec3::X]).pts.len(), 65);
    }

    #[test]
    fn lamps_example() {
        // 3.8.4 的例子：loop r=40 angle=-60,240，s = 0.3 → θ = 30°
        let c = arc(40.0, -60.0, 240.0, 0.0, ArcPlane::Xz).unwrap();
        let (p, tau) = c.point_at(0.3);
        assert!((p - Vec3::new(34.6410161513775, 0.0, -20.0)).length() < 1e-9);
        let psi = math::deg(math::atan2(-tau.z, tau.x));
        assert!((psi - 120.0).abs() < 1e-9);
    }
}
