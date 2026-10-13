//! 给评分、渲染、导出用的条目树：与 `output::to_json` 的 `items[]` 同一套收集规则（2.1.11），
//! 但保留 Rust 值（实体、世界变换、材质），不经过 JSON。另给节点的摘要（路径、源位置、元数据）。

use crate::ast::{CURVE_KINDS, SOLID_KINDS};
use crate::compile::{Compiled, Compiler};
use crate::scene::{NodeId, Role, ROOT};
use crate::value::Value;
use s3d_geom::{Solid, Transform};

/// 实体条目
#[derive(Clone, Debug)]
pub struct ItemSolid {
    pub path: String,
    pub solid: Solid,
    pub world: Transform,
    /// 有效材质的颜色 `#rrggbb`
    pub color: String,
}

/// 条目树：实体，或布尔容器（操作数按顺序）
#[derive(Clone, Debug)]
pub enum ItemTree {
    Solid(ItemSolid),
    Csg {
        op: String,
        path: String,
        color: String,
        children: Vec<ItemTree>,
    },
}

impl ItemTree {
    pub fn path(&self) -> &str {
        match self {
            ItemTree::Solid(s) => &s.path,
            ItemTree::Csg { path, .. } => path,
        }
    }
    pub fn color(&self) -> &str {
        match self {
            ItemTree::Solid(s) => &s.color,
            ItemTree::Csg { color, .. } => color,
        }
    }
}

/// 节点摘要（评分的描述长度归因、角色、网格来源用）
#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub id: NodeId,
    pub path: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
    /// 影子 / 副本（原型 ghost）
    pub ghost: bool,
    pub hidden: bool,
    pub degenerate: bool,
    /// `~src` 元数据（字符串）
    pub meta_src: Option<String>,
    /// `~role` 元数据（字符串）
    pub meta_role: Option<String>,
    /// 主定位关系的参照物
    pub placement_ref: Option<NodeId>,
    /// 有几何条目（实体或最外层布尔容器）
    pub has_item: bool,
}

fn is_bool(c: &Compiler, n: NodeId) -> bool {
    matches!(c.scene.nodes[n].kind.as_str(), "union" | "subtract" | "intersect")
}

fn solid_item(c: &mut Compiler, n: NodeId) -> Option<ItemTree> {
    let node = &c.scene.nodes[n];
    if node.degenerate || node.hidden || !SOLID_KINDS.contains(&node.kind.as_str()) {
        return None;
    }
    let solid = node.geom.clone()?;
    let world = node.world?;
    let path = c.scene.path(n);
    let color = c.effective_material(n).color;
    Some(ItemTree::Solid(ItemSolid {
        path,
        solid,
        world,
        color,
    }))
}

fn csg_item(c: &mut Compiler, n: NodeId) -> Option<ItemTree> {
    let node = &c.scene.nodes[n];
    if node.degenerate || node.hidden {
        return None;
    }
    let op = node.kind.clone();
    let path = c.scene.path(n);
    let kids = node.children.clone();
    let mut children = Vec::new();
    for k in kids {
        if let Some(e) = operand_item(c, k) {
            children.push(e);
        }
    }
    let color = c.effective_material(n).color;
    Some(ItemTree::Csg {
        op,
        path,
        color,
        children,
    })
}

fn operand_item(c: &mut Compiler, k: NodeId) -> Option<ItemTree> {
    let node = &c.scene.nodes[k];
    if CURVE_KINDS.contains(&node.kind.as_str())
        || matches!(node.role, Role::Skin)
        || node.degenerate
        || node.hidden
    {
        return None;
    }
    if SOLID_KINDS.contains(&node.kind.as_str()) {
        return solid_item(c, k);
    }
    if is_bool(c, k) {
        return csg_item(c, k);
    }
    let kids = node.children.clone();
    let path = c.scene.path(k);
    let mut inner = Vec::new();
    for j in kids {
        if let Some(e) = operand_item(c, j) {
            inner.push(e);
        }
    }
    match inner.len() {
        0 => None,
        1 => inner.pop(),
        _ => {
            let color = c.effective_material(k).color;
            Some(ItemTree::Csg {
                op: "union".to_string(),
                path,
                color,
                children: inner,
            })
        }
    }
}

/// 收集条目树（2.1.11 的收集规则；语义层已跑完时材质含风格）
pub fn collect_item_trees(c: &mut Compiled) -> Vec<ItemTree> {
    let comp = &mut c.compiler;
    comp.run_semantics();
    comp.build_material_table();
    let mut items = Vec::new();
    let mut stack: Vec<NodeId> = comp.scene.nodes[ROOT].children.iter().rev().copied().collect();
    while let Some(n) = stack.pop() {
        let kind = comp.scene.nodes[n].kind.clone();
        if CURVE_KINDS.contains(&kind.as_str()) {
            continue;
        }
        if is_bool(comp, n) {
            if let Some(e) = csg_item(comp, n) {
                items.push(e);
            }
            continue;
        }
        if SOLID_KINDS.contains(&kind.as_str()) {
            if let Some(e) = solid_item(comp, n) {
                items.push(e);
            }
        }
        for ch in comp.scene.nodes[n].children.iter().rev() {
            stack.push(*ch);
        }
    }
    items
}

fn meta_str(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Str(s)) => Some(s.clone()),
        Some(Value::Num(x)) => Some(format!("{}", x)),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

/// 全部节点（不含根）的摘要，按节点顺序
pub fn node_infos(c: &Compiled) -> Vec<NodeInfo> {
    let comp = &c.compiler;
    let mut out = Vec::new();
    let mut stack: Vec<NodeId> = comp.scene.nodes[ROOT].children.iter().rev().copied().collect();
    while let Some(n) = stack.pop() {
        let node = &comp.scene.nodes[n];
        // 原型的 ghost 只标影子（副本式 mirror 节点自己不是 ghost：它的语句长度归到它）
        let ghost = matches!(node.role, Role::Shadow { .. });
        let has_item = !node.degenerate
            && !node.hidden
            && (SOLID_KINDS.contains(&node.kind.as_str()) || is_bool(comp, n));
        out.push(NodeInfo {
            id: n,
            path: comp.scene.path(n),
            kind: node.kind.clone(),
            file: comp.file_display(node.file_index),
            line: node.line,
            ghost,
            hidden: node.hidden,
            degenerate: node.degenerate,
            meta_src: meta_str(node.meta.get("src")),
            meta_role: meta_str(node.meta.get("role")),
            placement_ref: node.placement_ref,
            has_item,
        });
        for ch in node.children.iter().rev() {
            stack.push(*ch);
        }
    }
    out
}

/// 描述文本：(可显示文件名, 文本)，主文件在前、import 的文件按加载顺序；文件名与 `NodeInfo::file` 一致
pub fn source_files(c: &Compiled) -> Vec<(String, String)> {
    c.compiler
        .prog
        .files
        .iter()
        .map(|f| (f.display.clone(), f.text.clone()))
        .collect()
}
