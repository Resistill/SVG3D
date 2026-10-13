//! 读取部件阶段的源侧决定（docs/PLAN.md 第 5 节首行、4.2.1 的"读取部件"行）：
//! drop / role / merge / type / keep-mesh / shape，选择器是 sources（部件 id）或 rule（源部件谓词）。
//! 优先级（4.6）：具体对象意图 > 规则型意图，同类之间 id 大的压小的；每个部件的每个键取优先级最高的决定。
//! 两条具体对象意图对同一部件、同一键给不同值记为冲突（写进报告，仍按优先级取值）；规则型之间的重叠不算冲突
//! （样本意图就靠 id 顺序复刻原型的判定顺序）。`drop` 不再无条件胜出：更高优先级的具体对象决定（role / shape /
//! type / keep-mesh / merge）压过规则型的 drop。
//! 没有任何意图时：不区分角色（role 为 None），全部部件照常。

use crate::intent::Intent;
use regex::Regex;
use s3d_geom::mesh::Part;
use serde_json::{json, Value as J};
use std::collections::{BTreeMap, BTreeSet};

/// 执行决定后的部件：原部件加上决定给的标记
#[derive(Clone, Debug)]
pub struct SourcePart {
    pub part: Part,
    /// `shape` / `keep-mesh` 指定的写法（种类名或 "mesh"）
    pub force: Option<String>,
    /// `merge` 指定的名字
    pub force_name: Option<String>,
    /// `type` 指定的类型名
    pub type_name: Option<String>,
    /// 作用到它上面的意图 id（去重、升序）
    pub intents: Vec<u64>,
}

/// 冲突：(部件 id, 键, [(意图 id, 值)])
pub type Conflict = (String, String, Vec<(u64, String)>);

/// 执行报告
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// (意图 id, op, 作用的部件数)
    pub applied: Vec<(u64, String, usize)>,
    /// (意图 id, op, 原因)
    pub unsatisfied: Vec<(u64, String, String)>,
    /// (意图 id, 找不到的源部件 id)
    pub dangling: Vec<(u64, String)>,
    /// (部件 id, 键, [(意图 id, 值)])
    pub conflicts: Vec<Conflict>,
}

impl Report {
    pub fn to_json(&self) -> J {
        json!({
            "applied": self.applied.iter().map(|(i, op, n)| json!({ "intent": i, "op": op, "parts": n })).collect::<Vec<_>>(),
            "unsatisfied": self.unsatisfied.iter().map(|(i, op, why)| json!({ "intent": i, "op": op, "why": why })).collect::<Vec<_>>(),
            "dangling": self.dangling.iter().map(|(i, s)| json!({ "intent": i, "source": s })).collect::<Vec<_>>(),
            "conflicts": self.conflicts.iter().map(|(p, k, vs)| json!({ "part": p, "key": k, "values": vs.iter().map(|(i, v)| json!({ "intent": i, "value": v })).collect::<Vec<_>>() })).collect::<Vec<_>>(),
        })
    }
}

struct CompiledRule {
    name: Option<Regex>,
    material: Option<Regex>,
    meta: Vec<(String, Regex)>,
}

fn compile_rule(r: &crate::intent::Rule) -> Option<CompiledRule> {
    let re = |s: &str| Regex::new(s).ok();
    Some(CompiledRule {
        name: match &r.name {
            Some(s) => Some(re(s)?),
            None => None,
        },
        material: match &r.material {
            Some(s) => Some(re(s)?),
            None => None,
        },
        meta: r
            .meta
            .iter()
            .map(|(k, v)| re(v).map(|x| (k.clone(), x)))
            .collect::<Option<Vec<_>>>()?,
    })
}

fn rule_matches(r: &CompiledRule, p: &Part) -> bool {
    if let Some(n) = &r.name {
        match &p.name {
            Some(name) if n.is_match(name) => {}
            _ => return false,
        }
    }
    if let Some(m) = &r.material {
        if !m.is_match(p.material.name.as_deref().unwrap_or("")) {
            return false;
        }
    }
    r.meta
        .iter()
        .all(|(k, x)| p.meta.get(k).map(|v| x.is_match(v)).unwrap_or(false))
}

/// 一个部件的全部 id：uid、name、from
fn ids_of(p: &Part) -> Vec<String> {
    [p.uid.as_ref(), p.name.as_ref(), p.from.as_ref()]
        .into_iter()
        .flatten()
        .cloned()
        .collect()
}

/// 一个待应用的决定（已展开到具体部件序号）
struct Pending {
    intent: u64,
    priority: (u8, u64),
    op: String,
    value: String,
    args: serde_json::Map<String, J>,
    parts: Vec<usize>,
}

/// 执行源侧决定。`intents` 里只有 accepted 的被读；返回新的部件列表与报告
pub fn apply_source_decisions(parts: Vec<Part>, intents: &[Intent]) -> (Vec<SourcePart>, Report) {
    let mut report = Report::default();
    let mut by_id: BTreeMap<String, usize> = BTreeMap::new();
    for (i, p) in parts.iter().enumerate() {
        for id in ids_of(p) {
            by_id.entry(id).or_insert(i);
        }
    }
    // 展开每条决定到部件
    let mut pending: Vec<Pending> = Vec::new();
    let mut active: Vec<&Intent> = intents.iter().filter(|it| it.is_active()).collect();
    active.sort_by_key(|it| it.priority());
    for it in &active {
        let rule = it.selection.rule.as_ref().map(compile_rule);
        if let Some(None) = rule {
            report
                .unsatisfied
                .push((it.id, "rule".into(), "rule 选择器里的正则不合法".into()));
            continue;
        }
        let rule = rule.flatten();
        for d in &it.formal.decisions {
            if crate::intent::stage_of(&d.op) != Some(crate::intent::Stage::Parts) {
                continue;
            }
            // 选择器：决定自带的 sources > 意图的 sources > 意图的 rule
            let sources = d.sources().unwrap_or_else(|| it.selection.sources.clone());
            let targets: Vec<usize> = if !sources.is_empty() {
                let mut t = Vec::new();
                for s in &sources {
                    match by_id.get(s) {
                        Some(&i) => {
                            if !t.contains(&i) {
                                t.push(i);
                            }
                        }
                        None => report.dangling.push((it.id, s.clone())),
                    }
                }
                t
            } else if let Some(r) = &rule {
                if !matches!(d.op.as_str(), "drop" | "role" | "type") {
                    report
                        .unsatisfied
                        .push((it.id, d.op.clone(), "这个决定不能用 rule 选择器".into()));
                    continue;
                }
                parts
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| rule_matches(r, p))
                    .map(|(i, _)| i)
                    .collect()
            } else {
                report
                    .unsatisfied
                    .push((it.id, d.op.clone(), "没有 sources / rule 选择器".into()));
                continue;
            };
            if targets.is_empty() {
                if sources.is_empty() {
                    report.applied.push((it.id, d.op.clone(), 0));
                }
                continue;
            }
            let value = match d.op.as_str() {
                "role" | "shape" => d.str_arg("value").unwrap_or("").to_string(),
                "type" | "merge" => d.str_arg("name").unwrap_or("").to_string(),
                _ => "true".to_string(),
            };
            pending.push(Pending {
                intent: it.id,
                priority: it.priority(),
                op: d.op.clone(),
                value,
                args: d.args.clone(),
                parts: targets,
            });
        }
    }
    // 每个部件、每个键：取优先级最高的；具体对象意图之间值不同记冲突
    #[derive(Default)]
    struct Slot {
        best: Option<(u64, (u8, u64), String, usize)>, // (意图, 优先级, 值, pending 序号)
        concrete_values: Vec<(u64, String)>,
    }
    let mut slots: BTreeMap<(usize, String), Slot> = BTreeMap::new();
    let mut touched: BTreeMap<usize, BTreeSet<u64>> = BTreeMap::new();
    for (k, pd) in pending.iter().enumerate() {
        for &pi in &pd.parts {
            touched.entry(pi).or_default().insert(pd.intent);
            let key = match pd.op.as_str() {
                "shape" | "keep-mesh" => "force".to_string(),
                other => other.to_string(),
            };
            let slot = slots.entry((pi, key)).or_default();
            if pd.priority.0 == 1 {
                slot.concrete_values.push((pd.intent, pd.value.clone()));
            }
            if slot.best.as_ref().map(|b| pd.priority > b.1).unwrap_or(true) {
                slot.best = Some((pd.intent, pd.priority, pd.value.clone(), k));
            }
        }
        report.applied.push((pd.intent, pd.op.clone(), pd.parts.len()));
    }
    for ((pi, key), slot) in &slots {
        let distinct: BTreeSet<&str> = slot.concrete_values.iter().map(|(_, v)| v.as_str()).collect();
        if distinct.len() > 1 {
            report
                .conflicts
                .push((parts[*pi].label(*pi), key.clone(), slot.concrete_values.clone()));
        }
    }
    // 应用
    let mut out: Vec<Option<SourcePart>> = parts
        .into_iter()
        .map(|p| {
            Some(SourcePart {
                part: p,
                force: None,
                force_name: None,
                type_name: None,
                intents: Vec::new(),
            })
        })
        .collect();
    for (pi, ids) in &touched {
        if let Some(sp) = out[*pi].as_mut() {
            sp.intents = ids.iter().copied().collect();
        }
    }
    // drop：被更高优先级的具体对象决定压过时不丢
    let mut dropped: BTreeSet<usize> = BTreeSet::new();
    for ((pi, key), slot) in &slots {
        if key != "drop" {
            continue;
        }
        let (_, prio, _, _) = slot.best.as_ref().unwrap();
        let overridden = slots.iter().any(|((pj, k2), s2)| {
            pj == pi
                && k2 != "drop"
                && s2
                    .best
                    .as_ref()
                    .map(|b| b.1 .0 == 1 && b.1 > *prio)
                    .unwrap_or(false)
        });
        if !overridden {
            dropped.insert(*pi);
        }
    }
    for ((pi, key), slot) in &slots {
        if dropped.contains(pi) {
            continue;
        }
        let (_, _, value, k) = slot.best.as_ref().unwrap();
        let sp = match out[*pi].as_mut() {
            Some(sp) => sp,
            None => continue,
        };
        match key.as_str() {
            "role" => sp.part.role = Some(value.clone()),
            "force" => {
                let pd = &pending[*k];
                sp.force = Some(if pd.op == "keep-mesh" {
                    "mesh".to_string()
                } else {
                    value.clone()
                });
                if pd.op == "keep-mesh" {
                    sp.part.mesh = None;
                    sp.part.matrix = None;
                }
            }
            "type" => sp.type_name = Some(value.clone()),
            _ => {}
        }
    }
    // merge：同一条 merge 决定的部件合成一个（三角形合并，来源记第一个），其余丢掉；按决定顺序
    let mut merged_away: BTreeSet<usize> = BTreeSet::new();
    for pd in pending.iter().filter(|p| p.op == "merge") {
        let members: Vec<usize> = pd
            .parts
            .iter()
            .copied()
            .filter(|i| !dropped.contains(i) && !merged_away.contains(i))
            .collect();
        if members.len() < 2 {
            if members.len() == 1 {
                // 单个部件的 merge 只是改名 / 指定形状
                let sp = out[members[0]].as_mut().unwrap();
                if !pd.value.is_empty() {
                    sp.force_name = Some(pd.value.clone());
                }
                if let Some(s) = pd.args.get("shape").and_then(|v| v.as_str()) {
                    sp.force = Some(s.to_string());
                }
            } else {
                report
                    .unsatisfied
                    .push((pd.intent, "merge".into(), "要合并的部件都已被丢掉".into()));
            }
            continue;
        }
        let first = members[0];
        let mut tris = Vec::new();
        let mut intents: BTreeSet<u64> = BTreeSet::new();
        for &i in &members {
            let sp = out[i].as_ref().unwrap();
            tris.extend_from_slice(&sp.part.tris);
            intents.extend(sp.intents.iter().copied());
        }
        let sp = out[first].as_mut().unwrap();
        sp.part.tris = tris;
        sp.part.mesh = None;
        sp.part.matrix = None;
        sp.part
            .meta
            .insert("merged".to_string(), members.len().to_string());
        if !pd.value.is_empty() {
            sp.force_name = Some(pd.value.clone());
        }
        if let Some(s) = pd.args.get("shape").and_then(|v| v.as_str()) {
            sp.force = Some(s.to_string());
        }
        sp.intents = intents.into_iter().collect();
        for &i in &members[1..] {
            merged_away.insert(i);
        }
    }
    let result: Vec<SourcePart> = out
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !dropped.contains(i) && !merged_away.contains(i))
        .filter_map(|(_, sp)| sp)
        .collect();
    (result, report)
}

/// 源侧事实：部件 id（uid，否则 name）→ 角色；没有角色的部件按 "structure"（与原型 state/source-roles.json 同口径）
pub fn source_roles(parts: &[SourcePart]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for sp in parts {
        let id = match sp.part.uid.clone().or_else(|| sp.part.name.clone()) {
            Some(id) => id,
            None => continue,
        };
        out.insert(
            id,
            sp.part.role.clone().unwrap_or_else(|| "structure".to_string()),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use s3d_geom::mesh::Material;

    fn part(name: &str) -> Part {
        Part {
            name: Some(name.to_string()),
            uid: Some(name.to_string()),
            from: None,
            tris: vec![[s3d_geom::Vec3::ZERO, s3d_geom::Vec3::X, s3d_geom::Vec3::Y]],
            material: Material {
                name: Some("M_Concrete".into()),
                color: "#ffffff".into(),
                metal: None,
                rough: None,
            },
            mesh: Some("1:0".into()),
            matrix: None,
            role: None,
            meta: BTreeMap::new(),
        }
    }

    fn intent(s: &str) -> Intent {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn priority_and_conflicts() {
        let parts = vec![
            part("Wall_01"),
            part("Blocker_Wall"),
            part("Lamp_01"),
            part("Fog_01"),
        ];
        let intents = vec![
            intent(
                r#"{"id":1,"author":"ai","status":{"state":"accepted"},"selection":{"rule":{"name":"Wall"}},"formal":{"decisions":[{"op":"role","value":"structure"}]}}"#,
            ),
            intent(
                r#"{"id":2,"author":"ai","status":{"state":"accepted"},"selection":{"rule":{"name":"Blocker"}},"formal":{"decisions":[{"op":"role","value":"collision"}]}}"#,
            ),
            intent(
                r#"{"id":3,"author":"ai","status":{"state":"accepted"},"selection":{"rule":{"name":"Fog"}},"formal":{"decisions":[{"op":"drop"}]}}"#,
            ),
            intent(
                r#"{"id":4,"status":{"state":"accepted"},"selection":{"sources":["Lamp_01"]},"formal":{"decisions":[{"op":"role","value":"decor"}]}}"#,
            ),
            intent(
                r#"{"id":5,"status":{"state":"accepted"},"selection":{"sources":["Lamp_01","Ghost"]},"formal":{"decisions":[{"op":"role","value":"structure"}]}}"#,
            ),
            intent(
                r#"{"id":6,"status":{"state":"accepted"},"selection":{"sources":["Fog_01"]},"formal":{"decisions":[{"op":"role","value":"helper"}]}}"#,
            ),
            intent(
                r#"{"id":7,"author":"ai","status":{"state":"proposed"},"selection":{"rule":{"name":"."}},"formal":{"decisions":[{"op":"drop"}]}}"#,
            ),
        ];
        let (out, rep) = apply_source_decisions(parts, &intents);
        let roles = source_roles(&out);
        assert_eq!(roles["Wall_01"], "structure");
        assert_eq!(roles["Blocker_Wall"], "collision"); // 规则 2 的 id 大，压过规则 1
        assert_eq!(roles["Lamp_01"], "structure"); // 具体意图 5 压过 4，并记冲突
        assert_eq!(roles["Fog_01"], "helper"); // 具体意图 6 压过规则型 drop
        assert_eq!(out.len(), 4);
        assert_eq!(rep.dangling, vec![(5, "Ghost".to_string())]);
        assert_eq!(rep.conflicts.len(), 1);
        assert_eq!(rep.conflicts[0].0, "Lamp_01");
        assert_eq!(
            out.iter()
                .find(|s| s.part.name.as_deref() == Some("Lamp_01"))
                .unwrap()
                .intents,
            vec![4, 5]
        );
    }

    #[test]
    fn merge_type_keep_mesh() {
        let parts = vec![part("a"), part("b"), part("c"), part("d")];
        let intents = vec![
            intent(
                r#"{"id":1,"status":{"state":"accepted"},"selection":{"sources":["a","b"]},"formal":{"decisions":[{"op":"merge","name":"hangar","shape":"box"}]}}"#,
            ),
            intent(
                r#"{"id":2,"status":{"state":"accepted"},"selection":{"sources":["c","d"]},"formal":{"decisions":[{"op":"type","name":"house"}]}}"#,
            ),
            intent(
                r#"{"id":3,"status":{"state":"accepted"},"selection":{"sources":["c"]},"formal":{"decisions":[{"op":"keep-mesh"}]}}"#,
            ),
        ];
        let (out, rep) = apply_source_decisions(parts, &intents);
        assert_eq!(out.len(), 3);
        let h = &out[0];
        assert_eq!(h.force_name.as_deref(), Some("hangar"));
        assert_eq!(h.force.as_deref(), Some("box"));
        assert_eq!(h.part.tris.len(), 2);
        assert_eq!(h.part.meta["merged"], "2");
        assert_eq!(out[1].type_name.as_deref(), Some("house"));
        assert_eq!(out[1].force.as_deref(), Some("mesh"));
        assert_eq!(out[2].type_name.as_deref(), Some("house"));
        assert!(rep.unsatisfied.is_empty() && rep.conflicts.is_empty());
    }
}
