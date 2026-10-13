//! 语法分析（spec/01 1.3、1.5）：手写递归下降。错误恢复按 1.1.4 第 1 条：出错的语句连同块丢弃，从下一条继续。

use crate::ast::*;
use crate::diag::Diagnostics;
use crate::lexer::{lex, Seg, Tok, Token};

pub struct ParseError {
    pub line: u32,
    pub msg: String,
}

type PResult<T> = Result<T, ParseError>;

/// （标签, 属性, 元数据）
type Items = (Vec<(String, u32)>, Vec<Prop>, Vec<Prop>);

fn perr<T>(line: u32, msg: impl Into<String>) -> PResult<T> {
    Err(ParseError {
        line,
        msg: msg.into(),
    })
}

pub struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    i: usize,
    file_index: u32,
    file: String,
    diags: &'a mut Diagnostics,
}

/// 解析一个文件。词法与语法错误记入 `diags`，返回能解析出来的全部语句。
pub fn parse_file(src: &str, file_index: u32, file: &str, diags: &mut Diagnostics) -> FileAst {
    let toks = lex(src);
    let mut p = Parser {
        src,
        toks,
        i: 0,
        file_index,
        file: file.to_string(),
        diags,
    };
    p.parse()
}

impl<'a> Parser<'a> {
    // —— 记号访问 ——
    fn tok(&self) -> &Tok {
        &self.toks[self.i.min(self.toks.len() - 1)].tok
    }
    fn at(&self, k: usize) -> &Tok {
        let j = self.i + k;
        if j < self.toks.len() {
            &self.toks[j].tok
        } else {
            &Tok::Nl
        }
    }
    fn token(&self) -> &Token {
        &self.toks[self.i.min(self.toks.len() - 1)]
    }
    fn eof(&self) -> bool {
        self.i >= self.toks.len()
    }
    fn line(&self) -> u32 {
        self.token().line
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.i.min(self.toks.len() - 1)].clone();
        if self.i < self.toks.len() {
            self.i += 1;
        }
        t
    }
    fn error(&mut self, line: u32, msg: impl Into<String>) {
        let file = self.file.clone();
        self.diags.error(self.file_index, &file, line, None, msg);
    }
    fn is_ident(&self, s: &str) -> bool {
        matches!(self.tok(), Tok::Ident(x) if x == s)
    }

    // —— 文件 ——
    fn parse(&mut self) -> FileAst {
        let mut out = FileAst::default();
        // 文件头：第一个非空行只由元数据项组成
        while matches!(self.tok(), Tok::Nl) && !self.eof() {
            self.bump();
        }
        if matches!(self.tok(), Tok::MetaKey(_)) {
            let save = self.i;
            let line = self.line();
            match self.parse_metas_line() {
                Ok(metas) => {
                    out.header = metas;
                    out.header_line = Some(line);
                }
                Err(_) => self.i = save,
            }
        }
        out.stmts = self.parse_stmts(false, 0);
        out
    }

    /// 只由元数据项组成的一行（文件头）；不是则 Err 且不消耗
    fn parse_metas_line(&mut self) -> PResult<Vec<Prop>> {
        let mut metas = Vec::new();
        loop {
            match self.tok().clone() {
                Tok::MetaKey(k) if matches!(self.at(1), Tok::Eq) => {
                    let line = self.line();
                    self.bump();
                    self.bump();
                    let value = self.parse_tuple()?;
                    metas.push(Prop { key: k, value, line });
                }
                Tok::Nl => {
                    self.bump();
                    return Ok(metas);
                }
                _ => return perr(self.line(), "不是文件头"),
            }
        }
    }

    /// 语句序列；`in_block` 时遇到 `}` 结束并消耗它
    fn parse_stmts(&mut self, in_block: bool, brace_line: u32) -> Vec<Stmt> {
        let mut out = Vec::new();
        loop {
            while matches!(self.tok(), Tok::Nl) && !self.eof() {
                self.bump();
            }
            if self.eof() {
                if in_block {
                    self.error(brace_line, "缺少 }");
                }
                return out;
            }
            match self.tok() {
                Tok::RBrace => {
                    if in_block {
                        self.bump();
                        return out;
                    }
                    let line = self.line();
                    self.bump();
                    self.error(line, "多余的 }");
                }
                Tok::LBrace => {
                    let line = self.line();
                    self.error(line, "{ 前缺少语句");
                    self.bump();
                    self.skip_block_body();
                }
                _ => {
                    let line = self.line();
                    match self.parse_statement(in_block) {
                        Ok(Some(s)) => out.push(s),
                        Ok(None) => {}
                        Err(e) => {
                            self.error(e.line.max(line.min(e.line)), e.msg);
                            self.recover(in_block);
                        }
                    }
                }
            }
        }
    }

    /// 跳过到语句结束：到 NL（消耗）；遇到 `{` 则跳过配对的块并停止；块内遇到 `}` 停止（不消耗）
    fn recover(&mut self, in_block: bool) {
        loop {
            if self.eof() {
                return;
            }
            match self.tok() {
                Tok::Nl => {
                    self.bump();
                    return;
                }
                Tok::LBrace => {
                    self.bump();
                    self.skip_block_body();
                    return;
                }
                Tok::RBrace if in_block => return,
                Tok::RBrace => {
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// 已消耗 `{`，跳到配对的 `}`（含）
    fn skip_block_body(&mut self) {
        let mut depth = 1;
        while !self.eof() && depth > 0 {
            match self.bump().tok {
                Tok::LBrace => depth += 1,
                Tok::RBrace => depth -= 1,
                _ => {}
            }
        }
    }

    // —— 语句 ——
    fn parse_statement(&mut self, in_block: bool) -> PResult<Option<Stmt>> {
        let line = self.line();
        let first = self.tok().clone();
        let kw = match &first {
            Tok::Ident(s) => s.clone(),
            Tok::Error(m) => return perr(line, m.clone()),
            Tok::MetaKey(_) => return perr(line, "语句缺少种类词"),
            Tok::Path(_) => return perr(line, "语句缺少种类词（路径不能作语句开头）"),
            other => return perr(line, format!("语句缺少种类词，得到 “{}”", other)),
        };
        if is_keyword(&kw) {
            return perr(line, format!("关键字 {} 用错了位置", kw));
        }
        self.bump();
        let kind = match kw.as_str() {
            "def" => self.parse_def(line, in_block)?,
            "let" => self.parse_let(line, in_block)?,
            "import" => self.parse_import(line, in_block)?,
            "material" => self.parse_material(line, in_block)?,
            "over" => self.parse_over(line, in_block)?,
            "drop" => self.parse_drop(line, in_block)?,
            "concept" => self.parse_concept(line, in_block)?,
            "pattern" => self.parse_pattern(line, in_block)?,
            "check" => self.parse_check(line, in_block)?,
            "style" => self.parse_style_variant(line, in_block, false)?,
            "variant" => self.parse_style_variant(line, in_block, true)?,
            "copy" => self.parse_copy(line, in_block)?,
            _ => self.parse_generic(kw, line, in_block)?,
        };
        Ok(kind.map(|k| Stmt {
            kind: k,
            line,
            file_index: self.file_index,
        }))
    }

    /// 语句结尾：块、NL 或（块内的）`}`。返回块（若有）。
    fn finish(&mut self, in_block: bool) -> PResult<Option<Vec<Stmt>>> {
        match self.tok() {
            Tok::LBrace => {
                let bl = self.line();
                self.bump();
                let body = self.parse_stmts(true, bl);
                Ok(Some(body))
            }
            Tok::Nl => {
                self.bump();
                Ok(None)
            }
            Tok::RBrace if in_block => Ok(None),
            Tok::Error(m) => perr(self.line(), m.clone()),
            Tok::MetaKey(_) => perr(self.line(), "元数据写成 ~名字=值，要写在属性之后"),
            other => perr(self.line(), format!("多余的内容：{}", other)),
        }
    }

    fn only_top(&self, what: &str, in_block: bool, line: u32) -> PResult<()> {
        if in_block {
            perr(line, format!("{} 只能写在最外层", what))
        } else {
            Ok(())
        }
    }

    /// `{ tag | prop | meta }`；`allow_tags` 为假时井号记号报错
    fn parse_items(&mut self, allow_tags: bool) -> PResult<Items> {
        let mut tags = Vec::new();
        let mut props = Vec::new();
        let mut metas = Vec::new();
        loop {
            match self.tok().clone() {
                Tok::Hash(h) => {
                    let line = self.line();
                    if !allow_tags {
                        return perr(line, "这种语句不接受标签");
                    }
                    self.bump();
                    tags.push((h, line));
                }
                Tok::Ident(k) if matches!(self.at(1), Tok::Eq) => {
                    let line = self.line();
                    if is_keyword(&k) {
                        return perr(line, format!("关键字 {} 用错了位置", k));
                    }
                    self.bump();
                    self.bump();
                    let value = self.parse_tuple()?;
                    props.push(Prop { key: k, value, line });
                }
                Tok::MetaKey(k) => {
                    let line = self.line();
                    if !matches!(self.at(1), Tok::Eq) {
                        return perr(line, "元数据写成 ~名字=值");
                    }
                    self.bump();
                    self.bump();
                    let value = self.parse_tuple()?;
                    metas.push(Prop { key: k, value, line });
                }
                _ => return Ok((tags, props, metas)),
            }
        }
    }

    fn parse_generic(&mut self, kind: String, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        // 名字：紧跟种类词的标识符，且后面不紧接 =
        let mut name = None;
        if let Tok::Ident(n) = self.tok().clone() {
            let eq_glued =
                matches!(self.at(1), Tok::Eq) && self.toks.get(self.i + 1).is_some_and(|t| t.glued);
            if !eq_glued {
                if is_keyword(&n) {
                    return perr(self.line(), format!("关键字 {} 用错了位置", n));
                }
                self.bump();
                name = Some(n);
            }
        }
        let (tags, props, metas) = self.parse_items(true)?;
        // 名字写错位置
        if let Tok::Ident(n) = self.tok() {
            if !matches!(self.at(1), Tok::Eq) {
                let n = n.clone();
                return perr(self.line(), format!("名字要写在种类词之后（“{}”）", n));
            }
        }
        let block = self.finish(in_block)?;
        let is_solid = SOLID_KINDS.contains(&kind.as_str());
        let is_curve = CURVE_KINDS.contains(&kind.as_str());
        let has_of = props.iter().any(|p| p.key == "of");
        if block.is_some() {
            if is_solid || is_curve {
                self.error(line, format!("{} 不能有子语句", kind));
                return Ok(None);
            }
            if (kind == "mirror" || kind == "rotate") && has_of {
                self.error(line, "写了 of= 就是复制已有对象，不能再有子语句");
                return Ok(None);
            }
            if !is_builtin_kind(&kind) {
                self.error(line, format!("模板实例 {} 不能有子语句", kind));
                return Ok(None);
            }
        }
        Ok(Some(StmtKind::Generic {
            kind,
            name,
            tags,
            props,
            metas,
            block,
        }))
    }

    fn parse_copy(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        let name = match self.tok().clone() {
            Tok::Ident(n) if !matches!(self.at(1), Tok::Eq) => {
                if is_keyword(&n) {
                    return perr(line, format!("关键字 {} 用错了位置", n));
                }
                self.bump();
                n
            }
            _ => return perr(line, "copy 需要名字"),
        };
        let (tags, props, metas) = self.parse_items(false)?;
        let block = self.finish(in_block)?;
        if !props.iter().any(|p| p.key == "of") {
            self.error(line, "copy 需要 of=原对象");
            return Ok(None);
        }
        Ok(Some(StmtKind::Copy {
            name,
            tags,
            props,
            metas,
            block,
        }))
    }

    fn parse_def(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("def", in_block, line)?;
        let name = match self.tok().clone() {
            Tok::Ident(n) => {
                self.bump();
                n
            }
            _ => return perr(line, "def 语法应为 def 名字(参数…) [: 基类型] { … }"),
        };
        if is_keyword(&name) {
            return perr(line, format!("关键字 {} 用错了位置", name));
        }
        let mut params = Vec::new();
        if matches!(self.tok(), Tok::LParen) {
            self.bump();
            if !matches!(self.tok(), Tok::RParen) {
                loop {
                    let pl = self.line();
                    let pname = match self.tok().clone() {
                        Tok::Ident(n) => {
                            self.bump();
                            n
                        }
                        _ => return perr(pl, "参数名应是标识符"),
                    };
                    let mut default = None;
                    if matches!(self.tok(), Tok::Eq) {
                        self.bump();
                        default = Some(self.parse_item()?);
                    }
                    params.push(Param {
                        name: pname,
                        default,
                        line: pl,
                    });
                    match self.tok() {
                        Tok::Comma => {
                            self.bump();
                        }
                        Tok::RParen => break,
                        _ => return perr(self.line(), "参数表写成 (a, b=1)"),
                    }
                }
            }
            self.bump(); // )
        }
        let mut base = None;
        if matches!(self.tok(), Tok::Colon) {
            self.bump();
            let bname = match self.tok().clone() {
                Tok::Ident(n) => {
                    self.bump();
                    n
                }
                _ => return perr(self.line(), "派生写成 def 子类型 : 基类型"),
            };
            let mut overrides = Vec::new();
            if matches!(self.tok(), Tok::LParen) {
                self.bump();
                if !matches!(self.tok(), Tok::RParen) {
                    loop {
                        let ol = self.line();
                        let (k, v) = match (self.tok().clone(), self.at(1).clone()) {
                            (Tok::Ident(k), Tok::Eq) => {
                                self.bump();
                                self.bump();
                                (k, self.parse_item()?)
                            }
                            _ => return perr(ol, "覆盖表写成 (参数=值, …)"),
                        };
                        overrides.push(Prop {
                            key: k,
                            value: v,
                            line: ol,
                        });
                        match self.tok() {
                            Tok::Comma => {
                                self.bump();
                            }
                            Tok::RParen => break,
                            _ => return perr(self.line(), "覆盖表写成 (参数=值, …)"),
                        }
                    }
                }
                self.bump();
            }
            base = Some((bname, overrides));
        }
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "def 的括号外只能写元数据");
        }
        let body = match self.finish(false)? {
            Some(b) => b,
            None => {
                self.error(line, "def 需要块");
                return Ok(None);
            }
        };
        Ok(Some(StmtKind::Def {
            name,
            params,
            base,
            metas,
            body,
        }))
    }

    fn parse_let(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        let name = match (self.tok().clone(), self.at(1).clone()) {
            (Tok::Ident(n), Tok::Eq) => {
                self.bump();
                self.bump();
                n
            }
            _ => return perr(line, "let 写成 let 名字 = 表达式"),
        };
        if is_keyword(&name) {
            return perr(line, format!("关键字 {} 用错了位置", name));
        }
        let value = self.parse_tuple()?;
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "let 之后只能写元数据");
        }
        if self.finish(in_block)?.is_some() {
            self.error(line, "let 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Let { name, value, metas }))
    }

    fn parse_import(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("import", in_block, line)?;
        let path = match self.tok().clone() {
            Tok::Str(s) => {
                self.bump();
                s
            }
            _ => return perr(line, "import 写成 import \"文件.s3d\""),
        };
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "import 之后只能写元数据");
        }
        if self.finish(false)?.is_some() {
            self.error(line, "import 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Import { path, metas }))
    }

    fn parse_material(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("material", in_block, line)?;
        let name = match self.tok().clone() {
            Tok::Ident(n) if !matches!(self.at(1), Tok::Eq) => {
                self.bump();
                n
            }
            _ => return perr(line, "material 写成 material 名字 color=… metal=…"),
        };
        let (_t, props, metas) = self.parse_items(false)?;
        if self.finish(false)?.is_some() {
            self.error(line, "material 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Material { name, props, metas }))
    }

    /// 选择器：一个表达式；为空时报给定文案
    fn parse_selector(&mut self, empty_msg: &str) -> PResult<Expr> {
        match self.tok() {
            Tok::Nl | Tok::LBrace | Tok::RBrace | Tok::Colon | Tok::MetaKey(_) => {
                perr(self.line(), empty_msg)
            }
            Tok::Ident(_) if matches!(self.at(1), Tok::Eq) => perr(self.line(), empty_msg),
            _ => self.parse_tuple(),
        }
    }

    fn parse_over(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        let selector = self.parse_selector("over 需要选择器")?;
        let mut retype = None;
        if matches!(self.tok(), Tok::Colon) {
            self.bump();
            match self.tok().clone() {
                Tok::Ident(t) if !is_keyword(&t) => {
                    self.bump();
                    retype = Some(t);
                }
                _ => return perr(self.line(), "换类型写成 over 对象 : 类型"),
            }
        }
        let (_t, props, metas) = self.parse_items(false)?;
        if !matches!(self.tok(), Tok::Nl | Tok::LBrace | Tok::RBrace) {
            return perr(self.line(), "属性之后不能再出现选择器");
        }
        let block = self.finish(in_block)?;
        let _ = line;
        Ok(Some(StmtKind::Over {
            selector,
            retype,
            props,
            metas,
            block,
        }))
    }

    fn parse_drop(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        let selector = self.parse_selector("drop 需要选择器")?;
        if matches!(self.tok(), Tok::Colon) {
            return perr(self.line(), "drop 不接受换类型");
        }
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "drop 不接受属性");
        }
        if self.finish(in_block)?.is_some() {
            self.error(line, "drop 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Drop { selector, metas }))
    }

    fn parse_concept(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("concept", in_block, line)?;
        let name = match self.tok().clone() {
            Tok::Ident(n) if !is_keyword(&n) => {
                self.bump();
                n
            }
            _ => return perr(line, "concept 写成 concept 名字 = 表达式"),
        };
        let mut parent = None;
        if matches!(self.tok(), Tok::Colon) {
            self.bump();
            match self.tok().clone() {
                Tok::Ident(p) if !is_keyword(&p) => {
                    self.bump();
                    parent = Some(p);
                }
                _ => return perr(self.line(), "上位概念写成 concept 名字 : 上位 = …"),
            }
        }
        if !matches!(self.tok(), Tok::Eq) {
            return perr(self.line(), "concept 写成 concept 名字 = 表达式");
        }
        self.bump();
        let rhs = if self.is_ident("where") {
            if parent.is_none() {
                return perr(
                    self.line(),
                    "关键字 where 用错了位置（没有上位概念时不能以 where 开头）",
                );
            }
            self.bump();
            ConceptRhs::Where(self.parse_or()?)
        } else {
            ConceptRhs::Expr(self.parse_tuple()?)
        };
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "concept 之后只能写元数据");
        }
        if self.finish(false)?.is_some() {
            self.error(line, "concept 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Concept {
            name,
            parent,
            rhs,
            metas,
        }))
    }

    fn parse_pattern(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("pattern", in_block, line)?;
        let name = match self.tok().clone() {
            Tok::Ident(n) if !is_keyword(&n) => {
                self.bump();
                n
            }
            _ => return perr(line, "pattern 写成 pattern 名字(角色, …) { 约束 }"),
        };
        if !matches!(self.tok(), Tok::LParen) {
            return perr(self.line(), "pattern 需要角色表 (角色, …)");
        }
        self.bump();
        let mut roles = Vec::new();
        loop {
            match self.tok().clone() {
                Tok::Ident(r) if !is_keyword(&r) => {
                    self.bump();
                    let set = if matches!(self.tok(), Tok::Star) && self.token().glued {
                        self.bump();
                        true
                    } else {
                        false
                    };
                    roles.push((r, set));
                }
                _ => return perr(self.line(), "角色名应是标识符"),
            }
            match self.tok() {
                Tok::Comma => {
                    self.bump();
                }
                Tok::RParen => {
                    self.bump();
                    break;
                }
                _ => return perr(self.line(), "角色表写成 (a, b*, …)"),
            }
        }
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "pattern 的括号后只能写元数据");
        }
        if !matches!(self.tok(), Tok::LBrace) {
            return perr(self.line(), "pattern 需要块");
        }
        let brace_line = self.line();
        self.bump();
        let mut constraints = Vec::new();
        loop {
            while matches!(self.tok(), Tok::Nl) && !self.eof() {
                self.bump();
            }
            if self.eof() {
                self.error(brace_line, "缺少 }");
                break;
            }
            if matches!(self.tok(), Tok::RBrace) {
                self.bump();
                break;
            }
            if matches!(self.tok(), Tok::LBrace) {
                let l = self.line();
                self.error(l, "pattern 的块里不能再有 {");
                self.bump();
                self.skip_block_body();
                continue;
            }
            let cl = self.line();
            match self.parse_constraint(&roles) {
                Ok(c) => constraints.push(ConstraintLine {
                    line: cl,
                    constraint: c,
                }),
                Err(e) => {
                    self.error(e.line, e.msg);
                    while !self.eof() && !matches!(self.tok(), Tok::Nl | Tok::RBrace) {
                        self.bump();
                    }
                }
            }
        }
        // 块后：同一行可以继续别的语句；这里不要求 NL
        Ok(Some(StmtKind::Pattern {
            name,
            roles,
            metas,
            constraints,
        }))
    }

    fn parse_constraint(&mut self, roles: &[(String, bool)]) -> PResult<Constraint> {
        if let (Tok::Ident(r), Tok::Ident(kw)) = (self.tok().clone(), self.at(1).clone()) {
            if kw == "in" {
                if !roles.iter().any(|(n, _)| *n == r) {
                    return perr(self.line(), format!("{} 不是这个 pattern 的角色", r));
                }
                self.bump();
                self.bump();
                let expr = self.parse_item()?;
                self.end_of_line()?;
                return Ok(Constraint::RoleIn { role: r, expr });
            }
        }
        let lhs = self.parse_item()?;
        if let Tok::Ident(w) = self.tok().clone() {
            if let Some(rel) = RelWord::parse(&w) {
                self.bump();
                let rhs = self.parse_item()?;
                self.end_of_line()?;
                return Ok(Constraint::Relation { lhs, rel, rhs });
            }
        }
        self.end_of_line()?;
        Ok(Constraint::Boolean(lhs))
    }

    fn end_of_line(&mut self) -> PResult<()> {
        match self.tok() {
            Tok::Nl => {
                self.bump();
                Ok(())
            }
            Tok::RBrace => Ok(()),
            Tok::Error(m) => perr(self.line(), m.clone()),
            other => perr(self.line(), format!("多余的内容：{}", other)),
        }
    }

    fn parse_check(&mut self, line: u32, in_block: bool) -> PResult<Option<StmtKind>> {
        self.only_top("check", in_block, line)?;
        let mut message = None;
        if let Tok::Str(s) = self.tok().clone() {
            self.bump();
            message = Some(s);
        }
        if matches!(self.tok(), Tok::Nl | Tok::RBrace | Tok::MetaKey(_)) {
            return perr(line, "check 写成：check \"说明\" 表达式");
        }
        let expr = self.parse_tuple()?;
        let source_text = self.src[expr.span.start..expr.span.end].trim().to_string();
        let (_t, props, metas) = self.parse_items(false)?;
        if let Some(p) = props.first() {
            return perr(p.line, "check 之后只能写元数据");
        }
        if self.finish(false)?.is_some() {
            self.error(line, "check 不能有块");
            return Ok(None);
        }
        Ok(Some(StmtKind::Check {
            message,
            expr,
            source_text,
            metas,
        }))
    }

    fn parse_style_variant(&mut self, line: u32, in_block: bool, variant: bool) -> PResult<Option<StmtKind>> {
        let what = if variant { "variant" } else { "style" };
        self.only_top(what, in_block, line)?;
        let selector = self.parse_selector(&format!("{} 需要选择器", what))?;
        let (_t, props, metas) = self.parse_items(false)?;
        if !matches!(self.tok(), Tok::Nl | Tok::LBrace | Tok::RBrace) {
            return perr(
                self.line(),
                format!("{} 只接受 key=value，看不懂 “{}”", what, self.tok()),
            );
        }
        if self.finish(false)?.is_some() {
            self.error(line, format!("{} 不能有块", what));
            return Ok(None);
        }
        if props.is_empty() && !variant {
            self.error(line, "style 需要至少一个属性（mat= 或 color=）");
            return Ok(None);
        }
        Ok(Some(if variant {
            StmtKind::Variant {
                selector,
                props,
                metas,
            }
        } else {
            StmtKind::Style {
                selector,
                props,
                metas,
            }
        }))
    }

    // —— 表达式（1.5.1）——
    fn span_from(&self, start_tok: usize) -> Span {
        let a = &self.toks[start_tok];
        let b = &self.toks[(self.i.max(start_tok + 1) - 1).min(self.toks.len() - 1)];
        Span {
            line: a.line,
            start: a.start,
            end: b.end,
        }
    }
    fn mk(&self, kind: ExprKind, start_tok: usize) -> Expr {
        Expr {
            kind,
            span: self.span_from(start_tok),
        }
    }
    fn binary(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
        let span = Span {
            line: lhs.span.line,
            start: lhs.span.start,
            end: rhs.span.end,
        };
        Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        }
    }

    pub fn parse_tuple(&mut self) -> PResult<Expr> {
        let st = self.i;
        let first = self.parse_item()?;
        if !matches!(self.tok(), Tok::Comma) {
            return Ok(first);
        }
        let mut items = vec![first];
        while matches!(self.tok(), Tok::Comma) {
            self.bump();
            items.push(self.parse_item()?);
        }
        Ok(self.mk(ExprKind::Tuple(items), st))
    }

    fn parse_item(&mut self) -> PResult<Expr> {
        let lhs = self.parse_or()?;
        if self.is_ident("where") {
            self.bump();
            let cond = self.parse_or()?;
            if self.is_ident("where") {
                return perr(self.line(), "where 不能连写（a where b where c），要加括号");
            }
            let span = Span {
                line: lhs.span.line,
                start: lhs.span.start,
                end: cond.span.end,
            };
            return Ok(Expr {
                kind: ExprKind::Where {
                    lhs: Box::new(lhs),
                    cond: Box::new(cond),
                },
                span,
            });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        loop {
            let is_or = matches!(self.tok(), Tok::Pipe) || self.is_ident("or");
            if !is_or {
                return Ok(lhs);
            }
            self.bump();
            let rhs = self.parse_and()?;
            lhs = Self::binary(BinOp::Or, lhs, rhs);
        }
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_not()?;
        loop {
            let is_and = matches!(self.tok(), Tok::Amp) || self.is_ident("and");
            if !is_and {
                return Ok(lhs);
            }
            self.bump();
            let rhs = self.parse_not()?;
            lhs = Self::binary(BinOp::And, lhs, rhs);
        }
    }

    fn parse_not(&mut self) -> PResult<Expr> {
        if self.is_ident("not") {
            let st = self.i;
            self.bump();
            let inner = self.parse_not()?;
            return Ok(self.mk(ExprKind::Not(Box::new(inner)), st));
        }
        self.parse_cmp()
    }

    fn cmp_op(&self) -> Option<BinOp> {
        Some(match self.tok() {
            Tok::EqEq => BinOp::Eq,
            Tok::Ne => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Gt => BinOp::Gt,
            Tok::Le => BinOp::Le,
            Tok::Ge => BinOp::Ge,
            _ => return None,
        })
    }

    fn parse_cmp(&mut self) -> PResult<Expr> {
        let lhs = self.parse_sum()?;
        if let Some(op) = self.cmp_op() {
            self.bump();
            let rhs = self.parse_sum()?;
            if self.cmp_op().is_some() || matches!(self.tok(), Tok::Tilde) {
                return perr(self.line(), "比较不能连写（a < b < c），要加括号");
            }
            return Ok(Self::binary(op, lhs, rhs));
        }
        if matches!(self.tok(), Tok::Tilde) {
            self.bump();
            return match self.tok().clone() {
                Tok::Regex(r) => {
                    let end = self.span_from(self.i);
                    self.bump();
                    let span = Span {
                        line: lhs.span.line,
                        start: lhs.span.start,
                        end: end.end,
                    };
                    Ok(Expr {
                        kind: ExprKind::Match {
                            lhs: Box::new(lhs),
                            regex: r,
                        },
                        span,
                    })
                }
                Tok::Error(m) => perr(self.line(), m),
                _ => perr(self.line(), "~ 的右边要写正则 /…/"),
            };
        }
        Ok(lhs)
    }

    fn parse_sum(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_term()?;
        loop {
            let op = match self.tok() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => return Ok(lhs),
            };
            self.bump();
            let rhs = self.parse_term()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    fn parse_term(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.tok() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => return Ok(lhs),
            };
            self.bump();
            let rhs = self.parse_unary()?;
            lhs = Self::binary(op, lhs, rhs);
        }
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        match self.tok() {
            Tok::Minus => {
                let st = self.i;
                self.bump();
                let inner = self.parse_unary()?;
                Ok(self.mk(ExprKind::Neg(Box::new(inner)), st))
            }
            Tok::Plus => perr(self.line(), "没有一元 +"),
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let st = self.i;
        let line = self.line();
        match self.tok().clone() {
            Tok::Number(n) => {
                self.bump();
                Ok(self.mk(ExprKind::Number(n), st))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(self.mk(ExprKind::Str(s), st))
            }
            Tok::Hash(h) => {
                self.bump();
                Ok(self.mk(ExprKind::Hash(h), st))
            }
            Tok::DirWord(w) => {
                self.bump();
                Ok(self.mk(ExprKind::DirWord(w), st))
            }
            Tok::Ident(name) => {
                if name == "true" || name == "false" {
                    self.bump();
                    return Ok(self.mk(ExprKind::Bool(name == "true"), st));
                }
                if is_keyword(&name) {
                    return perr(line, format!("关键字 {} 用错了位置", name));
                }
                self.bump();
                // 调用：名字紧跟 (
                if matches!(self.tok(), Tok::LParen) {
                    if !self.token().glued {
                        return perr(self.line(), "函数调用的括号要紧跟函数名（sin(30)）");
                    }
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.tok(), Tok::RParen) {
                        loop {
                            args.push(self.parse_item()?);
                            match self.tok() {
                                Tok::Comma => {
                                    self.bump();
                                }
                                Tok::RParen => break,
                                Tok::Error(m) => return perr(self.line(), m.clone()),
                                other => return perr(self.line(), format!("多余的内容：{}", other)),
                            }
                        }
                    }
                    self.bump();
                    return Ok(self.mk(ExprKind::Call { name, args }, st));
                }
                if matches!(self.tok(), Tok::LBracket) && self.token().glued {
                    let (coords, axis) = self.parse_coord_suffix()?;
                    return Ok(self.mk(
                        ExprKind::PathRef {
                            segs: vec![Seg::Ident(name)],
                            coords: Some(coords),
                            axis,
                        },
                        st,
                    ));
                }
                if matches!(self.tok(), Tok::LBracket) {
                    return perr(self.line(), "坐标后缀的 [ 要紧跟名字（B[1,2,3]）");
                }
                Ok(self.mk(ExprKind::Ident(name), st))
            }
            Tok::Path(segs) => {
                self.bump();
                if matches!(self.tok(), Tok::LParen) && self.token().glued {
                    return perr(self.line(), "函数名不能带路径");
                }
                let mut coords = None;
                let mut axis = None;
                if matches!(self.tok(), Tok::LBracket) {
                    if !self.token().glued {
                        return perr(self.line(), "坐标后缀的 [ 要紧跟路径（B[1,2,3]）");
                    }
                    let (c, a) = self.parse_coord_suffix()?;
                    coords = Some(c);
                    axis = a;
                }
                Ok(self.mk(ExprKind::PathRef { segs, coords, axis }, st))
            }
            Tok::LParen => {
                self.bump();
                let inner = self.parse_tuple()?;
                match self.tok() {
                    Tok::RParen => {
                        self.bump();
                    }
                    Tok::Error(m) => return perr(self.line(), m.clone()),
                    other => return perr(self.line(), format!("多余的内容：{}", other)),
                }
                Ok(Expr {
                    kind: inner.kind,
                    span: self.span_from(st),
                })
            }
            Tok::LBracket => {
                self.bump();
                if matches!(self.tok(), Tok::RBracket) {
                    return perr(line, "列表不能为空（[] 不合法）");
                }
                let mut items = vec![self.parse_tuple()?];
                loop {
                    match self.tok() {
                        Tok::Semi => {
                            self.bump();
                            if matches!(self.tok(), Tok::RBracket) {
                                return perr(self.line(), "列表项不能为空（[a;] 不合法）");
                            }
                            items.push(self.parse_tuple()?);
                        }
                        Tok::RBracket => {
                            self.bump();
                            break;
                        }
                        Tok::Error(m) => return perr(self.line(), m.clone()),
                        other => return perr(self.line(), format!("多余的内容：{}", other)),
                    }
                }
                if matches!(self.tok(), Tok::Axis(_)) {
                    return perr(self.line(), "列表后面不能再跟后缀");
                }
                Ok(self.mk(ExprKind::List(items), st))
            }
            Tok::Error(m) => perr(line, m),
            Tok::Nl | Tok::RBrace | Tok::LBrace | Tok::RParen | Tok::RBracket | Tok::Comma | Tok::Semi => {
                perr(line, "表达式不完整")
            }
            Tok::MetaKey(_) => perr(line, "元数据写成 ~名字=值，要写在属性之后"),
            other => perr(line, format!("表达式不完整，得到 “{}”", other)),
        }
    }

    /// 已在紧跟的 `[` 上：解析 `[ tuple ]` 与可选的分量后缀
    fn parse_coord_suffix(&mut self) -> PResult<(Vec<Expr>, Option<char>)> {
        self.bump(); // [
        let t = self.parse_tuple()?;
        match self.tok() {
            Tok::RBracket => {
                self.bump();
            }
            Tok::Error(m) => return perr(self.line(), m.clone()),
            other => return perr(self.line(), format!("多余的内容：{}", other)),
        }
        let coords = match t.kind {
            ExprKind::Tuple(items) => items,
            _ => vec![t],
        };
        let axis = if let Tok::Axis(a) = self.tok() {
            let a = *a;
            self.bump();
            Some(a)
        } else {
            None
        };
        Ok((coords, axis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> (FileAst, Diagnostics) {
        let mut d = Diagnostics::default();
        let ast = parse_file(src, 0, "t.s3d", &mut d);
        (ast, d)
    }

    #[test]
    fn corpus_examples_parse_without_errors() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/examples");
        let mut n = 0;
        for name in [
            "basics",
            "curves",
            "rotation",
            "semantics",
            "spaceport",
            "spaceport_blueprint",
            "spaceport_edit",
            "lib/semantics",
        ] {
            let src = std::fs::read_to_string(format!("{}/{}.s3d", dir, name)).unwrap();
            let (ast, d) = parse(&src);
            assert!(d.errors.is_empty(), "{}: {:?}", name, d.errors);
            n += ast.stmts.len();
        }
        assert!(n > 100);
    }

    #[test]
    fn header_and_generic() {
        let (ast, d) = parse("~lang=2 ~note=\"x\"\nbox floor #f size=40,1,24 pin=top ~src=\"n\"\n");
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        assert_eq!(ast.header.len(), 2);
        match &ast.stmts[0].kind {
            StmtKind::Generic {
                kind,
                name,
                tags,
                props,
                metas,
                block,
            } => {
                assert_eq!(kind, "box");
                assert_eq!(name.as_deref(), Some("floor"));
                assert_eq!(tags[0].0, "f");
                assert_eq!(props.len(), 2);
                assert!(matches!(props[0].value.kind, ExprKind::Tuple(ref v) if v.len() == 3));
                assert_eq!(metas.len(), 1);
                assert!(block.is_none());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn blocks_and_recovery() {
        let (ast, d) = parse("group g { box a\n box b size=1. }\nbox c\n{\n box z }\ncyl d size=1,1,1 { }\n");
        // box b 的 1. 是词法错误：该语句丢弃，g 仍有 a
        assert!(matches!(&ast.stmts[0].kind, StmtKind::Generic { block: Some(b), .. } if b.len() == 1));
        // box c 正常；孤立的 { 报错并丢弃块；cyl 带块报错
        assert_eq!(ast.stmts.len(), 2);
        let msgs: Vec<&str> = d.errors.iter().map(|e| e.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("小数点")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("{ 前缺少语句")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("不能有子语句")), "{:?}", msgs);
    }

    #[test]
    fn expressions() {
        let (ast, d) = parse("let a = kind(box) where h > 1 and not #x | all() - y\nlet b = B.top[0.5, 1].y + (1,2,3) * 2\nover kind(box) where name ~ /^t/ : cyl r=1\n");
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        assert_eq!(ast.stmts.len(), 3);
        assert!(matches!(&ast.stmts[2].kind, StmtKind::Over { retype: Some(t), .. } if t == "cyl"));
        let (_, d) = parse("let a = 1 < 2 < 3\nlet b = +3\nlet c = sin (3)\nlet d = [1;]\n");
        assert_eq!(d.errors.len(), 4, "{:?}", d.errors);
    }

    #[test]
    fn pattern_and_check() {
        let src = "pattern crossing(deck, piers*) ~k=1 {\n  deck in kind(box)\n  deck on piers\n  count(piers) >= 2\n}\ncheck \"x\" count(deck) == 1\ncheck count(kind(box)) > 0\n";
        let (ast, d) = parse(src);
        assert!(d.errors.is_empty(), "{:?}", d.errors);
        match &ast.stmts[0].kind {
            StmtKind::Pattern {
                roles, constraints, ..
            } => {
                assert_eq!(roles[1], ("piers".to_string(), true));
                assert_eq!(constraints.len(), 3);
                assert!(matches!(
                    constraints[1].constraint,
                    Constraint::Relation { rel: RelWord::On, .. }
                ));
            }
            _ => panic!(),
        }
        assert!(
            matches!(&ast.stmts[2].kind, StmtKind::Check { source_text, message: None, .. } if source_text == "count(kind(box)) > 0")
        );
    }
}
