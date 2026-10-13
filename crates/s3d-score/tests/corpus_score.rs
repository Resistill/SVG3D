//! 自评分数对原型的回归（`corpus/reference/exports/scores.json`，原型 commit 439abb8 的 `tools/score.mjs`）。
//! 用原型的分段（`TriParams::PROTOTYPE`，源 GLB 是原型按 three.js 默认分段导出的）：
//!   L_stmt / L_mesh / S_mesh 逐位相同；R_src 在 5% + 10 内（随机采样落在不同的三角形上；spaceport 原文的
//!   hangar.body 布尔体在原型的 three-bvh-csg 里多出一块共面的"门槛"面片，本版的 BSP 结果没有它，差 6.5）、
//!   R_recon 在 10% 内；
//!   R_vol 在 40% 内（原型 `insideMesh` 对 Float32 的 Lathe 网格漏判约一半内部点，本版按绕数交叉验证过）。
//! 另验：带缓存连续评分（增量）与全量逐位相同。

use s3d_geom::mesh::{load_parts, Part, Split};
use s3d_geom::tri::TriParams;
use s3d_lang::compile::{compile, CompileOptions};
use s3d_score::score::{score, Consts, Input, ScoreCache};
use serde_json::Value as J;

fn root() -> String {
    format!("{}/../../corpus/", env!("CARGO_MANIFEST_DIR"))
}

fn run(file: &str, parts: &[Part], cache: Option<&mut ScoreCache>) -> J {
    let text = std::fs::read_to_string(file).unwrap();
    let read = |p: &str| std::fs::read_to_string(p).ok();
    let display = file.rsplit('/').next().unwrap().to_string();
    let mut c = compile(&text, file, &display, &read, &CompileOptions::default());
    let items = s3d_lang::items::collect_item_trees(&mut c);
    let nodes = s3d_lang::items::node_infos(&c);
    let files = s3d_lang::items::source_files(&c);
    let edit = s3d_score::edit::edit_report(&c, &nodes);
    let consts = Consts {
        tri: TriParams::PROTOTYPE,
        ..Consts::default()
    };
    let input = Input {
        items: &items,
        nodes: &nodes,
        files: &files,
        parts,
        consts,
        edit: Some(&edit),
    };
    score(&input, cache)
}

fn parts_of(name: &str) -> Vec<Part> {
    let bytes = std::fs::read(format!("{}exports/{}.glb", root(), name)).unwrap();
    load_parts(&bytes, Split::Auto).unwrap()
}

fn reference() -> J {
    serde_json::from_str(
        &std::fs::read_to_string(format!("{}reference/exports/scores.json", root())).unwrap(),
    )
    .unwrap()
}

fn check(name: &str, variant: &str, got: &J, want: &J) {
    let g = |k: &str| got[k].as_f64().unwrap();
    let w = |k: &str| want[k].as_f64().unwrap();
    let exact = |k: &str| {
        assert!(
            (g(k) - w(k)).abs() < 0.011,
            "{name} {variant} {k}: {} vs {}",
            g(k),
            w(k)
        )
    };
    let within = |k: &str, rel: f64, abs: f64| {
        let (a, b) = (g(k), w(k));
        assert!(
            (a - b).abs() <= abs + rel * b.abs(),
            "{name} {variant} {k}: {} vs {}",
            a,
            b
        );
    };
    exact("L_stmt");
    exact("L_mesh");
    exact("S_mesh");
    within("R_src", 0.05, 10.0);
    within("R_recon", 0.10, 1.0);
    within("R_vol", 0.40, 1.0);
}

#[test]
fn prototype_scores_basics_semantics() {
    let r = reference();
    for name in ["basics.L0", "semantics.L0"] {
        let parts = parts_of(name);
        let orig = name.split('.').next().unwrap();
        for (variant, file) in [
            ("recon", format!("{}exports/{}.s3d", root(), name)),
            ("abs", format!("{}exports/{}.abs.s3d", root(), name)),
            ("orig", format!("{}examples/{}.s3d", root(), orig)),
        ] {
            let got = run(&file, &parts, None);
            check(name, variant, &got, &r["files"][name][variant]);
        }
    }
}

#[test]
fn prototype_scores_spaceport_and_incremental() {
    let r = reference();
    let name = "spaceport.L0";
    let parts = parts_of(name);
    let files = [
        ("recon", format!("{}exports/{}.s3d", root(), name)),
        ("abs", format!("{}exports/{}.abs.s3d", root(), name)),
        ("orig", format!("{}examples/spaceport.s3d", root())),
    ];
    let mut fresh = Vec::new();
    for (variant, file) in &files {
        let got = run(file, &parts, None);
        check(name, variant, &got, &r["files"][name][variant]);
        fresh.push(got);
    }
    // 增量：同一个缓存连续评三个版本再回到第一个，与全量逐位相同
    let mut cache = ScoreCache::default();
    let mut seq = Vec::new();
    for (_, file) in &files {
        seq.push(run(file, &parts, Some(&mut cache)));
    }
    let again = run(&files[0].1, &parts, Some(&mut cache));
    for (i, s) in seq.iter().enumerate() {
        assert_eq!(s["S"], fresh[i]["S"], "incremental {}", files[i].0);
        assert_eq!(s["R_src"], fresh[i]["R_src"]);
        assert_eq!(s["objects"], fresh[i]["objects"]);
    }
    assert_eq!(again["S"], fresh[0]["S"]);
    assert_eq!(again["sources"], fresh[0]["sources"]);
    // E_edit 报告：原文里坐标依赖的物体少于逆向的全坐标版
    let dep = |j: &J| j["edit"]["dependent"].as_u64().unwrap();
    assert!(
        dep(&fresh[2]) < dep(&fresh[1]),
        "orig {} vs abs {}",
        dep(&fresh[2]),
        dep(&fresh[1])
    );
}
