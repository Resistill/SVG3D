//! 编译器状态与入口。各阶段的实现分散在 elaborate.rs（阶段 2–6）、eval.rs（表达式）、solve.rs（阶段 7）。

use crate::ast::{Prop, Stmt};
use crate::diag::Diagnostics;
use crate::env::{Binding, Env, FrameId, FrameKind};
use crate::program::{load_program, Program, Reader};
use crate::scene::{NodeId, Scene, ROOT};
use crate::value::Value;
use indexmap::IndexMap;

/// 求值时机（1.5.8、1.7.4）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    /// 展开时：常量、元数据（不查对象）
    Expand,
    /// 几何时机 over / drop 的选择器：只能用静态量
    Selector,
    /// 求解时的属性值
    Geometry,
    /// 外观时机与语义层：无限制
    Appearance,
}

/// 表达式的求值上下文（1.5.8）
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    pub frame: FrameId,
    /// 搜索起点 P（父节点或根）
    pub start: NodeId,
    pub timing: Timing,
}

/// 模板表的一条定义（4.2.1）
#[derive(Clone, Debug)]
pub struct TemplateDef {
    pub name: String,
    pub stmt: Stmt,
    /// 就地扩展链里的基定义（def A : A 时登记时捕获）
    pub extends: Option<Box<TemplateDef>>,
    pub first_index: usize,
}

#[derive(Clone, Debug)]
pub struct MaterialDef {
    pub name: String,
    pub props: Vec<Prop>,
    pub metas: Vec<Prop>,
    pub line: u32,
    pub file_index: u32,
    pub first_index: usize,
}

pub struct CompileOptions {
    pub metre: f64,
}

impl Default for CompileOptions {
    fn default() -> Self {
        CompileOptions { metre: 1.0 }
    }
}

pub struct Compiler {
    pub prog: Program,
    pub scene: Scene,
    pub env: Env,
    pub diags: Diagnostics,
    pub global_frame: FrameId,
    pub defs: IndexMap<String, TemplateDef>,
    pub materials: IndexMap<String, MaterialDef>,
    pub metre: f64,
    /// 语义层语句（阶段 7 用）
    pub semantic_stmts: Vec<Stmt>,
    /// 几何时机的修改序列（阶段 5）
    pub modifications: Vec<crate::elaborate::Modification>,
    /// copy 表（阶段 4）
    pub copies: Vec<crate::elaborate::PendingCopy>,
    /// 节点顺序缓存（树定稿后建）
    pub order_idx: Vec<usize>,
    /// 求解中的"忙"标记（3.6：惰性求值的后置判定）
    pub busy_local: Vec<bool>,
    pub busy_box: Vec<bool>,
    /// 下一个新建节点的有效属性表（copy / 重建时替换源语句的属性，4.1.3）
    pub prop_override: Option<IndexMap<String, crate::scene::PropVal>>,
    /// 阶段 5 以后：copy 语句立即展开（4.3.5 末段）
    pub immediate_copies: bool,
    /// copy 的登记序号（占位换算用）
    pub copy_seq: usize,
    /// 已插入的 copy：(父节点, 序号)
    pub copies_inserted: Vec<(NodeId, usize)>,
    /// 推迟到阶段 6 的修改（4.4.4）
    pub deferred_mods: Vec<crate::elaborate::Modification>,
    /// 材质表（10.1.3）与默认材质名
    pub material_table: IndexMap<String, crate::material::MaterialEntry>,
    pub default_material: Option<String>,
    pub material_table_built: bool,
    pub mat_errors_reported: Vec<NodeId>,
    pub color_errors_reported: Vec<NodeId>,
    /// 样式动作（7.7、10.3.3）：(选择集, mat, color)，按语句顺序
    pub style_actions: Vec<(Vec<NodeId>, Option<String>, Option<crate::material::ColorVal>)>,
}

impl Compiler {
    pub fn new(prog: Program, diags: Diagnostics, opts: &CompileOptions) -> Compiler {
        let mut env = Env::default();
        let global_frame = env.new_frame(FrameKind::Global, None);
        env.bind(global_frame, "metre", Binding::Value(Value::Num(opts.metre)));
        let scene = Scene::new(global_frame);
        Compiler {
            prog,
            scene,
            env,
            diags,
            global_frame,
            defs: IndexMap::new(),
            materials: IndexMap::new(),
            metre: opts.metre,
            semantic_stmts: Vec::new(),
            modifications: Vec::new(),
            copies: Vec::new(),
            order_idx: Vec::new(),
            prop_override: None,
            immediate_copies: false,
            copy_seq: 0,
            copies_inserted: Vec::new(),
            deferred_mods: Vec::new(),
            material_table: IndexMap::new(),
            default_material: None,
            material_table_built: false,
            mat_errors_reported: Vec::new(),
            color_errors_reported: Vec::new(),
            style_actions: Vec::new(),
            busy_local: Vec::new(),
            busy_box: Vec::new(),
        }
    }

    pub fn file_display(&self, file_index: u32) -> String {
        self.prog
            .files
            .get(file_index as usize)
            .map(|f| f.display.clone())
            .unwrap_or_default()
    }

    /// 报一条归属于节点的错误
    pub fn node_error(&mut self, n: NodeId, msg: impl Into<String>) {
        let node = &self.scene.nodes[n];
        let (fi, line) = (node.file_index, node.line);
        let path = if n == ROOT { None } else { Some(self.scene.path(n)) };
        let file = self.file_display(fi);
        self.diags.error(fi, &file, line, path.as_deref(), msg);
    }

    pub fn node_warning(&mut self, n: NodeId, msg: impl Into<String>) {
        let node = &self.scene.nodes[n];
        let (fi, line) = (node.file_index, node.line);
        let path = if n == ROOT { None } else { Some(self.scene.path(n)) };
        let file = self.file_display(fi);
        self.diags.warning(fi, &file, line, path.as_deref(), msg);
    }

    /// 报一条归属于语句（文件、行）的错误
    pub fn stmt_error(&mut self, file_index: u32, line: u32, msg: impl Into<String>) {
        let file = self.file_display(file_index);
        self.diags.error(file_index, &file, line, None, msg);
    }

    /// 节点退化（1.1.4 第 2 条）：报错并标记
    pub fn degenerate(&mut self, n: NodeId, msg: impl Into<String>) {
        self.node_error(n, msg);
        self.scene.nodes[n].degenerate = true;
    }

    pub fn rebuild_order(&mut self) {
        self.order_idx = self.scene.order_index();
    }
}

/// 编译结果
pub struct Compiled {
    pub compiler: Compiler,
}

/// 从源码编译。`main_identity` 是主文件的规范化绝对路径，`main_display` 是可显示路径。
pub fn compile(
    main_text: &str,
    main_identity: &str,
    main_display: &str,
    read: Reader<'_>,
    opts: &CompileOptions,
) -> Compiled {
    let mut diags = Diagnostics::default();
    let prog = load_program(main_text, main_identity, main_display, read, &mut diags);
    let mut c = Compiler::new(prog, diags, opts);
    c.register();
    c.expand_all();
    c.finish_tree();
    c.solve_all();
    c.diags.sort();
    Compiled { compiler: c }
}
