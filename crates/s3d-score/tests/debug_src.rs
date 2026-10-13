//! 调试：S3D_FILE / S3D_GLB / S3D_PATH（重建物体路径）/ S3D_SRC（源部件名）：源部件采样点到重建物体的最远点
use s3d_geom::bvh::Bvh;
use s3d_lang::compile::{compile, CompileOptions};

#[test]
#[ignore]
fn worst_source_point() {
    let file = std::env::var("S3D_FILE").unwrap();
    let glb = std::env::var("S3D_GLB").unwrap();
    let path = std::env::var("S3D_PATH").unwrap();
    let src_name = std::env::var("S3D_SRC").unwrap_or(path.clone());
    let text = std::fs::read_to_string(&file).unwrap();
    let read = |p: &str| std::fs::read_to_string(p).ok();
    let mut c = compile(&text, &file, "x.s3d", &read, &CompileOptions::default());
    let items = s3d_lang::items::collect_item_trees(&mut c);
    let bytes = std::fs::read(&glb).unwrap();
    let parts = s3d_geom::mesh::load_parts(&bytes, s3d_geom::mesh::Split::Auto).unwrap();
    let objs = s3d_score::objects::scene_objects_with(&items, &parts, &s3d_geom::tri::TriParams::PROTOTYPE);
    let o = objs.iter().find(|o| o.path == path).unwrap();
    let (i, src) = parts
        .iter()
        .enumerate()
        .find(|(i, p)| p.label(*i) == src_name)
        .unwrap();
    let bvh = Bvh::new(&o.tris);
    let pts = s3d_score::sample::source_samples(&src.tris, i, 600);
    let mut worst: Vec<(f64, s3d_geom::Vec3)> = pts
        .iter()
        .map(|p| (bvh.closest(*p, 1e9).map(|x| x.1).unwrap_or(1e9), *p))
        .collect();
    worst.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (d, p) in worst.iter().take(8) {
        println!("d={:.5} at {:.4},{:.4},{:.4}", d, p.x, p.y, p.z);
    }
    let (v, a, b) = s3d_geom::tri::mesh_stats(&o.tris);
    println!(
        "obj tris={} vol={} area={} box={:?}..{:?}",
        o.tris.len(),
        v,
        a,
        b.min,
        b.max
    );
}
