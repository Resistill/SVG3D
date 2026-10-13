//! `s3d` 命令行（M1）：compile 输出 JSON；verify 对参考输出对拍（9.4）。

mod sha256;
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

/// 把一个文件与它 import 链上的文件打成 wasm 入口的 JSON 包（按文件名索引）
fn make_bundle(file: &str) -> String {
    use serde_json::{json, Map, Value as J};
    let mut files = Map::new();
    fn visit(p: &Path, files: &mut Map<String, J>) {
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if files.contains_key(&name) {
            return;
        }
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(_) => return,
        };
        files.insert(name, J::String(text.clone()));
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("import") {
                let rest = rest.trim_start();
                if let Some(q) = rest.strip_prefix('"') {
                    if let Some(end) = q.find('"') {
                        let rel = &q[..end];
                        let dir = p.parent().unwrap_or(Path::new("."));
                        visit(&dir.join(rel), files);
                    }
                }
            }
        }
    }
    visit(Path::new(file), &mut files);
    let main = Path::new(file)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    json!({ "main": main, "files": J::Object(files) }).to_string()
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
        Some("bench-nearest") if args.len() >= 2 => {
            // 最近点微基准（PLAN 第 8 节）：s3d bench-nearest <glb> [查询数=2620000] [线程数=8]
            let bytes = std::fs::read(&args[1]).expect("读不到 GLB");
            let n: usize = args.get(2).and_then(|x| x.parse().ok()).unwrap_or(2_620_000);
            let threads: usize = args.get(3).and_then(|x| x.parse().ok()).unwrap_or(8);
            let t0 = std::time::Instant::now();
            let mesh = s3d_geom::mesh::load_glb(&bytes).expect("GLB 解析失败");
            let (lo, hi) = mesh.bbox();
            let near = s3d_geom::mesh::Nearest::new(&mesh).expect("建 BVH 失败");
            println!(
                "mesh: {} 顶点 {} 三角形；读取 + 建 BVH {:.3} s",
                mesh.verts.len(),
                mesh.tris.len(),
                t0.elapsed().as_secs_f64()
            );
            // 确定性的查询点：LCG，落在包围盒外扩 10% 的范围里
            let ext = (hi - lo) * 0.1;
            let (a, b) = (lo - ext, hi + ext);
            let pts: Vec<s3d_geom::Vec3> = {
                let mut st: u64 = 0x9E3779B97F4A7C15;
                let mut next = || {
                    st = st
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    ((st >> 11) as f64) / ((1u64 << 53) as f64)
                };
                (0..n)
                    .map(|_| {
                        let (u, v, w) = (next(), next(), next());
                        s3d_geom::Vec3::new(
                            a.x + (b.x - a.x) * u,
                            a.y + (b.y - a.y) * v,
                            a.z + (b.z - a.z) * w,
                        )
                    })
                    .collect()
            };
            let t1 = std::time::Instant::now();
            let mut sum1 = 0.0f64;
            for p in &pts {
                sum1 += near.query(*p).1;
            }
            let single = t1.elapsed().as_secs_f64();
            // 多线程：静态分块，按块号顺序归约（原则 18）
            let t2 = std::time::Instant::now();
            let chunk = n.div_ceil(threads.max(1));
            let partial: Vec<f64> = std::thread::scope(|sc| {
                let hs: Vec<_> = pts
                    .chunks(chunk.max(1))
                    .map(|c| {
                        let near = &near;
                        sc.spawn(move || {
                            let mut s = 0.0f64;
                            for p in c {
                                s += near.query(*p).1;
                            }
                            s
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            let multi = t2.elapsed().as_secs_f64();
            let sum_m: f64 = partial.iter().fold(0.0, |acc, x| acc + x);
            println!(
                "{} 次查询：单线程 {:.3} s（{:.2} µs/次）；{} 线程 {:.3} s（{:.2} µs/次）",
                n,
                single,
                single * 1e6 / n as f64,
                threads,
                multi,
                multi * 1e6 / n as f64
            );
            println!("校验和：单线程 {:.9e}；多线程按块序归约 {:.9e}", sum1, sum_m);
        }
        Some("hash") if args.len() >= 2 => {
            // 三处哈希对拍的 native 一侧：与 wasm 走同一个 compile_bundle，打印 `sha256  文件名`
            for file in &args[1..] {
                let bundle = make_bundle(file);
                let out = s3d_wasm::compile_bundle(&bundle);
                let name = Path::new(file)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or(file.clone());
                println!("{}  {}", sha256::sha256_hex(out.as_bytes()), name);
            }
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
