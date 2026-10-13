//! 对参考输出的对拍（spec/09 9.4）：节点的 world / box / tight、items、curves、materials、defs、诊断条数。
//! 参考侧先按 9.4.5 规范化（N1、N11、N15、N16 等在比较里内联实现）。

use serde_json::Value as J;
use std::collections::BTreeMap;

const TAU_LEN: f64 = 1e-6;
const TAU_ROT: f64 = 1e-6;

pub struct Report {
    pub lines: Vec<String>,
    pub failures: usize,
}

impl Report {
    fn fail(&mut self, s: String) {
        self.failures += 1;
        if self.lines.len() < 400 {
            self.lines.push(s);
        }
    }
}

fn nums(v: &J) -> Vec<f64> {
    let mut out = Vec::new();
    fn walk(v: &J, out: &mut Vec<f64>) {
        match v {
            J::Number(n) => out.push(n.as_f64().unwrap()),
            J::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            J::Array(a) => a.iter().for_each(|x| walk(x, out)),
            J::Object(o) => o.values().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(v, &mut out);
    out
}

fn max_diff(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() {
        return None;
    }
    Some(a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max))
}

fn file_name(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}

/// 盒 {min,max} → [min3, max3]
fn aabb(v: &J) -> Option<[f64; 6]> {
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

fn aabb_union(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].min(b[2]),
        a[3].max(b[3]),
        a[4].max(b[4]),
        a[5].max(b[5]),
    ]
}

fn is_signed_permutation(r: &J) -> bool {
    let m = nums(r);
    if m.len() != 9 {
        return false;
    }
    m.iter().all(|x| (x.abs() - 1.0).abs() < 1e-9 || x.abs() < 1e-9)
}

/// 用参考里的实体 tight 补算布尔容器的 tight（N15a）
fn ref_bool_tight(
    path: &str,
    kind: &str,
    ref_nodes: &BTreeMap<String, &J>,
    ref_order: &[String],
) -> Option<[f64; 6]> {
    let prefix = format!("{}.", path);
    // 直接子节点（深度 +1）按节点顺序
    let kids: Vec<(&String, &&J)> = ref_order
        .iter()
        .filter(|p| p.starts_with(&prefix) && !p[prefix.len()..].contains('.'))
        .map(|p| (p, &ref_nodes[p]))
        .collect();
    let mut acc: Option<[f64; 6]> = None;
    for (p, n) in kids {
        if n["curve"].as_bool().unwrap_or(false) {
            continue;
        }
        let t = match n["tight"].as_object() {
            Some(_) => aabb(&n["tight"]),
            None => {
                // 容器操作数：内部实体 tight 的并
                let sub = format!("{}.", p);
                let mut u: Option<[f64; 6]> = None;
                for (q, m) in ref_nodes.iter() {
                    if q.starts_with(&sub) {
                        if let Some(t) = aabb(&m["tight"]) {
                            u = Some(match u {
                                None => t,
                                Some(a) => aabb_union(a, t),
                            });
                        }
                    }
                }
                u
            }
        };
        let t = match t {
            Some(t) => t,
            None => continue,
        };
        acc = Some(match (acc, kind) {
            (None, _) => t,
            (Some(a), "subtract") => a,
            (Some(a), "intersect") => [
                a[0].max(t[0]),
                a[1].max(t[1]),
                a[2].max(t[2]),
                a[3].min(t[3]),
                a[4].min(t[4]),
                a[5].min(t[5]),
            ],
            (Some(a), _) => aabb_union(a, t),
        });
    }
    acc
}

/// 实体种类的 δ_tight（9.4.2）：由本版 items 的 geom 求
fn delta_tight(kind: &str, geom: Option<&J>) -> (f64, f64) {
    let g = match geom {
        Some(g) => g,
        None => return (0.0, 0.0),
    };
    let cos15 = s3d_geom::math::cos_deg(15.0);
    match g["type"].as_str().unwrap_or("") {
        "cyl" => {
            let r = g["r1"]
                .as_f64()
                .unwrap_or(0.0)
                .max(g["r2"].as_f64().unwrap_or(0.0));
            let out = if kind == "cone" { 1e-4 } else { 0.0 };
            (r * (1.0 - cos15), out)
        }
        "rod" => (g["r"].as_f64().unwrap_or(0.0) * (1.0 - cos15), 0.0),
        "sweep" if g["smooth"].as_bool().unwrap_or(false) => {
            let r = g["prof"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|p| p[0].as_f64().unwrap_or(0.0).abs())
                        .fold(0.0, f64::max)
                })
                .unwrap_or(0.0);
            (r * (1.0 - s3d_geom::math::cos_deg(27.0)), 0.0)
        }
        _ => (0.0, 0.0),
    }
}

fn flatten_items<'a>(items: &'a [J], out: &mut Vec<&'a J>) {
    for it in items {
        out.push(it);
        if let Some(ch) = it["children"].as_array() {
            flatten_items(ch, out);
        }
    }
}

pub fn verify(mine: &J, reft: &J) -> Report {
    let mut rep = Report {
        lines: Vec::new(),
        failures: 0,
    };
    // ===== nodes =====
    let mut ref_nodes: BTreeMap<String, &J> = BTreeMap::new();
    let mut ref_order: Vec<String> = Vec::new();
    for n in reft["nodes"].as_array().unwrap() {
        let p = n["path"].as_str().unwrap().to_string();
        ref_order.push(p.clone());
        ref_nodes.insert(p, n);
    }
    let my_nodes = mine["nodes"].as_array().unwrap();
    let my_order: Vec<String> = my_nodes
        .iter()
        .map(|n| n["path"].as_str().unwrap().to_string())
        .collect();
    if my_order != ref_order {
        let missing: Vec<&String> = ref_order.iter().filter(|p| !my_order.contains(p)).collect();
        let extra: Vec<&String> = my_order.iter().filter(|p| !ref_order.contains(p)).collect();
        for p in missing.iter().take(20) {
            rep.fail(format!("  MISSING node {}", p));
        }
        for p in extra.iter().take(20) {
            rep.fail(format!("  EXTRA node {}", p));
        }
        if missing.is_empty() && extra.is_empty() {
            rep.fail("  节点顺序不同".to_string());
        }
    }
    // 本版 items 按 path 索引（δ_tight 用）
    let mut my_items_flat = Vec::new();
    flatten_items(mine["items"].as_array().unwrap(), &mut my_items_flat);
    let my_geom: BTreeMap<&str, &J> = my_items_flat
        .iter()
        .filter_map(|it| it["path"].as_str().map(|p| (p, &it["geom"])))
        .collect();
    let mut node_ok = 0usize;
    for n in my_nodes {
        let path = n["path"].as_str().unwrap();
        let r = match ref_nodes.get(path) {
            Some(r) => *r,
            None => continue,
        };
        let mut bad = false;
        // world
        match (
            max_diff(&nums(&n["world"]["R"]), &nums(&r["world"]["R"])),
            max_diff(&nums(&n["world"]["t"]), &nums(&r["world"]["t"])),
        ) {
            (Some(dr), Some(dt)) => {
                if dr > TAU_ROT || dt > TAU_LEN {
                    bad = true;
                    rep.fail(format!("  DIFF {} world ΔR={:.2e} Δt={:.2e}", path, dr, dt));
                }
            }
            _ => {
                bad = true;
                rep.fail(format!("  DIFF {} world 形状不同", path));
            }
        }
        // box
        match max_diff(&nums(&n["box"]), &nums(&r["box"])) {
            Some(d) if d <= TAU_LEN => {}
            Some(d) => {
                bad = true;
                rep.fail(format!("  DIFF {} box Δ={:.2e}", path, d));
            }
            None => {
                bad = true;
                rep.fail(format!("  DIFF {} box 形状不同", path));
            }
        }
        // 离散字段
        let ghost = r["ghost"].as_bool().unwrap_or(false) || r["of"].is_string();
        let rkind = r["kind"].as_str().unwrap_or("");
        if n["kind"] != r["kind"] && !ghost && !rkind.contains('→') {
            bad = true;
            rep.fail(format!("  DIFF {} kind {} vs {}", path, n["kind"], r["kind"]));
        }
        for key in [
            "late", "hidden", "skin", "curve", "tags", "id", "depth", "concepts",
        ] {
            if n[key] != r[key] {
                bad = true;
                rep.fail(format!("  DIFF {} {} {} vs {}", path, key, n[key], r[key]));
            }
        }
        // N5：copy 节点的 file / line 按本版是 copy 语句的，参考是原语句的；语料里是 watch2 与 hangar2
        let is_copy_node = matches!(path, "watch2" | "hangar2");
        if n["line"] != r["line"] && !path.ends_with(".skin") && !ghost && !is_copy_node {
            // copy 节点按 N5 预期不同：file/line 为 copy 语句的；这里只对非 copy 节点比
            let rf = r["file"].as_str().map(file_name).unwrap_or("");
            let mf = n["file"].as_str().map(file_name).unwrap_or("");
            if mf == rf {
                bad = true;
                rep.fail(format!("  DIFF {} line {} vs {}", path, n["line"], r["line"]));
            }
        }
        // meta
        if n["meta"] != r["meta"] && !ghost {
            bad = true;
            rep.fail(format!("  DIFF {} meta {} vs {}", path, n["meta"], r["meta"]));
        }
        // tight（N15 + δ 规则）
        let kind = n["kind"].as_str().unwrap_or("");
        let is_bool = matches!(kind, "union" | "subtract" | "intersect");
        let ref_t = if is_bool && r["tight"].is_null() {
            ref_bool_tight(path, kind, &ref_nodes, &ref_order)
        } else {
            aabb(&r["tight"])
        };
        let my_t = aabb(&n["tight"]);
        let skip_revolve = kind == "revolve"
            && (!is_signed_permutation(&n["world"]["R"])
                || my_geom
                    .get(path)
                    .map(|g| g["angle"].as_f64().unwrap_or(360.0) < 360.0)
                    .unwrap_or(false));
        if !skip_revolve {
            match (my_t, ref_t) {
                (None, None) => {}
                (Some(v), Some(u)) => {
                    let (dt, dout) = delta_tight(kind, my_geom.get(path).copied());
                    let mut worst = 0.0f64;
                    for k in 0..3 {
                        // 参考不得向外超出 δ_out；向内不超过 δ_tight
                        let out_lo = v[k] - u[k];
                        let out_hi = u[k + 3] - v[k + 3];
                        let in_lo = u[k] - v[k];
                        let in_hi = v[k + 3] - u[k + 3];
                        worst = worst
                            .max(out_lo - dout - TAU_LEN)
                            .max(out_hi - dout - TAU_LEN)
                            .max(in_lo - dt - TAU_LEN)
                            .max(in_hi - dt - TAU_LEN);
                    }
                    if worst > 0.0 {
                        bad = true;
                        rep.fail(format!(
                            "  DIFF {} tight 超出容许 {:.2e}（δ_tight={:.2e}）",
                            path, worst, dt
                        ));
                    }
                }
                (a, b) => {
                    bad = true;
                    rep.fail(format!(
                        "  DIFF {} tight 一边为空 mine={} ref={}",
                        path,
                        a.is_some(),
                        b.is_some()
                    ));
                }
            }
        }
        if !bad {
            node_ok += 1;
        }
    }
    rep.lines.push(format!(
        "nodes: {} / {} 一致（参考 {}）",
        node_ok,
        my_nodes.len(),
        ref_order.len()
    ));

    // ===== items =====
    let mut ref_items_flat = Vec::new();
    flatten_items(reft["items"].as_array().unwrap(), &mut ref_items_flat);
    if my_items_flat.len() != ref_items_flat.len() {
        rep.fail(format!(
            "  items 条数 {} vs {}",
            my_items_flat.len(),
            ref_items_flat.len()
        ));
    }
    let ref_by_path: BTreeMap<&str, &J> = ref_items_flat
        .iter()
        .filter_map(|it| it["path"].as_str().map(|p| (p, *it)))
        .collect();
    let mut item_ok = 0usize;
    for it in &my_items_flat {
        let path = it["path"].as_str().unwrap_or("");
        let r = match ref_by_path.get(path) {
            Some(r) => *r,
            None => {
                rep.fail(format!("  EXTRA item {}", path));
                continue;
            }
        };
        let mut bad = false;
        if it["kind"] != r["kind"] || it["tags"] != r["tags"] || it["op"] != r["op"] {
            bad = true;
            rep.fail(format!("  DIFF item {} kind/op/tags", path));
        }
        if it["kind"] == "solid" {
            if it["geom"]["type"] != r["geom"]["type"] {
                bad = true;
                rep.fail(format!(
                    "  DIFF item {} geom.type {} vs {}",
                    path, it["geom"]["type"], r["geom"]["type"]
                ));
            } else {
                // 枚举 / 布尔字段逐值；数值按 τ_len
                for (k, v) in it["geom"].as_object().unwrap() {
                    if k == "type" {
                        continue;
                    }
                    let rv = &r["geom"][k];
                    if v.is_string() || v.is_boolean() {
                        if v != rv {
                            bad = true;
                            rep.fail(format!("  DIFF item {} geom.{} {} vs {}", path, k, v, rv));
                        }
                    } else {
                        match max_diff(&nums(v), &nums(rv)) {
                            Some(d) if d <= TAU_LEN * 10.0 => {}
                            Some(d) => {
                                bad = true;
                                rep.fail(format!("  DIFF item {} geom.{} Δ={:.2e}", path, k, d));
                            }
                            None => {
                                bad = true;
                                rep.fail(format!(
                                    "  DIFF item {} geom.{} 形状不同（{} vs {}）",
                                    path,
                                    k,
                                    nums(v).len(),
                                    nums(rv).len()
                                ));
                            }
                        }
                    }
                }
            }
            match max_diff(&nums(&it["world"]), &nums(&r["world"])) {
                Some(d) if d <= TAU_LEN => {}
                _ => {
                    bad = true;
                    rep.fail(format!("  DIFF item {} world", path));
                }
            }
        }
        // material（N11）
        let m = &it["material"];
        let rm = &r["material"];
        let r_name = rm["name"].as_str().unwrap_or("");
        let color_only = rm.get("name").is_none() && rm.get("color").is_some();
        if m["color"] != rm["color"] {
            bad = true;
            rep.fail(format!(
                "  DIFF item {} material.color {} vs {}",
                path, m["color"], rm["color"]
            ));
        }
        if !color_only {
            if m["name"].as_str().unwrap_or("") != r_name {
                bad = true;
                rep.fail(format!(
                    "  DIFF item {} material.name {} vs {}",
                    path, m["name"], rm["name"]
                ));
            }
            for (k, dflt) in [("metal", 0.05), ("rough", 0.78), ("opacity", 1.0)] {
                let a = m[k].as_f64().unwrap_or(dflt);
                let b = rm[k].as_f64().unwrap_or(dflt);
                if (a - b).abs() > 1e-9 {
                    bad = true;
                    rep.fail(format!("  DIFF item {} material.{} {} vs {}", path, k, a, b));
                }
            }
        }
        if !bad {
            item_ok += 1;
        }
    }
    rep.lines.push(format!(
        "items: {} / {} 一致（参考 {}）",
        item_ok,
        my_items_flat.len(),
        ref_items_flat.len()
    ));

    // ===== curves =====
    let my_curves = mine["curves"].as_array().unwrap();
    let ref_curves = reft["curves"].as_array().unwrap();
    if my_curves.len() != ref_curves.len() {
        rep.fail(format!(
            "  curves 条数 {} vs {}",
            my_curves.len(),
            ref_curves.len()
        ));
    }
    let ref_c: BTreeMap<&str, &J> = ref_curves
        .iter()
        .filter_map(|c| c["path"].as_str().map(|p| (p, c)))
        .collect();
    let mut curve_ok = 0usize;
    for c in my_curves {
        let path = c["path"].as_str().unwrap_or("");
        match ref_c.get(path) {
            None => rep.fail(format!("  EXTRA curve {}", path)),
            Some(r) => {
                let mut bad = false;
                if c["closed"] != r["closed"] {
                    bad = true;
                    rep.fail(format!("  DIFF curve {} closed", path));
                }
                match max_diff(&nums(&c["pts"]), &nums(&r["pts"])) {
                    Some(d) if d <= TAU_LEN => {}
                    Some(d) => {
                        bad = true;
                        rep.fail(format!("  DIFF curve {} pts Δ={:.2e}", path, d));
                    }
                    None => {
                        bad = true;
                        rep.fail(format!(
                            "  DIFF curve {} 点数 {} vs {}",
                            path,
                            c["pts"].as_array().map(|a| a.len()).unwrap_or(0),
                            r["pts"].as_array().map(|a| a.len()).unwrap_or(0)
                        ));
                    }
                }
                if !bad {
                    curve_ok += 1;
                }
            }
        }
    }
    rep.lines.push(format!(
        "curves: {} / {} 一致（参考 {}）",
        curve_ok,
        my_curves.len(),
        ref_curves.len()
    ));

    // ===== materials =====
    let mm = mine["materials"].as_object().unwrap();
    let rm = reft["materials"].as_object().unwrap();
    let my_keys: Vec<&String> = mm.keys().collect();
    let ref_keys: Vec<&String> = rm.keys().collect();
    if my_keys != ref_keys {
        rep.fail(format!("  materials 键序 {:?} vs {:?}", my_keys, ref_keys));
    } else {
        for (k, v) in mm {
            let r = &rm[k];
            if v["color"] != r["color"] {
                rep.fail(format!(
                    "  DIFF material {} color {} vs {}",
                    k, v["color"], r["color"]
                ));
            }
            for (f, dflt) in [("metal", 0.05), ("rough", 0.78), ("opacity", 1.0)] {
                let a = v[f].as_f64().unwrap_or(dflt);
                let b = r[f].as_f64().unwrap_or(dflt);
                if (a - b).abs() > 1e-9 {
                    rep.fail(format!("  DIFF material {} {} {} vs {}", k, f, a, b));
                }
            }
            if v["default"].as_bool().unwrap_or(false) != r["default"].as_bool().unwrap_or(false) {
                rep.fail(format!("  DIFF material {} default", k));
            }
        }
    }
    // ===== defs（N16）=====
    let my_defs: Vec<&str> = mine["defs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["name"].as_str())
        .collect();
    let ref_defs: Vec<&str> = reft["defs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d.as_str().or_else(|| d["name"].as_str()))
        .collect();
    if my_defs != ref_defs {
        rep.fail(format!("  defs {:?} vs {:?}", my_defs, ref_defs));
    }
    // ===== facts（9.4.3；参考侧 N7：补 approx 并对布尔容器置 true；N8：示例里 radar.feed 对 radar.dish 的 inside）=====
    let my_facts_arr = mine["facts"].as_array().cloned().unwrap_or_default();
    let ref_facts_arr = reft["facts"].as_array().cloned().unwrap_or_default();
    let kind_of = |p: &str| -> String {
        ref_nodes
            .get(p)
            .and_then(|n| n["kind"].as_str())
            .unwrap_or("")
            .to_string()
    };
    let mut fact_ok = 0usize;
    if my_facts_arr.len() != ref_facts_arr.len() {
        rep.fail(format!(
            "  facts 条数 {} vs {}",
            my_facts_arr.len(),
            ref_facts_arr.len()
        ));
    }
    for (i, f) in my_facts_arr.iter().enumerate() {
        let r = match ref_facts_arr.get(i) {
            Some(r) => r,
            None => break,
        };
        let (ra, rb) = (r["a"].as_str().unwrap_or(""), r["b"].as_str().unwrap_or(""));
        let mut rrel = r["rel"].as_str().unwrap_or("").to_string();
        let mut rtext = r["text"].as_str().unwrap_or("").to_string();
        if ra == "radar.feed" && rb == "radar.dish" && rrel == "intersects" {
            rrel = "inside".to_string();
            rtext = format!("{} inside {}", ra, rb);
        }
        let is_bool = |k: &str| matches!(k, "union" | "subtract" | "intersect");
        let rapprox =
            r["approx"].as_bool().unwrap_or(false) || is_bool(&kind_of(ra)) || is_bool(&kind_of(rb));
        let mut bad = false;
        if f["rel"] != rrel
            || f["a"] != r["a"]
            || f["b"] != r["b"]
            || f["text"].as_str() != Some(rtext.as_str())
        {
            bad = true;
            rep.fail(format!("  DIFF fact #{} {} vs {}", i, f["text"], rtext));
        }
        if f["approx"].as_bool().unwrap_or(false) != rapprox {
            bad = true;
            rep.fail(format!(
                "  DIFF fact #{} {} approx {} vs {}",
                i, f["text"], f["approx"], rapprox
            ));
        }
        let (g1, g2) = (
            f["gap"].as_f64().unwrap_or(f64::NAN),
            r["gap"].as_f64().unwrap_or(f64::NAN),
        );
        if (g1 - g2).abs() > 0.5e-4 + TAU_LEN || g1.is_nan() || g2.is_nan() {
            bad = true;
            rep.fail(format!("  DIFF fact #{} {} gap {} vs {}", i, f["text"], g1, g2));
        }
        let (e1, e2) = (
            f["eps"].as_f64().unwrap_or(f64::NAN),
            r["eps"].as_f64().unwrap_or(f64::NAN),
        );
        if (e1 - e2).abs() > TAU_LEN || e1.is_nan() || e2.is_nan() {
            bad = true;
            rep.fail(format!("  DIFF fact #{} eps {} vs {}", i, e1, e2));
        }
        if !bad {
            fact_ok += 1;
        }
    }
    if mine["factsSkipped"] != reft["factsSkipped"] || mine["factProviders"] != reft["factProviders"] {
        rep.fail("  factsSkipped / factProviders 不同".to_string());
    }
    rep.lines.push(format!(
        "facts: {} / {} 一致（参考 {}）",
        fact_ok,
        my_facts_arr.len(),
        ref_facts_arr.len()
    ));
    // ===== 语义层（9.4.3；参考侧 N10 / N14：members 按节点顺序重排）=====
    let node_pos: BTreeMap<&str, usize> = ref_order
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_str(), i))
        .collect();
    let by_node_order = |v: &J| -> Vec<String> {
        let mut xs: Vec<String> = v
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        xs.sort_by_key(|p| node_pos.get(p.as_str()).copied().unwrap_or(usize::MAX));
        xs
    };
    let mut sem_ok = 0usize;
    let mut sem_total = 0usize;
    for k in ["concepts", "patterns", "checks", "variants"] {
        let ma = mine[k].as_array().cloned().unwrap_or_default();
        let ra = reft[k].as_array().cloned().unwrap_or_default();
        if ma.len() != ra.len() {
            rep.fail(format!("  {} 条数 {} vs {}", k, ma.len(), ra.len()));
        }
        for (i, (a, b)) in ma.iter().zip(ra.iter()).enumerate() {
            sem_total += 1;
            let mut bad = false;
            let label = a["name"]
                .as_str()
                .or(a["message"].as_str())
                .or(a["use"].as_str())
                .unwrap_or("");
            let fields: &[&str] = match k {
                "concepts" => &["name", "parent", "line", "src"],
                "patterns" => &["name", "line", "roles", "matches"],
                "checks" => &["line", "message", "ok", "violators"],
                _ => &["line", "use", "src"],
            };
            for f in fields {
                if a[*f] != b[*f] {
                    bad = true;
                    rep.fail(format!(
                        "  DIFF {} #{} {} {} {} vs {}",
                        k, i, label, f, a[*f], b[*f]
                    ));
                }
            }
            if k == "concepts" || k == "variants" {
                let x: Vec<String> = a["members"]
                    .as_array()
                    .map(|v| v.iter().filter_map(|s| s.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                let y = by_node_order(&b["members"]);
                if x != y {
                    bad = true;
                    rep.fail(format!(
                        "  DIFF {} #{} {} members {:?} vs {:?}",
                        k, i, label, x, y
                    ));
                }
            }
            if a.get("error").is_some() != b.get("error").is_some() {
                bad = true;
                rep.fail(format!(
                    "  DIFF {} #{} {} error {} vs {}",
                    k, i, label, a["error"], b["error"]
                ));
            }
            if !bad {
                sem_ok += 1;
            }
        }
    }
    rep.lines
        .push(format!("semantics: {} / {} 条目一致", sem_ok, sem_total));
    // ===== 诊断 =====
    let me = mine["errors"].as_array().map(|a| a.len()).unwrap_or(0);
    let re = reft["errors"].as_array().map(|a| a.len()).unwrap_or(0);
    let mw = mine["warnings"].as_array().map(|a| a.len()).unwrap_or(0);
    let rw = reft["warnings"].as_array().map(|a| a.len()).unwrap_or(0);
    rep.lines.push(format!(
        "errors: mine {} ref {}; warnings: mine {} ref {}",
        me, re, mw, rw
    ));
    for e in mine["errors"].as_array().unwrap().iter().take(15) {
        rep.lines.push(format!(
            "  ERR {}:{} {} {}",
            e["file"], e["line"], e["path"], e["message"]
        ));
    }
    rep
}
