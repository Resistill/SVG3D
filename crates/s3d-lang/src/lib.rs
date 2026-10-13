//! SVG3D 编译器（spec/01、03–10）。M1 逐步填入：词法 → 语法 → 展开 → 求解 → 事实 → 语义 → 输出。

pub mod ast;
pub mod compile;
pub mod diag;
pub mod elaborate;
pub mod env;
pub mod eval;
pub mod lexer;
pub mod modify;
pub mod output;
pub mod parser;
pub mod program;
pub mod scene;
pub mod solve;
pub mod value;
