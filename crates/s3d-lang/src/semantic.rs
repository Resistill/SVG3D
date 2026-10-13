//! 语义层（spec/07）：语义名字空间、concept / pattern / check、style 与 variant 的选择集、变体挂载（4.7）。

use crate::ast::{ConceptRhs, Constraint, ConstraintLine, Expr, ExprKind, Prop, Span, Stmt, StmtKind};
use crate::compile::{Compiler, Ctx, Timing};
use crate::elaborate::P_GROUP_ALL;
use crate::env::{Binding, FrameId, FrameKind};
use crate::eval::{EResult, EvalErr};
use crate::lexer::Seg;
use crate::material::parse_color;
use crate::modify::expr_text;
use crate::scene::{NodeId, PropVal, Role, ROOT};
use crate::value::{RefVal, Value};
use indexmap::IndexMap;
use s3d_geom::Vec3;
use serde_json::{json, Map, Value as J};

const STEP_MAX: usize = 200_000;
const MATCH_MAX: usize = 500;

#[derive(Clone, Debug)]
pub struct ConceptDef {
    pub name: String,
    pub parent: Option<String>,
    pub rhs: ConceptRhs,
    pub src: String,
    pub stmt: Stmt,
    pub metas: Vec<Prop>,
}

#[derive(Clone, Debug)]
pub struct PatternDef {
    pub name: String,
    pub roles: Vec<(String, bool)>,
    pub constraints: Vec<ConstraintLine>,
    pub stmt: Stmt,
    pub metas: Vec<Prop>,
}

/// 一个匹配：角色 → 节点（标量）或节点列表（集合）
#[derive(Clone, Debug)]
pub enum RoleVal {
    One(NodeId),
    Many(Vec<NodeId>),
}

#[derive(Clone, Debug, Default)]
pub struct PatternResult {
    pub matches: Vec<IndexMap<String, RoleVal>>,
    pub steps: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug)]
pub struct VariantEntry {
    pub stmt: Stmt,
    pub use_t: String,
    pub src: String,
    pub members: Vec<NodeId>,
    pub error: Option<String>,
    pub valid: bool,
    /// 写明的参数与 mat / color
    pub props: Vec<Prop>,
}

#[derive(Clone, Debug)]
pub struct CheckEntry {
    pub stmt: Stmt,
    pub message: String,
    pub ok: bool,
    pub violators: Vec<NodeId>,
    pub error: Option<String>,
}

/// 语义层的全部状态
#[derive(Default)]
pub struct Semantics {
    pub frame: Option<FrameId>,
    pub concepts: IndexMap<String, ConceptDef>,
    pub patterns: IndexMap<String, PatternDef>,
    pub concept_values: IndexMap<String, Vec<NodeId>>,
    pub concept_errors: IndexMap<String, String>,
    pub pattern_values: IndexMap<String, PatternResult>,
    pub pattern_errors: IndexMap<String, String>,
    /// 正在求值的名字栈（7.4.3）
    pub stack: Vec<String>,
    pub checks: Vec<CheckEntry>,
    pub variants: Vec<VariantEntry>,
    pub done: bool,
}

fn role_set(pr: &PatternResult, role: Option<&str>) -> Vec<NodeId> {
    let mut out = Vec::new();
    for m in &pr.matches {
        for (r, v) in m {
            if let Some(want) = role {
                if r != want {
                    continue;
                }
            }
            match v {
                RoleVal::One(n) => out.push(*n),
                RoleVal::Many(ns) => out.extend(ns.iter().copied()),
            }
        }
    }
    out
}

/// 表达式树里出现的角色名（7.5.2 的 deps）
fn collect_roles(e: &Expr, roles: &[(String, bool)], out: &mut Vec<String>) {
    let is_role = |s: &str| roles.iter().any(|(r, _)| r == s);
    match &e.kind {
        ExprKind::Ident(s) => {
            if is_role(s) && !out.contains(s) {
                out.push(s.clone());
            }
        }
        ExprKind::PathRef { segs, coords, .. } => {
            if let Some(Seg::Ident(s)) = segs.first() {
                if is_role(s) && !out.contains(s) {
                    out.push(s.clone());
                }
            }
            if let Some(cs) = coords {
                for c in cs {
                    collect_roles(c, roles, out);
                }
            }
        }
        ExprKind::Call { args, .. } => args.iter().for_each(|a| collect_roles(a, roles, out)),
        ExprKind::Tuple(items) | ExprKind::List(items) => {
            items.iter().for_each(|a| collect_roles(a, roles, out))
        }
        ExprKind::Neg(x) | ExprKind::Not(x) => collect_roles(x, roles, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_roles(lhs, roles, out);
            collect_roles(rhs, roles, out);
        }
        ExprKind::Match { lhs, .. } => collect_roles(lhs, roles, out),
        ExprKind::Where { lhs, cond } => {
            collect_roles(lhs, roles, out);
            collect_roles(cond, roles, out);
        }
        _ => {}
    }
}

fn is_bare_role(e: &Expr, role: &str) -> bool {
    match &e.kind {
        ExprKind::Ident(s) => s == role,
        ExprKind::PathRef { segs, coords, axis } => {
            segs.len() == 1
                && coords.is_none()
                && axis.is_none()
                && matches!(&segs[0], Seg::Ident(s) if s == role)
        }
        _ => false,
    }
}

impl Compiler {
    fn sem_ctx(&self) -> Ctx {
        Ctx {
            frame: self.sem.frame.unwrap_or(self.global_frame),
            start: ROOT,
            timing: Timing::Appearance,
        }
    }

    /// 阶段一（7.1.2 第 4–8 步）
    pub fn run_semantics(&mut self) {
        if self.sem.done {
            return;
        }
        self.sem.done = true;
        self.derive_facts();
        self.build_material_table();
        let stmts: Vec<Stmt> = std::mem::take(&mut self.semantic_stmts);
        if stmts.is_empty() {
            return;
        }
        // 语义帧：父帧是全局帧
        let gf = self.global_frame;
        let sf = self.env.new_frame(FrameKind::Semantic, Some(gf));
        self.sem.frame = Some(sf);
        // 第 4 步：名字空间折叠与静态检查
        self.fold_namespace(&stmts);
        let names: Vec<String> = self
            .sem
            .concepts
            .keys()
            .chain(self.sem.patterns.keys())
            .cloned()
            .collect();
        for n in names {
            self.env
                .bind(sf, &n, Binding::Value(Value::Str(format!("__sem__{}", n))));
        }
        // 第 5 步：强制求值（模式先、概念后，按 P 的顺序）
        let pats: Vec<String> = self.sem.patterns.keys().cloned().collect();
        for p in pats {
            let _ = self.semantic_value(&p);
        }
        let cons: Vec<String> = self.sem.concepts.keys().cloned().collect();
        for c in cons {
            let _ = self.semantic_value(&c);
        }
        // 节点标注（7.4.4）
        self.annotate_concepts();
        // 第 6–8 步：check、style、variant 按 P 的顺序
        for s in &stmts {
            match &s.kind {
                StmtKind::Check { .. } => self.run_check(s),
                StmtKind::Style { selector, props, .. } => self.run_style(s, selector, props, false),
                StmtKind::Variant {
                    selector, src, props, ..
                } => self.run_variant(s, selector, src, props, false),
                StmtKind::Over { selector, props, .. } => {
                    if props.iter().any(|p| p.key == "use") {
                        let src = expr_text(selector);
                        self.run_variant(s, selector, &src, props, true);
                    } else {
                        self.run_style(s, selector, props, true);
                    }
                }
                _ => {}
            }
        }
        // 阶段二第 1–2 步：挂载并求解皮肤
        self.mount_variants();
    }

    fn fold_namespace(&mut self, stmts: &[Stmt]) {
        let let_names: Vec<String> = self
            .prog
            .stmts
            .iter()
            .filter_map(|s| match &s.kind {
                StmtKind::Let { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        let mut kinds: IndexMap<String, &str> = IndexMap::new();
        for s in stmts {
            match &s.kind {
                StmtKind::Concept {
                    name,
                    parent,
                    rhs,
                    src,
                    metas,
                } => {
                    if let_names.contains(name) {
                        self.stmt_error(s.file_index, s.line, format!("名字 {} 已经用作 let", name));
                        continue;
                    }
                    if kinds.get(name) == Some(&"pattern") {
                        self.stmt_error(s.file_index, s.line, format!("名字 {} 已经用作 pattern", name));
                        continue;
                    }
                    if let Some(u) = parent {
                        let is_concept = stmts
                            .iter()
                            .any(|t| matches!(&t.kind, StmtKind::Concept { name: n2, .. } if n2 == u));
                        if !is_concept {
                            self.stmt_error(s.file_index, s.line, format!("上位概念 {} 没有定义", u));
                            continue;
                        }
                    }
                    kinds.insert(name.clone(), "concept");
                    self.sem.concepts.insert(
                        name.clone(),
                        ConceptDef {
                            name: name.clone(),
                            parent: parent.clone(),
                            rhs: rhs.clone(),
                            src: src.clone(),
                            stmt: s.clone(),
                            metas: metas.clone(),
                        },
                    );
                }
                StmtKind::Pattern {
                    name,
                    roles,
                    metas,
                    constraints,
                } => {
                    if let_names.contains(name) {
                        self.stmt_error(s.file_index, s.line, format!("名字 {} 已经用作 let", name));
                        continue;
                    }
                    if kinds.get(name) == Some(&"concept") {
                        self.stmt_error(s.file_index, s.line, format!("名字 {} 已经用作 concept", name));
                        continue;
                    }
                    if let Err(msg) = Self::pattern_static_check(roles, constraints) {
                        self.stmt_error(s.file_index, s.line, msg);
                        continue;
                    }
                    kinds.insert(name.clone(), "pattern");
                    self.sem.patterns.insert(
                        name.clone(),
                        PatternDef {
                            name: name.clone(),
                            roles: roles.clone(),
                            constraints: constraints.clone(),
                            stmt: s.clone(),
                            metas: metas.clone(),
                        },
                    );
                }
                _ => {}
            }
        }
    }

    /// 7.5.1、7.5.2 的静态检查
    fn pattern_static_check(roles: &[(String, bool)], constraints: &[ConstraintLine]) -> Result<(), String> {
        let mut seen_in: Vec<String> = Vec::new();
        let scalar: Vec<&String> = roles.iter().filter(|(_, set)| !*set).map(|(r, _)| r).collect();
        for c in constraints {
            match &c.constraint {
                Constraint::RoleIn { role, expr } => {
                    if seen_in.contains(role) {
                        return Err(format!("角色 {} 的 in 约束重复", role));
                    }
                    seen_in.push(role.clone());
                    let mut deps = Vec::new();
                    collect_roles(expr, roles, &mut deps);
                    let is_set = roles.iter().any(|(r, s)| r == role && *s);
                    for d in deps {
                        let d_is_set = roles.iter().any(|(r, s)| *r == d && *s);
                        let ok = if is_set {
                            !d_is_set
                        } else {
                            let my = scalar.iter().position(|r| **r == *role).unwrap_or(0);
                            let dp = scalar.iter().position(|r| **r == d);
                            matches!(dp, Some(x) if x < my)
                        };
                        if !ok {
                            return Err(format!(
                                "角色 {} 的 in 不能引用 {}（只能引用更早声明的标量角色）",
                                role, d
                            ));
                        }
                    }
                }
                Constraint::Relation { lhs, rhs, .. } => {
                    let mut deps = Vec::new();
                    collect_roles(lhs, roles, &mut deps);
                    collect_roles(rhs, roles, &mut deps);
                    let sets: Vec<&String> = deps
                        .iter()
                        .filter(|d| roles.iter().any(|(r, s)| r == *d && *s))
                        .collect();
                    if sets.len() > 1
                        || (sets.len() == 1 && !(is_bare_role(lhs, sets[0]) || is_bare_role(rhs, sets[0])))
                    {
                        return Err("一条关系约束里最多一个集合角色，且它要单独站在关系的一边".to_string());
                    }
                }
                Constraint::Boolean(_) => {}
            }
        }
        Ok(())
    }

    /// 读语义名（7.4.3）：缓存成功值；失败不缓存；栈上的名字报循环
    pub fn semantic_value(&mut self, name: &str) -> EResult<Value> {
        if let Some(v) = self.sem.concept_values.get(name) {
            return Ok(Value::Set(v.clone()));
        }
        if self.sem.pattern_values.contains_key(name) {
            return Ok(Value::Pattern(name.to_string()));
        }
        if self.sem.stack.iter().any(|x| x == name) {
            return Err(EvalErr(format!("{} 循环引用了自己", name)));
        }
        if let Some(def) = self.sem.concepts.get(name).cloned() {
            self.sem.stack.push(name.to_string());
            let r = self.eval_concept(&def);
            self.sem.stack.pop();
            return match r {
                Ok(set) => {
                    self.sem.concept_values.insert(name.to_string(), set.clone());
                    Ok(Value::Set(set))
                }
                Err(e) => {
                    if !self.sem.concept_errors.contains_key(name) {
                        self.sem.concept_errors.insert(name.to_string(), e.0.clone());
                        self.stmt_error(def.stmt.file_index, def.stmt.line, e.0.clone());
                    }
                    Err(e)
                }
            };
        }
        if let Some(def) = self.sem.patterns.get(name).cloned() {
            self.sem.stack.push(name.to_string());
            let r = self.match_pattern(&def);
            self.sem.stack.pop();
            return match r {
                Ok(res) => {
                    if res.truncated {
                        let file = self.file_display(def.stmt.file_index);
                        self.diags.warning(
                            def.stmt.file_index,
                            &file,
                            def.stmt.line,
                            None,
                            format!(
                                "模式 {} 的匹配超过 {} 个，只保留前 {} 个",
                                name, MATCH_MAX, MATCH_MAX
                            ),
                        );
                    }
                    self.sem.pattern_values.insert(name.to_string(), res);
                    Ok(Value::Pattern(name.to_string()))
                }
                Err(e) => {
                    if !self.sem.pattern_errors.contains_key(name) {
                        self.sem.pattern_errors.insert(name.to_string(), e.0.clone());
                        self.stmt_error(def.stmt.file_index, def.stmt.line, e.0.clone());
                    }
                    Err(e)
                }
            };
        }
        Err(EvalErr(format!("找不到对象 {}", name)))
    }

    fn eval_concept(&mut self, def: &ConceptDef) -> EResult<Vec<NodeId>> {
        let ctx = self.sem_ctx();
        let v = match (&def.parent, &def.rhs) {
            (Some(u), ConceptRhs::Where(cond)) => {
                // U where P
                let uv = self.semantic_value(u)?;
                let set = self.to_set(uv)?;
                let mut out = Vec::new();
                for m in set {
                    let wf = self.where_frame_pub(m, ctx.frame);
                    let c = Ctx { frame: wf, ..ctx };
                    match self.eval(cond, c)? {
                        Value::Bool(true) => out.push(m),
                        Value::Bool(false) | Value::Missing => {}
                        other => {
                            return Err(EvalErr(format!(
                                "where 的条件需要真假值，得到 {}",
                                other.describe()
                            )))
                        }
                    }
                }
                Value::Set(out)
            }
            (Some(u), ConceptRhs::Expr(e)) => {
                let uv = self.semantic_value(u)?;
                let a = self.to_set(uv)?;
                let ev = self.eval(e, ctx)?;
                if !Self::is_setlike_pub(&ev) {
                    return Err(EvalErr(format!(
                        "concept {} 的值需要集合，得到 {}",
                        def.name,
                        ev.describe()
                    )));
                }
                let b = self.to_set(ev)?;
                Value::Set(a.into_iter().filter(|x| b.contains(x)).collect())
            }
            (None, ConceptRhs::Expr(e)) | (None, ConceptRhs::Where(e)) => {
                let ev = self.eval(e, ctx)?;
                if !Self::is_setlike_pub(&ev) {
                    return Err(EvalErr(format!(
                        "concept {} 的值需要集合，得到 {}",
                        def.name,
                        ev.describe()
                    )));
                }
                ev
            }
        };
        let set = self.to_set(v)?;
        Ok(self.sorted_nodes(set))
    }

    // ====== pattern（7.5）======
    fn match_pattern(&mut self, def: &PatternDef) -> EResult<PatternResult> {
        let scalars: Vec<String> = def
            .roles
            .iter()
            .filter(|(_, s)| !*s)
            .map(|(r, _)| r.clone())
            .collect();
        let sets: Vec<String> = def
            .roles
            .iter()
            .filter(|(_, s)| *s)
            .map(|(r, _)| r.clone())
            .collect();
        // 约束分类
        let mut domains: IndexMap<String, Expr> = IndexMap::new();
        let mut by_depth: Vec<Vec<&ConstraintLine>> = vec![Vec::new(); scalars.len() + 1];
        let mut rules: IndexMap<String, Vec<&ConstraintLine>> = IndexMap::new();
        let mut finals: Vec<&ConstraintLine> = Vec::new();
        for s in &sets {
            rules.insert(s.clone(), Vec::new());
        }
        for c in &def.constraints {
            match &c.constraint {
                Constraint::RoleIn { role, expr } => {
                    domains.insert(role.clone(), expr.clone());
                }
                Constraint::Relation { lhs, rhs, .. } => {
                    let mut deps = Vec::new();
                    collect_roles(lhs, &def.roles, &mut deps);
                    collect_roles(rhs, &def.roles, &mut deps);
                    let set_dep = deps.iter().find(|d| sets.contains(d)).cloned();
                    if let Some(t) = set_dep {
                        rules.get_mut(&t).unwrap().push(c);
                    } else {
                        let depth = deps
                            .iter()
                            .filter_map(|d| scalars.iter().position(|r| r == d))
                            .map(|i| i + 1)
                            .max()
                            .unwrap_or(0);
                        if depth == 0 {
                            finals.push(c);
                        } else {
                            by_depth[depth].push(c);
                        }
                    }
                }
                Constraint::Boolean(e) => {
                    let mut deps = Vec::new();
                    collect_roles(e, &def.roles, &mut deps);
                    if deps.iter().any(|d| sets.contains(d)) {
                        finals.push(c);
                    } else {
                        let depth = deps
                            .iter()
                            .filter_map(|d| scalars.iter().position(|r| r == d))
                            .map(|i| i + 1)
                            .max()
                            .unwrap_or(0);
                        if depth == 0 {
                            finals.push(c);
                        } else {
                            by_depth[depth].push(c);
                        }
                    }
                }
            }
        }
        let mut res = PatternResult::default();
        let mut keys: Vec<String> = Vec::new();
        let mut beta: IndexMap<String, RoleVal> = IndexMap::new();
        self.dfs(
            def, &scalars, &sets, &domains, &by_depth, &rules, &finals, 0, &mut beta, &mut res, &mut keys,
        )?;
        Ok(res)
    }

    /// 角色绑定帧
    fn role_frame(&mut self, beta: &IndexMap<String, RoleVal>) -> FrameId {
        let parent = self.sem.frame.unwrap_or(self.global_frame);
        let f = self.env.new_frame(FrameKind::Block, Some(parent));
        for (r, v) in beta {
            let val = match v {
                RoleVal::One(n) => Value::Ref(RefVal {
                    node: *n,
                    anchor: None,
                    offset: Vec3::ZERO,
                }),
                RoleVal::Many(ns) => Value::Set(ns.clone()),
            };
            self.env.bind(f, r, Binding::Value(val));
        }
        f
    }

    fn eval_domain(
        &mut self,
        def: &PatternDef,
        role: &str,
        domains: &IndexMap<String, Expr>,
        beta: &IndexMap<String, RoleVal>,
    ) -> EResult<Vec<NodeId>> {
        match domains.get(role) {
            None => {
                let all: Vec<NodeId> = self
                    .scene
                    .order()
                    .into_iter()
                    .filter(|&n| !self.scene.in_skin(n))
                    .collect();
                Ok(all)
            }
            Some(e) => {
                let f = self.role_frame(beta);
                let ctx = Ctx {
                    frame: f,
                    ..self.sem_ctx()
                };
                let v = self.eval(e, ctx)?;
                if !Self::is_setlike_pub(&v) {
                    return Err(EvalErr(format!(
                        "角色 {} 的 in 需要集合，得到 {}",
                        role,
                        v.describe()
                    )));
                }
                let _ = def;
                let s = self.to_set(v)?;
                Ok(self.sorted_nodes(s))
            }
        }
    }

    /// 一条非域约束在绑定下是否成立
    fn check_constraint(
        &mut self,
        c: &ConstraintLine,
        def: &PatternDef,
        beta: &IndexMap<String, RoleVal>,
    ) -> EResult<bool> {
        let f = self.role_frame(beta);
        let ctx = Ctx {
            frame: f,
            ..self.sem_ctx()
        };
        match &c.constraint {
            Constraint::Boolean(e) => match self.eval(e, ctx)? {
                Value::Bool(b) => Ok(b),
                Value::Missing => Ok(false),
                other => Err(EvalErr(format!(
                    "模式 {} 的约束需要真假值，得到 {}",
                    def.name,
                    other.describe()
                ))),
            },
            Constraint::Relation { lhs, rel, rhs } => {
                let lv = self.eval(lhs, ctx)?;
                let rv = self.eval(rhs, ctx)?;
                let xs = self.to_set(lv)?;
                let ys = self.to_set(rv)?;
                let ri = crate::facts::REL_NAMES
                    .iter()
                    .position(|r| *r == rel.name())
                    .unwrap();
                self.derive_facts();
                if self.facts_skipped {
                    self.pending_facts_warning = Some(format!("空间事实未推导，{}() 为空集", rel.name()));
                }
                for x in &xs {
                    for y in &ys {
                        if self.rel_index[ri][*x].contains(y) {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            Constraint::RoleIn { .. } => Ok(true),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        &mut self,
        def: &PatternDef,
        scalars: &[String],
        sets: &[String],
        domains: &IndexMap<String, Expr>,
        by_depth: &[Vec<&ConstraintLine>],
        rules: &IndexMap<String, Vec<&ConstraintLine>>,
        finals: &[&ConstraintLine],
        i: usize,
        beta: &mut IndexMap<String, RoleVal>,
        res: &mut PatternResult,
        keys: &mut Vec<String>,
    ) -> EResult<()> {
        res.steps += 1;
        if res.steps > STEP_MAX {
            return Err(EvalErr(format!(
                "模式 {} 的搜索空间太大，请用 in 缩小各角色的范围",
                def.name
            )));
        }
        if res.matches.len() >= MATCH_MAX {
            res.truncated = true;
            return Ok(());
        }
        if i >= scalars.len() {
            return self.finish_match(def, scalars, sets, domains, rules, finals, beta, res, keys);
        }
        let role = &scalars[i];
        let dom = self.eval_domain(def, role, domains, beta)?;
        for c in dom {
            let used = beta.values().any(|v| matches!(v, RoleVal::One(n) if *n == c));
            if used {
                continue;
            }
            beta.insert(role.clone(), RoleVal::One(c));
            let mut ok = true;
            for cl in &by_depth[i + 1] {
                if !self.check_constraint(cl, def, beta)? {
                    ok = false;
                    break;
                }
            }
            if ok {
                self.dfs(
                    def,
                    scalars,
                    sets,
                    domains,
                    by_depth,
                    rules,
                    finals,
                    i + 1,
                    beta,
                    res,
                    keys,
                )?;
            }
            beta.shift_remove(role);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_match(
        &mut self,
        def: &PatternDef,
        scalars: &[String],
        sets: &[String],
        domains: &IndexMap<String, Expr>,
        rules: &IndexMap<String, Vec<&ConstraintLine>>,
        finals: &[&ConstraintLine],
        beta: &mut IndexMap<String, RoleVal>,
        res: &mut PatternResult,
        keys: &mut Vec<String>,
    ) -> EResult<()> {
        let used: Vec<NodeId> = beta
            .values()
            .filter_map(|v| match v {
                RoleVal::One(n) => Some(*n),
                _ => None,
            })
            .collect();
        for t in sets {
            let dom = self.eval_domain(def, t, domains, beta)?;
            let mut members = Vec::new();
            for cand in dom {
                if used.contains(&cand) {
                    continue;
                }
                let mut ok = true;
                for cl in &rules[t] {
                    // 把 T 的那一侧换成 {cand}
                    let mut b2 = beta.clone();
                    b2.insert(t.clone(), RoleVal::Many(vec![cand]));
                    if !self.check_constraint(cl, def, &b2)? {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    members.push(cand);
                }
            }
            beta.insert(t.clone(), RoleVal::Many(members));
        }
        let mut all_ok = true;
        for cl in finals {
            if !self.check_constraint(cl, def, beta)? {
                all_ok = false;
                break;
            }
        }
        if !all_ok {
            for t in sets {
                beta.shift_remove(t);
            }
            return Ok(());
        }
        let mut paths: Vec<String> = Vec::new();
        for v in beta.values() {
            match v {
                RoleVal::One(n) => paths.push(self.scene.path(*n)),
                RoleVal::Many(ns) => paths.extend(ns.iter().map(|n| self.scene.path(*n))),
            }
        }
        paths.sort();
        paths.dedup();
        let key = paths.join("|");
        if !keys.contains(&key) {
            keys.push(key);
            // 键按签名顺序
            let mut m: IndexMap<String, RoleVal> = IndexMap::new();
            for (r, _) in &def.roles {
                if let Some(v) = beta.get(r) {
                    m.insert(r.clone(), v.clone());
                }
            }
            res.matches.push(m);
        }
        for t in sets {
            beta.shift_remove(t);
        }
        let _ = scalars;
        Ok(())
    }

    /// 模式值的集合用法（7.5.7）
    pub fn pattern_nodes(&mut self, name: &str, role: Option<&str>) -> EResult<Vec<NodeId>> {
        let pr = match self.sem.pattern_values.get(name) {
            Some(p) => p.clone(),
            None => {
                self.semantic_value(name)?;
                match self.sem.pattern_values.get(name) {
                    Some(p) => p.clone(),
                    None => return Err(EvalErr(format!("找不到对象 {}", name))),
                }
            }
        };
        if let Some(r) = role {
            let def = self.sem.patterns.get(name).cloned();
            if let Some(d) = def {
                if !d.roles.iter().any(|(x, _)| x == r) {
                    let names: Vec<String> = d.roles.iter().map(|(x, _)| x.clone()).collect();
                    return Err(EvalErr(format!(
                        "模式 {} 没有角色 {}（有：{}）",
                        name,
                        r,
                        names.join("、")
                    )));
                }
            }
        }
        let out = role_set(&pr, role);
        Ok(self.sorted_nodes(out))
    }

    pub fn pattern_count(&mut self, name: &str) -> EResult<usize> {
        self.semantic_value(name)?;
        Ok(self
            .sem
            .pattern_values
            .get(name)
            .map(|p| p.matches.len())
            .unwrap_or(0))
    }

    // ====== 节点标注（7.4.4）======
    fn annotate_concepts(&mut self) {
        let pats: Vec<(String, Vec<(String, bool)>)> = self
            .sem
            .patterns
            .iter()
            .map(|(n, d)| (n.clone(), d.roles.clone()))
            .collect();
        for (pname, roles) in pats {
            if let Some(pr) = self.sem.pattern_values.get(&pname).cloned() {
                for (r, _) in roles {
                    let nodes = role_set(&pr, Some(&r));
                    let tag = format!("{}.{}", pname, r);
                    for n in nodes {
                        if !self.scene.nodes[n].concepts.contains(&tag) {
                            self.scene.nodes[n].concepts.push(tag.clone());
                        }
                    }
                }
            }
        }
        let cons: Vec<(String, Vec<NodeId>)> = self
            .sem
            .concept_values
            .iter()
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect();
        for (cname, members) in cons {
            for n in members {
                if !self.scene.nodes[n].concepts.contains(&cname) {
                    self.scene.nodes[n].concepts.push(cname.clone());
                }
            }
        }
    }

    // ====== check（7.6）======
    fn run_check(&mut self, s: &Stmt) {
        let (message, expr, source_text) = match &s.kind {
            StmtKind::Check {
                message,
                expr,
                source_text,
                ..
            } => (message.clone(), expr.clone(), source_text.clone()),
            _ => return,
        };
        let msg = message.unwrap_or(source_text);
        self.pending_facts_warning = None;
        let ctx = self.sem_ctx();
        let r = self.eval(&expr, ctx);
        self.flush_facts_warning(s);
        let entry = match r {
            Ok(Value::Bool(b)) => CheckEntry {
                stmt: s.clone(),
                message: msg,
                ok: b,
                violators: vec![],
                error: None,
            },
            Ok(v) if Self::is_setlike_pub(&v) => match self.to_set(v) {
                Ok(set) => {
                    let set = self.sorted_nodes(set);
                    CheckEntry {
                        stmt: s.clone(),
                        message: msg,
                        ok: set.is_empty(),
                        violators: set,
                        error: None,
                    }
                }
                Err(e) => {
                    self.stmt_error(s.file_index, s.line, e.0.clone());
                    CheckEntry {
                        stmt: s.clone(),
                        message: msg,
                        ok: false,
                        violators: vec![],
                        error: Some(e.0),
                    }
                }
            },
            Ok(v) => {
                let e = format!("check 的结果需要真假值或集合，得到 {}", v.describe());
                self.stmt_error(s.file_index, s.line, e.clone());
                CheckEntry {
                    stmt: s.clone(),
                    message: msg,
                    ok: false,
                    violators: vec![],
                    error: Some(e),
                }
            }
            Err(e) => {
                self.stmt_error(s.file_index, s.line, e.0.clone());
                CheckEntry {
                    stmt: s.clone(),
                    message: msg,
                    ok: false,
                    violators: vec![],
                    error: Some(e.0),
                }
            }
        };
        self.sem.checks.push(entry);
    }

    fn flush_facts_warning(&mut self, s: &Stmt) {
        if let Some(w) = self.pending_facts_warning.take() {
            let file = self.file_display(s.file_index);
            self.diags.warning(s.file_index, &file, s.line, None, w);
        }
    }

    // ====== style（7.7）======
    fn run_style(&mut self, s: &Stmt, selector: &Expr, props: &[Prop], is_over: bool) {
        let what = if is_over { "over" } else { "style" };
        let mut mat: Option<String> = None;
        let mut color: Option<(String, Option<u8>)> = None;
        for p in props {
            match p.key.as_str() {
                "mat" => match self.literal_name(&p.value) {
                    Some(m) if self.material_table.contains_key(&m) => mat = Some(m),
                    Some(m) => {
                        self.stmt_error(s.file_index, s.line, format!("未定义的材质 {}", m));
                        return;
                    }
                    None => {
                        self.stmt_error(s.file_index, s.line, "mat 需要材质名");
                        return;
                    }
                },
                "color" => match &p.value.kind {
                    ExprKind::Hash(h) => match parse_color(h) {
                        Some(c) => color = Some(c),
                        None => {
                            self.stmt_error(
                                s.file_index,
                                s.line,
                                "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa",
                            );
                            return;
                        }
                    },
                    _ => {
                        self.stmt_error(s.file_index, s.line, "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa");
                        return;
                    }
                },
                _ => {
                    self.stmt_error(s.file_index, s.line, "style 只支持 mat 和 color");
                    return;
                }
            }
        }
        if mat.is_none() && color.is_none() {
            self.stmt_error(s.file_index, s.line, "style 需要 mat 或 color");
            return;
        }
        self.pending_facts_warning = None;
        let ctx = self.sem_ctx();
        let members = match self.eval(selector, ctx) {
            Ok(v) if Self::is_setlike_pub(&v) => match self.to_set(v) {
                Ok(set) => set,
                Err(e) => {
                    self.stmt_error(s.file_index, s.line, e.0);
                    return;
                }
            },
            Ok(v) => {
                self.stmt_error(
                    s.file_index,
                    s.line,
                    format!("{} 的选择器需要集合，得到 {}", what, v.describe()),
                );
                return;
            }
            Err(e) => {
                self.stmt_error(s.file_index, s.line, e.0);
                return;
            }
        };
        self.flush_facts_warning(s);
        if members.is_empty() && is_over {
            self.stmt_error(
                s.file_index,
                s.line,
                format!("{} 没有选中任何对象", expr_text(selector)),
            );
            return;
        }
        self.style_actions.push((members, mat, color));
    }

    // ====== variant（7.8）======
    fn run_variant(&mut self, s: &Stmt, selector: &Expr, src: &str, props: &[Prop], is_over: bool) {
        let mut entry = VariantEntry {
            stmt: s.clone(),
            use_t: String::new(),
            src: src.to_string(),
            members: vec![],
            error: None,
            valid: false,
            props: props.iter().filter(|p| p.key != "use").cloned().collect(),
        };
        let fail = |me: &mut Compiler, entry: &mut VariantEntry, msg: String| {
            me.stmt_error(s.file_index, s.line, msg.clone());
            entry.error = Some(msg);
        };
        let use_p = match props.iter().find(|p| p.key == "use") {
            Some(p) => p,
            None => {
                fail(self, &mut entry, "variant 需要 use=模板名".to_string());
                self.sem.variants.push(entry);
                return;
            }
        };
        let tname = match self.literal_name(&use_p.value) {
            Some(t) => t,
            None => {
                fail(self, &mut entry, "variant 需要 use=模板名".to_string());
                self.sem.variants.push(entry);
                return;
            }
        };
        entry.use_t = tname.clone();
        if !self.defs.contains_key(&tname) {
            fail(self, &mut entry, format!("variant 的模板 {} 没有定义", tname));
            self.sem.variants.push(entry);
            return;
        }
        let params = self.template_params(&tname);
        for p in props {
            match p.key.as_str() {
                "use" => {}
                "mat" => match self.literal_name(&p.value) {
                    Some(m) if self.material_table.contains_key(&m) => {}
                    Some(m) => {
                        fail(self, &mut entry, format!("未定义的材质 {}", m));
                        self.sem.variants.push(entry);
                        return;
                    }
                    None => {
                        fail(self, &mut entry, "mat 需要材质名".to_string());
                        self.sem.variants.push(entry);
                        return;
                    }
                },
                "color" => {
                    let ok = matches!(&p.value.kind, ExprKind::Hash(h) if parse_color(h).is_some());
                    if !ok {
                        fail(
                            self,
                            &mut entry,
                            "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa".to_string(),
                        );
                        self.sem.variants.push(entry);
                        return;
                    }
                }
                "at" | "pin" => {
                    fail(self, &mut entry, "变体的位置由外包络决定".to_string());
                    self.sem.variants.push(entry);
                    return;
                }
                k => {
                    if !params.contains(&p.key) {
                        fail(self, &mut entry, format!("模板 {} 没有参数 {}", tname, k));
                        self.sem.variants.push(entry);
                        return;
                    }
                }
            }
        }
        // 必填参数
        let required = self.template_required_params(&tname);
        for r in required {
            let written = props.iter().any(|p| p.key == r);
            if !written && !matches!(r.as_str(), "w" | "h" | "d") {
                fail(self, &mut entry, format!("模板 {} 缺少参数 {}", tname, r));
                self.sem.variants.push(entry);
                return;
            }
        }
        // 选择集
        self.pending_facts_warning = None;
        let ctx = self.sem_ctx();
        let members = match self.eval(selector, ctx) {
            Ok(v) if Self::is_setlike_pub(&v) => match self.to_set(v) {
                Ok(set) => set,
                Err(e) => {
                    fail(self, &mut entry, e.0);
                    self.sem.variants.push(entry);
                    return;
                }
            },
            Ok(v) => {
                fail(
                    self,
                    &mut entry,
                    format!("variant 的选择器需要集合，得到 {}", v.describe()),
                );
                self.sem.variants.push(entry);
                return;
            }
            Err(e) => {
                fail(self, &mut entry, e.0);
                self.sem.variants.push(entry);
                return;
            }
        };
        self.flush_facts_warning(s);
        if members.is_empty() && is_over {
            fail(
                self,
                &mut entry,
                format!("{} 没有选中任何对象", expr_text(selector)),
            );
            self.sem.variants.push(entry);
            return;
        }
        entry.members = self.sorted_nodes(members);
        entry.valid = true;
        self.sem.variants.push(entry);
    }

    /// 模板（含派生链）的必填参数名
    fn template_required_params(&self, tname: &str) -> Vec<String> {
        let mut req: Vec<String> = Vec::new();
        let mut has_default: Vec<String> = Vec::new();
        if let Ok(chain) = self.derivation_chain(tname) {
            for d in chain.iter().rev() {
                if let StmtKind::Def { params, base, .. } = &d.stmt.kind {
                    if let Some((_, overrides)) = base {
                        for o in overrides {
                            has_default.push(o.key.clone());
                        }
                    }
                    for p in params {
                        if p.default.is_some() {
                            has_default.push(p.name.clone());
                        } else if !req.contains(&p.name) {
                            req.push(p.name.clone());
                        }
                    }
                }
            }
        }
        req.retain(|r| !has_default.contains(r));
        req
    }

    /// 7.8.2 的裁决与 4.7 的挂载
    fn mount_variants(&mut self) {
        // 后写的赢：节点 → 最后一条有效 variant 的下标
        let mut owner: IndexMap<NodeId, usize> = IndexMap::new();
        for (i, v) in self.sem.variants.iter().enumerate() {
            if !v.valid {
                continue;
            }
            for &m in &v.members {
                owner.insert(m, i);
            }
        }
        let selected: Vec<NodeId> = owner.keys().copied().collect();
        // 祖先裁决
        let mut mount: Vec<(NodeId, usize)> = Vec::new();
        for (&n, &i) in &owner {
            let has_anc = selected.iter().any(|&a| a != n && self.scene.is_ancestor(a, n));
            if !has_anc {
                mount.push((n, i));
            }
        }
        let idx = self.scene.order_index();
        mount.sort_by_key(|(n, _)| idx[*n]);
        for v in self.sem.variants.iter_mut() {
            v.members.clear();
        }
        for (n, i) in mount {
            // 7.8.6：布尔容器的操作数不挂
            if self.has_bool_ancestor_pub(n) {
                let path = self.scene.path(n);
                let s = self.sem.variants[i].stmt.clone();
                self.stmt_error(
                    s.file_index,
                    s.line,
                    format!("{} 是布尔容器的操作数，不能挂变体：请对整个布尔容器挂", path),
                );
                continue;
            }
            self.sem.variants[i].members.push(n);
            self.mount_skin(n, i);
        }
    }

    /// 4.7：对宿主 N 挂皮肤
    fn mount_skin(&mut self, host: NodeId, vi: usize) {
        let (stmt, tname, props) = {
            let v = &self.sem.variants[vi];
            (v.stmt.clone(), v.use_t.clone(), v.props.clone())
        };
        // 1. 隐藏
        for d in self.scene.subtree_of(host) {
            self.scene.nodes[d].hidden = true;
        }
        // 2–4. 皮肤语句与属性表
        let sf = self.sem.frame.unwrap_or(self.global_frame);
        let mut table: IndexMap<String, PropVal> = IndexMap::new();
        for p in &props {
            table.insert(
                p.key.clone(),
                PropVal {
                    expr: p.value.clone(),
                    frame: sf,
                    start: ROOT,
                    line: p.line,
                    file_index: stmt.file_index,
                },
            );
        }
        let params = self.template_params(&tname);
        let size = self.scene.nodes[host]
            .lbox
            .map(|b| b.size())
            .unwrap_or(Vec3::ZERO);
        for (k, v) in [("w", size.x), ("h", size.y), ("d", size.z)] {
            if params.iter().any(|p| p == k) && !table.contains_key(k) {
                table.insert(
                    k.to_string(),
                    PropVal {
                        expr: Expr {
                            kind: ExprKind::Number(v),
                            span: Span {
                                line: stmt.line,
                                start: 0,
                                end: 0,
                            },
                        },
                        frame: sf,
                        start: ROOT,
                        line: stmt.line,
                        file_index: stmt.file_index,
                    },
                );
            }
        }
        let skin_stmt = Stmt {
            kind: StmtKind::Generic {
                kind: tname.clone(),
                name: Some("skin".to_string()),
                tags: vec![],
                props: vec![],
                metas: vec![],
                block: None,
            },
            line: stmt.line,
            file_index: stmt.file_index,
        };
        let before = self.scene.nodes[host].children.len();
        let created_saved = self.scene.nodes[host].created;
        let mark = self.modifications.len();
        let depth = self.scene.nodes[host].depth;
        self.immediate_copies = true;
        self.prop_override = Some(table);
        self.mounting_skin = true;
        let k = self.expand_generic(&skin_stmt, host, sf, depth, &[]);
        self.mounting_skin = false;
        self.prop_override = None;
        self.scene.nodes[host].created = created_saved;
        let k = match k {
            Some(k) => k,
            None => return,
        };
        let _ = before;
        self.scene.nodes[k].role = Role::Skin;
        self.scene.nodes[k].file_index = stmt.file_index;
        self.scene.nodes[k].line = stmt.line;
        // 皮肤体里的 over / drop 立即施加
        let news: Vec<crate::elaborate::Modification> = self.modifications.drain(mark..).collect();
        for m in news {
            self.apply_modification_pub(&m);
        }
        // 5. late（3.7.6）
        let hk = self.scene.nodes[host].kind.clone();
        let host_env_from_children = !self.scene.nodes[host].degenerate
            && (matches!(hk.as_str(), "group") && !self.scene.nodes[host].props.contains_key("envelope")
                || matches!(
                    hk.as_str(),
                    "union" | "intersect" | "repeat" | "mirror" | "rotate"
                )
                || matches!(self.scene.nodes[host].role, Role::Instance { .. })
                || self.defs.contains_key(&hk));
        self.scene.nodes[k].late = host_env_from_children;
        // 6. 求解皮肤子树
        self.solve_subtree(k);
    }

    // ====== 输出（7.4.4、7.5.8、7.6.3、7.8.9）======
    pub fn semantic_json(&mut self) -> Vec<(String, J)> {
        self.run_semantics();
        let gf = self.global_frame;
        let mut concepts = Vec::new();
        let cdefs: Vec<ConceptDef> = self.sem.concepts.values().cloned().collect();
        for d in cdefs {
            let meta = self.eval_metas(&d.metas, gf, &d.stmt);
            let members: Vec<String> = self
                .sem
                .concept_values
                .get(&d.name)
                .map(|v| v.iter().map(|n| self.scene.path(*n)).collect())
                .unwrap_or_default();
            let mut o = Map::new();
            o.insert("name".into(), json!(d.name));
            o.insert("parent".into(), json!(d.parent));
            o.insert("line".into(), json!(d.stmt.line));
            o.insert("file".into(), json!(self.file_display(d.stmt.file_index)));
            o.insert("src".into(), json!(d.src));
            o.insert("members".into(), json!(members));
            o.insert("meta".into(), crate::output::meta_json_pub(&meta));
            if let Some(e) = self.sem.concept_errors.get(&d.name) {
                o.insert("error".into(), json!(e));
            }
            concepts.push(J::Object(o));
        }
        let mut patterns = Vec::new();
        let pdefs: Vec<PatternDef> = self.sem.patterns.values().cloned().collect();
        for d in pdefs {
            let meta = self.eval_metas(&d.metas, gf, &d.stmt);
            let roles: Vec<String> = d
                .roles
                .iter()
                .map(|(r, s)| if *s { format!("{}*", r) } else { r.clone() })
                .collect();
            let matches: Vec<J> = self
                .sem
                .pattern_values
                .get(&d.name)
                .map(|pr| {
                    pr.matches
                        .iter()
                        .map(|m| {
                            let mut o = Map::new();
                            for (r, v) in m {
                                o.insert(
                                    r.clone(),
                                    match v {
                                        RoleVal::One(n) => json!(self.scene.path(*n)),
                                        RoleVal::Many(ns) => {
                                            json!(ns.iter().map(|n| self.scene.path(*n)).collect::<Vec<_>>())
                                        }
                                    },
                                );
                            }
                            J::Object(o)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut o = Map::new();
            o.insert("name".into(), json!(d.name));
            o.insert("line".into(), json!(d.stmt.line));
            o.insert("file".into(), json!(self.file_display(d.stmt.file_index)));
            o.insert("roles".into(), json!(roles));
            o.insert("matches".into(), J::Array(matches));
            o.insert("meta".into(), crate::output::meta_json_pub(&meta));
            if let Some(e) = self.sem.pattern_errors.get(&d.name) {
                o.insert("error".into(), json!(e));
            }
            patterns.push(J::Object(o));
        }
        let mut checks = Vec::new();
        let cs: Vec<CheckEntry> = self.sem.checks.clone();
        for c in cs {
            let metas = match &c.stmt.kind {
                StmtKind::Check { metas, .. } => metas.clone(),
                _ => vec![],
            };
            let meta = self.eval_metas(&metas, gf, &c.stmt);
            let mut o = Map::new();
            o.insert("line".into(), json!(c.stmt.line));
            o.insert("file".into(), json!(self.file_display(c.stmt.file_index)));
            o.insert("message".into(), json!(c.message));
            o.insert("ok".into(), json!(c.ok));
            o.insert(
                "violators".into(),
                json!(c
                    .violators
                    .iter()
                    .map(|n| self.scene.path(*n))
                    .collect::<Vec<_>>()),
            );
            o.insert("meta".into(), crate::output::meta_json_pub(&meta));
            if let Some(e) = c.error {
                o.insert("error".into(), json!(e));
            }
            checks.push(J::Object(o));
        }
        let mut variants = Vec::new();
        let vs: Vec<VariantEntry> = self.sem.variants.clone();
        for v in vs {
            let metas = match &v.stmt.kind {
                StmtKind::Variant { metas, .. } | StmtKind::Over { metas, .. } => metas.clone(),
                _ => vec![],
            };
            let meta = self.eval_metas(&metas, gf, &v.stmt);
            let mut o = Map::new();
            o.insert("line".into(), json!(v.stmt.line));
            o.insert("file".into(), json!(self.file_display(v.stmt.file_index)));
            o.insert("use".into(), json!(v.use_t));
            o.insert("src".into(), json!(v.src));
            o.insert(
                "members".into(),
                json!(v.members.iter().map(|n| self.scene.path(*n)).collect::<Vec<_>>()),
            );
            o.insert("meta".into(), crate::output::meta_json_pub(&meta));
            if let Some(e) = v.error {
                o.insert("error".into(), json!(e));
            }
            variants.push(J::Object(o));
        }
        vec![
            ("concepts".to_string(), J::Array(concepts)),
            ("patterns".to_string(), J::Array(patterns)),
            ("variants".to_string(), J::Array(variants)),
            ("checks".to_string(), J::Array(checks)),
        ]
    }
}

/// P 组键是否可写在 variant 上（只用于错误文案的归类）
pub fn is_p_group(k: &str) -> bool {
    P_GROUP_ALL.contains(&k)
}
