//! 空间事实（spec/06）：对象、候选对、分离量、五种关系、输出、关系索引与关系谓词。

use crate::ast::{Expr, SOLID_KINDS};
use crate::compile::{Compiler, Ctx};
use crate::eval::{EResult, EvalErr};
use crate::scene::{NodeId, Role, ROOT};
use crate::value::Value;
use s3d_geom::convex::{self, Part, Shape};
use s3d_geom::{Aabb, Vec3};
use serde_json::{json, Value as J};

const EPS_DIR: f64 = 1e-6;
const EPS_LEN: f64 = 1e-9;
const KAPPA_IN: f64 = 0.9;

/// 关系索引的七个表（6.7.1）
pub const REL_NAMES: [&str; 7] = [
    "on",
    "supports",
    "against",
    "inside",
    "contains",
    "touching",
    "intersects",
];

#[derive(Clone, Debug)]
pub struct Fact {
    pub rel: String,
    pub a: NodeId,
    pub b: NodeId,
    pub face: Option<&'static str>,
    pub gap: f64,
    pub approx: bool,
}

/// 事实对象（6.2.2）
struct Obj {
    node: NodeId,
    /// 世界系凸块
    parts: Vec<Part>,
    part_boxes: Vec<Aabb>,
    omega: Aabb,
    approx: bool,
    boxy: bool,
    vol: f64,
    lbox: Aabb,
    world: s3d_geom::Transform,
    /// 节点顺序序号
    order: usize,
}

fn ov(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    a1.min(b1) - a0.max(b0)
}

fn ov_k(a: &Aabb, b: &Aabb, k: usize) -> f64 {
    ov(a.min[k], a.max[k], b.min[k], b.max[k])
}

impl Compiler {
    fn has_bool_ancestor(&self, n: NodeId) -> bool {
        let mut p = self.scene.nodes[n].parent;
        while p != ROOT {
            if matches!(
                self.scene.nodes[p].kind.as_str(),
                "union" | "subtract" | "intersect"
            ) {
                return true;
            }
            p = self.scene.nodes[p].parent;
        }
        false
    }

    /// 6.2.1 的对象列表（按节点顺序）
    fn fact_objects(&mut self) -> Vec<Obj> {
        let order = self.scene.order();
        let mut out = Vec::new();
        for (idx, &n) in order.iter().enumerate() {
            let node = &self.scene.nodes[n];
            let kind = node.kind.as_str();
            let is_bool = matches!(kind, "union" | "subtract" | "intersect");
            if !(SOLID_KINDS.contains(&kind) || is_bool) {
                continue;
            }
            if node.degenerate
                || matches!(node.role, Role::Skin)
                || self.scene.in_skin(n)
                || self.has_bool_ancestor(n)
            {
                continue;
            }
            let lbox = match node.lbox {
                Some(b) => b,
                None => continue,
            };
            let size = lbox.size();
            if size.x + size.y + size.z <= EPS_LEN {
                continue;
            }
            let world = match node.world {
                Some(w) => w,
                None => continue,
            };
            let (local_parts, boxy) = if is_bool {
                (
                    vec![Part {
                        shape: convex::box_shape(&lbox),
                        approx: true,
                    }],
                    true,
                )
            } else {
                let geom = match &node.geom {
                    Some(g) => g,
                    None => continue,
                };
                let boxy = matches!(
                    geom,
                    s3d_geom::Solid::Box { .. }
                        | s3d_geom::Solid::Wedge { .. }
                        | s3d_geom::Solid::Mesh { .. }
                        | s3d_geom::Solid::Beam { .. }
                );
                (convex::parts_of(geom), boxy)
            };
            if local_parts.is_empty() {
                continue;
            }
            let parts: Vec<Part> = local_parts
                .iter()
                .map(|p| Part {
                    shape: p.shape.transformed(&world),
                    approx: p.approx,
                })
                .collect();
            let part_boxes: Vec<Aabb> = parts.iter().map(|p| p.shape.bbox()).collect();
            let mut omega = part_boxes[0];
            for b in &part_boxes[1..] {
                omega = omega.union(b);
            }
            let approx = parts.iter().any(|p| p.approx);
            out.push(Obj {
                node: n,
                parts,
                part_boxes,
                omega,
                approx,
                boxy,
                vol: lbox.volume(),
                lbox,
                world,
                order: idx,
            });
        }
        out
    }

    /// 推导全部事实并建关系索引（阶段 7）
    pub fn derive_facts(&mut self) {
        if self.facts_done {
            return;
        }
        self.facts_done = true;
        self.rel_index = vec![vec![Vec::new(); self.scene.nodes.len()]; 7];
        if self.facts_skipped {
            return;
        }
        let eps = 0.02 * self.metre;
        let objs = self.fact_objects();
        // 扫描序：(Ω.min.x, 节点序)
        let mut scan: Vec<usize> = (0..objs.len()).collect();
        scan.sort_by(|&a, &b| {
            objs[a]
                .omega
                .min
                .x
                .partial_cmp(&objs[b].omega.min.x)
                .unwrap()
                .then(objs[a].order.cmp(&objs[b].order))
        });
        let mut facts: Vec<Fact> = Vec::new();
        // 活动集：按扫描序追加，x 上已远离的剔除
        let mut active: Vec<usize> = Vec::new();
        for &p in &scan {
            let op = &objs[p];
            active.retain(|&q| objs[q].omega.max.x >= op.omega.min.x - eps);
            for &q in &active {
                let oq = &objs[q];
                if ov_k(&op.omega, &oq.omega, 0) < -eps
                    || ov_k(&op.omega, &oq.omega, 1) < -eps
                    || ov_k(&op.omega, &oq.omega, 2) < -eps
                {
                    continue;
                }
                if self.scene.is_ancestor(op.node, oq.node) || self.scene.is_ancestor(oq.node, op.node) {
                    continue;
                }
                if let Some(f) = self.judge_pair(op, oq, eps) {
                    facts.push(f);
                }
            }
            active.push(p);
        }
        // 关系索引（6.7.1）
        for f in &facts {
            self.index_fact(f);
        }
        self.facts = facts;
    }

    fn index_fact(&mut self, f: &Fact) {
        let ca = self.scene.chain(f.a);
        let cb = self.scene.chain(f.b);
        let xs: Vec<NodeId> = ca.iter().copied().filter(|x| !cb.contains(x)).collect();
        let ys: Vec<NodeId> = cb.iter().copied().filter(|y| !ca.contains(y)).collect();
        let add = |idx: &mut Vec<Vec<Vec<NodeId>>>, r: usize, x: NodeId, y: NodeId| {
            if !idx[r][x].contains(&y) {
                idx[r][x].push(y);
            }
        };
        for &x in &xs {
            for &y in &ys {
                match f.rel.as_str() {
                    "on" => {
                        add(&mut self.rel_index, 0, x, y);
                        add(&mut self.rel_index, 1, y, x);
                        add(&mut self.rel_index, 5, x, y);
                        add(&mut self.rel_index, 5, y, x);
                    }
                    "against" => {
                        add(&mut self.rel_index, 2, x, y);
                        add(&mut self.rel_index, 2, y, x);
                        add(&mut self.rel_index, 5, x, y);
                        add(&mut self.rel_index, 5, y, x);
                    }
                    "inside" => {
                        add(&mut self.rel_index, 3, x, y);
                        add(&mut self.rel_index, 4, y, x);
                    }
                    "touching" => {
                        add(&mut self.rel_index, 5, x, y);
                        add(&mut self.rel_index, 5, y, x);
                    }
                    _ => {
                        add(&mut self.rel_index, 6, x, y);
                        add(&mut self.rel_index, 6, y, x);
                        add(&mut self.rel_index, 5, x, y);
                        add(&mut self.rel_index, 5, y, x);
                    }
                }
            }
        }
    }

    /// 对象间隙与接触块对（6.4.3）
    fn pair_gap(p: &Obj, q: &Obj, eps: f64) -> (f64, Vec<(usize, usize)>) {
        let mut best = f64::INFINITY;
        let mut touch = Vec::new();
        for (i, pb) in p.part_boxes.iter().enumerate() {
            for (j, qb) in q.part_boxes.iter().enumerate() {
                if ov_k(pb, qb, 0) < -eps || ov_k(pb, qb, 1) < -eps || ov_k(pb, qb, 2) < -eps {
                    continue;
                }
                let g = convex::gap(&p.parts[i].shape, &q.parts[j].shape);
                if g < best {
                    best = g;
                }
                if g <= eps {
                    touch.push((i, j));
                }
            }
        }
        (best, touch)
    }

    /// 6.5.1：一个候选对的事实
    fn judge_pair(&self, p: &Obj, q: &Obj, eps: f64) -> Option<Fact> {
        let (gap, touch_pq) = Self::pair_gap(p, q, eps);
        if gap > eps {
            return None;
        }
        let approx = p.approx || q.approx;
        if let Some(g) = Self::on_test(p, q, &touch_pq, eps) {
            return Some(Fact {
                rel: "on".into(),
                a: p.node,
                b: q.node,
                face: None,
                gap: g,
                approx,
            });
        }
        let touch_qp: Vec<(usize, usize)> = touch_pq.iter().map(|&(i, j)| (j, i)).collect();
        if let Some(g) = Self::on_test(q, p, &touch_qp, eps) {
            return Some(Fact {
                rel: "on".into(),
                a: q.node,
                b: p.node,
                face: None,
                gap: g,
                approx,
            });
        }
        let (a, b) = if p.vol <= q.vol { (p, q) } else { (q, p) };
        if let Some(face) = Self::against_test(a, b, eps) {
            return Some(Fact {
                rel: "against".into(),
                a: a.node,
                b: b.node,
                face: Some(face),
                gap,
                approx,
            });
        }
        if Self::inside_test(a, b, eps) {
            return Some(Fact {
                rel: "inside".into(),
                a: a.node,
                b: b.node,
                face: None,
                gap,
                approx,
            });
        }
        Some(Fact {
            rel: if gap >= -eps {
                "touching".into()
            } else {
                "intersects".into()
            },
            a: a.node,
            b: b.node,
            face: None,
            gap,
            approx,
        })
    }

    /// 6.5.2：A on B
    fn on_test(a: &Obj, b: &Obj, touch: &[(usize, usize)], eps: f64) -> Option<f64> {
        let r_ab = b.world.r.transpose() * a.world.r;
        if ((r_ab.y_axis.y).abs() - 1.0).abs() > EPS_DIR {
            return None;
        }
        let u = b.world.r * Vec3::Y;
        let ex = b.world.r * Vec3::X;
        let ez = b.world.r * Vec3::Z;
        let mut beta = f64::INFINITY;
        for part in &a.parts {
            beta = beta.min(part.shape.proj(u).0);
        }
        for &(i, j) in touch {
            let tau = b.parts[j].shape.proj(u).1;
            if (beta - tau).abs() >= eps {
                continue;
            }
            let (ax0, ax1) = a.parts[i].shape.proj(ex);
            let (bx0, bx1) = b.parts[j].shape.proj(ex);
            let (az0, az1) = a.parts[i].shape.proj(ez);
            let (bz0, bz1) = b.parts[j].shape.proj(ez);
            if ov(ax0, ax1, bx0, bx1) > eps && ov(az0, az1, bz0, bz1) > eps {
                return Some(beta - tau);
            }
        }
        None
    }

    /// 6.5.3：A against B.面
    fn against_test(a: &Obj, b: &Obj, eps: f64) -> Option<&'static str> {
        if !(a.boxy && b.boxy) {
            return None;
        }
        let r_ab = b.world.r.transpose() * a.world.r;
        for col in [r_ab.x_axis, r_ab.y_axis, r_ab.z_axis] {
            for r in [col.x, col.y, col.z] {
                if !(r.abs() <= EPS_DIR || (r.abs() - 1.0).abs() <= EPS_DIR) {
                    return None;
                }
            }
        }
        let t_ab = b.world.inverse().then_inner(&a.world);
        let abox = a.lbox.transformed(&t_ab);
        let bbox = b.lbox;
        let ox = ov_k(&abox, &bbox, 0);
        let oy = ov_k(&abox, &bbox, 1);
        let oz = ov_k(&abox, &bbox, 2);
        if oy <= eps {
            return None;
        }
        if (abox.max.x - bbox.min.x).abs() < eps && oz > eps {
            return Some("left");
        }
        if (abox.min.x - bbox.max.x).abs() < eps && oz > eps {
            return Some("right");
        }
        if (abox.max.z - bbox.min.z).abs() < eps && ox > eps {
            return Some("back");
        }
        if (abox.min.z - bbox.max.z).abs() < eps && ox > eps {
            return Some("front");
        }
        None
    }

    /// 6.5.4：A inside B
    fn inside_test(a: &Obj, b: &Obj, eps: f64) -> bool {
        if b.parts.len() != 1 {
            return false;
        }
        if a.vol >= KAPPA_IN * b.vol {
            return false;
        }
        match &b.parts[0].shape {
            Shape::Sphere { c, r } => {
                for part in &a.parts {
                    match &part.shape {
                        Shape::Sphere { c: c2, r: r2 } => {
                            if s3d_geom::math::length(*c2 - *c) + r2 > r + eps {
                                return false;
                            }
                        }
                        Shape::Poly { verts, .. } => {
                            for v in verts {
                                if s3d_geom::math::length(*v - *c) > r + eps {
                                    return false;
                                }
                            }
                        }
                    }
                }
                true
            }
            Shape::Poly { normals, .. } => {
                for nu in normals {
                    let (l, h) = b.parts[0].shape.proj(*nu);
                    for part in &a.parts {
                        let (l2, h2) = part.shape.proj(*nu);
                        if h2 > h + eps || l2 < l - eps {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    /// 调试：某个节点的世界凸块（顶点列表）
    pub fn debug_parts(&mut self, path: &str) -> Vec<Vec<Vec3>> {
        let objs = self.fact_objects();
        for o in objs {
            if self.scene.path(o.node) == path {
                return o
                    .parts
                    .iter()
                    .map(|p| match &p.shape {
                        Shape::Poly {
                            verts,
                            normals,
                            edges,
                        } => {
                            let mut v = verts.clone();
                            v.push(Vec3::splat(f64::NAN));
                            v.extend(normals.iter().copied());
                            v.push(Vec3::splat(f64::NAN));
                            v.extend(edges.iter().copied());
                            v
                        }
                        Shape::Sphere { c, r } => vec![*c, Vec3::splat(*r)],
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// facts / factsSkipped / factProviders 三个顶层字段（6.6）
    pub fn facts_json(&mut self) -> Vec<(String, J)> {
        self.derive_facts();
        let eps = 0.02 * self.metre;
        let mut arr = Vec::with_capacity(self.facts.len());
        for f in &self.facts {
            let a = self.scene.path(f.a);
            let b = self.scene.path(f.b);
            let text = match f.face {
                Some(face) => format!("{} against {}.{}", a, b, face),
                None => format!("{} {} {}", a, f.rel, b),
            };
            arr.push(json!({
                "rel": f.rel,
                "a": a,
                "b": b,
                "text": text,
                "gap": f.gap,
                "eps": eps,
                "approx": f.approx,
                "provider": "contact",
            }));
        }
        vec![
            ("facts".to_string(), J::Array(arr)),
            ("factsSkipped".to_string(), json!(self.facts_skipped)),
            ("factProviders".to_string(), json!(["contact"])),
        ]
    }

    /// 关系谓词（6.7.2）：rel(S, k)
    pub fn relation_predicate(&mut self, name: &str, args: &[Expr], ctx: Ctx) -> EResult<Value> {
        self.derive_facts();
        let ri = match REL_NAMES.iter().position(|r| *r == name) {
            Some(i) => i,
            None => return Err(EvalErr(format!("未知的函数 {}", name))),
        };
        if args.is_empty() || args.len() > 2 {
            return Err(EvalErr(format!("{}() 需要参照集合，如 {}(ground)", name, name)));
        }
        let sv = self.eval(&args[0], ctx)?;
        let s = match self.to_set(sv) {
            Ok(s) => s,
            Err(_) => return Err(EvalErr(format!("{}() 需要参照集合，如 {}(ground)", name, name))),
        };
        let k: usize = if args.len() == 2 {
            match self.eval(&args[1], ctx)? {
                Value::Num(x) => {
                    let r = x.round();
                    if (x - r).abs() <= 1e-9 && r >= 1.0 {
                        r as usize
                    } else {
                        return Err(EvalErr(format!("{}() 的第二个参数需要不小于 1 的整数", name)));
                    }
                }
                _ => return Err(EvalErr(format!("{}() 的第二个参数需要不小于 1 的整数", name))),
            }
        } else {
            1
        };
        if self.facts_skipped {
            self.facts_warn_stmt(name);
        }
        let mut out = Vec::new();
        for x in 1..self.scene.nodes.len() {
            if self.scene.nodes[x].dropped || self.scene.in_skin(x) {
                continue;
            }
            let cnt = self.rel_index[ri][x].iter().filter(|y| s.contains(y)).count();
            if cnt >= k {
                out.push(x);
            }
        }
        Ok(Value::Set(self.sorted_nodes(out)))
    }

    /// 事实未推导时的警告（6.9）：每条语句一条，由语义层在语句边界去重
    fn facts_warn_stmt(&mut self, name: &str) {
        self.pending_facts_warning = Some(format!("空间事实未推导，{}() 为空集", name));
    }
}
