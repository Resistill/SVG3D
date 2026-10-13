//! 场景树（spec/01 1.6）：节点、路径、节点顺序、全局索引。

use crate::ast::{Expr, Prop, Stmt};
use crate::env::FrameId;
use crate::value::Value;
use indexmap::IndexMap;
use s3d_geom::{Aabb, Transform};
use std::rc::Rc;

pub type NodeId = usize;
/// 根的编号；根不是节点（没有路径）
pub const ROOT: NodeId = 0;

/// 节点的角色（决定路径段、是否进全局索引、变换怎么求）
#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    /// 一般节点：实体、曲线、容器、mesh、模板实例、copy 节点
    Normal,
    /// 阵列实例（repeat 的子节点），带下标
    Instance { i: u32, j: u32, k: u32, idx: u32 },
    /// 副本组 `mirrored` / `r<k>`（容器式 mirror / rotate）
    TwinGroup { k: u32 },
    /// 副本式节点 `mirror R of=A`
    TwinOf { original: NodeId },
    /// 影子：原件子树里某个节点的对应
    Shadow { original: NodeId },
    /// 变体皮肤
    Skin,
}

/// 一条属性的值：表达式 + 写下它的上下文（1.5.8：写在哪里就在哪里求值）
#[derive(Clone, Debug)]
pub struct PropVal {
    pub expr: Expr,
    pub frame: FrameId,
    /// 搜索起点（父节点）
    pub start: NodeId,
    pub line: u32,
    pub file_index: u32,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub seg: String,
    /// 具名节点的名字；匿名、阵列实例、副本组等按 1.6.7
    pub name: String,
    /// 种类（1.6.7）
    pub kind: String,
    /// 类型链（模板实例；4.2.7），最派生在前
    pub types: Vec<String>,
    pub role: Role,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
    /// 已创建的子节点总数（匿名段编号用，永不回收）
    pub created: u32,
    /// 有效属性（4.1.3）
    pub props: IndexMap<String, PropVal>,
    pub tags: Vec<String>,
    /// 元数据（展开时求值）
    pub meta: IndexMap<String, Value>,
    /// 源语句的位置
    pub file_index: u32,
    pub line: u32,
    /// 体内语句的词法环境帧（模板实例：模板体帧；容器：块帧）
    pub frame: FrameId,
    /// 语句的词法帧（属性值在这里求值）
    pub stmt_frame: FrameId,
    /// 退化节点（1.1.4）
    pub degenerate: bool,
    pub hidden: bool,
    pub late: bool,
    /// 求解结果
    pub local: Option<Transform>,
    pub lbox: Option<Aabb>,
    pub world: Option<Transform>,
    pub wbox: Option<Aabb>,
    pub tight: Option<Aabb>,
    /// 几何参数求值结果（求解阶段填）
    pub geom: Option<s3d_geom::Solid>,
    /// 曲线的采样结果（求解阶段填）
    pub curve: Option<s3d_geom::Curve>,
    /// 源语句（copy、重建用；阵列实例、副本组没有）
    pub src_stmt: Option<Rc<Stmt>>,
    /// 展开深度（4.1.4）与展开位置（4.1.2）
    pub depth: u32,
    pub pos: Vec<u64>,
    /// 已被 drop / 重建拿掉（不在树里）
    pub dropped: bool,
    /// 命中的概念名（7.4.4）
    pub concepts: Vec<String>,
}

impl Node {
    pub fn is_named(&self) -> bool {
        matches!(self.role, Role::Normal | Role::TwinOf { .. }) && !self.name.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct Scene {
    pub nodes: Vec<Node>,
}

impl Scene {
    pub fn new(root_frame: FrameId) -> Scene {
        let root = Node {
            seg: String::new(),
            name: String::new(),
            kind: "root".to_string(),
            types: vec![],
            role: Role::Normal,
            parent: ROOT,
            children: vec![],
            created: 0,
            props: IndexMap::new(),
            tags: vec![],
            meta: IndexMap::new(),
            file_index: 0,
            line: 0,
            frame: root_frame,
            stmt_frame: root_frame,
            degenerate: false,
            hidden: false,
            late: false,
            local: Some(Transform::IDENTITY),
            lbox: Some(Aabb::ZERO),
            world: Some(Transform::IDENTITY),
            wbox: Some(Aabb::ZERO),
            tight: None,
            geom: None,
            curve: None,
            src_stmt: None,
            depth: 0,
            pos: vec![],
            dropped: false,
            concepts: vec![],
        };
        Scene { nodes: vec![root] }
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id]
    }

    /// 在 parent 下新建节点；匿名时按 1.6.1 编号 `种类词#i`
    pub fn add(&mut self, parent: NodeId, mut node: Node) -> NodeId {
        let created = self.nodes[parent].created;
        if node.seg.is_empty() {
            node.seg = format!("{}#{}", node.kind, created);
        }
        node.parent = parent;
        let id = self.nodes.len();
        self.nodes.push(node);
        self.nodes[parent].children.push(id);
        self.nodes[parent].created = created + 1;
        id
    }

    /// 把 n 从父节点的子节点列表里拿掉，整棵子树标记为 dropped（4.5.6）
    pub fn detach(&mut self, n: NodeId) {
        let p = self.nodes[n].parent;
        self.nodes[p].children.retain(|c| *c != n);
        let mut stack = vec![n];
        while let Some(x) = stack.pop() {
            self.nodes[x].dropped = true;
            stack.extend(self.nodes[x].children.iter().copied());
        }
    }

    /// 子树（含自身）的全部节点
    pub fn subtree_of(&self, n: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![n];
        while let Some(x) = stack.pop() {
            out.push(x);
            for c in self.nodes[x].children.iter().rev() {
                stack.push(*c);
            }
        }
        out
    }

    /// 路径（段用 `.` 连接）；根为空串
    pub fn path(&self, id: NodeId) -> String {
        let mut segs = Vec::new();
        let mut cur = id;
        while cur != ROOT {
            segs.push(self.nodes[cur].seg.clone());
            cur = self.nodes[cur].parent;
        }
        segs.reverse();
        segs.join(".")
    }

    pub fn is_ancestor(&self, a: NodeId, mut n: NodeId) -> bool {
        while n != ROOT {
            n = self.nodes[n].parent;
            if n == a {
                return true;
            }
        }
        a == ROOT
    }

    /// 节点顺序（先序深度优先，1.6.1），不含根
    pub fn order(&self) -> Vec<NodeId> {
        let mut out = Vec::with_capacity(self.nodes.len());
        let mut stack: Vec<NodeId> = self.nodes[ROOT].children.iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            out.push(n);
            for c in self.nodes[n].children.iter().rev() {
                stack.push(*c);
            }
        }
        out
    }

    /// 节点顺序的序号表（NodeId → 在节点顺序里的位置）
    pub fn order_index(&self) -> Vec<usize> {
        let mut idx = vec![usize::MAX; self.nodes.len()];
        for (i, n) in self.order().into_iter().enumerate() {
            idx[n] = i;
        }
        idx
    }

    /// 从根到 n 的祖先链（不含根，含 n）
    pub fn chain(&self, mut n: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        while n != ROOT {
            out.push(n);
            n = self.nodes[n].parent;
        }
        out.reverse();
        out
    }

    /// 是否在皮肤子树里
    pub fn in_skin(&self, mut n: NodeId) -> bool {
        while n != ROOT {
            if matches!(self.nodes[n].role, Role::Skin) {
                return true;
            }
            n = self.nodes[n].parent;
        }
        false
    }

    /// 是否在全局索引里（1.6.5 第 3 步）：具名、非阵列实例、非副本组、非影子、不在皮肤里
    pub fn in_global_index(&self, n: NodeId) -> bool {
        let node = &self.nodes[n];
        node.is_named() && !node.dropped && !self.in_skin(n)
    }

    /// 把一组节点按节点顺序排序并去重
    pub fn sorted_set(&self, order_idx: &[usize], mut ids: Vec<NodeId>) -> Vec<NodeId> {
        ids.sort_by_key(|n| order_idx[*n]);
        ids.dedup();
        ids
    }
}

/// 从语句属性建 PropVal 表（重复键由调用方检查）
pub fn props_from(
    props: &[Prop],
    frame: FrameId,
    start: NodeId,
    file_index: u32,
) -> IndexMap<String, PropVal> {
    let mut m = IndexMap::new();
    for p in props {
        m.insert(
            p.key.clone(),
            PropVal {
                expr: p.value.clone(),
                frame,
                start,
                line: p.line,
                file_index,
            },
        );
    }
    m
}
