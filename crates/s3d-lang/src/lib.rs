//! SVG3D 编译器（spec/01、03–10）。M1 逐步填入：词法 → 语法 → 展开 → 求解 → 事实 → 语义 → 输出。

pub mod ast;
pub mod diag;
pub mod lexer;
pub mod parser;
