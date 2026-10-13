//! 词法环境（spec/01 1.5.8）：帧链与惰性绑定。

use crate::ast::Expr;
use crate::scene::NodeId;
use crate::value::Value;
use indexmap::IndexMap;

pub type FrameId = usize;

#[derive(Clone, Debug)]
pub enum Binding {
    /// 已有值（阵列下标、it、metre、参数的写明值已求…）
    Value(Value),
    /// 惰性：表达式在自己的上下文里求值，首次读取时求并缓存
    Lazy {
        expr: Expr,
        frame: FrameId,
        start: NodeId,
        line: u32,
        file_index: u32,
        state: LazyState,
    },
}

#[derive(Clone, Debug)]
pub enum LazyState {
    Unevaluated,
    Evaluating,
    Done(Value),
    /// 求值失败（错误已报）
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    Global,
    Block,
    TemplateBody,
    Instance,
    BlockIt,
    Where,
    Semantic,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub kind: FrameKind,
    pub parent: Option<FrameId>,
    pub bindings: IndexMap<String, Binding>,
}

#[derive(Debug, Default)]
pub struct Env {
    pub frames: Vec<Frame>,
}

impl Env {
    pub fn new_frame(&mut self, kind: FrameKind, parent: Option<FrameId>) -> FrameId {
        self.frames.push(Frame {
            kind,
            parent,
            bindings: IndexMap::new(),
        });
        self.frames.len() - 1
    }

    pub fn bind(&mut self, frame: FrameId, name: &str, b: Binding) {
        self.frames[frame].bindings.insert(name.to_string(), b);
    }

    pub fn has(&self, frame: FrameId, name: &str) -> bool {
        self.frames[frame].bindings.contains_key(name)
    }

    /// 沿帧链找名字，返回所在帧
    pub fn lookup(&self, mut frame: FrameId, name: &str) -> Option<FrameId> {
        loop {
            if self.frames[frame].bindings.contains_key(name) {
                return Some(frame);
            }
            frame = self.frames[frame].parent?;
        }
    }
}
