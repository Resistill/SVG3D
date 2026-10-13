//! 意图（docs/PLAN.md 第 4 节）的数据形状与词汇表校验。意图是 JSON，不是语言；这里只放逆向与工程都要用的部分：
//! 形状（4.2）、决定的词汇表（4.2.1）、选择器、优先级（4.6）。文件读写、重绑与生命周期在 s3d-project。
//! 未知字段原样保留（`extra`），所以读回再写出不丢东西。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as J};
use std::collections::BTreeMap;

/// 角色的取值表
pub const ROLES: [&str; 6] = ["structure", "decor", "terrain", "decal", "collision", "helper"];
/// `shape` 决定可指定的种类
pub const SHAPES: [&str; 9] = [
    "box", "cyl", "cone", "sphere", "wedge", "plate", "hull", "revolve", "mesh",
];

/// 规则选择器：源部件谓词（正则，只在读取部件阶段求值）
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

/// 几何指纹：形状种类 + 尺寸 + 世界位置
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fingerprint {
    pub kind: String,
    pub size: [f64; 3],
    pub center: [f64; 3],
}

/// 区域：世界坐标盒
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Region {
    #[serde(rename = "box")]
    pub box_: [f64; 6],
    #[serde(default = "scene_unit")]
    pub unit: String,
}

fn scene_unit() -> String {
    "scene".to_string()
}

/// 锚定（4.3）：多种锚并存
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingerprints: Vec<Fingerprint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<Rule>,
}

/// 一条决定：`op` 加该操作的参数（4.2.1）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    pub op: String,
    #[serde(flatten)]
    pub args: Map<String, J>,
}

impl Decision {
    pub fn str_arg(&self, k: &str) -> Option<&str> {
        self.args.get(k).and_then(|v| v.as_str())
    }
    pub fn num_arg(&self, k: &str) -> Option<f64> {
        self.args.get(k).and_then(|v| v.as_f64())
    }
    /// 决定自带的 `sources`（覆盖意图的选择器）
    pub fn sources(&self) -> Option<Vec<String>> {
        self.args
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Formal {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<Decision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unformalized: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Bound {
    pub paths: usize,
    pub sources: usize,
    pub fingerprints: usize,
    pub suspicious: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Status {
    #[serde(default = "proposed")]
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound: Option<Bound>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dangling: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satisfied: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<J>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<J>,
    #[serde(flatten)]
    pub extra: Map<String, J>,
}

fn proposed() -> String {
    "proposed".to_string()
}

impl Default for Status {
    fn default() -> Status {
        Status {
            state: proposed(),
            bound: None,
            dangling: Vec::new(),
            satisfied: None,
            conflicts: Vec::new(),
            effect: None,
            extra: Map::new(),
        }
    }
}

/// 一条意图（intents/NNN.json）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub id: u64,
    #[serde(default)]
    pub created: String,
    #[serde(default = "human")]
    pub author: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub selection: Selection,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub formal: Formal,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<J>,
    #[serde(flatten)]
    pub extra: Map<String, J>,
}

fn human() -> String {
    "human".to_string()
}

impl Intent {
    /// 重跑时读的意图：accepted（AI 的意图在被人接受前是 proposed，不读）
    pub fn is_active(&self) -> bool {
        self.status.state == "accepted"
    }
    /// 规则型（选择器是源部件谓词）
    pub fn is_rule(&self) -> bool {
        self.selection.rule.is_some()
    }
    /// 优先级（4.6）：具体对象意图 > 规则型意图；同类之间 id 大的压小的
    pub fn priority(&self) -> (u8, u64) {
        (if self.is_rule() { 0 } else { 1 }, self.id)
    }
}

/// 决定作用的阶段
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// 读取部件（按源部件；选择器 sources / rule）
    Parts,
    /// 逆向（按物体；选择器 paths / region）
    Objects,
}

/// 词汇表（4.2.1）：op → 作用阶段
pub fn stage_of(op: &str) -> Option<Stage> {
    match op {
        "drop" | "role" | "merge" | "type" | "keep-mesh" | "shape" => Some(Stage::Parts),
        "hollow" | "tol" | "lock" | "array" | "mirror" => Some(Stage::Objects),
        _ => None,
    }
}

/// 校验一条意图：返回问题列表（空 = 合法）。不在词汇表里的操作、缺参数、选择器与阶段不配都算问题
pub fn validate(it: &Intent) -> Vec<String> {
    let mut out = Vec::new();
    if !matches!(it.status.state.as_str(), "proposed" | "accepted" | "retired") {
        out.push(format!(
            "意图 {}：status.state 只能是 proposed / accepted / retired",
            it.id
        ));
    }
    if !matches!(it.author.as_str(), "human" | "ai") {
        out.push(format!("意图 {}：author 只能是 human / ai", it.id));
    }
    let sel = &it.selection;
    if let Some(r) = &sel.rule {
        if r.name.is_none() && r.material.is_none() && r.meta.is_empty() {
            out.push(format!(
                "意图 {}：rule 选择器要有 name / material / meta 之一",
                it.id
            ));
        }
        for (what, re) in [("name", r.name.as_deref()), ("material", r.material.as_deref())] {
            if let Some(re) = re {
                if regex::Regex::new(re).is_err() {
                    out.push(format!("意图 {}：rule.{} 不是合法的正则：{}", it.id, what, re));
                }
            }
        }
        for (k, v) in &r.meta {
            if regex::Regex::new(v).is_err() {
                out.push(format!("意图 {}：rule.meta.{} 不是合法的正则：{}", it.id, k, v));
            }
        }
    }
    for d in &it.formal.decisions {
        let stage = match stage_of(&d.op) {
            Some(s) => s,
            None => {
                out.push(format!("意图 {}：不在词汇表里的决定 {}", it.id, d.op));
                continue;
            }
        };
        let has_sources = !sel.sources.is_empty() || d.sources().map(|s| !s.is_empty()).unwrap_or(false);
        match stage {
            Stage::Parts => {
                let rule_ok = sel.rule.is_some() && matches!(d.op.as_str(), "drop" | "role" | "type");
                if !has_sources && !rule_ok {
                    out.push(format!(
                        "意图 {}：决定 {} 作用在源部件上，要有 sources 选择器{}",
                        it.id,
                        d.op,
                        if matches!(d.op.as_str(), "drop" | "role" | "type") {
                            "或 rule 选择器"
                        } else {
                            ""
                        }
                    ));
                }
            }
            Stage::Objects => {
                let region_ok = sel.region.is_some() && d.op == "tol";
                if sel.paths.is_empty() && !region_ok {
                    out.push(format!(
                        "意图 {}：决定 {} 作用在物体上，要有 paths 选择器",
                        it.id, d.op
                    ));
                }
            }
        }
        match d.op.as_str() {
            "role" => match d.str_arg("value") {
                Some(v) if ROLES.contains(&v) => {}
                _ => out.push(format!(
                    "意图 {}：role 的 value 只能是 {}",
                    it.id,
                    ROLES.join(" / ")
                )),
            },
            "shape" => match d.str_arg("value") {
                Some(v) if SHAPES.contains(&v) => {}
                _ => out.push(format!(
                    "意图 {}：shape 的 value 只能是 {}",
                    it.id,
                    SHAPES.join(" / ")
                )),
            },
            "merge" => {
                if let Some(s) = d.str_arg("shape") {
                    if !SHAPES.contains(&s) {
                        out.push(format!(
                            "意图 {}：merge 的 shape 只能是 {}",
                            it.id,
                            SHAPES.join(" / ")
                        ));
                    }
                }
            }
            "hollow" => {
                if let Some(f) = d.args.get("floors") {
                    if f.as_u64().is_none() {
                        out.push(format!("意图 {}：hollow 的 floors 要是非负整数", it.id));
                    }
                }
            }
            "tol" => {
                if d.num_arg("value").map(|v| v > 0.0) != Some(true) {
                    out.push(format!("意图 {}：tol 要有正数 value", it.id));
                }
            }
            "lock" => match d.str_arg("value") {
                Some("place") | Some("shape") | Some("all") => {}
                _ => out.push(format!(
                    "意图 {}：lock 的 value 只能是 place / shape / all",
                    it.id
                )),
            },
            _ => {}
        }
    }
    out
}

/// 路径模式：按 `.` 分段，`*` 一段、`**` 任意多段；无通配时就是相等
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Intent {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn roundtrip_keeps_unknown_fields() {
        let it = parse(
            r#"{"id":3,"author":"ai","status":{"state":"accepted","note":"x"},"selection":{"rule":{"name":"^A"}},"text":"t","formal":{"decisions":[{"op":"role","value":"decor"}]},"custom":1}"#,
        );
        assert!(it.is_active() && it.is_rule());
        assert_eq!(it.priority(), (0, 3));
        let s = serde_json::to_string(&it).unwrap();
        assert!(s.contains("\"custom\":1") && s.contains("\"note\":\"x\""));
        assert_eq!(parse(&s), it);
        assert!(validate(&it).is_empty());
    }

    #[test]
    fn vocabulary() {
        let bad = parse(
            r#"{"id":1,"selection":{"paths":["a"]},"formal":{"decisions":[{"op":"fly"},{"op":"role","value":"wall"},{"op":"hollow","floors":2},{"op":"lock","value":"x"}]}}"#,
        );
        let errs = validate(&bad);
        assert_eq!(errs.len(), 4, "{:?}", errs);
        let ok = parse(
            r#"{"id":2,"selection":{"sources":["a","b"]},"formal":{"decisions":[{"op":"merge","name":"hangar","shape":"box"},{"op":"type","name":"house"}]}}"#,
        );
        assert!(validate(&ok).is_empty());
        let missing = parse(r#"{"id":4,"selection":{"paths":["x"]},"formal":{"decisions":[{"op":"drop"}]}}"#);
        assert_eq!(validate(&missing).len(), 1);
    }

    #[test]
    fn patterns() {
        assert!(path_matches("dorm_02.*", "dorm_02.wall"));
        assert!(!path_matches("dorm_02.*", "dorm_02.a.b"));
        assert!(path_matches("dorm_02.**", "dorm_02.a.b"));
        assert!(path_matches("x", "x") && !path_matches("x", "y"));
    }
}
