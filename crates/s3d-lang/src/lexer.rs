//! 词法分析（spec/01 1.2）。把一个文件切成记号；深度为 0 的换行与分号产生 `Nl`。
//! 词法错误不中止：产生 `Error` 记号，由语法分析按 1.1.4 第 1 条丢弃整条语句（连同块）。

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    Ident(String),
    /// 纯数字段（阵列实例，1.6.2）；保留原文以便输出
    Digits(String),
    Star,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Nl,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    /// 列表分隔符（只在 `[ ]` 内；深度 0 的 `;` 变成 Nl）
    Semi,
    Colon,
    Eq,
    /// 正则匹配运算符 `~`（元数据前缀另见 MetaKey）
    Tilde,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Pipe,
    Amp,
    Number(f64),
    Str(String),
    Ident(String),
    /// 多段路径（至少两段）
    Path(Vec<Seg>),
    /// 方向组合词（2–3 个方向词）
    DirWord(Vec<String>),
    /// 井号记号 `#name`
    Hash(String),
    /// 正则字面量（不含两侧的 `/`，`\/` 已还原为 `/`）
    Regex(String),
    /// 元数据前缀 `~k`
    MetaKey(String),
    /// `]` 后紧跟的分量后缀 `.x` / `.y` / `.z`
    Axis(char),
    /// 词法错误（文案）
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    /// 源码字节区间 [start, end)
    pub start: usize,
    pub end: usize,
    /// 与前一个记号之间没有任何空白或注释（"紧跟"）
    pub glued: bool,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Nl => write!(f, "换行"),
            Tok::LBrace => write!(f, "{{"),
            Tok::RBrace => write!(f, "}}"),
            Tok::LParen => write!(f, "("),
            Tok::RParen => write!(f, ")"),
            Tok::LBracket => write!(f, "["),
            Tok::RBracket => write!(f, "]"),
            Tok::Comma => write!(f, ","),
            Tok::Semi => write!(f, ";"),
            Tok::Colon => write!(f, ":"),
            Tok::Eq => write!(f, "="),
            Tok::Tilde => write!(f, "~"),
            Tok::Plus => write!(f, "+"),
            Tok::Minus => write!(f, "-"),
            Tok::Star => write!(f, "*"),
            Tok::Slash => write!(f, "/"),
            Tok::EqEq => write!(f, "=="),
            Tok::Ne => write!(f, "!="),
            Tok::Lt => write!(f, "<"),
            Tok::Gt => write!(f, ">"),
            Tok::Le => write!(f, "<="),
            Tok::Ge => write!(f, ">="),
            Tok::Pipe => write!(f, "|"),
            Tok::Amp => write!(f, "&"),
            Tok::Number(n) => write!(f, "{}", n),
            Tok::Str(s) => write!(f, "\"{}\"", s),
            Tok::Ident(s) => write!(f, "{}", s),
            Tok::Path(segs) => {
                let parts: Vec<String> = segs
                    .iter()
                    .map(|s| match s {
                        Seg::Ident(i) => i.clone(),
                        Seg::Digits(d) => d.clone(),
                        Seg::Star => "*".to_string(),
                    })
                    .collect();
                write!(f, "{}", parts.join("."))
            }
            Tok::DirWord(w) => write!(f, "{}", w.join("-")),
            Tok::Hash(h) => write!(f, "#{}", h),
            Tok::Regex(r) => write!(f, "/{}/", r),
            Tok::MetaKey(k) => write!(f, "~{}", k),
            Tok::Axis(a) => write!(f, ".{}", a),
            Tok::Error(m) => write!(f, "<错误：{}>", m),
        }
    }
}

pub const DIR_WORDS: [&str; 13] = [
    "top", "up", "bottom", "down", "left", "west", "right", "east", "front", "south", "back", "north",
    "center",
];

/// 方向词所属的轴：0 = x，1 = y，2 = z；center 为 3。
pub fn dir_axis(w: &str) -> Option<u8> {
    match w {
        "left" | "west" | "right" | "east" => Some(0),
        "top" | "up" | "bottom" | "down" => Some(1),
        "front" | "south" | "back" | "north" => Some(2),
        "center" => Some(3),
        _ => None,
    }
}

pub fn is_dir_word(w: &str) -> bool {
    DIR_WORDS.contains(&w)
}

pub fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

pub fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphabetic() || c.is_ascii_digit() || is_unicode_nd(c)
}

/// Unicode 十进制数字（Nd）：常见区段（全角数字、阿拉伯-印度数字、天城文数字等），每段从 0 起连续 10 个码点。
fn is_unicode_nd(c: char) -> bool {
    const ZEROS: [u32; 11] = [
        0x0660, 0x06F0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0E50, 0xFF10,
    ];
    let u = c as u32;
    ZEROS.iter().any(|z| u >= *z && u < z + 10)
}

pub struct Lexer<'a> {
    src: &'a str,
    chars: Vec<(usize, char)>,
    i: usize,
    line: u32,
    depth_stack: Vec<char>,
    tokens: Vec<Token>,
    /// 上一个记号的结束位置（用于 glued 判定）
    last_end: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        let src = src.strip_prefix('\u{FEFF}').unwrap_or(src);
        Lexer {
            src,
            chars: src.char_indices().collect(),
            i: 0,
            line: 1,
            depth_stack: Vec::new(),
            tokens: Vec::new(),
            last_end: usize::MAX,
        }
    }

    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.i + k).map(|(_, c)| *c)
    }
    fn pos(&self) -> usize {
        self.chars.get(self.i).map(|(p, _)| *p).unwrap_or(self.src.len())
    }
    fn push(&mut self, tok: Tok, start: usize, end: usize) {
        let glued = start == self.last_end;
        self.tokens.push(Token {
            tok,
            line: self.line,
            start,
            end,
            glued,
        });
        self.last_end = end;
    }
    fn err(&mut self, msg: impl Into<String>, start: usize, end: usize) {
        self.push(Tok::Error(msg.into()), start, end);
    }
    fn depth(&self) -> usize {
        self.depth_stack.len()
    }

    pub fn run(mut self) -> Vec<Token> {
        while let Some(c) = self.peek(0) {
            let start = self.pos();
            match c {
                ' ' | '\t' | '\r' => {
                    self.i += 1;
                }
                '\n' => {
                    self.i += 1;
                    if self.depth() == 0 {
                        self.push(Tok::Nl, start, start + 1);
                    }
                    self.line += 1;
                    // 换行后 glued 一定为假
                    self.last_end = usize::MAX;
                }
                '/' if self.peek(1) == Some('/') => {
                    // 注释到行尾
                    while let Some(ch) = self.peek(0) {
                        if ch == '\n' {
                            break;
                        }
                        self.i += 1;
                    }
                    self.last_end = usize::MAX;
                }
                '/' if self.after_tilde_operator() => self.lex_regex(),
                '"' => self.lex_string(),
                '#' => self.lex_hash(),
                '~' => {
                    self.i += 1;
                    if self.peek(0).is_some_and(is_ident_char) {
                        let kstart = self.pos();
                        while self.peek(0).is_some_and(is_ident_char) {
                            self.i += 1;
                        }
                        let key = self.src[kstart..self.pos()].to_string();
                        let end = self.pos();
                        self.push(Tok::MetaKey(key), start, end);
                    } else {
                        self.push(Tok::Tilde, start, start + 1);
                    }
                }
                '(' | '[' => {
                    self.i += 1;
                    self.depth_stack.push(c);
                    self.push(
                        if c == '(' { Tok::LParen } else { Tok::LBracket },
                        start,
                        start + 1,
                    );
                }
                ')' | ']' => {
                    self.i += 1;
                    let want = if c == ')' { '(' } else { '[' };
                    match self.depth_stack.last() {
                        Some(&open) if open == want => {
                            self.depth_stack.pop();
                            self.push(
                                if c == ')' { Tok::RParen } else { Tok::RBracket },
                                start,
                                start + 1,
                            );
                            if c == ']' {
                                self.lex_axis_suffix();
                            }
                        }
                        Some(_) => self.err("括号不配对", start, start + 1),
                        None => self.err("多余的右括号", start, start + 1),
                    }
                }
                '{' | '}' => {
                    self.i += 1;
                    if self.depth() > 0 {
                        self.err("花括号不能写在括号内", start, start + 1);
                    } else {
                        self.push(if c == '{' { Tok::LBrace } else { Tok::RBrace }, start, start + 1);
                    }
                }
                ';' => {
                    self.i += 1;
                    match self.depth_stack.last() {
                        None => self.push(Tok::Nl, start, start + 1),
                        Some('[') => self.push(Tok::Semi, start, start + 1),
                        Some(_) => self.err("分号不能写在圆括号内", start, start + 1),
                    }
                }
                ',' => {
                    self.i += 1;
                    self.push(Tok::Comma, start, start + 1);
                }
                ':' => {
                    self.i += 1;
                    self.push(Tok::Colon, start, start + 1);
                }
                '=' => {
                    self.i += 1;
                    if self.peek(0) == Some('=') {
                        self.i += 1;
                        self.push(Tok::EqEq, start, start + 2);
                    } else {
                        self.push(Tok::Eq, start, start + 1);
                    }
                }
                '!' => {
                    self.i += 1;
                    if self.peek(0) == Some('=') {
                        self.i += 1;
                        self.push(Tok::Ne, start, start + 2);
                    } else {
                        self.err("无法识别的字符 “!”", start, start + 1);
                    }
                }
                '<' | '>' => {
                    self.i += 1;
                    if self.peek(0) == Some('=') {
                        self.i += 1;
                        self.push(if c == '<' { Tok::Le } else { Tok::Ge }, start, start + 2);
                    } else {
                        self.push(if c == '<' { Tok::Lt } else { Tok::Gt }, start, start + 1);
                    }
                }
                '+' => {
                    self.i += 1;
                    self.push(Tok::Plus, start, start + 1);
                }
                '-' => {
                    self.i += 1;
                    self.push(Tok::Minus, start, start + 1);
                }
                '*' => {
                    self.i += 1;
                    self.push(Tok::Star, start, start + 1);
                }
                '/' => {
                    self.i += 1;
                    self.push(Tok::Slash, start, start + 1);
                }
                '|' => {
                    self.i += 1;
                    self.push(Tok::Pipe, start, start + 1);
                }
                '&' => {
                    self.i += 1;
                    self.push(Tok::Amp, start, start + 1);
                }
                '.' if self.peek(1).is_some_and(|d| d.is_ascii_digit()) => self.lex_number(),
                d if d.is_ascii_digit() => self.lex_number(),
                c if is_ident_start(c) => self.lex_word(),
                other => {
                    self.i += 1;
                    let end = self.pos();
                    self.err(format!("无法识别的字符 “{}”", other), start, end);
                }
            }
        }
        // 文件末尾视为有一个 NL
        if !matches!(self.tokens.last().map(|t| &t.tok), Some(Tok::Nl) | None) {
            let p = self.src.len();
            self.push(Tok::Nl, p, p);
        }
        if let Some(&open) = self.depth_stack.first() {
            let p = self.src.len();
            self.err(format!("括号没有关闭：{}", open), p, p);
            self.push(Tok::Nl, p, p);
        }
        self.tokens
    }

    /// 上一个记号是正则匹配运算符 `~`，且 `/` 紧随（允许空白）
    fn after_tilde_operator(&self) -> bool {
        matches!(self.tokens.last().map(|t| &t.tok), Some(Tok::Tilde))
    }

    fn lex_regex(&mut self) {
        let start = self.pos();
        self.i += 1; // 开头的 /
        let mut out = String::new();
        loop {
            match self.peek(0) {
                None | Some('\n') => {
                    let end = self.pos();
                    self.err("正则字面量缺少结束的 /", start, end);
                    return;
                }
                Some('\\') if self.peek(1) == Some('/') => {
                    out.push('/');
                    self.i += 2;
                }
                Some('\\') => {
                    out.push('\\');
                    if let Some(n) = self.peek(1) {
                        if n != '\n' {
                            out.push(n);
                            self.i += 2;
                            continue;
                        }
                    }
                    self.i += 1;
                }
                Some('/') => {
                    self.i += 1;
                    break;
                }
                Some(ch) => {
                    out.push(ch);
                    self.i += 1;
                }
            }
        }
        let end = self.pos();
        self.push(Tok::Regex(out), start, end);
    }

    fn lex_string(&mut self) {
        let start = self.pos();
        self.i += 1;
        let mut out = String::new();
        loop {
            match self.peek(0) {
                None | Some('\n') => {
                    let end = self.pos();
                    self.err("字符串缺少结束引号", start, end);
                    return;
                }
                Some('\\') => match self.peek(1) {
                    Some('"') => {
                        out.push('"');
                        self.i += 2;
                    }
                    Some('\\') => {
                        out.push('\\');
                        self.i += 2;
                    }
                    Some('n') => {
                        out.push('\n');
                        self.i += 2;
                    }
                    Some(x) => {
                        let s = self.pos();
                        self.i += 2;
                        let e = self.pos();
                        self.err(format!("未知的转义 \\{}", x), s, e);
                        // 继续到结束引号，避免后续记号被当成代码
                        while let Some(ch) = self.peek(0) {
                            if ch == '"' || ch == '\n' {
                                break;
                            }
                            self.i += 1;
                        }
                        if self.peek(0) == Some('"') {
                            self.i += 1;
                        }
                        return;
                    }
                    None => {
                        let end = self.pos();
                        self.err("字符串缺少结束引号", start, end);
                        return;
                    }
                },
                Some('"') => {
                    self.i += 1;
                    break;
                }
                Some(ch) => {
                    out.push(ch);
                    self.i += 1;
                }
            }
        }
        let end = self.pos();
        self.push(Tok::Str(out), start, end);
    }

    fn lex_hash(&mut self) {
        let start = self.pos();
        self.i += 1;
        let nstart = self.pos();
        while self.peek(0).is_some_and(is_ident_char) {
            self.i += 1;
        }
        let end = self.pos();
        if nstart == end {
            self.err("井号后面要紧跟名字（#标签 / #颜色）", start, end);
            return;
        }
        let name = self.src[nstart..end].to_string();
        self.push(Tok::Hash(name), start, end);
    }

    fn lex_number(&mut self) {
        let start = self.pos();
        while self.peek(0).is_some_and(|c| c.is_ascii_digit()) {
            self.i += 1;
        }
        if self.peek(0) == Some('.') {
            if self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
                self.i += 1;
                while self.peek(0).is_some_and(|c| c.is_ascii_digit()) {
                    self.i += 1;
                }
            } else {
                self.i += 1;
                let end = self.pos();
                self.err("数字写法不合法（小数点后要有数字）", start, end);
                return;
            }
        }
        if matches!(self.peek(0), Some('e') | Some('E')) {
            let save = self.i;
            self.i += 1;
            if matches!(self.peek(0), Some('+') | Some('-')) {
                self.i += 1;
            }
            if self.peek(0).is_some_and(|c| c.is_ascii_digit()) {
                while self.peek(0).is_some_and(|c| c.is_ascii_digit()) {
                    self.i += 1;
                }
            } else {
                self.i = save;
                // `1e` / `1ex`：数字后紧跟标识符字符
            }
        }
        let end = self.pos();
        if self.peek(0).is_some_and(is_ident_char) {
            while self.peek(0).is_some_and(is_ident_char) {
                self.i += 1;
            }
            let e2 = self.pos();
            self.err("数字后面不能紧跟字母", start, e2);
            return;
        }
        let text = &self.src[start..end];
        match text.parse::<f64>() {
            Ok(v) if v.is_finite() => self.push(Tok::Number(v), start, end),
            _ => self.err("数字不合法", start, end),
        }
    }

    fn lex_word(&mut self) {
        let start = self.pos();
        while self.peek(0).is_some_and(is_ident_char) {
            self.i += 1;
        }
        let first = self.src[start..self.pos()].to_string();
        // 方向组合词：方向词 "-" 方向词 [ "-" 方向词 ]，全部紧跟
        if is_dir_word(&first) && self.peek(0) == Some('-') && self.peek(1).is_some_and(is_ident_start) {
            let save = self.i;
            let mut words = vec![first.clone()];
            while self.peek(0) == Some('-') && self.peek(1).is_some_and(is_ident_start) && words.len() < 4 {
                let s = self.i;
                self.i += 1;
                let ws = self.pos();
                while self.peek(0).is_some_and(is_ident_char) {
                    self.i += 1;
                }
                let w = self.src[ws..self.pos()].to_string();
                if is_dir_word(&w) {
                    words.push(w);
                } else {
                    self.i = s;
                    break;
                }
            }
            if words.len() >= 2 {
                // 校验：不同轴、不含 center、不超过 3 个
                let mut axes = Vec::new();
                let mut ok = words.len() <= 3;
                for w in &words {
                    let a = dir_axis(w).unwrap();
                    if a == 3 || axes.contains(&a) {
                        ok = false;
                    }
                    axes.push(a);
                }
                let end = self.pos();
                if ok {
                    self.push(Tok::DirWord(words), start, end);
                } else {
                    self.err("方向组合词矛盾", start, end);
                }
                return;
            }
            self.i = save;
        }
        // 路径：ident { "." segment }
        if self.peek(0) == Some('.') {
            let mut segs = vec![Seg::Ident(first)];
            loop {
                if self.peek(0) != Some('.') {
                    break;
                }
                let dot = self.i;
                self.i += 1;
                match self.peek(0) {
                    Some('*') => {
                        self.i += 1;
                        segs.push(Seg::Star);
                    }
                    Some(c) if c.is_ascii_digit() => {
                        let ds = self.pos();
                        while self.peek(0).is_some_and(|c| c.is_ascii_digit()) {
                            self.i += 1;
                        }
                        if self.peek(0).is_some_and(is_ident_char) {
                            while self.peek(0).is_some_and(is_ident_char) {
                                self.i += 1;
                            }
                            let e = self.pos();
                            self.err("路径段不合法", start, e);
                            return;
                        }
                        segs.push(Seg::Digits(self.src[ds..self.pos()].to_string()));
                    }
                    Some(c) if is_ident_start(c) => {
                        let ws = self.pos();
                        while self.peek(0).is_some_and(is_ident_char) {
                            self.i += 1;
                        }
                        let w = self.src[ws..self.pos()].to_string();
                        // 段也可以是方向组合词（B.top-left）
                        if is_dir_word(&w)
                            && self.peek(0) == Some('-')
                            && self.peek(1).is_some_and(is_ident_start)
                        {
                            let mut words = vec![w.clone()];
                            while self.peek(0) == Some('-') && self.peek(1).is_some_and(is_ident_start) {
                                let s = self.i;
                                self.i += 1;
                                let ws2 = self.pos();
                                while self.peek(0).is_some_and(is_ident_char) {
                                    self.i += 1;
                                }
                                let w2 = self.src[ws2..self.pos()].to_string();
                                if is_dir_word(&w2) {
                                    words.push(w2);
                                } else {
                                    self.i = s;
                                    break;
                                }
                            }
                            if words.len() >= 2 {
                                let mut axes = Vec::new();
                                let mut ok = words.len() <= 3;
                                for x in &words {
                                    let a = dir_axis(x).unwrap();
                                    if a == 3 || axes.contains(&a) {
                                        ok = false;
                                    }
                                    axes.push(a);
                                }
                                if !ok {
                                    let e = self.pos();
                                    self.err("方向组合词矛盾", start, e);
                                    return;
                                }
                                segs.push(Seg::Ident(words.join("-")));
                                continue;
                            }
                        }
                        segs.push(Seg::Ident(w));
                    }
                    _ => {
                        let _ = dot;
                        let e = self.pos();
                        self.err("路径不完整（“.” 后面要有名字）", start, e);
                        return;
                    }
                }
            }
            let end = self.pos();
            self.push(Tok::Path(segs), start, end);
            return;
        }
        let end = self.pos();
        self.push(Tok::Ident(first), start, end);
    }

    /// `]` 之后紧跟 `.x` / `.y` / `.z`：分量后缀记号；跟其他 `.…` 是语法错误
    fn lex_axis_suffix(&mut self) {
        if self.peek(0) != Some('.') {
            return;
        }
        let start = self.pos();
        match self.peek(1) {
            Some(a @ ('x' | 'y' | 'z')) if !self.peek(2).is_some_and(is_ident_char) => {
                self.i += 2;
                let end = self.pos();
                self.push(Tok::Axis(a), start, end);
            }
            _ => {
                self.i += 1;
                while self.peek(0).is_some_and(is_ident_char) {
                    self.i += 1;
                }
                let end = self.pos();
                self.err("坐标后缀后面只能跟 .x / .y / .z", start, end);
            }
        }
    }
}

pub fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        lex(src).into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn basics() {
        let t = toks("box floor size=40,1,24 pin=top // c\n");
        assert_eq!(
            t,
            vec![
                Tok::Ident("box".into()),
                Tok::Ident("floor".into()),
                Tok::Ident("size".into()),
                Tok::Eq,
                Tok::Number(40.0),
                Tok::Comma,
                Tok::Number(1.0),
                Tok::Comma,
                Tok::Number(24.0),
                Tok::Ident("pin".into()),
                Tok::Eq,
                Tok::Ident("top".into()),
                Tok::Nl,
            ]
        );
    }

    #[test]
    fn paths_dirwords_hash_meta() {
        let t = toks("cyl p #post on=a.top-left[0.5,1,0] ~src=\"n1\" r=0.18");
        assert!(matches!(&t[2], Tok::Hash(h) if h == "post"));
        assert!(matches!(&t[5], Tok::Path(s) if s.len() == 2 && s[1] == Seg::Ident("top-left".into())));
        assert_eq!(t[6], Tok::LBracket);
        assert!(matches!(&t[13], Tok::MetaKey(k) if k == "src"));
        // 元数据前缀与正则运算符
        let t2 = toks("x where name ~ /^a\\/b/ and y ~/c/");
        assert!(matches!(&t2[3], Tok::Tilde));
        assert!(matches!(&t2[4], Tok::Regex(r) if r == "^a/b"));
        assert!(matches!(&t2[8], Tok::Regex(r) if r == "c"));
    }

    #[test]
    fn newline_depth_and_semicolons() {
        let t = toks("plate p points=[(1,2);\n (3,4)] h=1; box b");
        let rb = t.iter().position(|x| *x == Tok::RBracket).unwrap();
        assert!(!t[..rb].contains(&Tok::Nl)); // 括号内的换行是空白
        assert!(t.contains(&Tok::Semi));
        let nl_count = t.iter().filter(|x| **x == Tok::Nl).count();
        assert_eq!(nl_count, 2);
    }

    #[test]
    fn errors() {
        assert!(matches!(toks("x=1.")[2], Tok::Error(_)));
        assert!(matches!(toks("x='a'")[2], Tok::Error(_)));
        assert!(matches!(toks("a=left-right")[2], Tok::Error(_)));
        assert!(matches!(toks("a=top-foo")[2], Tok::Ident(ref s) if s == "top"));
        assert!(matches!(toks("a=B[0.5].w")[6], Tok::Error(_)));
        assert!(matches!(toks("a=B[0.5].y")[6], Tok::Axis('y')));
        assert!(matches!(toks("a=\"abc")[2], Tok::Error(_)));
    }

    #[test]
    fn glued_flags() {
        let t = lex("sin(30) sin (30) B [1]");
        assert!(t[1].glued); // sin( 紧跟
        assert!(!t[5].glued); // sin (
        assert!(!t[9].glued); // B [
    }
}
