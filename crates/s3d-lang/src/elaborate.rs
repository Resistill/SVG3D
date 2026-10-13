//! 阶段 2–6（spec/04 4.1.1）：登记、展开、copy、修改、副本树定稿。
//! 本文件先实现登记与展开（含模板实例、repeat 实例）；copy / over / drop / 副本树在后续步骤补齐。

use crate::ast::{is_builtin_kind, is_keyword, Expr, Param, Prop, Stmt, StmtKind, CURVE_KINDS, SOLID_KINDS};
use crate::compile::{Compiler, Ctx, TemplateDef, Timing};
use crate::env::{Binding, FrameId, FrameKind, LazyState};
use crate::scene::{props_from, Node, NodeId, PropVal, Role, ROOT};
use crate::value::Value;
use indexmap::IndexMap;
use std::rc::Rc;

/// 几何时机的一条修改（over / drop）
#[derive(Clone, Debug)]
pub struct Modification {
    pub stmt: Rc<Stmt>,
    pub frame: FrameId,
    /// 选择器的搜索起点（写它的块的节点）
    pub start: NodeId,
    /// 作用范围（块内的 over 只作用于该块的后代；最外层为 ROOT）
    pub scope: NodeId,
    /// 展开位置（4.1.2）
    pub pos: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct PendingCopy {
    pub stmt: Rc<Stmt>,
    pub frame: FrameId,
    pub parent: NodeId,
    /// 在父节点子节点列表里占住的位置（登记时的子节点数）
    pub slot: usize,
    /// 登记序号
    pub seq: usize,
    pub pos: Vec<u64>,
    pub depth: u32,
}

pub const P_GROUP: [&str; 15] = [
    "at", "on", "under", "against", "inside", "pin", "gap", "offset", "align", "alignto", "turn", "tilt",
    "roll", "rot", "aim",
];
pub const P_GROUP_ALL: [&str; 18] = [
    "at", "on", "under", "against", "inside", "pin", "gap", "offset", "align", "alignto", "turn", "tilt",
    "roll", "rot", "aim", "aimwith", "mat", "color",
];

/// 软保留名（1.6.9）：后缀保留词与其他
const SUFFIX_RESERVED: [&str; 25] = [
    "top", "up", "bottom", "down", "left", "west", "right", "east", "front", "south", "back", "north",
    "center", "w", "h", "d", "x", "y", "z", "start", "mid", "end", "side", "surface", "origin",
];
const OTHER_SOFT: [&str; 60] = [
    "box",
    "wedge",
    "plate",
    "hull",
    "mesh",
    "cyl",
    "cone",
    "sphere",
    "revolve",
    "span",
    "sweep",
    "pipe",
    "line",
    "bezier",
    "spline",
    "arc",
    "helix",
    "group",
    "union",
    "subtract",
    "intersect",
    "repeat",
    "mirror",
    "rotate",
    "copy",
    "def",
    "let",
    "import",
    "material",
    "over",
    "drop",
    "concept",
    "pattern",
    "check",
    "style",
    "variant",
    "all",
    "count",
    "children",
    "descendants",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sqrt",
    "abs",
    "min",
    "max",
    "floor",
    "ceil",
    "round",
    "pow",
    "metre",
    "on",
    "supports",
    "against",
    "inside",
];
const HARD_FOR_NODES: [&str; 11] = [
    "it", "kind", "name", "mat", "idx", "i", "j", "k", "mirrored", "skin", "instance",
];

fn is_hard_reserved_name(s: &str) -> bool {
    is_keyword(s) || HARD_FOR_NODES.contains(&s) || is_rotate_seg(s)
}

fn is_rotate_seg(s: &str) -> bool {
    s.len() >= 2 && s.starts_with('r') && s[1..].chars().all(|c| c.is_ascii_digit())
}

impl Compiler {
    // ====== 阶段 2：登记 ======
    pub fn register(&mut self) {
        let stmts: Vec<Stmt> = self.prog.stmts.clone();
        for (i, s) in stmts.iter().enumerate() {
            match &s.kind {
                StmtKind::Def {
                    name, params, base, ..
                } => self.register_def(s, name, params, base.as_ref(), i),
                StmtKind::Material { name, props, metas } => {
                    if is_keyword(name) {
                        self.stmt_error(s.file_index, s.line, format!("关键字 {} 用错了位置", name));
                        continue;
                    }
                    let first_index = self.materials.get(name).map(|m| m.first_index).unwrap_or(i);
                    self.materials.insert(
                        name.clone(),
                        crate::compile::MaterialDef {
                            name: name.clone(),
                            props: props.clone(),
                            metas: metas.clone(),
                            line: s.line,
                            file_index: s.file_index,
                            first_index,
                        },
                    );
                }
                StmtKind::Let { name, value, .. } => {
                    if self.check_let_name(name, s) {
                        let gf = self.global_frame;
                        self.env.bind(
                            gf,
                            name,
                            Binding::Lazy {
                                expr: value.clone(),
                                frame: gf,
                                start: ROOT,
                                line: s.line,
                                file_index: s.file_index,
                                state: LazyState::Unevaluated,
                            },
                        );
                    }
                }
                StmtKind::Concept { .. }
                | StmtKind::Pattern { .. }
                | StmtKind::Check { .. }
                | StmtKind::Style { .. }
                | StmtKind::Variant { .. } => self.semantic_stmts.push(s.clone()),
                _ => {}
            }
        }
        // 外观时机的 over（4.5.1）：模板表齐了再判（use= 的参数键按最终定义检查）
        for s in &stmts {
            if let StmtKind::Over { props, .. } = &s.kind {
                if self.is_appearance_over(s) {
                    if let Some(use_p) = props.iter().find(|p| p.key == "use") {
                        let tname = self.literal_name(&use_p.value).unwrap_or_default();
                        let params = self.template_params(&tname);
                        let mut bad = None;
                        for p in props {
                            if matches!(p.key.as_str(), "use" | "mat" | "color") {
                                continue;
                            }
                            if !params.contains(&p.key) {
                                bad = Some(p.key.clone());
                                break;
                            }
                        }
                        if let Some(k) = bad {
                            self.stmt_error(s.file_index, s.line, format!("模板 {} 没有参数 {}", tname, k));
                            continue;
                        }
                    }
                    self.semantic_stmts.push(s.clone());
                }
            }
        }
    }

    /// 一条最外层的 over 是否属于外观时机（4.5.1 的四个条件；调用方保证它在最外层）
    pub fn is_appearance_over(&self, s: &Stmt) -> bool {
        match &s.kind {
            StmtKind::Over {
                props, retype, block, ..
            } => {
                if block.is_some() || retype.is_some() || props.is_empty() {
                    return false;
                }
                let has_use = props.iter().any(|p| p.key == "use");
                if has_use {
                    true
                } else {
                    props.iter().all(|p| matches!(p.key.as_str(), "mat" | "color"))
                }
            }
            _ => false,
        }
    }

    /// 模板（最终定义）的参数名表，沿派生链收集；不是模板时为空
    pub fn template_params(&self, tname: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if let Ok(chain) = self.derivation_chain(tname) {
            for d in chain.iter().rev() {
                if let StmtKind::Def { params, .. } = &d.stmt.kind {
                    for p in params {
                        if !out.contains(&p.name) {
                            out.push(p.name.clone());
                        }
                    }
                }
            }
        }
        out
    }

    fn check_let_name(&mut self, name: &str, s: &Stmt) -> bool {
        if is_keyword(name)
            || name == "metre"
            || HARD_FOR_NODES.contains(&name)
            || is_builtin_kind(name)
            || OTHER_SOFT.contains(&name)
            || (SUFFIX_RESERVED.contains(&name) && !matches!(name, "w" | "h" | "d" | "x" | "y" | "z"))
        {
            self.stmt_error(
                s.file_index,
                s.line,
                format!("{} 是保留名，不能用作 let 名", name),
            );
            return false;
        }
        true
    }

    fn register_def(
        &mut self,
        s: &Stmt,
        name: &str,
        params: &[Param],
        base: Option<&(String, Vec<Prop>)>,
        index: usize,
    ) {
        if is_builtin_kind(name) {
            self.stmt_error(s.file_index, s.line, "模板不能与内置种类同名");
            return;
        }
        if is_keyword(name)
            || HARD_FOR_NODES.contains(&name)
            || OTHER_SOFT.contains(&name)
            || SUFFIX_RESERVED.contains(&name)
        {
            self.stmt_error(s.file_index, s.line, format!("{} 是保留名，不能用作模板名", name));
            return;
        }
        let mut seen: Vec<&str> = Vec::new();
        for p in params {
            if is_keyword(&p.name) || (HARD_FOR_NODES.contains(&p.name.as_str())) {
                self.stmt_error(
                    s.file_index,
                    p.line,
                    format!("{} 是保留名，不能用作参数名", p.name),
                );
                return;
            }
            if P_GROUP_ALL.contains(&p.name.as_str()) || p.name == "use" {
                self.stmt_error(s.file_index, p.line, format!("参数名与属性名重名：{}", p.name));
                return;
            }
            if seen.contains(&p.name.as_str()) {
                self.stmt_error(s.file_index, p.line, format!("重复的参数 {}", p.name));
                return;
            }
            seen.push(&p.name);
        }
        if let Some((bname, overrides)) = base {
            for o in overrides {
                if params.iter().any(|p| p.name == o.key) {
                    self.stmt_error(
                        s.file_index,
                        o.line,
                        format!("参数 {} 既被覆盖又被重新声明", o.key),
                    );
                    return;
                }
            }
            if bname == name {
                // 就地扩展：捕获当前定义
                match self.defs.get(name).cloned() {
                    Some(cur) => {
                        let first_index = cur.first_index;
                        self.defs.insert(
                            name.to_string(),
                            TemplateDef {
                                name: name.to_string(),
                                stmt: s.clone(),
                                extends: Some(Box::new(cur)),
                                first_index,
                            },
                        );
                    }
                    None => {
                        self.stmt_error(s.file_index, s.line, format!("要扩展的类型 {} 还没有定义", name))
                    }
                }
                return;
            }
        }
        let first_index = self.defs.get(name).map(|d| d.first_index).unwrap_or(index);
        self.defs.insert(
            name.to_string(),
            TemplateDef {
                name: name.to_string(),
                stmt: s.clone(),
                extends: None,
                first_index,
            },
        );
    }

    // ====== 阶段 3：展开 ======
    pub fn expand_all(&mut self) {
        let stmts: Vec<Stmt> = self.prog.stmts.clone();
        let gf = self.global_frame;
        // 最外层：全局帧已含 let；最外层语句的块帧就是全局帧
        self.expand_block(&stmts, ROOT, gf, 0, &[]);
    }

    /// 展开一个块里的语句（块帧已建好并含 let）
    pub(crate) fn expand_block(
        &mut self,
        stmts: &[Stmt],
        parent: NodeId,
        frame: FrameId,
        depth: u32,
        pos_prefix: &[u64],
    ) {
        for (i, s) in stmts.iter().enumerate() {
            let mut pos = pos_prefix.to_vec();
            if pos_prefix.is_empty() {
                pos = vec![s.file_index as u64, s.line as u64, i as u64];
            }
            self.expand_stmt(s, parent, frame, depth, &pos);
        }
    }

    /// 新建块帧并登记块内 let（同名报错"重复定义 X"）
    pub(crate) fn block_frame(
        &mut self,
        stmts: &[Stmt],
        parent_frame: FrameId,
        kind: FrameKind,
        start: NodeId,
    ) -> FrameId {
        let f = self.env.new_frame(kind, Some(parent_frame));
        self.bind_lets(stmts, f, start);
        f
    }

    pub(crate) fn bind_lets(&mut self, stmts: &[Stmt], f: FrameId, start: NodeId) {
        for s in stmts {
            if let StmtKind::Let { name, value, .. } = &s.kind {
                if !self.check_let_name(name, s) {
                    continue;
                }
                if self.env.has(f, name) {
                    self.stmt_error(s.file_index, s.line, format!("重复定义 {}", name));
                    continue;
                }
                self.env.bind(
                    f,
                    name,
                    Binding::Lazy {
                        expr: value.clone(),
                        frame: f,
                        start,
                        line: s.line,
                        file_index: s.file_index,
                        state: LazyState::Unevaluated,
                    },
                );
            }
        }
    }

    pub(crate) fn expand_stmt(&mut self, s: &Stmt, parent: NodeId, frame: FrameId, depth: u32, pos: &[u64]) {
        if depth > 32 {
            self.stmt_error(s.file_index, s.line, "展开层数超过 32（模板递归？）");
            return;
        }
        match &s.kind {
            StmtKind::Generic { .. } => {
                self.expand_generic(s, parent, frame, depth, pos);
            }
            StmtKind::Let { .. } => {}
            StmtKind::Copy { .. } => {
                // 占位：先在父节点下占一个序号（1.6.1 第 2 条），阶段 4 展开；
                // 阶段 5 以后（over 块、重建的模板体里）立即展开（4.3.5 末段）
                let slot = self.scene.nodes[parent].children.len();
                self.scene.nodes[parent].created += 1;
                let seq = self.copy_seq;
                self.copy_seq += 1;
                let c = PendingCopy {
                    stmt: Rc::new(s.clone()),
                    frame,
                    parent,
                    slot,
                    seq,
                    pos: pos.to_vec(),
                    depth,
                };
                if self.immediate_copies {
                    self.expand_copy_now(&c);
                } else {
                    self.copies.push(c);
                }
            }
            StmtKind::Over {
                props, block, retype, ..
            } => {
                if props.is_empty() && block.is_none() && retype.is_none() {
                    self.stmt_error(s.file_index, s.line, "over 需要属性或块");
                    return;
                }
                // 外观时机的 over（4.5.1）在阶段 2 登记为语义层语句，不进修改序列
                if parent == ROOT && self.is_appearance_over(s) {
                    return;
                }
                if props.iter().any(|p| p.key == "use") {
                    self.stmt_error(
                        s.file_index,
                        s.line,
                        "use= 只能写在最外层、不带块与换形、只改外观的 over 里",
                    );
                    return;
                }
                self.modifications.push(Modification {
                    stmt: Rc::new(s.clone()),
                    frame,
                    start: parent,
                    scope: parent,
                    pos: pos.to_vec(),
                });
            }
            StmtKind::Drop { .. } => {
                self.modifications.push(Modification {
                    stmt: Rc::new(s.clone()),
                    frame,
                    start: parent,
                    scope: parent,
                    pos: pos.to_vec(),
                });
            }
            StmtKind::Def { .. }
            | StmtKind::Material { .. }
            | StmtKind::Import { .. }
            | StmtKind::Concept { .. }
            | StmtKind::Pattern { .. }
            | StmtKind::Check { .. }
            | StmtKind::Style { .. }
            | StmtKind::Variant { .. } => {
                if parent != ROOT {
                    let what = match &s.kind {
                        StmtKind::Def { .. } => "def",
                        StmtKind::Material { .. } => "material",
                        StmtKind::Import { .. } => "import",
                        StmtKind::Concept { .. } => "concept",
                        StmtKind::Pattern { .. } => "pattern",
                        StmtKind::Check { .. } => "check",
                        StmtKind::Style { .. } => "style",
                        _ => "variant",
                    };
                    self.stmt_error(s.file_index, s.line, format!("{} 只能写在最外层", what));
                }
            }
        }
    }

    /// 元数据求值（展开时；1.9、8.1）
    pub(crate) fn eval_metas(&mut self, metas: &[Prop], frame: FrameId, s: &Stmt) -> IndexMap<String, Value> {
        let mut out = IndexMap::new();
        for m in metas {
            if out.contains_key(&m.key) {
                self.stmt_error(s.file_index, m.line, format!("重复的元数据 {}", m.key));
                continue;
            }
            let ctx = Ctx {
                frame,
                start: ROOT,
                timing: Timing::Expand,
            };
            match self.eval(&m.value, ctx) {
                Ok(v) => {
                    out.insert(m.key.clone(), v);
                }
                Err(e) => self.stmt_error(s.file_index, m.line, format!("元数据 {}：{}", m.key, e.0)),
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn new_node(
        &mut self,
        s: &Stmt,
        kind: &str,
        name: Option<&str>,
        tags: &[(String, u32)],
        props: &[Prop],
        metas: &[Prop],
        parent: NodeId,
        frame: FrameId,
    ) -> NodeId {
        let meta = self.eval_metas(metas, frame, s);
        let mut tag_names: Vec<String> = Vec::new();
        let mut dup = false;
        for (t, line) in tags {
            if tag_names.contains(t) {
                self.stmt_error(s.file_index, *line, format!("重复的标签 #{}", t));
                dup = true;
            } else {
                tag_names.push(t.clone());
            }
        }
        let mut seen: Vec<&str> = Vec::new();
        for p in props {
            if seen.contains(&p.key.as_str()) {
                self.stmt_error(s.file_index, p.line, format!("重复的属性 {}", p.key));
                dup = true;
            }
            seen.push(&p.key);
        }
        let props_table = match self.prop_override.take() {
            Some(t) => t,
            None => props_from(props, frame, parent, s.file_index),
        };
        let node = Node {
            seg: name.unwrap_or("").to_string(),
            name: name.unwrap_or("").to_string(),
            kind: kind.to_string(),
            types: vec![],
            role: Role::Normal,
            parent,
            children: vec![],
            created: 0,
            props: props_table,
            tags: tag_names,
            meta,
            file_index: s.file_index,
            line: s.line,
            frame,
            stmt_frame: frame,
            degenerate: dup,
            hidden: false,
            late: false,
            local: None,
            lbox: None,
            world: None,
            wbox: None,
            tight: None,
            geom: None,
            curve: None,
            src_stmt: Some(Rc::new(s.clone())),
            depth: 0,
            pos: vec![],
            dropped: false,
            concepts: vec![],
            placement_ref: None,
        };
        let id = self.scene.add(parent, node);
        if let Some(n) = name {
            if SUFFIX_RESERVED.contains(&n) {
                self.node_warning(
                    id,
                    format!("节点名 {} 与保留名相同，作为路径的后续段时不能按名字引用", n),
                );
            } else if OTHER_SOFT.contains(&n) {
                self.node_warning(id, format!("节点名 {} 与保留名相同", n));
            }
        }
        id
    }

    pub(crate) fn expand_generic(
        &mut self,
        s: &Stmt,
        parent: NodeId,
        frame: FrameId,
        depth: u32,
        pos: &[u64],
    ) -> Option<NodeId> {
        let (kind, name, tags, props, metas) = match &s.kind {
            StmtKind::Generic {
                kind,
                name,
                tags,
                props,
                metas,
                ..
            } => (kind, name, tags, props, metas),
            _ => unreachable!(),
        };
        if let Some(n) = name {
            if is_hard_reserved_name(n) && !(self.mounting_skin && n == "skin") {
                self.stmt_error(s.file_index, s.line, format!("{} 是保留名，不能用作节点名", n));
                return None;
            }
        }
        let kind_s = kind.as_str();
        let id = self.expand_generic_inner(
            s,
            kind_s,
            name.as_deref(),
            tags,
            props,
            metas,
            parent,
            frame,
            depth,
            pos,
        );
        if let Some(id) = id {
            self.scene.nodes[id].depth = depth;
            self.scene.nodes[id].pos = pos.to_vec();
        }
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn expand_generic_inner(
        &mut self,
        s: &Stmt,
        kind_s: &str,
        name: Option<&str>,
        tags: &[(String, u32)],
        props: &[Prop],
        metas: &[Prop],
        parent: NodeId,
        frame: FrameId,
        depth: u32,
        pos: &[u64],
    ) -> Option<NodeId> {
        let name = name.map(|x| x.to_string());
        let block: Option<Vec<Stmt>> = match &s.kind {
            StmtKind::Generic { block, .. } => block.clone(),
            _ => None,
        };
        let block = &block;
        if SOLID_KINDS.contains(&kind_s) || CURVE_KINDS.contains(&kind_s) {
            let id = self.new_node(s, kind_s, name.as_deref(), tags, props, metas, parent, frame);
            self.check_prop_keys(id, kind_s);
            return Some(id);
        }
        match kind_s {
            "group" | "union" | "subtract" | "intersect" => {
                let id = self.new_node(s, kind_s, name.as_deref(), tags, props, metas, parent, frame);
                self.check_prop_keys(id, kind_s);
                let body = block.clone().unwrap_or_default();
                let bf = self.block_frame(&body, frame, FrameKind::Block, id);
                self.scene.nodes[id].frame = bf;
                self.expand_block(&body, id, bf, depth, pos);
                Some(id)
            }
            "repeat" => {
                let id = self.new_node(s, "repeat", name.as_deref(), tags, props, metas, parent, frame);
                self.check_prop_keys(id, "repeat");
                self.expand_repeat(s, id, frame, depth, pos);
                Some(id)
            }
            "mirror" | "rotate" => {
                let has_of = props.iter().any(|p| p.key == "of");
                let id = self.new_node(s, kind_s, name.as_deref(), tags, props, metas, parent, frame);
                if has_of {
                    // 副本式：原件在阶段 6 解析；种类随原件
                    self.scene.nodes[id].role = Role::TwinOf { original: ROOT };
                    self.check_prop_keys(
                        id,
                        if kind_s == "mirror" {
                            "mirror_of"
                        } else {
                            "rotate_of"
                        },
                    );
                } else {
                    self.check_prop_keys(id, kind_s);
                    let body = block.clone().unwrap_or_default();
                    let bf = self.block_frame(&body, frame, FrameKind::Block, id);
                    self.scene.nodes[id].frame = bf;
                    self.expand_block(&body, id, bf, depth, pos);
                }
                Some(id)
            }
            _ => self.expand_instance(
                s,
                kind_s,
                name.as_deref(),
                tags,
                props,
                metas,
                parent,
                frame,
                depth,
                pos,
            ),
        }
    }

    /// 检查属性键是否被该种类接受（1.4.4、1.4.5）；不接受的报错并退化
    pub fn check_prop_keys(&mut self, id: NodeId, kind: &str) {
        let keys: Vec<String> = self.scene.nodes[id].props.keys().cloned().collect();
        for k in keys {
            if !accepts_prop(kind, &k) {
                let msg = format!("{} 没有属性 {}", kind.trim_end_matches("_of"), k);
                self.degenerate(id, msg);
            }
        }
        // 互斥：五个定位关系至多一个
        let rel: Vec<&str> = ["at", "on", "under", "against", "inside"]
            .iter()
            .copied()
            .filter(|k| self.scene.nodes[id].props.contains_key(*k))
            .collect();
        if rel.len() > 1 {
            self.degenerate(id, "定位关系只能写一个");
        }
        let has = |c: &Compiler, k: &str| c.scene.nodes[id].props.contains_key(k);
        if has(self, "alignto") && !has(self, "align") {
            self.degenerate(id, "alignto 需要 align");
        }
        if has(self, "aimwith") && !has(self, "aim") {
            self.degenerate(id, "aimwith 需要 aim");
        }
        if kind == "repeat" {
            if has(self, "along") && (has(self, "step") || has(self, "center")) {
                self.degenerate(id, "along 与 step / center 不能同时写");
            }
            if has(self, "orient") && !has(self, "along") {
                self.degenerate(id, "orient 只能和 along 一起写");
            }
        }
    }

    // —— 模板实例（4.2.3–4.2.6）——
    #[allow(clippy::too_many_arguments)]
    fn expand_instance(
        &mut self,
        s: &Stmt,
        tname: &str,
        name: Option<&str>,
        tags: &[(String, u32)],
        props: &[Prop],
        metas: &[Prop],
        parent: NodeId,
        frame: FrameId,
        depth: u32,
        pos: &[u64],
    ) -> Option<NodeId> {
        if !self.defs.contains_key(tname) {
            self.stmt_error(s.file_index, s.line, format!("未知的语句种类 {}", tname));
            return None;
        }
        let id = self.new_node(s, tname, name, tags, props, metas, parent, frame);
        // 派生链（最派生在前）
        let chain = match self.derivation_chain(tname) {
            Ok(c) => c,
            Err((file_index, line, msg)) => {
                let file = self.file_display(file_index);
                let path = self.scene.path(id);
                self.diags.error(file_index, &file, line, Some(&path), msg);
                self.scene.nodes[id].degenerate = true;
                return Some(id);
            }
        };
        let mut types: Vec<String> = Vec::new();
        for d in &chain {
            if !types.contains(&d.name) {
                types.push(d.name.clone());
            }
        }
        self.scene.nodes[id].types = types;
        // 参数表：从链尾到链头
        let mut params: IndexMap<String, (Option<Expr>, u32, u32)> = IndexMap::new(); // name → (默认值, line, file)
        let mut chain_err: Option<(u32, u32, String)> = None;
        for d in chain.iter().rev() {
            let (dparams, base) = match &d.stmt.kind {
                StmtKind::Def { params, base, .. } => (params, base),
                _ => unreachable!(),
            };
            if let Some((_, overrides)) = base {
                for o in overrides {
                    if !params.contains_key(&o.key) {
                        chain_err = Some((
                            d.stmt.file_index,
                            d.stmt.line,
                            format!("{} 覆盖了基类型没有的参数 {}", d.name, o.key),
                        ));
                        break;
                    }
                    params.insert(
                        o.key.clone(),
                        (Some(o.value.clone()), d.stmt.line, d.stmt.file_index),
                    );
                }
            }
            if chain_err.is_some() {
                break;
            }
            for p in dparams {
                params.insert(
                    p.name.clone(),
                    (p.default.clone(), d.stmt.line, d.stmt.file_index),
                );
            }
        }
        if let Some((fi, line, msg)) = chain_err {
            let file = self.file_display(fi);
            let path = self.scene.path(id);
            self.diags.error(fi, &file, line, Some(&path), msg);
            self.scene.nodes[id].degenerate = true;
            return Some(id);
        }
        // 模板体帧：父帧是全局帧
        let gf = self.global_frame;
        let body_frame = self.env.new_frame(FrameKind::TemplateBody, Some(gf));
        self.scene.nodes[id].frame = body_frame;
        let mut missing = false;
        let table = self.scene.nodes[id].props.clone();
        let mut unknown_keys: Vec<String> = Vec::new();
        for k in table.keys() {
            if !P_GROUP_ALL.contains(&k.as_str()) && !params.contains_key(k) {
                unknown_keys.push(k.clone());
            }
        }
        for k in unknown_keys {
            let msg = format!("模板 {} 没有参数 {}", tname, k);
            self.degenerate(id, msg);
            self.scene.nodes[id].props.shift_remove(&k);
        }
        let _ = props;
        for (pname, (default, dline, dfile)) in &params {
            // 写明的值在写它的上下文里求值（有效属性表带各自的帧，4.1.3）
            let written = table.get(pname);
            let binding = match (written, default) {
                (Some(w), _) => Binding::Lazy {
                    expr: w.expr.clone(),
                    frame: w.frame,
                    start: w.start,
                    line: w.line,
                    file_index: w.file_index,
                    state: LazyState::Unevaluated,
                },
                (None, Some(d)) => Binding::Lazy {
                    expr: d.clone(),
                    frame: body_frame,
                    start: parent,
                    line: *dline,
                    file_index: *dfile,
                    state: LazyState::Unevaluated,
                },
                (None, None) => {
                    let msg = format!("模板 {} 缺少参数 {}", tname, pname);
                    self.degenerate(id, msg);
                    missing = true;
                    continue;
                }
            };
            self.env.bind(body_frame, pname, binding);
            // 写明的参数留在有效属性表里（重建与换形要用，4.5.4、4.6.2）；它们不是几何属性，求解不读
        }
        if missing {
            return Some(id);
        }
        // 体：先最基础的
        let bodies: Vec<Vec<Stmt>> = chain
            .iter()
            .rev()
            .map(|d| match &d.stmt.kind {
                StmtKind::Def { body, .. } => body.clone(),
                _ => unreachable!(),
            })
            .collect();
        for body in &bodies {
            self.bind_lets(body, body_frame, id);
        }
        for body in &bodies {
            self.expand_block(body, id, body_frame, depth + 1, pos);
        }
        Some(id)
    }

    /// 派生链：最派生在前（4.2.5、4.2.6）
    pub(crate) fn derivation_chain(&self, tname: &str) -> Result<Vec<TemplateDef>, (u32, u32, String)> {
        let mut out: Vec<TemplateDef> = Vec::new();
        let mut cur = self.defs.get(tname).cloned();
        let mut seen: Vec<String> = Vec::new();
        let mut guard = 0;
        while let Some(d) = cur {
            guard += 1;
            if guard > 32 {
                return Err((
                    d.stmt.file_index,
                    d.stmt.line,
                    format!("类型 {} 的派生链形成了循环", tname),
                ));
            }
            let base_name = match &d.stmt.kind {
                StmtKind::Def { base, .. } => base.as_ref().map(|(b, _)| b.clone()),
                _ => None,
            };
            let key = format!("{}@{}:{}", d.name, d.stmt.file_index, d.stmt.line);
            if seen.contains(&key) {
                return Err((
                    d.stmt.file_index,
                    d.stmt.line,
                    format!("类型 {} 的派生链形成了循环", tname),
                ));
            }
            seen.push(key);
            let ext = d.extends.clone();
            let (fi, line, dname) = (d.stmt.file_index, d.stmt.line, d.name.clone());
            out.push(d);
            cur = match ext {
                Some(e) => Some(*e),
                None => match base_name {
                    None => None,
                    Some(b) if b == dname => None,
                    Some(b) => match self.defs.get(&b) {
                        Some(bd) => Some(bd.clone()),
                        None => return Err((fi, line, format!("{} 的基类型 {} 没有定义", dname, b))),
                    },
                },
            };
        }
        Ok(out)
    }

    // —— repeat（3.8）——
    fn expand_repeat(&mut self, s: &Stmt, id: NodeId, frame: FrameId, depth: u32, pos: &[u64]) {
        let body = match &s.kind {
            StmtKind::Generic { block, .. } => block.clone().unwrap_or_default(),
            _ => unreachable!(),
        };
        // n 是常量，展开时求值
        let n_prop = self.scene.nodes[id].props.get("n").cloned();
        let (ni, nj, nk) = match n_prop {
            None => {
                self.degenerate(id, format!("{} 缺少 n", self.scene.path(id)));
                return;
            }
            Some(pv) => {
                let ctx = Ctx {
                    frame: pv.frame,
                    start: ROOT,
                    timing: Timing::Expand,
                };
                let v = match self.eval(&pv.expr, ctx) {
                    Ok(v) => v,
                    Err(e) => {
                        self.degenerate(id, format!("repeat 的 n 必须是常量：{}", e.0));
                        return;
                    }
                };
                let to_int = |x: f64| -> Option<u32> {
                    let r = x.round();
                    if (x - r).abs() <= 1e-9 && (1.0..=5000.0).contains(&r) {
                        Some(r as u32)
                    } else {
                        None
                    }
                };
                let dims = match v {
                    Value::Num(x) => to_int(x).map(|a| (a, 1, 1)),
                    Value::Vector(ref vv) if vv.len() == 3 => {
                        match (to_int(vv[0]), to_int(vv[1]), to_int(vv[2])) {
                            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                match dims {
                    Some(d) => d,
                    None => {
                        self.degenerate(id, format!("{} 的 n 需要正整数或三个正整数", self.scene.path(id)));
                        return;
                    }
                }
            }
        };
        let total = ni as u64 * nj as u64 * nk as u64;
        if total > 5000 {
            self.degenerate(id, format!("{} 的实例数超过 5000", self.scene.path(id)));
            return;
        }
        for k in 0..nk {
            for j in 0..nj {
                for i in 0..ni {
                    let idx = i + ni * (j + nj * k);
                    let inst_frame = self.env.new_frame(FrameKind::Instance, Some(frame));
                    for (nm, v) in [("i", i), ("j", j), ("k", k), ("idx", idx)] {
                        self.env
                            .bind(inst_frame, nm, Binding::Value(Value::Num(v as f64)));
                    }
                    let inst = Node {
                        seg: idx.to_string(),
                        name: idx.to_string(),
                        kind: "instance".to_string(),
                        types: vec![],
                        role: Role::Instance { i, j, k, idx },
                        parent: id,
                        children: vec![],
                        created: 0,
                        props: IndexMap::new(),
                        tags: vec![],
                        meta: IndexMap::new(),
                        file_index: s.file_index,
                        line: s.line,
                        frame: inst_frame,
                        stmt_frame: inst_frame,
                        degenerate: false,
                        hidden: false,
                        late: false,
                        local: None,
                        lbox: None,
                        world: None,
                        wbox: None,
                        tight: None,
                        geom: None,
                        curve: None,
                        src_stmt: None,
                        depth: 0,
                        pos: vec![],
                        dropped: false,
                        concepts: vec![],
                        placement_ref: None,
                    };
                    let inst_id = self.scene.add(id, inst);
                    self.scene.nodes[inst_id].depth = depth;
                    self.scene.nodes[inst_id].pos = pos.to_vec();
                    let bf = self.block_frame(&body, inst_frame, FrameKind::Block, inst_id);
                    self.scene.nodes[inst_id].frame = bf;
                    self.expand_block(&body, inst_id, bf, depth, pos);
                }
            }
        }
    }

    // ====== 阶段 4–6（后续补齐）======
    pub fn finish_tree(&mut self) {
        self.expand_copies();
        self.apply_modifications();
        self.build_twins();
        self.apply_deferred_modifications();
        self.check_sibling_names();
        self.rebuild_order();
    }

    /// 阶段 6：副本树定稿（4.4.2）。副本根按依赖拓扑序生成：先里面的、再外面的。
    fn build_twins(&mut self) {
        let mut roots: Vec<NodeId> = (1..self.scene.nodes.len())
            .filter(|&n| {
                let node = &self.scene.nodes[n];
                matches!(node.role, Role::TwinOf { .. })
                    || (matches!(node.role, Role::Normal)
                        && matches!(node.kind.as_str(), "mirror" | "rotate"))
            })
            .collect();
        let mut done: Vec<NodeId> = Vec::new();
        let mut guard = 0;
        while !roots.is_empty() {
            guard += 1;
            let mut progressed = false;
            let mut i = 0;
            while i < roots.len() {
                let r = roots[i];
                let ready = match self.scene.nodes[r].role.clone() {
                    Role::TwinOf { .. } => match self.resolve_of(r) {
                        Ok(a) => {
                            // 依赖：A 自身是副本根、A 子树里的副本根
                            let inside = self.subtree(a);
                            let pending = roots.iter().any(|&x| x != r && (x == a || inside.contains(&x)));
                            if pending {
                                None
                            } else {
                                Some(Some(a))
                            }
                        }
                        Err(None) => None, // 暂时解析不到（可能指向尚未生成的影子）
                        Err(Some(_)) => Some(None),
                    },
                    _ => {
                        let inside = self.subtree(r);
                        let pending = roots.iter().any(|&x| x != r && inside.contains(&x));
                        if pending {
                            None
                        } else {
                            Some(None)
                        }
                    }
                };
                match ready {
                    None => i += 1,
                    Some(a) => {
                        self.make_twins(r, a);
                        done.push(r);
                        roots.remove(i);
                        progressed = true;
                    }
                }
            }
            if !progressed || guard > 10000 {
                for r in roots.clone() {
                    let path = self.scene.path(r);
                    match self.resolve_of(r) {
                        Err(Some(msg)) => self.degenerate(r, msg),
                        _ => self.degenerate(r, format!("复制关系形成循环：{}", path)),
                    }
                }
                break;
            }
        }
    }

    /// 子树里的全部节点（不含自己）
    fn subtree(&self, n: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack: Vec<NodeId> = self.scene.nodes[n].children.clone();
        while let Some(c) = stack.pop() {
            out.push(c);
            stack.extend(self.scene.nodes[c].children.iter().copied());
        }
        out
    }

    /// 解析副本式节点的 of=：Ok(A)；Err(None) = 暂时找不到；Err(Some(msg)) = 确定的错误
    fn resolve_of(&mut self, r: NodeId) -> Result<NodeId, Option<String>> {
        let pv = match self.scene.nodes[r].props.get("of").cloned() {
            Some(pv) => pv,
            None => return Err(Some(format!("{}：of 需要写一个对象名", self.scene.path(r)))),
        };
        if !matches!(
            pv.expr.kind,
            crate::ast::ExprKind::Ident(_)
                | crate::ast::ExprKind::PathRef {
                    coords: None,
                    axis: None,
                    ..
                }
        ) {
            return Err(Some(format!("{}：of 需要写一个对象名", self.scene.path(r))));
        }
        let ctx = Ctx {
            frame: pv.frame,
            start: pv.start,
            timing: Timing::Selector,
        };
        let saved = self.diags.errors.len();
        let res = self.eval(&pv.expr, ctx);
        self.diags.errors.truncate(saved);
        match res {
            Ok(Value::Ref(rf)) if rf.anchor.is_none() => {
                let a = rf.node;
                if a == r || self.scene.is_ancestor(a, r) {
                    return Err(Some(format!("{} 不能复制包含自己的对象", self.scene.path(r))));
                }
                Ok(a)
            }
            Ok(_) => Err(Some(format!("{}：of 需要写一个对象名", self.scene.path(r)))),
            Err(e) => {
                if e.0.contains("找不到对象") || e.0.contains("没有子对象") {
                    Err(None)
                } else {
                    Err(Some(e.0))
                }
            }
        }
    }

    fn make_twins(&mut self, root: NodeId, of: Option<NodeId>) {
        match of {
            Some(a) => {
                // 副本式 R：种类、类型链、标签随 A；子节点是 A 的子节点的影子
                let (kind, types, tags) = {
                    let an = &self.scene.nodes[a];
                    (an.kind.clone(), an.types.clone(), an.tags.clone())
                };
                {
                    let rn = &mut self.scene.nodes[root];
                    rn.kind = kind;
                    rn.types = types;
                    for t in tags {
                        if !rn.tags.contains(&t) {
                            rn.tags.push(t);
                        }
                    }
                    rn.role = Role::TwinOf { original: a };
                }
                let kids = self.scene.nodes[a].children.clone();
                for c in kids {
                    self.shadow(c, root);
                }
            }
            None => {
                let kind = self.scene.nodes[root].kind.clone();
                let originals = self.scene.nodes[root].children.clone();
                let groups = if kind == "mirror" {
                    1
                } else {
                    match self.rotate_n(root) {
                        Some(n) => n,
                        None => return,
                    }
                };
                for k in 1..=groups {
                    let seg = if kind == "mirror" {
                        "mirrored".to_string()
                    } else {
                        format!("r{}", k)
                    };
                    let (fi, line, frame) = {
                        let c = &self.scene.nodes[root];
                        (c.file_index, c.line, c.frame)
                    };
                    let g = Node {
                        seg: seg.clone(),
                        name: seg,
                        kind: kind.clone(),
                        types: vec![],
                        role: Role::TwinGroup { k },
                        parent: root,
                        children: vec![],
                        created: 0,
                        props: IndexMap::new(),
                        tags: vec![],
                        meta: IndexMap::new(),
                        file_index: fi,
                        line,
                        frame,
                        stmt_frame: frame,
                        degenerate: false,
                        hidden: false,
                        late: false,
                        local: None,
                        lbox: None,
                        world: None,
                        wbox: None,
                        tight: None,
                        geom: None,
                        curve: None,
                        src_stmt: None,
                        depth: 0,
                        pos: vec![],
                        dropped: false,
                        concepts: vec![],
                        placement_ref: None,
                    };
                    let gid = self.scene.add(root, g);
                    for &c in &originals {
                        self.shadow(c, gid);
                    }
                }
            }
        }
    }

    /// rotate 容器式的 n（常量，1..360）
    fn rotate_n(&mut self, c: NodeId) -> Option<u32> {
        let pv = match self.scene.nodes[c].props.get("n").cloned() {
            None => return Some(1),
            Some(pv) => pv,
        };
        let ctx = Ctx {
            frame: pv.frame,
            start: ROOT,
            timing: Timing::Expand,
        };
        let path = self.scene.path(c);
        match self.eval(&pv.expr, ctx) {
            Ok(Value::Num(x)) => {
                let r = x.round();
                if (x - r).abs() <= 1e-9 && (1.0..=360.0).contains(&r) {
                    Some(r as u32)
                } else {
                    self.degenerate(c, format!("{}：rotate 的 n 需要 1..360 的整数", path));
                    None
                }
            }
            Ok(_) => {
                self.degenerate(c, format!("{}：rotate 的 n 需要 1..360 的整数", path));
                None
            }
            Err(_) => {
                self.degenerate(c, "rotate 的 n 必须是常量");
                None
            }
        }
    }

    /// shadow(o, p)（4.4.2）
    fn shadow(&mut self, o: NodeId, p: NodeId) -> NodeId {
        let (seg, kind, types, tags, meta, fi, line, frame) = {
            let on = &self.scene.nodes[o];
            let seg = if on.name.is_empty() && matches!(on.role, Role::Normal | Role::TwinOf { .. }) {
                String::new() // 匿名：按 p 的计数器重新编号
            } else {
                on.seg.clone()
            };
            (
                seg,
                on.kind.clone(),
                on.types.clone(),
                on.tags.clone(),
                on.meta.clone(),
                on.file_index,
                on.line,
                on.frame,
            )
        };
        let name = self.scene.nodes[o].name.clone();
        let s = Node {
            seg,
            name,
            kind,
            types,
            role: Role::Shadow { original: o },
            parent: p,
            children: vec![],
            created: 0,
            props: IndexMap::new(),
            tags,
            meta,
            file_index: fi,
            line,
            frame,
            stmt_frame: frame,
            degenerate: false,
            hidden: false,
            late: false,
            local: None,
            lbox: None,
            world: None,
            wbox: None,
            tight: None,
            geom: None,
            curve: None,
            src_stmt: None,
            depth: 0,
            pos: vec![],
            dropped: false,
            concepts: vec![],
            placement_ref: None,
        };
        let sid = self.scene.add(p, s);
        let kids = self.scene.nodes[o].children.clone();
        for c in kids {
            self.shadow(c, sid);
        }
        sid
    }

    /// 同级重名（1.6.6）：后出现的报错并移除
    fn check_sibling_names(&mut self) {
        let n = self.scene.nodes.len();
        for p in 0..n {
            let kids = self.scene.nodes[p].children.clone();
            let mut seen: Vec<String> = Vec::new();
            let mut remove: Vec<NodeId> = Vec::new();
            for c in kids {
                let seg = self.scene.nodes[c].seg.clone();
                if seg.is_empty() {
                    continue;
                }
                if seen.contains(&seg) {
                    self.node_error(c, format!("同级重名 {}", seg));
                    remove.push(c);
                } else {
                    seen.push(seg);
                }
            }
            if !remove.is_empty() {
                self.scene.nodes[p].children.retain(|c| !remove.contains(c));
            }
        }
    }
}

/// 某种类是否接受某属性（1.4.4、1.4.5）
pub fn accepts_prop(kind: &str, key: &str) -> bool {
    let p_all = P_GROUP_ALL.contains(&key);
    let own: &[&str] = match kind {
        "box" | "wedge" => &["size"],
        "plate" => &["points", "h"],
        "hull" => &["points"],
        "mesh" => &["src", "size", "pivot"],
        "cyl" => &["r", "h", "axis"],
        "cone" => &["r", "r2", "h", "axis"],
        "sphere" => &["r"],
        "revolve" => &["profile", "angle", "smooth"],
        "span" => {
            return matches!(
                key,
                "from" | "to" | "drop" | "dir" | "section" | "pin" | "r" | "steps" | "mat" | "color"
            )
        }
        "sweep" => {
            return matches!(
                key,
                "path" | "profile" | "r" | "section" | "pin" | "smooth" | "mat" | "color"
            )
        }
        "pipe" => return matches!(key, "path" | "r" | "smooth" | "mat" | "color"),
        "line" | "bezier" => return key == "path",
        "spline" => return matches!(key, "path" | "closed"),
        "arc" => {
            return matches!(key, "r" | "angle" | "rise" | "plane")
                || (p_all && !matches!(key, "mat" | "color"))
        }
        "helix" => {
            return matches!(key, "r" | "pitch" | "turns") || (p_all && !matches!(key, "mat" | "color"))
        }
        "group" => &["envelope"],
        "union" | "subtract" | "intersect" => &[],
        "repeat" => {
            return matches!(key, "n" | "step" | "turn" | "center" | "along" | "orient")
                || (p_all && !matches!(key, "tilt" | "roll" | "rot" | "aim" | "aimwith" | "turn"))
        }
        "mirror" => &["across", "axis"],
        "rotate" => &["n", "about", "axis", "angle"],
        "mirror_of" => return matches!(key, "of" | "across" | "axis" | "mat" | "color"),
        "rotate_of" => return matches!(key, "of" | "about" | "axis" | "angle" | "mat" | "color"),
        _ => &[],
    };
    own.contains(&key) || p_all
}

/// 属性值的求值上下文
pub fn prop_ctx(pv: &PropVal, timing: Timing) -> Ctx {
    Ctx {
        frame: pv.frame,
        start: pv.start,
        timing,
    }
}
