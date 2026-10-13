//! 语法树（spec/01 1.3、1.5）。只记录形态；属性类型、名字解析、求值都在后面的阶段。

use crate::lexer::Seg;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    /// `or` / `|`
    Or,
    /// `and` / `&`
    And,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Number(f64),
    Str(String),
    Bool(bool),
    /// 井号记号：标签引用或颜色，由位置决定（1.2.7）
    Hash(String),
    /// 方向组合词字面量（独立出现，如 `pin=top-left`）
    DirWord(Vec<String>),
    /// 单段裸标识符（可能是对象名、方向词、枚举值、let 名……由类型决定）
    Ident(String),
    /// 路径引用：段、可选坐标后缀（1–3 个数的向量项）、可选分量后缀
    PathRef {
        segs: Vec<Seg>,
        coords: Option<Vec<Expr>>,
        axis: Option<char>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    /// 顶层逗号：向量（≥ 2 项）
    Tuple(Vec<Expr>),
    /// 列表 `[a; b]`
    List(Vec<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `lhs ~ /re/`
    Match {
        lhs: Box<Expr>,
        regex: String,
    },
    /// `lhs where cond`
    Where {
        lhs: Box<Expr>,
        cond: Box<Expr>,
    },
}

/// `k=v`（属性）或 `~k=v`（元数据）
#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub key: String,
    pub value: Expr,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub default: Option<Expr>,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConceptRhs {
    /// `concept a = expr` 或 `concept a : b = expr`（后者是 `b & expr`）
    Expr(Expr),
    /// `concept a : b = where cond`
    Where(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RelWord {
    On,
    Supports,
    Against,
    Inside,
    Contains,
    Touching,
    Intersects,
}

impl RelWord {
    pub fn parse(s: &str) -> Option<RelWord> {
        Some(match s {
            "on" => RelWord::On,
            "supports" => RelWord::Supports,
            "against" => RelWord::Against,
            "inside" => RelWord::Inside,
            "contains" => RelWord::Contains,
            "touching" => RelWord::Touching,
            "intersects" => RelWord::Intersects,
            _ => return None,
        })
    }
    pub fn name(&self) -> &'static str {
        match self {
            RelWord::On => "on",
            RelWord::Supports => "supports",
            RelWord::Against => "against",
            RelWord::Inside => "inside",
            RelWord::Contains => "contains",
            RelWord::Touching => "touching",
            RelWord::Intersects => "intersects",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    RoleIn { role: String, expr: Expr },
    Relation { lhs: Expr, rel: RelWord, rhs: Expr },
    Boolean(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintLine {
    pub line: u32,
    pub constraint: Constraint,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    /// 实体、曲线、容器、mesh、模板实例（1.3.3）
    Generic {
        kind: String,
        name: Option<String>,
        tags: Vec<(String, u32)>,
        props: Vec<Prop>,
        metas: Vec<Prop>,
        block: Option<Vec<Stmt>>,
    },
    Def {
        name: String,
        params: Vec<Param>,
        base: Option<(String, Vec<Prop>)>,
        metas: Vec<Prop>,
        body: Vec<Stmt>,
    },
    Let {
        name: String,
        value: Expr,
        metas: Vec<Prop>,
    },
    Import {
        path: String,
        metas: Vec<Prop>,
    },
    Material {
        name: String,
        props: Vec<Prop>,
        metas: Vec<Prop>,
    },
    Over {
        selector: Expr,
        retype: Option<String>,
        props: Vec<Prop>,
        metas: Vec<Prop>,
        block: Option<Vec<Stmt>>,
    },
    Drop {
        selector: Expr,
        metas: Vec<Prop>,
    },
    Concept {
        name: String,
        parent: Option<String>,
        rhs: ConceptRhs,
        /// `=` 右边的源码文本（7.4.4 的 src）
        src: String,
        metas: Vec<Prop>,
    },
    Pattern {
        name: String,
        roles: Vec<(String, bool)>,
        metas: Vec<Prop>,
        constraints: Vec<ConstraintLine>,
    },
    Check {
        message: Option<String>,
        expr: Expr,
        source_text: String,
        metas: Vec<Prop>,
    },
    Style {
        selector: Expr,
        props: Vec<Prop>,
        metas: Vec<Prop>,
    },
    Variant {
        selector: Expr,
        /// 选择器的源码文本（7.8.9 的 src）
        src: String,
        props: Vec<Prop>,
        metas: Vec<Prop>,
    },
    Copy {
        name: String,
        tags: Vec<(String, u32)>,
        props: Vec<Prop>,
        metas: Vec<Prop>,
        block: Option<Vec<Stmt>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    /// 首行
    pub line: u32,
    /// 文件序号（主文件 0）
    pub file_index: u32,
}

/// 一个文件的解析结果
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileAst {
    /// 文件头的元数据项（1.3.1）；没有文件头时为空
    pub header: Vec<Prop>,
    pub header_line: Option<u32>,
    pub stmts: Vec<Stmt>,
}

pub const BUILTIN_KINDS: [&str; 24] = [
    "box",
    "wedge",
    "plate",
    "hull",
    "mesh",
    "cyl",
    "cone",
    "sphere",
    "revolve",
    "span",
    "sweep",
    "pipe",
    "line",
    "bezier",
    "spline",
    "arc",
    "helix",
    "group",
    "union",
    "subtract",
    "intersect",
    "repeat",
    "mirror",
    "rotate",
];
pub const SOLID_KINDS: [&str; 12] = [
    "box", "wedge", "plate", "hull", "mesh", "cyl", "cone", "sphere", "revolve", "span", "sweep", "pipe",
];
pub const CURVE_KINDS: [&str; 5] = ["line", "bezier", "spline", "arc", "helix"];
pub const STATEMENT_KEYWORDS: [&str; 12] = [
    "def", "let", "import", "material", "over", "drop", "concept", "pattern", "check", "style", "variant",
    "copy",
];
pub const KEYWORDS: [&str; 7] = ["where", "and", "or", "not", "in", "true", "false"];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}
pub fn is_builtin_kind(s: &str) -> bool {
    BUILTIN_KINDS.contains(&s)
}

/// 文件头 `lang` 检查（1.3.1）：缺省或数字 2 合法；其他值报"不支持的语言版本"。
pub fn parse_header_lang(header: &[Prop]) -> Result<(), String> {
    for p in header {
        if p.key == "lang" {
            match &p.value.kind {
                ExprKind::Number(n) if *n == 2.0 => {}
                _ => return Err("不支持的语言版本（文件头 ~lang 只能是 2）".to_string()),
            }
        }
    }
    Ok(())
}
