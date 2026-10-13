//! 编译输出的 JSON 形状（各章的"输出形状"小节；M1 先出 nodes / errors / warnings）。

use crate::compile::Compiled;
use crate::scene::{Role, ROOT};
use crate::value::Value;
use serde_json::{json, Map, Value as J};

fn vec3_json(v: s3d_geom::Vec3) -> J {
    json!([v.x, v.y, v.z])
}

fn box_json(b: Option<s3d_geom::Aabb>) -> J {
    match b {
        Some(b) => json!({ "min": vec3_json(b.min), "max": vec3_json(b.max) }),
        None => J::Null,
    }
}

fn value_json(v: &Value) -> J {
    match v {
        Value::Num(n) => json!(n),
        Value::Bool(b) => json!(b),
        Value::Str(s) => json!(s),
        Value::Vector(xs) => json!(xs),
        Value::List(items) => J::Array(items.iter().map(value_json).collect()),
        _ => J::Null,
    }
}

pub fn to_json(c: &Compiled) -> J {
    let comp = &c.compiler;
    let mut nodes = Vec::new();
    for id in comp.scene.order() {
        let n = &comp.scene.nodes[id];
        if id == ROOT {
            continue;
        }
        let depth = comp.scene.chain(id).len();
        let world = n.world.map(|w| {
            json!({
                "R": [[w.r.x_axis.x, w.r.y_axis.x, w.r.z_axis.x], [w.r.x_axis.y, w.r.y_axis.y, w.r.z_axis.y], [w.r.x_axis.z, w.r.y_axis.z, w.r.z_axis.z]],
                "t": vec3_json(w.t),
            })
        });
        let mut meta = Map::new();
        for (k, v) in &n.meta {
            meta.insert(k.clone(), value_json(v));
        }
        let (ghost, of) = match &n.role {
            Role::Shadow { original } | Role::TwinOf { original } => (true, Some(comp.scene.path(*original))),
            _ => (false, None),
        };
        nodes.push(json!({
            "path": comp.scene.path(id),
            "kind": n.kind,
            "id": if n.name.is_empty() { J::Null } else { json!(n.name) },
            "tags": n.tags,
            "concepts": [],
            "line": n.line,
            "file": comp.file_display(n.file_index),
            "types": if n.types.is_empty() { J::Null } else { json!(n.types) },
            "meta": J::Object(meta),
            "depth": depth,
            "late": n.late,
            "ghost": ghost,
            "of": of,
            "curve": crate::ast::CURVE_KINDS.contains(&n.kind.as_str()),
            "hidden": n.hidden,
            "skin": matches!(n.role, Role::Skin),
            "world": world,
            "box": box_json(n.wbox),
            "tight": box_json(n.tight),
            "degenerate": n.degenerate,
        }));
    }
    let diag = |d: &crate::diag::Diagnostic| json!({ "file": d.file, "line": d.line, "path": d.path, "message": d.message });
    json!({
        "meta": { "lang": 2 },
        "files": comp.prog.files.iter().map(|f| json!({ "file": f.display })).collect::<Vec<_>>(),
        "nodes": nodes,
        "errors": comp.diags.errors.iter().map(diag).collect::<Vec<_>>(),
        "warnings": comp.diags.warnings.iter().map(diag).collect::<Vec<_>>(),
    })
}
