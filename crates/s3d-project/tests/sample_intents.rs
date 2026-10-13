//! 规则样本意图（corpus/intents/roles.spacecenter.json）的执行：
//!   spaceport.L0 的 111 个部件：ground / road → terrain，4 盏 lamp → decor，其余没有意图命中（默认 structure）；
//!   SpaceCenter（原型 work-snapshot 的 state/source-roles.json，32 157 个部件名；clone 不在时跳过）：
//!   与原型的角色一致 ≥ 74%，且规则命中的部件全部与原型一致——不一致只来自样本自述的近似
//!   （规则只认"角色词在名字末尾"，原型取"最后一个有归属的词"并有几何回退）。

use s3d_geom::mesh::{load_parts, Material, Part, Split};
use s3d_project::store::{load_file, problems};
use s3d_reverse::parts::{apply_source_decisions, source_roles};
use std::collections::BTreeMap;
use std::path::Path;

fn root() -> String {
    format!("{}/../../", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn spaceport_roles() {
    let intents = load_file(Path::new(&format!(
        "{}corpus/intents/roles.spacecenter.json",
        root()
    )))
    .unwrap();
    assert_eq!(intents.len(), 8);
    assert!(problems(&intents).is_empty());
    let bytes = std::fs::read(format!("{}corpus/exports/spaceport.L0.glb", root())).unwrap();
    let parts = load_parts(&bytes, Split::Auto).unwrap();
    let (out, report) = apply_source_decisions(parts, &intents);
    assert_eq!(out.len(), 111);
    assert!(report.unsatisfied.is_empty() && report.dangling.is_empty() && report.conflicts.is_empty());
    let roles = source_roles(&out);
    let special: BTreeMap<&str, &str> = roles
        .iter()
        .filter(|(_, r)| r.as_str() != "structure")
        .map(|(k, r)| (k.as_str(), r.as_str()))
        .collect();
    assert_eq!(
        special,
        BTreeMap::from([
            ("ground", "terrain"),
            ("road", "terrain"),
            ("lights.0.lamp", "decor"),
            ("lights.1.lamp", "decor"),
            ("lights.2.lamp", "decor"),
            ("lights.3.lamp", "decor"),
        ])
    );
    // 意图归属：命中的部件记下意图 id
    let lamp = out
        .iter()
        .find(|s| s.part.name.as_deref() == Some("lights.0.lamp"))
        .unwrap();
    assert_eq!(lamp.intents, vec![2]);
}

#[test]
fn spacecenter_roles_against_prototype() {
    let file = std::env::var("S3D_PROTOTYPE")
        .unwrap_or_else(|_| "/home/user/resistill/svg3d-prototype".to_string())
        + "/work-snapshot/sc/full/state/source-roles.json";
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("跳过：没有 {}", file);
            return;
        }
    };
    let theirs: BTreeMap<String, String> = serde_json::from_str(&text).unwrap();
    let intents = load_file(Path::new(&format!(
        "{}corpus/intents/roles.spacecenter.json",
        root()
    )))
    .unwrap();
    let parts: Vec<Part> = theirs
        .keys()
        .map(|n| Part {
            name: Some(n.clone()),
            uid: Some(n.clone()),
            from: None,
            tris: Vec::new(),
            material: Material {
                name: None,
                color: "#c9c9c9".into(),
                metal: None,
                rough: None,
            },
            mesh: None,
            matrix: None,
            role: None,
            meta: BTreeMap::new(),
        })
        .collect();
    let (out, _) = apply_source_decisions(parts, &intents);
    let mut same = 0usize;
    let mut fired_mismatch = Vec::new();
    for sp in &out {
        let id = sp.part.uid.as_ref().unwrap();
        let ours = sp.part.role.clone().unwrap_or_else(|| "structure".to_string());
        let want = &theirs[id];
        if &ours == want {
            same += 1;
        } else if sp.part.role.is_some() {
            fired_mismatch.push((id.clone(), ours, want.clone()));
        }
    }
    let ratio = same as f64 / out.len() as f64;
    assert!(ratio >= 0.74, "一致率 {:.4}", ratio);
    assert!(
        fired_mismatch.is_empty(),
        "规则命中但与原型不同：{:?}",
        &fired_mismatch[..fired_mismatch.len().min(5)]
    );
}
