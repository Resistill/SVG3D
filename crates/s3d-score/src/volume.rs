//! 体积误差（原型 `tools/volume.mjs`）：重建物体的实体里、源模型的实体外面的部分。
//! 在重建物体（封闭网格）的内部撒点；离源表面不到 δ 的点不计，在源模型实体里的点不计，其余按
//! log2(1 + (d − δ)/δ) 计价。在不在源模型实体里：沿 6 个轴向打射线，至少 5 条"穿出得多"。

use crate::sample::Rng;
use crate::surfaces::Group;
use s3d_geom::math::{self, Vec3};
use s3d_geom::tri::{mesh_stats, Tri};

const TILT: [f64; 3] = [1.0, 1.3e-4, 2.7e-4];

/// 点在不在封闭网格里：沿（稍歪的）+x 方向的射线与三角形相交的次数是奇数
pub fn inside_mesh(tris: &[Tri], p: Vec3) -> bool {
    let d = Vec3::new(TILT[0], TILT[1], TILT[2]);
    let mut n = 0;
    for t in tris {
        let e1 = t[1] - t[0];
        let e2 = t[2] - t[0];
        let pv = d.cross(e2);
        let det = e1.dot(pv);
        if det.abs() < 1e-12 {
            continue;
        }
        let inv = 1.0 / det;
        let s = p - t[0];
        let u = s.dot(pv) * inv;
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = s.cross(e1);
        let v = d.dot(q) * inv;
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        if e2.dot(q) * inv > 0.0 {
            n += 1;
        }
    }
    n % 2 == 1
}

/// 一个重建物体的内部采样：(体积, 点)；薄的（体积 / 表面积 < δ）、不封闭的返回 None。
/// 包围盒里均匀撒点、留下在网格里的，最多 3n 个、最多撒 8n 次
pub fn interior_samples(tris: &[Tri], delta: f64, seed: u32, n: usize) -> Option<(f64, Vec<Vec3>)> {
    if tris.len() < 4 {
        return None;
    }
    let (vol, area, b) = mesh_stats(tris);
    if vol <= 0.0 || vol.is_nan() || vol / area < delta {
        return None;
    }
    let mut r = Rng::new(seed);
    let mut out = Vec::new();
    let mut m = 0;
    while m < 8 * n && out.len() < 3 * n {
        let x = b.min.x + r.gen() * (b.max.x - b.min.x);
        let y = b.min.y + r.gen() * (b.max.y - b.min.y);
        let z = b.min.z + r.gen() * (b.max.z - b.min.z);
        let p = Vec3::new(x, y, z);
        if inside_mesh(tris, p) {
            out.push(p);
        }
        m += 1;
    }
    if out.is_empty() {
        None
    } else {
        Some((vol, out))
    }
}

/// 在不在源模型的实体里：6 条射线里至少 5 条穿出得多
pub fn inside_source(q: &Group, p: Vec3) -> bool {
    let rays = [
        Vec3::new(1.0, TILT[1], TILT[2]),
        Vec3::new(-1.0, TILT[1], TILT[2]),
        Vec3::new(TILT[1], 1.0, TILT[2]),
        Vec3::new(TILT[1], -1.0, TILT[2]),
        Vec3::new(TILT[1], TILT[2], 1.0),
        Vec3::new(TILT[1], TILT[2], -1.0),
    ];
    let mut other = 0;
    for d in rays {
        if q.crossings(p, d) <= 0 {
            other += 1;
        }
        if other > 1 {
            return false;
        }
    }
    true
}

/// 一组内部点的平均代价 r 与错填比例 wrong（k 已换算成场景单位）
pub fn volume_stats(pts: &[Vec3], q: Option<&Group>, delta: f64, d_max: f64) -> (f64, f64) {
    let n = pts.len();
    let q = match q {
        Some(q) if n > 0 => q,
        _ => return (0.0, 0.0),
    };
    let mut sum = 0.0;
    let mut bad = 0usize;
    for p in pts {
        let d = match q.closest(*p, d_max) {
            Some((d, _)) => d.min(d_max),
            None => d_max,
        };
        if d <= delta || inside_source(q, *p) {
            continue;
        }
        sum += math::log2(1.0 + (d - delta) / delta);
        bad += 1;
    }
    (sum / n as f64, bad as f64 / n as f64)
}
