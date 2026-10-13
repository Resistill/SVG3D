//! 阶段 7 的求解（spec/03）：局部变换、局部盒、世界变换、世界盒。惰性求值 + 忙标记判定后置与循环（3.6）。

use crate::ast::{Expr, ExprKind};
use crate::compile::{Compiler, Ctx, Timing};
use crate::elaborate::prop_ctx;
use crate::eval::{dir_vector, EResult, EvalErr};
use crate::scene::{NodeId, Role, ROOT};
use crate::value::{Anchor, RefVal, Value};
use s3d_geom::{math, Aabb, Axis, Mat3, Solid, Transform, Vec3};

#[derive(Clone, Copy, Debug)]
pub enum SolveErr {
    /// 求 n 的局部变换时再次进入 n 的局部变换（3.6.5 循环）
    Cycle(NodeId),
    /// 求容器 a 的盒时再次进入 a 的盒（3.6.3 后置）
    BoxCycle(NodeId),
}

const FACES: [(&str, Vec3); 6] = [
    ("left", Vec3::NEG_X),
    ("right", Vec3::X),
    ("front", Vec3::Z),
    ("back", Vec3::NEG_Z),
    ("top", Vec3::Y),
    ("bottom", Vec3::NEG_Y),
];

/// 关系求出的量（3.3.1）
struct Relation {
    q: Vec3,
    nu: Vec3,
    /// 缺省定位点：None = 自动选面（against）
    default_pin: Option<Vec3>,
    pin_origin: bool,
    inherit: bool,
    /// R_{B→P}（继承用）
    r_base: Mat3,
    /// 对齐目标（参照物或集合）
    target: Option<Value>,
}

impl Compiler {
    // ====== 入口 ======
    pub fn solve_all(&mut self) {
        let n = self.scene.nodes.len();
        self.busy_local = vec![false; n];
        self.busy_box = vec![false; n];
        for id in self.scene.order() {
            let _ = self.require_world(id);
            let _ = self.require_lbox(id);
            let _ = self.require_wbox(id);
        }
    }

    pub fn require_world(&mut self, n: NodeId) -> Transform {
        if let Some(w) = self.scene.nodes[n].world {
            return w;
        }
        let parent = self.scene.nodes[n].parent;
        let wp = if n == ROOT {
            Transform::IDENTITY
        } else {
            self.require_world(parent)
        };
        let l = self.require_local(n);
        let w = wp.then_inner(&l);
        self.scene.nodes[n].world = Some(w);
        w
    }

    pub fn require_wbox(&mut self, n: NodeId) -> Aabb {
        if let Some(b) = self.scene.nodes[n].wbox {
            return b;
        }
        let w = self.require_world(n);
        let lb = self.require_lbox(n);
        let b = lb.transformed(&w);
        self.scene.nodes[n].wbox = Some(b);
        b
    }

    /// 局部变换；循环时报错、退化并返回恒等
    pub fn require_local(&mut self, n: NodeId) -> Transform {
        match self.local(n) {
            Ok(t) => t,
            Err(_) => {
                self.scene.nodes[n].local = Some(Transform::IDENTITY);
                Transform::IDENTITY
            }
        }
    }

    pub fn local(&mut self, n: NodeId) -> Result<Transform, SolveErr> {
        if let Some(t) = self.scene.nodes[n].local {
            return Ok(t);
        }
        if n == ROOT {
            return Ok(Transform::IDENTITY);
        }
        if self.busy_local[n] {
            // 正在求 L(n) 的过程中求 X(n)（定位点要读自己的盒），子节点又读到了 L(n)：
            // 这是 3.6.3 的"子节点依赖容器自己的位置或盒"，按容器盒的环处理（子节点后置）
            if self.busy_box[n] {
                return Err(SolveErr::BoxCycle(n));
            }
            return Err(SolveErr::Cycle(n));
        }
        if self.scene.nodes[n].degenerate {
            self.scene.nodes[n].local = Some(Transform::IDENTITY);
            return Ok(Transform::IDENTITY);
        }
        self.busy_local[n] = true;
        let res = self.compute_local(n);
        self.busy_local[n] = false;
        match res {
            Ok(t) => {
                self.scene.nodes[n].local = Some(t);
                Ok(t)
            }
            Err(SolveErr::Cycle(c)) if c == n => {
                self.degenerate(n, "定位关系形成了循环依赖");
                self.scene.nodes[n].local = Some(Transform::IDENTITY);
                Ok(Transform::IDENTITY)
            }
            Err(e @ SolveErr::Cycle(_)) => {
                // 环上的其他节点：同样退化
                self.degenerate(n, "定位关系形成了循环依赖");
                self.scene.nodes[n].local = Some(Transform::IDENTITY);
                Err(e)
            }
            Err(e) => Err(e),
        }
    }

    fn compute_local(&mut self, n: NodeId) -> Result<Transform, SolveErr> {
        let role = self.scene.nodes[n].role.clone();
        let kind = self.scene.nodes[n].kind.clone();
        match role {
            Role::Instance { .. } => self.instance_local(n),
            Role::TwinGroup { .. } | Role::TwinOf { .. } | Role::Shadow { .. } => Ok(Transform::IDENTITY),
            Role::Skin => Ok(Transform::IDENTITY),
            Role::Normal => match kind.as_str() {
                "span" => self.span_local(n),
                "sweep" | "pipe" | "line" | "bezier" | "spline" => Ok(Transform::IDENTITY),
                _ => self.general_local(n),
            },
        }
    }

    // ====== 属性读取 ======
    /// 属性值求值（Geometry 时机）；出错时报错并退化
    fn prop_value(&mut self, n: NodeId, key: &str) -> Result<Option<Value>, SolveErr> {
        let pv = match self.scene.nodes[n].props.get(key).cloned() {
            Some(pv) => pv,
            None => return Ok(None),
        };
        let ctx = prop_ctx(&pv, Timing::Geometry);
        match self.eval(&pv.expr, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(EvalErr(m)) => {
                if let Some(c) = m.strip_prefix("__cycle__") {
                    // 求值期间触发的求解循环
                    let id: NodeId = c.parse().unwrap_or(n);
                    return Err(SolveErr::Cycle(id));
                }
                if let Some(c) = m.strip_prefix("__boxcycle__") {
                    let id: NodeId = c.parse().unwrap_or(n);
                    return Err(SolveErr::BoxCycle(id));
                }
                let msg = format!("{} 的 {}：{}", self.scene.path(n), key, m);
                self.degenerate(n, msg);
                Err(SolveErr::Cycle(usize::MAX))
            }
        }
    }

    fn prop_num(&mut self, n: NodeId, key: &str, default: f64) -> Result<f64, SolveErr> {
        match self.prop_value(n, key)? {
            None => Ok(default),
            Some(Value::Num(x)) if x.is_finite() => Ok(x),
            Some(v) => {
                let msg = format!(
                    "{} 的 {} 需要数字，得到 {}",
                    self.scene.path(n),
                    key,
                    v.describe()
                );
                self.degenerate(n, msg);
                Err(SolveErr::Cycle(usize::MAX))
            }
        }
    }

    fn prop_vec3(&mut self, n: NodeId, key: &str, default: Vec3) -> Result<Vec3, SolveErr> {
        match self.prop_value(n, key)? {
            None => Ok(default),
            Some(v) => match v.as_vec3() {
                Some(x) => Ok(x),
                None => {
                    let msg = format!("{} 的 {} 需要三维向量 x,y,z", self.scene.path(n), key);
                    self.degenerate(n, msg);
                    Err(SolveErr::Cycle(usize::MAX))
                }
            },
        }
    }

    /// 字面量类型属性（枚举、方向、材质名…）
    fn prop_literal(&mut self, n: NodeId, key: &str) -> Option<String> {
        let pv = self.scene.nodes[n].props.get(key)?.clone();
        self.literal_name(&pv.expr)
    }

    fn fail<T>(&mut self, n: NodeId, msg: String) -> Result<T, SolveErr> {
        self.degenerate(n, msg);
        Err(SolveErr::Cycle(usize::MAX))
    }

    // ====== 几何参数 → Solid ======
    fn solid_of(&mut self, n: NodeId) -> Result<Option<Solid>, SolveErr> {
        if let Some(g) = self.scene.nodes[n].geom.clone() {
            return Ok(Some(g));
        }
        let kind = self.scene.nodes[n].kind.clone();
        let path = self.scene.path(n);
        let solid = match kind.as_str() {
            "box" | "wedge" => {
                let size = self.prop_vec3(n, "size", Vec3::ONE)?;
                if !(size.x > 0.0 && size.y > 0.0 && size.z > 0.0) {
                    return self.fail(n, format!("{} 的 size 三个分量都要大于 0", path));
                }
                if kind == "box" {
                    Solid::Box { size }
                } else {
                    Solid::Wedge { size }
                }
            }
            "mesh" => {
                let size = match self.prop_value(n, "size")? {
                    None => return self.fail(n, format!("{} 缺少 size", path)),
                    Some(v) => match v.as_vec3() {
                        Some(s) => s,
                        None => return self.fail(n, format!("{} 的 size 需要三维向量 x,y,z", path)),
                    },
                };
                let pivot = self.prop_vec3(n, "pivot", Vec3::ZERO)?;
                let src = match self.prop_value(n, "src")? {
                    Some(Value::Str(s)) => s,
                    None => return self.fail(n, format!("{} 缺少 src", path)),
                    Some(_) => return self.fail(n, format!("{} 的 src 需要字符串", path)),
                };
                Solid::Mesh { size, pivot, src }
            }
            "cyl" | "cone" => {
                let r = self.prop_num(n, "r", 0.5)?;
                let r2 = if kind == "cone" {
                    self.prop_num(n, "r2", 0.0)?
                } else {
                    r
                };
                let h = self.prop_num(n, "h", 1.0)?;
                let axis = match self.prop_literal(n, "axis") {
                    None => Axis::Y,
                    Some(a) => match Axis::parse(&a) {
                        Some(ax) => ax,
                        None => return self.fail(n, format!("{} 的 axis 只能是 x / y / z", path)),
                    },
                };
                if kind == "cyl" && r <= 0.0 {
                    return self.fail(n, format!("{} 的 r 要大于 0", path));
                }
                if kind == "cone" && (r < 0.0 || r2 < 0.0 || (r == 0.0 && r2 == 0.0)) {
                    return self.fail(n, format!("{} 的 r 与 r2 不能都是 0", path));
                }
                if h <= 0.0 {
                    return self.fail(n, format!("{} 的 h 要大于 0", path));
                }
                Solid::Cyl { r1: r, r2, h, axis }
            }
            "sphere" => {
                let r = self.prop_num(n, "r", 0.5)?;
                if r <= 0.0 {
                    return self.fail(n, format!("{} 的 r 要大于 0", path));
                }
                Solid::Sphere { r }
            }
            "plate" => {
                let pts = self.prop_pairs(n, "points")?;
                let h = self.prop_num(n, "h", 0.2)?;
                if h <= 0.0 {
                    return self.fail(n, format!("{}：plate 的厚度 h 要大于 0", path));
                }
                if pts.len() < 3 {
                    return self.fail(n, format!("{} 的 points 至少要 3 个点", path));
                }
                Solid::Plate { points: pts, h }
            }
            "hull" => {
                let pts = self.prop_points_const(n, "points")?;
                Solid::Hull { points: pts }
            }
            "revolve" => {
                let prof = self.prop_pairs(n, "profile")?;
                let angle = self.prop_num(n, "angle", 360.0)?;
                let smooth = self.prop_bool(n, "smooth", false)?;
                Solid::Revolve {
                    profile: prof,
                    angle,
                    smooth,
                }
            }
            _ => return Ok(None),
        };
        self.scene.nodes[n].geom = Some(solid.clone());
        Ok(Some(solid))
    }

    fn prop_bool(&mut self, n: NodeId, key: &str, default: bool) -> Result<bool, SolveErr> {
        let pv = match self.scene.nodes[n].props.get(key).cloned() {
            Some(pv) => pv,
            None => return Ok(default),
        };
        match &pv.expr.kind {
            ExprKind::Bool(b) => Ok(*b),
            _ => {
                let msg = format!("{} 的 {} 只能是 true / false", self.scene.path(n), key);
                self.fail(n, msg)
            }
        }
    }

    /// 二元组列表 [(a,b); (c,d); …]
    fn prop_pairs(&mut self, n: NodeId, key: &str) -> Result<Vec<[f64; 2]>, SolveErr> {
        let v = match self.prop_value(n, key)? {
            None => return self.fail(n, format!("{} 缺少 {}", self.scene.path(n), key)),
            Some(v) => v,
        };
        let items = match v {
            Value::List(items) => items,
            Value::Vector(ref vv) if vv.len() == 2 => vec![v.clone()],
            _ => {
                return self.fail(
                    n,
                    format!("{} 的 {} 需要二元组列表 [(a,b); …]", self.scene.path(n), key),
                )
            }
        };
        let mut out = Vec::new();
        for it in items {
            match it {
                Value::Vector(vv) if vv.len() == 2 => out.push([vv[0], vv[1]]),
                _ => {
                    return self.fail(
                        n,
                        format!("{} 的 {} 需要二元组列表 [(a,b); …]", self.scene.path(n), key),
                    )
                }
            }
        }
        Ok(out)
    }

    /// 坐标点列表（hull）
    fn prop_points_const(&mut self, n: NodeId, key: &str) -> Result<Vec<Vec3>, SolveErr> {
        let v = match self.prop_value(n, key)? {
            None => return self.fail(n, format!("{} 缺少 {}", self.scene.path(n), key)),
            Some(v) => v,
        };
        let items = match v {
            Value::List(items) => items,
            other => vec![other],
        };
        let mut out = Vec::new();
        for it in items {
            match it.as_vec3() {
                Some(p) if !matches!(it, Value::Num(_)) => out.push(p),
                _ => {
                    return self.fail(
                        n,
                        format!("{} 的 {} 需要点列表 [(x,y,z); …]", self.scene.path(n), key),
                    )
                }
            }
        }
        Ok(out)
    }

    // ====== 局部包围盒 ======
    pub fn require_lbox(&mut self, n: NodeId) -> Aabb {
        match self.lbox(n) {
            Ok(b) => b,
            Err(_) => Aabb::ZERO,
        }
    }

    pub fn lbox(&mut self, n: NodeId) -> Result<Aabb, SolveErr> {
        if let Some(b) = self.scene.nodes[n].lbox {
            return Ok(b);
        }
        if n == ROOT {
            return Ok(Aabb::ZERO);
        }
        if self.scene.nodes[n].degenerate {
            self.scene.nodes[n].lbox = Some(Aabb::ZERO);
            return Ok(Aabb::ZERO);
        }
        if self.busy_box[n] {
            return Err(SolveErr::BoxCycle(n));
        }
        self.busy_box[n] = true;
        let res = self.compute_lbox(n);
        self.busy_box[n] = false;
        match res {
            Ok(b) => {
                self.scene.nodes[n].lbox = Some(b);
                Ok(b)
            }
            Err(e) => {
                if self.scene.nodes[n].degenerate {
                    self.scene.nodes[n].lbox = Some(Aabb::ZERO);
                    return Ok(Aabb::ZERO);
                }
                Err(e)
            }
        }
    }

    fn compute_lbox(&mut self, n: NodeId) -> Result<Aabb, SolveErr> {
        let kind = self.scene.nodes[n].kind.clone();
        let role = self.scene.nodes[n].role.clone();
        if matches!(role, Role::Normal) {
            if let Some(s) = self.solid_of(n)? {
                return Ok(s.local_box());
            }
            match kind.as_str() {
                "span" => {
                    // span 的盒由 span_local 一并求出（存入 geom）
                    self.local(n)?;
                    return Ok(self.scene.nodes[n]
                        .geom
                        .as_ref()
                        .map(|g| g.local_box())
                        .unwrap_or(Aabb::ZERO));
                }
                "sweep" | "pipe" | "line" | "bezier" | "spline" | "arc" | "helix" => {
                    // 曲线与扫掠：采样尚未实现，先用零盒
                    return Ok(Aabb::ZERO);
                }
                "group" => {
                    if let Some(env) = self.prop_value(n, "envelope")? {
                        return match env {
                            Value::Vector(v) if v.len() == 3 => {
                                if v.iter().any(|x| *x < 0.0) {
                                    return self
                                        .fail(n, format!("{} 的 envelope 不能有负尺寸", self.scene.path(n)));
                                }
                                Ok(Aabb::new(
                                    Vec3::new(-v[0] / 2.0, 0.0, -v[2] / 2.0),
                                    Vec3::new(v[0] / 2.0, v[1], v[2] / 2.0),
                                ))
                            }
                            Value::Vector(v) if v.len() == 6 => {
                                if v[0] > v[3] || v[1] > v[4] || v[2] > v[5] {
                                    return self.fail(
                                        n,
                                        format!("{} 的 envelope 最小值不能大于最大值", self.scene.path(n)),
                                    );
                                }
                                Ok(Aabb::new(
                                    Vec3::new(v[0], v[1], v[2]),
                                    Vec3::new(v[3], v[4], v[5]),
                                ))
                            }
                            _ => self.fail(
                                n,
                                format!(
                                    "{} 的 envelope 写成 w,h,d 或 x0,y0,z0,x1,y1,z1",
                                    self.scene.path(n)
                                ),
                            ),
                        };
                    }
                }
                _ => {}
            }
        }
        // 容器：子节点盒经局部变换后之并（2.6）；后置子节点不计
        let kids = self.scene.nodes[n].children.clone();
        let is_bool = matches!(kind.as_str(), "union" | "subtract" | "intersect");
        let mut acc: Option<Aabb> = None;
        for (i, c) in kids.iter().enumerate() {
            if is_bool && crate::ast::CURVE_KINDS.contains(&self.scene.nodes[*c].kind.as_str()) {
                continue;
            }
            if matches!(self.scene.nodes[*c].role, Role::Skin) {
                continue;
            }
            if kind == "subtract" && i > 0 {
                break;
            }
            let t = match self.local(*c) {
                Ok(t) => t,
                Err(SolveErr::BoxCycle(a)) if a == n => {
                    // 3.6.3：子节点依赖容器自己的盒 → 后置，不计入盒
                    self.scene.nodes[*c].late = true;
                    continue;
                }
                Err(e) => return Err(e),
            };
            let cb = match self.lbox(*c) {
                Ok(b) => b,
                Err(SolveErr::BoxCycle(a)) if a == n => {
                    self.scene.nodes[*c].late = true;
                    continue;
                }
                Err(e) => return Err(e),
            };
            let wb = cb.transformed(&t);
            acc = Some(match (acc, kind.as_str()) {
                (None, _) => wb,
                (Some(a), "intersect") => a.intersection(&wb).unwrap_or(Aabb::ZERO),
                (Some(a), _) => a.union(&wb),
            });
        }
        Ok(acc.unwrap_or(Aabb::ZERO))
    }

    // ====== 相对变换（3.1.1）======
    /// T_{B→F} 只用两条链上的局部变换计算
    pub fn rel_transform(&mut self, b: NodeId, f: NodeId) -> Result<Transform, SolveErr> {
        let cb = self.scene.chain(b);
        let cf = self.scene.chain(f);
        let mut k = 0;
        while k < cb.len() && k < cf.len() && cb[k] == cf[k] {
            k += 1;
        }
        let mut tb = Transform::IDENTITY;
        for &x in &cb[k..] {
            let l = self.local(x)?;
            tb = tb.then_inner(&l);
        }
        let mut tf = Transform::IDENTITY;
        for &x in &cf[k..] {
            let l = self.local(x)?;
            tf = tf.then_inner(&l);
        }
        Ok(tf.inverse().then_inner(&tb))
    }

    // ====== 锚点（3.2）======
    /// 锚点在 B 局部系里的 (p, ν, τ)
    fn anchor_local(&mut self, r: &RefVal) -> Result<(Vec3, Vec3, Option<Vec3>), SolveErr> {
        let b = self.lbox(r.node)?;
        let (c, h) = (b.center(), b.half());
        Ok(match &r.anchor {
            None => (c, Vec3::ZERO, None),
            Some(Anchor::Origin) => (Vec3::ZERO, Vec3::ZERO, None),
            Some(Anchor::Dir(a)) => (c + h * *a, dominant(*a), None),
            Some(Anchor::Norm(u)) => (c + h * *u, dominant(*u), None),
            Some(Anchor::CurveT(_)) | Some(Anchor::CurveNamed(_)) => (c, Vec3::ZERO, None),
            Some(Anchor::Surface { .. }) => (c, Vec3::ZERO, None),
        })
    }

    /// 锚点换到框架 F（3.2.4）；供表达式分量使用的公开版本
    pub fn anchor_in_frame(&mut self, r: &RefVal, f: NodeId) -> EResult<(Vec3, Vec3, Option<Vec3>)> {
        match self.anchor_in_frame_s(r, f) {
            Ok(x) => Ok(x),
            Err(SolveErr::Cycle(c)) => Err(EvalErr(format!("__cycle__{}", c))),
            Err(SolveErr::BoxCycle(c)) => Err(EvalErr(format!("__boxcycle__{}", c))),
        }
    }

    fn anchor_in_frame_s(&mut self, r: &RefVal, f: NodeId) -> Result<(Vec3, Vec3, Option<Vec3>), SolveErr> {
        let (p, nu, tau) = self.anchor_local(r)?;
        let t = self.rel_transform(r.node, f)?;
        let q = t.apply(p) + r.offset;
        Ok((q, t.r * nu, tau.map(|v| t.r * v)))
    }

    /// 集合的世界盒（3.2.8）
    pub fn set_box_world(&mut self, set: &[NodeId]) -> Aabb {
        let mut acc: Option<Aabb> = None;
        for &m in set {
            let b = self.require_wbox(m);
            acc = Some(match acc {
                None => b,
                Some(a) => a.union(&b),
            });
        }
        acc.unwrap_or(Aabb::ZERO)
    }

    /// 点（3.2.9）在框架 F 里：返回 (点, 法线)
    fn point_in_frame(
        &mut self,
        v: &Value,
        f: NodeId,
        n: NodeId,
        key: &str,
    ) -> Result<(Vec3, Vec3), SolveErr> {
        match v {
            Value::Vector(vv) if vv.len() == 3 => Ok((Vec3::new(vv[0], vv[1], vv[2]), Vec3::ZERO)),
            Value::Ref(r) => {
                let (q, nu, _) = self.anchor_in_frame_s(r, f)?;
                Ok((q, nu))
            }
            Value::Set(s) => {
                if s.is_empty() {
                    return self.fail(
                        n,
                        format!("{} 的 {}：集合是空的，不能当参照物", self.scene.path(n), key),
                    );
                }
                let b = self.set_box_world(s);
                let wf = self.require_world(f);
                Ok((wf.inverse().apply(b.center()), Vec3::ZERO))
            }
            other => {
                let msg = format!(
                    "{} 的 {} 需要点坐标或锚点，得到 {}",
                    self.scene.path(n),
                    key,
                    other.describe()
                );
                self.fail(n, msg)
            }
        }
    }

    pub fn top_bottom(&mut self, n: NodeId) -> (f64, f64) {
        let b = self.scene.nodes[n].tight.unwrap_or_else(|| self.require_wbox(n));
        (b.min.y, b.max.y)
    }

    /// 有效材质名（10.5.3 的 E_g；先用自有 / 祖先的 mat 字面量）
    pub fn effective_mat_name(&mut self, n: NodeId) -> Option<String> {
        let mut cur = n;
        while cur != ROOT {
            if let Some(m) = self.prop_literal(cur, "mat") {
                return Some(m);
            }
            cur = self.scene.nodes[cur].parent;
        }
        None
    }

    pub fn relation_predicate(&mut self, name: &str, _args: &[Expr], _ctx: Ctx) -> EResult<Value> {
        Err(EvalErr(format!("{}() 尚未实现", name)))
    }

    // ====== 一般规则（3.1.3）======
    fn general_local(&mut self, n: NodeId) -> Result<Transform, SolveErr> {
        let parent = self.scene.nodes[n].parent;
        let kind = self.scene.nodes[n].kind.clone();
        let path = self.scene.path(n);
        let rel = self.relation(n, parent)?;
        // 朝向
        let mut r_base = if rel.inherit { rel.r_base } else { Mat3::IDENTITY };
        if rel.inherit && r_base.determinant() < 0.0 {
            r_base = r_base * Mat3::from_diagonal(Vec3::new(-1.0, 1.0, 1.0));
        }
        let r_n = if kind == "repeat" {
            r_base
        } else {
            let turn = self.prop_num(n, "turn", 0.0)?;
            let tilt = self.prop_num(n, "tilt", 0.0)?;
            let roll = self.prop_num(n, "roll", 0.0)?;
            let r1 = r_base * math::rot_y_deg(turn);
            let head = if self.scene.nodes[n].props.contains_key("aim") {
                self.aim_matrix(n, parent, &rel, r1)?
            } else {
                r1
            };
            let mut r = head * math::rot_x_deg(tilt) * roll_matrix(roll);
            if let Some(v) = self.prop_value(n, "rot")? {
                match v {
                    Value::Vector(vv) if vv.len() == 4 => {
                        let axis = Vec3::new(vv[0], vv[1], vv[2]);
                        match math::unit(axis, 1e-9) {
                            Some(u) => r = r * math::rot_axis_deg(u, vv[3]),
                            None => return self.fail(n, format!("{} 的 rot 的轴不能是零向量", path)),
                        }
                    }
                    _ => return self.fail(n, format!("{} 的 rot 写成 (轴x,轴y,轴z,角度)", path)),
                }
            }
            r
        };
        // 定位点
        let lb = self.lbox(n)?;
        let (c, h) = (lb.center(), lb.half());
        let pi = match self.scene.nodes[n].props.get("pin").cloned() {
            Some(pv) => match self.literal_name(&pv.expr) {
                Some(s) if s == "origin" => Vec3::ZERO,
                Some(s) => match dir_vector(&s) {
                    Some(a) => c + h * a,
                    None => return self.fail(n, "pin 写成 origin、方向词或 u,v,w".to_string()),
                },
                None => {
                    let ctx = prop_ctx(&pv, Timing::Geometry);
                    match self.eval(&pv.expr, ctx) {
                        Ok(Value::Vector(v)) if v.len() == 3 => c + h * Vec3::new(v[0], v[1], v[2]),
                        _ => return self.fail(n, "pin 写成 origin、方向词或 u,v,w".to_string()),
                    }
                }
            },
            None => {
                if rel.pin_origin {
                    Vec3::ZERO
                } else {
                    match rel.default_pin {
                        Some(a) => c + h * a,
                        None => {
                            // against 自动选面：(R_n f)·ν 最小
                            let mut best = FACES[0].1;
                            let mut best_v = f64::INFINITY;
                            for (_, f) in FACES.iter() {
                                let v = (r_n * *f).dot(rel.nu);
                                if v < best_v {
                                    best_v = v;
                                    best = *f;
                                }
                            }
                            c + h * best
                        }
                    }
                }
            }
        };
        let mut t = rel.q - r_n * pi;
        // gap
        if self.scene.nodes[n].props.contains_key("gap") {
            let g = self.prop_num(n, "gap", 0.0)?;
            if rel.nu == Vec3::ZERO {
                return self.fail(
                    n,
                    format!(
                        "{}：gap 需要有法线的定位关系（on / under / against，或 at= 带面锚点）",
                        path
                    ),
                );
            }
            t += rel.nu * g;
        }
        // align
        if let Some(pv) = self.scene.nodes[n].props.get("align").cloned() {
            let faces = match align_faces(&pv.expr) {
                Some(f) => f,
                None => return self.fail(n, "align 只接受面名或 x / y / z，如 left,back".to_string()),
            };
            let target = match self.prop_value(n, "alignto")? {
                Some(v) => Some(v),
                None => rel.target.clone(),
            };
            let target =
                match target {
                    Some(t) => t,
                    None => return self.fail(
                        n,
                        format!(
                            "{}：align 需要参照（on / under / against / inside / at=对象 或 alignto=对象）",
                            path
                        ),
                    ),
                };
            for f in faces {
                let cur = Transform::new(r_n, t);
                match f.as_str() {
                    "x" | "y" | "z" => {
                        let k = match f.as_str() {
                            "x" => 0,
                            "y" => 1,
                            _ => 2,
                        };
                        let (qc, _) = self.target_anchor(&target, Vec3::ZERO, parent, n)?;
                        let pc = cur.apply(c);
                        let mut tt = t;
                        tt[k] += qc[k] - pc[k];
                        t = tt;
                    }
                    _ => {
                        let af = dir_vector(&f).unwrap();
                        let (qf, nuf) = self.target_anchor(&target, af, parent, n)?;
                        let mut best = FACES[0].1;
                        let mut best_v = f64::NEG_INFINITY;
                        for (_, g) in FACES.iter() {
                            let v = (r_n * *g).dot(nuf);
                            if v > best_v {
                                best_v = v;
                                best = *g;
                            }
                        }
                        let pg = cur.apply(c + h * best);
                        t += nuf * (qf - pg).dot(nuf);
                    }
                }
            }
        }
        // offset
        let o = self.prop_vec3(n, "offset", Vec3::ZERO)?;
        t += o;
        Ok(Transform::new(r_n, t))
    }

    /// 对齐目标上的锚点（面向量 a；a = 0 为盒心）换到 P：返回 (q, ν)
    fn target_anchor(
        &mut self,
        target: &Value,
        a: Vec3,
        p: NodeId,
        n: NodeId,
    ) -> Result<(Vec3, Vec3), SolveErr> {
        match target {
            Value::Ref(r) => {
                let rr = RefVal {
                    node: r.node,
                    anchor: Some(Anchor::Dir(a)),
                    offset: Vec3::ZERO,
                };
                let (q, nu, _) = self.anchor_in_frame_s(&rr, p)?;
                Ok((q, if a == Vec3::ZERO { Vec3::ZERO } else { nu }))
            }
            Value::Set(s) => {
                if s.is_empty() {
                    return self.fail(n, format!("{}：集合是空的，不能当参照物", self.scene.path(n)));
                }
                let b = self.set_box_world(s);
                let wp = self.require_world(p);
                let inv = wp.inverse();
                Ok((inv.apply(b.anchor(a)), inv.r * a))
            }
            other => {
                let msg = format!(
                    "{} 的对齐目标需要引用一个对象，得到 {}",
                    self.scene.path(n),
                    other.describe()
                );
                self.fail(n, msg)
            }
        }
    }

    /// 主定位关系（3.3）
    fn relation(&mut self, n: NodeId, p: NodeId) -> Result<Relation, SolveErr> {
        let path = self.scene.path(n);
        let keys = ["on", "under", "against", "inside", "at"];
        let key = keys
            .iter()
            .find(|k| self.scene.nodes[n].props.contains_key(**k))
            .copied();
        let none = Relation {
            q: Vec3::ZERO,
            nu: Vec3::ZERO,
            default_pin: None,
            pin_origin: true,
            inherit: false,
            r_base: Mat3::IDENTITY,
            target: None,
        };
        let key = match key {
            None => return Ok(none),
            Some(k) => k,
        };
        let v = match self.prop_value(n, key)? {
            Some(v) => v,
            None => return Ok(none),
        };
        if key == "at" {
            let (q, nu) = self.point_in_frame(&v, p, n, "at")?;
            let target = match &v {
                Value::Ref(_) => Some(v.clone()),
                _ => None,
            };
            return Ok(Relation {
                q,
                nu,
                default_pin: Some(Vec3::NEG_Y),
                pin_origin: false,
                inherit: false,
                r_base: Mat3::IDENTITY,
                target,
            });
        }
        // 参照物：引用或非空集合
        let (r_bp, q, anchor_nu, is_set) = match &v {
            Value::Ref(r) => {
                let default_anchor = match key {
                    "on" => Some(Anchor::Dir(Vec3::Y)),
                    "under" => Some(Anchor::Dir(Vec3::NEG_Y)),
                    "inside" => Some(Anchor::Dir(Vec3::ZERO)),
                    _ => None,
                };
                let anchor = match (&r.anchor, default_anchor) {
                    (Some(a), _) => Some(a.clone()),
                    (None, Some(d)) => Some(d),
                    (None, None) => {
                        return self.fail(
                            n,
                            format!("{}：against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]", path),
                        );
                    }
                };
                let rr = RefVal {
                    node: r.node,
                    anchor,
                    offset: r.offset,
                };
                if r.node == n || self.scene.is_ancestor(r.node, n) || self.scene.is_ancestor(n, r.node) {
                    return self.fail(n, format!("{}：不能相对包含自己的对象定位", path));
                }
                let (q, nu, _) = self.anchor_in_frame_s(&rr, p)?;
                let t = self.rel_transform(r.node, p)?;
                (t.r, q, nu, false)
            }
            Value::Set(s) => {
                if s.is_empty() {
                    return self.fail(n, format!("{} 的 {}：集合是空的，不能当参照物", path, key));
                }
                if s.contains(&n)
                    || s.iter()
                        .any(|&m| self.scene.is_ancestor(m, n) || self.scene.is_ancestor(n, m))
                {
                    return self.fail(n, format!("{} 的 {}：集合里包含它自己或它的上下级", path, key));
                }
                let b = self.set_box_world(s);
                let wp = self.require_world(p);
                let inv = wp.inverse();
                let a = match key {
                    "on" => Vec3::Y,
                    "under" => Vec3::NEG_Y,
                    "inside" => Vec3::ZERO,
                    _ => {
                        return self.fail(
                            n,
                            format!("{}：against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]", path),
                        )
                    }
                };
                (inv.r, inv.apply(b.anchor(a)), inv.r * dominant(a), true)
            }
            other => {
                let msg = format!("{} 的 {} 需要引用一个对象，得到 {}", path, key, other.describe());
                return self.fail(n, msg);
            }
        };
        let _ = is_set;
        let (nu, default_pin) = match key {
            "on" => (r_bp * Vec3::Y, Some(Vec3::NEG_Y)),
            "under" => (-(r_bp * Vec3::Y), Some(Vec3::Y)),
            "inside" => (Vec3::ZERO, Some(Vec3::ZERO)),
            _ => {
                if anchor_nu == Vec3::ZERO {
                    return self.fail(
                        n,
                        format!("{}：against 要指明贴哪个面，如 B.left 或 B.side[90,0.5]", path),
                    );
                }
                (anchor_nu, None)
            }
        };
        Ok(Relation {
            q,
            nu,
            default_pin,
            pin_origin: false,
            inherit: true,
            r_base: r_bp,
            target: Some(v),
        })
    }

    /// aim（3.5.4）
    fn aim_matrix(&mut self, n: NodeId, p: NodeId, rel: &Relation, r1: Mat3) -> Result<Mat3, SolveErr> {
        let path = self.scene.path(n);
        let g = match self.prop_value(n, "aim")? {
            Some(v) => self.point_in_frame(&v, p, n, "aim")?.0,
            None => unreachable!(),
        };
        let has_rel = !rel.pin_origin;
        let has_pin = self.scene.nodes[n].props.contains_key("pin");
        let s = if has_rel || has_pin {
            rel.q
        } else {
            self.prop_vec3(n, "offset", Vec3::ZERO)?
        };
        let d = g - s;
        let dh = match math::unit(d, 1e-9) {
            Some(u) => u,
            None => return self.fail(n, format!("{}：aim 的目标与自身重合", path)),
        };
        let a_word = self
            .prop_literal(n, "aimwith")
            .unwrap_or_else(|| "front".to_string());
        let a = match dir_vector(&a_word) {
            Some(v) if v != Vec3::ZERO => v,
            _ => {
                return self.fail(
                    n,
                    format!("{} 的 aimwith 需要方向词，如 front / top / right", path),
                )
            }
        };
        let ah = math::unit(a, 0.0).unwrap();
        let b = if ah.y.abs() < 0.99 { Vec3::Y } else { Vec3::Z };
        let bh = math::unit(b - ah * b.dot(ah), 0.0).unwrap();
        let e0 = r1 * b;
        let e1 = e0 - dh * e0.dot(dh);
        let eh = match math::unit(e1, 1e-6) {
            Some(u) => u,
            None => s3d_geom::solid::perp(dh),
        };
        let m1 = Mat3::from_cols(dh, eh, dh.cross(eh));
        let m2 = Mat3::from_cols(ah, bh, ah.cross(bh));
        Ok(m1 * m2.transpose())
    }

    // ====== span（3.10、2.2.8）======
    fn span_local(&mut self, n: NodeId) -> Result<Transform, SolveErr> {
        let p = self.scene.nodes[n].parent;
        let path = self.scene.path(n);
        let from_v = match self.prop_value(n, "from")? {
            Some(v) => v,
            None => return self.fail(n, format!("{} 缺少 from", path)),
        };
        let (a, _) = self.point_in_frame(&from_v, p, n, "from")?;
        let has_to = self.scene.nodes[n].props.contains_key("to");
        let has_drop = self.scene.nodes[n].props.contains_key("drop");
        if has_to == has_drop {
            return self.fail(n, format!("{}：to 与 drop 要恰好写一个", path));
        }
        let b = if has_to {
            let v = self.prop_value(n, "to")?.unwrap();
            self.point_in_frame(&v, p, n, "to")?.0
        } else {
            let v = self.prop_value(n, "drop")?.unwrap();
            let r = match v {
                Value::Ref(r) => r,
                other => {
                    return self.fail(
                        n,
                        format!("{} 的 drop 需要引用一个对象，得到 {}", path, other.describe()),
                    )
                }
            };
            let rr = RefVal {
                node: r.node,
                anchor: Some(r.anchor.clone().unwrap_or(Anchor::Dir(Vec3::Y))),
                offset: r.offset,
            };
            let (q, mut nu, _) = self.anchor_in_frame_s(&rr, p)?;
            if nu == Vec3::ZERO {
                let t = self.rel_transform(r.node, p)?;
                nu = t.r * Vec3::Y;
            }
            let dir = match self.prop_value(n, "dir")? {
                None => Vec3::NEG_Y,
                Some(v) => match v.as_vec3() {
                    Some(d) => match math::unit(d, 1e-9) {
                        Some(u) => u,
                        None => return self.fail(n, format!("{} 的 dir 不能是零向量", path)),
                    },
                    None => return self.fail(n, format!("{} 的 dir 需要方向", path)),
                },
            };
            let denom = dir.dot(nu);
            if denom.abs() < 1e-9 {
                return self.fail(n, format!("{}：drop 的方向与目标面平行", path));
            }
            let s = (q - a).dot(nu) / denom;
            if s < 0.0 {
                return self.fail(n, format!("{}：drop 的目标在反方向", path));
            }
            a + dir * s
        };
        let steps = self.scene.nodes[n].props.contains_key("steps");
        let is_rod = self.scene.nodes[n].props.contains_key("r");
        if steps && is_rod {
            return self.fail(n, format!("{}：steps 与 r 不能同时写", path));
        }
        let (t, l, hsp) = match s3d_geom::solid::span_frame(a, b, steps) {
            Some(x) => x,
            None => return self.fail(n, format!("{}：from 与 to 重合", path)),
        };
        let solid = if is_rod {
            if self.scene.nodes[n].props.contains_key("section")
                || self.scene.nodes[n].props.contains_key("pin")
            {
                return self.fail(n, format!("{}：圆杆没有 section / pin", path));
            }
            let r = self.prop_num(n, "r", 0.3)?;
            if r <= 0.0 {
                return self.fail(n, format!("{} 的 r 要大于 0", path));
            }
            Solid::Rod { l, r }
        } else if steps {
            if self.scene.nodes[n].props.contains_key("pin") {
                return self.fail(n, format!("{}：台阶没有 pin", path));
            }
            let sec = self.section(n, [1.2, 0.2])?;
            let ns = self.prop_num(n, "steps", 1.0)?;
            if (ns - ns.round()).abs() > 1e-9 || ns < 1.0 {
                return self.fail(n, format!("{} 的 steps 需要正整数", path));
            }
            Solid::Steps {
                h_span: hsp,
                dy: b.y - a.y,
                n: ns.round() as u32,
                w: sec[0],
                t: sec[1],
            }
        } else {
            let sec = self.section(n, [1.0, 0.3])?;
            let dy = match self.prop_literal(n, "pin").as_deref() {
                None | Some("center") => 0.0,
                Some("bottom") => sec[1] / 2.0,
                Some("top") => -sec[1] / 2.0,
                Some(_) => return self.fail(n, format!("{} 的 pin 只能是 bottom / top / center", path)),
            };
            Solid::Beam {
                l,
                w: sec[0],
                t: sec[1],
                dy,
            }
        };
        self.scene.nodes[n].geom = Some(solid);
        Ok(t)
    }

    fn section(&mut self, n: NodeId, default: [f64; 2]) -> Result<[f64; 2], SolveErr> {
        match self.prop_value(n, "section")? {
            None => Ok(default),
            Some(Value::Vector(v)) if v.len() == 2 && v[0] > 0.0 && v[1] > 0.0 => Ok([v[0], v[1]]),
            Some(_) => {
                let msg = format!("{} 的 section 需要两个数：宽,厚", self.scene.path(n));
                self.fail(n, msg)
            }
        }
    }

    // ====== 阵列实例（3.8；先实现网格阵列）======
    fn instance_local(&mut self, n: NodeId) -> Result<Transform, SolveErr> {
        let (i, j, k, idx) = match self.scene.nodes[n].role {
            Role::Instance { i, j, k, idx } => (i, j, k, idx),
            _ => unreachable!(),
        };
        let rep = self.scene.nodes[n].parent;
        let step = self.prop_vec3(rep, "step", Vec3::ZERO)?;
        let center = self.prop_bool(rep, "center", false)?;
        let turn = self.prop_num(rep, "turn", 0.0)?;
        // 维度：从兄弟数推回 n（3.8.3）；网格 vs 线性由 n 的形态决定，这里按下标公式
        let (ni, nj, nk) = self.repeat_dims(rep);
        let grid = !(nj == 1 && nk == 1);
        let mut pos = if grid {
            Vec3::new(step.x * i as f64, step.y * j as f64, step.z * k as f64)
        } else {
            step * idx as f64
        };
        if center {
            let c = if grid {
                Vec3::new(
                    step.x * (ni as f64 - 1.0) / 2.0,
                    step.y * (nj as f64 - 1.0) / 2.0,
                    step.z * (nk as f64 - 1.0) / 2.0,
                )
            } else {
                step * ((ni as f64 - 1.0) / 2.0)
            };
            pos -= c;
        }
        let r = math::rot_y_deg(turn * idx as f64);
        Ok(Transform::new(r, pos))
    }

    fn repeat_dims(&self, rep: NodeId) -> (u32, u32, u32) {
        let mut ni = 0;
        let mut nj = 0;
        let mut nk = 0;
        for &c in &self.scene.nodes[rep].children {
            if let Role::Instance { i, j, k, .. } = self.scene.nodes[c].role {
                ni = ni.max(i + 1);
                nj = nj.max(j + 1);
                nk = nk.max(k + 1);
            }
        }
        (ni.max(1), nj.max(1), nk.max(1))
    }
}

/// 主法线（3.2.3）
pub fn dominant(a: Vec3) -> Vec3 {
    let m = a.x.abs().max(a.y.abs()).max(a.z.abs());
    if m < 1e-9 {
        return Vec3::ZERO;
    }
    if a.y.abs() == m {
        Vec3::new(0.0, a.y.signum(), 0.0)
    } else if a.x.abs() == m {
        Vec3::new(a.x.signum(), 0.0, 0.0)
    } else {
        Vec3::new(0.0, 0.0, a.z.signum())
    }
}

/// roll：绕局部 +z 的左手旋转 = R_z(−θ)（3.5.2）
pub fn roll_matrix(theta: f64) -> Mat3 {
    let c = math::cos_deg(theta);
    let s = math::sin_deg(theta);
    Mat3::from_cols(Vec3::new(c, -s, 0.0), Vec3::new(s, c, 0.0), Vec3::Z)
}

/// align 的面名列表（1.4.1）
fn align_faces(e: &Expr) -> Option<Vec<String>> {
    let one = |x: &Expr| -> Option<String> {
        match &x.kind {
            ExprKind::Ident(s)
                if matches!(
                    s.as_str(),
                    "left" | "right" | "front" | "back" | "top" | "bottom" | "x" | "y" | "z"
                ) =>
            {
                Some(s.clone())
            }
            _ => None,
        }
    };
    match &e.kind {
        ExprKind::Tuple(items) => items.iter().map(one).collect(),
        _ => one(e).map(|s| vec![s]),
    }
}
