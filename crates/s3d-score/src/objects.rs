//! 编译结果 → 每个物体的世界三角形（原型 `tools/worldtris.mjs`）。
//! mesh 引用用源模型里对应部件的真实三角形（源部件已在世界坐标里，按 W · T(−pivot) 搬到引用处）；
//! 找不到源部件时用外包络盒代替并标记 placeholder。布尔容器按 2.5.3 做网格布尔。

use s3d_geom::csg::{boolean, Op};
use s3d_geom::mesh::Part;
use s3d_geom::tri::{transform_tris, triangulate_with, Tri, TriParams};
use s3d_geom::{Solid, Transform};
use s3d_lang::items::ItemTree;
use std::collections::BTreeMap;

/// 网格引用的信息
#[derive(Clone, Debug)]
pub struct MeshRef {
    /// 引用的源部件名（`#` 之后）
    pub src: String,
    /// 源部件序号（找到时）
    pub part: Option<usize>,
    /// 享元键：源部件的网格编号 `m:<mesh>`，否则部件名
    pub key: String,
    pub tris: usize,
    pub placeholder: bool,
    pub role: Option<String>,
}

/// 一个重建物体
#[derive(Clone, Debug)]
pub struct Obj {
    pub path: String,
    pub tris: Vec<Tri>,
    pub color: String,
    pub mesh: Option<MeshRef>,
}

/// 源部件按名字查：唯一标识（名字@k）优先；只给名字时取第一个同名的；分量另有 `from`（`#` 也可写成 `~`）
pub fn source_index(parts: &[Part]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for (i, p) in parts.iter().enumerate() {
        if let Some(u) = &p.uid {
            m.insert(u.clone(), i);
        }
        if let Some(n) = &p.name {
            m.entry(n.clone()).or_insert(i);
        }
        if let Some(f) = &p.from {
            m.insert(f.clone(), i);
            m.insert(f.replace('#', "~"), i);
        }
    }
    m
}

fn mesh_tris(
    solid: &Solid,
    world: &Transform,
    parts: &[Part],
    by_name: &BTreeMap<String, usize>,
    tp: &TriParams,
) -> (Vec<Tri>, Option<MeshRef>) {
    match solid {
        Solid::Mesh { pivot, src, .. } => {
            let name = src.rsplit('#').next().unwrap_or(src).to_string();
            match by_name.get(&name) {
                Some(&i) => {
                    let sp = &parts[i];
                    let w = world.then_inner(&Transform::translation(-*pivot));
                    let tris = transform_tris(&sp.tris, &w);
                    let key = sp
                        .mesh
                        .as_ref()
                        .map(|m| format!("m:{}", m))
                        .unwrap_or_else(|| name.clone());
                    (
                        tris,
                        Some(MeshRef {
                            src: name,
                            part: Some(i),
                            key,
                            tris: sp.tris.len(),
                            placeholder: false,
                            role: sp.role.clone(),
                        }),
                    )
                }
                None => (
                    transform_tris(&triangulate_with(solid, tp), world),
                    Some(MeshRef {
                        src: name.clone(),
                        part: None,
                        key: name,
                        tris: 0,
                        placeholder: true,
                        role: None,
                    }),
                ),
            }
        }
        _ => (transform_tris(&triangulate_with(solid, tp), world), None),
    }
}

fn csg_tris(t: &ItemTree, parts: &[Part], by_name: &BTreeMap<String, usize>, tp: &TriParams) -> Vec<Tri> {
    match t {
        ItemTree::Solid(s) => mesh_tris(&s.solid, &s.world, parts, by_name, tp).0,
        ItemTree::Csg { op, children, .. } => {
            let op = match op.as_str() {
                "subtract" => Op::Subtract,
                "intersect" => Op::Intersect,
                _ => Op::Union,
            };
            let mut acc: Option<Vec<Tri>> = None;
            for c in children {
                let b = csg_tris(c, parts, by_name, tp);
                acc = Some(match acc {
                    None => b,
                    Some(a) => boolean(&a, &b, op),
                });
            }
            acc.unwrap_or_default()
        }
    }
}

/// 全部条目的世界三角形（没有三角形的条目也保留，调用方按需过滤）；分段按规范
pub fn scene_objects(items: &[ItemTree], parts: &[Part]) -> Vec<Obj> {
    scene_objects_with(items, parts, &TriParams::SPEC)
}

/// 同上，分段按 `tp`
pub fn scene_objects_with(items: &[ItemTree], parts: &[Part], tp: &TriParams) -> Vec<Obj> {
    let by_name = source_index(parts);
    items
        .iter()
        .map(|it| match it {
            ItemTree::Solid(s) => {
                let (tris, mesh) = mesh_tris(&s.solid, &s.world, parts, &by_name, tp);
                Obj {
                    path: s.path.clone(),
                    tris,
                    color: s.color.clone(),
                    mesh,
                }
            }
            ItemTree::Csg { path, color, .. } => Obj {
                path: path.clone(),
                tris: csg_tris(it, parts, &by_name, tp),
                color: color.clone(),
                mesh: None,
            },
        })
        .collect()
}
