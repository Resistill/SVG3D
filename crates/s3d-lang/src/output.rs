//! 编译输出的 JSON 形状（各章的"输出形状"小节：2.1.11、3.1.6、6.6、7.x、8.4、10.6）。

use crate::ast::{StmtKind, CURVE_KINDS, SOLID_KINDS};
use crate::compile::{Compiled, Compiler};
use crate::material::MaterialValue;
use crate::modify::expr_text;
use crate::scene::{NodeId, Role, ROOT};
use crate::value::Value;
use s3d_geom::{Solid, Transform, Vec3};
use serde_json::{json, Map, Value as J};

fn vec3_json(v: Vec3) -> J {
    json!([v.x, v.y, v.z])
}

fn box_json(b: Option<s3d_geom::Aabb>) -> J {
    match b {
        Some(b) => json!({ "min": vec3_json(b.min), "max": vec3_json(b.max) }),
        None => J::Null,
    }
}

fn world_json(w: &Transform) -> J {
    json!({
        "R": [[w.r.x_axis.x, w.r.y_axis.x, w.r.z_axis.x], [w.r.x_axis.y, w.r.y_axis.y, w.r.z_axis.y], [w.r.x_axis.z, w.r.y_axis.z, w.r.z_axis.z]],
        "t": vec3_json(w.t),
    })
}

/// 元数据值到 JSON（8.4.2）
pub fn value_json(v: &Value) -> J {
    match v {
        Value::Num(n) => num_json(*n),
        Value::Bool(b) => json!(b),
        Value::Str(s) => json!(s),
        Value::Vector(xs) => J::Array(xs.iter().map(|x| num_json(*x)).collect()),
        Value::List(items) => J::Array(items.iter().map(value_json).collect()),
        _ => J::Null,
    }
}

fn num_json(n: f64) -> J {
    if n == 0.0 {
        return json!(0);
    }
    if n.fract() == 0.0 && n.abs() < 9007199254740992.0 {
        return json!(n as i64);
    }
    json!(n)
}

pub fn meta_json_pub(m: &indexmap::IndexMap<String, Value>) -> J {
    meta_json(m)
}

fn meta_json(m: &indexmap::IndexMap<String, Value>) -> J {
    let mut out = Map::new();
    for (k, v) in m {
        out.insert(k.clone(), value_json(v));
    }
    J::Object(out)
}

fn material_json(m: &MaterialValue) -> J {
    json!({ "name": m.name, "color": m.color, "metal": m.metal, "rough": m.rough, "opacity": m.opacity })
}

fn geom_json(s: &Solid) -> J {
    match s {
        Solid::Box { size } => json!({ "type": "box", "w": size.x, "h": size.y, "d": size.z }),
        Solid::Wedge { size } => json!({ "type": "wedge", "w": size.x, "h": size.y, "d": size.z }),
        Solid::Cyl { r1, r2, h, axis } => {
            json!({ "type": "cyl", "r1": r1, "r2": r2, "h": h, "axis": axis.name() })
        }
        Solid::Sphere { r } => json!({ "type": "sphere", "r": r }),
        Solid::Plate { points, h } => json!({ "type": "plate", "pts": points, "h": h }),
        Solid::Hull { points } => {
            let pts: Vec<J> = points.iter().map(|p| vec3_json(*p)).collect();
            json!({ "type": "hull", "pts": pts, "faces": [] })
        }
        Solid::Revolve { profile, angle, .. } => {
            json!({ "type": "revolve", "prof": profile, "angle": angle })
        }
        Solid::Beam { l, w, t, dy } => json!({ "type": "beam", "L": l, "w": w, "t": t, "dy": dy }),
        Solid::Rod { l, r } => json!({ "type": "rod", "L": l, "r": r }),
        Solid::Steps { h_span, dy, n, w, t } => {
            let nn = *n as f64;
            let treads: Vec<J> = (0..*n)
                .map(|i| {
                    let fi = i as f64;
                    let tau = -dy / 2.0 + dy * (fi + 1.0) / nn;
                    let cx = -h_span / 2.0 + h_span * (fi + 0.5) / nn;
                    json!({ "c": [cx, tau - t / 2.0, 0.0], "s": [h_span / nn, t, w] })
                })
                .collect();
            json!({ "type": "steps", "treads": treads })
        }
        Solid::Sweep {
            profile,
            closed,
            frames,
            circle,
            ..
        } => {
            let fr: Vec<J> = frames
                .iter()
                .map(|(p, side, up)| json!({ "p": vec3_json(*p), "side": vec3_json(*side), "up": vec3_json(*up) }))
                .collect();
            json!({ "type": "sweep", "frames": fr, "prof": profile, "closed": closed, "smooth": circle })
        }
        Solid::Mesh { size, pivot, src } => {
            json!({ "type": "mesh", "src": src, "w": size.x, "h": size.y, "d": size.z, "pivot": vec3_json(*pivot) })
        }
    }
}

impl Compiler {
    fn is_bool(&self, n: NodeId) -> bool {
        matches!(
            self.scene.nodes[n].kind.as_str(),
            "union" | "subtract" | "intersect"
        )
    }

    /// 实体条目（2.1.11）；退化 / 隐藏 / 没有几何的返回 None
    fn solid_item(&mut self, n: NodeId) -> Option<J> {
        let node = &self.scene.nodes[n];
        if node.degenerate || node.hidden || !SOLID_KINDS.contains(&node.kind.as_str()) {
            return None;
        }
        let geom = node.geom.clone()?;
        let world = node.world?;
        let path = self.scene.path(n);
        let tags = node.tags.clone();
        let mat = self.effective_material(n);
        Some(json!({
            "kind": "solid",
            "path": path,
            "tags": tags,
            "geom": geom_json(&geom),
            "world": world_json(&world),
            "material": material_json(&mat),
        }))
    }

    /// 布尔条目（2.1.11）
    fn csg_item(&mut self, n: NodeId) -> Option<J> {
        let node = &self.scene.nodes[n];
        if node.degenerate || node.hidden {
            return None;
        }
        let op = node.kind.clone();
        let path = self.scene.path(n);
        let tags = node.tags.clone();
        let kids = node.children.clone();
        let mut children = Vec::new();
        for c in kids {
            if let Some(e) = self.operand_item(c) {
                children.push(e);
            }
        }
        let mat = self.effective_material(n);
        Some(json!({
            "kind": "csg",
            "op": op,
            "path": path,
            "tags": tags,
            "children": children,
            "material": material_json(&mat),
        }))
    }

    /// 操作数条目：实体 → 实体条目；布尔容器 → 布尔条目；其他容器 → 内含条目恰一个时直接用它，否则包一层 union
    fn operand_item(&mut self, c: NodeId) -> Option<J> {
        let node = &self.scene.nodes[c];
        if CURVE_KINDS.contains(&node.kind.as_str())
            || matches!(node.role, Role::Skin)
            || node.degenerate
            || node.hidden
        {
            return None;
        }
        if SOLID_KINDS.contains(&node.kind.as_str()) {
            return self.solid_item(c);
        }
        if self.is_bool(c) {
            return self.csg_item(c);
        }
        let kids = node.children.clone();
        let path = self.scene.path(c);
        let tags = node.tags.clone();
        let mut inner = Vec::new();
        for k in kids {
            if let Some(e) = self.operand_item(k) {
                inner.push(e);
            }
        }
        match inner.len() {
            0 => None,
            1 => inner.pop(),
            _ => {
                let mat = self.effective_material(c);
                Some(json!({
                    "kind": "csg",
                    "op": "union",
                    "path": path,
                    "tags": tags,
                    "children": inner,
                    "material": material_json(&mat),
                }))
            }
        }
    }

    /// items[] 与 curves[]（2.1.11 的收集规则）
    fn collect_items(&mut self) -> (Vec<J>, Vec<J>) {
        let mut items = Vec::new();
        let mut curves = Vec::new();
        let mut stack: Vec<NodeId> = self.scene.nodes[ROOT].children.iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            let kind = self.scene.nodes[n].kind.clone();
            let hidden = self.scene.nodes[n].hidden;
            let degenerate = self.scene.nodes[n].degenerate;
            if CURVE_KINDS.contains(&kind.as_str()) {
                if !hidden && !degenerate {
                    if let (Some(c), Some(w)) = (self.curve_for_output(n), self.scene.nodes[n].world) {
                        let pts: Vec<J> = c.pts.iter().map(|p| vec3_json(w.apply(*p))).collect();
                        curves.push(json!({ "path": self.scene.path(n), "closed": c.closed, "pts": pts }));
                    }
                }
            } else if self.is_bool(n) {
                if let Some(e) = self.csg_item(n) {
                    items.push(e);
                }
                // 布尔容器内部只收曲线
                for d in self.scene.subtree_of(n) {
                    if d != n && CURVE_KINDS.contains(&self.scene.nodes[d].kind.as_str()) {
                        let dn = &self.scene.nodes[d];
                        if !dn.hidden && !dn.degenerate {
                            if let (Some(c), Some(w)) = (self.curve_for_output(d), dn.world) {
                                let pts: Vec<J> = c.pts.iter().map(|p| vec3_json(w.apply(*p))).collect();
                                curves.push(
                                    json!({ "path": self.scene.path(d), "closed": c.closed, "pts": pts }),
                                );
                            }
                        }
                    }
                }
                continue;
            } else if SOLID_KINDS.contains(&kind.as_str()) {
                if let Some(e) = self.solid_item(n) {
                    items.push(e);
                }
            }
            for c in self.scene.nodes[n].children.iter().rev() {
                stack.push(*c);
            }
        }
        (items, curves)
    }

    /// 曲线节点的采样点列；影子 / 副本式节点沿原件取
    fn curve_for_output(&self, n: NodeId) -> Option<s3d_geom::Curve> {
        let mut cur = n;
        loop {
            if let Some(c) = &self.scene.nodes[cur].curve {
                return Some(c.clone());
            }
            match self.scene.nodes[cur].role {
                Role::Shadow { original } | Role::TwinOf { original } => cur = original,
                _ => return None,
            }
        }
    }

    fn defs_json(&mut self) -> Vec<J> {
        let mut defs: Vec<crate::compile::TemplateDef> = self.defs.values().cloned().collect();
        defs.sort_by_key(|d| d.first_index);
        let gf = self.global_frame;
        let mut out = Vec::new();
        for d in defs {
            let (params, base, metas) = match &d.stmt.kind {
                StmtKind::Def {
                    params, base, metas, ..
                } => (
                    params.clone(),
                    base.as_ref().map(|(b, _)| b.clone()),
                    metas.clone(),
                ),
                _ => continue,
            };
            let ps: Vec<J> = params
                .iter()
                .map(|p| {
                    json!({
                        "name": p.name,
                        "required": p.default.is_none(),
                        "default": p.default.as_ref().map(expr_text),
                    })
                })
                .collect();
            let meta = self.eval_metas(&metas, gf, &d.stmt);
            out.push(json!({
                "name": d.name,
                "line": d.stmt.line,
                "file": self.file_display(d.stmt.file_index),
                "params": ps,
                "base": base,
                "meta": meta_json(&meta),
            }));
        }
        out
    }

    fn materials_json(&mut self) -> J {
        self.build_material_table();
        let mut out = Map::new();
        for (name, e) in self.material_table.clone() {
            out.insert(
                name,
                json!({
                    "color": e.value.color,
                    "metal": e.value.metal,
                    "rough": e.value.rough,
                    "opacity": e.value.opacity,
                    "default": e.default,
                    "meta": meta_json(&e.meta),
                }),
            );
        }
        J::Object(out)
    }

    fn files_json(&mut self) -> (J, Vec<J>) {
        let gf = self.global_frame;
        let files = self.prog.files.clone();
        let mut top_meta = json!({ "lang": 2 });
        let mut out = Vec::new();
        for f in &files {
            let stmt = crate::ast::Stmt {
                kind: StmtKind::Import {
                    path: String::new(),
                    metas: vec![],
                },
                line: f.header_line.unwrap_or(1),
                file_index: f.index,
            };
            let header: Vec<crate::ast::Prop> =
                f.header.iter().filter(|p| p.key != "lang").cloned().collect();
            let hm = self.eval_metas(&header, gf, &stmt);
            let mut m = Map::new();
            m.insert("lang".to_string(), json!(2));
            for (k, v) in &hm {
                m.insert(k.clone(), value_json(v));
            }
            let meta = J::Object(m);
            if f.index == 0 {
                top_meta = meta.clone();
            }
            let import = match &f.import {
                None => J::Null,
                Some((fi, line, metas)) => {
                    let istmt = crate::ast::Stmt {
                        kind: StmtKind::Import {
                            path: String::new(),
                            metas: vec![],
                        },
                        line: *line,
                        file_index: *fi,
                    };
                    let im = self.eval_metas(metas, gf, &istmt);
                    json!({ "file": self.file_display(*fi), "line": line, "meta": meta_json(&im) })
                }
            };
            out.push(json!({ "file": f.display, "meta": meta, "import": import }));
        }
        (top_meta, out)
    }
}

pub fn to_json(c: &mut Compiled) -> J {
    let comp = &mut c.compiler;
    comp.run_semantics();
    comp.build_material_table();
    let (items, curves) = comp.collect_items();
    let defs = comp.defs_json();
    let materials = comp.materials_json();
    let (top_meta, files) = comp.files_json();
    let facts = comp.facts_json();
    let semantic = comp.semantic_json();
    comp.diags.sort();
    let mut nodes = Vec::new();
    for id in comp.scene.order() {
        let n = &comp.scene.nodes[id];
        if id == ROOT {
            continue;
        }
        let depth = comp.scene.chain(id).len() - 1;
        let world = n.world.map(|w| world_json(&w));
        let (ghost, of) = match &n.role {
            Role::Shadow { original } | Role::TwinOf { original } => (true, Some(comp.scene.path(*original))),
            _ => (false, None),
        };
        nodes.push(json!({
            "path": comp.scene.path(id),
            "kind": n.kind,
            "id": if n.name.is_empty() { J::Null } else { json!(n.name) },
            "tags": n.tags,
            "concepts": n.concepts,
            "line": n.line,
            "file": comp.file_display(n.file_index),
            "types": if n.types.is_empty() { J::Null } else { json!(n.types) },
            "meta": meta_json(&n.meta),
            "depth": depth,
            "late": n.late,
            "ghost": ghost,
            "of": of,
            "curve": CURVE_KINDS.contains(&n.kind.as_str()),
            "hidden": n.hidden,
            "skin": comp.scene.in_skin(id),
            "world": world,
            "box": box_json(n.wbox),
            "tight": if n.degenerate { J::Null } else { box_json(n.tight) },
            "degenerate": n.degenerate,
        }));
    }
    let diag = |d: &crate::diag::Diagnostic| json!({ "file": d.file, "line": d.line, "path": d.path, "message": d.message });
    let mut top = Map::new();
    top.insert("meta".into(), top_meta);
    top.insert("files".into(), J::Array(files));
    top.insert("items".into(), J::Array(items));
    top.insert("curves".into(), J::Array(curves));
    for (k, v) in facts {
        top.insert(k, v);
    }
    top.insert("nodes".into(), J::Array(nodes));
    for (k, v) in semantic {
        top.insert(k, v);
    }
    top.insert("defs".into(), J::Array(defs));
    top.insert("materials".into(), materials);
    top.insert(
        "errors".into(),
        J::Array(comp.diags.errors.iter().map(diag).collect()),
    );
    top.insert(
        "warnings".into(),
        J::Array(comp.diags.warnings.iter().map(diag).collect()),
    );
    J::Object(top)
}
