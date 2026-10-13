//! 表达式求值（spec/01 1.5、1.6.5、1.7.3）。

use crate::ast::{BinOp, Expr, ExprKind};
use crate::compile::{Compiler, Ctx, Timing};
use crate::env::{Binding, FrameId, FrameKind, LazyState};
use crate::lexer::{dir_axis, is_dir_word, Seg};
use crate::scene::{NodeId, Role, ROOT};
use crate::value::{Anchor, RefVal, SurfaceKind, Value};
use s3d_geom::{math, Vec3};

pub struct EvalErr(pub String);
pub type EResult<T> = Result<T, EvalErr>;

fn err<T>(msg: impl Into<String>) -> EResult<T> {
    Err(EvalErr(msg.into()))
}

pub const SUFFIX_WORDS: [&str; 15] = [
    "w", "h", "d", "x", "y", "z", "start", "mid", "end", "top", "side", "surface", "origin", "center", "up",
];

/// 方向词 / 组合词 → 向量（1.6.8）
pub fn dir_vector(word: &str) -> Option<Vec3> {
    let mut v = Vec3::ZERO;
    for w in word.split('-') {
        v += match w {
            "top" | "up" => Vec3::Y,
            "bottom" | "down" => -Vec3::Y,
            "right" | "east" => Vec3::X,
            "left" | "west" => -Vec3::X,
            "front" | "south" => Vec3::Z,
            "back" | "north" => -Vec3::Z,
            "center" => Vec3::ZERO,
            _ => return None,
        };
    }
    Some(v)
}

fn is_dir_compound(s: &str) -> bool {
    !s.is_empty() && s.split('-').all(is_dir_word)
}

fn is_suffix_word(s: &str) -> bool {
    SUFFIX_WORDS.contains(&s) || is_dir_compound(s)
}

impl Compiler {
    // —— 入口 ——
    pub fn eval(&mut self, e: &Expr, ctx: Ctx) -> EResult<Value> {
        match &e.kind {
            ExprKind::Number(n) => Ok(Value::Num(*n)),
            ExprKind::Str(s) => Ok(Value::Str(s.clone())),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Hash(h) => {
                if ctx.timing == Timing::Expand {
                    return err("展开时不能引用对象（#标签）");
                }
                Ok(Value::Set(self.nodes_with_tag(h)))
            }
            ExprKind::DirWord(w) => err(format!("方向词 {} 用错了位置", w.join("-"))),
            ExprKind::Ident(name) => self.resolve_path(&[Seg::Ident(name.clone())], None, None, ctx),
            ExprKind::PathRef { segs, coords, axis } => {
                let coords = match coords {
                    Some(cs) => {
                        let mut out = Vec::new();
                        for c in cs {
                            match self.eval(c, ctx)? {
                                Value::Num(n) => out.push(n),
                                v => return err(format!("坐标后缀里需要数字，得到 {}", v.describe())),
                            }
                        }
                        Some(out)
                    }
                    None => None,
                };
                self.resolve_path(segs, coords, *axis, ctx)
            }
            ExprKind::Call { name, args } => self.call(name, args, ctx),
            ExprKind::Tuple(items) => {
                let mut v = Vec::with_capacity(items.len());
                for it in items {
                    match self.eval(it, ctx)? {
                        Value::Num(n) => v.push(n),
                        other => return err(format!("向量分量必须是数字，得到 {}", other.describe())),
                    }
                }
                Ok(Value::Vector(v))
            }
            ExprKind::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for it in items {
                    v.push(self.eval(it, ctx)?);
                }
                Ok(Value::List(v))
            }
            ExprKind::Neg(inner) => match self.eval(inner, ctx)? {
                Value::Num(n) => Ok(Value::Num(-n)),
                Value::Vector(v) => Ok(Value::Vector(v.into_iter().map(|x| -x).collect())),
                Value::Missing => Ok(Value::Missing),
                other => err(format!("无法计算 -{}", other.describe())),
            },
            ExprKind::Not(inner) => match self.eval(inner, ctx)? {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                Value::Missing => Ok(Value::Bool(false)),
                other => err(format!("需要真假值，得到 {}", other.describe())),
            },
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, ctx),
            ExprKind::Match { lhs, regex } => {
                let l = self.eval(lhs, ctx)?;
                let s = match l {
                    Value::Str(s) => s,
                    Value::Missing => return Ok(Value::Bool(false)),
                    other => return err(format!("~ 左边需要字符串，得到 {}", other.describe())),
                };
                let re = regex::Regex::new(regex).map_err(|_| EvalErr("正则不合法".to_string()))?;
                Ok(Value::Bool(re.is_match(&s)))
            }
            ExprKind::Where { lhs, cond } => {
                if ctx.timing == Timing::Expand {
                    return err("where 只能在求解阶段使用");
                }
                let lv = self.eval(lhs, ctx)?;
                let set = self.to_set(lv)?;
                let mut out = Vec::new();
                for m in set {
                    let wf = self.where_frame(m, ctx.frame);
                    let c = Ctx { frame: wf, ..ctx };
                    match self.eval(cond, c)? {
                        Value::Bool(true) => out.push(m),
                        Value::Bool(false) | Value::Missing => {}
                        other => return err(format!("where 的条件需要真假值，得到 {}", other.describe())),
                    }
                }
                Ok(Value::Set(out))
            }
        }
    }

    /// 字面量类型的属性值（1.4.2 第 1 条）：裸标识符 / 方向组合词 / 字符串按字面取
    pub fn literal_name(&self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Ident(s) => Some(s.clone()),
            ExprKind::DirWord(w) => Some(w.join("-")),
            ExprKind::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    // —— where 帧 ——
    fn where_frame(&mut self, member: NodeId, parent: FrameId) -> FrameId {
        let f = self.env.new_frame(FrameKind::Where, Some(parent));
        self.env.bind(
            f,
            "it",
            Binding::Value(Value::Ref(RefVal {
                node: member,
                anchor: None,
                offset: Vec3::ZERO,
            })),
        );
        f
    }

    /// where 字段（1.7.3）；不是字段返回 None
    fn where_field(&mut self, member: NodeId, name: &str, timing: Timing) -> EResult<Option<Value>> {
        let node = &self.scene.nodes[member];
        Ok(Some(match name {
            "kind" => Value::Str(node.kind.clone()),
            "name" => Value::Str(node.name.clone()),
            "mat" => match self.where_mat_name(member) {
                Some(m) => Value::Str(m),
                None => Value::Missing,
            },
            "w" | "h" | "d" | "x" | "y" | "z" | "top" | "bottom" => {
                if timing != Timing::Appearance {
                    return err("几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）");
                }
                let v = match name {
                    "w" => self.require_lbox(member).size().x,
                    "h" => self.require_lbox(member).size().y,
                    "d" => self.require_lbox(member).size().z,
                    "x" => self.require_wbox(member).center().x,
                    "y" => self.require_wbox(member).center().y,
                    "z" => self.require_wbox(member).center().z,
                    "top" => self.top_bottom(member).1,
                    _ => self.top_bottom(member).0,
                };
                Value::Num(v)
            }
            _ => return Ok(None),
        }))
    }

    // —— 名字解析（1.6.5）——
    fn resolve_path(
        &mut self,
        segs: &[Seg],
        coords: Option<Vec<f64>>,
        axis: Option<char>,
        ctx: Ctx,
    ) -> EResult<Value> {
        let first = match &segs[0] {
            Seg::Ident(s) => s.clone(),
            _ => return err("路径首段必须是名字"),
        };
        let rest = &segs[1..];
        // 第 1 步（含 where 帧的字段与元数据，8.2.2）
        let mut frame = Some(ctx.frame);
        let mut in_where = false;
        while let Some(f) = frame {
            let fk = self.env.frames[f].kind;
            if fk == FrameKind::Where {
                in_where = true;
                let member = match self.env.frames[f].bindings.get("it") {
                    Some(Binding::Value(Value::Ref(r))) => r.node,
                    _ => ROOT,
                };
                if let Some(v) = self.where_field(member, &first, ctx.timing)? {
                    return self.apply_suffix_value(v, rest, coords, axis, ctx);
                }
                if let Some(mv) = self.scene.nodes[self.meta_owner(member)]
                    .meta
                    .get(&first)
                    .cloned()
                {
                    return self.apply_suffix_value(mv, rest, coords, axis, ctx);
                }
                if self.env.frames[f].bindings.contains_key(&first) {
                    let v = self.binding_value(f, &first)?;
                    return self.apply_suffix_value(v, rest, coords, axis, ctx);
                }
            } else if fk == FrameKind::Semantic && self.env.frames[f].bindings.contains_key(&first) {
                // 语义名：惰性求值，失败不缓存，循环报错（7.4.3）
                let v = self.semantic_value(&first)?;
                return self.apply_suffix_value(v, rest, coords, axis, ctx);
            } else if self.env.frames[f].bindings.contains_key(&first) {
                let v = self.binding_value(f, &first)?;
                return self.apply_suffix_value(v, rest, coords, axis, ctx);
            }
            frame = self.env.frames[f].parent;
        }
        if ctx.timing == Timing::Expand {
            return err(format!("找不到名字 {}（展开时只能引用 let、参数与常量）", first));
        }
        // 第 2 步：场景树，从搜索起点向上
        let mut found: Option<NodeId> = None;
        let mut p = ctx.start;
        loop {
            match self.child(p, &first)? {
                Some(n) => {
                    found = Some(n);
                    break;
                }
                None => {
                    if p == ROOT {
                        break;
                    }
                    p = self.scene.nodes[p].parent;
                }
            }
        }
        // 第 3 步：全局索引
        if found.is_none() {
            let cands: Vec<NodeId> = (1..self.scene.nodes.len())
                .filter(|&n| self.scene.nodes[n].name == first && self.scene.in_global_index(n))
                .collect();
            match cands.len() {
                1 => found = Some(cands[0]),
                0 => {
                    if in_where {
                        return Ok(Value::Missing);
                    }
                    return err(format!("找不到对象 {}", first));
                }
                _ => {
                    let paths: Vec<String> = cands.iter().map(|n| self.scene.path(*n)).collect();
                    return err(format!(
                        "名字 {} 不唯一，请写完整路径（{}）",
                        first,
                        paths.join(" / ")
                    ));
                }
            }
        }
        self.descend(found.unwrap(), rest, coords, axis, ctx)
    }

    /// 元数据属于哪个节点：影子取原件（8.3.4）
    fn meta_owner(&self, n: NodeId) -> NodeId {
        match self.scene.nodes[n].role {
            Role::Shadow { original } => self.meta_owner(original),
            _ => n,
        }
    }

    /// 读取词法环境里的绑定（惰性求值，循环检测）
    pub fn binding_value(&mut self, frame: FrameId, name: &str) -> EResult<Value> {
        let b = self.env.frames[frame].bindings.get(name).cloned().unwrap();
        match b {
            Binding::Value(v) => Ok(v),
            Binding::Lazy {
                expr,
                frame: lf,
                start,
                line,
                file_index,
                state,
            } => match state {
                LazyState::Done(v) => Ok(v),
                LazyState::Failed => err(format!("{} 的值无法求出（已在定义处报错）", name)),
                LazyState::Evaluating => {
                    self.stmt_error(file_index, line, format!("{} 循环引用了自己", name));
                    self.set_lazy_state(frame, name, LazyState::Failed);
                    err(format!("{} 循环引用了自己", name))
                }
                LazyState::Unevaluated => {
                    self.set_lazy_state(frame, name, LazyState::Evaluating);
                    let ctx = Ctx {
                        frame: lf,
                        start,
                        timing: Timing::Geometry,
                    };
                    match self.eval(&expr, ctx) {
                        Ok(v) => {
                            self.set_lazy_state(frame, name, LazyState::Done(v.clone()));
                            Ok(v)
                        }
                        Err(e) => {
                            self.stmt_error(file_index, line, e.0.clone());
                            self.set_lazy_state(frame, name, LazyState::Failed);
                            err(format!("{} 的值无法求出", name))
                        }
                    }
                }
            },
        }
    }

    fn set_lazy_state(&mut self, frame: FrameId, name: &str, st: LazyState) {
        if let Some(Binding::Lazy { state, .. }) = self.env.frames[frame].bindings.get_mut(name) {
            *state = st;
        }
    }

    /// child(X, s)（1.6.5）：直接候选取节点顺序最先的一个；匿名非叶子容器穿透
    fn child(&self, x: NodeId, s: &str) -> EResult<Option<NodeId>> {
        let mut direct: Option<NodeId> = None;
        let mut through: Vec<NodeId> = Vec::new();
        for &c in &self.scene.nodes[x].children {
            let node = &self.scene.nodes[c];
            if node.is_named() && node.name == s && matches!(node.role, Role::Normal | Role::TwinOf { .. }) {
                if direct.is_none() {
                    direct = Some(c);
                }
                continue;
            }
            // 穿透：匿名的非叶子容器（group / union / subtract / intersect / repeat、容器式与副本式 mirror / rotate、模板实例）
            let anon = matches!(node.role, Role::Normal | Role::TwinOf { .. }) && node.name.is_empty();
            if anon && !node.children.is_empty() && !node.kind.is_empty() {
                if let Some(found) = self.child(c, s)? {
                    through.push(found);
                }
            }
        }
        let mut cands: Vec<NodeId> = direct.into_iter().collect();
        cands.extend(through);
        match cands.len() {
            0 => Ok(None),
            1 => Ok(Some(cands[0])),
            _ => {
                let paths: Vec<String> = cands.iter().map(|n| self.scene.path(*n)).collect();
                err(format!(
                    "名字 {} 不唯一，请写完整路径（{}）",
                    s,
                    paths.join(" / ")
                ))
            }
        }
    }

    /// 从节点 b 沿余下的段下降，后缀按 1.5.4 解释
    fn descend(
        &mut self,
        b: NodeId,
        rest: &[Seg],
        coords: Option<Vec<f64>>,
        axis: Option<char>,
        ctx: Ctx,
    ) -> EResult<Value> {
        let mut cur = b;
        let mut i = 0;
        while i < rest.len() {
            match &rest[i] {
                Seg::Star => {
                    // 通配：该层全部子节点 → 集合；余下段对每个成员继续
                    let kids: Vec<NodeId> = self.scene.nodes[cur].children.clone();
                    let mut out = Vec::new();
                    for k in kids {
                        match self.descend(k, &rest[i + 1..], None, None, ctx)? {
                            Value::Ref(r) if r.anchor.is_none() => out.push(r.node),
                            Value::Set(s) => out.extend(s),
                            _ => return err("通配路径后面不能取锚点"),
                        }
                    }
                    if coords.is_some() || axis.is_some() {
                        return err("通配路径后面不能取锚点");
                    }
                    return Ok(Value::Set(self.sorted_nodes(out)));
                }
                Seg::Digits(d) => {
                    let found = self.scene.nodes[cur].children.iter().copied().find(|&c| {
                        matches!(self.scene.nodes[c].role, Role::Instance { .. })
                            && self.scene.nodes[c].seg == *d
                    });
                    match found {
                        Some(n) => {
                            cur = n;
                            i += 1;
                        }
                        None => return err(format!("{} 中没有子对象或锚点 {}", self.scene.path(cur), d)),
                    }
                }
                Seg::Ident(s) => {
                    if is_suffix_word(s) {
                        break;
                    }
                    // 副本组 / 皮肤等带段名的子节点按段匹配；具名子节点经 child()
                    let by_seg = self.scene.nodes[cur].children.iter().copied().find(|&c| {
                        !matches!(self.scene.nodes[c].role, Role::Normal | Role::TwinOf { .. })
                            && self.scene.nodes[c].seg == *s
                    });
                    let next = match by_seg {
                        Some(n) => Some(n),
                        None => self.child(cur, s)?,
                    };
                    match next {
                        Some(n) => {
                            cur = n;
                            i += 1;
                        }
                        None => break,
                    }
                }
            }
        }
        let tail: Vec<String> = rest[i..]
            .iter()
            .map(|s| match s {
                Seg::Ident(x) => x.clone(),
                Seg::Digits(d) => d.clone(),
                Seg::Star => "*".to_string(),
            })
            .collect();
        self.suffix(cur, &tail, coords, axis, ctx)
    }

    /// 后缀解释（1.5.4）
    fn suffix(
        &mut self,
        b: NodeId,
        tail: &[String],
        coords: Option<Vec<f64>>,
        axis: Option<char>,
        ctx: Ctx,
    ) -> EResult<Value> {
        let path = || self.scene.path(b);
        let is_curve = crate::ast::CURVE_KINDS.contains(&self.scene.nodes[b].kind.as_str());
        let mut anchor: Option<Anchor> = None;
        let mut measure: Option<char> = None;
        let mut comp: Option<char> = None;
        let mut idx = 0;
        if let Some(cs) = &coords {
            // 坐标后缀直接跟在路径后（tail 为空）或跟在曲面名后
            if tail.is_empty() {
                anchor = Some(match cs.len() {
                    1 => {
                        if !is_curve {
                            return err(format!("{} 不是曲线，不能用 [t] 取点", path()));
                        }
                        Anchor::CurveT(cs[0])
                    }
                    3 => Anchor::Norm(Vec3::new(cs[0], cs[1], cs[2])),
                    _ => {
                        return err("锚点写成 名字[u,v,w]（包围盒内 -1..1），曲线上的点写成 名字[t]（0..1）")
                    }
                });
            } else if tail.len() == 1 && matches!(tail[0].as_str(), "top" | "side" | "surface") {
                if cs.len() != 2 {
                    return err("曲面锚点写成 B.top[a,b]（两个数）");
                }
                let which = match tail[0].as_str() {
                    "top" => SurfaceKind::Top,
                    "side" => SurfaceKind::Side,
                    _ => SurfaceKind::Surface,
                };
                anchor = Some(Anchor::Surface {
                    which,
                    a: cs[0],
                    b: cs[1],
                });
                idx = 1;
            } else {
                return err(format!("{} 中没有子对象或锚点 {}", path(), tail.join(".")));
            }
        } else if !tail.is_empty() {
            let s = tail[0].as_str();
            idx = 1;
            match s {
                "w" | "h" | "d" => measure = Some(s.chars().next().unwrap()),
                "x" | "y" | "z" => comp = Some(s.chars().next().unwrap()),
                "origin" => anchor = Some(Anchor::Origin),
                "start" | "mid" | "end" => {
                    if !is_curve {
                        return err(format!("{} 中没有子对象或锚点 {}", path(), s));
                    }
                    anchor = Some(Anchor::CurveNamed(match s {
                        "start" => 0.0,
                        "mid" => 0.5,
                        _ => 1.0,
                    }));
                }
                _ if is_dir_compound(s) => anchor = Some(Anchor::Dir(dir_vector(s).unwrap())),
                _ => return err(format!("{} 中没有子对象或锚点 {}", path(), tail.join("."))),
            }
        }
        // 锚点后面的分量
        if idx < tail.len() {
            if anchor.is_some()
                && tail.len() == idx + 1
                && matches!(tail[idx].as_str(), "x" | "y" | "z")
                && axis.is_none()
            {
                comp = Some(tail[idx].chars().next().unwrap());
                idx += 1;
            } else {
                return err(format!("{} 中没有子对象或锚点 {}", path(), tail.join(".")));
            }
        }
        if let Some(a) = axis {
            if comp.is_some() || anchor.is_none() {
                return err(format!("{} 中没有子对象或锚点 .{}", path(), a));
            }
            comp = Some(a);
        }
        let _ = idx;
        if let Some(m) = measure {
            if ctx.timing == Timing::Selector {
                return err("几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）");
            }
            let size = self.require_lbox(b).size();
            return Ok(Value::Num(match m {
                'w' => size.x,
                'h' => size.y,
                _ => size.z,
            }));
        }
        let r = RefVal {
            node: b,
            anchor,
            offset: Vec3::ZERO,
        };
        if let Some(c) = comp {
            if ctx.timing == Timing::Selector {
                return err("几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）");
            }
            let (q, _nu, _tau) = self.anchor_in_frame(&r, ctx.start)?;
            return Ok(Value::Num(match c {
                'x' => q.x,
                'y' => q.y,
                _ => q.z,
            }));
        }
        Ok(Value::Ref(r))
    }

    /// 词法环境命中后，余下的段作用在值上（1.6.5 第 1 步）
    fn apply_suffix_value(
        &mut self,
        v: Value,
        rest: &[Seg],
        coords: Option<Vec<f64>>,
        axis: Option<char>,
        ctx: Ctx,
    ) -> EResult<Value> {
        if rest.is_empty() && coords.is_none() && axis.is_none() {
            return Ok(v);
        }
        match v {
            Value::Ref(r) if r.anchor.is_none() && r.offset == Vec3::ZERO => {
                self.descend(r.node, rest, coords, axis, ctx)
            }
            Value::Set(s) => {
                let tail: Vec<String> = rest
                    .iter()
                    .map(|x| match x {
                        Seg::Ident(i) => i.clone(),
                        Seg::Digits(d) => d.clone(),
                        Seg::Star => "*".to_string(),
                    })
                    .collect();
                self.set_suffix(s, &tail, coords, axis, ctx)
            }
            Value::Pattern(p) => {
                // 模式值后面只能跟一个角色名（7.5.7）
                let tail: Vec<String> = rest
                    .iter()
                    .map(|x| match x {
                        Seg::Ident(i) => i.clone(),
                        Seg::Digits(d) => d.clone(),
                        Seg::Star => "*".to_string(),
                    })
                    .collect();
                if tail.len() != 1 || coords.is_some() || axis.is_some() {
                    let names: Vec<String> = self
                        .sem
                        .patterns
                        .get(&p)
                        .map(|d| d.roles.iter().map(|(r, _)| r.clone()).collect())
                        .unwrap_or_default();
                    return err(format!(
                        "模式 {} 没有角色 {}（有：{}）",
                        p,
                        tail.join("."),
                        names.join("、")
                    ));
                }
                let s = self.pattern_nodes(&p, Some(&tail[0]))?;
                Ok(Value::Set(s))
            }
            other => {
                let rest_s: Vec<String> = rest
                    .iter()
                    .map(|x| match x {
                        Seg::Ident(i) => i.clone(),
                        Seg::Digits(d) => d.clone(),
                        Seg::Star => "*".to_string(),
                    })
                    .collect();
                err(format!(
                    "{} 不是对象，不能取 .{}",
                    other.describe(),
                    rest_s.join(".")
                ))
            }
        }
    }

    /// 集合的锚点 / 度量 / 分量（3.2.8）
    fn set_suffix(
        &mut self,
        s: Vec<NodeId>,
        tail: &[String],
        coords: Option<Vec<f64>>,
        axis: Option<char>,
        ctx: Ctx,
    ) -> EResult<Value> {
        if ctx.timing == Timing::Selector {
            return err("几何时机的选择器只能用静态量（路径、#标签、kind()、name / 元数据 / mat）");
        }
        if s.is_empty() {
            return err("集合是空的，不能当参照物");
        }
        let b = self.set_box_world(&s);
        let mut anchor: Option<Anchor> = None;
        let mut comp: Option<char> = None;
        let mut i = 0;
        if let Some(cs) = coords {
            if tail.is_empty() && cs.len() == 3 {
                anchor = Some(Anchor::Norm(Vec3::new(cs[0], cs[1], cs[2])));
            } else if tail.is_empty() && cs.len() == 1 {
                return err("{…} 不是曲线，不能用 [t] 取点");
            } else {
                return err("{…} 中没有子对象或锚点".to_string());
            }
        } else if !tail.is_empty() {
            i = 1;
            match tail[0].as_str() {
                "w" => return Ok(Value::Num(b.size().x)),
                "h" => return Ok(Value::Num(b.size().y)),
                "d" => return Ok(Value::Num(b.size().z)),
                "x" | "y" | "z" => comp = Some(tail[0].chars().next().unwrap()),
                "origin" => anchor = Some(Anchor::Origin),
                w if is_dir_compound(w) => anchor = Some(Anchor::Dir(dir_vector(w).unwrap())),
                w => return err(format!("{{…}} 中没有子对象或锚点 {}", w)),
            }
        }
        if i < tail.len() {
            if anchor.is_some() && tail.len() == i + 1 && matches!(tail[i].as_str(), "x" | "y" | "z") {
                comp = Some(tail[i].chars().next().unwrap());
            } else {
                return err(format!("{{…}} 中没有子对象或锚点 {}", tail[i..].join(".")));
            }
        }
        if let Some(a) = axis {
            comp = Some(a);
        }
        // 集合的点在世界系里，换到 P
        let p_world = match anchor {
            None => b.center(),
            Some(Anchor::Origin) => Vec3::ZERO,
            Some(Anchor::Dir(a)) => b.anchor(a),
            Some(Anchor::Norm(u)) => b.anchor(u),
            _ => return err("{…} 中没有子对象或锚点"),
        };
        if let Some(c) = comp {
            let wp = self.require_world(ctx.start);
            let q = wp.inverse().apply(p_world);
            return Ok(Value::Num(match c {
                'x' => q.x,
                'y' => q.y,
                _ => q.z,
            }));
        }
        // 带锚点的集合引用：用一个特殊表示——集合值 + 锚点由调用方处理。这里退化为集合本身（无锚点）
        if anchor.is_some() {
            return err("集合引用的锚点只能用于定位属性（通过 let 绑定）");
        }
        Ok(Value::Set(s))
    }

    // —— 集合工具 ——
    pub fn nodes_with_tag(&self, tag: &str) -> Vec<NodeId> {
        let order = self.scene.order();
        order
            .into_iter()
            .filter(|&n| self.scene.nodes[n].tags.iter().any(|t| t == tag) && !self.scene.in_skin(n))
            .collect()
    }

    pub fn sorted_nodes(&self, ids: Vec<NodeId>) -> Vec<NodeId> {
        if self.order_idx.len() == self.scene.nodes.len() {
            self.scene.sorted_set(&self.order_idx, ids)
        } else {
            let idx = self.scene.order_index();
            self.scene.sorted_set(&idx, ids)
        }
    }

    /// 转成集合类（1.5.5）
    pub fn to_set(&mut self, v: Value) -> EResult<Vec<NodeId>> {
        match v {
            Value::Set(s) => Ok(s),
            Value::Pattern(p) => self.pattern_nodes(&p, None),
            Value::Ref(r) => {
                if r.anchor.is_some() || r.offset != Vec3::ZERO {
                    return err(format!(
                        "{} 不是集合（锚点与偏移不能当集合）",
                        self.scene.path(r.node)
                    ));
                }
                Ok(vec![r.node])
            }
            Value::List(items) => {
                let mut out = Vec::new();
                for it in items {
                    out.extend(self.to_set(it)?);
                }
                Ok(self.sorted_nodes(out))
            }
            other => err(format!(
                "需要集合（对象、#标签、kind(…)、列表…），得到 {}",
                other.describe()
            )),
        }
    }

    pub fn is_setlike_pub(v: &Value) -> bool {
        Self::is_setlike(v)
    }

    pub fn where_frame_pub(&mut self, member: NodeId, parent: FrameId) -> FrameId {
        self.where_frame(member, parent)
    }

    fn is_setlike(v: &Value) -> bool {
        matches!(
            v,
            Value::Set(_) | Value::Ref(_) | Value::List(_) | Value::Pattern(_)
        )
    }

    // —— 二元运算 ——
    fn binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, ctx: Ctx) -> EResult<Value> {
        // 名字字面量规则（1.5.5）：kind == box 等
        if matches!(op, BinOp::Eq | BinOp::Ne) {
            if let ExprKind::Ident(f) = &lhs.kind {
                if matches!(f.as_str(), "kind" | "name" | "mat") && self.in_where(ctx.frame) {
                    if let ExprKind::Ident(r) = &rhs.kind {
                        let l = self.eval(lhs, ctx)?;
                        let equal = matches!(&l, Value::Str(s) if s == r);
                        return Ok(Value::Bool(if op == BinOp::Eq {
                            equal
                        } else {
                            !equal && !matches!(l, Value::Missing)
                        }));
                    }
                }
            }
        }
        let l = self.eval(lhs, ctx)?;
        let r = self.eval(rhs, ctx)?;
        match op {
            BinOp::Add | BinOp::Sub => match (&l, &r) {
                (Value::Missing, _) | (_, Value::Missing) => Ok(Value::Missing),
                (Value::Num(a), Value::Num(b)) => {
                    Ok(Value::Num(if op == BinOp::Add { a + b } else { a - b }))
                }
                (Value::Vector(a), Value::Vector(b)) if a.len() == b.len() => Ok(Value::Vector(
                    a.iter()
                        .zip(b)
                        .map(|(x, y)| if op == BinOp::Add { x + y } else { x - y })
                        .collect(),
                )),
                (Value::Ref(rf), Value::Vector(v)) if v.len() == 3 => {
                    let d = Vec3::new(v[0], v[1], v[2]);
                    let mut nr = rf.clone();
                    nr.offset += if op == BinOp::Add { d } else { -d };
                    Ok(Value::Ref(nr))
                }
                _ if op == BinOp::Sub && Self::is_setlike(&l) && Self::is_setlike(&r) => {
                    let a = self.to_set(l)?;
                    let b = self.to_set(r)?;
                    Ok(Value::Set(a.into_iter().filter(|x| !b.contains(x)).collect()))
                }
                _ => err(format!(
                    "无法计算 {} {} {}",
                    l.describe(),
                    if op == BinOp::Add { "+" } else { "-" },
                    r.describe()
                )),
            },
            BinOp::Mul => match (&l, &r) {
                (Value::Missing, _) | (_, Value::Missing) => Ok(Value::Missing),
                (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
                (Value::Vector(a), Value::Num(b)) | (Value::Num(b), Value::Vector(a)) => {
                    Ok(Value::Vector(a.iter().map(|x| x * b).collect()))
                }
                _ => err(format!("无法计算 {} * {}", l.describe(), r.describe())),
            },
            BinOp::Div => match (&l, &r) {
                (Value::Missing, _) | (_, Value::Missing) => Ok(Value::Missing),
                (_, Value::Num(b)) if *b == 0.0 => err("除以零"),
                (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),
                (Value::Vector(a), Value::Num(b)) => Ok(Value::Vector(a.iter().map(|x| x / b).collect())),
                _ => err(format!("无法计算 {} / {}", l.describe(), r.describe())),
            },
            BinOp::Eq | BinOp::Ne => {
                if matches!(l, Value::Missing) || matches!(r, Value::Missing) {
                    return Ok(Value::Bool(false));
                }
                let eq = self.values_equal(&l, &r)?;
                Ok(Value::Bool(if op == BinOp::Eq { eq } else { !eq }))
            }
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => match (&l, &r) {
                (Value::Missing, _) | (_, Value::Missing) => Ok(Value::Bool(false)),
                (Value::Num(a), Value::Num(b)) => Ok(Value::Bool(match op {
                    BinOp::Lt => a < b,
                    BinOp::Gt => a > b,
                    BinOp::Le => a <= b,
                    _ => a >= b,
                })),
                _ => err(format!(
                    "比较两边需要数字，得到 {} 和 {}",
                    l.describe(),
                    r.describe()
                )),
            },
            BinOp::And | BinOp::Or => {
                if Self::is_setlike(&l) && Self::is_setlike(&r) {
                    let a = self.to_set(l)?;
                    let b = self.to_set(r)?;
                    let out: Vec<NodeId> = if op == BinOp::And {
                        a.into_iter().filter(|x| b.contains(x)).collect()
                    } else {
                        let mut v = a;
                        v.extend(b);
                        self.sorted_nodes(v)
                    };
                    return Ok(Value::Set(out));
                }
                match (&l, &r) {
                    (Value::Missing, _) | (_, Value::Missing) => Ok(Value::Bool(false)),
                    (Value::Bool(a), Value::Bool(b)) => {
                        Ok(Value::Bool(if op == BinOp::And { *a && *b } else { *a || *b }))
                    }
                    _ => {
                        let bad = if !matches!(l, Value::Bool(_)) { &l } else { &r };
                        err(format!("需要真假值，得到 {}", bad.describe()))
                    }
                }
            }
        }
    }

    fn in_where(&self, mut frame: FrameId) -> bool {
        loop {
            if self.env.frames[frame].kind == FrameKind::Where {
                return true;
            }
            match self.env.frames[frame].parent {
                Some(p) => frame = p,
                None => return false,
            }
        }
    }

    fn values_equal(&mut self, l: &Value, r: &Value) -> EResult<bool> {
        Ok(match (l, r) {
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::Ref(a), Value::Ref(b)) => a.node == b.node,
            (Value::Set(_) | Value::List(_), Value::Set(_) | Value::List(_)) => {
                let a = self.to_set(l.clone())?;
                let b = self.to_set(r.clone())?;
                a == b
            }
            (Value::Ref(a), Value::Set(b)) | (Value::Set(b), Value::Ref(a)) => b.len() == 1 && b[0] == a.node,
            _ => false,
        })
    }

    // —— 函数（1.5.6）——
    fn call(&mut self, name: &str, args: &[Expr], ctx: Ctx) -> EResult<Value> {
        let nums = |c: &mut Compiler, n: usize| -> EResult<Vec<f64>> {
            if args.len() != n {
                return err(format!("{}() 需要 {} 个参数", name, n));
            }
            let mut out = Vec::new();
            for a in args {
                match c.eval(a, ctx)? {
                    Value::Num(x) => out.push(x),
                    Value::Missing => return Err(EvalErr("__missing__".to_string())),
                    other => return err(format!("{}() 需要数字参数，得到 {}", name, other.describe())),
                }
            }
            Ok(out)
        };
        let finite = |v: f64| -> EResult<Value> {
            if v.is_finite() {
                Ok(Value::Num(v))
            } else {
                err(format!("{}() 的结果不是有限数", name))
            }
        };
        let res = match name {
            "sin" => nums(self, 1).map(|a| math::sin(math::rad(a[0]))).and_then(finite),
            "cos" => nums(self, 1).map(|a| math::cos(math::rad(a[0]))).and_then(finite),
            "tan" => nums(self, 1).map(|a| math::tan(math::rad(a[0]))).and_then(finite),
            "asin" => nums(self, 1)
                .map(|a| math::deg(math::asin(a[0])))
                .and_then(finite),
            "acos" => nums(self, 1)
                .map(|a| math::deg(math::acos(a[0])))
                .and_then(finite),
            "atan" => nums(self, 1)
                .map(|a| math::deg(math::atan(a[0])))
                .and_then(finite),
            "atan2" => nums(self, 2)
                .map(|a| math::deg(math::atan2(a[0], a[1])))
                .and_then(finite),
            "sqrt" => nums(self, 1).map(|a| math::sqrt(a[0])).and_then(finite),
            "abs" => nums(self, 1).map(|a| a[0].abs()).and_then(finite),
            "floor" => nums(self, 1).map(|a| a[0].floor()).and_then(finite),
            "ceil" => nums(self, 1).map(|a| a[0].ceil()).and_then(finite),
            "round" => nums(self, 1).map(|a| a[0].round()).and_then(finite),
            "pow" => nums(self, 2).map(|a| math::pow(a[0], a[1])).and_then(finite),
            "min" | "max" => {
                if args.is_empty() {
                    return err(format!("{}() 需要至少 1 个参数", name));
                }
                let mut best: Option<f64> = None;
                for a in args {
                    match self.eval(a, ctx)? {
                        Value::Num(x) => {
                            best = Some(match best {
                                None => x,
                                Some(b) => {
                                    if name == "min" {
                                        b.min(x)
                                    } else {
                                        b.max(x)
                                    }
                                }
                            })
                        }
                        Value::Missing => return Ok(Value::Missing),
                        other => return err(format!("{}() 需要数字参数，得到 {}", name, other.describe())),
                    }
                }
                Ok(Value::Num(best.unwrap()))
            }
            "count" => {
                if args.len() != 1 {
                    return err("count() 需要 1 个参数");
                }
                if ctx.timing == Timing::Expand {
                    return err("count() 不能在展开时使用");
                }
                let v = self.eval(&args[0], ctx)?;
                if let Value::Pattern(p) = &v {
                    return Ok(Value::Num(self.pattern_count(p)? as f64));
                }
                if !Self::is_setlike(&v) {
                    return err("count() 需要集合或模式");
                }
                let s = self.to_set(v)?;
                Ok(Value::Num(s.len() as f64))
            }
            "all" => {
                if !args.is_empty() {
                    return err("all() 需要 0 个参数");
                }
                if ctx.timing == Timing::Expand {
                    return err("all() 不能在展开时使用");
                }
                let all: Vec<NodeId> = self
                    .scene
                    .order()
                    .into_iter()
                    .filter(|&n| !self.scene.in_skin(n))
                    .collect();
                Ok(Value::Set(all))
            }
            "kind" => {
                if args.is_empty() {
                    return err("kind() 需要至少 1 个参数");
                }
                if ctx.timing == Timing::Expand {
                    return err("kind() 不能在展开时使用");
                }
                let mut names = Vec::new();
                for a in args {
                    match self.literal_name(a) {
                        Some(n) => names.push(n),
                        None => return err("kind() 的参数要写种类名或模板名"),
                    }
                }
                for n in &names {
                    if !crate::ast::is_builtin_kind(n)
                        && !self.defs.contains_key(n)
                        && !matches!(n.as_str(), "instance" | "copy")
                    {
                        let msg = format!("kind() 不认识 {}", n);
                        let file = self.file_display(0);
                        self.diags.warning(0, &file, 0, None, msg);
                    }
                }
                let out: Vec<NodeId> = self
                    .scene
                    .order()
                    .into_iter()
                    .filter(|&n| {
                        let node = &self.scene.nodes[n];
                        !self.scene.in_skin(n)
                            && (names.contains(&node.kind) || node.types.iter().any(|t| names.contains(t)))
                    })
                    .collect();
                Ok(Value::Set(out))
            }
            "children" | "descendants" => {
                if args.len() != 1 {
                    return err(format!("{}() 需要 1 个参数", name));
                }
                if ctx.timing == Timing::Expand {
                    return err(format!("{}() 不能在展开时使用", name));
                }
                let v = self.eval(&args[0], ctx)?;
                let s = self.to_set(v)?;
                let mut out = Vec::new();
                for m in s {
                    if name == "children" {
                        out.extend(self.scene.nodes[m].children.iter().copied());
                    } else {
                        let mut stack: Vec<NodeId> = self.scene.nodes[m].children.clone();
                        while let Some(c) = stack.pop() {
                            out.push(c);
                            stack.extend(self.scene.nodes[c].children.iter().copied());
                        }
                    }
                }
                Ok(Value::Set(self.sorted_nodes(out)))
            }
            "on" | "supports" | "against" | "inside" | "contains" | "touching" | "intersects" => {
                if ctx.timing != Timing::Appearance {
                    return err(format!(
                        "{}() 依赖求解后的空间事实，不能用来决定位置 / 几何",
                        name
                    ));
                }
                self.relation_predicate(name, args, ctx)
            }
            _ => err(format!("未知函数 {}()", name)),
        };
        match res {
            Err(EvalErr(m)) if m == "__missing__" => Ok(Value::Missing),
            other => other,
        }
    }

    /// 方向词 / 组合词的轴（供 align 等用）
    pub fn dir_axis_of(word: &str) -> Option<u8> {
        dir_axis(word)
    }
}
