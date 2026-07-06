use crate::ast::*;
use crate::lexer::{self, parse_compact_line, parse_shebang};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: &str) -> Self {
        Self { tokens, pos: 0, source: source.to_string() }
    }

    fn current(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token {
                kind: TokenKind::TkEOF,
                literal: String::new(),
                line: 0,
                col: 0,
            }
        }
    }

    fn peek(&self) -> Token {
        if self.pos + 1 < self.tokens.len() {
            self.tokens[self.pos + 1].clone()
        } else {
            Token {
                kind: TokenKind::TkEOF,
                literal: String::new(),
                line: 0,
                col: 0,
            }
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn skip_newlines(&mut self) {
        while self.pos < self.tokens.len() && self.current().kind == TokenKind::TkNewline {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let tok = self.current().clone();
        if tok.kind != kind {
            return Err(format!(
                "expected {} at {}:{}, got {}({:?})",
                kind, tok.line, tok.col, tok.kind, tok.literal
            ));
        }
        self.pos += 1;
        Ok(tok)
    }

    pub fn parse(&mut self) -> Result<Script, String> {
        let mut script = Script {
            shebang: None,
            compact: None,
            body: Vec::new(),
        };

        self.skip_newlines();

        if self.current().kind == TokenKind::TkShebang {
            let sh = self.parse_shebang_decl()?;
            script.shebang = Some(sh);
            self.skip_newlines();
        }

        if self.current().kind == TokenKind::TkCompactConfig {
            let tok = self.advance();
            if let Some(parsed) = parse_compact_line(&tok.literal) {
                script.compact = Some(CompactConfig {
                    pos: Pos {
                        line: tok.line,
                        col: tok.col,
                    },
                    mode: parsed.mode,
                    window: parsed.window,
                    strategy: parsed.strategy,
                });
            }
            self.skip_newlines();
        }

        while self.current().kind != TokenKind::TkEOF {
            let mut stmt = self.parse_statement()?;
            if self.current().kind == TokenKind::TkAmpersand {
                self.advance();
                stmt = Node::Background(Background {
                    pos: stmt.pos().clone(),
                    stmt: Box::new(stmt),
                });
            }
            script.body.push(stmt);
            self.skip_newlines();

            if self.current().kind == TokenKind::TkNewline {
                self.pos += 1;
                self.skip_newlines();
            }
        }

        Ok(script)
    }

    fn parse_shebang_decl(&mut self) -> Result<ShebangDecl, String> {
        let tok = self.advance();
        let sh = parse_shebang(&tok.literal).map_err(|e| {
            format!("at {}:{}: {}", tok.line, tok.col, e)
        })?;
        Ok(ShebangDecl {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            engine: sh.engine,
            version: sh.version,
            model: sh.model,
        })
    }

    fn parse_statement(&mut self) -> Result<Node, String> {
        let tok = self.current().clone();

        match tok.kind {
            TokenKind::TkIdent => match tok.literal.as_str() {
                "if" => self.parse_if(),
                "for" => self.parse_for(),
                "while" => self.parse_while(),
                "fn" => self.parse_fn_decl(),
                "do" => self.parse_do(),
                "try" => self.parse_try(),
                "within" => self.parse_within(),
                "wait" => self.parse_wait(),
                "exec" => self.parse_exec(),
                "print" => self.parse_print(),
                "exit" => self.parse_exit(),
                "env" => self.parse_env(),
                "include" => self.parse_include(),
                "compact" => self.parse_compact_stmt(),
                "session" => self.parse_session(),
                "return" => self.parse_return(),
                "break" => self.parse_break(),
                "continue" => self.parse_continue(),
                _ => {
                    if self.peek().kind == TokenKind::TkAssign {
                        self.parse_var_assign()
                    } else if self.peek().kind == TokenKind::TkLParen {
                        self.parse_fn_call()
                    } else {
                        self.parse_expr()
                    }
                }
            },
            TokenKind::TkDollarLBrace
            | TokenKind::TkDollarLParen
            | TokenKind::TkDollar
            | TokenKind::TkString
            | TokenKind::TkTextBlock
            | TokenKind::TkInt
            | TokenKind::TkFloat
            | TokenKind::TkLParen => self.parse_expr(),
            TokenKind::TkLBrace => self.parse_block().map(|b| Node::Block(b)),
            _ => self.parse_expr(),
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        let open_tok = self.expect(TokenKind::TkLBrace)?;
        let mut block = Block {
            pos: Pos {
                line: open_tok.line,
                col: open_tok.col,
            },
            statements: Vec::new(),
        };
        self.skip_newlines();

        while self.current().kind != TokenKind::TkRBrace
            && self.current().kind != TokenKind::TkEOF
        {
            let stmt = self.parse_statement()?;
            block.statements.push(stmt);
            self.skip_newlines();
        }

        self.expect(TokenKind::TkRBrace)?;
        Ok(block)
    }

    fn parse_var_assign(&mut self) -> Result<Node, String> {
        let name = self.advance();
        self.expect(TokenKind::TkAssign)?;
        let val = self.parse_expr()?;
        Ok(Node::VarAssign(VarAssign {
            pos: Pos {
                line: name.line,
                col: name.col,
            },
            name: name.literal,
            value: Box::new(val),
        }))
    }

    fn parse_fn_call(&mut self) -> Result<Node, String> {
        let name_tok = self.advance();
        self.parse_fn_call_args(name_tok)
    }

    fn parse_fn_call_args(&mut self, name_tok: Token) -> Result<Node, String> {
        self.expect(TokenKind::TkLParen)?;
        self.skip_newlines();

        let mut args = Vec::new();
        if self.current().kind != TokenKind::TkRParen {
            let arg = self.parse_binary_expr(0)?;
            args.push(arg);
            while self.current().kind == TokenKind::TkComma {
                self.advance();
                self.skip_newlines();
                let arg = self.parse_binary_expr(0)?;
                args.push(arg);
            }
        }

        self.expect(TokenKind::TkRParen)?;

        Ok(Node::FnCall(FnCall {
            pos: Pos {
                line: name_tok.line,
                col: name_tok.col,
            },
            name: name_tok.literal,
            args,
        }))
    }

    fn parse_if(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let cond = self.parse_binary_expr(0)?;
        self.skip_newlines();
        let body = self.parse_block()?;

        let mut stmt = IfStmt {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            cond: Box::new(cond),
            body: Box::new(Node::Block(body)),
            else_ifs: Vec::new(),
            else_body: None,
        };

        loop {
            if self.current().kind == TokenKind::TkIdent
                && self.current().literal == "else"
            {
                self.advance();
                if self.current().kind == TokenKind::TkIdent
                    && self.current().literal == "if"
                {
                    self.advance();
                    self.skip_newlines();
                    let ei_cond = self.parse_binary_expr(0)?;
                    self.skip_newlines();
                    let ei_body = self.parse_block()?;
                    stmt.else_ifs.push(ElseIf {
                        pos: Pos {
                            line: tok.line,
                            col: tok.col,
                        },
                        cond: Box::new(ei_cond),
                        body: Box::new(Node::Block(ei_body)),
                    });
                } else {
                    self.skip_newlines();
                    let else_body = self.parse_block()?;
                    stmt.else_body = Some(Box::new(Node::Block(else_body)));
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Node::IfStmt(stmt))
    }

    fn parse_for(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let var_tok = self.expect(TokenKind::TkIdent)?;

        let in_tok = self.expect(TokenKind::TkIdent)?;
        if in_tok.literal != "in" {
            return Err(format!(
                "expected 'in' at {}:{}",
                var_tok.line, var_tok.col
            ));
        }

        let list = self.parse_binary_expr(0)?;
        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Node::ForStmt(ForStmt {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            var: var_tok.literal,
            list: Box::new(list),
            body: Box::new(Node::Block(body)),
        }))
    }

    fn parse_while(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let cond = self.parse_binary_expr(0)?;
        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Node::WhileStmt(WhileStmt {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            cond: Box::new(cond),
            body: Box::new(Node::Block(body)),
        }))
    }

    fn parse_fn_decl(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let name_tok = self.expect(TokenKind::TkIdent)?;
        self.expect(TokenKind::TkLParen)?;

        let mut params = Vec::new();
        if self.current().kind == TokenKind::TkIdent {
            params.push(self.advance().literal);
            while self.current().kind == TokenKind::TkComma {
                self.advance();
                let param_tok = self.expect(TokenKind::TkIdent)?;
                params.push(param_tok.literal);
            }
        }

        self.expect(TokenKind::TkRParen)?;
        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Node::FnDecl(FnDecl {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            name: name_tok.literal,
            params,
            body: Box::new(Node::Block(body)),
        }))
    }

    fn parse_do(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let prompt = self.parse_expr()?;

        let mut agent = AgentCall {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            prompt: Box::new(prompt),
            agent: None,
            subagent: String::new(),
            model: None,
            dir: None,
            compact: None,
        };

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "with"
        {
            self.advance();
            let ident = self.expect(TokenKind::TkIdent)?;

            if ident.literal == "subagent" {
                // with subagent <name>  — backward compat, use shebang engine as agent
                agent.subagent = self.parse_hyphenated_ident()?;
            } else {
                // with <agent> [subagent <name>]
                let mut agent_name = ident.literal;
                while self.current().kind == TokenKind::TkMinus
                    && self.peek().kind == TokenKind::TkIdent
                {
                    agent_name.push('-');
                    self.advance();
                    let part = self.expect(TokenKind::TkIdent)?;
                    agent_name.push_str(&part.literal);
                }
                agent.agent = Some(agent_name);
                if self.current().kind == TokenKind::TkIdent
                    && self.current().literal == "subagent"
                {
                    self.advance();
                    agent.subagent = self.parse_hyphenated_ident()?;
                }
            }
        }

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "using"
        {
            self.advance();
            let model = self.parse_primary()?;
            agent.model = Some(Box::new(model));
        }

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "in"
        {
            self.advance();
            let dir = self.parse_expr()?;
            agent.dir = Some(Box::new(dir));
        }

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "compact"
        {
            self.advance();
            let compact = self.parse_primary()?;
            agent.compact = Some(Box::new(compact));
        }

        Ok(Node::AgentCall(agent))
    }

    fn parse_try(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let body = self.parse_block()?;
        self.skip_newlines();

        let mut eval_mode = false;
        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "evaluate"
        {
            self.advance();
            let with_tok = self.expect(TokenKind::TkIdent)?;
            if with_tok.literal != "with" {
                return Err(format!(
                    "expected 'with' at {}:{}",
                    tok.line, tok.col
                ));
            }
            self.skip_newlines();
            eval_mode = true;
        }

        if eval_mode {
            let eval_body = self.parse_block()?;
            self.skip_newlines();

            let mut accept_block = None;
            let mut partial_block = None;
            let mut fail_block = None;

            if self.current().kind == TokenKind::TkIdent
                && self.current().literal == "accept"
            {
                self.advance();
                self.skip_newlines();
                accept_block = Some(self.parse_block()?);
                self.skip_newlines();
            }

            if self.current().kind == TokenKind::TkIdent
                && self.current().literal == "partial"
            {
                self.advance();
                self.skip_newlines();
                partial_block = Some(self.parse_block()?);
                self.skip_newlines();
            }

            if self.current().kind == TokenKind::TkIdent
                && self.current().literal == "fail"
            {
                self.advance();
                self.skip_newlines();
                fail_block = Some(self.parse_block()?);
                self.skip_newlines();
            }

            let upto_tok = self.expect(TokenKind::TkIdent)?;
            if upto_tok.literal != "upto" {
                return Err(format!(
                    "expected 'upto' at {}:{}",
                    tok.line, tok.col + 1
                ));
            }

            let max_expr = self.parse_expr()?;

            return Ok(Node::EvalTry(EvalTry {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                body: Box::new(Node::Block(body)),
                eval: Box::new(Node::Block(eval_body)),
                accept: accept_block.map(|b| Box::new(Node::Block(b))),
                partial: partial_block.map(|b| Box::new(Node::Block(b))),
                fail: fail_block.map(|b| Box::new(Node::Block(b))),
                max: Box::new(max_expr),
            }));
        }

        let mut fail_block = None;
        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "fail"
        {
            self.advance();
            self.skip_newlines();
            fail_block = Some(self.parse_block()?);
            self.skip_newlines();
        }

        let upto_tok = self.expect(TokenKind::TkIdent)?;
        if upto_tok.literal != "upto" {
            return Err(format!(
                "expected 'upto' at {}:{}",
                tok.line, tok.col + 1
            ));
        }

        let max_expr = self.parse_expr()?;

        Ok(Node::BinaryTry(BinaryTry {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            body: Box::new(Node::Block(body)),
            fail: fail_block.map(|b| Box::new(Node::Block(b))),
            max: Box::new(max_expr),
        }))
    }

    fn parse_within(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "begin"
        {
            self.advance();
            self.skip_newlines();
            let path = self.parse_expr()?;
            return Ok(Node::WithinToggle(WithinToggle {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                active: true,
                path: Some(Box::new(path)),
            }));
        }
        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "end"
        {
            self.advance();
            return Ok(Node::WithinToggle(WithinToggle {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                active: false,
                path: None,
            }));
        }

        let dir = self.parse_expr()?;
        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Node::DirBlock(DirBlock {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            dir: Box::new(dir),
            body: Box::new(Node::Block(body)),
        }))
    }

    fn parse_wait(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();

        let body = if self.current().kind == TokenKind::TkLBrace {
            Some(Box::new(Node::Block(self.parse_block()?)))
        } else {
            None
        };

        Ok(Node::WaitBlock(WaitBlock {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            body,
        }))
    }

    fn parse_exec(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();

        let raw = if self.source.is_empty() {
            let mut parts: Vec<String> = Vec::new();
            while self.pos < self.tokens.len() {
                let cur = self.current();
                if cur.kind == TokenKind::TkNewline || cur.kind == TokenKind::TkEOF {
                    break;
                }
                parts.push(cur.literal.clone());
                self.advance();
            }
            parts.join(" ")
        } else {
            let line_text = self.source.lines()
                .nth(tok.line - 1)
                .unwrap_or("");
            let after_exec = &line_text[tok.col.saturating_sub(1).min(line_text.len())..];
            after_exec.trim().to_string()
        };

        while self.pos < self.tokens.len() {
            let cur = self.current();
            if cur.kind == TokenKind::TkNewline || cur.kind == TokenKind::TkEOF {
                break;
            }
            self.advance();
        }

        let cmd = Node::StringLiteral(StringLiteral {
            pos: Pos { line: tok.line, col: tok.col },
            value: raw,
            interps: vec![],
        });

        Ok(Node::Exec(Exec {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            cmd: Box::new(cmd),
        }))
    }

    fn parse_print(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let msg = self.parse_expr()?;

        Ok(Node::Print(Print {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            message: Box::new(msg),
        }))
    }

    fn parse_exit(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let code = self.parse_expr()?;

        Ok(Node::Exit(Exit {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            code: Box::new(code),
        }))
    }

    fn parse_include(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let path = self.parse_expr()?;

        Ok(Node::Include(Include {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            path: Box::new(path),
        }))
    }

    fn parse_compact_stmt(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();
        let arg = self.parse_expr()?;

        Ok(Node::CompactStmt(CompactStmt {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            arg: Box::new(arg),
        }))
    }

    fn parse_session(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();

        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "begin"
        {
            self.advance();
            return Ok(Node::SessionToggle(SessionToggle {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                active: true,
            }));
        }
        if self.current().kind == TokenKind::TkIdent
            && self.current().literal == "end"
        {
            self.advance();
            return Ok(Node::SessionToggle(SessionToggle {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                active: false,
            }));
        }

        let body = self.parse_block()?;

        Ok(Node::SessionBlock(SessionBlock {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            body: Box::new(Node::Block(body)),
        }))
    }

    fn parse_env(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        let key_tok = self.expect(TokenKind::TkIdent)?;

        Ok(Node::Env(Env {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            key: key_tok.literal,
        }))
    }

    fn parse_return(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        self.skip_newlines();

        if self.current().kind == TokenKind::TkNewline
            || self.current().kind == TokenKind::TkRBrace
            || self.current().kind == TokenKind::TkEOF
        {
            return Ok(Node::Return(Return {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: None,
            }));
        }

        let val = self.parse_expr()?;

        Ok(Node::Return(Return {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            value: Some(Box::new(val)),
        }))
    }

    fn parse_break(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        Ok(Node::Break(Break {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
        }))
    }

    fn parse_continue(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        Ok(Node::Continue(Continue {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
        }))
    }

    // --- Expression parsing ---

    fn parse_expr(&mut self) -> Result<Node, String> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: u32) -> Result<Node, String> {
        let mut left = self.parse_primary()?;

        while self.current().kind == TokenKind::TkLBracket {
            self.advance();
            let idx = self.parse_binary_expr(0)?;
            self.expect(TokenKind::TkRBracket)?;
            let pos = left.pos().clone();
            left = Node::IndexExpr(IndexExpr {
                pos,
                object: Box::new(left),
                index: Box::new(idx),
            });
        }

        loop {
            let cur = self.current();
            let op_str = match cur.kind {
                TokenKind::TkEq => "==",
                TokenKind::TkNeq => "!=",
                TokenKind::TkGt => ">",
                TokenKind::TkLt => "<",
                TokenKind::TkGte => ">=",
                TokenKind::TkLte => "<=",
                TokenKind::TkPlus => "+",
                TokenKind::TkMinus => "-",
                TokenKind::TkStar => "*",
                TokenKind::TkSlash => "/",
                TokenKind::TkIdent if cur.literal == "and" || cur.literal == "or" => {
                    &cur.literal
                }
                _ => break,
            };

            let prec = match op_str {
                "or" => 1,
                "and" => 2,
                "==" | "!=" | ">" | "<" | ">=" | "<=" => 3,
                "+" | "-" => 4,
                "*" | "/" => 5,
                _ => unreachable!(),
            };

            if prec < min_prec {
                break;
            }

            self.advance();

            let right = self.parse_binary_expr(prec + 1)?;
            let pos = left.pos().clone();
            left = Node::BinaryExpr(BinaryExpr {
                pos,
                left: Box::new(left),
                op: op_str.to_string(),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Node, String> {
        let tok = self.current().clone();

        if tok.kind == TokenKind::TkLBracket {
            return self.parse_array_literal();
        }

        if tok.kind == TokenKind::TkLParen {
            self.advance();
            let expr = self.parse_binary_expr(0)?;
            self.expect(TokenKind::TkRParen)?;
            return Ok(Node::GroupExpr(GroupExpr {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                inner: Box::new(expr),
            }));
        }

        if tok.kind == TokenKind::TkIdent && tok.literal == "not" {
            self.advance();
            let right = self.parse_primary()?;
            return Ok(Node::UnaryExpr(UnaryExpr {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                op: "not".to_string(),
                right: Box::new(right),
            }));
        }

        if tok.kind == TokenKind::TkMinus {
            self.advance();
            let right = self.parse_primary()?;
            return Ok(Node::UnaryExpr(UnaryExpr {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                op: "-".to_string(),
                right: Box::new(right),
            }));
        }

        if tok.kind == TokenKind::TkString {
            self.advance();
            return Ok(Node::StringLiteral(StringLiteral {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: tok.literal,
                interps: Vec::new(),
            }));
        }

        if tok.kind == TokenKind::TkTextBlock {
            self.advance();
            return Ok(Node::TextBlock(TextBlock {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: tok.literal,
                interps: Vec::new(),
            }));
        }

        if tok.kind == TokenKind::TkInt {
            self.advance();
            let val = tok.literal.parse::<i64>().unwrap_or(0);
            return Ok(Node::IntLiteral(IntLiteral {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: val,
            }));
        }

        if tok.kind == TokenKind::TkFloat {
            self.advance();
            let val = tok.literal.parse::<f64>().unwrap_or(0.0);
            return Ok(Node::FloatLiteral(FloatLiteral {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: val,
            }));
        }

        if tok.kind == TokenKind::TkDollarLBrace {
            self.advance();
            return Ok(Node::VarRef(VarRef {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                name: tok.literal,
            }));
        }

        if tok.kind == TokenKind::TkDollarLParen {
            self.advance();
            return Ok(Node::CommandSubst(CommandSubst {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                cmd: tok.literal,
            }));
        }

        if tok.kind == TokenKind::TkAt {
            self.advance();
            let path = self.parse_primary()?;
            return Ok(Node::FilePath(FilePath {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                path: Box::new(path),
            }));
        }

        if tok.kind == TokenKind::TkIdent && tok.literal == "true" {
            self.advance();
            return Ok(Node::BoolLiteral(BoolLiteral {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: true,
            }));
        }

        if tok.kind == TokenKind::TkIdent && tok.literal == "false" {
            self.advance();
            return Ok(Node::BoolLiteral(BoolLiteral {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                value: false,
            }));
        }

        if tok.kind == TokenKind::TkIdent && self.peek().kind == TokenKind::TkLParen {
            return self.parse_fn_call();
        }

        if tok.kind == TokenKind::TkIdent {
            if tok.literal == "and" || tok.literal == "or" {
                return Err(format!(
                    "unexpected keyword {:?} in expression at {}:{}",
                    tok.literal, tok.line, tok.col
                ));
            }
            self.advance();
            return Ok(Node::VarRef(VarRef {
                pos: Pos {
                    line: tok.line,
                    col: tok.col,
                },
                name: tok.literal,
            }));
        }

        Err(format!(
            "unexpected token {}({:?}) at {}:{}",
            tok.kind, tok.literal, tok.line, tok.col
        ))
    }

    fn parse_array_literal(&mut self) -> Result<Node, String> {
        let tok = self.advance();
        let mut elements = Vec::new();
        self.skip_newlines();

        if self.current().kind != TokenKind::TkRBracket {
            let expr = self.parse_binary_expr(0)?;
            elements.push(expr);
            self.skip_newlines();

            while self.current().kind == TokenKind::TkComma {
                self.advance();
                self.skip_newlines();
                if self.current().kind == TokenKind::TkRBracket {
                    break;
                }
                let expr = self.parse_binary_expr(0)?;
                elements.push(expr);
                self.skip_newlines();
            }
        }

        self.expect(TokenKind::TkRBracket)?;

        Ok(Node::ArrayLiteral(ArrayLiteral {
            pos: Pos {
                line: tok.line,
                col: tok.col,
            },
            elements,
        }))
    }

    fn parse_hyphenated_ident(&mut self) -> Result<String, String> {
        let mut name = String::new();
        loop {
            let tok = self.expect(TokenKind::TkIdent)?;
            name.push_str(&tok.literal);
            if self.current().kind == TokenKind::TkMinus
                && self.peek().kind == TokenKind::TkIdent
            {
                name.push('-');
                self.advance();
            } else {
                break;
            }
        }
        Ok(name)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Script, String> {
    Parser::new(tokens, "").parse()
}

pub fn parse_str(src: &str) -> Result<Script, String> {
    let tokens = lexer::tokenize(src)?;
    Parser::new(tokens, src).parse()
}

pub fn parse_with_source(tokens: Vec<Token>, source: &str) -> Result<Script, String> {
    Parser::new(tokens, source).parse()
}

pub fn parse_expr_str(src: &str) -> Result<Node, String> {
    let tokens = lexer::tokenize(src)?;
    Parser::new(tokens, src).parse_expr()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(src: &str) -> Script {
        super::parse_str(src).unwrap()
    }

    #[test]
    fn test_empty_script() {
        let script = parse_str("");
        assert_eq!(script.body.len(), 0);
    }

    #[test]
    fn test_shebang() {
        let script = parse_str("#!opencode:1.2.0:sonnet");
        assert!(script.shebang.is_some());
        let sh = script.shebang.unwrap();
        assert_eq!(sh.engine, "opencode");
        assert_eq!(sh.version, "1.2.0");
        assert_eq!(sh.model, "sonnet");
    }

    #[test]
    fn test_compact_config() {
        let script = parse_str("#!compact mode=on window=64000\nprint \"hello\"");
        assert!(script.compact.is_some());
        let cc = script.compact.unwrap();
        assert_eq!(cc.mode, "on");
        assert_eq!(cc.window, "64000");
    }

    #[test]
    fn test_var_assign_string() {
        let script = parse_str(r#"MSG = "hello""#);
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::VarAssign(va) => {
                assert_eq!(va.name, "MSG");
                match &*va.value {
                    Node::StringLiteral(s) => assert_eq!(s.value, "hello"),
                    _ => panic!("expected StringLiteral"),
                }
            }
            _ => panic!("expected VarAssign"),
        }
    }

    #[test]
    fn test_var_assign_int() {
        let script = parse_str("X = 42");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::VarAssign(va) => {
                assert_eq!(va.name, "X");
                match &*va.value {
                    Node::IntLiteral(i) => assert_eq!(i.value, 42),
                    _ => panic!("expected IntLiteral"),
                }
            }
            _ => panic!("expected VarAssign"),
        }
    }

    #[test]
    fn test_var_assign_var_ref() {
        let script = parse_str("MSG = $NAME");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::VarAssign(va) => {
                match &*va.value {
                    Node::VarRef(vr) => assert_eq!(vr.name, "NAME"),
                    _ => panic!("expected VarRef"),
                }
            }
            _ => panic!("expected VarAssign"),
        }
    }

    #[test]
    fn test_var_assign_bare_ident() {
        let script = parse_str("MSG = NAME");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::VarAssign(va) => {
                match &*va.value {
                    Node::VarRef(vr) => assert_eq!(vr.name, "NAME"),
                    _ => panic!("expected VarRef"),
                }
            }
            _ => panic!("expected VarAssign"),
        }
    }

    #[test]
    fn test_expr() {
        let cases = [
            "$? == 0",
            "$COUNT > 5",
            "$OK and $READY",
            "not $FAILED",
            "$TOTAL + 1",
            "$N * 2",
            "($X > 0 and $Y < 10)",
        ];
        for src in &cases {
            let script = parse_str(src);
            assert_eq!(
                script.body.len(),
                1,
                "{:?}: expected 1 statement, got {}",
                src,
                script.body.len()
            );
        }
    }

    #[test]
    fn test_expr_bare_ident() {
        let cases = [
            "? == 0",
            "COUNT > 5",
            "OK and READY",
            "not FAILED",
            "TOTAL + 1",
            "N * 2",
            "(X > 0 and Y < 10)",
        ];
        for src in &cases {
            let script = parse_str(src);
            assert_eq!(
                script.body.len(),
                1,
                "{:?}: expected 1 statement, got {}",
                src,
                script.body.len()
            );
        }
    }

    #[test]
    fn test_if_stmt() {
        let script = parse_str("if $OK { print 1 }");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::IfStmt(s) => {
                assert!(s.else_ifs.is_empty());
                assert!(s.else_body.is_none());
            }
            _ => panic!("expected IfStmt"),
        }
    }

    #[test]
    fn test_if_else_stmt() {
        let script = parse_str("if $OK { print 1 } else { print 2 }");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::IfStmt(s) => {
                assert!(s.else_ifs.is_empty());
                assert!(s.else_body.is_some());
            }
            _ => panic!("expected IfStmt"),
        }
    }

    #[test]
    fn test_for_stmt() {
        let script = parse_str("for x in [1, 2, 3] { print x }");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::ForStmt(s) => {
                assert_eq!(s.var, "x");
            }
            _ => panic!("expected ForStmt"),
        }
    }

    #[test]
    fn test_while_stmt() {
        let script = parse_str("while $OK { print 1 }");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::WhileStmt(_) => {}
            _ => panic!("expected WhileStmt"),
        }
    }

    #[test]
    fn test_fn_decl() {
        let script = parse_str("fn greet(name) { print name }");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::FnDecl(f) => {
                assert_eq!(f.name, "greet");
                assert_eq!(f.params, vec!["name"]);
            }
            _ => panic!("expected FnDecl"),
        }
    }

    #[test]
    fn test_fn_call() {
        let script = parse_str("greet(\"world\")");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::FnCall(f) => {
                assert_eq!(f.name, "greet");
                assert_eq!(f.args.len(), 1);
            }
            _ => panic!("expected FnCall"),
        }
    }

    #[test]
    fn test_full_example() {
        let src = "#!opencode:1.2.0\n#!compact mode=on strategy=truncate window=64000\n\nMSG = \"hello\"\nprint MSG\n";
        let script = parse_str(src);
        assert!(script.shebang.is_some());
        assert!(script.compact.is_some());
        assert_eq!(script.body.len(), 2);
    }

    #[test]
    fn test_parse_errors() {
        let cases = [
            "if { }",
            "for { }",
            "do \"prompt\" with",
            "try { } upto",
        ];
        for src in &cases {
            let tokens = crate::lexer::tokenize(src).unwrap();
            let result = parse(tokens);
            assert!(result.is_err(), "expected parse error for {:?}", src);
        }
    }

    #[test]
    fn test_index_expr_precedence() {
        let script = parse_str("\"prefix\" + arr[0]");
        assert_eq!(script.body.len(), 1);
        match &script.body[0] {
            Node::BinaryExpr(b) => {
                assert_eq!(b.op, "+");
                match &*b.left {
                    Node::StringLiteral(_) => {}
                    other => panic!("expected StringLiteral on left, got {:?}", other),
                }
                match &*b.right {
                    Node::IndexExpr(_) => {}
                    other => panic!("expected IndexExpr on right, got {:?}", other),
                }
            }
            other => panic!("expected BinaryExpr, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_binary_try() {
        let src = "try { print 1 } fail { print 2 } upto 3";
        let script = parse_str(src);
        match &script.body[0] {
            Node::BinaryTry(_) => {}
            other => panic!("expected BinaryTry, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_eval_try() {
        let src = "try { print 1 } evaluate with { print 2 } accept { print 3 } partial { print 4 } fail { print 5 } upto 3";
        let script = parse_str(src);
        match &script.body[0] {
            Node::EvalTry(_) => {}
            other => panic!("expected EvalTry, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_session_block() {
        let script = parse_str("session { print 1 }");
        match &script.body[0] {
            Node::SessionBlock(_) => {}
            other => panic!("expected SessionBlock, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_session_toggle() {
        let script = parse_str("session begin");
        match &script.body[0] {
            Node::SessionToggle(t) => assert!(t.active),
            other => panic!("expected SessionToggle, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_within_toggle() {
        let script = parse_str("within begin \"/tmp\"");
        match &script.body[0] {
            Node::WithinToggle(t) => assert!(t.active),
            other => panic!("expected WithinToggle, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_background() {
        let script = parse_str("print 1 &");
        match &script.body[0] {
            Node::Background(_) => {}
            other => panic!("expected Background, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_compact_stmt() {
        let script = parse_str("compact \"truncate 32000\"");
        match &script.body[0] {
            Node::CompactStmt(_) => {}
            other => panic!("expected CompactStmt, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_dir_block() {
        let script = parse_str("within \"/tmp\" { print 1 }");
        match &script.body[0] {
            Node::DirBlock(_) => {}
            other => panic!("expected DirBlock, got {:?}", other),
        }
    }
}
