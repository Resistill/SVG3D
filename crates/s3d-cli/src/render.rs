//! `s3d render` 与 `s3d compare`：CPU 光栅（s3d_geom::raster / compare）的命令行入口
//! （原型 `tools/agent.mjs` 的 render / compare 命令）。

use crate::abs_path;
use s3d_geom::compare::compare;
use s3d_geom::mesh::{load_parts, Part, Split};
use s3d_geom::raster::{hex_rgb, rasterize, view_dir, view_names, Legend, Mode, Options, RObj};
use s3d_geom::tri::TriParams;
use s3d_geom::{Aabb, Vec3};
use s3d_lang::compile::{compile, CompileOptions, Compiled};
use s3d_lang::items::NodeInfo;
use serde_json::{json, Map, Value as J};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

/// 角色表（原型 roles-view.js 的 ROLE_LABELS 键）
const ROLES: [&str; 6] = ["structure", "decor", "terrain", "decal", "collision", "helper"];
/// 看得见的角色：默认只画这些（碰撞块、辅助部件看不见）
const VISIBLE: [&str; 4] = ["structure", "terrain", "decor", "decal"];

pub const RENDER_USAGE: &str = "s3d render <场景.s3d> [--glb 源.glb] [--view iso|iso2|top|front|back|left|right] \
[--mode color|id|labels|error] [--size 1024] [--out 文件.png] [--source] [--only 角色,…] [--box x0,y0,z0,x1,y1,z1] \
[--ortho] [--cut h] [--elev 度] [--segments prototype] [--delta d]
  默认 --view iso --mode labels --out renders/<视角>-<模式>.png；--source 画源模型（按部件材质色）而不是重建；
  --only 只画这些角色（默认 structure,terrain,decor,decal；源部件按默认规则判角色，重建按 ~role 沿路径向上找）；
  --mode error 先按默认常数（可给 --delta）算一次 score，按逐物体（--source 时逐源部件）的 meanErr / 4δ 从绿到红上色；
  stdout 是 JSON：{file, view, mode, box, legend}（box 可原样传回 --box，两张图取景一致）";

pub const COMPARE_USAGE: &str = "s3d compare <场景.s3d> --glb 源.glb [--views iso,iso2,top] [--size 512] [--blur r] \
[--meshes] [--only 角色,…] [--box x0,y0,z0,x1,y1,z1] [--out-dir renders] [--segments prototype]
  源模型与重建同一取景（默认源模型的包围盒）各渲一张、模糊（默认半径 size/64）后比：missing = 源有而重建没有的占比，
  extra = 重建多出的占比，diff = 模糊后的灰度差（0–1）。重建默认不画网格引用（--meshes 才画）。
  差异图 <out-dir>/compare-<视角>[-meshes][-角色+…].png：红 = 缺，蓝 = 多，灰 = 两边都有";

struct Args<'a>(&'a [String]);

impl Args<'_> {
    fn opt(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .position(|a| a == name)
            .and_then(|i| self.0.get(i + 1))
            .cloned()
    }
    fn flag(&self, name: &str) -> bool {
        self.0.iter().any(|a| a == name)
    }
    fn num(&self, name: &str) -> Result<Option<f64>, String> {
        match self.opt(name) {
            None => Ok(None),
            Some(v) => v
                .parse::<f64>()
                .ok()
                .filter(|x| x.is_finite())
                .map(Some)
                .ok_or_else(|| format!("{} 要一个数（收到 {}）", name, v)),
        }
    }
    fn only(&self) -> Result<Option<Vec<String>>, String> {
        let Some(s) = self.opt("--only") else {
            return Ok(None);
        };
        let v: Vec<String> = s
            .split(',')
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .collect();
        for r in &v {
            if !ROLES.contains(&r.as_str()) {
                return Err(format!("没有角色 {}（可用：{}）", r, ROLES.join(" / ")));
            }
        }
        Ok(Some(v))
    }
    fn frame(&self) -> Result<Option<Aabb>, String> {
        let Some(s) = self.opt("--box") else {
            return Ok(None);
        };
        let v: Vec<f64> = s
            .split(',')
            .filter_map(|x| x.trim().parse::<f64>().ok())
            .collect();
        if v.len() != 6 || v.iter().any(|x| !x.is_finite()) {
            return Err("box 写成 x0,y0,z0,x1,y1,z1".into());
        }
        Ok(Some(Aabb::new(
            Vec3::new(v[0], v[1], v[2]),
            Vec3::new(v[3], v[4], v[5]),
        )))
    }
    fn segments(&self) -> TriParams {
        if self.opt("--segments").as_deref() == Some("prototype") {
            TriParams::PROTOTYPE
        } else {
            TriParams::SPEC
        }
    }
}

fn keep(only: &Option<Vec<String>>, role: &str) -> bool {
    match only {
        Some(v) => v.iter().any(|r| r == role),
        None => VISIBLE.contains(&role),
    }
}

/// 节点的 `~role` 沿路径向上继承，没有就是 structure（与 s3d_score::score 的读法相同）
fn role_lookup(nodes: &[NodeInfo]) -> impl Fn(&str) -> String + '_ {
    let m: BTreeMap<&str, Option<&str>> = nodes
        .iter()
        .map(|n| (n.path.as_str(), n.meta_role.as_deref()))
        .collect();
    move |path: &str| {
        let mut p = path.to_string();
        loop {
            if let Some(Some(r)) = m.get(p.as_str()) {
                return r.to_string();
            }
            match p.rfind('.') {
                Some(cut) => p.truncate(cut),
                None => return "structure".to_string(),
            }
        }
    }
}

fn compile_file(file: &str) -> Result<Compiled, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("读不到 {}：{}", file, e))?;
    let display = Path::new(file)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(file.to_string());
    let read = |p: &str| std::fs::read_to_string(p).ok();
    Ok(compile(
        &text,
        &abs_path(file),
        &display,
        &read,
        &CompileOptions::default(),
    ))
}

/// 读源模型，按默认规则判角色（已有角色的不动）
fn load_source(glb: Option<String>) -> Result<Vec<Part>, String> {
    let Some(g) = glb else {
        return Ok(Vec::new());
    };
    let bytes = std::fs::read(&g).map_err(|e| format!("读不到 {}：{}", g, e))?;
    let mut parts = load_parts(&bytes, Split::Auto)?;
    let roles = s3d_score::roles::assign_roles(&parts, &[]);
    for (p, r) in parts.iter_mut().zip(roles) {
        p.role.get_or_insert(r);
    }
    Ok(parts)
}

fn part_role(p: &Part) -> &str {
    p.role.as_deref().unwrap_or("structure")
}

fn write_file(path: &str, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = Path::new(path).parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir).map_err(|e| format!("建不了目录 {}：{}", dir.display(), e))?;
        }
    }
    std::fs::write(path, bytes).map_err(|e| format!("写不了 {}：{}", path, e))
}

fn r4(x: f64) -> f64 {
    (x * 1e4 + 0.5).floor() / 1e4
}

fn box_str(b: &Aabb) -> String {
    [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z]
        .iter()
        .map(|v| format!("{}", r4(*v)))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn render(args: &[String]) -> Result<J, String> {
    let a = Args(args);
    let file = &args[1];
    let only = a.only()?;
    let view = a.opt("--view").unwrap_or_else(|| "iso".into());
    if view_dir(&view).is_none() {
        return Err(format!("没有视角 {}（可用：{}）", view, view_names().join(" ")));
    }
    let mode_s = a.opt("--mode").unwrap_or_else(|| "labels".into());
    let mode = Mode::parse(&mode_s)
        .ok_or_else(|| format!("没有模式 {}（可用：color / id / labels / error）", mode_s))?;
    let size = a.num("--size")?.unwrap_or(1024.0).max(1.0) as usize;
    let frame = a.frame()?;
    let tp = a.segments();
    let source = a.flag("--source");
    let parts = load_source(a.opt("--glb"))?;
    let mut out = compile_file(file)?;
    let items = s3d_lang::items::collect_item_trees(&mut out);
    let nodes = s3d_lang::items::node_infos(&out);

    // error 模式：逐物体 / 逐源部件的 meanErr，以 4δ 为满刻度
    let mut heat: BTreeMap<String, f64> = BTreeMap::new();
    if mode == Mode::Error {
        let files = s3d_lang::items::source_files(&out);
        let edit = s3d_score::edit::edit_report(&out, &nodes);
        let mut consts = s3d_score::score::Consts::default();
        consts.delta = a.num("--delta")?.unwrap_or(consts.delta);
        consts.tri = tp;
        let full = 4.0 * consts.delta;
        let input = s3d_score::score::Input {
            items: &items,
            nodes: &nodes,
            files: &files,
            parts: &parts,
            consts,
            edit: Some(&edit),
        };
        let sc = s3d_score::score::score(&input, None);
        let (list, key) = if source {
            ("sources", "name")
        } else {
            ("objects", "path")
        };
        for x in sc.get(list).and_then(|v| v.as_array()).into_iter().flatten() {
            if let (Some(k), Some(e)) = (
                x.get(key).and_then(|v| v.as_str()),
                x.get("meanErr").and_then(|v| v.as_f64()),
            ) {
                heat.insert(k.to_string(), e / full);
            }
        }
    }

    let objs: Vec<RObj> = if source {
        parts
            .iter()
            .enumerate()
            .filter(|(_, p)| keep(&only, part_role(p)))
            .map(|(i, p)| {
                let path = p.label(i);
                RObj {
                    heat: heat.get(&path).copied(),
                    path,
                    tris: Cow::Borrowed(&p.tris[..]),
                    color: hex_rgb(&p.material.color),
                }
            })
            .collect()
    } else {
        let role_of = role_lookup(&nodes);
        s3d_score::objects::scene_objects_with(&items, &parts, &tp)
            .into_iter()
            .filter(|o| keep(&only, &role_of(&o.path)))
            .map(|o| RObj {
                heat: heat.get(&o.path).copied(),
                color: hex_rgb(&o.color),
                path: o.path,
                tris: Cow::Owned(o.tris),
            })
            .collect()
    };
    let res = rasterize(
        &objs,
        &Options {
            view: view.clone(),
            size,
            mode,
            frame,
            ortho: a.flag("--ortho"),
            cut: a.num("--cut")?,
            elev: a.num("--elev")?,
        },
    );
    let path = a
        .opt("--out")
        .unwrap_or_else(|| format!("renders/{}-{}.png", view, mode_s));
    write_file(&path, &res.image.png())?;
    let legend: Vec<J> = res
        .legend
        .iter()
        .map(|l| match l {
            Legend::Color { color, path } => json!({ "color": color, "path": path }),
            Legend::Label {
                n,
                path,
                pixels,
                labeled,
            } => {
                let mut m = json!({ "n": n, "path": path, "pixels": pixels });
                if !labeled {
                    m["labeled"] = J::Bool(false);
                }
                m
            }
        })
        .collect();
    Ok(json!({
        "file": path, "view": view, "mode": mode_s, "objects": objs.len(),
        "box": box_str(&res.frame), "legend": legend,
    }))
}

pub fn compare_cmd(args: &[String]) -> Result<J, String> {
    let a = Args(args);
    let file = &args[1];
    let glb = a.opt("--glb").ok_or("compare 要 --glb 源模型")?;
    let only = a.only()?;
    let meshes = a.flag("--meshes");
    let size = a.num("--size")?.unwrap_or(512.0).max(1.0) as usize;
    let blur = match a.num("--blur")? {
        Some(b) => b,
        None => size as f64 / 64.0,
    };
    let blur = ((blur + 0.5).floor() as i64).max(1) as usize;
    let views: Vec<String> = a
        .opt("--views")
        .unwrap_or_else(|| "iso,iso2,top".into())
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    for v in &views {
        if view_dir(v).is_none() {
            return Err(format!("没有视角 {}（可用：{}）", v, view_names().join(" ")));
        }
    }
    let frame = a.frame()?;
    let tp = a.segments();
    let out_dir = a.opt("--out-dir").unwrap_or_else(|| "renders".into());
    let parts = load_source(Some(glb))?;
    let mut out = compile_file(file)?;
    let items = s3d_lang::items::collect_item_trees(&mut out);
    let nodes = s3d_lang::items::node_infos(&out);
    let role_of = role_lookup(&nodes);
    let grey = [0xc9, 0xc9, 0xc9];
    let src: Vec<RObj> = parts
        .iter()
        .enumerate()
        .filter(|(_, p)| keep(&only, part_role(p)))
        .map(|(i, p)| RObj {
            path: p.label(i),
            tris: Cow::Borrowed(&p.tris[..]),
            color: grey,
            heat: None,
        })
        .collect();
    let rec: Vec<RObj> = s3d_score::objects::scene_objects_with(&items, &parts, &tp)
        .into_iter()
        .filter(|o| !o.tris.is_empty() && (meshes || o.mesh.is_none()) && keep(&only, &role_of(&o.path)))
        .map(|o| RObj {
            path: o.path,
            tris: Cow::Owned(o.tris),
            color: grey,
            heat: None,
        })
        .collect();
    let (res, fr) = compare(&src, &rec, &views, size, blur, frame);
    let mut vm = Map::new();
    for v in &res {
        let name = format!(
            "compare-{}{}{}.png",
            v.view,
            if meshes { "-meshes" } else { "" },
            only.as_ref()
                .map(|o| format!("-{}", o.join("+")))
                .unwrap_or_default()
        );
        let path = Path::new(&out_dir)
            .join(name)
            .to_string_lossy()
            .replace('\\', "/");
        write_file(&path, &v.image.png())?;
        vm.insert(
            v.view.clone(),
            json!({ "missing": r4(v.missing), "extra": r4(v.extra), "diff": r4(v.diff), "file": path }),
        );
    }
    Ok(json!({
        "meshes": meshes, "only": only, "blur": blur,
        "box": fr.map(|b| box_str(&b)), "sources": src.len(), "objects": rec.len(),
        "views": J::Object(vm),
    }))
}
