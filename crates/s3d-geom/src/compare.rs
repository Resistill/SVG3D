//! 渲染对比（原型 `tools/agent.mjs` 的 compare 命令的定义重写）：源模型与重建在同一取景下各渲一张（color 模式），
//! 覆盖遮罩与灰度各做一次盒式模糊后比：
//!   missing = Σ max(0, 源 − 重建) / Σ 源（模糊后的覆盖），extra = Σ max(0, 重建 − 源) / Σ 源，
//!   diff = 模糊后灰度差的像素平均（0–1；背景的灰度按 1）。
//! 差异图：红 = 缺（源有、重建没有），蓝 = 多，灰 = 两边都有，白 = 都没有。
//! 单线程、按固定顺序累加，结果逐位可复现。

use crate::raster::{rasterize, Image, Mode, Options, RObj};
use crate::Aabb;

/// 盒式模糊（半径 r，两遍一维，越界按 0 计、按 2r + 1 归一）
pub fn box_blur(v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let norm = (2 * r + 1) as f64;
    let (wi, hi, ri) = (w as i64, h as i64, r as i64);
    let mut t = vec![0.0; w * h];
    for y in 0..h {
        let row = &v[y * w..(y + 1) * w];
        let mut acc = 0.0;
        for x in -ri..wi {
            if x + ri < wi {
                acc += row[(x + ri) as usize];
            }
            if x - ri > 0 {
                acc -= row[(x - ri - 1) as usize];
            }
            if x >= 0 {
                t[y * w + x as usize] = acc / norm;
            }
        }
    }
    let mut o = vec![0.0; w * h];
    for x in 0..w {
        let mut acc = 0.0;
        for y in -ri..hi {
            if y + ri < hi {
                acc += t[(y + ri) as usize * w + x];
            }
            if y - ri > 0 {
                acc -= t[(y - ri - 1) as usize * w + x];
            }
            if y >= 0 {
                o[y as usize * w + x] = acc / norm;
            }
        }
    }
    o
}

/// 一个视角的对比结果（未取整）
#[derive(Clone, Debug)]
pub struct ViewDiff {
    pub view: String,
    pub missing: f64,
    pub extra: f64,
    pub diff: f64,
    /// 差异图（RGB8，与渲染同尺寸）
    pub image: Image,
}

/// 两张同尺寸渲染图的对比
pub fn diff_images(a: &Image, b: &Image, blur: usize) -> (f64, f64, f64, Image) {
    let (w, h) = (a.w, a.h);
    let n = w * h;
    let occ = |im: &Image| -> Vec<f64> { (0..n).map(|k| if im.covered(k) { 1.0 } else { 0.0 }).collect() };
    let lum = |im: &Image, m: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|k| {
                if m[k] != 0.0 {
                    (im.rgb[3 * k] as f64 + im.rgb[3 * k + 1] as f64 + im.rgb[3 * k + 2] as f64) / 765.0
                } else {
                    1.0
                }
            })
            .collect()
    };
    let (ma, mb) = (occ(a), occ(b));
    let (ba, bb) = (box_blur(&ma, w, h, blur), box_blur(&mb, w, h, blur));
    let la = box_blur(&lum(a, &ma), w, h, blur);
    let lb = box_blur(&lum(b, &mb), w, h, blur);
    let (mut sa, mut miss, mut extra, mut diff) = (0.0, 0.0, 0.0, 0.0);
    for k in 0..n {
        sa += ba[k];
        miss += (ba[k] - bb[k]).max(0.0);
        extra += (bb[k] - ba[k]).max(0.0);
        diff += (la[k] - lb[k]).abs();
    }
    let mut rgb = vec![0u8; 3 * n];
    for k in 0..n {
        let (ia, ib) = (ma[k] != 0.0, mb[k] != 0.0);
        let c = match (ia, ib) {
            (true, false) => [230, 40, 40],
            (true, true) => [170, 170, 170],
            (false, true) => [60, 90, 230],
            (false, false) => [255, 255, 255],
        };
        rgb[3 * k..3 * k + 3].copy_from_slice(&c);
    }
    let s = if sa == 0.0 { 1.0 } else { sa };
    let img = Image {
        w,
        h,
        rgb,
        depth: vec![f64::INFINITY; n],
        ids: vec![-1; n],
    };
    (miss / s, extra / s, diff / n as f64, img)
}

/// 多视角对比：取景不给就按源模型（第一张源图）的包围盒（保留 4 位小数），之后各视角与重建都用它
pub fn compare(
    src: &[RObj],
    rec: &[RObj],
    views: &[String],
    size: usize,
    blur: usize,
    frame: Option<Aabb>,
) -> (Vec<ViewDiff>, Option<Aabb>) {
    let mut frame = frame;
    let mut out = Vec::new();
    for view in views {
        let opt = |frame: Option<Aabb>| Options {
            view: view.clone(),
            size,
            mode: Mode::Color,
            frame,
            ..Options::default()
        };
        let a = rasterize(src, &opt(frame));
        let fr = *frame.get_or_insert(a.frame);
        let b = rasterize(rec, &opt(Some(fr)));
        let (missing, extra, diff, image) = diff_images(&a.image, &b.image, blur);
        out.push(ViewDiff {
            view: view.clone(),
            missing,
            extra,
            diff,
            image,
        });
    }
    (out, frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;
    use std::borrow::Cow;

    #[test]
    fn blur_preserves_interior_mass() {
        let (w, h) = (9, 9);
        let mut v = vec![0.0; w * h];
        v[4 * w + 4] = 1.0;
        let o = box_blur(&v, w, h, 1);
        let s: f64 = o.iter().sum();
        assert!((s - 1.0).abs() < 1e-12);
        assert!((o[3 * w + 3] - 1.0 / 9.0).abs() < 1e-12);
        assert_eq!(o[0], 0.0);
    }

    fn cube(c: Vec3, s: f64) -> Vec<crate::tri::Tri> {
        let mut t = Vec::new();
        crate::tri::box_tris(&Aabb::new(c - Vec3::splat(s), c + Vec3::splat(s)), &mut t);
        t
    }

    #[test]
    fn identical_and_missing() {
        let o = |tris| RObj {
            path: "x".into(),
            tris: Cow::Owned(tris),
            color: [201, 201, 201],
            heat: None,
        };
        let a = vec![o(cube(Vec3::ZERO, 1.0)), o(cube(Vec3::new(3.0, 0.0, 0.0), 1.0))];
        let views = vec!["iso".to_string(), "top".to_string()];
        let (same, fr) = compare(&a, &a, &views, 64, 1, None);
        assert!(fr.is_some());
        for v in &same {
            assert_eq!((v.missing, v.extra, v.diff), (0.0, 0.0, 0.0));
        }
        // 重建少了一个立方体：missing 明显、extra 为 0
        let (d, _) = compare(&a, &a[..1], &views, 64, 1, None);
        for v in &d {
            assert!(
                v.missing > 0.2 && v.missing < 0.8,
                "{} missing {}",
                v.view,
                v.missing
            );
            assert!(v.extra < 1e-9);
            assert!(v.diff > 0.0);
            // 差异图里有红色
            assert!(v.image.rgb.chunks(3).any(|c| c == [230, 40, 40]));
        }
        // 反过来：extra（取景给到两个立方体都在画面里）
        let (e, _) = compare(&a[..1], &a, &views, 64, 1, fr);
        assert!(e.iter().all(|v| v.extra > 0.0));
    }
}
