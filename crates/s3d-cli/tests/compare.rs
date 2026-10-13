//! compare / render 的集成测试：原文与自己的导出（spaceport.L0）几乎一致；同一输入两次运行，
//! stdout 与差异图逐字节相同（确定性，PLAN 原则 18）。

use std::path::PathBuf;
use std::process::Command;

fn corpus(f: &str) -> String {
    format!("{}/../../corpus/exports/{}", env!("CARGO_MANIFEST_DIR"), f)
}

fn run(args: &[&str]) -> Vec<u8> {
    let out = Command::new(env!("CARGO_BIN_EXE_s3d"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "s3d {:?} 失败：{}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn tmp(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(tag);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn spaceport_compare_close_and_deterministic() {
    let (s3d, glb) = (corpus("spaceport.L0.s3d"), corpus("spaceport.L0.glb"));
    let d1 = tmp("cmp1");
    let d2 = tmp("cmp2");
    let go = |d: &PathBuf| {
        run(&[
            "compare",
            &s3d,
            "--glb",
            &glb,
            "--size",
            "256",
            "--out-dir",
            d.to_str().unwrap(),
        ])
    };
    let o1 = go(&d1);
    let o2 = go(&d2);
    let j: serde_json::Value = serde_json::from_slice(&o1).unwrap();
    let views = j["views"].as_object().unwrap();
    assert_eq!(views.keys().cloned().collect::<Vec<_>>(), ["iso", "iso2", "top"]);
    for (v, r) in views {
        let (m, e) = (r["missing"].as_f64().unwrap(), r["extra"].as_f64().unwrap());
        assert!(m < 0.05 && e < 0.05, "{}：missing {} extra {}", v, m, e);
        let name = format!("compare-{}.png", v);
        let (p1, p2) = (
            std::fs::read(d1.join(&name)).unwrap(),
            std::fs::read(d2.join(&name)).unwrap(),
        );
        assert_eq!(p1, p2, "{} 两次不同", name);
    }
    // stdout 里的 file 带输出目录，换掉后应逐字节相同
    let s1 = String::from_utf8(o1).unwrap().replace(d1.to_str().unwrap(), "D");
    let s2 = String::from_utf8(o2).unwrap().replace(d2.to_str().unwrap(), "D");
    assert_eq!(s1, s2);
}

#[test]
fn render_modes_deterministic() {
    let (s3d, glb) = (corpus("spaceport.L0.s3d"), corpus("spaceport.L0.glb"));
    let d = tmp("render");
    for mode in ["color", "id", "labels"] {
        let mut outs = Vec::new();
        for k in 0..2 {
            let f = d.join(format!("{}-{}.png", mode, k));
            let so = run(&[
                "render",
                &s3d,
                "--glb",
                &glb,
                "--mode",
                mode,
                "--size",
                "128",
                "--out",
                f.to_str().unwrap(),
            ]);
            let j: serde_json::Value = serde_json::from_slice(&so).unwrap();
            assert!(j["box"].as_str().unwrap().split(',').count() == 6);
            outs.push(std::fs::read(&f).unwrap());
        }
        assert_eq!(outs[0], outs[1], "{} 两次不同", mode);
        assert_eq!(&outs[0][..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
    // 源模型、剖切 + 正交俯视
    let f = d.join("cut.png");
    run(&[
        "render",
        &s3d,
        "--glb",
        &glb,
        "--source",
        "--view",
        "top",
        "--ortho",
        "--cut",
        "5",
        "--mode",
        "color",
        "--size",
        "128",
        "--out",
        f.to_str().unwrap(),
    ]);
    assert!(std::fs::metadata(&f).unwrap().len() > 100);
}
