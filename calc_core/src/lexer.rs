#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(String),
    Ident(String),
    Str(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Parallel, // //
    Caret,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Assign,
    Hash,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    let mut idx = 0usize;

    while let Some(&ch) = chars.peek() {
        let start = idx;
        match ch {
            '0'..='9' => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' || c == '_' {
                        s.push(c);
                        chars.next();
                        idx += c.len_utf8();
                    } else { break; }
                }
                tokens.push(Token { kind: TokenKind::Number(s), start, end: idx });
            }
            'a'..='z' | 'A'..='Z' | '_' | 'Ω' | 'μ' | 'π' | '°' => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || matches!(c, '_' | 'Ω' | 'μ' | 'π' | '°') {
                        s.push(c);
                        chars.next();
                        idx += c.len_utf8();
                    } else { break; }
                }
                tokens.push(Token { kind: TokenKind::Ident(s), start, end: idx });
            }
            '"' => {
                chars.next(); idx += 1;
                let mut s = String::new();
                while let Some(c) = chars.next() { idx += c.len_utf8(); if c == '"' { break; } s.push(c); }
                tokens.push(Token{ kind: TokenKind::Str(s), start, end: idx});
            }
            '+' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Plus,start,end:idx}); }
            '-' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Minus,start,end:idx}); }
            '*' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Star,start,end:idx}); }
            '%' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Percent,start,end:idx}); }
            '^' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Caret,start,end:idx}); }
            '(' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::LParen,start,end:idx}); }
            ')' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::RParen,start,end:idx}); }
            '[' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::LBracket,start,end:idx}); }
            ']' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::RBracket,start,end:idx}); }
            '{' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::LBrace,start,end:idx}); }
            '}' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::RBrace,start,end:idx}); }
            ',' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Comma,start,end:idx}); }
            ':' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Colon,start,end:idx}); }
            '=' => { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Assign,start,end:idx}); }
            '#' => { break; }
            '/' => {
                chars.next(); idx += 1;
                if let Some('/') = chars.peek().copied() { chars.next(); idx += 1; tokens.push(Token{kind:TokenKind::Parallel,start,end:idx}); }
                else { tokens.push(Token{kind:TokenKind::Slash,start,end:idx}); }
            }
            c if c.is_whitespace() => { chars.next(); idx += 1; }
            _ => { chars.next(); idx += 1; }
        }
    }
    tokens.push(Token{ kind: TokenKind::Eof, start: idx, end: idx });
    tokens
}

