//! 阶段 4–6（spec/04 4.3、4.5、4.6）：copy 展开、over / drop 修改、保名换形、推迟到阶段 6 的修改。

use crate::ast::{BinOp, Expr, ExprKind, Prop, Stmt, StmtKind, CURVE_KINDS, SOLID_KINDS};
use crate::compile::{Compiler, Ctx, Timing};
use crate::elaborate::{accepts_prop, Modification, PendingCopy, P_GROUP_ALL};
use crate::env::{Binding, FrameId, FrameKind};
use crate::lexer::Seg;
use crate::scene::{props_from, NodeId, PropVal, Role, ROOT};
use crate::value::{RefVal, Value};
use indexmap::IndexMap;
use s3d_geom::Vec3;
use std::rc::Rc;

const RELATION_KEYS: [&str; 5] = ["at", "on", "under", "against", "inside"];
const PLACEMENT_KEYS: [&str; 10] = [
    "at", "on", "under", "against", "inside", "pin", "gap", "offset", "align", "alignto",
];
const CONTAINER_KINDS: [&str; 4] = ["group", "union", "subtract", "intersect"];

enum CopyOutcome {
    Done,
    /// 名字解析失败：留到下一轮（带最后一次的文案）
    Retry(String),
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Atom,
    Container,
    Template,
}

fn is_name_error(msg: &str) -> bool {
    msg.contains("找不到对象") || msg.contains("没有子对象") || msg.contains("不唯一")
}

/// 名字解析失败的那一段（报循环时与待展开 copy 的名字比较）
fn failed_segment(msg: &str) -> Option<String> {
    if let Some(rest) = msg.strip_prefix("找不到对象 ") {
        return Some(
            rest.split(|c: char| c.is_whitespace() || c == '（' || c == ',')
                .next()?
                .to_string(),
        );
    }
    if let Some(i) = msg.find(" 中没有子对象或锚点 ") {
        let tail = &msg[i + " 中没有子对象或锚点 ".len()..];
        return Some(
            tail.split(|c: char| c.is_whitespace() || c == '（')
                .next()?
                .to_string(),
        );
    }
    None
}

/// 选择器的源码形态（报错文案用）
pub fn expr_text(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Number(n) => format!("{}", n),
        ExprKind::Str(s) => format!("\"{}\"", s),
        ExprKind::Bool(b) => format!("{}", b),
        ExprKind::Hash(h) => format!("#{}", h),
        ExprKind::DirWord(w) => w.join("-"),
        ExprKind::Ident(s) => s.clone(),
        ExprKind::PathRef { segs, coords, axis } => {
            let mut s = segs
                .iter()
                .map(|g| match g {
                    Seg::Ident(x) => x.clone(),
                    Seg::Digits(d) => d.clone(),
                    Seg::Star => "*".to_string(),
                })
                .collect::<Vec<_>>()
                .join(".");
            if let Some(cs) = coords {
                s.push('[');
                s.push_str(&cs.iter().map(expr_text).collect::<Vec<_>>().join(","));
                s.push(']');
            }
            if let Some(a) = axis {
                s.push('.');
                s.push(*a);
            }
            s
        }
        ExprKind::Call { name, args } => {
            format!(
                "{}({})",
                name,
                args.iter().map(expr_text).collect::<Vec<_>>().join(", ")
            )
        }
        ExprKind::Tuple(items) => items.iter().map(expr_text).collect::<Vec<_>>().join(","),
        ExprKind::List(items) => format!("[{}]", items.iter().map(expr_text).collect::<Vec<_>>().join("; ")),
        ExprKind::Neg(x) => format!("-{}", expr_text(x)),
        ExprKind::Not(x) => format!("not {}", expr_text(x)),
        ExprKind::Binary { op, lhs, rhs } => {
            let o = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Le => "<=",
                BinOp::Ge => ">=",
                BinOp::Or => "|",
                BinOp::And => "&",
            };
            format!("{} {} {}", expr_text(lhs), o, expr_text(rhs))
        }
        ExprKind::Match { lhs, regex } => format!("{} ~ /{}/", expr_text(lhs), regex),
        ExprKind::Where { lhs, cond } => format!("{} where {}", expr_text(lhs), expr_text(cond)),
    }
}

impl Compiler {
    // ====== 阶段 4：copy（4.3.5）======
    pub fn expand_copies(&mut self) {
        let mut queue: Vec<PendingCopy> = std::mem::take(&mut self.copies);
        queue.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.seq.cmp(&b.seq)));
        let mut guard = 0;
        loop {
            if queue.is_empty() {
                break;
            }
            guard += 1;
            let mut progressed = false;
            let mut next: Vec<(PendingCopy, String)> = Vec::new();
            for c in queue.drain(..) {
                match self.try_expand_copy(&c) {
                    CopyOutcome::Done => progressed = true,
                    CopyOutcome::Retry(msg) => next.push((c, msg)),
                    CopyOutcome::Failed => {}
                }
            }
            // 展开里出现的新 copy（源语句块里的、copy 块里的）
            let added: Vec<PendingCopy> = std::mem::take(&mut self.copies);
            if !progressed || guard > 10000 {
                let pending_names: Vec<String> = next
                    .iter()
                    .map(|(c, _)| match &c.stmt.kind {
                        StmtKind::Copy { name, .. } => name.clone(),
                        _ => String::new(),
                    })
                    .collect();
                for (c, msg) in next {
                    let (fi, line) = (c.stmt.file_index, c.stmt.line);
                    let seg = failed_segment(&msg);
                    let cyclic = seg.map(|s| pending_names.contains(&s)).unwrap_or(false);
                    if cyclic {
                        let name = match &c.stmt.kind {
                            StmtKind::Copy { name, .. } => name.clone(),
                            _ => String::new(),
                        };
                        self.stmt_error(
                            fi,
                            line,
                            format!("copy {} 的原对象无法确定（复制关系形成了循环？）", name),
                        );
                    } else {
                        self.stmt_error(fi, line, msg);
                    }
                }
                break;
            }
            queue = next.into_iter().map(|(c, _)| c).collect();
            queue.extend(added);
            queue.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.seq.cmp(&b.seq)));
        }
    }

    /// 阶段 5 以后的 copy：立即展开，解析失败直接报错（4.3.5 末段）
    pub(crate) fn expand_copy_now(&mut self, c: &PendingCopy) {
        if let CopyOutcome::Retry(msg) = self.try_expand_copy(c) {
            self.stmt_error(c.stmt.file_index, c.stmt.line, msg);
        }
    }

    fn try_expand_copy(&mut self, c: &PendingCopy) -> CopyOutcome {
        let (fi, line) = (c.stmt.file_index, c.stmt.line);
        let props = match &c.stmt.kind {
            StmtKind::Copy { props, .. } => props,
            _ => return CopyOutcome::Failed,
        };
        let of = match props.iter().find(|p| p.key == "of") {
            Some(p) => p,
            None => {
                self.stmt_error(fi, line, "copy 需要 of=原对象");
                return CopyOutcome::Failed;
            }
        };
        if !matches!(
            of.value.kind,
            ExprKind::Ident(_)
                | ExprKind::PathRef {
                    coords: None,
                    axis: None,
                    ..
                }
        ) {
            self.stmt_error(fi, line, "copy 的 of 写一个对象名");
            return CopyOutcome::Failed;
        }
        let ctx = Ctx {
            frame: c.frame,
            start: c.parent,
            timing: Timing::Selector,
        };
        let saved = self.diags.errors.len();
        let res = self.eval(&of.value, ctx);
        self.diags.errors.truncate(saved);
        let a = match res {
            Ok(Value::Ref(r)) if r.anchor.is_none() && r.offset == Vec3::ZERO => r.node,
            Ok(_) => {
                self.stmt_error(fi, line, "copy 的 of 写一个对象名");
                return CopyOutcome::Failed;
            }
            Err(e) => {
                if is_name_error(&e.0) {
                    return CopyOutcome::Retry(e.0);
                }
                self.stmt_error(fi, line, e.0);
                return CopyOutcome::Failed;
            }
        };
        if a == c.parent || self.scene.is_ancestor(a, c.parent) {
            let path = self.scene.path(a);
            self.stmt_error(fi, line, format!("copy 不能复制包含自己的对象 {}", path));
            return CopyOutcome::Failed;
        }
        if self.expand_copy_of(c, a) {
            CopyOutcome::Done
        } else {
            CopyOutcome::Failed
        }
    }

    /// copy 的插入下标：占位 + 此前已插入的同父 copy 数
    fn copy_index(&self, c: &PendingCopy) -> usize {
        let earlier = self
            .copies_inserted
            .iter()
            .filter(|(p, seq)| *p == c.parent && *seq < c.seq)
            .count();
        (c.slot + earlier).min(self.scene.nodes[c.parent].children.len())
    }

    /// 按 4.3.2 / 4.3.3 展开一条 copy；返回是否生成了节点
    fn expand_copy_of(&mut self, c: &PendingCopy, a: NodeId) -> bool {
        let (fi, line) = (c.stmt.file_index, c.stmt.line);
        let (rname, ctags, cprops, cmetas, cblock) = match &c.stmt.kind {
            StmtKind::Copy {
                name,
                tags,
                props,
                metas,
                block,
            } => (
                name.clone(),
                tags.clone(),
                props.clone(),
                metas.clone(),
                block.clone(),
            ),
            _ => return false,
        };
        let sc: Vec<Prop> = cprops.into_iter().filter(|p| p.key != "of").collect();
        let a_role = self.scene.nodes[a].role.clone();
        // —— 源语句 ——
        let (src, env_frame): (Rc<Stmt>, FrameId) = match a_role {
            Role::Instance { .. } => {
                // 阵列实例：合成一个 group，子语句 = 阵列的体，在该实例的帧里展开
                let rep = self.scene.nodes[a].parent;
                let body = match self.scene.nodes[rep].src_stmt.as_ref().map(|s| &s.kind) {
                    Some(StmtKind::Generic { block, .. }) => block.clone().unwrap_or_default(),
                    _ => Vec::new(),
                };
                for p in &sc {
                    if !P_GROUP_ALL.contains(&p.key.as_str()) {
                        self.stmt_error(fi, line, format!("copy 没有属性 {}", p.key));
                        return false;
                    }
                }
                let stmt = Stmt {
                    kind: StmtKind::Generic {
                        kind: "group".to_string(),
                        name: Some(rname.clone()),
                        tags: ctags.clone(),
                        props: vec![],
                        metas: vec![],
                        block: Some(body),
                    },
                    line,
                    file_index: fi,
                };
                (Rc::new(stmt), self.scene.nodes[a].stmt_frame)
            }
            _ => match self.scene.nodes[a].src_stmt.clone() {
                Some(s) => (s, self.scene.nodes[a].stmt_frame),
                None => {
                    self.stmt_error(fi, line, "copy 的 of 写一个对象名");
                    return false;
                }
            },
        };
        let (skind, stags, sprops, smetas, sblock) = match &src.kind {
            StmtKind::Generic {
                kind,
                tags,
                props,
                metas,
                block,
                ..
            } => (
                kind.clone(),
                tags.clone(),
                props.clone(),
                metas.clone(),
                block.clone(),
            ),
            _ => return false,
        };
        let is_instance_src = matches!(a_role, Role::Instance { .. });
        // —— 属性表（4.3.3）——
        let is_template = self.defs.contains_key(&skind);
        let params = if is_template {
            self.template_params(&skind)
        } else {
            Vec::new()
        };
        for p in &sc {
            if !P_GROUP_ALL.contains(&p.key.as_str()) && !params.contains(&p.key) {
                self.stmt_error(fi, line, format!("copy 没有属性 {}", p.key));
                return false;
            }
        }
        let mut table: IndexMap<String, PropVal> = if is_instance_src {
            IndexMap::new()
        } else {
            props_from(&sprops, env_frame, c.parent, src.file_index)
        };
        if sc.iter().any(|p| RELATION_KEYS.contains(&p.key.as_str())) {
            table.retain(|k, _| !PLACEMENT_KEYS.contains(&k.as_str()));
        }
        for p in &sc {
            table.insert(
                p.key.clone(),
                PropVal {
                    expr: p.value.clone(),
                    frame: c.frame,
                    start: c.parent,
                    line: p.line,
                    file_index: fi,
                },
            );
        }
        // 新种类不接受的键：报在 copy 语句上，新节点退化
        let has_of = sprops.iter().any(|p| p.key == "of");
        let kind_check = match (skind.as_str(), has_of) {
            ("mirror", true) => "mirror_of".to_string(),
            ("rotate", true) => "rotate_of".to_string(),
            (k, _) => k.to_string(),
        };
        let mut degenerate = false;
        if !is_template {
            let bad: Vec<String> = table
                .keys()
                .filter(|k| !accepts_prop(&kind_check, k))
                .cloned()
                .collect();
            for k in bad {
                self.stmt_error(fi, line, format!("copy 没有属性 {}", k));
                table.shift_remove(&k);
                degenerate = true;
            }
        }
        // —— 标签并集 ——
        let mut tags = stags.clone();
        for (t, l) in &ctags {
            if !tags.iter().any(|(x, _)| x == t) {
                tags.push((t.clone(), *l));
            }
        }
        let new_stmt = Stmt {
            kind: StmtKind::Generic {
                kind: skind.clone(),
                name: Some(rname.clone()),
                tags,
                props: sprops.clone(),
                metas: if is_instance_src {
                    Vec::new()
                } else {
                    smetas.clone()
                },
                block: sblock.clone(),
            },
            line,
            file_index: fi,
        };
        // —— 展开到占住的位置 ——
        let idx = self.copy_index(c);
        let created_saved = self.scene.nodes[c.parent].created;
        let before = self.scene.nodes[c.parent].children.len();
        self.prop_override = Some(table);
        let id = self.expand_generic(&new_stmt, c.parent, env_frame, c.depth + 1, &c.pos);
        self.prop_override = None;
        self.scene.nodes[c.parent].created = created_saved;
        let id = match id {
            Some(id) => id,
            None => return false,
        };
        if self.scene.nodes[c.parent].children.len() == before + 1 {
            let last = self.scene.nodes[c.parent].children.pop().unwrap();
            self.scene.nodes[c.parent].children.insert(idx, last);
        }
        self.copies_inserted.push((c.parent, c.seq));
        {
            let n = &mut self.scene.nodes[id];
            n.file_index = fi;
            n.line = line;
            n.depth = c.depth + 1;
            n.pos = c.pos.clone();
            n.src_stmt = Some(Rc::new(new_stmt.clone()));
            if degenerate {
                n.degenerate = true;
            }
        }
        // copy 语句自己的元数据覆盖（8.3.5）
        if !cmetas.is_empty() {
            let extra = self.eval_metas(&cmetas, c.frame, &c.stmt);
            for (k, v) in extra {
                self.scene.nodes[id].meta.insert(k, v);
            }
        }
        // —— 块：等于对 R 施加一条 over 块（4.3.4）——
        if let Some(b) = cblock {
            self.apply_block(id, &b, c.frame, &c.pos, c.depth);
        }
        true
    }

    // ====== 块（4.5.5）======
    /// 对成员 m 施加一个块：m 是 repeat 节点时对每个阵列实例各展开一份
    pub(crate) fn apply_block(
        &mut self,
        m: NodeId,
        stmts: &[Stmt],
        parent_frame: FrameId,
        pos: &[u64],
        depth: u32,
    ) {
        let is_repeat =
            self.scene.nodes[m].kind == "repeat" && matches!(self.scene.nodes[m].role, Role::Normal);
        if is_repeat {
            let insts = self.scene.nodes[m].children.clone();
            for inst in insts {
                if matches!(self.scene.nodes[inst].role, Role::Instance { .. }) {
                    self.expand_block_on(inst, stmts, parent_frame, pos, depth);
                }
            }
        } else {
            self.expand_block_on(m, stmts, parent_frame, pos, depth);
        }
    }

    fn expand_block_on(
        &mut self,
        target: NodeId,
        stmts: &[Stmt],
        parent_frame: FrameId,
        pos: &[u64],
        depth: u32,
    ) {
        let bf = self.block_frame(stmts, parent_frame, FrameKind::Block, target);
        self.env.bind(
            bf,
            "it",
            Binding::Value(Value::Ref(RefVal {
                node: target,
                anchor: None,
                offset: Vec3::ZERO,
            })),
        );
        if let Role::Instance { i, j, k, idx } = self.scene.nodes[target].role {
            for (nm, v) in [("i", i), ("j", j), ("k", k), ("idx", idx)] {
                self.env.bind(bf, nm, Binding::Value(Value::Num(v as f64)));
            }
        }
        let mark = self.modifications.len();
        self.expand_block(stmts, target, bf, depth, pos);
        if self.immediate_copies {
            // 阶段 5：块里的 over / drop 紧随这条修改施加（4.1.2 的展开序）
            self.drain_new_mods(mark);
        }
    }

    fn drain_new_mods(&mut self, mark: usize) {
        if self.modifications.len() <= mark {
            return;
        }
        let new: Vec<Modification> = self.modifications.drain(mark..).collect();
        for m in new {
            self.apply_modification(&m, false);
        }
    }

    // ====== 阶段 5：修改序列 ======
    pub fn apply_modifications(&mut self) {
        self.immediate_copies = true;
        let mut queue: Vec<Modification> = std::mem::take(&mut self.modifications);
        queue.sort_by(|a, b| a.pos.cmp(&b.pos));
        for m in queue {
            self.apply_modification(&m, false);
            // 施加过程中新出现而未被就地处理的项（理论上为空）
            let rest: Vec<Modification> = std::mem::take(&mut self.modifications);
            for r in rest {
                self.apply_modification(&r, false);
            }
        }
    }

    /// 阶段 6：推迟的修改（4.4.4）——影子已生成，解析失败与空选择在这里报
    pub fn apply_deferred_modifications(&mut self) {
        let deferred: Vec<Modification> = std::mem::take(&mut self.deferred_mods);
        for m in deferred {
            self.apply_modification(&m, true);
        }
        let rest: Vec<Modification> = std::mem::take(&mut self.modifications);
        for r in rest {
            self.apply_modification(&r, true);
        }
    }

    fn apply_modification(&mut self, m: &Modification, final_stage: bool) {
        let (fi, line) = (m.stmt.file_index, m.stmt.line);
        let selector: &Expr = match &m.stmt.kind {
            StmtKind::Over { selector, .. } | StmtKind::Drop { selector, .. } => selector,
            _ => return,
        };
        let ctx = Ctx {
            frame: m.frame,
            start: m.start,
            timing: Timing::Selector,
        };
        let saved = self.diags.errors.len();
        let res = self.eval(selector, ctx).and_then(|v| self.to_set(v));
        self.diags.errors.truncate(saved);
        let set = match res {
            Ok(s) => s,
            Err(e) => {
                if is_name_error(&e.0) && !final_stage {
                    self.deferred_mods.push(m.clone());
                } else {
                    self.stmt_error(fi, line, e.0);
                }
                return;
            }
        };
        // 作用范围、已删除、皮肤
        let members: Vec<NodeId> = set
            .into_iter()
            .filter(|&n| {
                n != ROOT
                    && !self.scene.nodes[n].dropped
                    && (m.scope == ROOT || self.scene.is_ancestor(m.scope, n))
                    && !self.scene.in_skin(n)
            })
            .collect();
        if members.is_empty() {
            if final_stage {
                self.stmt_error(fi, line, format!("{} 没有选中任何对象", expr_text(selector)));
            } else {
                self.deferred_mods.push(m.clone());
            }
            return;
        }
        let idx = self.scene.order_index();
        let mut members = members;
        members.sort_by_key(|n| idx[*n]);
        members.dedup();
        match &m.stmt.kind {
            StmtKind::Drop { .. } => {
                for n in members {
                    if !self.scene.nodes[n].dropped {
                        self.scene.detach(n);
                    }
                }
            }
            StmtKind::Over {
                retype, props, block, ..
            } => {
                if let Some(t) = retype {
                    let t = t.as_str();
                    if matches!(t, "repeat" | "mirror" | "rotate" | "copy") || CURVE_KINDS.contains(&t) {
                        self.stmt_error(fi, line, format!("不能把物体换成 {}", t));
                        return;
                    }
                    if !(SOLID_KINDS.contains(&t)
                        || CONTAINER_KINDS.contains(&t)
                        || self.defs.contains_key(t))
                    {
                        self.stmt_error(fi, line, format!("over … : {}：没有这个类型或形状种类", t));
                        return;
                    }
                }
                for n in members {
                    if self.scene.nodes[n].dropped {
                        continue;
                    }
                    self.modify_member(m, n, retype.as_deref(), props, block.as_deref());
                }
            }
            _ => {}
        }
    }

    fn mod_prop(&self, m: &Modification, n: NodeId, p: &Prop) -> PropVal {
        PropVal {
            expr: p.value.clone(),
            frame: m.frame,
            start: self.scene.nodes[n].parent,
            line: p.line,
            file_index: m.stmt.file_index,
        }
    }

    /// 就地写入属性（4.5.3）：定位关系键先删掉原有的五个关系键
    fn write_inplace(&mut self, m: &Modification, n: NodeId, props: &[&Prop]) {
        for p in props {
            if RELATION_KEYS.contains(&p.key.as_str()) {
                for k in RELATION_KEYS {
                    self.scene.nodes[n].props.shift_remove(k);
                }
            }
            let pv = self.mod_prop(m, n, p);
            self.scene.nodes[n].props.insert(p.key.clone(), pv);
        }
    }

    fn modify_member(
        &mut self,
        m: &Modification,
        n: NodeId,
        retype: Option<&str>,
        props: &[Prop],
        block: Option<&[Stmt]>,
    ) {
        let (fi, line) = (m.stmt.file_index, m.stmt.line);
        let path = self.scene.path(n);
        let role = self.scene.nodes[n].role.clone();
        let kind = self.scene.nodes[n].kind.clone();
        let depth = self.scene.nodes[n].depth;
        match role {
            Role::Instance { .. } | Role::TwinGroup { .. } => {
                if !props.is_empty() || retype.is_some() {
                    let what = if matches!(role, Role::Instance { .. }) {
                        "阵列实例"
                    } else {
                        "副本组"
                    };
                    self.stmt_error(fi, line, format!("{} 是{}，不接受属性", path, what));
                }
                if let Some(b) = block {
                    self.apply_block(n, b, m.frame, &m.pos, depth);
                }
                return;
            }
            Role::Skin => return,
            Role::Shadow { .. } => {
                // 影子覆盖（4.4.4）：只允许就地修改
                if retype.is_some() {
                    self.stmt_error(fi, line, format!("{} 是副本，不能单独换类型", path));
                    return;
                }
                let is_template = self.defs.contains_key(&kind);
                let params = if is_template {
                    self.template_params(&kind)
                } else {
                    Vec::new()
                };
                let mut inplace: Vec<&Prop> = Vec::new();
                for p in props {
                    let k = p.key.as_str();
                    if params.contains(&p.key) || (kind == "repeat" && k == "n") {
                        self.stmt_error(fi, line, format!("{} 是副本，不能改模板参数或阵列数量", path));
                        return;
                    }
                    if RELATION_KEYS.contains(&k) {
                        self.stmt_error(fi, line, format!("{} 是副本，不能单独定位", path));
                        return;
                    }
                    let ok = if is_template {
                        P_GROUP_ALL.contains(&k)
                    } else {
                        accepts_prop(&kind, k)
                    };
                    if !ok {
                        self.stmt_error(fi, line, format!("{} 没有属性 {}", path, k));
                        return;
                    }
                    inplace.push(p);
                }
                self.write_inplace(m, n, &inplace);
                if let Some(b) = block {
                    self.apply_block(n, b, m.frame, &m.pos, depth);
                }
                return;
            }
            _ => {}
        }
        if let Some(t) = retype {
            self.retype_member(m, n, t, props, block);
            return;
        }
        let is_template = self.defs.contains_key(&kind);
        let params = if is_template {
            self.template_params(&kind)
        } else {
            Vec::new()
        };
        let kind_check = match (&role, kind.as_str()) {
            (Role::TwinOf { .. }, "mirror") => "mirror_of".to_string(),
            (Role::TwinOf { .. }, "rotate") => "rotate_of".to_string(),
            _ => kind.clone(),
        };
        let mut rebuild: Vec<&Prop> = Vec::new();
        let mut inplace: Vec<&Prop> = Vec::new();
        for p in props {
            let k = p.key.as_str();
            if (is_template && params.contains(&p.key)) || (kind == "repeat" && k == "n") {
                rebuild.push(p);
            } else {
                let ok = if is_template {
                    P_GROUP_ALL.contains(&k)
                } else {
                    accepts_prop(&kind_check, k)
                };
                if !ok {
                    self.stmt_error(fi, line, format!("{} 没有属性 {}", path, k));
                    return;
                }
                inplace.push(p);
            }
        }
        if matches!(role, Role::TwinOf { .. }) && !rebuild.is_empty() {
            self.stmt_error(fi, line, format!("{} 是副本，不能改模板参数或阵列数量", path));
            return;
        }
        let mut cur = n;
        if !rebuild.is_empty() {
            let mut table = self.scene.nodes[n].props.clone();
            for p in &rebuild {
                let pv = self.mod_prop(m, n, p);
                table.insert(p.key.clone(), pv);
            }
            let src = match self.scene.nodes[n].src_stmt.clone() {
                Some(s) => (*s).clone(),
                None => return,
            };
            match self.rebuild_node(n, src, table) {
                Some(id) => cur = id,
                None => return,
            }
        }
        self.write_inplace(m, cur, &inplace);
        if let Some(b) = block {
            self.apply_block(cur, b, m.frame, &m.pos, depth);
        }
    }

    /// 重建（4.5.4 第 2–4 步）：同一位置、同一名字按新语句重新展开；体内的 over / drop 立即施加
    fn rebuild_node(&mut self, n: NodeId, stmt: Stmt, table: IndexMap<String, PropVal>) -> Option<NodeId> {
        let p = self.scene.nodes[n].parent;
        let idx = self.scene.nodes[p].children.iter().position(|c| *c == n)?;
        let old_seg = self.scene.nodes[n].seg.clone();
        let anonymous = self.scene.nodes[n].name.is_empty();
        let depth = self.scene.nodes[n].depth;
        let pos = self.scene.nodes[n].pos.clone();
        let frame = self.scene.nodes[n].stmt_frame;
        self.scene.detach(n);
        let created_saved = self.scene.nodes[p].created;
        let before = self.scene.nodes[p].children.len();
        let mark = self.modifications.len();
        self.prop_override = Some(table);
        let id = self.expand_generic(&stmt, p, frame, depth, &pos);
        self.prop_override = None;
        self.scene.nodes[p].created = created_saved;
        let id = id?;
        if self.scene.nodes[p].children.len() == before + 1 {
            let last = self.scene.nodes[p].children.pop().unwrap();
            self.scene.nodes[p].children.insert(idx.min(before), last);
        }
        if anonymous {
            if let Some(i) = old_seg.rsplit('#').next() {
                let k = self.scene.nodes[id].kind.clone();
                self.scene.nodes[id].seg = format!("{}#{}", k, i);
            }
        }
        self.scene.nodes[id].src_stmt = Some(Rc::new(stmt));
        self.drain_new_mods(mark);
        Some(id)
    }

    // ====== 保名换形（4.6）======
    fn retype_member(
        &mut self,
        m: &Modification,
        n: NodeId,
        t: &str,
        props: &[Prop],
        block: Option<&[Stmt]>,
    ) {
        let (fi, line) = (m.stmt.file_index, m.stmt.line);
        let path = self.scene.path(n);
        let kind = self.scene.nodes[n].kind.clone();
        let role = self.scene.nodes[n].role.clone();
        let depth = self.scene.nodes[n].depth;
        let origin = if matches!(role, Role::TwinOf { .. }) {
            self.stmt_error(fi, line, format!("{} 是副本，不能单独换类型", path));
            return;
        } else if SOLID_KINDS.contains(&kind.as_str()) || CURVE_KINDS.contains(&kind.as_str()) {
            Class::Atom
        } else if CONTAINER_KINDS.contains(&kind.as_str()) {
            Class::Container
        } else if self.defs.contains_key(&kind) {
            Class::Template
        } else if kind == "repeat" {
            self.stmt_error(fi, line, "不能对阵列 / 阵列实例换类型：改 n 或 drop 后重写");
            return;
        } else {
            self.stmt_error(fi, line, "不能对 mirror / rotate 容器换类型");
            return;
        };
        let target = if SOLID_KINDS.contains(&t) {
            Class::Atom
        } else if CONTAINER_KINDS.contains(&t) {
            Class::Container
        } else {
            Class::Template
        };
        if origin == Class::Container && target == Class::Atom {
            self.stmt_error(
                fi,
                line,
                format!("不能把容器 {} 换成 {}：它的子物体会丢失，先 drop 再新建", path, t),
            );
            return;
        }
        // 保留表（4.6.2）
        let eff = self.scene.nodes[n].props.clone();
        let t_params = if target == Class::Template {
            self.template_params(t)
        } else {
            Vec::new()
        };
        let o_params = if origin == Class::Template {
            self.template_params(&kind)
        } else {
            Vec::new()
        };
        let accepts = |k: &str| -> bool {
            if target == Class::Template {
                P_GROUP_ALL.contains(&k)
            } else {
                accepts_prop(t, k)
            }
        };
        let related = origin == Class::Template
            && target == Class::Template
            && (self
                .derivation_chain(&kind)
                .map(|c| c.iter().any(|d| d.name == t))
                .unwrap_or(false)
                || self
                    .derivation_chain(t)
                    .map(|c| c.iter().any(|d| d.name == kind))
                    .unwrap_or(false));
        let mut table: IndexMap<String, PropVal> = IndexMap::new();
        for (k, pv) in eff {
            if P_GROUP_ALL.contains(&k.as_str()) {
                if accepts(&k) {
                    table.insert(k, pv);
                }
            } else if origin == Class::Template && o_params.contains(&k) && target == Class::Template {
                if t_params.contains(&k) {
                    table.insert(k, pv);
                } else if !related {
                    self.stmt_error(fi, line, format!("模板 {} 没有参数 {}（换形自 {}）", t, k, kind));
                    return;
                }
            }
            // 专有属性一律不带过去
        }
        for p in props {
            let k = p.key.as_str();
            let ok = if target == Class::Template {
                P_GROUP_ALL.contains(&k) || t_params.contains(&p.key)
            } else {
                accepts_prop(t, k)
            };
            if !ok {
                self.stmt_error(fi, line, format!("{} 没有属性 {}", t, k));
                return;
            }
        }
        for p in props {
            if RELATION_KEYS.contains(&p.key.as_str()) {
                for rk in RELATION_KEYS {
                    table.shift_remove(rk);
                }
            }
            let pv = self.mod_prop(m, n, p);
            table.insert(p.key.clone(), pv);
        }
        let (name, tags, metas, sline, sfile) = {
            let node = &self.scene.nodes[n];
            let metas = match node.src_stmt.as_ref().map(|s| &s.kind) {
                Some(StmtKind::Generic { metas, .. }) => metas.clone(),
                _ => Vec::new(),
            };
            let (sline, sfile) = node
                .src_stmt
                .as_ref()
                .map(|s| (s.line, s.file_index))
                .unwrap_or((node.line, node.file_index));
            let name = if node.name.is_empty() {
                None
            } else {
                Some(node.name.clone())
            };
            let tags: Vec<(String, u32)> = node.tags.iter().map(|t| (t.clone(), sline)).collect();
            (name, tags, metas, sline, sfile)
        };
        let new_stmt = Stmt {
            kind: StmtKind::Generic {
                kind: t.to_string(),
                name,
                tags,
                props: vec![],
                metas,
                block: None,
            },
            line: sline,
            file_index: sfile,
        };
        if let Some(id) = self.rebuild_node(n, new_stmt, table) {
            if let Some(b) = block {
                self.apply_block(id, b, m.frame, &m.pos, depth);
            }
        }
    }
}
