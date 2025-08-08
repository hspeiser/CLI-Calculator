use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::diag::Diagnostic;
use crate::lexer::{lex, Token, TokenKind};
use crate::units::resolve_prefixed_unit;

#[derive(Debug)]
pub struct ParseResult {
    pub expr: Expr,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_cell(text: &str) -> ParseResult {
    let tokens = lex(text);
    let mut p = Parser { tokens, pos: 0, diagnostics: Vec::new(), text };
    let expr = p.parse_top();
    ParseResult { expr, diagnostics: p.diagnostics }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    text: &'a str,
}

impl<'a> Parser<'a> {
    fn looks_like_fn_def(&self) -> bool {
        // Check pattern: Ident '(' ... ')' '=' from current position
        if let Some(Token { kind: TokenKind::Ident(_), .. }) = self.tokens.get(self.pos) {
            if let Some(Token { kind: TokenKind::LParen, .. }) = self.tokens.get(self.pos + 1) {
                let mut i = self.pos + 2;
                let mut depth: i32 = 1;
                while let Some(tok) = self.tokens.get(i) {
                    match tok.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                // next token must be '=' to be a function definition
                                return matches!(self.tokens.get(i + 1).map(|t| &t.kind), Some(TokenKind::Assign));
                            }
                        }
                        TokenKind::Eof => break,
                        _ => {}
                    }
                    i += 1;
                }
            }
        }
        false
    }
    fn peek(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }
    fn bump(&mut self) -> &Token {
        if self.pos >= self.tokens.len() { return self.tokens.last().expect("lexer must produce EOF"); }
        let pos = self.pos;
        self.pos += 1;
        &self.tokens[pos]
    }
    fn at(&self, kind: &TokenKind) -> bool {
        matches!(self.tokens.get(self.pos).map(|t| &t.kind), Some(k) if k == kind)
    }

    fn parse_top(&mut self) -> Expr {
        // assignment or function: name = expr OR name(params) = expr
        if let TokenKind::Ident(name) = self.peek() {
            // function def only if there is an '=' after the parameter list
            if self.looks_like_fn_def() {
                self.bump(); // name
                let params = self.parse_params();
                if matches!(self.peek(), TokenKind::Assign) { let _ = self.bump(); } // '='
                let body = self.parse_expr(0);
                return Expr::Function { name, params, body: Box::new(body) };
            }
            if self.tokens.get(self.pos + 1).map(|t| matches!(t.kind, TokenKind::Assign)).unwrap_or(false) {
                self.bump(); // name
                self.bump(); // =
                let expr = self.parse_expr(0);
                return Expr::Assign { name, expr: Box::new(expr) };
            }
        }
        self.parse_expr(0)
    }

    // Pratt parser
    fn parse_expr(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();
        loop {
            let (op, lbp, rbp, implicit) = match self.peek() {
                TokenKind::Plus => (BinaryOp::Add, 10, 11, false),
                TokenKind::Minus => (BinaryOp::Sub, 10, 11, false),
                TokenKind::Star => (BinaryOp::Mul, 20, 21, false),
                TokenKind::Slash => (BinaryOp::Div, 20, 21, false),
                TokenKind::Percent => (BinaryOp::Mod, 20, 21, false),
                TokenKind::Parallel => (BinaryOp::Parallel, 18, 19, false),
                TokenKind::Caret => (BinaryOp::Pow, 30, 29, false), // right associative
                TokenKind::Ident(ref s) if s == "to" => (BinaryOp::Convert, 1, 2, false),
                // Implicit multiplication: adjacency like `10x`, `x y` or `2( … )`
                TokenKind::Ident(ref s) if s != "to" => (BinaryOp::Mul, 20, 21, true),
                TokenKind::LParen => (BinaryOp::Mul, 20, 21, true),
                _ => break,
            };
            if lbp < min_bp { break; }
            if !implicit { let _ = self.bump(); }
            let rhs = self.parse_expr(rbp);
            lhs = Expr::Binary { op, left: Box::new(lhs), right: Box::new(rhs) };
        }
        lhs
    }

    fn parse_prefix(&mut self) -> Expr {
        match self.peek() {
            TokenKind::Number(s) => {
                self.bump();
                let n: f64 = s.replace('_', "").parse().unwrap_or(0.0);
                // number followed by unit -> quantity
                if let TokenKind::Ident(unit_tok) = self.peek() {
                    if let Some((dim, scale, canon)) = resolve_prefixed_unit(&unit_tok) {
                        self.bump();
                        return Expr::Quantity { value: n * scale, unit: canon.to_string() };
                    }
                }
                Expr::Number(n)
            }
            TokenKind::Ident(id) => {
                self.bump();
                // bare unit literal (value 1)
                if let Some((_dim, _scale, canon)) = resolve_prefixed_unit(&id) {
                    return Expr::Quantity { value: 1.0, unit: canon.to_string() };
                }
                // call: name(args)
                if let TokenKind::LParen = self.peek() {
                    let _lp = self.bump(); // (
                    let mut args = Vec::new();
                    loop {
                        if let TokenKind::RParen = self.peek() { self.bump(); break; }
                        let e = self.parse_expr(0);
                        args.push(e);
                        if let TokenKind::Comma = self.peek() { self.bump(); continue; } else {
                            let _ = self.bump(); // best effort consume ')'
                            break;
                        }
                    }
                    return Expr::Call { callee: Box::new(Expr::Ident(id)), args };
                }
                Expr::Ident(id)
            }
            TokenKind::Minus => { self.bump(); let e = self.parse_expr(25); Expr::Unary { op: UnaryOp::Minus, expr: Box::new(e) } }
            TokenKind::Plus => { self.bump(); let e = self.parse_expr(25); Expr::Unary { op: UnaryOp::Plus, expr: Box::new(e) } }
            TokenKind::LParen => {
                self.bump();
                let e = self.parse_expr(0);
                if let TokenKind::RParen = self.peek() { let _ = self.bump(); }
                e
            }
            _ => { self.bump(); Expr::Error }
        }
    }

    fn parse_params(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        let _lp = self.bump(); // (
        loop {
            match self.peek() {
                TokenKind::Ident(id) => { self.bump(); params.push(id); }
                TokenKind::RParen => { self.bump(); break; }
                TokenKind::Comma => { self.bump(); }
                _ => { break; }
            }
        }
        // expect '=' next
        if !matches!(self.peek(), TokenKind::Assign) { /* tolerate */ }
        params
    }
}

