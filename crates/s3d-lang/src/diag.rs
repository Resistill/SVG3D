//! 诊断（spec/01 1.10）：错误与警告携带文件序号、文件、行、节点路径、文案。

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// 文件序号（主文件 0，被 import 的文件按处理先后从 1 起；1.1.5）
    pub file_index: u32,
    /// 可显示路径（1.8）
    pub file: String,
    pub line: u32,
    /// 出错节点的路径；没有节点时为空
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn error(
        &mut self,
        file_index: u32,
        file: &str,
        line: u32,
        path: Option<&str>,
        message: impl Into<String>,
    ) {
        self.errors.push(Diagnostic {
            file_index,
            file: file.to_string(),
            line,
            path: path.map(str::to_string),
            message: message.into(),
        });
    }
    pub fn warning(
        &mut self,
        file_index: u32,
        file: &str,
        line: u32,
        path: Option<&str>,
        message: impl Into<String>,
    ) {
        self.warnings.push(Diagnostic {
            file_index,
            file: file.to_string(),
            line,
            path: path.map(str::to_string),
            message: message.into(),
        });
    }
    /// 1.1.5：按 (文件序号, 行号, 发现先后) 稳定排序。
    pub fn sort(&mut self) {
        self.errors.sort_by_key(|d| (d.file_index, d.line));
        self.warnings.sort_by_key(|d| (d.file_index, d.line));
    }
}
