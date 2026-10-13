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
        Some("bench-bvh") if args.len() >= 2 => {
            // 自写 BVH（s3d_geom::bvh）的最近点微基准：s3d bench-bvh <glb> [查询数=2620000]
            let bytes = std::fs::read(&args[1]).expect("读不到 GLB");
            let parts =
                s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::None).expect("GLB 读取失败");
            let tris: Vec<s3d_geom::tri::Tri> = parts.iter().flat_map(|p| p.tris.iter().copied()).collect();
            let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2_620_000);
            let t0 = std::time::Instant::now();
            let bvh = s3d_geom::bvh::Bvh::new(&tris);
            let built = t0.elapsed();
            let (_, _, b) = s3d_geom::tri::mesh_stats(&tris);
            let ext = b.max - b.min;
            let mut seed: u64 = 0x9e3779b97f4a7c15;
            let mut next = || {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((seed >> 11) as f64) / ((1u64 << 53) as f64)
            };
            let pts: Vec<s3d_geom::Vec3> = (0..n)
                .map(|_| {
                    b.min - ext * 0.1
                        + s3d_geom::Vec3::new(
                            next() * ext.x * 1.2,
                            next() * ext.y * 1.2,
                            next() * ext.z * 1.2,
                        )
                })
                .collect();
            let t1 = std::time::Instant::now();
            let mut sum = 0.0;
            for p in &pts {
                sum += bvh.closest(*p, f64::INFINITY).map(|x| x.1).unwrap_or(0.0);
            }
            let dt = t1.elapsed();
            println!(
                "{} 个三角形，建树 {:.3} s；{} 次查询单线程 {:.3} s（{:.2} µs/次）校验和 {:.6}",
                tris.len(),
                built.as_secs_f64(),
                n,
                dt.as_secs_f64(),
                dt.as_secs_f64() * 1e6 / n as f64,
                sum
            );
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
        Some("score-hash") if args.len() >= 2 => {
            // 评分的三处哈希对拍（native 一侧）：源模型取同目录的 <名>.glb（<名>.abs.s3d / <名>.rev.s3d 也用 <名>.glb）
            for file in &args[1..] {
                let bundle = make_bundle(file);
                let p = Path::new(file);
                let stem = p
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let base = stem.trim_end_matches(".s3d");
                let base = base.trim_end_matches(".abs").trim_end_matches(".rev");
                let glb_path = p.with_file_name(format!("{}.glb", base));
                let glb = match std::fs::read(&glb_path) {
                    Ok(g) => g,
                    Err(_) => {
                        eprintln!("跳过 {}：没有 {}", stem, glb_path.display());
                        continue;
                    }
                };
                let out = s3d_wasm::score_bundle(&bundle, &glb);
                println!("{}  {}", sha256::sha256_hex(out.as_bytes()), stem);
            }
        }
        Some("intents") if args.len() >= 2 => {
            // 读取部件阶段执行源侧决定：s3d intents <意图文件或工程目录> (--glb <源模型.glb> | --names <id→角色的 JSON，只取键>)
            //   [--roles-out <文件>]；打印报告，--roles-out 写出 源部件 id → 角色（原型 state/source-roles.json 的形状）
            let opt = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
            let src = Path::new(&args[1]);
            let intents = if src.is_dir() {
                s3d_project::store::load_all(src)
            } else {
                s3d_project::store::load_file(src)
            }
            .unwrap_or_else(|e| panic!("{}", e));
            let problems = s3d_project::store::problems(&intents);
            let parts: Vec<s3d_geom::mesh::Part> = if let Some(g) = opt("--glb") {
                let bytes = std::fs::read(&g).expect("读不到 GLB");
                s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::Auto).expect("GLB 读取失败")
            } else if let Some(n) = opt("--names") {
                let v: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&n).expect("读不到名字表"))
                        .expect("名字表不是 JSON");
                let names: Vec<String> = match &v {
                    serde_json::Value::Object(o) => o.keys().cloned().collect(),
                    serde_json::Value::Array(a) => {
                        a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()
                    }
                    _ => Vec::new(),
                };
                names
                    .into_iter()
                    .map(|n| s3d_geom::mesh::Part {
                        name: Some(n.clone()),
                        uid: Some(n),
                        from: None,
                        tris: Vec::new(),
                        material: s3d_geom::mesh::Material {
                            name: None,
                            color: "#c9c9c9".into(),
                            metal: None,
                            rough: None,
                        },
                        mesh: None,
                        matrix: None,
                        role: None,
                        meta: std::collections::BTreeMap::new(),
                    })
                    .collect()
            } else {
                panic!("要给 --glb 或 --names");
            };
            let n_in = parts.len();
            let (out, report) = s3d_reverse::parts::apply_source_decisions(parts, &intents);
            let roles = s3d_reverse::parts::source_roles(&out);
            let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
            for r in roles.values() {
                *counts.entry(r.as_str()).or_insert(0) += 1;
            }
            let summary = serde_json::json!({
                "intents": intents.len(), "active": intents.iter().filter(|i| i.is_active()).count(), "problems": problems,
                "partsIn": n_in, "partsOut": out.len(), "roles": counts, "report": report.to_json(),
            });
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            if let Some(f) = opt("--roles-out") {
                std::fs::write(&f, serde_json::to_string(&roles).unwrap() + "\n").expect("写不出角色表");
            }
        }
        Some("roles-diff") if args.len() >= 3 => {
            // 两份 id → 角色 的表对比：s3d roles-diff <ours.json> <theirs.json>；打印一致率与混淆矩阵（ours 行、theirs 列）
            let load = |p: &str| -> std::collections::BTreeMap<String, String> {
                serde_json::from_str(&std::fs::read_to_string(p).expect("读不到角色表"))
                    .expect("角色表不是 JSON 对象")
            };
            let a = load(&args[1]);
            let b = load(&args[2]);
            let roles = ["structure", "decor", "terrain", "decal", "collision", "helper"];
            let mut m = vec![vec![0usize; roles.len()]; roles.len()];
            let (mut same, mut total, mut missing) = (0usize, 0usize, 0usize);
            let idx = |r: &str| roles.iter().position(|x| *x == r).unwrap_or(0);
            for (k, ra) in &a {
                match b.get(k) {
                    Some(rb) => {
                        total += 1;
                        if ra == rb {
                            same += 1;
                        }
                        m[idx(ra)][idx(rb)] += 1;
                    }
                    None => missing += 1,
                }
            }
            println!(
                "共同的部件 {}，一致 {}（{:.2}%），只在第一份里 {}，只在第二份里 {}",
                total,
                same,
                100.0 * same as f64 / total.max(1) as f64,
                missing,
                b.len() - total
            );
            println!(
                "{:>10} {}",
                "ours\\theirs",
                roles
                    .iter()
                    .map(|r| format!("{:>10}", r))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            for (i, r) in roles.iter().enumerate() {
                println!(
                    "{:>10} {}",
                    r,
                    m[i].iter()
                        .map(|c| format!("{:>10}", c))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
        }
        Some("intent-rebind") if args.len() >= 3 => {
            // 重绑：s3d intent-rebind <工程目录> <场景.s3d> [--glb <源模型.glb>] [--iou 0.5] [--write]
            let opt = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
            let dir = Path::new(&args[1]);
            let file = &args[2];
            let text = std::fs::read_to_string(file).expect("读不到场景");
            let display = Path::new(file)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file.clone());
            let c = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions::default(),
            );
            let scene = s3d_project::rebind::SceneIndex::from_compiled(&c);
            let sources: Vec<String> = match opt("--glb") {
                Some(g) => {
                    let bytes = std::fs::read(&g).expect("读不到 GLB");
                    let parts = s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::Auto)
                        .expect("GLB 读取失败");
                    parts
                        .iter()
                        .enumerate()
                        .flat_map(|(i, p)| vec![p.label(i), p.name.clone().unwrap_or_default()])
                        .collect()
                }
                None => Vec::new(),
            };
            let threshold = opt("--iou").and_then(|v| v.parse().ok()).unwrap_or(0.5);
            let when = opt("--when").unwrap_or_else(|| "rebind".to_string());
            let mut intents = s3d_project::store::load_all(dir).unwrap_or_else(|e| panic!("{}", e));
            let mut rows = Vec::new();
            for it in intents.iter_mut() {
                let o = s3d_project::rebind::rebind(it, &scene, &sources, threshold, &when);
                rows.push(serde_json::json!({ "id": it.id, "bound": it.status.bound, "suspicious": o.suspicious, "dangling": o.dangling, "rebound": o.rebound, "region": o.region_count }));
            }
            println!("{}", serde_json::to_string_pretty(&rows).unwrap());
            if args.iter().any(|a| a == "--write") {
                for it in &intents {
                    s3d_project::store::save(dir, it).unwrap_or_else(|e| panic!("{}", e));
                }
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
        Some("score") if args.len() >= 2 => {
            // 自评分数：s3d score <文件.s3d> --glb <源模型.glb> [--delta d] [--unit u] [--samples n] [--cres c] [--cmesh c]
            //   [--dmax d] [--edit-weight w] [--segments prototype] [--no-facts] [--full]；不给 --full 时省略 objects / sources
            let file = &args[1];
            let opt = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
            let num = |name: &str, d: f64| opt(name).and_then(|v| v.parse::<f64>().ok()).unwrap_or(d);
            let text = std::fs::read_to_string(file).expect("读不到源文件");
            let display = Path::new(file)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(file.clone());
            let facts = !args.iter().any(|a| a == "--no-facts");
            let mut out = compile(
                &text,
                &abs_path(file),
                &display,
                &read,
                &CompileOptions {
                    facts,
                    ..CompileOptions::default()
                },
            );
            let parts = match opt("--glb") {
                Some(g) => {
                    let bytes = std::fs::read(&g).expect("读不到 GLB");
                    s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::Auto).expect("GLB 读取失败")
                }
                None => Vec::new(),
            };
            let items = s3d_lang::items::collect_item_trees(&mut out);
            let nodes = s3d_lang::items::node_infos(&out);
            let files = s3d_lang::items::source_files(&out);
            let edit = s3d_score::edit::edit_report(&out, &nodes);
            let mut consts = s3d_score::score::Consts::default();
            consts.delta = num("--delta", consts.delta);
            consts.unit = num("--unit", consts.unit);
            consts.samples = num("--samples", consts.samples as f64) as usize;
            consts.c_res = num("--cres", consts.c_res);
            consts.c_mesh = num("--cmesh", consts.c_mesh);
            consts.d_max = num("--dmax", consts.d_max);
            consts.edit_weight = num("--edit-weight", consts.edit_weight);
            if opt("--segments").as_deref() == Some("prototype") {
                consts.tri = s3d_geom::tri::TriParams::PROTOTYPE;
            }
            let input = s3d_score::score::Input {
                items: &items,
                nodes: &nodes,
                files: &files,
                parts: &parts,
                consts,
                edit: Some(&edit),
            };
            let mut r = s3d_score::score::score(&input, None);
            if !args.iter().any(|a| a == "--full") {
                if let Some(o) = r.as_object_mut() {
                    o.remove("objects");
                    o.remove("sources");
                    if let Some(e) = o.get_mut("edit").and_then(|e| e.as_object_mut()) {
                        e.remove("objects");
                    }
                }
            }
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
        }
        Some("objects") if args.len() >= 2 => {
            // 调试：每个重建物体与每个源部件的三角形数与包围盒：s3d objects <文件.s3d> [--glb <源模型.glb>]
            let file = &args[1];
            let opt = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
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
            let parts = match opt("--glb") {
                Some(g) => {
                    let bytes = std::fs::read(&g).expect("读不到 GLB");
                    s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::Auto).expect("GLB 读取失败")
                }
                None => Vec::new(),
            };
            let items = s3d_lang::items::collect_item_trees(&mut out);
            let bb = |tris: &[s3d_geom::tri::Tri]| {
                let (v, a, b) = s3d_geom::tri::mesh_stats(tris);
                format!(
                    "vol={:.4} area={:.4} [{:.4},{:.4},{:.4}]..[{:.4},{:.4},{:.4}]",
                    v, a, b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z
                )
            };
            for o in s3d_score::objects::scene_objects(&items, &parts) {
                println!(
                    "obj {} tris={} color={} mesh={} {}",
                    o.path,
                    o.tris.len(),
                    o.color,
                    o.mesh.is_some(),
                    bb(&o.tris)
                );
            }
            for (i, p) in parts.iter().enumerate() {
                println!(
                    "src {} tris={} color={} {}",
                    p.label(i),
                    p.tris.len(),
                    p.material.color,
                    bb(&p.tris)
                );
            }
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
