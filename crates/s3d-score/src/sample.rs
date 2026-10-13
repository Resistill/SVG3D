//! 表面采样（原型 `tools/parts.mjs`）：确定性随机数、顶点去重与按面积的随机采样。
//! 采样点 = 去重后的三角形顶点 + n 个按面积加权的随机点；种子由调用方给。

use s3d_geom::math::{self, Vec3};
use s3d_geom::tri::Tri;
use std::collections::BTreeSet;

/// 原型 `rng(seed)`：32 位整数的混合器，与 JS 的 Math.imul / >>> 逐位相同
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Rng {
        Rng(if seed == 0 { 1 } else { seed })
    }
    /// [0, 1) 的双精度数
    pub fn gen(&mut self) -> f64 {
        let s = self.0;
        let a = (s ^ (s >> 15)).wrapping_mul(2246822507);
        let b = (s ^ (s >> 13)).wrapping_mul(3266489909);
        let c = s.wrapping_add(0x9e3779b9);
        self.0 = a ^ b ^ c;
        self.0 as f64 / 4294967296.0
    }
}

/// 路径的采样种子（FNV-1a，原型 `seedOf`：按 UTF-16 码元）
pub fn seed_of(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for u in s.encode_utf16() {
        h = (h ^ u as u32).wrapping_mul(16777619);
    }
    1000 + h % 1000003
}

/// JS `x.toFixed(4)` 的键：(是否负号, |x| × 1e4 四舍五入)。平局（|x| = 奇数/32）与 JS 同样进位
fn fixed4_key(x: f64) -> (bool, i64) {
    let a = x.abs() * 1e4;
    (x < 0.0, (a + 0.5).floor() as i64)
}

/// 三角形顶点去重：先按精确数值，再按 toFixed(4) 的键；保留第一次出现的
pub fn unique_vertices(tris: &[Tri]) -> Vec<Vec3> {
    let mut exact: BTreeSet<[u64; 3]> = BTreeSet::new();
    let mut seen: BTreeSet<[(bool, i64); 3]> = BTreeSet::new();
    let mut out = Vec::new();
    for t in tris {
        for v in t {
            let bits = [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()];
            if !exact.insert(bits) {
                continue;
            }
            let k = [fixed4_key(v.x), fixed4_key(v.y), fixed4_key(v.z)];
            if seen.insert(k) {
                out.push(*v);
            }
        }
    }
    out
}

pub fn tri_area(t: &Tri) -> f64 {
    let c = (t[1] - t[0]).cross(t[2] - t[0]);
    math::sqrt(c.x * c.x + c.y * c.y + c.z * c.z) / 2.0
}

/// 去重顶点 + n 个按面积加权的随机点（原型 `samplePoints`）
pub fn sample_points(tris: &[Tri], n: usize, seed: u32) -> Vec<Vec3> {
    let areas: Vec<f64> = tris.iter().map(tri_area).collect();
    let mut total = 0.0;
    let mut cum = Vec::with_capacity(areas.len());
    for a in &areas {
        total += a;
        cum.push(total);
    }
    if total == 0.0 {
        total = 1.0;
    }
    let mut pts = unique_vertices(tris);
    if tris.is_empty() {
        return pts;
    }
    let mut r = Rng::new(seed);
    for _ in 0..n {
        let x = r.gen() * total;
        let (mut lo, mut hi) = (0usize, cum.len() - 1);
        while lo < hi {
            let m = (lo + hi) >> 1;
            if cum[m] < x {
                lo = m + 1;
            } else {
                hi = m;
            }
        }
        let t = &tris[lo];
        let mut u = r.gen();
        let mut v = r.gen();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        pts.push(t[0] + ((t[1] - t[0]) * u + (t[2] - t[0]) * v));
    }
    pts
}

/// 源部件的表面采样：第 i 个部件的种子是 i + 1
pub fn source_samples(tris: &[Tri], i: usize, n: usize) -> Vec<Vec3> {
    sample_points(tris, n, (i + 1) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_matches_js() {
        // node -e "…rng(1)…" 的前三个值
        let mut r = Rng::new(1);
        let a = r.gen();
        let b = r.gen();
        let c = r.gen();
        assert_eq!(a, 0.8493364984169602);
        assert_eq!(b, 0.10365291335619986);
        assert_eq!(c, 0.9825238147750497);
    }

    #[test]
    fn seed_of_known() {
        assert_eq!(seed_of(""), 130763);
        assert_eq!(seed_of("rocket_nose"), 134983);
        assert_eq!(seed_of("tower.deck"), 34209);
    }
}
