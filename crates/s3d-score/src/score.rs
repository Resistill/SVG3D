//! 自评分数（原型 `tools/score.mjs` 的移植；docs/PLAN.md 第 3、6 节）：
//!   S = L_stmt + L_mesh + R_src + R_recon + R_vol（E_edit 另报），越低越好
//!   L_stmt  描述长度：名字、关键字、引用各 1；数字 1 + 0.5 × (小数位 − 1)；注释与 ~元数据不计
//!   L_mesh  仍以源网格引用保留的部分：按角色 full（4.5 × 顶点数 + 3 × 三角形数）或 ref（c_mesh × log2(1 + 三角形数)）；
//!           同一个网格（享元）只在第一次出现时付
//!   R_src   源部件表面采样到重建表面的距离 d → log2(1 + max(0, d − δ)/δ)，按 c_res × log2(1 + 面积/δ²) 加权；
//!           距离优先量到同材质的重建表面，没有时量到全部表面并多计一个 δ
//!   R_recon 重建物体表面到源表面，同样计价；权重以顶层语句为单位、内部按面积分摊
//!   R_vol   重建实体里、源实体外的内部采样点，权重 c_res × log2(1 + 体积/δ³)，以顶层语句为单位、内部按体积分摊
//! 解释率 explain = 1 − S / S_mesh，S_mesh 是"全部写成 mesh 引用、世界坐标定位"的基线。
//! 长度常数按米给出，unit 是每个场景单位多少米；计算前换算成场景单位，输出的误差再换回米。
//!
//! 增量评分（`ScoreCache`）：同一组源部件上反复评分时只重算变了的部分，结果与全量一致。源侧固定的东西只算一次；
//! R_recon 每个重建物体的残差按"路径 + 颜色 + 几何哈希"缓存；R_src 每个源部件逐点记距离与最近的物体，
//! 删掉的物体只影响以它为最近表面的点，新增的物体只影响比现在的距离离它更近的点；某种颜色整组出现或消失时
//! 这种颜色的源部件全部重算；改动超过三成时全部重算。

use crate::edit::EditReport;
use crate::objects::{scene_objects_with, Obj};
use crate::roles::{assign_roles, RoleRule};
use crate::sample::{sample_points, seed_of, source_samples, tri_area, unique_vertices};
use crate::surfaces::{color_key, Surfaces};
use crate::volume::{interior_samples, volume_stats};
use regex::Regex;
use s3d_geom::math::{self, Vec3};
use s3d_geom::mesh::Part;
use s3d_geom::tri::{Tri, TriParams};
use s3d_lang::items::{ItemTree, NodeInfo};
use serde_json::{json, Map, Value as J};
use std::collections::{BTreeMap, BTreeSet};

/// 评分常数（长度按米）
#[derive(Clone, Debug)]
pub struct Consts {
    pub delta: f64,
    pub c_res: f64,
    pub c_mesh: f64,
    pub d_max: f64,
    pub samples: usize,
    pub unit: f64,
    /// 按角色改网格引用的计价：角色 → "full" / "ref"
    pub mesh_price: BTreeMap<String, String>,
    /// E_edit 的权重（每个坐标依赖的物体；M2 默认 0：只报告）
    pub edit_weight: f64,
    /// 工程的角色规则（部件没挂角色时用）
    pub role_rules: Vec<RoleRule>,
    /// 三角化分段（默认规范；对拍原型导出时用 `TriParams::PROTOTYPE`）
    pub tri: TriParams,
}

impl Default for Consts {
    fn default() -> Consts {
        Consts {
            delta: 0.005,
            c_res: 4.0,
            c_mesh: 10.0,
            d_max: 2.0,
            samples: 600,
            unit: 1.0,
            mesh_price: BTreeMap::new(),
            edit_weight: 0.0,
            role_rules: Vec::new(),
            tri: TriParams::SPEC,
        }
    }
}

/// 换算到场景单位后的常数
#[derive(Clone, Copy)]
struct K {
    delta: f64,
    d_max: f64,
    c_res: f64,
    samples: usize,
}

fn mesh_price_mode<'a>(role: Option<&str>, k: &'a Consts) -> &'a str {
    let r = role.unwrap_or("structure");
    match k.mesh_price.get(r) {
        Some(m) => m.as_str(),
        None => {
            if r == "helper" {
                "ref"
            } else {
                "full"
            }
        }
    }
}

/// 网格引用的价钱（L_mesh）：full = 4.5 × 顶点数 + 3 × 三角形数；ref = c_mesh × log2(1 + 三角形数)
pub fn mesh_price(tris: Option<&[Tri]>, role: Option<&str>, k: &Consts) -> f64 {
    let n = tris.map(|t| t.len()).unwrap_or(0);
    if mesh_price_mode(role, k) == "full" && n > 0 {
        return 4.5 * unique_vertices(tris.unwrap()).len() as f64 + 3.0 * n as f64;
    }
    k.c_mesh * math::log2(1.0 + n.max(1) as f64)
}

// ====== 描述长度 ======

/// 一条语句拆成词：方括号、圆括号、引号里的空格不拆
fn split_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut q = false;
    for ch in line.trim().chars() {
        if q {
            cur.push(ch);
            if ch == '"' {
                q = false;
            }
            continue;
        }
        if ch == '"' {
            q = true;
            cur.push(ch);
            continue;
        }
        if ch == '[' || ch == '(' {
            depth += 1;
        }
        if ch == ']' || ch == ')' {
            depth -= 1;
        }
        if ch.is_whitespace() && depth == 0 {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

const PLACE_KEYS: [&str; 12] = [
    "on", "at", "against", "offset", "pin", "align", "turn", "rot", "inside", "gap", "under", "mat",
];

struct Res {
    token: Regex,
    meta: Regex,
    semantic: Regex,
    key_eq: Regex,
    number: Regex,
    not_decl: Regex,
    decl: Regex,
    over_reshape: Regex,
    head2: Regex,
    reshape_skip: Regex,
    open_brace: Regex,
}

fn res() -> &'static Res {
    static CELL: std::sync::OnceLock<Res> = std::sync::OnceLock::new();
    CELL.get_or_init(|| Res {
        token: Regex::new(r#""[^"]*"|-?\d+(?:\.\d+)?(?:e[-+]?\d+)?|[\p{L}_][\p{L}\p{N}_]*"#).unwrap(),
        meta: Regex::new(r#"\s~[\p{L}_][\p{L}\p{N}_]*=("[^"]*"|\S+)"#).unwrap(),
        semantic: Regex::new(r"^\s*(concept|pattern|check|style|variant|import)\b").unwrap(),
        key_eq: Regex::new(r"^([\p{L}_][\p{L}\p{N}_]*)=").unwrap(),
        number: Regex::new(r"\.(\d+)").unwrap(),
        not_decl: Regex::new(
            r"^\s*(material|let|def|over|drop|copy|import|concept|pattern|check|style|variant|\})\b",
        )
        .unwrap(),
        decl: Regex::new(r"^\s*[\p{L}_][\p{L}\p{N}_]*\s+[\p{L}_]").unwrap(),
        over_reshape: Regex::new(r"(?m)^\s*over\s+([\p{L}_][\p{L}\p{N}_]*)\s*:\s*[\p{L}_]").unwrap(),
        head2: Regex::new(r"^([\p{L}_][\p{L}\p{N}_]*)\s+([\p{L}_][\p{L}\p{N}_]*)").unwrap(),
        reshape_skip: Regex::new(r"^(over|def|material|drop)$").unwrap(),
        open_brace: Regex::new(r"\{\s*$").unwrap(),
    })
}

/// 一个数字词的长度：1 + 0.5 × (小数位 − 1)，末尾的 0 不算
pub fn number_cost(tok: &str) -> f64 {
    let d = res()
        .number
        .captures(tok)
        .map(|c| c[1].trim_end_matches('0').len())
        .unwrap_or(0);
    1.0 + 0.5 * (d as f64 - 1.0).max(0.0)
}

/// 一行语句的长度（去掉注释与 ~元数据；语义层与风格层的语句不计）
pub fn line_cost(line: &str) -> f64 {
    let r = res();
    let mut s = r.meta.replace_all(line, "").to_string();
    if let Some(q) = s.find("//") {
        if s[..q].matches('"').count().is_multiple_of(2) {
            s.truncate(q);
        }
    }
    if s.trim().is_empty() || r.semantic.is_match(&s) {
        return 0.0;
    }
    let mut c = 0.0;
    for m in r.token.find_iter(&s) {
        let t = m.as_str();
        let first = t.trim_start_matches('-').chars().next().unwrap_or(' ');
        c += if first.is_ascii_digit() {
            number_cost(t)
        } else {
            1.0
        };
    }
    c
}

/// 被 over X : 种类 换了形的语句能省下的长度：只留名字、定位与材质
pub fn reshape_saving(line: &str) -> f64 {
    let r = res();
    let ts = split_tokens(line);
    let kept: Vec<String> = ts
        .iter()
        .enumerate()
        .filter(|(i, x)| {
            *i < 2
                || x.starts_with('~')
                || r.key_eq
                    .captures(x)
                    .map(|c| PLACE_KEYS.contains(&&c[1]))
                    .unwrap_or(false)
        })
        .map(|(_, x)| x.clone())
        .collect();
    line_cost(line) - line_cost(&kept.join(" "))
}

#[derive(Clone)]
struct LineCost {
    file: String,
    line: usize,
    cost: f64,
}

/// 一段描述（若干文件）的长度；逐行
fn description_length(files: &[(String, String)]) -> (f64, Vec<LineCost>) {
    let mut total = 0.0;
    let mut lines = Vec::new();
    for (file, text) in files {
        for (i, l) in text.split('\n').enumerate() {
            let c = line_cost(l);
            if c != 0.0 {
                total += c;
                lines.push(LineCost {
                    file: file.clone(),
                    line: i + 1,
                    cost: c,
                });
            }
        }
    }
    (total, lines)
}

// ====== 缓存 ======

/// 源侧（只取决于源部件与角色规则）
struct SrcEntry {
    i: usize,
    role: Option<String>,
    skip: bool,
    name: String,
    mesh_key: Option<String>,
    color: String,
    area: f64,
    lo: Vec3,
    hi: Vec3,
}

struct Shared {
    src: Vec<SrcEntry>,
    src_bvh: Surfaces,
    /// 采样点数 → 每个部件的采样点
    samples: BTreeMap<usize, Vec<Vec<Vec3>>>,
    tris_hash: u64,
}

/// 一个源部件的逐点结果与汇总
#[derive(Clone)]
struct PartRes {
    c: f64,
    mean: f64,
    max: f64,
    who: i64,
    owners: Vec<i64>,
    d: Vec<f64>,
    own: Vec<i64>,
}

#[derive(Clone)]
struct ReconRes {
    r: f64,
    mean: f64,
    max: f64,
    area: f64,
    vol: f64,
    rv: f64,
}

#[derive(Clone)]
struct PrevObj {
    j: usize,
    path: String,
    lo: Vec3,
    hi: Vec3,
    color: String,
}

#[derive(Default)]
struct State {
    recon: BTreeMap<String, ReconRes>,
    prev: Option<BTreeMap<String, PrevObj>>,
    src_res: Option<Vec<PartRes>>,
    ids_path: Vec<String>,
    ids_of: BTreeMap<String, usize>,
}

/// 增量评分的缓存：同一组源部件上反复评分时复用
#[derive(Default)]
pub struct ScoreCache {
    shared: Option<Shared>,
    states: BTreeMap<String, State>,
}

fn fnv_tris(parts: &[Part]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for p in parts {
        for t in &p.tris {
            for v in t {
                for x in [v.x, v.y, v.z] {
                    h = (h ^ x.to_bits()).wrapping_mul(0x100000001b3);
                }
            }
        }
        h = (h ^ p.tris.len() as u64).wrapping_mul(0x100000001b3);
    }
    h
}

fn hash_tris(tris: &[Tri]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for t in tris {
        for v in t {
            for x in [v.x, v.y, v.z] {
                h = (h ^ x.to_bits()).wrapping_mul(0x100000001b3);
            }
        }
    }
    format!("{:x}:{}", h, tris.len() * 9)
}

fn bbox_tris(tris: &[Tri]) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for t in tris {
        for v in t {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
    }
    (lo, hi)
}

fn boxes_meet(a: (Vec3, Vec3), b: (Vec3, Vec3), pad: f64) -> bool {
    (0..3).all(|j| a.0[j] - pad <= b.1[j] && b.0[j] <= a.1[j] + pad)
}

fn box_dist(p: Vec3, b: (Vec3, Vec3)) -> f64 {
    let q = p.clamp(b.0, b.1) - p;
    math::sqrt(q.length_squared())
}

fn shared_of(parts: &[Part], consts: &Consts, cache: Option<&mut ScoreCache>) -> Shared {
    let h = fnv_tris(parts);
    if let Some(c) = &cache {
        if let Some(s) = &c.shared {
            if s.tris_hash == h && s.src.len() == parts.len() {
                // 复用：把 shared 搬出来（调用方用完放回）
            }
        }
    }
    let _ = cache;
    let roles = if parts.iter().all(|p| p.role.is_some()) {
        None
    } else {
        Some(assign_roles(parts, &consts.role_rules))
    };
    let src: Vec<SrcEntry> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (lo, hi) = bbox_tris(&p.tris);
            let role = p.role.clone().or_else(|| roles.as_ref().map(|r| r[i].clone()));
            SrcEntry {
                i,
                skip: role.as_deref() == Some("helper"),
                role,
                name: p.label(i),
                mesh_key: p.mesh.as_ref().map(|m| format!("m:{}", m)),
                color: p.material.color.clone(),
                area: p.tris.iter().map(tri_area).sum(),
                lo,
                hi,
            }
        })
        .collect();
    let objs: Vec<(&[Tri], String, u32)> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| (p.tris.as_slice(), color_key(Some(&p.material.color)), i as u32))
        .collect();
    let refs: Vec<(&[Tri], &str, u32)> = objs.iter().map(|(t, c, i)| (*t, c.as_str(), *i)).collect();
    Shared {
        src,
        src_bvh: Surfaces::new(&refs),
        samples: BTreeMap::new(),
        tris_hash: h,
    }
}

fn aggregate(d: &[f64], k: &K) -> (f64, f64, f64) {
    let mut rs = 0.0;
    let mut sum = 0.0;
    let mut mx = 0.0f64;
    for &x in d {
        rs += math::log2(1.0 + (x - k.delta).max(0.0) / k.delta);
        sum += x;
        mx = mx.max(x);
    }
    let n = d.len().max(1) as f64;
    (rs / n, sum / n, mx)
}

fn part_result(s: &SrcEntry, d: Vec<f64>, own: Vec<i64>, k: &K) -> PartRes {
    let (r, mean, max) = aggregate(&d, k);
    // 投票：按首次出现顺序累计，取票数最多的（并列取先出现的，与 JS 的稳定排序相同）
    let mut votes: Vec<(i64, usize)> = Vec::new();
    for &o in &own {
        if o >= 0 {
            match votes.iter_mut().find(|v| v.0 == o) {
                Some(v) => v.1 += 1,
                None => votes.push((o, 1)),
            }
        }
    }
    let mut top: Option<(i64, usize)> = None;
    for v in &votes {
        if top.map(|t| v.1 > t.1).unwrap_or(true) {
            top = Some(*v);
        }
    }
    PartRes {
        c: k.c_res * math::log2(1.0 + s.area / (k.delta * k.delta)) * r,
        mean,
        max,
        who: top.map(|t| t.0).unwrap_or(-1),
        owners: votes.iter().map(|v| v.0).collect(),
        d,
        own,
    }
}

#[derive(Clone, Default)]
struct Per {
    path: String,
    stmt: f64,
    mesh: f64,
    r_src: f64,
    r_recon: f64,
    r_vol: f64,
    mean_err: f64,
    max_err: f64,
    src: Vec<String>,
}

fn round(x: f64, digits: i32) -> f64 {
    let m = math::powi(10.0, digits);
    (x * m).round() / m
}

/// JS `String(Math.round(x * 1e4) / 1e4)`
fn fmt4(x: f64) -> String {
    let v = (x * 1e4).round() / 1e4;
    if v == 0.0 {
        return "0".to_string();
    }
    format!("{}", v)
}

/// 评分的输入
pub struct Input<'a> {
    pub items: &'a [ItemTree],
    pub nodes: &'a [NodeInfo],
    /// 描述文本：(文件的可显示名, 文本)，已包括 import 的文件；文件名与 `NodeInfo::file` 一致
    pub files: &'a [(String, String)],
    pub parts: &'a [Part],
    pub consts: Consts,
    pub edit: Option<&'a EditReport>,
}

/// 评分；`cache` 给了就增量评分
pub fn score(input: &Input, cache: Option<&mut ScoreCache>) -> J {
    let k0 = &input.consts;
    let u = k0.unit;
    let k = K {
        delta: k0.delta / u,
        d_max: k0.d_max / u,
        c_res: k0.c_res,
        samples: k0.samples,
    };
    let recon: Vec<Obj> = scene_objects_with(input.items, input.parts, &k0.tri)
        .into_iter()
        .filter(|o| !o.tris.is_empty())
        .collect();
    // 源侧
    let mut cache = cache;
    let shared_fresh = match &cache {
        Some(c) => match &c.shared {
            Some(s) => !(s.tris_hash == fnv_tris(input.parts) && s.src.len() == input.parts.len()),
            None => true,
        },
        None => true,
    };
    let mut shared_local: Option<Shared> = None;
    if let Some(c) = cache.as_deref_mut() {
        if shared_fresh {
            c.shared = Some(shared_of(input.parts, k0, None));
            c.states.clear();
        }
    } else {
        shared_local = Some(shared_of(input.parts, k0, None));
    }
    let ckey = {
        let skip: Vec<String> = input
            .parts
            .iter()
            .enumerate()
            .filter(|(_, p)| p.role.as_deref() == Some("helper"))
            .map(|(i, _)| i.to_string())
            .collect();
        format!(
            "vol1|{}|{}|{}|{}|{}",
            k.delta,
            k.d_max,
            k.samples,
            k.c_res,
            skip.join(",")
        )
    };
    // 拿到 shared 与 state 的可变引用
    let (shared, mut state): (&mut Shared, Option<&mut State>) = match cache {
        Some(c) => {
            let st = c.states.entry(ckey.clone()).or_default();
            (c.shared.as_mut().unwrap(), Some(st))
        }
        None => (shared_local.as_mut().unwrap(), None),
    };
    let role_of_src: BTreeMap<String, Option<String>> = shared
        .src
        .iter()
        .map(|s| (s.name.clone(), s.role.clone()))
        .collect();
    let mut recon = recon;
    for o in recon.iter_mut() {
        if let Some(m) = &mut o.mesh {
            if m.role.is_none() {
                m.role = role_of_src.get(&m.src).cloned().flatten();
            }
        }
    }

    // ---- 描述长度 ----
    let r = res();
    let (_d0_total, d0_lines) = description_length(input.files);
    let declared: BTreeSet<String> = input
        .nodes
        .iter()
        .map(|n| format!("{}:{}", n.file, n.line))
        .collect();
    let file_text: BTreeMap<&str, Vec<&str>> = input
        .files
        .iter()
        .map(|(f, t)| (f.as_str(), t.split('\n').collect()))
        .collect();
    let text_at = |x: &LineCost| -> &str {
        file_text
            .get(x.file.as_str())
            .and_then(|ls| ls.get(x.line - 1))
            .copied()
            .unwrap_or("")
    };
    let keep = |x: &LineCost| -> bool {
        let t = text_at(x);
        r.not_decl.is_match(t) || !r.decl.is_match(t) || declared.contains(&format!("{}:{}", x.file, x.line))
    };
    let mut reshaped: BTreeSet<String> = BTreeSet::new();
    for (_, text) in input.files {
        for c in r.over_reshape.captures_iter(text) {
            reshaped.insert(c[1].to_string());
        }
    }
    let lines: Vec<LineCost> = d0_lines
        .iter()
        .filter(|x| keep(x))
        .map(|x| {
            let t = text_at(x);
            match r.head2.captures(t) {
                Some(m)
                    if reshaped.contains(&m[2])
                        && !r.reshape_skip.is_match(&m[1])
                        && !r.open_brace.is_match(t) =>
                {
                    LineCost {
                        cost: line_cost(t) - reshape_saving(t),
                        ..x.clone()
                    }
                }
                _ => x.clone(),
            }
        })
        .collect();
    let d_total: f64 = lines.iter().map(|x| x.cost).sum();
    let dropped = d0_lines.len() - lines.len();
    let by_line: BTreeMap<String, f64> = lines
        .iter()
        .map(|x| (format!("{}:{}", x.file, x.line), x.cost))
        .collect();
    // 每行的长度归到在这一行声明的物体（阵列、模板实例的那一行归到容器）
    let mut stmt_of: Vec<(String, f64)> = Vec::new();
    let mut stmt_seen: BTreeSet<String> = BTreeSet::new();
    for n in input.nodes {
        let key = format!("{}:{}", n.file, n.line);
        if let Some(&c) = by_line.get(&key) {
            if c != 0.0 && !n.ghost && n.kind != "instance" && !stmt_seen.contains(&key) {
                stmt_seen.insert(key);
                stmt_of.push((n.path.clone(), c));
            }
        }
    }

    // ---- 重建物体表 ----
    let mut per: Vec<Per> = recon
        .iter()
        .map(|o| Per {
            path: o.path.clone(),
            ..Default::default()
        })
        .collect();
    let per_index: BTreeMap<String, usize> =
        per.iter().enumerate().map(|(i, p)| (p.path.clone(), i)).collect();
    let per_paths: Vec<String> = per.iter().map(|p| p.path.clone()).collect();
    let owner_of_path = |path: &str| -> Option<usize> {
        if let Some(&i) = per_index.get(path) {
            return Some(i);
        }
        let prefix = format!("{}.", path);
        if let Some(i) = per_paths.iter().position(|p| p.starts_with(&prefix)) {
            return Some(i);
        }
        let mut p = path.to_string();
        while let Some(cut) = p.rfind('.') {
            p.truncate(cut);
            if let Some(&i) = per_index.get(&p) {
                return Some(i);
            }
        }
        None
    };
    let mut attributed = 0.0;
    let owners_stmt: Vec<(Option<usize>, f64)> =
        stmt_of.iter().map(|(p, c)| (owner_of_path(p), *c)).collect();
    for (o, c) in &owners_stmt {
        if let Some(i) = o {
            per[*i].stmt += c;
            attributed += c;
        }
    }
    // 没归到任何物体的行（声明的物体没有几何、或语句不声明物体）
    let attributed_keys: BTreeSet<String> = input
        .nodes
        .iter()
        .filter(|n| stmt_of.iter().any(|(p, _)| *p == n.path))
        .map(|n| format!("{}:{}", n.file, n.line))
        .collect();
    let unattributed_lines: Vec<J> = lines
        .iter()
        .filter(|x| {
            let key = format!("{}:{}", x.file, x.line);
            !attributed_keys.contains(&key)
                || stmt_of.iter().zip(owners_stmt.iter()).any(|((p, _), (o, _))| {
                    o.is_none()
                        && input
                            .nodes
                            .iter()
                            .any(|n| n.path == *p && format!("{}:{}", n.file, n.line) == key)
                })
        })
        .map(|x| json!({ "file": x.file, "line": x.line, "cost": x.cost }))
        .collect();

    // ---- L_mesh ----
    let mut l_mesh = 0.0;
    let mut mesh_seen: BTreeSet<String> = BTreeSet::new();
    for (j, o) in recon.iter().enumerate() {
        if let Some(m) = &o.mesh {
            if !mesh_seen.insert(m.key.clone()) {
                continue;
            }
            let tris = m.part.map(|i| input.parts[i].tris.as_slice());
            let c = mesh_price(tris, m.role.as_deref(), k0);
            l_mesh += c;
            per[j].mesh += c;
        }
    }

    // ---- 这一次的重建物体：键与包围盒；与上一次比 ----
    let keys: Vec<String> = recon
        .iter()
        .map(|o| format!("{}|{}|{}", o.path, color_key(Some(&o.color)), hash_tris(&o.tris)))
        .collect();
    let now: BTreeMap<String, PrevObj> = keys
        .iter()
        .enumerate()
        .map(|(j, key)| {
            let (lo, hi) = bbox_tris(&recon[j].tris);
            (
                key.clone(),
                PrevObj {
                    j,
                    path: recon[j].path.clone(),
                    lo,
                    hi,
                    color: color_key(Some(&recon[j].color)),
                },
            )
        })
        .collect();
    let (mut ids_path, mut ids_of) = match &state {
        Some(st) => (st.ids_path.clone(), st.ids_of.clone()),
        None => (Vec::new(), BTreeMap::new()),
    };
    let mut id_of = |p: &str| -> usize {
        match ids_of.get(p) {
            Some(&i) => i,
            None => {
                let i = ids_path.len();
                ids_path.push(p.to_string());
                ids_of.insert(p.to_string(), i);
                i
            }
        }
    };
    let recon_id: Vec<usize> = recon.iter().map(|o| id_of(&o.path)).collect();
    // 删掉的物体的编号、新增的物体、颜色翻转
    struct Delta {
        added: Vec<PrevObj>,
        removed: BTreeSet<i64>,
        flipped: BTreeSet<String>,
        colors_now: BTreeSet<String>,
        added_surf: Option<Surfaces>,
    }
    let mut delta: Option<Delta> = None;
    if let Some(st) = &state {
        if let (Some(prev), Some(_)) = (&st.prev, &st.src_res) {
            let colors_now: BTreeSet<String> = now.values().map(|x| x.color.clone()).collect();
            let colors_prev: BTreeSet<String> = prev.values().map(|x| x.color.clone()).collect();
            let added: Vec<PrevObj> = now
                .iter()
                .filter(|(k, _)| !prev.contains_key(*k))
                .map(|(_, x)| x.clone())
                .collect();
            let mut removed = BTreeSet::new();
            for (key, x) in prev {
                if !now.contains_key(key) {
                    removed.insert(id_of(&x.path) as i64);
                }
            }
            let flipped: BTreeSet<String> = colors_now.symmetric_difference(&colors_prev).cloned().collect();
            if added.len() + removed.len() <= (0.3 * recon.len() as f64) as usize {
                let added_surf = if added.is_empty() {
                    None
                } else {
                    let objs: Vec<(&[Tri], &str, u32)> = added
                        .iter()
                        .map(|a| (recon[a.j].tris.as_slice(), recon[a.j].color.as_str(), a.j as u32))
                        .collect();
                    Some(Surfaces::new(&objs))
                };
                delta = Some(Delta {
                    added,
                    removed,
                    flipped,
                    colors_now,
                    added_surf,
                });
            }
        }
    }

    // 重建物体的树：用到时才建
    let recon_objs: Vec<(&[Tri], &str, u32)> = recon
        .iter()
        .enumerate()
        .map(|(j, o)| (o.tris.as_slice(), o.color.as_str(), j as u32))
        .collect();
    let mut recon_bvh: Option<Surfaces> = None;
    let mut need_bvh = || Surfaces::new(&recon_objs);

    // ~src 指向源部件的物体：它就是"负责解释这块网格"的
    let mut src_owner: BTreeMap<String, usize> = BTreeMap::new();
    for n in input.nodes {
        if let Some(s) = &n.meta_src {
            if !src_owner.contains_key(s) {
                if let Some(o) = owner_of_path(&n.path) {
                    src_owner.insert(s.clone(), o);
                }
            }
        }
    }

    // ---- R_src ----
    shared.samples.entry(k.samples).or_insert_with(|| {
        input
            .parts
            .iter()
            .enumerate()
            .map(|(i, p)| source_samples(&p.tris, i, k.samples))
            .collect()
    });
    let src_pts = &shared.samples[&k.samples];
    let mut r_src = 0.0;
    let mut src_out: Vec<J> = Vec::new();
    let mut src_res: Vec<PartRes> = Vec::with_capacity(shared.src.len());
    let mut touched_parts = 0usize;
    let mut touched_points = 0usize;
    for s in &shared.src {
        let x: PartRes = match (&delta, &state) {
            (Some(dl), Some(st)) => {
                let was = &st.src_res.as_ref().unwrap()[s.i];
                let c = color_key(Some(&s.color));
                let same = dl.colors_now.contains(&c);
                let adds: Vec<&PrevObj> = dl
                    .added
                    .iter()
                    .filter(|a| (!same || a.color == c) && boxes_meet((s.lo, s.hi), (a.lo, a.hi), was.max))
                    .collect();
                // None：全部重算；Some(false)：不用动；Some(true)：逐点核对
                let hit: Option<bool> = if dl.flipped.contains(&c) {
                    None
                } else if was.owners.iter().any(|o| dl.removed.contains(o)) || !adds.is_empty() {
                    Some(true)
                } else {
                    Some(false)
                };
                match hit {
                    Some(false) => was.clone(),
                    _ => {
                        let bvh = recon_bvh.get_or_insert_with(&mut need_bvh);
                        let pts = &src_pts[s.i];
                        let partial = hit == Some(true);
                        let mut d = if partial {
                            was.d.clone()
                        } else {
                            vec![0.0; pts.len()]
                        };
                        let mut own = if partial {
                            was.own.clone()
                        } else {
                            vec![-1; pts.len()]
                        };
                        for (i, p) in pts.iter().enumerate() {
                            if partial && !dl.removed.contains(&own[i]) {
                                let near_add = adds.iter().any(|a| box_dist(*p, (a.lo, a.hi)) < d[i]);
                                let closer = near_add && {
                                    let surf = dl.added_surf.as_ref().unwrap();
                                    let g = if same {
                                        surf.by_color.get(&c)
                                    } else {
                                        surf.all.as_ref()
                                    };
                                    let pen = if !same { k.delta } else { 0.0 };
                                    match g {
                                        Some(g) if d[i] - pen > 0.0 => g
                                            .closest(*p, d[i] - pen)
                                            .map(|(dd, _)| dd + pen < d[i])
                                            .unwrap_or(false),
                                        _ => false,
                                    }
                                };
                                if !closer {
                                    continue;
                                }
                            }
                            let (di, oi) = bvh.nearest(Some(&s.color), *p, k.delta, k.d_max);
                            d[i] = di;
                            own[i] = oi.map(|o| recon_id[o as usize] as i64).unwrap_or(-1);
                            touched_points += 1;
                        }
                        touched_parts += 1;
                        part_result(s, d, own, &k)
                    }
                }
            }
            _ => {
                let bvh = recon_bvh.get_or_insert_with(&mut need_bvh);
                let pts = &src_pts[s.i];
                let mut d = Vec::with_capacity(pts.len());
                let mut own = Vec::with_capacity(pts.len());
                for p in pts {
                    let (di, oi) = bvh.nearest(Some(&s.color), *p, k.delta, k.d_max);
                    d.push(di);
                    own.push(oi.map(|o| recon_id[o as usize] as i64).unwrap_or(-1));
                }
                touched_parts += 1;
                touched_points += pts.len();
                part_result(s, d, own, &k)
            }
        };
        if s.skip {
            src_out.push(json!({ "name": s.name, "tris": input.parts[s.i].tris.len(), "cost": 0, "role": "helper", "skipped": true }));
            src_res.push(x);
            continue;
        }
        r_src += x.c;
        let who: Option<usize> = src_owner.get(&s.name).copied().or_else(|| {
            if x.who >= 0 {
                per_index.get(&ids_path[x.who as usize]).copied()
            } else {
                None
            }
        });
        if let Some(w) = who {
            let p = &mut per[w];
            p.r_src += x.c;
            p.src.push(s.name.clone());
            p.mean_err = p.mean_err.max(x.mean);
            p.max_err = p.max_err.max(x.max);
        }
        src_out.push(json!({
            "name": s.name, "tris": input.parts[s.i].tris.len(), "cost": round(x.c, 3),
            "meanErr": round(x.mean * u, 5), "maxErr": round(x.max * u, 5),
            "explainedBy": who.map(|w| J::String(per[w].path.clone())).unwrap_or(J::Null),
        }));
        src_res.push(x);
    }
    let _ = (touched_parts, touched_points);

    // ---- R_recon / R_vol ----
    let mut recon_res: BTreeMap<String, ReconRes> = BTreeMap::new();
    let mut fresh = 0usize;
    let rec: Vec<ReconRes> = recon
        .iter()
        .enumerate()
        .map(|(j, o)| {
            if let Some(st) = &state {
                if let Some(x) = st.recon.get(&keys[j]) {
                    recon_res.insert(keys[j].clone(), x.clone());
                    return x.clone();
                }
            }
            fresh += 1;
            let pts = sample_points(&o.tris, k.samples, seed_of(&o.path));
            let d: Vec<f64> = pts
                .iter()
                .map(|p| shared.src_bvh.nearest(Some(&o.color), *p, k.delta, k.d_max).0)
                .collect();
            let (r, mean, max) = aggregate(&d, &k);
            let inner = if o.mesh.is_some() {
                None
            } else {
                interior_samples(&o.tris, k.delta, seed_of(&o.path), 24)
            };
            let (vol, rv) = match inner {
                Some((vol, pts)) => (
                    vol,
                    volume_stats(&pts, shared.src_bvh.all.as_ref(), k.delta, k.d_max).0,
                ),
                None => (0.0, 0.0),
            };
            let x = ReconRes {
                r,
                mean,
                max,
                area: o.tris.iter().map(tri_area).sum(),
                vol,
                rv,
            };
            recon_res.insert(keys[j].clone(), x.clone());
            x
        })
        .collect();
    let _ = fresh;
    // 顶层语句分组
    let top_of = |p: &str| p.split('.').next().unwrap_or("").to_string();
    let mut groups: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for (j, x) in rec.iter().enumerate() {
        let g = groups.entry(top_of(&recon[j].path)).or_insert((0.0, 0.0));
        g.0 += x.area;
        g.1 += x.vol;
    }
    let mut r_recon = 0.0;
    let mut r_vol = 0.0;
    for (j, x) in rec.iter().enumerate() {
        let g = groups[&top_of(&recon[j].path)];
        let share = if g.0 > 0.0 { x.area / g.0 } else { 1.0 };
        let c = k.c_res * math::log2(1.0 + g.0 / (k.delta * k.delta)) * share * x.r;
        r_recon += c;
        let p = &mut per[j];
        p.r_recon += c;
        p.mean_err = p.mean_err.max(x.mean);
        p.max_err = p.max_err.max(x.max);
        if x.rv > 0.0 {
            let cv = k.c_res * math::log2(1.0 + g.1 / (k.delta * k.delta * k.delta)) * (x.vol / g.1) * x.rv;
            r_vol += cv;
            p.r_vol += cv;
        }
    }
    // 记住这一次的状态
    if let Some(st) = state.as_mut() {
        st.prev = Some(now);
        st.src_res = Some(src_res.clone());
        st.recon = recon_res;
        st.ids_path = ids_path.clone();
        st.ids_of = ids_of.clone();
    }

    // ---- 基线 S_mesh ----
    let mut s_mesh = 0.0;
    let mut base_seen: BTreeSet<String> = BTreeSet::new();
    let mut base_of = vec![0.0; shared.src.len()];
    for s in &shared.src {
        if s.skip {
            continue;
        }
        let first = match &s.mesh_key {
            Some(mk) => base_seen.insert(mk.clone()),
            None => true,
        };
        let pv = [(s.lo.x + s.hi.x) / 2.0, s.lo.y, (s.lo.z + s.hi.z) / 2.0];
        let line = format!(
            "mesh p src=\"x#{}\" size={},{},{} pivot={},{},{} at={},{},{} mat=m",
            s.name,
            fmt4(s.hi.x - s.lo.x),
            fmt4(s.hi.y - s.lo.y),
            fmt4(s.hi.z - s.lo.z),
            fmt4(pv[0]),
            fmt4(pv[1]),
            fmt4(pv[2]),
            fmt4(pv[0]),
            fmt4(pv[1]),
            fmt4(pv[2])
        );
        let before = s_mesh;
        s_mesh += line_cost(&line)
            + if first {
                mesh_price(Some(&input.parts[s.i].tris), s.role.as_deref(), k0)
            } else {
                0.0
            };
        base_of[s.i] = s_mesh - before;
    }

    // ---- E_edit ----
    let (e_edit, coord_dep, edit_json) = match input.edit {
        Some(e) => (
            k0.edit_weight * e.dependent.len() as f64,
            e.dependent.len(),
            json!({ "dependent": e.dependent.len(), "checked": e.checked, "weight": k0.edit_weight, "factsMissing": e.facts_missing, "objects": e.dependent }),
        ),
        None => (0.0, 0, J::Null),
    };
    let _ = coord_dep;

    let s_total = d_total + l_mesh + r_src + r_recon + r_vol + e_edit;

    // ---- 按角色 ----
    let role_of_node: BTreeMap<&str, Option<&str>> = input
        .nodes
        .iter()
        .map(|n| (n.path.as_str(), n.meta_role.as_deref()))
        .collect();
    let role_of = |path: &str| -> String {
        let mut p = path.to_string();
        loop {
            if let Some(Some(r)) = role_of_node.get(p.as_str()) {
                return r.to_string();
            }
            match p.rfind('.') {
                Some(cut) => p.truncate(cut),
                None => return "structure".to_string(),
            }
        }
    };
    #[derive(Default)]
    struct RoleAcc {
        s: f64,
        l_stmt: f64,
        l_mesh: f64,
        r_src: f64,
        r_recon: f64,
        r_vol: f64,
        s_mesh: f64,
        objects: usize,
        sources: usize,
    }
    let mut by_role: BTreeMap<String, RoleAcc> = BTreeMap::new();
    for p in &per {
        let b = by_role.entry(role_of(&p.path)).or_default();
        b.l_stmt += p.stmt;
        b.l_mesh += p.mesh;
        b.r_recon += p.r_recon;
        b.r_vol += p.r_vol;
        b.objects += 1;
    }
    for (i, x) in shared.src.iter().enumerate() {
        if x.skip {
            continue;
        }
        let b = by_role
            .entry(x.role.clone().unwrap_or_else(|| "structure".to_string()))
            .or_default();
        b.r_src += src_res[i].c;
        b.s_mesh += base_of[i];
        b.sources += 1;
    }
    let mut by_role_json = Map::new();
    for (name, b) in by_role.iter_mut() {
        b.s = b.l_stmt + b.l_mesh + b.r_src + b.r_recon + b.r_vol;
        let explain = if b.s_mesh != 0.0 {
            json!(round(1.0 - b.s / b.s_mesh, 4))
        } else {
            J::Null
        };
        by_role_json.insert(
            name.clone(),
            json!({
                "S": round(b.s, 2), "L_stmt": round(b.l_stmt, 2), "L_mesh": round(b.l_mesh, 2), "R_src": round(b.r_src, 2),
                "R_recon": round(b.r_recon, 2), "R_vol": round(b.r_vol, 2), "S_mesh": round(b.s_mesh, 2),
                "objects": b.objects, "sources": b.sources, "explain": explain,
            }),
        );
    }

    // ---- 逐物体 ----
    let objects: Vec<J> = per
        .iter()
        .map(|p| {
            json!({
                "path": p.path, "stmt": round(p.stmt, 2), "mesh": round(p.mesh, 2), "rSrc": round(p.r_src, 3),
                "rRecon": round(p.r_recon, 3), "rVol": round(p.r_vol, 3), "meanErr": round(p.mean_err * u, 5),
                "maxErr": round(p.max_err * u, 5), "src": p.src,
                "total": round(p.stmt + p.mesh + p.r_src + p.r_recon + p.r_vol, 3),
            })
        })
        .collect();
    let mut worst: Vec<&Per> = per.iter().collect();
    worst.sort_by(|a, b| {
        let ca = a.mesh + a.r_src + a.r_recon + a.r_vol;
        let cb = b.mesh + b.r_src + b.r_recon + b.r_vol;
        cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
    });
    let worst: Vec<J> = worst
        .iter()
        .take(20)
        .map(|o| {
            json!({ "path": o.path, "cost": round(o.mesh + o.r_src + o.r_recon + o.r_vol, 2), "meanErr": round(o.mean_err * u, 5), "mesh": o.mesh > 0.0 })
        })
        .collect();

    json!({
        "S": round(s_total, 2), "L_stmt": round(d_total, 2), "unattributed": round(d_total - attributed, 2),
        "droppedLines": dropped, "L_mesh": round(l_mesh, 2), "R_src": round(r_src, 2), "R_recon": round(r_recon, 2),
        "R_vol": round(r_vol, 2), "E_edit": round(e_edit, 2), "S_mesh": round(s_mesh, 2),
        "explain": round(1.0 - s_total / s_mesh, 4),
        "byRole": by_role_json,
        "consts": { "delta": k0.delta, "cRes": k0.c_res, "cMesh": k0.c_mesh, "dMax": k0.d_max, "samples": k0.samples, "unit": k0.unit, "editWeight": k0.edit_weight },
        "edit": edit_json,
        "objects": objects, "sources": src_out, "worst": worst, "unattributedLines": unattributed_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costs() {
        assert_eq!(number_cost("3"), 1.0);
        assert_eq!(number_cost("3.5"), 1.0);
        assert_eq!(number_cost("3.25"), 1.5);
        assert_eq!(number_cost("3.250"), 1.5);
        assert_eq!(
            line_cost("box crate size=2,1.5,0.25 on=floor ~role=\"x\" // 注释"),
            8.5
        );
        assert_eq!(line_cost("concept walls = kind(box)"), 0.0);
        assert_eq!(line_cost("  "), 0.0);
        assert_eq!(reshape_saving("box crate size=2,1,1 on=floor mat=m"), 4.0);
    }
}
