//! 运行时值（spec/01 1.5.7）与锚点写法（1.5.4）。

use crate::scene::NodeId;
use s3d_geom::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub enum Anchor {
    /// `B.origin`
    Origin,
    /// 方向词 / 组合词：a ∈ {-1,0,1}^3（`center` 是 0）
    Dir(Vec3),
    /// `B[u,v,w]`
    Norm(Vec3),
    /// `C[t]`
    CurveT(f64),
    /// `C.start` / `C.mid` / `C.end`（t = 0 / 0.5 / 1）
    CurveNamed(f64),
    /// `B.top[u,w]`、`B.side[θ,v]`、`B.surface[θ,φ]`
    Surface { which: SurfaceKind, a: f64, b: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Top,
    Side,
    Surface,
}

/// 对象引用：节点 + 可选锚点 + 累加的偏移（1.5.2）
#[derive(Clone, Debug, PartialEq)]
pub struct RefVal {
    pub node: NodeId,
    pub anchor: Option<Anchor>,
    pub offset: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Str(String),
    /// 向量：长度 2、3、4 或 6
    Vector(Vec<f64>),
    List(Vec<Value>),
    Ref(RefVal),
    /// 集合：成员按节点顺序，无重复
    Set(Vec<NodeId>),
    /// 模式值（pattern 名）
    Pattern(String),
    /// 只来自 where 里不存在的元数据键
    Missing,
}

impl Value {
    /// 错误文案里值的显示（1.5.2）
    pub fn describe(&self) -> String {
        match self {
            Value::Num(n) => format!("{}", n),
            Value::Bool(b) => format!("{}", b),
            Value::Str(s) => format!("\"{}\"", s),
            Value::Vector(v) => {
                let parts: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
                format!("({})", parts.join(","))
            }
            Value::List(l) => format!("[{} 项]", l.len()),
            Value::Ref(_) => "引用".to_string(),
            Value::Set(s) => format!("{{{} 个对象}}", s.len()),
            Value::Pattern(p) => format!("模式 {}", p),
            Value::Missing => "缺失".to_string(),
        }
    }

    pub fn vec3(v: Vec3) -> Value {
        Value::Vector(vec![v.x, v.y, v.z])
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            Value::Vector(v) if v.len() == 3 => Some(Vec3::new(v[0], v[1], v[2])),
            Value::Num(n) => Some(Vec3::new(*n, *n, *n)),
            _ => None,
        }
    }
}
