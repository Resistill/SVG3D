//! 空间事实（spec/06）。本步先给出输出骨架；判定与扫描在后续步骤实现。

use crate::compile::Compiler;
use serde_json::{json, Value as J};

impl Compiler {
    /// facts / factsSkipped / factProviders 三个顶层字段
    pub fn facts_json(&mut self) -> Vec<(String, J)> {
        vec![
            ("facts".to_string(), json!([])),
            ("factsSkipped".to_string(), json!(false)),
            ("factProviders".to_string(), json!(["contact"])),
        ]
    }
}
