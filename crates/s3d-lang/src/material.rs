//! 材质与外观（spec/10）：材质表、颜色规范形式、材质链与几何时机的有效材质 E_g、样式后的 E。

use crate::ast::ExprKind;
use crate::compile::{Compiler, Ctx, Timing};
use crate::scene::{NodeId, Role, ROOT};
use crate::value::Value;
use indexmap::IndexMap;

/// 材质值（10.1.2）
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialValue {
    pub name: String,
    /// 规范形式 `#rrggbb`（小写，不含 alpha）
    pub color: String,
    pub metal: f64,
    pub rough: f64,
    pub opacity: f64,
}

impl MaterialValue {
    /// 内置材质 B（10.1.4）
    pub fn builtin() -> MaterialValue {
        MaterialValue {
            name: String::new(),
            color: "#c9c9c9".to_string(),
            metal: 0.05,
            rough: 0.78,
            opacity: 1.0,
        }
    }
}

/// 材质表的一个条目（10.1.3、10.6）
#[derive(Clone, Debug)]
pub struct MaterialEntry {
    pub value: MaterialValue,
    pub default: bool,
    pub meta: IndexMap<String, Value>,
}

/// 解析颜色字面量（不含或含 `#`）：返回 (规范 #rrggbb, 可选 alpha 0..255)
pub fn parse_color(lit: &str) -> Option<(String, Option<u8>)> {
    let h = lit.trim_start_matches('#');
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let digits: Vec<u8> = h.chars().map(|c| c.to_digit(16).unwrap() as u8).collect();
    let (r, g, b, a) = match digits.len() {
        3 => (digits[0] * 17, digits[1] * 17, digits[2] * 17, None),
        4 => (
            digits[0] * 17,
            digits[1] * 17,
            digits[2] * 17,
            Some(digits[3] * 17),
        ),
        6 => (
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
            None,
        ),
        8 => (
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
            Some(digits[6] * 16 + digits[7]),
        ),
        _ => return None,
    };
    Some((format!("#{:02x}{:02x}{:02x}", r, g, b), a))
}

/// 节点的自有外观属性（10.3 开头的术语）：(mat 名, (颜色, alpha))
#[derive(Clone, Debug, Default)]
pub struct OwnAppearance {
    pub mat: Option<String>,
    pub color: Option<(String, Option<u8>)>,
}

impl Compiler {
    /// 构造材质表（10.1.2、10.1.3）；只算一次
    pub fn build_material_table(&mut self) {
        if self.material_table_built {
            return;
        }
        self.material_table_built = true;
        let defs: Vec<crate::compile::MaterialDef> = self.materials.values().cloned().collect();
        let mut table: IndexMap<String, MaterialEntry> = IndexMap::new();
        let gf = self.global_frame;
        for d in &defs {
            let mut v = MaterialValue::builtin();
            v.name = d.name.clone();
            let mut default = false;
            let mut seen: Vec<String> = Vec::new();
            let mut alpha_from_color: Option<u8> = None;
            let mut wrote_opacity = false;
            let (fi, line) = (d.file_index, d.line);
            for p in &d.props {
                if seen.contains(&p.key) {
                    self.stmt_error(fi, p.line, format!("重复的属性 {}", p.key));
                    continue;
                }
                seen.push(p.key.clone());
                match p.key.as_str() {
                    "color" => match &p.value.kind {
                        ExprKind::Hash(h) => match parse_color(h) {
                            Some((c, a)) => {
                                v.color = c;
                                alpha_from_color = a;
                            }
                            None => self.stmt_error(fi, p.line, "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa"),
                        },
                        _ => self.stmt_error(fi, p.line, "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa"),
                    },
                    "metal" | "rough" | "opacity" => {
                        let ctx = Ctx {
                            frame: gf,
                            start: ROOT,
                            timing: Timing::Expand,
                        };
                        match self.eval(&p.value, ctx) {
                            Ok(Value::Num(x)) if x.is_finite() && (0.0..=1.0).contains(&x) => {
                                match p.key.as_str() {
                                    "metal" => v.metal = x,
                                    "rough" => v.rough = x,
                                    _ => {
                                        v.opacity = x;
                                        wrote_opacity = true;
                                    }
                                }
                            }
                            Ok(Value::Num(_)) => self.stmt_error(
                                fi,
                                p.line,
                                format!("material {} 的 {} 需要 0..1 之间的数", d.name, p.key),
                            ),
                            Ok(_) => self.stmt_error(
                                fi,
                                p.line,
                                format!("material {} 的 {} 需要 0..1 之间的数", d.name, p.key),
                            ),
                            Err(e) => self.stmt_error(
                                fi,
                                p.line,
                                format!("material {} 的 {} 必须是常量：{}", d.name, p.key, e.0),
                            ),
                        }
                    }
                    "default" => match &p.value.kind {
                        ExprKind::Bool(b) => default = *b,
                        _ => self.stmt_error(fi, p.line, "default 只能是 true / false"),
                    },
                    other => self.stmt_error(fi, p.line, format!("material 没有属性 {}", other)),
                }
            }
            if let Some(a) = alpha_from_color {
                if wrote_opacity {
                    self.stmt_error(
                        fi,
                        line,
                        format!("material {} 同时写了颜色的透明度与 opacity", d.name),
                    );
                } else {
                    v.opacity = a as f64 / 255.0;
                }
            }
            let stmt = crate::ast::Stmt {
                kind: crate::ast::StmtKind::Material {
                    name: d.name.clone(),
                    props: vec![],
                    metas: vec![],
                },
                line,
                file_index: fi,
            };
            let meta = self.eval_metas(&d.metas, gf, &stmt);
            table.insert(
                d.name.clone(),
                MaterialEntry {
                    value: v,
                    default,
                    meta,
                },
            );
        }
        // 默认材质的唯一性（10.1.3 第 4 条）
        let mut first_default: Option<String> = None;
        let mut errs: Vec<(u32, u32, String)> = Vec::new();
        for (name, e) in &table {
            if e.default {
                match &first_default {
                    None => first_default = Some(name.clone()),
                    Some(y) => {
                        let d = &self.materials[name];
                        errs.push((
                            d.file_index,
                            d.line,
                            format!("多个默认材质：{}（已有默认材质 {}）", name, y),
                        ));
                    }
                }
            }
        }
        for (fi, line, m) in errs {
            self.stmt_error(fi, line, m);
        }
        self.default_material = first_default;
        self.material_table = table;
    }

    /// 默认材质 D（10.1.4）
    pub fn default_material_value(&self) -> MaterialValue {
        match &self.default_material {
            Some(n) => self.material_table[n].value.clone(),
            None => MaterialValue::builtin(),
        }
    }

    /// 节点的自有外观属性；`mat` 引用未定义材质时视为没写（并报一次错）
    pub fn own_appearance(&mut self, n: NodeId) -> OwnAppearance {
        let role = self.scene.nodes[n].role.clone();
        let src = match role {
            Role::Instance { .. } | Role::TwinGroup { .. } => return OwnAppearance::default(),
            Role::Shadow { original } => {
                // 影子读孪生体：原件的属性被影子的覆盖逐键替换（4.4.4）；影子的影子沿原件链回到真实节点
                if self.scene.nodes[n].props.contains_key("mat")
                    || self.scene.nodes[n].props.contains_key("color")
                {
                    n
                } else {
                    let mut o = original;
                    while let Role::Shadow { original: oo } = self.scene.nodes[o].role {
                        if self.scene.nodes[o].props.contains_key("mat")
                            || self.scene.nodes[o].props.contains_key("color")
                        {
                            break;
                        }
                        o = oo;
                    }
                    o
                }
            }
            _ => n,
        };
        let mut out = OwnAppearance::default();
        if let Some(pv) = self.scene.nodes[src].props.get("mat").cloned() {
            match self.literal_name(&pv.expr) {
                Some(name) if !matches!(pv.expr.kind, ExprKind::DirWord(_)) => {
                    if self.material_table.contains_key(&name) {
                        out.mat = Some(name);
                    } else if !self.mat_errors_reported.contains(&src) {
                        self.mat_errors_reported.push(src);
                        self.node_error(src, format!("未定义的材质 {}", name));
                    }
                }
                _ => {
                    if !self.mat_errors_reported.contains(&src) {
                        self.mat_errors_reported.push(src);
                        self.degenerate(src, "mat 需要材质名");
                    }
                }
            }
        }
        if let Some(pv) = self.scene.nodes[src].props.get("color").cloned() {
            match &pv.expr.kind {
                ExprKind::Hash(h) => match parse_color(h) {
                    Some(c) => out.color = Some(c),
                    None => {
                        if !self.color_errors_reported.contains(&src) {
                            self.color_errors_reported.push(src);
                            self.degenerate(src, "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa");
                        }
                    }
                },
                _ => {
                    if !self.color_errors_reported.contains(&src) {
                        self.color_errors_reported.push(src);
                        self.degenerate(src, "颜色写成 #rgb、#rgba、#rrggbb 或 #rrggbbaa");
                    }
                }
            }
        }
        out
    }

    /// 材质链上的下一个节点（10.3.1）；None 表示到达终点
    fn chain_next(&self, n: NodeId) -> Option<NodeId> {
        match self.scene.nodes[n].role {
            Role::TwinOf { original } => Some(original),
            Role::Skin => None,
            _ => {
                let p = self.scene.nodes[n].parent;
                if p == ROOT {
                    None
                } else {
                    Some(p)
                }
            }
        }
    }

    /// 几何时机的有效材质 E_g(n)（10.3.2）
    pub fn effective_material_g(&mut self, n: NodeId) -> MaterialValue {
        self.build_material_table();
        let mut color: Option<(String, Option<u8>)> = None;
        let mut base: Option<MaterialValue> = None;
        let mut cur = n;
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 10000 {
                break;
            }
            let own = self.own_appearance(cur);
            if color.is_none() {
                if let Some(c) = own.color {
                    color = Some(c);
                }
            }
            if let Some(m) = own.mat {
                base = Some(self.material_table[&m].value.clone());
                break;
            }
            match self.chain_next(cur) {
                Some(x) => cur = x,
                None => {
                    // 终点：皮肤子树用 E(N)，其他用 D
                    if matches!(self.scene.nodes[cur].role, Role::Skin) {
                        let host = self.scene.nodes[cur].parent;
                        base = Some(self.effective_material(host));
                    }
                    break;
                }
            }
        }
        let mut e = base.unwrap_or_else(|| self.default_material_value());
        if let Some((c, a)) = color {
            e.color = c;
            if let Some(a) = a {
                e.opacity = a as f64 / 255.0;
            }
        }
        e
    }

    /// 外观时机的有效材质 E(n)（10.3.3）：在 E_g 上按样式序列应用
    pub fn effective_material(&mut self, n: NodeId) -> MaterialValue {
        let mut e = self.effective_material_g(n);
        let styles = self.styles_for(n);
        for (mat, color) in styles {
            if let Some(m) = mat {
                if let Some(entry) = self.material_table.get(&m) {
                    e = entry.value.clone();
                }
            }
            if let Some((c, a)) = color {
                e.color = c;
                if let Some(a) = a {
                    e.opacity = a as f64 / 255.0;
                }
            }
        }
        e
    }

    /// 作用于 n 的样式序列（10.3.3）：每项 (mat, color)，按语句顺序。
    /// 样式动作由语义层登记在 `style_actions`（选择集的成员集合）；副本与影子并上原件的。
    pub fn styles_for(&mut self, n: NodeId) -> Vec<(Option<String>, Option<(String, Option<u8>)>)> {
        if self.style_actions.is_empty() {
            return Vec::new();
        }
        let mut targets = vec![n];
        match self.scene.nodes[n].role {
            Role::TwinOf { original } | Role::Shadow { original } => targets.push(original),
            _ => {}
        }
        let mut out = Vec::new();
        for (members, mat, color) in self.style_actions.clone() {
            let hit = targets
                .iter()
                .any(|&t| members.iter().any(|&m| m == t || self.scene.is_ancestor(m, t)))
                && !self.scene.in_skin(n);
            if hit {
                out.push((mat, color));
            }
        }
        out
    }

    /// `where` 里的 mat 字段：E_g 的 name；空串为缺失（10.5.3）
    pub fn where_mat_name(&mut self, n: NodeId) -> Option<String> {
        if crate::ast::CURVE_KINDS.contains(&self.scene.nodes[n].kind.as_str()) {
            return None;
        }
        let e = self.effective_material_g(n);
        if e.name.is_empty() {
            None
        } else {
            Some(e.name)
        }
    }
}
