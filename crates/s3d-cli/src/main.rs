//! `s3d` 命令行（M1）：compile 输出 JSON；verify 对参考输出比 world / box。

use s3d_lang::compile::{compile, CompileOptions};
use std::path::Path;

fn abs_path(p: &str) -> String {
    let path = Path::new(p);
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    };
    abs.to_string_lossy().replace('\\', "/")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let read = |p: &str| std::fs::read_to_string(p).ok();
    match args.first().map(String::as_str) {
        Some("compile") if args.len() >= 2 => {
            let file = &args[1];
            let text = std::fs::read_to_string(file).unwrap_or_else(|e| {
                eprintln!("读不到 {}：{}", file, e);
                std::process::exit(2)
            });
            let display = Path::new(file)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file.clone());
            let out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let j = s3d_lang::output::to_json(&out);
            println!("{}", serde_json::to_string_pretty(&j).unwrap());
        }
        Some("verify") if args.len() >= 3 => {
            let file = &args[1];
            let reference = &args[2];
            let text = std::fs::read_to_string(file).expect("读不到源文件");
            let display = Path::new(file)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file.clone());
            let out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let mine = s3d_lang::output::to_json(&out);
            let reft: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(reference).expect("读不到参考")).unwrap();
            let code = verify(&mine, &reft);
            std::process::exit(code);
        }
        _ => {
            eprintln!("用法：s3d compile <文件.s3d>\n      s3d verify <文件.s3d> <参考.compile.json>");
            std::process::exit(2);
        }
    }
}

/// 对参考输出比 nodes[] 的 world.R / world.t / box（容差 1e-6）；打印差异
fn verify(mine: &serde_json::Value, reft: &serde_json::Value) -> i32 {
    use std::collections::BTreeMap;
    let tol = 1e-6;
    let mut ref_nodes: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    for n in reft["nodes"].as_array().unwrap() {
        ref_nodes.insert(n["path"].as_str().unwrap().to_string(), n);
    }
    let mut ok = 0;
    let mut bad = 0;
    let mut missing = 0;
    let mut seen = std::collections::BTreeSet::new();
    let flat = |v: &serde_json::Value| -> Vec<f64> {
        let mut out = Vec::new();
        fn walk(v: &serde_json::Value, out: &mut Vec<f64>) {
            match v {
                serde_json::Value::Number(n) => out.push(n.as_f64().unwrap()),
                serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
                serde_json::Value::Object(o) => o.values().for_each(|x| walk(x, out)),
                _ => {}
            }
        }
        walk(v, &mut out);
        out
    };
    for n in mine["nodes"].as_array().unwrap() {
        let path = n["path"].as_str().unwrap();
        seen.insert(path.to_string());
        match ref_nodes.get(path) {
            None => {
                missing += 1;
                println!("  EXTRA {}", path);
            }
            Some(r) => {
                let mut diffs = Vec::new();
                for key in ["world", "box"] {
                    let a = flat(&n[key]);
                    let b = flat(&r[key]);
                    if a.len() != b.len() {
                        diffs.push(format!("{} 形状不同", key));
                        continue;
                    }
                    let d = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
                    if d > tol {
                        diffs.push(format!("{} Δ={:.3e}", key, d));
                    }
                }
                if n["kind"] != r["kind"] && !(r["kind"].as_str().unwrap_or("").contains('→')) {
                    diffs.push(format!("kind {} vs {}", n["kind"], r["kind"]));
                }
                if diffs.is_empty() {
                    ok += 1;
                } else {
                    bad += 1;
                    println!("  DIFF {}: {}", path, diffs.join("; "));
                }
            }
        }
    }
    let mut ref_missing = 0;
    for p in ref_nodes.keys() {
        if !seen.contains(p) {
            ref_missing += 1;
            if ref_missing <= 20 {
                println!("  MISSING {}", p);
            }
        }
    }
    println!(
        "nodes: ok {} bad {} extra {} missing {}; errors mine {} ref {}",
        ok,
        bad,
        missing,
        ref_missing,
        mine["errors"].as_array().map(|a| a.len()).unwrap_or(0),
        reft["errors"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    for e in mine["errors"].as_array().unwrap().iter().take(15) {
        println!("  ERR {}:{} {} {}", e["file"], e["line"], e["path"], e["message"]);
    }
    if bad == 0 && ref_missing == 0 && missing == 0 {
        0
    } else {
        1
    }
}
