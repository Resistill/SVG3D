//! 意图文件的读写（`<工程>/intents/NNN.json`）与生命周期（4.4、4.6）。

use s3d_reverse::intent::{validate, Intent};
use std::path::{Path, PathBuf};

pub const DIR: &str = "intents";

fn file_of(dir: &Path, id: u64) -> PathBuf {
    dir.join(DIR).join(format!("{:03}.json", id))
}

/// 读工程里的全部意图，按 id 升序；文件名不是 NNN.json 的跳过。校验不过的也读进来（问题由 `problems` 给出）
pub fn load_all(dir: &Path) -> Result<Vec<Intent>, String> {
    let d = dir.join(DIR);
    let mut out = Vec::new();
    let rd = match std::fs::read_dir(&d) {
        Ok(r) => r,
        Err(_) => return Ok(out),
    };
    let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    files.sort();
    for f in files {
        if f.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{}：{}", f.display(), e))?;
        let it: Intent = serde_json::from_str(&text).map_err(|e| format!("{}：{}", f.display(), e))?;
        out.push(it);
    }
    out.sort_by_key(|it| it.id);
    Ok(out)
}

/// 读一个 JSON 文件：既可以是单条意图，也可以是 `{ "intents": [...] }` 的样本集（corpus/intents）
pub fn load_file(path: &Path) -> Result<Vec<Intent>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}：{}", path.display(), e))?;
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}：{}", path.display(), e))?;
    let list = match v.get("intents") {
        Some(a) => a.clone(),
        None => serde_json::Value::Array(vec![v]),
    };
    serde_json::from_value(list).map_err(|e| format!("{}：{}", path.display(), e))
}

/// 全部意图的问题列表（空 = 都合法）
pub fn problems(intents: &[Intent]) -> Vec<String> {
    let mut out: Vec<String> = intents.iter().flat_map(validate).collect();
    let mut seen = std::collections::BTreeSet::new();
    for it in intents {
        if !seen.insert(it.id) {
            out.push(format!("意图 {}：id 重复", it.id));
        }
    }
    out
}

/// 写一条意图（覆盖同 id 的文件）；校验不过的拒绝写
pub fn save(dir: &Path, it: &Intent) -> Result<PathBuf, String> {
    let errs = validate(it);
    if !errs.is_empty() {
        return Err(errs.join("；"));
    }
    let f = file_of(dir, it.id);
    if let Some(p) = f.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(it).map_err(|e| e.to_string())? + "\n";
    std::fs::write(&f, text).map_err(|e| format!("{}：{}", f.display(), e))?;
    Ok(f)
}

/// 下一个可用的 id（永不回收：取最大 id + 1）
pub fn next_id(intents: &[Intent]) -> u64 {
    intents.iter().map(|it| it.id).max().unwrap_or(0) + 1
}

/// 状态变迁：accept / retire；非法变迁报错
pub fn transition(it: &mut Intent, to: &str, when: &str, why: &str) -> Result<(), String> {
    let from = it.status.state.clone();
    let ok = matches!(
        (from.as_str(), to),
        ("proposed", "accepted")
            | ("accepted", "retired")
            | ("proposed", "retired")
            | ("retired", "accepted")
    );
    if !ok {
        return Err(format!("意图 {}：不能从 {} 变成 {}", it.id, from, to));
    }
    it.status.state = to.to_string();
    it.history
        .push(serde_json::json!({ "when": when, "event": to, "from": from, "why": why }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_transition() {
        let dir = std::env::temp_dir().join(format!("s3d-intents-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut it: Intent = serde_json::from_str(
            r#"{"id":1,"selection":{"sources":["a"]},"text":"x","formal":{"decisions":[{"op":"role","value":"decor"}]}}"#,
        )
        .unwrap();
        save(&dir, &it).unwrap();
        let all = load_all(&dir).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0], it);
        assert_eq!(next_id(&all), 2);
        transition(&mut it, "accepted", "t", "").unwrap();
        assert!(transition(&mut it, "proposed", "t", "").is_err());
        transition(&mut it, "retired", "t", "不要了").unwrap();
        assert_eq!(it.history.len(), 2);
        let bad: Intent = serde_json::from_str(r#"{"id":2,"formal":{"decisions":[{"op":"fly"}]}}"#).unwrap();
        assert!(save(&dir, &bad).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
