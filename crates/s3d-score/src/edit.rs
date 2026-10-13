//! E_edit（docs/PLAN.md 第 6 节）：坐标依赖的代价。一个物体算坐标依赖，当且仅当它的定位没有参照物
//! （`at=` 坐标，或没有定位属性），或参照物与它之间没有接触类事实（on / against / inside / touching /
//! intersects，按 6.7.1 的关系索引、含祖先提升）。每个坐标依赖的物体付一个固定项（量级相当于一个名字），
//! 权重可配置；M2 只报告它（决定 7 的权重在 M3 末定）。

use s3d_lang::compile::Compiled;
use s3d_lang::items::NodeInfo;
use s3d_lang::scene::Role;

/// 报告：坐标依赖的物体路径（按节点顺序）与被检查的物体数
#[derive(Clone, Debug, Default)]
pub struct EditReport {
    pub dependent: Vec<String>,
    pub checked: usize,
    /// 事实没有推导（facts=false 或被跳过）时为 true：接触判断不可用，只按"有没有参照物"判
    pub facts_missing: bool,
}

/// 以语句为单位：有几何条目、不是影子 / 副本 / 阵列实例的节点各算一个物体
pub fn edit_report(c: &Compiled, nodes: &[NodeInfo]) -> EditReport {
    let comp = &c.compiler;
    let facts_ok = comp.facts_done && !comp.facts_skipped && comp.rel_index.len() > 5;
    let mut out = EditReport {
        facts_missing: !facts_ok,
        ..Default::default()
    };
    for n in nodes {
        if !n.has_item || n.ghost || n.kind == "instance" {
            continue;
        }
        if !matches!(comp.scene.nodes[n.id].role, Role::Normal) {
            continue;
        }
        out.checked += 1;
        let dep = match n.placement_ref {
            None => true,
            Some(r) => {
                if facts_ok {
                    let contact = &comp.rel_index[5];
                    // 自己或祖先与参照物（或其祖先）之间有接触事实
                    let mine = comp.scene.chain(n.id);
                    let theirs = comp.scene.chain(r);
                    !mine.iter().any(|&x| {
                        contact
                            .get(x)
                            .map(|l| l.iter().any(|y| theirs.contains(y)))
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            }
        };
        if dep {
            out.dependent.push(n.path.clone());
        }
    }
    out
}
