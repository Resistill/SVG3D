//! 真值评测（原型 `tools/eval.mjs` 的移植；指标定义见 docs/PLAN.md 第 2 节"真值评测的指标定义"）。
//!
//! 把原始描述与逆向描述各自编译（都附加同一份通用语义库），按世界紧包围盒的 IoU 贪心一一匹配对象，然后比较：
//! 几何（匹配率、平均 IoU、形状种类正确率）、关系（事实的 P / R / F1，逆向侧按匹配换名）、
//! 压缩召回（阵列 / 镜像）、语义（库里每个概念的成员 P / R / F1）、描述质量（坐标依赖率、锚点率、语句数比）。

use indexmap::IndexMap;
use regex::Regex;
use s3d_lang::ast::StmtKind;
use s3d_lang::compile::{compile, CompileOptions, Compiled};
use s3d_lang::program::Reader;
use s3d_lang::scene::{Role, ROOT};
use serde_json::{json, Map, Value as J};
use std::collections::{BTreeMap, BTreeSet};

/// 一侧的输入：源码文本、可显示路径、规范化绝对路径（供 import 解析）
pub struct Side<'a> {
    pub text: &'a str,
    pub identity: &'a str,
    pub display: &'a str,
}

struct Obj {
    path: String,
    cls: &'static str,
    box_: [f64; 6],
    array: bool,
    mirror: bool,
}

fn round3(x: f64) -> f64 {
    format!("{:.3}", x).parse::<f64>().unwrap_or(x)
}

fn iou(a: &[f64; 6], b: &[f64; 6]) -> f64 {
    let mut inter = 1.0;
    for k in 0..3 {
        let ov = (a[k + 3].min(b[k + 3]) - a[k].max(b[k])).max(0.0);
        inter *= ov;
    }
    let vol = |x: &[f64; 6]| (0..3).fold(1.0, |s, k| s * (x[k + 3] - x[k]).max(1e-9));
    inter / (vol(a) + vol(b) - inter)
}

fn class_of(item: &J) -> &'static str {
    if item["kind"] == "csg" {
        return "complex";
    }
    let g = &item["geom"];
    match g["type"].as_str().unwrap_or("") {
        "box" | "beam" => "box",
        "wedge" => "wedge",
        "sphere" => "sphere",
        "cyl" => {
            if (g["r1"].as_f64().unwrap_or(0.0) - g["r2"].as_f64().unwrap_or(0.0)).abs() < 1e-6 {
                "cyl"
            } else {
                "cone"
            }
        }
        "rod" => "cyl",
        "mesh" => "mesh",
        _ => "complex",
    }
}

fn aabb6(v: &J) -> Option<[f64; 6]> {
    let mn = v.get("min")?.as_array()?;
    let mx = v.get("max")?.as_array()?;
    Some([
        mn[0].as_f64()?,
        mn[1].as_f64()?,
        mn[2].as_f64()?,
        mx[0].as_f64()?,
        mx[1].as_f64()?,
        mx[2].as_f64()?,
    ])
}

/// 一侧的对象列表（原型 objectsOf）：items 的最外层条目，盒取 tight（没有则 box）；
/// `array` = 在阵列实例 / rotate 副本里；`mirror` = 在 mirror 副本里（含副本式节点自身与原件）
fn objects_of(c: &mut Compiled) -> (J, Vec<Obj>) {
    let out = s3d_lang::output::to_json(c);
    let comp = &c.compiler;
    let mut by_path: BTreeMap<String, &J> = BTreeMap::new();
    for n in out["nodes"].as_array().unwrap() {
        by_path.insert(n["path"].as_str().unwrap().to_string(), n);
    }
    // 副本式 mirror 节点（`mirror R of=…`）的路径；容器式 rotate 节点的路径
    let mut mirror_twins: Vec<String> = Vec::new();
    let mut rot_roots: Vec<String> = Vec::new();
    for id in comp.scene.order() {
        let node = &comp.scene.nodes[id];
        if comp.scene.in_skin(id) {
            continue;
        }
        if let Role::TwinOf { .. } = node.role {
            let is_mirror = matches!(node.src_stmt.as_ref().map(|s| &s.kind), Some(StmtKind::Generic { kind, .. }) if kind == "mirror");
            if is_mirror {
                mirror_twins.push(comp.scene.path(id));
            }
        } else if node.kind == "rotate" && matches!(node.role, Role::Normal) {
            rot_roots.push(comp.scene.path(id));
        }
    }
    let seg_re = Regex::new(r"(^|\.)\d+(\.|$)|\.r\d+(\.|$)").unwrap();
    let mut objs = Vec::new();
    for it in out["items"].as_array().unwrap() {
        let path = it["path"].as_str().unwrap_or("").to_string();
        let node = match by_path.get(&path) {
            Some(n) => *n,
            None => continue,
        };
        let b = match aabb6(&node["tight"]).or_else(|| aabb6(&node["box"])) {
            Some(b) => b,
            None => continue,
        };
        let array = seg_re.is_match(&path) || rot_roots.iter().any(|g| path.starts_with(&format!("{}.", g)));
        let mirror = mirror_twins
            .iter()
            .any(|g| path == *g || path.starts_with(&format!("{}.", g)))
            || path.contains(".mirrored.")
            || path.ends_with(".mirrored");
        objs.push(Obj {
            path,
            cls: class_of(it),
            box_: b,
            array,
            mirror,
        });
    }
    // 镜像没有方向：副本的原件也算处在镜像关系里
    let sources: BTreeSet<String> = objs
        .iter()
        .filter(|o| o.mirror)
        .filter_map(|o| {
            by_path
                .get(&o.path)
                .and_then(|n| n["of"].as_str().map(str::to_string))
        })
        .collect();
    for o in objs.iter_mut() {
        if sources.contains(&o.path) {
            o.mirror = true;
        }
    }
    let _ = ROOT;
    (out, objs)
}

fn prf(truth: &BTreeSet<String>, mapped: &BTreeSet<String>, filter: &dyn Fn(&str) -> bool) -> J {
    let t: Vec<&String> = truth.iter().filter(|x| filter(x)).collect();
    let m: Vec<&String> = mapped.iter().filter(|x| filter(x)).collect();
    let tp = m.iter().filter(|x| truth.contains(**x)).count();
    let p = if m.is_empty() {
        1.0
    } else {
        tp as f64 / m.len() as f64
    };
    let r = if t.is_empty() {
        1.0
    } else {
        tp as f64 / t.len() as f64
    };
    let f1 = if p + r > 0.0 { 2.0 * p * r / (p + r) } else { 0.0 };
    json!({ "truth": t.len(), "found": m.len(), "precision": round3(p), "recall": round3(r), "f1": round3(f1) })
}

/// 展开 import 后的非空、非注释行数（原型 eval 的 lines）
fn count_lines(text: &str, identity: &str, read: Reader<'_>, seen: &mut BTreeSet<String>) -> usize {
    let imp = Regex::new(r#"^\s*import\s+"([^"]+)""#).unwrap();
    let mut n = 0;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        if let Some(c) = imp.captures(line) {
            let rel = &c[1];
            let dir = match identity.rfind('/') {
                Some(i) => &identity[..=i],
                None => "",
            };
            if let Ok(p) = s3d_lang::program::join_path(dir, rel) {
                if seen.insert(p.clone()) {
                    if let Some(sub) = read(&p) {
                        n += count_lines(&sub, &p, read, seen);
                    }
                }
            }
            continue;
        }
        n += 1;
    }
    n
}

/// 评测：返回与原型 `bin/eval.mjs --json` 同形的 JSON
pub fn evaluate(orig: Side<'_>, recon: Side<'_>, lib: &str, read: Reader<'_>) -> J {
    let with_lib = |t: &str| format!("{}\n{}", t, lib);
    let opts = CompileOptions::default();
    let mut a = compile(&with_lib(orig.text), orig.identity, orig.display, read, &opts);
    let mut b = compile(&with_lib(recon.text), recon.identity, recon.display, read, &opts);
    let (ja, oa) = objects_of(&mut a);
    let (jb, ob) = objects_of(&mut b);
    // 贪心匹配：IoU 从高到低（并列按枚举顺序：a 外层、b 内层）
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for (i, x) in oa.iter().enumerate() {
        for (j, y) in ob.iter().enumerate() {
            let s = iou(&x.box_, &y.box_);
            if s > 0.3 {
                pairs.push((s, i, j));
            }
        }
    }
    pairs.sort_by(|x, y| {
        y.0.partial_cmp(&x.0)
            .unwrap()
            .then(x.1.cmp(&y.1))
            .then(x.2.cmp(&y.2))
    });
    let mut m_a: IndexMap<usize, (usize, f64)> = IndexMap::new();
    let mut m_b: BTreeMap<String, String> = BTreeMap::new();
    for (s, i, j) in pairs {
        if !m_a.contains_key(&i) && !m_b.contains_key(&ob[j].path) {
            m_a.insert(i, (j, s));
            m_b.insert(ob[j].path.clone(), oa[i].path.clone());
        }
    }
    let fittable: Vec<usize> = (0..oa.len())
        .filter(|&i| matches!(oa[i].cls, "box" | "wedge" | "sphere" | "cyl" | "cone"))
        .collect();
    let kind_ok = fittable
        .iter()
        .filter(|&&i| m_a.get(&i).map(|(j, _)| ob[*j].cls == oa[i].cls).unwrap_or(false))
        .count();
    let complex: Vec<usize> = (0..oa.len()).filter(|&i| oa[i].cls == "complex").collect();
    let complex_as_mesh = complex
        .iter()
        .filter(|&&i| m_a.get(&i).map(|(j, _)| ob[*j].cls == "mesh").unwrap_or(false))
        .count();
    let complex_described = complex
        .iter()
        .filter(|&&i| m_a.get(&i).map(|(j, _)| ob[*j].cls == "complex").unwrap_or(false))
        .count();
    let mut confusion: IndexMap<String, usize> = IndexMap::new();
    for &i in &fittable {
        let to = m_a.get(&i).map(|(j, _)| ob[*j].cls).unwrap_or("无");
        *confusion.entry(format!("{}→{}", oa[i].cls, to)).or_insert(0) += 1;
    }
    // 关系
    let key = |f: &J, a: &str, b: &str| format!("{}|{}|{}", f["rel"].as_str().unwrap_or(""), a, b);
    let truth: BTreeSet<String> = ja["facts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| key(f, f["a"].as_str().unwrap_or(""), f["b"].as_str().unwrap_or("")))
        .collect();
    let mapped: BTreeSet<String> = jb["facts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| {
            let a = m_b.get(f["a"].as_str().unwrap_or(""))?;
            let b = m_b.get(f["b"].as_str().unwrap_or(""))?;
            Some(key(f, a, b))
        })
        .collect();
    let rels = ["on", "against", "inside", "touching", "intersects"];
    let mut facts = Map::new();
    facts.insert("all".into(), prf(&truth, &mapped, &|_| true));
    for r in rels {
        let pre = format!("{}|", r);
        facts.insert(r.into(), prf(&truth, &mapped, &|x| x.starts_with(&pre)));
    }
    facts.insert(
        "onAgainst".into(),
        prf(&truth, &mapped, &|x| {
            x.starts_with("on|") || x.starts_with("against|")
        }),
    );
    // 压缩召回
    let recall = |flag: &dyn Fn(&Obj) -> bool| -> J {
        let t: Vec<usize> = (0..oa.len()).filter(|&i| flag(&oa[i])).collect();
        let hit = t
            .iter()
            .filter(|&&i| m_a.get(&i).map(|(j, _)| flag(&ob[*j])).unwrap_or(false))
            .count();
        json!({ "orig": t.len(), "recovered": hit, "recall": round3(hit as f64 / (t.len().max(1)) as f64) })
    };
    let compression = json!({ "array": recall(&|o| o.array), "mirror": recall(&|o| o.mirror) });
    // 语义：库里每个概念
    let lib_re = Regex::new(r"(?m)^\s*concept\s+([\p{L}_][\p{L}\p{N}_]*)").unwrap();
    let lib_names: Vec<String> = lib_re.captures_iter(lib).map(|c| c[1].to_string()).collect();
    let is_obj: BTreeSet<&str> = oa.iter().map(|o| o.path.as_str()).collect();
    let members_of = |j: &J, name: &str| -> Vec<String> {
        j["concepts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .and_then(|c| c["members"].as_array())
            .map(|m| m.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let mut semantic = Map::new();
    let (mut stp, mut sfound, mut struth) = (0usize, 0usize, 0usize);
    for name in &lib_names {
        let t: BTreeSet<String> = members_of(&ja, name)
            .into_iter()
            .filter(|p| is_obj.contains(p.as_str()))
            .collect();
        let m: BTreeSet<String> = members_of(&jb, name)
            .into_iter()
            .filter_map(|p| m_b.get(&p).cloned())
            .collect();
        let tp = m.iter().filter(|x| t.contains(*x)).count();
        stp += tp;
        sfound += m.len();
        struth += t.len();
        if t.is_empty() && m.is_empty() {
            continue;
        }
        let p = if m.is_empty() {
            1.0
        } else {
            tp as f64 / m.len() as f64
        };
        let r = if t.is_empty() {
            1.0
        } else {
            tp as f64 / t.len() as f64
        };
        let f1 = if p + r > 0.0 { 2.0 * p * r / (p + r) } else { 0.0 };
        semantic.insert(
            name.clone(),
            json!({ "truth": t.len(), "found": m.len(), "precision": round3(p), "recall": round3(r), "f1": round3(f1) }),
        );
    }
    {
        let p = if sfound == 0 {
            1.0
        } else {
            stp as f64 / sfound as f64
        };
        let r = if struth == 0 {
            1.0
        } else {
            stp as f64 / struth as f64
        };
        let f1 = if p + r > 0.0 { 2.0 * p * r / (p + r) } else { 0.0 };
        semantic.insert(
            "all".into(),
            json!({ "truth": struth, "found": sfound, "precision": round3(p), "recall": round3(r), "f1": round3(f1) }),
        );
    }
    // 描述质量：只看逆向原文的最外层生成节点的语句
    let top_re = Regex::new(r"^(box|cyl|cone|sphere|wedge|plate|hull|mesh|revolve|span|sweep|pipe|repeat|mirror|rotate|copy|group)\s").unwrap();
    let coord_re = Regex::new(r"\sat=[-\d(.]").unwrap();
    let anchor_re = Regex::new(r"\sat=[\p{L}_]").unwrap();
    let top: Vec<&str> = recon.text.lines().filter(|l| top_re.is_match(l)).collect();
    let coord = top.iter().filter(|l| coord_re.is_match(l)).count();
    let anchor = top.iter().filter(|l| anchor_re.is_match(l)).count();
    let denom = top.len().max(1) as f64;
    let lines_recon = count_lines(recon.text, recon.identity, read, &mut BTreeSet::new());
    let lines_orig = count_lines(orig.text, orig.identity, read, &mut BTreeSet::new());
    let matched = m_a.len();
    let mean_iou = m_a.values().map(|(_, s)| *s).sum::<f64>() / (matched.max(1)) as f64;
    json!({
        "errors": { "orig": ja["errors"].as_array().map(|a| a.len()).unwrap_or(0), "recon": jb["errors"].as_array().map(|a| a.len()).unwrap_or(0) },
        "objects": { "orig": oa.len(), "recon": ob.len(), "matched": matched, "meanIoU": round3(mean_iou) },
        "kinds": {
            "fittable": fittable.len(), "correct": kind_ok, "accuracy": round3(kind_ok as f64 / (fittable.len().max(1)) as f64),
            "complex": complex.len(), "complexAsMesh": complex_as_mesh, "complexDescribed": complex_described,
            "confusion": J::Object(confusion.iter().map(|(k, v)| (k.clone(), json!(v))).collect()),
        },
        "facts": facts,
        "compression": compression,
        "semantic": semantic,
        "description": {
            "coordinateRatio": round3(coord as f64 / denom),
            "anchorRatio": round3(anchor as f64 / denom),
            "statementRatio": round3(lines_recon as f64 / (lines_orig.max(1)) as f64),
        },
    })
}
