//! `s3d` 命令行（M1）：compile 输出 JSON；verify 对参考输出对拍（9.4）。

mod verify;

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
            let mut out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let j = s3d_lang::output::to_json(&mut out);
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
            let mut out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let mine = s3d_lang::output::to_json(&mut out);
            let reft: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(reference).expect("读不到参考")).unwrap();
            let rep = verify::verify(&mine, &reft);
            for l in &rep.lines {
                println!("{}", l);
            }
            println!("failures: {}", rep.failures);
            std::process::exit(if rep.failures == 0 { 0 } else { 1 });
        }
        Some("eval") if args.len() >= 3 => {
            // 真值评测：s3d eval <原始.s3d> <逆向.s3d> [--lib <semantics.s3d>]
            let orig = &args[1];
            let recon = &args[2];
            let lib_path = args
                .iter()
                .position(|a| a == "--lib")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| {
                    Path::new(orig)
                        .parent()
                        .unwrap_or(Path::new("."))
                        .join("lib/semantics.s3d")
                        .to_string_lossy()
                        .to_string()
                });
            let lib = std::fs::read_to_string(&lib_path).unwrap_or_default();
            let ot = std::fs::read_to_string(orig).expect("读不到原始文件");
            let rt = std::fs::read_to_string(recon).expect("读不到逆向文件");
            let name = |p: &str| {
                Path::new(p)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or(p.to_string())
            };
            let r = s3d_score::eval::evaluate(
                s3d_score::eval::Side {
                    text: &ot,
                    identity: &abs_path(orig),
                    display: &name(orig),
                },
                s3d_score::eval::Side {
                    text: &rt,
                    identity: &abs_path(recon),
                    display: &name(recon),
                },
                &lib,
                &read,
            );
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
        }
        Some("parts") if args.len() >= 3 => {
            let file = &args[1];
            let text = std::fs::read_to_string(file).expect("读不到源文件");
            let display = Path::new(file)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file.clone());
            let mut out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let parts = out.compiler.debug_parts(&args[2]);
            println!(
                "{}",
                serde_json::to_string(
                    &parts
                        .iter()
                        .map(|p| p.iter().map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>())
                        .collect::<Vec<_>>()
                )
                .unwrap()
            );
        }
        _ => {
            eprintln!("用法：s3d compile <文件.s3d>\n      s3d verify <文件.s3d> <参考.compile.json>");
            std::process::exit(2);
        }
    }
}
