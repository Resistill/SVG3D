//! 语义层（spec/07）。本步先给出输出骨架；concept / pattern / check / style / variant 在后续步骤实现。

use crate::compile::Compiler;
use serde_json::{json, Value as J};

impl Compiler {
    /// concepts / patterns / variants / checks 四个顶层字段
    pub fn semantic_json(&mut self) -> Vec<(String, J)> {
        vec![
            ("concepts".to_string(), json!([])),
            ("patterns".to_string(), json!([])),
            ("variants".to_string(), json!([])),
            ("checks".to_string(), json!([])),
        ]
    }
}
