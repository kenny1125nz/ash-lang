use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    TkEOF,
    TkNewline,
    TkIdent,
    TkString,
    TkTextBlock,
    TkInt,
    TkFloat,
    TkAssign,
    TkEq,
    TkNeq,
    TkGt,
    TkLt,
    TkGte,
    TkLte,
    TkPlus,
    TkMinus,
    TkStar,
    TkSlash,
    TkLParen,
    TkRParen,
    TkLBrace,
    TkRBrace,
    TkComma,
    TkAmpersand,
    TkAt,
    TkDollar,
    TkDollarLParen,
    TkDollarLBrace,
    TkLBracket,
    TkRBracket,
    TkColon,
    TkHash,
    TkShebang,
    TkCompactConfig,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TkEOF => write!(f, "EOF"),
            Self::TkNewline => write!(f, "newline"),
            Self::TkIdent => write!(f, "identifier"),
            Self::TkString => write!(f, "string"),
            Self::TkTextBlock => write!(f, "text block"),
            Self::TkInt => write!(f, "integer"),
            Self::TkFloat => write!(f, "float"),
            Self::TkAssign => write!(f, "="),
            Self::TkEq => write!(f, "=="),
            Self::TkNeq => write!(f, "!="),
            Self::TkGt => write!(f, ">"),
            Self::TkLt => write!(f, "<"),
            Self::TkGte => write!(f, ">="),
            Self::TkLte => write!(f, "<="),
            Self::TkPlus => write!(f, "+"),
            Self::TkMinus => write!(f, "-"),
            Self::TkStar => write!(f, "*"),
            Self::TkSlash => write!(f, "/"),
            Self::TkLParen => write!(f, "("),
            Self::TkRParen => write!(f, ")"),
            Self::TkLBrace => write!(f, "{{"),
            Self::TkRBrace => write!(f, "}}"),
            Self::TkComma => write!(f, ","),
            Self::TkAmpersand => write!(f, "&"),
            Self::TkAt => write!(f, "@"),
            Self::TkDollar => write!(f, "$"),
            Self::TkDollarLParen => write!(f, "$("),
            Self::TkDollarLBrace => write!(f, "${{"),
            Self::TkLBracket => write!(f, "["),
            Self::TkRBracket => write!(f, "]"),
            Self::TkColon => write!(f, ":"),
            Self::TkHash => write!(f, "#"),
            Self::TkShebang => write!(f, "shebang"),
            Self::TkCompactConfig => write!(f, "compact config"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.literal.is_empty() {
            write!(f, "{}@{}:{}", self.kind, self.line, self.col)
        } else {
            write!(f, "{}({:?})@{}:{}", self.kind, self.literal, self.line, self.col)
        }
    }
}

pub fn keywords() -> HashMap<&'static str, TokenKind> {
    let mut m = HashMap::new();
    for kw in &[
        "true", "false", "if", "else", "for", "in", "while", "fn",
        "return", "break", "continue", "print", "exec", "exit", "env",
        "include", "within", "wait", "try", "upto", "fail", "evaluate",
        "with", "accept", "partial", "compact", "using", "do", "and", "or", "not",
        "session", "begin", "end",
    ] {
        m.insert(*kw, TokenKind::TkIdent);
    }
    m
}

pub fn is_keyword(s: &str) -> bool {
    keywords().contains_key(s)
}
