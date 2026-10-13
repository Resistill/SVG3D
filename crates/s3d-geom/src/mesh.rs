//! 三角网格：GLB 读取（按节点 × 图元出部件，三角形按节点世界矩阵展开；材质颜色转成 sRGB 十六进制）与
//! 点到网格最近点（parry3d-f64 的 BVH，M1 的微基准用；评分用 `bvh` 模块）。

use crate::math::{self, Vec3};
use crate::tri::Tri;
use parry3d_f64::math::Vector;
use parry3d_f64::query::PointQuery;
use parry3d_f64::shape::TriMesh;
use std::collections::BTreeMap;

/// 展开到世界系的三角网格
pub struct Mesh {
    pub verts: Vec<Vec3>,
    pub tris: Vec<[u32; 3]>,
}

impl Mesh {
    pub fn bbox(&self) -> (Vec3, Vec3) {
        let mut lo = Vec3::splat(f64::INFINITY);
        let mut hi = Vec3::splat(f64::NEG_INFINITY);
        for v in &self.verts {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
        (lo, hi)
    }
}

/// 最近点查询器（BVH 建在 parry 的 TriMesh 上）
pub struct Nearest {
    tm: TriMesh,
}

impl Nearest {
    pub fn new(m: &Mesh) -> Result<Nearest, String> {
        let pts: Vec<Vector> = m.verts.iter().map(|v| Vector::new(v.x, v.y, v.z)).collect();
        let tm = TriMesh::new(pts, m.tris.clone()).map_err(|e| format!("{:?}", e))?;
        Ok(Nearest { tm })
    }

    /// 返回 (最近点, 距离)
    pub fn query(&self, p: Vec3) -> (Vec3, f64) {
        let q = Vector::new(p.x, p.y, p.z);
        let proj = self.tm.project_local_point(q, false);
        let c = Vec3::new(proj.point.x, proj.point.y, proj.point.z);
        (c, crate::math::length(c - p))
    }
}

// ====== 源部件 ======

/// 图元的材质（原型 parts.mjs：没有材质时 color = #c9c9c9、metal / rough 缺省）
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub name: Option<String>,
    /// sRGB 十六进制 `#rrggbb`
    pub color: String,
    pub metal: Option<f64>,
    pub rough: Option<f64>,
}

/// 源模型的一个部件：一个节点的一个图元（或合并网格拆出的一个连通分量），三角形在文件场景坐标系里
#[derive(Clone, Debug)]
pub struct Part {
    /// 节点名（没有名字、或拆分出的分量为 None）
    pub name: Option<String>,
    /// 唯一标识：重名的第 k 个写成 `名字@k`
    pub uid: Option<String>,
    /// 拆分出的分量的来源 `uid#k`
    pub from: Option<String>,
    pub tris: Vec<Tri>,
    pub material: Material,
    /// 享元线索：`网格序号:图元序号`
    pub mesh: Option<String>,
    /// 节点的世界矩阵（列主序 16 个数）
    pub matrix: Option<[f64; 16]>,
    /// 角色（工程按意图判定后填；读取时为 None）
    pub role: Option<String>,
    /// 旁路表的元数据
    pub meta: BTreeMap<String, String>,
}

impl Part {
    /// 查找用的名字：uid → name → from → `part_<i>`
    pub fn label(&self, i: usize) -> String {
        self.uid
            .clone()
            .or_else(|| self.name.clone())
            .or_else(|| self.from.clone())
            .unwrap_or_else(|| format!("part_{}", i))
    }
}

/// 合并网格的拆分方式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    /// 有名字以 `merged_` 开头的部件时拆分
    Auto,
    Components,
    None,
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * math::pow(c, 1.0 / 2.4) - 0.055
    }
}

/// 线性 RGB → `#rrggbb`（JS Math.round：.5 向上）
pub fn hex_color(rgb: [f64; 3]) -> String {
    let mut s = String::from("#");
    for c in rgb {
        let v = linear_to_srgb(c).clamp(0.0, 1.0) * 255.0;
        let k = (v + 0.5).floor() as u32;
        s.push_str(&format!("{:02x}", k.min(255)));
    }
    s
}

fn le_u32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *b.get(at)?,
        *b.get(at + 1)?,
        *b.get(at + 2)?,
        *b.get(at + 3)?,
    ]))
}

struct Accessor<'a> {
    data: &'a [u8],
    count: usize,
    stride: usize,
    comp_type: u32,
    ncomp: usize,
}

fn accessor<'a>(doc: &serde_json::Value, bin: &'a [u8], idx: usize) -> Option<Accessor<'a>> {
    let acc = doc["accessors"].get(idx)?;
    let bv = doc["bufferViews"].get(acc["bufferView"].as_u64()? as usize)?;
    let offset =
        bv["byteOffset"].as_u64().unwrap_or(0) as usize + acc["byteOffset"].as_u64().unwrap_or(0) as usize;
    let count = acc["count"].as_u64()? as usize;
    let comp_type = acc["componentType"].as_u64()? as u32;
    let ncomp = match acc["type"].as_str()? {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        _ => return None,
    };
    let comp_size = match comp_type {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        _ => return None,
    };
    let stride = bv["byteStride"]
        .as_u64()
        .map(|s| s as usize)
        .unwrap_or(comp_size * ncomp);
    let end = offset + stride * (count.saturating_sub(1)) + comp_size * ncomp;
    if end > bin.len() {
        return None;
    }
    Some(Accessor {
        data: &bin[offset..],
        count,
        stride,
        comp_type,
        ncomp,
    })
}

fn read_f32(a: &Accessor, i: usize, k: usize) -> f64 {
    let at = i * a.stride + k * 4;
    f32::from_le_bytes([a.data[at], a.data[at + 1], a.data[at + 2], a.data[at + 3]]) as f64
}

fn read_index(a: &Accessor, i: usize) -> u32 {
    let at = i * a.stride;
    match a.comp_type {
        5121 => a.data[at] as u32,
        5123 => u16::from_le_bytes([a.data[at], a.data[at + 1]]) as u32,
        _ => u32::from_le_bytes([a.data[at], a.data[at + 1], a.data[at + 2], a.data[at + 3]]),
    }
}

/// 节点的局部矩阵（列主序 16 个数）
fn node_matrix(n: &serde_json::Value) -> [f64; 16] {
    if let Some(m) = n["matrix"].as_array() {
        let mut out = [0.0; 16];
        for (i, x) in m.iter().enumerate().take(16) {
            out[i] = x.as_f64().unwrap_or(0.0);
        }
        return out;
    }
    let t = n["translation"]
        .as_array()
        .map(|a| {
            [
                a[0].as_f64().unwrap_or(0.0),
                a[1].as_f64().unwrap_or(0.0),
                a[2].as_f64().unwrap_or(0.0),
            ]
        })
        .unwrap_or([0.0; 3]);
    let r = n["rotation"]
        .as_array()
        .map(|a| {
            [
                a[0].as_f64().unwrap_or(0.0),
                a[1].as_f64().unwrap_or(0.0),
                a[2].as_f64().unwrap_or(0.0),
                a[3].as_f64().unwrap_or(1.0),
            ]
        })
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let s = n["scale"]
        .as_array()
        .map(|a| {
            [
                a[0].as_f64().unwrap_or(1.0),
                a[1].as_f64().unwrap_or(1.0),
                a[2].as_f64().unwrap_or(1.0),
            ]
        })
        .unwrap_or([1.0; 3]);
    let (x, y, z, w) = (r[0], r[1], r[2], r[3]);
    let rm = [
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + z * w),
        2.0 * (x * z - y * w),
        2.0 * (x * y - z * w),
        1.0 - 2.0 * (x * x + z * z),
        2.0 * (y * z + x * w),
        2.0 * (x * z + y * w),
        2.0 * (y * z - x * w),
        1.0 - 2.0 * (x * x + y * y),
    ];
    // 列主序：列 j 是 rm[3j..3j+3] * s[j]
    [
        rm[0] * s[0],
        rm[1] * s[0],
        rm[2] * s[0],
        0.0,
        rm[3] * s[1],
        rm[4] * s[1],
        rm[5] * s[1],
        0.0,
        rm[6] * s[2],
        rm[7] * s[2],
        rm[8] * s[2],
        0.0,
        t[0],
        t[1],
        t[2],
        1.0,
    ]
}

fn mat_mul(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut out = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            let mut s = 0.0;
            for k in 0..4 {
                s += a[k * 4 + r] * b[c * 4 + k];
            }
            out[c * 4 + r] = s;
        }
    }
    out
}

fn mat_apply(m: &[f64; 16], p: Vec3) -> Vec3 {
    Vec3::new(
        m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
        m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
        m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
    )
}

fn material_of(doc: &serde_json::Value, idx: Option<u64>) -> Material {
    let m = match idx.and_then(|i| doc["materials"].get(i as usize)) {
        Some(m) => m,
        None => {
            return Material {
                name: None,
                color: "#c9c9c9".to_string(),
                metal: None,
                rough: None,
            }
        }
    };
    let pbr = &m["pbrMetallicRoughness"];
    let bc = pbr["baseColorFactor"].as_array();
    let rgb = [0, 1, 2].map(|k| bc.and_then(|a| a.get(k)).and_then(|x| x.as_f64()).unwrap_or(1.0));
    let name = m["name"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string());
    Material {
        name,
        color: hex_color(rgb),
        metal: Some(pbr["metallicFactor"].as_f64().unwrap_or(1.0)),
        rough: Some(pbr["roughnessFactor"].as_f64().unwrap_or(1.0)),
    }
}

/// 读 GLB 的全部部件（原型 `partsOfDocument`）：场景节点深度优先，每个节点的每个三角形图元一个部件；
/// 重名的节点第 k 个写成 `名字@k`；`split` 决定是否按连通分量拆开合并网格
pub fn load_parts(bytes: &[u8], split: Split) -> Result<Vec<Part>, String> {
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err("不是 GLB 文件".to_string());
    }
    let json_len = le_u32(bytes, 12).ok_or("GLB 头损坏")? as usize;
    let json_bytes = bytes.get(20..20 + json_len).ok_or("GLB JSON 块越界")?;
    let doc: serde_json::Value =
        serde_json::from_slice(json_bytes).map_err(|e| format!("GLB JSON：{}", e))?;
    let bin_start = 20 + json_len;
    let bin = if bin_start + 8 <= bytes.len() {
        let bin_len = le_u32(bytes, bin_start).unwrap_or(0) as usize;
        bytes.get(bin_start + 8..bin_start + 8 + bin_len).unwrap_or(&[])
    } else {
        &[]
    };
    let mut parts: Vec<Part> = Vec::new();
    let ident = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let scenes: Vec<usize> = doc["scenes"]
        .as_array()
        .map(|a| (0..a.len()).collect())
        .unwrap_or_default();
    for si in scenes {
        let roots: Vec<usize> = doc["scenes"][si]["nodes"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as usize)).collect())
            .unwrap_or_default();
        let mut stack: Vec<(usize, [f64; 16])> = roots.iter().rev().map(|&n| (n, ident)).collect();
        while let Some((ni, parent)) = stack.pop() {
            let node = match doc["nodes"].get(ni) {
                Some(n) => n,
                None => continue,
            };
            let world = mat_mul(&parent, &node_matrix(node));
            if let Some(mi) = node["mesh"].as_u64() {
                if let Some(prims) = doc["meshes"][mi as usize]["primitives"].as_array() {
                    for (pk, prim) in prims.iter().enumerate() {
                        if prim["mode"].as_u64().map(|m| m != 4).unwrap_or(false) {
                            continue;
                        }
                        let pos_idx = match prim["attributes"]["POSITION"].as_u64() {
                            Some(i) => i as usize,
                            None => continue,
                        };
                        let pos = match accessor(&doc, bin, pos_idx) {
                            Some(a) if a.comp_type == 5126 && a.ncomp == 3 => a,
                            _ => continue,
                        };
                        let vert = |i: usize| {
                            mat_apply(
                                &world,
                                Vec3::new(read_f32(&pos, i, 0), read_f32(&pos, i, 1), read_f32(&pos, i, 2)),
                            )
                        };
                        let mut tris: Vec<Tri> = Vec::new();
                        match prim["indices"].as_u64().and_then(|ii| accessor(&doc, bin, ii as usize)) {
                            Some(ix) => {
                                let mut k = 0;
                                while k + 2 < ix.count {
                                    tris.push([
                                        vert(read_index(&ix, k) as usize),
                                        vert(read_index(&ix, k + 1) as usize),
                                        vert(read_index(&ix, k + 2) as usize),
                                    ]);
                                    k += 3;
                                }
                            }
                            None => {
                                let mut k = 0;
                                while k + 2 < pos.count {
                                    tris.push([vert(k), vert(k + 1), vert(k + 2)]);
                                    k += 3;
                                }
                            }
                        }
                        parts.push(Part {
                            name: node["name"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                            uid: None,
                            from: None,
                            tris,
                            material: material_of(&doc, prim["material"].as_u64()),
                            mesh: Some(format!("{}:{}", mi, pk)),
                            matrix: Some(world),
                            role: None,
                            meta: BTreeMap::new(),
                        });
                    }
                }
            }
            if let Some(ch) = node["children"].as_array() {
                for c in ch.iter().rev() {
                    if let Some(ci) = c.as_u64() {
                        stack.push((ci as usize, world));
                    }
                }
            }
        }
    }
    // 唯一标识
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for p in &parts {
        if let Some(n) = &p.name {
            *count.entry(n.clone()).or_insert(0) += 1;
        }
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for p in parts.iter_mut() {
        if let Some(n) = p.name.clone() {
            if count[&n] > 1 {
                let k = seen.entry(n.clone()).or_insert(0);
                *k += 1;
                p.uid = Some(format!("{}@{}", n, k));
            } else {
                p.uid = Some(n);
            }
        }
    }
    let do_split = match split {
        Split::Components => true,
        Split::None => false,
        Split::Auto => parts
            .iter()
            .any(|p| p.name.as_deref().map(|n| n.starts_with("merged_")).unwrap_or(false)),
    };
    if do_split {
        let mut out = Vec::new();
        for p in parts {
            out.extend(split_components(p));
        }
        Ok(out)
    } else {
        Ok(parts)
    }
}

fn weld_key(p: Vec3) -> [i64; 3] {
    let r = |x: f64| (x * 1e4 + 0.5).floor() as i64;
    [r(p.x), r(p.y), r(p.z)]
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// 按共享顶点（位置按 1e-4 焊接）把三角形分成连通分量；分量按第一个三角形的出现顺序排列
pub fn split_components(part: Part) -> Vec<Part> {
    let n = part.tris.len();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut owner: BTreeMap<[i64; 3], usize> = BTreeMap::new();
    for (i, t) in part.tris.iter().enumerate() {
        for v in t {
            let k = weld_key(*v);
            match owner.get(&k) {
                Some(&j) => {
                    let a = find(&mut parent, i);
                    let b = find(&mut parent, j);
                    if a != b {
                        parent[a] = b;
                    }
                }
                None => {
                    owner.insert(k, i);
                }
            }
        }
    }
    let mut groups: Vec<(usize, Vec<Tri>)> = Vec::new();
    let mut index_of: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, t) in part.tris.iter().enumerate() {
        let r = find(&mut parent, i);
        let g = match index_of.get(&r) {
            Some(&g) => g,
            None => {
                groups.push((r, Vec::new()));
                index_of.insert(r, groups.len() - 1);
                groups.len() - 1
            }
        };
        groups[g].1.push(*t);
    }
    let base = part.uid.clone().or(part.name.clone()).unwrap_or_default();
    groups
        .into_iter()
        .enumerate()
        .map(|(k, (_, tris))| Part {
            name: None,
            uid: None,
            from: Some(format!("{}#{}", base, k)),
            tris,
            material: part.material.clone(),
            mesh: None,
            matrix: None,
            role: None,
            meta: BTreeMap::new(),
        })
        .collect()
}

/// 读 GLB：全部网格按节点变换展开成一张世界系三角网格（微基准用）
pub fn load_glb(bytes: &[u8]) -> Result<Mesh, String> {
    let parts = load_parts(bytes, Split::None)?;
    let mut mesh = Mesh {
        verts: Vec::new(),
        tris: Vec::new(),
    };
    for p in &parts {
        for t in &p.tris {
            let base = mesh.verts.len() as u32;
            mesh.verts.extend_from_slice(t);
            mesh.tris.push([base, base + 1, base + 2]);
        }
    }
    if mesh.tris.is_empty() {
        return Err("GLB 里没有三角形".to_string());
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_rounding() {
        assert_eq!(hex_color([1.0, 1.0, 1.0]), "#ffffff");
        assert_eq!(hex_color([0.0, 0.0, 0.0]), "#000000");
        // 0.5 线性 ≈ 0.7354 sRGB → 188 (bc)
        assert_eq!(hex_color([0.5, 0.5, 0.5]), "#bcbcbc");
    }
}
