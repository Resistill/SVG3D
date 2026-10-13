//! 阶段 1：解析与 import 内联（spec/01 1.8、spec/05 5.2）。
//! 宿主提供读文件的回调；本模块只做路径拼接与规范化、循环 / 重复判定、文件序号分配。

use crate::ast::{parse_header_lang, FileAst, Prop, Stmt, StmtKind};
use crate::diag::Diagnostics;
use crate::parser::parse_file;

/// 参与编译的一个文件（5.2.3 第 5 条分配序号；8.4.3 输出 `files[]`）
#[derive(Clone, Debug)]
pub struct FileRecord {
    pub index: u32,
    /// 可显示路径（主文件：宿主给出；被 import 的：相对主文件目录，不在其下则用身份）
    pub display: String,
    /// 规范化绝对路径（文件身份）
    pub identity: String,
    /// 文件头元数据项（未求值）
    pub header: Vec<Prop>,
    pub header_line: Option<u32>,
    /// 导入它的 import 语句：(文件序号, 行, 元数据)；主文件为 None
    pub import: Option<(u32, u32, Vec<Prop>)>,
    /// 原文（评分的描述长度用）
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub files: Vec<FileRecord>,
}

/// 宿主的文件读取：输入规范化绝对路径，读不到返回 None
pub type Reader<'a> = &'a dyn Fn(&str) -> Option<String>;

/// 把目录与相对路径拼接并规范化（1.8）。`dir` 必须是绝对路径（以 `/` 或 `字母:/` 开头）。
/// 越出文件系统的根返回 Err。
pub fn join_path(dir: &str, rel: &str) -> Result<String, String> {
    let (prefix, body) = if rel.starts_with('/') {
        ("/".to_string(), rel.trim_start_matches('/').to_string())
    } else if is_drive_abs(rel) {
        (rel[..3].to_string(), rel[3..].to_string())
    } else if dir.starts_with('/') {
        (
            "/".to_string(),
            format!("{}/{}", dir.trim_start_matches('/'), rel),
        )
    } else if is_drive_abs(dir) {
        (dir[..3].to_string(), format!("{}/{}", &dir[3..], rel))
    } else {
        // 宿主没给绝对目录：把它当作相对根（"宿主的当前目录"）
        ("".to_string(), format!("{}/{}", dir, rel))
    };
    let mut segs: Vec<&str> = Vec::new();
    for s in body.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                if segs.pop().is_none() {
                    return Err("import 路径越出根目录".to_string());
                }
            }
            x => segs.push(x),
        }
    }
    Ok(format!("{}{}", prefix, segs.join("/")))
}

fn is_drive_abs(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'/'
}

fn dir_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) => "/",
        Some(i) => &path[..i],
        None => "",
    }
}

/// 可显示路径：在主文件目录之下时取相对路径，否则取身份
fn display_of(identity: &str, main_dir: &str) -> String {
    let base = if main_dir.ends_with('/') {
        main_dir.to_string()
    } else {
        format!("{}/", main_dir)
    };
    match identity.strip_prefix(&base) {
        Some(rest) if !main_dir.is_empty() => rest.to_string(),
        _ => identity.to_string(),
    }
}

/// 加载主文件并内联全部 import。`main_identity` 是主文件的规范化绝对路径（宿主给出；
/// 从字符串编译时给 `<cwd>/<名>` 形式的假路径即可），`main_display` 是它的可显示路径。
pub fn load_program(
    main_text: &str,
    main_identity: &str,
    main_display: &str,
    read: Reader<'_>,
    diags: &mut Diagnostics,
) -> Program {
    let mut prog = Program::default();
    // 主文件身份也规范化（宿主可能给出含 `..` 的路径）
    let normalized = join_path(main_identity, "").unwrap_or_else(|_| main_identity.to_string());
    let main_identity = normalized.as_str();
    let main_dir = dir_of(main_identity).to_string();
    let mut chain: Vec<String> = vec![main_identity.to_string()];
    let mut loaded: Vec<String> = vec![main_identity.to_string()];
    let ast = parse_file(main_text, 0, main_display, diags);
    let lang_ok = check_header(&ast, 0, main_display, diags);
    prog.files.push(FileRecord {
        index: 0,
        display: main_display.to_string(),
        identity: main_identity.to_string(),
        header: ast.header.clone(),
        header_line: ast.header_line,
        import: None,
        text: main_text.to_string(),
    });
    if lang_ok {
        let mut out = Vec::new();
        inline(
            ast.stmts,
            0,
            &main_dir,
            &main_dir,
            read,
            &mut chain,
            &mut loaded,
            &mut prog,
            diags,
            &mut out,
        );
        prog.stmts = out;
    }
    prog
}

/// 文件头检查（1.3.1、5.1.4）：`lang` 缺省或 2 为合法；否则报错且该文件不编译
fn check_header(ast: &FileAst, file_index: u32, display: &str, diags: &mut Diagnostics) -> bool {
    match parse_header_lang(&ast.header) {
        Ok(()) => true,
        Err(msg) => {
            diags.error(file_index, display, ast.header_line.unwrap_or(1), None, msg);
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn inline(
    stmts: Vec<Stmt>,
    file_index: u32,
    file_dir: &str,
    main_dir: &str,
    read: Reader<'_>,
    chain: &mut Vec<String>,
    loaded: &mut Vec<String>,
    prog: &mut Program,
    diags: &mut Diagnostics,
    out: &mut Vec<Stmt>,
) {
    let display = prog.files[file_index as usize].display.clone();
    for s in stmts {
        let (path, metas) = match &s.kind {
            StmtKind::Import { path, metas } => (path.clone(), metas.clone()),
            _ => {
                out.push(s);
                continue;
            }
        };
        let line = s.line;
        // 1. 路径合法性
        if path.contains('\\') {
            diags.error(file_index, &display, line, None, "import 路径用 / 分隔");
            continue;
        }
        let identity = match join_path(file_dir, &path) {
            Ok(p) => p,
            Err(m) => {
                diags.error(file_index, &display, line, None, m);
                continue;
            }
        };
        // 2. 循环
        if let Some(pos) = chain.iter().position(|c| *c == identity) {
            let shown: Vec<String> = chain[pos..].iter().map(|c| display_of(c, main_dir)).collect();
            let msg = format!(
                "import 形成了循环：{} → {}",
                shown.join(" → "),
                display_of(&identity, main_dir)
            );
            diags.error(file_index, &display, line, None, msg);
            continue;
        }
        // 3. 重复
        if loaded.contains(&identity) {
            diags.error(
                file_index,
                &display,
                line,
                None,
                format!("文件 {} 已经 import 过", display_of(&identity, main_dir)),
            );
            continue;
        }
        // 4. 读文件
        let text = match read(&identity) {
            Some(t) => t,
            None => {
                diags.error(
                    file_index,
                    &display,
                    line,
                    None,
                    format!("import 找不到文件 {}", display_of(&identity, main_dir)),
                );
                continue;
            }
        };
        // 5. 加载
        let new_index = prog.files.len() as u32;
        loaded.push(identity.clone());
        chain.push(identity.clone());
        let new_display = display_of(&identity, main_dir);
        let ast = parse_file(&text, new_index, &new_display, diags);
        prog.files.push(FileRecord {
            index: new_index,
            display: new_display.clone(),
            identity: identity.clone(),
            header: ast.header.clone(),
            header_line: ast.header_line,
            import: Some((file_index, line, metas)),
            text: text.clone(),
        });
        if check_header(&ast, new_index, &new_display, diags) {
            let sub_dir = dir_of(&identity).to_string();
            inline(
                ast.stmts, new_index, &sub_dir, main_dir, read, chain, loaded, prog, diags, out,
            );
        }
        chain.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_paths() {
        assert_eq!(join_path("/a/b", "c.s3d").unwrap(), "/a/b/c.s3d");
        assert_eq!(join_path("/a/b", "../lib/x.s3d").unwrap(), "/a/lib/x.s3d");
        assert_eq!(join_path("/a/b", "./c/../d.s3d").unwrap(), "/a/b/d.s3d");
        assert!(join_path("/a", "../../x.s3d").is_err());
        assert_eq!(join_path("C:/w/p", "q.s3d").unwrap(), "C:/w/p/q.s3d");
        assert_eq!(join_path("/a/b", "/z/q.s3d").unwrap(), "/z/q.s3d");
    }

    #[test]
    fn imports_cycle_and_duplicate() {
        let files = |p: &str| -> Option<String> {
            Some(
                match p {
                    "/r/main.s3d" => "import \"a.s3d\"\nimport \"a.s3d\"\nbox m\n",
                    "/r/a.s3d" => "import \"main.s3d\"\nbox a\n",
                    _ => return None,
                }
                .to_string(),
            )
        };
        let mut d = Diagnostics::default();
        let p = load_program(
            &files("/r/main.s3d").unwrap(),
            "/r/main.s3d",
            "main.s3d",
            &files,
            &mut d,
        );
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.files[1].display, "a.s3d");
        assert_eq!(p.stmts.len(), 2); // a 与 m
        let msgs: Vec<&str> = d.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("循环")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("已经 import 过")), "{:?}", msgs);
        assert!(d.errors.iter().any(|e| e.file_index == 1 && e.file == "a.s3d"));
    }

    #[test]
    fn corpus_blueprint_imports_spaceport() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/examples");
        let main = format!("{}/spaceport_blueprint.s3d", dir);
        let text = std::fs::read_to_string(&main).unwrap();
        let read = |p: &str| std::fs::read_to_string(p).ok();
        let mut d = Diagnostics::default();
        let p = load_program(&text, &main, "spaceport_blueprint.s3d", &read, &mut d);
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        assert_eq!(p.files.len(), 2);
        assert_eq!(p.files[1].display, "spaceport.s3d");
        assert!(p.stmts.iter().any(|s| s.file_index == 1));
    }
}
