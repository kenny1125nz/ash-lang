use crate::lang::token::{Token, TokenKind};
use regex::Regex;

pub struct Lexer {
    src: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    start_of_line: bool,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            src: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            start_of_line: true,
        }
    }

    pub fn next(&mut self) -> Token {
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];

            if ch.is_whitespace() && ch != '\n' {
                self.advance();
                continue;
            }

            if self.start_of_line && ch == '#' && self.pos + 1 < self.src.len()
                && self.src[self.pos + 1] == '!'
            {
                return self.read_shebang();
            }

            self.start_of_line = false;

            if ch == '#' && (self.pos + 1 >= self.src.len() || self.src[self.pos + 1] != '!') {
                return self.read_comment();
            }

            if ch == '\n' {
                self.start_of_line = true;
                let tok = self.make_token(TokenKind::TkNewline, "\\n");
                self.advance();
                return tok;
            }

            if ch == '`' {
                return self.read_text_block();
            }

            if ch == '"' {
                return self.read_string();
            }

            if ch == '$' {
                return self.read_dollar();
            }

            if ch == '@' {
                let tok = self.make_token(TokenKind::TkAt, "@");
                self.advance();
                return tok;
            }

            if ch == '&' {
                let tok = self.make_token(TokenKind::TkAmpersand, "&");
                self.advance();
                return tok;
            }

            if ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']'
                || ch == ',' || ch == ':'
            {
                let (kind, lit) = match ch {
                    '(' => (TokenKind::TkLParen, "("),
                    ')' => (TokenKind::TkRParen, ")"),
                    '{' => (TokenKind::TkLBrace, "{"),
                    '}' => (TokenKind::TkRBrace, "}"),
                    '[' => (TokenKind::TkLBracket, "["),
                    ']' => (TokenKind::TkRBracket, "]"),
                    ',' => (TokenKind::TkComma, ","),
                    ':' => (TokenKind::TkColon, ":"),
                    _ => unreachable!(),
                };
                let tok = self.make_token(kind, lit);
                self.advance();
                return tok;
            }

            if ch == '=' || ch == '!' || ch == '>' || ch == '<' {
                return self.read_comparison();
            }

            if ch == '+' || ch == '-' || ch == '*' || ch == '/' {
                return self.read_operator();
            }

            if ch == '0' && self.pos + 1 < self.src.len() && self.src[self.pos + 1] == 'x' {
                return self.read_hex();
            }

            if ch.is_ascii_digit() {
                return self.read_number();
            }

            if ch.is_ascii_alphabetic() || ch == '_' {
                return self.read_ident();
            }

            let tok = self.make_token(TokenKind::TkIdent, &ch.to_string());
            self.advance();
            return tok;
        }

        self.make_token(TokenKind::TkEOF, "")
    }

    fn advance(&mut self) {
        if self.pos < self.src.len() {
            if self.src[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    fn make_token(&self, kind: TokenKind, literal: &str) -> Token {
        Token {
            kind,
            literal: literal.to_string(),
            line: self.line,
            col: self.col,
        }
    }

    fn read_shebang(&mut self) -> Token {
        let start_pos = self.pos;
        while self.pos < self.src.len() && self.src[self.pos] != '\n' {
            self.advance();
        }
        let line: String = self.src[start_pos..self.pos].iter().collect();
        Token {
            kind: TokenKind::TkShebang,
            literal: line,
            line: self.line,
            col: self.col,
        }
    }

    fn read_comment(&mut self) -> Token {
        while self.pos < self.src.len() && self.src[self.pos] != '\n' {
            self.advance();
        }
        self.next()
    }

    fn read_text_block(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();

        let is_triple = self.pos + 1 < self.src.len()
            && self.src[self.pos] == '`'
            && self.src[self.pos + 1] == '`';

        if !is_triple {
            return Token {
                kind: TokenKind::TkIdent,
                literal: "`".to_string(),
                line: start_line,
                col: start_col,
            };
        }

        self.advance();
        self.advance();

        let mut buf = String::new();
        while self.pos < self.src.len() {
            if self.src[self.pos] == '`' {
                if self.pos + 2 < self.src.len()
                    && self.src[self.pos + 1] == '`'
                    && self.src[self.pos + 2] == '`'
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }
            }
            buf.push(self.src[self.pos]);
            self.advance();
        }

        Token {
            kind: TokenKind::TkTextBlock,
            literal: buf,
            line: start_line,
            col: start_col,
        }
    }

    fn read_string(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();
        let mut buf = String::new();
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == '"' {
                self.advance();
                break;
            }
            if ch == '\\' {
                self.advance();
                if self.pos < self.src.len() {
                    let next = self.src[self.pos];
                    match next {
                        '"' => buf.push('"'),
                        '$' => buf.push('$'),
                        '\\' => buf.push('\\'),
                        'n' => buf.push('\n'),
                        't' => buf.push('\t'),
                        _ => {
                            buf.push('\\');
                            buf.push(next);
                        }
                    }
                    self.advance();
                }
                continue;
            }
            buf.push(ch);
            self.advance();
        }
        Token {
            kind: TokenKind::TkString,
            literal: buf,
            line: start_line,
            col: start_col,
        }
    }

    fn read_dollar(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();

        if self.pos < self.src.len() && self.src[self.pos] == '{' {
            self.advance();
            let mut name = String::new();
            while self.pos < self.src.len() && self.src[self.pos] != '}' {
                name.push(self.src[self.pos]);
                self.advance();
            }
            if self.pos < self.src.len() {
                self.advance();
            }
            return Token {
                kind: TokenKind::TkDollarLBrace,
                literal: name.trim().to_string(),
                line: start_line,
                col: start_col,
            };
        }

        if self.pos < self.src.len() && self.src[self.pos] == '(' {
            self.advance();
            let mut depth = 1;
            let mut cmd = String::new();
            while self.pos < self.src.len() && depth > 0 {
                let ch = self.src[self.pos];
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                }
                cmd.push(ch);
                self.advance();
            }
            return Token {
                kind: TokenKind::TkDollarLParen,
                literal: cmd.trim().to_string(),
                line: start_line,
                col: start_col,
            };
        }

        if self.pos < self.src.len() && self.src[self.pos] == '?' {
            let tok = Token {
                kind: TokenKind::TkDollarLBrace,
                literal: "?".to_string(),
                line: start_line,
                col: start_col,
            };
            self.advance();
            return tok;
        }

        let mut name = String::new();
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == '_')
        {
            name.push(self.src[self.pos]);
            self.advance();
        }
        if name.is_empty() {
            return Token {
                kind: TokenKind::TkDollar,
                literal: "$".to_string(),
                line: start_line,
                col: start_col,
            };
        }
        Token {
            kind: TokenKind::TkDollarLBrace,
            literal: name,
            line: start_line,
            col: start_col,
        }
    }

    fn read_comparison(&mut self) -> Token {
        let ch = self.src[self.pos];
        let start_line = self.line;
        let start_col = self.col;
        self.advance();

        let (kind, lit) = match ch {
            '=' => {
                if self.pos < self.src.len() && self.src[self.pos] == '=' {
                    self.advance();
                    (TokenKind::TkEq, "==")
                } else {
                    (TokenKind::TkAssign, "=")
                }
            }
            '!' => {
                if self.pos < self.src.len() && self.src[self.pos] == '=' {
                    self.advance();
                    (TokenKind::TkNeq, "!=")
                } else {
                    (TokenKind::TkIdent, "!")
                }
            }
            '>' => {
                if self.pos < self.src.len() && self.src[self.pos] == '=' {
                    self.advance();
                    (TokenKind::TkGte, ">=")
                } else {
                    (TokenKind::TkGt, ">")
                }
            }
            '<' => {
                if self.pos < self.src.len() && self.src[self.pos] == '=' {
                    self.advance();
                    (TokenKind::TkLte, "<=")
                } else {
                    (TokenKind::TkLt, "<")
                }
            }
            _ => (TokenKind::TkIdent, &ch.to_string() as &str),
        };
        Token {
            kind,
            literal: lit.to_string(),
            line: start_line,
            col: start_col,
        }
    }

    fn read_operator(&mut self) -> Token {
        let ch = self.src[self.pos];
        let start_line = self.line;
        let start_col = self.col;
        self.advance();
        let (kind, lit) = match ch {
            '+' => (TokenKind::TkPlus, "+"),
            '-' => (TokenKind::TkMinus, "-"),
            '*' => (TokenKind::TkStar, "*"),
            '/' => (TokenKind::TkSlash, "/"),
            _ => (TokenKind::TkIdent, &ch.to_string() as &str),
        };
        Token {
            kind,
            literal: lit.to_string(),
            line: start_line,
            col: start_col,
        }
    }

    fn read_hex(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        self.advance();
        self.advance();
        let mut buf = String::from("0x");
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_digit()
                || ('a'..='f').contains(&self.src[self.pos])
                || ('A'..='F').contains(&self.src[self.pos]))
        {
            buf.push(self.src[self.pos]);
            self.advance();
        }
        Token {
            kind: TokenKind::TkInt,
            literal: buf,
            line: start_line,
            col: start_col,
        }
    }

    fn read_number(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let mut buf = String::new();
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            buf.push(self.src[self.pos]);
            self.advance();
        }
        if self.pos < self.src.len() && self.src[self.pos] == '.' {
            buf.push('.');
            self.advance();
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                buf.push(self.src[self.pos]);
                self.advance();
            }
            return Token {
                kind: TokenKind::TkFloat,
                literal: buf,
                line: start_line,
                col: start_col,
            };
        }
        Token {
            kind: TokenKind::TkInt,
            literal: buf,
            line: start_line,
            col: start_col,
        }
    }

    fn read_ident(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let mut buf = String::new();
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == '_')
        {
            buf.push(self.src[self.pos]);
            self.advance();
        }
        Token {
            kind: TokenKind::TkIdent,
            literal: buf,
            line: start_line,
            col: start_col,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut prev_newline = false;

        loop {
            let tok = self.next();
            if tok.kind == TokenKind::TkEOF {
                break;
            }
            if tok.kind == TokenKind::TkNewline {
                if prev_newline {
                    continue;
                }
                prev_newline = true;
                tokens.push(tok);
                continue;
            }
            prev_newline = false;

            let mut tok = tok;
            if tok.kind == TokenKind::TkShebang
                && tok.literal.len() >= 10
                && &tok.literal[..10] == "#!compact "
            {
                tok.kind = TokenKind::TkCompactConfig;
            }

            tokens.push(tok);
        }

        tokens
    }
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    Ok(Lexer::new(src).tokenize())
}

pub struct ShebangDecl {
    pub engine: String,
    pub version: String,
    pub model: String,
}

pub struct CompactConfigResult {
    pub mode: String,
    pub window: String,
    pub strategy: String,
}

pub fn parse_shebang(line: &str) -> Result<ShebangDecl, String> {
    let re = Regex::new(r"^#!\s*(\w[\w.-]*)\s*:\s*(\S+?)(?:\s*:\s*(\S+?))?\s*$").unwrap();
    if let Some(caps) = re.captures(line) {
        Ok(ShebangDecl {
            engine: caps.get(1).unwrap().as_str().to_string(),
            version: caps.get(2).unwrap().as_str().to_string(),
            model: caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default(),
        })
    } else {
        Err(format!("invalid shebang: {:?}", line))
    }
}

pub fn parse_compact_line(line: &str) -> Option<CompactConfigResult> {
    let re = Regex::new(r"^#!compact\s+(.+)$").unwrap();
    let caps = re.captures(line)?;
    let body = caps.get(1).unwrap().as_str();
    let mut mode = String::new();
    let mut window = String::new();
    let mut strategy = String::new();
    for pair in body.split_whitespace() {
        if let Some(eq) = pair.find('=') {
            let k = pair[..eq].trim();
            let v = pair[eq + 1..].trim();
            match k {
                "mode" => mode = v.to_string(),
                "window" => window = v.to_string(),
                "strategy" => strategy = v.to_string(),
                _ => {}
            }
        }
    }
    Some(CompactConfigResult {
        mode,
        window,
        strategy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::token::TokenKind;

    fn lex_str(src: &str) -> Vec<Token> {
        tokenize(src).unwrap()
    }

    fn ident(s: &str) -> Token {
        Token {
            kind: TokenKind::TkIdent,
            literal: s.to_string(),
            line: 0,
            col: 0,
        }
    }

    fn assert_tokens(got: &[Token], expected: &[Token]) {
        assert_eq!(
            got.len(),
            expected.len(),
            "token count: expected {}, got {}\n  expected: {:?}\n  got:      {:?}",
            expected.len(),
            got.len(),
            expected,
            got
        );
        for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
            if g.kind != e.kind || g.literal != e.literal {
                panic!(
                    "token {}:\n  expected {}({:?})\n  got      {}({:?})",
                    i, e.kind, e.literal, g.kind, g.literal
                );
            }
        }
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex_str("hello world _foo bar123");
        let expected = vec![
            ident("hello"),
            ident("world"),
            ident("_foo"),
            ident("bar123"),
        ];
        assert_tokens(&tokens, &expected);
    }

    #[test]
    fn test_keywords() {
        let src = "if else for while fn return do with subagent using try fail evaluate accept partial upto wait compact include exec print exit env break continue not and or";
        let tokens = lex_str(src);
        let expected = [
            "if", "else", "for", "while", "fn", "return", "do", "with", "subagent", "using",
            "try", "fail", "evaluate", "accept", "partial", "upto", "wait", "compact", "include",
            "exec", "print", "exit", "env", "break", "continue", "not", "and", "or",
        ];
        let got: Vec<&Token> = tokens.iter().filter(|t| t.kind != TokenKind::TkNewline).collect();
        assert_eq!(
            got.len(),
            expected.len(),
            "expected {} tokens, got {}: {:?}",
            expected.len(),
            got.len(),
            got
        );
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(
                got[i].kind, TokenKind::TkIdent,
                "token {}: expected ident, got {:?}",
                i, got[i]
            );
            assert_eq!(
                got[i].literal, *exp,
                "token {}: expected literal {:?}, got {:?}",
                i, exp, got[i].literal
            );
        }
    }

    #[test]
    fn test_numbers() {
        let cases = [
            ("123", TokenKind::TkInt, "123"),
            ("3.14", TokenKind::TkFloat, "3.14"),
            ("0xDEAD", TokenKind::TkInt, "0xDEAD"),
            ("0xbeef", TokenKind::TkInt, "0xbeef"),
        ];
        for (src, kind, lit) in &cases {
            let tokens = lex_str(src);
            assert_eq!(
                tokens.len(),
                1,
                "{:?}: expected 1 token, got {}",
                src,
                tokens.len()
            );
            assert_eq!(
                tokens[0].kind, *kind,
                "{:?}: expected {:?}, got {:?}",
                src, kind, tokens[0].kind
            );
            assert_eq!(
                tokens[0].literal, *lit,
                "{:?}: expected literal {:?}, got {:?}",
                src, lit, tokens[0].literal
            );
        }
    }

    #[test]
    fn test_operators_and_delimiters() {
        let cases = [
            ("=", TokenKind::TkAssign, "="),
            ("==", TokenKind::TkEq, "=="),
            ("!=", TokenKind::TkNeq, "!="),
            (">", TokenKind::TkGt, ">"),
            ("<", TokenKind::TkLt, "<"),
            (">=", TokenKind::TkGte, ">="),
            ("<=", TokenKind::TkLte, "<="),
            ("+", TokenKind::TkPlus, "+"),
            ("-", TokenKind::TkMinus, "-"),
            ("*", TokenKind::TkStar, "*"),
            ("/", TokenKind::TkSlash, "/"),
            ("(", TokenKind::TkLParen, "("),
            (")", TokenKind::TkRParen, ")"),
            ("{", TokenKind::TkLBrace, "{"),
            ("}", TokenKind::TkRBrace, "}"),
            (",", TokenKind::TkComma, ","),
            ("&", TokenKind::TkAmpersand, "&"),
            ("@", TokenKind::TkAt, "@"),
            (":", TokenKind::TkColon, ":"),
        ];
        for (src, kind, lit) in &cases {
            let tokens = lex_str(src);
            assert_eq!(
                tokens.len(),
                1,
                "{:?}: expected 1 token, got {}",
                src,
                tokens.len()
            );
            assert_eq!(
                tokens[0].kind, *kind,
                "{:?}: expected {:?}, got {:?}",
                src, kind, tokens[0].kind
            );
            assert_eq!(
                tokens[0].literal, *lit,
                "{:?}: expected literal {:?}, got {:?}",
                src, lit, tokens[0].literal
            );
        }
    }

    #[test]
    fn test_string() {
        let tokens = lex_str(r#""hello world""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TkString);
        assert_eq!(tokens[0].literal, "hello world");
    }

    #[test]
    fn test_string_escapes() {
        let tokens = lex_str(r#""hello \"world\" \\ \$""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TkString);
        assert_eq!(tokens[0].literal, r#"hello "world" \ $"#);
    }

    #[test]
    fn test_text_block() {
        let src = "```\nhello\nworld\n```";
        let tokens = lex_str(src);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TkTextBlock);
    }

    #[test]
    fn test_var_ref_bare() {
        let tokens = lex_str("$NAME $? ${stderr}");
        assert_eq!(
            tokens.len(),
            3,
            "expected 3 tokens, got {}: {:?}",
            tokens.len(),
            tokens
        );
        assert_eq!(tokens[0].kind, TokenKind::TkDollarLBrace);
        assert_eq!(tokens[0].literal, "NAME");
        assert_eq!(tokens[1].kind, TokenKind::TkDollarLBrace);
        assert_eq!(tokens[1].literal, "?");
        assert_eq!(tokens[2].kind, TokenKind::TkDollarLBrace);
        assert_eq!(tokens[2].literal, "stderr");
    }

    #[test]
    fn test_command_subst() {
        let tokens = lex_str("$(echo hello)");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TkDollarLParen);
        assert_eq!(tokens[0].literal, "echo hello");
    }

    #[test]
    fn test_shebang() {
        let src = "#!opencode:1.2.0\nprint hello";
        let tokens = tokenize(src).unwrap();
        assert!(tokens.len() >= 3, "expected >=3 tokens, got {}", tokens.len());
        assert_eq!(tokens[0].kind, TokenKind::TkShebang);
        assert_eq!(tokens[0].literal, "#!opencode:1.2.0");
    }

    #[test]
    fn test_compact_config() {
        let src = "#!compact mode=on window=64000 strategy=truncate";
        let tokens = lex_str(src);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::TkCompactConfig);
    }

    #[test]
    fn test_comment() {
        let tokens = lex_str("hello # this is a comment\nworld");
        assert_eq!(
            tokens.len(),
            3,
            "expected 3 tokens, got {}: {:?}",
            tokens.len(),
            tokens
        );
        assert_eq!(tokens[0].literal, "hello");
        assert_eq!(tokens[2].literal, "world");
    }

    #[test]
    fn test_newlines() {
        let tokens = lex_str("a\nb\nc");
        assert_eq!(
            tokens.len(),
            5,
            "expected 5 tokens, got {}: {:?}",
            tokens.len(),
            tokens
        );
        assert_eq!(tokens[0].literal, "a");
        assert_eq!(tokens[1].kind, TokenKind::TkNewline);
        assert_eq!(tokens[2].literal, "b");
        assert_eq!(tokens[3].kind, TokenKind::TkNewline);
        assert_eq!(tokens[4].literal, "c");
    }

    #[test]
    fn test_complex_expr() {
        let tokens = lex_str("$? == 0 and not $FAILED");
        assert_eq!(
            tokens.len(),
            6,
            "expected 6 tokens, got {}: {:?}",
            tokens.len(),
            tokens
        );
        let expected = [
            TokenKind::TkDollarLBrace,
            TokenKind::TkEq,
            TokenKind::TkInt,
            TokenKind::TkIdent,
            TokenKind::TkIdent,
            TokenKind::TkDollarLBrace,
        ];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(
                tokens[i].kind, *exp,
                "token {}: expected {:?}, got {:?}({:?})",
                i, exp, tokens[i].kind, tokens[i].literal
            );
        }
    }

    #[test]
    fn test_parse_shebang() {
        let cases = [
            ("#!opencode:1.2.0", "opencode", "1.2.0", ""),
            ("#!opencode:1.2.0:sonnet", "opencode", "1.2.0", "sonnet"),
            (
                "#!claude-code:2.0.0:claude-sonnet-4",
                "claude-code",
                "2.0.0",
                "claude-sonnet-4",
            ),
            ("#!aider:v0.45.0", "aider", "v0.45.0", ""),
        ];
        for (line, engine, version, model) in &cases {
            let decl = parse_shebang(line).unwrap();
            assert_eq!(&decl.engine, engine, "{:?}: engine mismatch", line);
            assert_eq!(&decl.version, version, "{:?}: version mismatch", line);
            assert_eq!(&decl.model, model, "{:?}: model mismatch", line);
        }
    }

    #[test]
    fn test_parse_shebang_invalid() {
        let invalid = ["# not shebang", "#!", "#!foo", "//!opencode:1.0", ""];
        for line in &invalid {
            assert!(parse_shebang(line).is_err(), "{:?}: expected error", line);
        }
    }

    #[test]
    fn test_parse_compact_config() {
        let cases = [
            (
                "#!compact mode=on window=64000 strategy=truncate",
                "on",
                "64000",
                "truncate",
            ),
            ("#!compact mode=auto", "auto", "", ""),
        ];
        for (line, mode, window, strat) in &cases {
            let cc = parse_compact_line(line);
            assert!(cc.is_some(), "{:?}: expected non-None config", line);
            let cc = cc.unwrap();
            assert_eq!(&cc.mode, mode, "{:?}: mode mismatch", line);
            assert_eq!(&cc.window, window, "{:?}: window mismatch", line);
            assert_eq!(&cc.strategy, strat, "{:?}: strategy mismatch", line);
        }
    }

    #[test]
    fn test_parse_compact_config_empty() {
        let cc = parse_compact_line("#!compact");
        // May or may not return a config; just must not panic
        if let Some(_cc) = cc {
            // non-empty path: fine
        }
    }
}
