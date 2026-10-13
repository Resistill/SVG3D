//! CPU 软件光栅（原型 `tools/raster.mjs` 与 `src/views.js` 的定义重写）：z-buffer 平头着色，
//! 给 compare（可复现的尺子，PLAN 原则 18）与无 GPU 环境下的 `render` 用。
//! 顶点变换与投影全部 f64，超越函数走 `math`（libm）；单线程逐三角形按输入顺序画，结果只取决于输入。
//!
//! 与原型的差别：原型的顶点是 Float32、深度缓冲是 Float32Array，本版全程 f64（边缘像素可能差一两个）；
//! 聚焦（focus）与高亮（highlight）没有移植（compare 用不到，界面另有 GPU 预览）。

use crate::math::{self, Vec3};
use crate::tri::Tri;
use crate::Aabb;
use std::borrow::Cow;

/// 视角预设：从目标指向相机的方向（与原型 views.js 相同；top 带一点 z，屏幕上方是 −z）
pub const VIEWS: [(&str, [f64; 3]); 7] = [
    ("iso", [0.75, 0.55, 0.9]),
    ("iso2", [-0.75, 0.55, -0.9]),
    ("top", [0.0, 1.0, 0.0001]),
    ("front", [0.0, 0.25, 1.0]),
    ("back", [0.0, 0.25, -1.0]),
    ("left", [-1.0, 0.25, 0.0]),
    ("right", [1.0, 0.25, 0.0]),
];

/// 按名字取视角方向
pub fn view_dir(name: &str) -> Option<Vec3> {
    VIEWS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, d)| Vec3::from_array(*d))
}

/// 视角名字列表（报错用）
pub fn view_names() -> Vec<&'static str> {
    VIEWS.iter().map(|(n, _)| *n).collect()
}

fn unit_or_self(v: Vec3) -> Vec3 {
    let l = math::length(v);
    if l == 0.0 {
        v
    } else {
        v / l
    }
}

/// JS 的 Math.round（.5 向 +∞）
fn jround(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// 相机：位置、目标、竖直视野（度）、近平面、取景半径
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub fov: f64,
    pub near: f64,
    pub r: f64,
}

/// 包围盒取景（views.js 的 camera）：整体 1.6 倍半径、聚焦 2.4 倍；半径至少 2。
/// `elev`（度）给了就沿视角的水平方向按这个俯仰角重新定方向（0 是真正的立面）。未知视角按 iso
pub fn camera(bx: &Aabb, view: &str, focus: bool, elev: Option<f64>) -> Camera {
    let v = view_dir(view).unwrap_or(Vec3::new(0.75, 0.55, 0.9));
    let hz = math::hypot(v.x, v.z);
    let d = match elev {
        Some(e) if hz > 1e-3 => unit_or_self(Vec3::new(v.x / hz, math::tan(math::rad(e)), v.z / hz)),
        _ => unit_or_self(v),
    };
    let c = (bx.min + bx.max) / 2.0;
    let r = (math::length(bx.max - bx.min) / 2.0).max(2.0);
    let eye = c + d * (r * if focus { 2.4 } else { 1.6 });
    Camera {
        eye,
        target: c,
        fov: 45.0,
        near: 0.1,
        r,
    }
}

/// 着色模式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// 按材质颜色平头着色、描轮廓
    Color,
    /// 每个物体一种颜色（不着色、不描轮廓），图例给出颜色 → 路径
    Id,
    /// color 图上在每个物体可见部分的中心标编号，图例给出编号 → 路径
    Labels,
    /// 按物体的 heat（0..1）从绿到红上色，没有 heat 的画成浅灰
    Error,
}

impl Mode {
    pub fn parse(s: &str) -> Option<Mode> {
        match s {
            "color" => Some(Mode::Color),
            "id" => Some(Mode::Id),
            "labels" => Some(Mode::Labels),
            "error" => Some(Mode::Error),
            _ => None,
        }
    }
}

/// 一个要画的物体（三角形在世界坐标里）
#[derive(Clone, Debug)]
pub struct RObj<'a> {
    pub path: String,
    pub tris: Cow<'a, [Tri]>,
    pub color: [u8; 3],
    pub heat: Option<f64>,
}

/// 渲染参数
#[derive(Clone, Debug)]
pub struct Options {
    pub view: String,
    pub size: usize,
    pub mode: Mode,
    /// 取景范围（不给就按全部物体）
    pub frame: Option<Aabb>,
    /// 正交相机（按包围球取景，半视野 = 1.08 r）
    pub ortho: bool,
    /// 水平剖切高度：高于它的部分切掉，剖面画成 1.5 像素宽的剖面线
    pub cut: Option<f64>,
    /// 俯仰角（度）
    pub elev: Option<f64>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            view: "iso".into(),
            size: 1024,
            mode: Mode::Color,
            frame: None,
            ortho: false,
            cut: None,
            elev: None,
        }
    }
}

/// 图像：RGB8、深度（无穷大 = 背景）、物体编号（−1 = 背景）
#[derive(Clone, Debug)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<u8>,
    pub depth: Vec<f64>,
    pub ids: Vec<i32>,
}

impl Image {
    pub fn png(&self) -> Vec<u8> {
        crate::png::encode(&self.rgb, self.w, self.h)
    }
    /// 有物体覆盖的像素
    pub fn covered(&self, k: usize) -> bool {
        self.depth[k].is_finite()
    }
}

/// 图例的一项
#[derive(Clone, Debug, PartialEq)]
pub enum Legend {
    /// id 模式：颜色 `#rrggbb` → 路径
    Color { color: String, path: String },
    /// labels 模式：编号 → 路径、可见像素数、是否标上了（放不下就没标）
    Label {
        n: usize,
        path: String,
        pixels: usize,
        labeled: bool,
    },
}

/// 渲染结果
#[derive(Clone, Debug)]
pub struct Render {
    pub image: Image,
    pub legend: Vec<Legend>,
    pub camera: Camera,
    /// 实际取景范围（保留 4 位小数；原样传回下一次渲染可得同一取景）
    pub frame: Aabb,
}

/// `#rrggbb` → RGB；解析不了按 #c9c9c9
pub fn hex_rgb(s: &str) -> [u8; 3] {
    let t = s.strip_prefix('#').unwrap_or(s);
    match (t.len(), u32::from_str_radix(t, 16)) {
        (6, Ok(n)) => [(n >> 16) as u8, (n >> 8) as u8, n as u8],
        _ => [0xc9, 0xc9, 0xc9],
    }
}

/// 误差 → 颜色（0 绿 → 0.5 黄 → 1 红）
pub fn heat_color(t: f64) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        [jround(60.0 + 390.0 * t) as u8, 190, 70]
    } else {
        [235, jround(190.0 - 300.0 * (t - 0.5)) as u8, 60]
    }
}

/// 物体编号 → 区分度高的颜色（黄金角色相，HSL）
pub fn id_color(i: usize) -> [u8; 3] {
    let h = (i as f64 * 137.508) % 360.0;
    let s = 0.65;
    let l = 0.5 + 0.12 * ((i % 3) as f64 - 1.0);
    let a = s * l.min(1.0 - l);
    let ch = |n: f64| {
        let k = (n + h / 30.0) % 12.0;
        jround(255.0 * (l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0))) as u8
    };
    [ch(0.0), ch(8.0), ch(4.0)]
}

/// 把三角形裁剪到 y ≤ h（Sutherland–Hodgman，扇形三角化）；
/// w > 0 时把三角形与剖切面的交线画成剖切高度略下方、宽 w 的双面细长条
pub fn clip_below(tris: &[Tri], h: f64, w: f64) -> Vec<Tri> {
    let mut out = Vec::new();
    let mut segs: Vec<(Vec3, Vec3)> = Vec::new();
    for t in tris {
        if t.iter().all(|p| p.y <= h) {
            out.push(*t);
            continue;
        }
        if t.iter().all(|p| p.y > h) {
            continue;
        }
        let mut poly: Vec<Vec3> = Vec::with_capacity(4);
        for k in 0..3 {
            let a = t[k];
            let b = t[(k + 1) % 3];
            let (ain, bin) = (a.y <= h, b.y <= h);
            if ain {
                poly.push(a);
            }
            if ain != bin {
                let s = (h - a.y) / (b.y - a.y);
                poly.push(Vec3::new(a.x + s * (b.x - a.x), h, a.z + s * (b.z - a.z)));
            }
        }
        for k in 1..poly.len().saturating_sub(1) {
            out.push([poly[0], poly[k], poly[k + 1]]);
        }
        let xs: Vec<Vec3> = poly.iter().copied().filter(|p| p.y == h).collect();
        if xs.len() == 2 {
            segs.push((xs[0], xs[1]));
        }
    }
    if w > 0.0 {
        for (a, b) in segs {
            let (dx, dz) = (b.x - a.x, b.z - a.z);
            let l = math::hypot(dx, dz);
            if l < 1e-9 {
                continue;
            }
            let (nx, nz) = (-dz / l * (w / 2.0), dx / l * (w / 2.0));
            let y = h - 1e-4;
            let pa = Vec3::new(a.x + nx, y, a.z + nz);
            let pb = Vec3::new(b.x + nx, y, b.z + nz);
            let pc = Vec3::new(b.x - nx, y, b.z - nz);
            let pd = Vec3::new(a.x - nx, y, a.z - nz);
            out.extend([[pa, pb, pc], [pa, pc, pd], [pa, pc, pb], [pa, pd, pc]]);
        }
    }
    out
}

fn bounds(objs: &[RObj]) -> Option<Aabb> {
    Aabb::of_points(
        objs.iter()
            .flat_map(|o| o.tris.iter().flat_map(|t| t.iter().copied())),
    )
}

fn round4(b: &Aabb) -> Aabb {
    let r = |v: Vec3| Vec3::new(jround4(v.x), jround4(v.y), jround4(v.z));
    Aabb::new(r(b.min), r(b.max))
}

fn jround4(x: f64) -> f64 {
    jround(x * 1e4) / 1e4
}

/// 光栅化（原型 raster.mjs 的 rasterize，去掉聚焦 / 高亮）
pub fn rasterize(objects: &[RObj], opt: &Options) -> Render {
    let size = opt.size.max(1);
    let (w, h) = (size, size);
    // 剖切：剖面线宽 = 画面尺度的 1.5 像素
    let clipped: Option<Vec<RObj>> = opt.cut.map(|cut| {
        let fb = match &opt.frame {
            Some(f) => (f.max.x - f.min.x).max(f.max.z - f.min.z),
            None => match bounds(objects) {
                Some(b) => (b.max.x - b.min.x).max(b.max.z - b.min.z).max(1e-6),
                None => 1e-6,
            },
        };
        let lw = 1.5 * fb / size as f64;
        objects
            .iter()
            .map(|o| RObj {
                path: o.path.clone(),
                tris: Cow::Owned(clip_below(&o.tris, cut, lw)),
                color: o.color,
                heat: o.heat,
            })
            .collect()
    });
    let objects: &[RObj] = clipped.as_deref().unwrap_or(objects);
    let bx = match &opt.frame {
        Some(f) => *f,
        None => bounds(objects).unwrap_or(Aabb::new(Vec3::splat(-1.0), Vec3::splat(1.0))),
    };
    let cam = camera(&bx, &opt.view, false, opt.elev);
    let fwd = unit_or_self(cam.target - cam.eye);
    let helper = if fwd.y.abs() > 0.999 {
        Vec3::new(0.0, 0.0, -1.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let right = unit_or_self(fwd.cross(helper));
    let up = right.cross(fwd);
    let f = 1.0 / math::tan(math::rad(cam.fov) / 2.0);
    let light = unit_or_self(right * 0.4 + up * 0.8 - fwd * 0.5);
    let (wf, hf) = (w as f64, h as f64);
    let mut img = Image {
        w,
        h,
        rgb: vec![0; w * h * 3],
        depth: vec![f64::INFINITY; w * h],
        ids: vec![-1; w * h],
    };
    // 背景：浅灰渐变
    for y in 0..h {
        let g = (236.0 - jround(18.0 * y as f64 / hf)) as u8;
        for x in 0..w {
            let k = (y * w + x) * 3;
            img.rgb[k..k + 3].copy_from_slice(&[g, g, g + 3]);
        }
    }
    let half = cam.r * 1.08;
    let proj = |p: Vec3| -> [f64; 3] {
        let v = p - cam.eye;
        let z = v.dot(fwd);
        if opt.ortho {
            [
                wf / 2.0 + v.dot(right) / half * (wf / 2.0),
                hf / 2.0 - v.dot(up) / half * (hf / 2.0),
                z,
            ]
        } else {
            [
                wf / 2.0 + v.dot(right) * f * wf / (2.0 * z),
                hf / 2.0 - v.dot(up) * f * hf / (2.0 * z),
                z,
            ]
        }
    };
    for (id, o) in objects.iter().enumerate() {
        let base = match opt.mode {
            Mode::Id => id_color(id),
            Mode::Error => o.heat.map(heat_color).unwrap_or([205, 205, 205]),
            _ => o.color,
        };
        for t in o.tris.iter() {
            let p = [proj(t[0]), proj(t[1]), proj(t[2])];
            // 跨过近平面的三角形直接丢弃（相机在场景外，影响很小）
            if p.iter().any(|q| q[2] < cam.near) {
                continue;
            }
            let col = if opt.mode == Mode::Id {
                base
            } else {
                let n = unit_or_self((t[1] - t[0]).cross(t[2] - t[0]));
                let shade = 0.42 + 0.58 * n.dot(light).abs();
                base.map(|c| jround(c as f64 * shade).min(255.0) as u8)
            };
            let fx = |a: f64, b: f64, c: f64| a.min(b).min(c);
            let gx = |a: f64, b: f64, c: f64| a.max(b).max(c);
            let min_x = fx(p[0][0], p[1][0], p[2][0]).floor().max(0.0);
            let max_x = gx(p[0][0], p[1][0], p[2][0]).ceil().min(wf - 1.0);
            let min_y = fx(p[0][1], p[1][1], p[2][1]).floor().max(0.0);
            let max_y = gx(p[0][1], p[1][1], p[2][1]).ceil().min(hf - 1.0);
            if !(min_x <= max_x && min_y <= max_y) {
                continue;
            }
            let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
            if area.abs() < 1e-12 {
                continue;
            }
            // 透视正确的深度：对 1/z 做重心插值（正交直接插值 z）
            let iz = p.map(|q| if opt.ortho { q[2] } else { 1.0 / q[2] });
            for y in min_y as usize..=max_y as usize {
                let py = y as f64 + 0.5;
                for x in min_x as usize..=max_x as usize {
                    let px = x as f64 + 0.5;
                    let w0 = ((p[1][0] - px) * (p[2][1] - py) - (p[2][0] - px) * (p[1][1] - py)) / area;
                    let w1 = ((p[2][0] - px) * (p[0][1] - py) - (p[0][0] - px) * (p[2][1] - py)) / area;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                        continue;
                    }
                    let zi = w0 * iz[0] + w1 * iz[1] + w2 * iz[2];
                    let z = if opt.ortho { zi } else { 1.0 / zi };
                    let k = y * w + x;
                    // NaN（退化）也不写
                    if z.is_nan() || z >= img.depth[k] {
                        continue;
                    }
                    img.depth[k] = z;
                    img.ids[k] = id as i32;
                    img.rgb[k * 3..k * 3 + 3].copy_from_slice(&col);
                }
            }
        }
    }
    // 轮廓：与左边或上边的像素属于不同物体时描深
    if opt.mode != Mode::Id {
        for y in 1..h {
            for x in 1..w {
                let k = y * w + x;
                let i = img.ids[k];
                if i >= 0 && (i != img.ids[k - 1] || i != img.ids[k - w]) {
                    for j in 0..3 {
                        img.rgb[k * 3 + j] = jround(img.rgb[k * 3 + j] as f64 * 0.6) as u8;
                    }
                }
            }
        }
    }
    let mut legend = Vec::new();
    if opt.mode == Mode::Id {
        for (id, o) in objects.iter().enumerate() {
            let c = id_color(id);
            legend.push(Legend::Color {
                color: format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]),
                path: o.path.clone(),
            });
        }
    }
    if opt.mode == Mode::Labels {
        legend = place_labels(&mut img, objects);
    }
    Render {
        image: img,
        legend,
        camera: cam,
        frame: round4(&bx),
    }
}

/// labels 模式：按可见面积从大到小在每个物体可见像素的中心标编号；
/// 与已放的标签重叠就在物体自己的像素里就近挪（半径 0..60 步 6、16 个方向），放不下就不标
fn place_labels(img: &mut Image, objects: &[RObj]) -> Vec<Legend> {
    let (w, h) = (img.w, img.h);
    // (sx, sy, 像素数, 第一次出现的像素序号)
    let mut acc: Vec<(u64, u64, usize, usize)> = vec![(0, 0, 0, usize::MAX); objects.len()];
    for k in 0..w * h {
        let i = img.ids[k];
        if i >= 0 {
            let a = &mut acc[i as usize];
            a.0 += (k % w) as u64;
            a.1 += (k / w) as u64;
            a.2 += 1;
            if a.3 == usize::MAX {
                a.3 = k;
            }
        }
    }
    let mut order: Vec<usize> = (0..objects.len()).filter(|&i| acc[i].2 > 0).collect();
    // 原型按 Map 的插入序（像素首次出现的顺序）再按像素数稳定排序
    order.sort_by_key(|&i| acc[i].3);
    order.sort_by(|&a, &b| acc[b].2.cmp(&acc[a].2));
    let mut placed: Vec<[f64; 4]> = Vec::new();
    let mut legend = Vec::new();
    let mut n = 0usize;
    for id in order {
        let (sx, sy, c, _) = acc[id];
        if c < 40 {
            continue;
        }
        n += 1;
        let cx = jround(sx as f64 / c as f64);
        let cy = jround(sy as f64 / c as f64);
        let digits = n.to_string().len() as f64;
        let rect = |x: f64, y: f64| {
            let (rw, rh) = (digits * 12.0 + 4.0, 18.0);
            [
                x - rw / 2.0 - 2.0,
                y - rh / 2.0 - 2.0,
                x + rw / 2.0 + 2.0,
                y + rh / 2.0 + 2.0,
            ]
        };
        let mut spot: Option<(i64, i64, [f64; 4])> = None;
        let mut rad = 0;
        while rad <= 60 && spot.is_none() {
            let dirs = if rad == 0 { 1 } else { 16 };
            for a in 0..dirs {
                let ang = a as f64 * std::f64::consts::PI / 8.0;
                let x = jround(cx + rad as f64 * math::cos(ang));
                let y = jround(cy + rad as f64 * math::sin(ang));
                if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
                    continue;
                }
                if img.ids[y as usize * w + x as usize] != id as i32 {
                    continue;
                }
                let r = rect(x, y);
                if !placed
                    .iter()
                    .any(|q| r[0] < q[2] && r[2] > q[0] && r[1] < q[3] && r[3] > q[1])
                {
                    spot = Some((x as i64, y as i64, r));
                    break;
                }
            }
            rad += 6;
        }
        if let Some((x, y, r)) = spot {
            placed.push(r);
            draw_number(img, x, y, n);
        }
        legend.push(Legend::Label {
            n,
            path: objects[id].path.clone(),
            pixels: c,
            labeled: spot.is_some(),
        });
    }
    legend
}

/// 5×7 点阵数字（逐行，1 = 墨）
const DIGITS: [&str; 10] = [
    "01110100011001110101110011000101110",
    "00100011000010000100001000010001110",
    "01110100010000100010001000100011111",
    "11111000100010000010000011000101110",
    "00010001100101010010111110001000010",
    "11111100001111000001000011000101110",
    "00110010001000011110100011000101110",
    "11111000010001000100010000100001000",
    "01110100011000101110100011000101110",
    "01110100011000101111000010001001100",
];

/// 白底黑框黑字的编号，点阵放大 2 倍
fn draw_number(img: &mut Image, cx: i64, cy: i64, n: usize) {
    let s = n.to_string();
    let sc = 2i64;
    let cw = 6 * sc;
    let bw = s.len() as i64 * cw + 2 * sc;
    let bh = 7 * sc + 4 * sc;
    let x0 = jround(cx as f64 - bw as f64 / 2.0) as i64;
    let y0 = jround(cy as f64 - bh as f64 / 2.0) as i64;
    let (w, h) = (img.w as i64, img.h as i64);
    let mut put = |x: i64, y: i64, c: [u8; 3]| {
        if x >= 0 && y >= 0 && x < w && y < h {
            let k = ((y * w + x) * 3) as usize;
            img.rgb[k..k + 3].copy_from_slice(&c);
        }
    };
    for y in 0..bh {
        for x in 0..bw {
            let edge = y == 0 || x == 0 || y == bh - 1 || x == bw - 1;
            put(x0 + x, y0 + y, if edge { [30, 30, 30] } else { [255, 255, 255] });
        }
    }
    for (i, ch) in s.bytes().enumerate() {
        let bits = DIGITS[(ch - b'0') as usize].as_bytes();
        for r in 0..7i64 {
            for c in 0..5i64 {
                if bits[(r * 5 + c) as usize] == b'1' {
                    for dy in 0..sc {
                        for dx in 0..sc {
                            put(
                                x0 + 2 * sc + i as i64 * cw + c * sc + dx,
                                y0 + 2 * sc + r * sc + dy,
                                [20, 20, 20],
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(tris: Vec<Tri>) -> RObj<'static> {
        RObj {
            path: "a".into(),
            tris: Cow::Owned(tris),
            color: [200, 100, 50],
            heat: None,
        }
    }

    #[test]
    fn id_and_heat_colors() {
        // 与原型 idColor(0) / idColor(1) 的值相同
        assert_eq!(id_color(0), [160, 34, 34]);
        assert_eq!(id_color(1), [45, 210, 93]);
        assert_eq!(id_color(7), [52, 45, 210]);
        assert_eq!(heat_color(0.0), [60, 190, 70]);
        assert_eq!(heat_color(1.0), [235, 40, 60]);
        assert_eq!(hex_rgb("#c9c9c9"), [201, 201, 201]);
        assert_eq!(hex_rgb("bad"), [201, 201, 201]);
    }

    #[test]
    fn camera_iso() {
        let b = Aabb::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let c = camera(&b, "iso", false, None);
        assert_eq!(c.r, 2.0); // 半对角 √3 < 2，取下限 2
        let d = (c.eye - c.target) / 3.2;
        assert!((math::length(d) - 1.0).abs() < 1e-12);
        // elev = 0：水平
        let c0 = camera(&b, "front", false, Some(0.0));
        assert!((c0.eye.y - c0.target.y).abs() < 1e-12);
    }

    /// 正交俯视一个铺满取景框中心的水平正方形：中心像素被覆盖、角上是背景
    #[test]
    fn square_top_ortho() {
        let (a, b, c, d) = (
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(-1.0, 0.0, 1.0),
        );
        let o = obj(vec![[a, b, c], [a, c, d]]);
        let frame = Aabb::new(Vec3::new(-3.0, -1.0, -3.0), Vec3::new(3.0, 1.0, 3.0));
        let r = rasterize(
            &[o],
            &Options {
                view: "top".into(),
                size: 64,
                frame: Some(frame),
                ortho: true,
                ..Options::default()
            },
        );
        let im = &r.image;
        assert!(im.covered(32 * 64 + 32));
        assert!(!im.covered(0));
        assert_eq!(im.ids[32 * 64 + 32], 0);
        // 正方形边长 2，半视野 1.08 r（r = 半对角 ≈ 4.36），占 64 × 2 / (2 × 4.71) ≈ 13.6 像素
        let n = (0..64 * 64).filter(|&k| im.covered(k)).count();
        assert!((150..230).contains(&n), "覆盖像素 {}", n);
        // 背景渐变首行 236
        assert_eq!(&im.rgb[0..3], &[236, 236, 239]);
        // 俯视的水平面：法线与光的夹角固定，内部像素是 base × shade
        let k = (32 * 64 + 32) * 3;
        assert!(im.rgb[k] > 100 && im.rgb[k] <= 200);
    }

    /// 透视下两个前后重叠的三角形：近的赢
    #[test]
    fn zbuffer_near_wins() {
        let tri = |z: f64| {
            [
                Vec3::new(-1.0, -1.0, z),
                Vec3::new(1.0, -1.0, z),
                Vec3::new(0.0, 1.0, z),
            ]
        };
        let far = RObj {
            path: "far".into(),
            tris: Cow::Owned(vec![tri(-0.5)]),
            color: [255, 0, 0],
            heat: None,
        };
        let near = RObj {
            path: "near".into(),
            tris: Cow::Owned(vec![tri(0.5)]),
            color: [0, 255, 0],
            heat: None,
        };
        let opt = Options {
            view: "front".into(),
            size: 32,
            mode: Mode::Id,
            elev: Some(0.0),
            ..Options::default()
        };
        let r1 = rasterize(&[far.clone(), near.clone()], &opt);
        let r2 = rasterize(&[near, far], &opt);
        let c = 16 * 32 + 16;
        assert_eq!(r1.image.ids[c], 1);
        assert_eq!(r2.image.ids[c], 0);
        assert_eq!(r1.legend.len(), 2);
    }

    #[test]
    fn clip_below_halves() {
        let t: Tri = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let out = clip_below(&[t], 1.0, 0.0);
        assert_eq!(out.len(), 2); // 梯形 → 2 个三角形
        assert!(out.iter().all(|t| t.iter().all(|p| p.y <= 1.0)));
        let out = clip_below(&[t], 1.0, 0.1);
        assert_eq!(out.len(), 6); // 再加 4 个剖面线三角形
    }
}
