//! 重绑（docs/PLAN.md 4.3）：源模型改版、重跑逆向、名字变化之后把意图的锚找回来。
//! 规则：路径命中后必须用指纹或来源交叉验证（同种类且 IoU 过阈值，或 `~src` 一致）才算 bound；
//! 只有路径命中、指纹不符的记为 suspicious；路径没命中时按指纹（同种类、IoU 最高且过阈值）重绑并改写路径；
//! 还找不到的列为 dangling。来源锚按源部件 id 是否存在判；区域锚总是成立（数区域里的物体）。
//! 稳定命名与指纹锚共用 `best_by_fingerprint`。

use s3d_lang::compile::Compiled;
use s3d_lang::scene::ROOT;
use s3d_reverse::intent::{path_matches, Fingerprint, Intent};
use serde_json::json;
use std::collections::BTreeMap;

/// 场景索引：重绑用到的每个节点的信息
#[derive(Clone, Debug)]
pub struct SceneNode {
    pub path: String,
    pub kind: String,
    /// 世界紧包围盒 [x0,y0,z0,x1,y1,z1]
    pub tight: Option<[f64; 6]>,
    pub src: Option<String>,
}

pub struct SceneIndex {
    pub nodes: Vec<SceneNode>,
    by_path: BTreeMap<String, usize>,
}

impl SceneIndex {
    pub fn from_compiled(c: &Compiled) -> SceneIndex {
        let comp = &c.compiler;
        let mut nodes = Vec::new();
        let mut stack: Vec<usize> = comp.scene.nodes[ROOT].children.iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            let node = &comp.scene.nodes[n];
            if node.dropped {
                continue;
            }
            let tight = node
                .tight
                .map(|b| [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z]);
            let src = match node.meta.get("src") {
                Some(s3d_lang::value::Value::Str(s)) => Some(s.clone()),
                _ => None,
            };
            nodes.push(SceneNode {
                path: comp.scene.path(n),
                kind: node.kind.clone(),
                tight,
                src,
            });
            for ch in node.children.iter().rev() {
                stack.push(*ch);
            }
        }
        let by_path = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.path.clone(), i))
            .collect();
        SceneIndex { nodes, by_path }
    }

    pub fn get(&self, path: &str) -> Option<&SceneNode> {
        self.by_path.get(path).map(|&i| &self.nodes[i])
    }

    /// 一个节点的指纹（有紧盒的节点才有）
    pub fn fingerprint(&self, path: &str) -> Option<Fingerprint> {
        let n = self.get(path)?;
        let b = n.tight?;
        Some(Fingerprint {
            kind: n.kind.clone(),
            size: [b[3] - b[0], b[4] - b[1], b[5] - b[2]],
            center: [(b[0] + b[3]) / 2.0, (b[1] + b[4]) / 2.0, (b[2] + b[5]) / 2.0],
        })
    }
}

fn fp_box(f: &Fingerprint) -> [f64; 6] {
    [
        f.center[0] - f.size[0] / 2.0,
        f.center[1] - f.size[1] / 2.0,
        f.center[2] - f.size[2] / 2.0,
        f.center[0] + f.size[0] / 2.0,
        f.center[1] + f.size[1] / 2.0,
        f.center[2] + f.size[2] / 2.0,
    ]
}

pub fn iou(a: &[f64; 6], b: &[f64; 6]) -> f64 {
    let mut inter = 1.0;
    for k in 0..3 {
        inter *= (a[k + 3].min(b[k + 3]) - a[k].max(b[k])).max(0.0);
    }
    let vol = |x: &[f64; 6]| (0..3).fold(1.0, |s, k| s * (x[k + 3] - x[k]).max(1e-9));
    inter / (vol(a) + vol(b) - inter)
}

/// 指纹对一个节点：同种类且 IoU ≥ 阈值
pub fn fingerprint_matches(f: &Fingerprint, n: &SceneNode, threshold: f64) -> bool {
    n.kind == f.kind && n.tight.map(|t| iou(&t, &fp_box(f)) >= threshold).unwrap_or(false)
}

/// 全场里与指纹最像的节点（同种类、IoU 最高且过阈值）；并列取路径靠前的
pub fn best_by_fingerprint<'a>(
    f: &Fingerprint,
    scene: &'a SceneIndex,
    threshold: f64,
) -> Option<&'a SceneNode> {
    let fb = fp_box(f);
    let mut best: Option<(f64, &SceneNode)> = None;
    for n in &scene.nodes {
        if n.kind != f.kind {
            continue;
        }
        if let Some(t) = n.tight {
            let s = iou(&t, &fb);
            if s >= threshold && best.map(|b| s > b.0).unwrap_or(true) {
                best = Some((s, n));
            }
        }
    }
    best.map(|b| b.1)
}

/// 重绑的结果（写回 status 之外的摘要）
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub bound_paths: usize,
    pub bound_sources: usize,
    pub bound_fingerprints: usize,
    pub suspicious: Vec<String>,
    pub dangling: Vec<String>,
    /// (旧路径, 新路径)
    pub rebound: Vec<(String, String)>,
    pub region_count: usize,
}

/// 对一条意图重绑；`sources` 是现在的源部件 id 集合（没有源模型时给空切片，来源锚全部按 dangling）；
/// `when` 写进 history。结果写回 `status.bound` / `status.dangling`，路径改写进 `selection.paths`
pub fn rebind(
    it: &mut Intent,
    scene: &SceneIndex,
    sources: &[String],
    threshold: f64,
    when: &str,
) -> Outcome {
    let mut out = Outcome::default();
    let fps = it.selection.fingerprints.clone();
    let srcs = it.selection.sources.clone();
    let mut new_paths = Vec::new();
    for pat in &it.selection.paths {
        let hits: Vec<&SceneNode> = if pat.contains('*') {
            scene
                .nodes
                .iter()
                .filter(|n| path_matches(pat, &n.path))
                .collect()
        } else {
            scene.get(pat).into_iter().collect()
        };
        if hits.is_empty() {
            // 按指纹重绑：取这条路径对应的指纹（创建时按顺序记）；通配模式没有指纹
            let idx = it
                .selection
                .paths
                .iter()
                .position(|p| p == pat)
                .unwrap_or(usize::MAX);
            if let Some(f) = fps.get(idx) {
                if let Some(n) = best_by_fingerprint(f, scene, threshold) {
                    out.rebound.push((pat.clone(), n.path.clone()));
                    out.bound_paths += 1;
                    new_paths.push(n.path.clone());
                    continue;
                }
            }
            out.dangling.push(pat.clone());
            new_paths.push(pat.clone());
            continue;
        }
        // 交叉验证：指纹或来源
        let verified = |n: &SceneNode| -> Option<bool> {
            if fps.is_empty() && srcs.is_empty() {
                return None;
            }
            let by_fp = fps.iter().any(|f| fingerprint_matches(f, n, threshold));
            let by_src = n.src.as_ref().map(|s| srcs.contains(s)).unwrap_or(false);
            Some(by_fp || by_src)
        };
        let mut any_bound = false;
        let mut any_suspicious = false;
        for n in &hits {
            match verified(n) {
                None | Some(true) => any_bound = true,
                Some(false) => any_suspicious = true,
            }
        }
        if any_bound {
            out.bound_paths += 1;
        } else if any_suspicious {
            out.suspicious.push(pat.clone());
        }
        new_paths.push(pat.clone());
    }
    it.selection.paths = new_paths;
    for s in &srcs {
        if sources.contains(s) {
            out.bound_sources += 1;
        } else {
            out.dangling.push(format!("src:{}", s));
        }
    }
    for f in &fps {
        if scene.nodes.iter().any(|n| fingerprint_matches(f, n, threshold)) {
            out.bound_fingerprints += 1;
        }
    }
    if let Some(r) = &it.selection.region {
        out.region_count = scene
            .nodes
            .iter()
            .filter(|n| {
                n.tight
                    .map(|t| (0..3).all(|k| t[k] >= r.box_[k] && t[k + 3] <= r.box_[k + 3]))
                    .unwrap_or(false)
            })
            .count();
    }
    it.status.bound = Some(s3d_reverse::intent::Bound {
        paths: out.bound_paths,
        sources: out.bound_sources,
        fingerprints: out.bound_fingerprints,
        suspicious: out.suspicious.len(),
    });
    it.status.dangling = out.dangling.clone();
    if !out.rebound.is_empty() || !out.dangling.is_empty() {
        it.history.push(json!({
            "when": when,
            "event": if out.dangling.is_empty() { "rebound" } else { "unsatisfied" },
            "rebound": out.rebound.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>(),
            "dangling": out.dangling,
        }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use s3d_lang::compile::{compile, CompileOptions};

    fn scene_of(text: &str) -> SceneIndex {
        let read = |_: &str| None;
        let c = compile(text, "/x/t.s3d", "t.s3d", &read, &CompileOptions::default());
        SceneIndex::from_compiled(&c)
    }

    #[test]
    fn path_fingerprint_source_region() {
        let s1 = scene_of("box ground size=20,0.2,20\nbox hut size=2,2,2 on=ground ~src=\"Hut_01\"\nbox shed size=4,3,4 on=ground[0.9,1,0.9]\n");
        let mut it: Intent = serde_json::from_str(r#"{"id":1,"status":{"state":"accepted"},"selection":{"paths":["hut","shed"],"sources":["Hut_01"],"region":{"box":[-11,-1,-11,11,5,11]}},"formal":{"decisions":[{"op":"hollow"}]}}"#).unwrap();
        it.selection.fingerprints = vec![s1.fingerprint("hut").unwrap(), s1.fingerprint("shed").unwrap()];
        let o = rebind(&mut it, &s1, &["Hut_01".to_string()], 0.5, "t0");
        assert_eq!(o.bound_paths, 2);
        assert_eq!(o.bound_sources, 1);
        assert_eq!(o.bound_fingerprints, 2);
        assert_eq!(o.region_count, 3);
        assert!(o.dangling.is_empty() && it.history.is_empty());
        // 改名 + 挪动：hut 改叫 cabin（同位置），shed 原名但挪远了（可疑），源部件没了
        let s2 = scene_of("box ground size=20,0.2,20\nbox cabin size=2,2,2 on=ground ~src=\"Hut_02\"\nbox shed size=4,3,4 on=ground[0.1,1,0.1]\n");
        let o = rebind(&mut it, &s2, &[], 0.5, "t1");
        assert_eq!(o.rebound, vec![("hut".to_string(), "cabin".to_string())]);
        assert_eq!(it.selection.paths, vec!["cabin", "shed"]);
        assert_eq!(o.suspicious, vec!["shed".to_string()]);
        assert_eq!(o.dangling, vec!["src:Hut_01".to_string()]);
        assert_eq!(it.status.bound.as_ref().unwrap().suspicious, 1);
        assert_eq!(it.history.len(), 1);
    }
}
