//! wasm 入口：输入一个 JSON 包 `{ "main": "<文件名>", "files": { "<文件名>": "<源码>", … } }`，
//! 输出编译结果的紧凑 JSON（与 native 的 `s3d hash` 用同一份序列化，三处哈希对拍据此比较）。
//!
//! 内存约定：宿主用 `s3d_alloc(len)` 申请输入缓冲并写入字节，调用 `s3d_compile(ptr, len)` 得到
//! 结果指针；结果的前 4 字节是小端 u32 长度，后面是 UTF-8 字节；用 `s3d_free(ptr, cap)` 释放。
//! `s3d_score(bundle, len, glb, glb_len)` 同样约定，另传源模型 GLB 的字节，输出评分的紧凑 JSON。

use s3d_lang::compile::{compile, CompileOptions};
use serde_json::Value as J;

/// 编译一个文件包，返回紧凑 JSON 字符串
pub fn compile_bundle(bundle: &str) -> String {
    let v: J = match serde_json::from_str(bundle) {
        Ok(v) => v,
        Err(e) => return format!("{{\"errors\":[{{\"message\":\"bad bundle: {}\"}}]}}", e),
    };
    let main = v["main"].as_str().unwrap_or("main.s3d").to_string();
    let files = v["files"].clone();
    let read = move |p: &str| -> Option<String> {
        let name = p.rsplit('/').next().unwrap_or(p);
        files.get(name).and_then(|t| t.as_str()).map(str::to_string)
    };
    let text = match read(&main) {
        Some(t) => t,
        None => return "{\"errors\":[{\"message\":\"main file missing\"}]}".to_string(),
    };
    let identity = format!("/bundle/{}", main);
    let mut out = compile(&text, &identity, &main, &read, &CompileOptions::default());
    let j = s3d_lang::output::to_json(&mut out);
    serde_json::to_string(&j).unwrap_or_default()
}

/// 评分一个文件包（源模型是 GLB 字节；包里可选 `"segments": "prototype"` 用原型的分段对拍）
pub fn score_bundle(bundle: &str, glb: &[u8]) -> String {
    let v: J = match serde_json::from_str(bundle) {
        Ok(v) => v,
        Err(e) => return format!("{{\"errors\":[{{\"message\":\"bad bundle: {}\"}}]}}", e),
    };
    let main = v["main"].as_str().unwrap_or("main.s3d").to_string();
    let files = v["files"].clone();
    let read = move |p: &str| -> Option<String> {
        let name = p.rsplit('/').next().unwrap_or(p);
        files.get(name).and_then(|t| t.as_str()).map(str::to_string)
    };
    let text = match read(&main) {
        Some(t) => t,
        None => return "{\"errors\":[{\"message\":\"main file missing\"}]}".to_string(),
    };
    let parts = match s3d_geom::mesh::load_parts(glb, s3d_geom::mesh::Split::Auto) {
        Ok(p) => p,
        Err(e) => return format!("{{\"errors\":[{{\"message\":\"bad glb: {}\"}}]}}", e),
    };
    let identity = format!("/bundle/{}", main);
    let mut out = compile(&text, &identity, &main, &read, &CompileOptions::default());
    let items = s3d_lang::items::collect_item_trees(&mut out);
    let nodes = s3d_lang::items::node_infos(&out);
    let src_files = s3d_lang::items::source_files(&out);
    let edit = s3d_score::edit::edit_report(&out, &nodes);
    let mut consts = s3d_score::score::Consts::default();
    if v["segments"].as_str() == Some("prototype") {
        consts.tri = s3d_geom::tri::TriParams::PROTOTYPE;
    }
    let input = s3d_score::score::Input {
        items: &items,
        nodes: &nodes,
        files: &src_files,
        parts: &parts,
        consts,
        edit: Some(&edit),
    };
    let r = s3d_score::score::score(&input, None);
    serde_json::to_string(&r).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn s3d_alloc(len: usize) -> *mut u8 {
    let mut v: Vec<u8> = Vec::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// # Safety
/// `ptr` 必须来自 `s3d_alloc` / `s3d_compile`，`cap` 是其容量
#[no_mangle]
pub unsafe extern "C" fn s3d_free(ptr: *mut u8, cap: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, cap.max(1)));
    }
}

/// # Safety
/// `ptr..ptr+len` 必须是宿主写好的 UTF-8 字节
#[no_mangle]
pub unsafe extern "C" fn s3d_compile(ptr: *const u8, len: usize) -> *mut u8 {
    let bytes = std::slice::from_raw_parts(ptr, len);
    let bundle = String::from_utf8_lossy(bytes);
    let out = compile_bundle(&bundle);
    let n = out.len() as u32;
    let mut v: Vec<u8> = Vec::with_capacity(4 + out.len());
    v.extend_from_slice(&n.to_le_bytes());
    v.extend_from_slice(out.as_bytes());
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// # Safety
/// 两段缓冲都必须是宿主写好的字节（文件包 JSON 为 UTF-8，GLB 为原始字节）
#[no_mangle]
pub unsafe extern "C" fn s3d_score(ptr: *const u8, len: usize, glb: *const u8, glb_len: usize) -> *mut u8 {
    let bytes = std::slice::from_raw_parts(ptr, len);
    let glb = std::slice::from_raw_parts(glb, glb_len);
    let bundle = String::from_utf8_lossy(bytes);
    let out = score_bundle(&bundle, glb);
    let n = out.len() as u32;
    let mut v: Vec<u8> = Vec::with_capacity(4 + out.len());
    v.extend_from_slice(&n.to_le_bytes());
    v.extend_from_slice(out.as_bytes());
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}
