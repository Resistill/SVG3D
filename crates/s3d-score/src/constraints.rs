//! 具名约束（docs/PLAN.md 第 6 节）：原型里的硬规则改成有名字的过滤器。每条约束可以关、带选择器
//! （默认空集 = 不约束任何物体），作用在评分的逐物体结果上；报告列出哪些物体因为哪条约束被否。
//! M2 只定义约束与判定；M3 的候选筛选把它当过滤器用（所有候选都被否时发射现有最好写法并标记）。
//!
//! 配置（工程 project.json 的 `constraints`，JSON）：
//! ```json
//! { "solidFill": { "maxWrong": 0.3, "minVolume": 8, "select": ["hangar_body", "tower.*"] },
//!   "keepLargeParts": { "minSize": 1.0, "select": ["*"], "enabled": false } }
//! ```
//! 选择器是路径模式：`*` 匹配任意一段（不含点），`**` 匹配任意多段；省略 `select` 等于空集。

use serde_json::{json, Value as J};

/// 一条约束
#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    /// 实心体块不能把源模型里空的地方填实：错填比例 ≤ maxWrong；只看体积 ≥ minVolume（场景单位³）的物体
    SolidFill { max_wrong: f64, min_volume: f64 },
    /// 大部件不能被丢掉或缩成小东西：包围盒最长边 ≥ minSize 的源部件必须有物体负责解释（explainedBy 非空）
    KeepLargeParts { min_size: f64 },
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub name: String,
    pub rule: Rule,
    pub enabled: bool,
    /// 路径模式；空 = 不约束任何物体
    pub select: Vec<String>,
}

/// 被否的一项
#[derive(Clone, Debug, PartialEq)]
pub struct Rejection {
    pub constraint: String,
    pub path: String,
    pub reason: String,
}

fn num(v: &J, key: &str, d: f64) -> f64 {
    v[key].as_f64().unwrap_or(d)
}

/// 从 JSON 配置解析；未知的约束名、缺参数的条目报错
pub fn parse(cfg: &J) -> Result<Vec<Constraint>, String> {
    let obj = match cfg.as_object() {
        Some(o) => o,
        None => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for (name, v) in obj {
        let rule = match name.as_str() {
            "solidFill" => Rule::SolidFill {
                max_wrong: num(v, "maxWrong", 0.3),
                min_volume: num(v, "minVolume", 8.0),
            },
            "keepLargeParts" => Rule::KeepLargeParts {
                min_size: num(v, "minSize", 1.0),
            },
            other => return Err(format!("未知的约束 {}", other)),
        };
        let select: Vec<String> = v["select"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        out.push(Constraint {
            name: name.clone(),
            rule,
            enabled: v["enabled"].as_bool().unwrap_or(true),
            select,
        });
    }
    Ok(out)
}

/// 路径模式匹配：按 `.` 分段，`*` 一段、`**` 任意多段
pub fn path_matches(pattern: &str, path: &str) -> bool {
    fn go(p: &[&str], s: &[&str]) -> bool {
        match (p.first(), s.first()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(&"**"), _) => go(&p[1..], s) || (!s.is_empty() && go(p, &s[1..])),
            (Some(_), None) => false,
            (Some(&x), Some(&y)) => (x == "*" || x == y) && go(&p[1..], &s[1..]),
        }
    }
    let p: Vec<&str> = pattern.split('.').collect();
    let s: Vec<&str> = path.split('.').collect();
    go(&p, &s)
}

fn selected(c: &Constraint, path: &str) -> bool {
    c.select.iter().any(|p| path_matches(p, path))
}

/// 对一份评分结果（`score()` 的 JSON，需带 objects / sources，即 `--full`）判定；源部件按 `explainedBy` 的物体路径选
pub fn check(
    constraints: &[Constraint],
    score: &J,
    part_size: &dyn Fn(&str) -> Option<f64>,
) -> Vec<Rejection> {
    let mut out = Vec::new();
    for c in constraints.iter().filter(|c| c.enabled) {
        match c.rule {
            Rule::SolidFill {
                max_wrong,
                min_volume,
            } => {
                for o in score["objects"].as_array().into_iter().flatten() {
                    let path = o["path"].as_str().unwrap_or("");
                    if !selected(c, path) {
                        continue;
                    }
                    let vol = o["vol"].as_f64().unwrap_or(0.0);
                    let wrong = o["volWrong"].as_f64().unwrap_or(0.0);
                    if vol >= min_volume && wrong > max_wrong {
                        out.push(Rejection {
                            constraint: c.name.clone(),
                            path: path.to_string(),
                            reason: format!("体积 {:.3} 的实体错填比例 {:.3} > {}", vol, wrong, max_wrong),
                        });
                    }
                }
            }
            Rule::KeepLargeParts { min_size } => {
                for s in score["sources"].as_array().into_iter().flatten() {
                    if s["skipped"].as_bool().unwrap_or(false) {
                        continue;
                    }
                    let name = s["name"].as_str().unwrap_or("");
                    let who = s["explainedBy"].as_str().unwrap_or("");
                    // 选择器作用在源部件名或负责解释它的物体路径上
                    if !(selected(c, name) || (!who.is_empty() && selected(c, who))) {
                        continue;
                    }
                    let size = part_size(name).unwrap_or(0.0);
                    if size >= min_size && who.is_empty() {
                        out.push(Rejection {
                            constraint: c.name.clone(),
                            path: name.to_string(),
                            reason: format!("最长边 {:.3} 的源部件没有物体解释它", size),
                        });
                    }
                }
            }
        }
    }
    out
}

/// 报告：每条约束通过与否与被否的项
pub fn report(constraints: &[Constraint], rejections: &[Rejection]) -> J {
    let items: Vec<J> = constraints
        .iter()
        .map(|c| {
            let rej: Vec<J> = rejections
                .iter()
                .filter(|r| r.constraint == c.name)
                .map(|r| json!({ "path": r.path, "reason": r.reason }))
                .collect();
            json!({ "name": c.name, "enabled": c.enabled, "select": c.select, "passed": rej.is_empty(), "rejected": rej })
        })
        .collect();
    json!({ "passed": rejections.is_empty(), "constraints": items })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns() {
        assert!(path_matches("*", "a"));
        assert!(!path_matches("*", "a.b"));
        assert!(path_matches("tower.*", "tower.deck"));
        assert!(path_matches("**", "a.b.c"));
        assert!(path_matches("a.**.leg", "a.legs.0.leg"));
        assert!(!path_matches("a.*.leg", "a.legs.0.leg"));
    }

    #[test]
    fn solid_fill_and_large_parts() {
        let cfg = json!({
            "solidFill": { "maxWrong": 0.3, "minVolume": 8, "select": ["**"] },
            "keepLargeParts": { "minSize": 1.0, "select": ["**"] },
            "off": null
        });
        assert!(parse(&cfg).is_err());
        let cfg = json!({
            "solidFill": { "maxWrong": 0.3, "minVolume": 8, "select": ["**"] },
            "keepLargeParts": { "minSize": 1.0, "select": ["**"] }
        });
        let cs = parse(&cfg).unwrap();
        let score = json!({
            "objects": [
                { "path": "hangar_body", "vol": 100.0, "volWrong": 0.5 },
                { "path": "small", "vol": 1.0, "volWrong": 0.9 },
                { "path": "fine", "vol": 50.0, "volWrong": 0.1 }
            ],
            "sources": [
                { "name": "big", "explainedBy": null },
                { "name": "tiny", "explainedBy": null },
                { "name": "ok", "explainedBy": "fine" }
            ]
        });
        let size = |n: &str| Some(if n == "big" { 5.0 } else { 0.2 });
        let rej = check(&cs, &score, &size);
        let paths: Vec<&str> = rej.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, vec!["hangar_body", "big"]);
        let r = report(&cs, &rej);
        assert_eq!(r["passed"], false);
        assert_eq!(r["constraints"][0]["rejected"].as_array().unwrap().len(), 1);
        // 选择器为空：不约束
        let none = parse(&json!({ "solidFill": { "maxWrong": 0.3 } })).unwrap();
        assert!(check(&none, &score, &size).is_empty());
    }
}
