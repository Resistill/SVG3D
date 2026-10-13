//! 三角网格：GLB 读取（只取 POSITION 与索引，按节点变换展开）与点到网格最近点（parry3d-f64 的 BVH，
//! enhanced-determinism）。M1 用它跑最近点微基准；M2 评分的双向残差也从这里取。

use crate::math::Vec3;
use parry3d_f64::math::Vector;
use parry3d_f64::query::PointQuery;
use parry3d_f64::shape::TriMesh;

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

// ====== GLB ======

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

/// 读 GLB：全部网格按节点变换展开成一张世界系三角网格
pub fn load_glb(bytes: &[u8]) -> Result<Mesh, String> {
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
    let mut mesh = Mesh {
        verts: Vec::new(),
        tris: Vec::new(),
    };
    let scene_idx = doc["scene"].as_u64().unwrap_or(0) as usize;
    let roots: Vec<usize> = doc["scenes"]
        .get(scene_idx)
        .and_then(|s| s["nodes"].as_array())
        .map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as usize)).collect())
        .unwrap_or_default();
    let ident = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let mut stack: Vec<(usize, [f64; 16])> = roots.iter().rev().map(|&n| (n, ident)).collect();
    while let Some((ni, parent)) = stack.pop() {
        let node = match doc["nodes"].get(ni) {
            Some(n) => n,
            None => continue,
        };
        let world = mat_mul(&parent, &node_matrix(node));
        if let Some(mi) = node["mesh"].as_u64() {
            if let Some(prims) = doc["meshes"][mi as usize]["primitives"].as_array() {
                for prim in prims {
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
                    let base = mesh.verts.len() as u32;
                    for i in 0..pos.count {
                        let p = Vec3::new(read_f32(&pos, i, 0), read_f32(&pos, i, 1), read_f32(&pos, i, 2));
                        mesh.verts.push(mat_apply(&world, p));
                    }
                    match prim["indices"].as_u64() {
                        Some(ii) => {
                            if let Some(ix) = accessor(&doc, bin, ii as usize) {
                                let mut k = 0;
                                while k + 2 < ix.count {
                                    mesh.tris.push([
                                        base + read_index(&ix, k),
                                        base + read_index(&ix, k + 1),
                                        base + read_index(&ix, k + 2),
                                    ]);
                                    k += 3;
                                }
                            }
                        }
                        None => {
                            let mut k = 0;
                            while k + 2 < pos.count {
                                mesh.tris
                                    .push([base + k as u32, base + k as u32 + 1, base + k as u32 + 2]);
                                k += 3;
                            }
                        }
                    }
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
    if mesh.tris.is_empty() {
        return Err("GLB 里没有三角形".to_string());
    }
    Ok(mesh)
}
