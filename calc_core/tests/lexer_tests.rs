use calc_core::lexer::{lex, TokenKind};

#[test]
fn lex_numbers_idents_and_symbols() {
    let toks = lex("10kΩ // 5 Ω # comment");
    assert!(matches!(toks[0].kind, TokenKind::Number(_)));
    assert!(matches!(toks[1].kind, TokenKind::Ident(ref s) if s == "kΩ"));
    assert!(matches!(toks[2].kind, TokenKind::Parallel));
    assert!(matches!(toks[3].kind, TokenKind::Number(_)));
    assert!(matches!(toks[4].kind, TokenKind::Ident(ref s) if s == "Ω"));
}

#[test]
fn lex_unicode_idents() {
    let toks = lex("π μ ° Ω ohm");
    assert!(matches!(toks[0].kind, TokenKind::Ident(ref s) if s == "π"));
    assert!(matches!(toks[1].kind, TokenKind::Ident(ref s) if s == "μ"));
    assert!(matches!(toks[2].kind, TokenKind::Ident(ref s) if s == "°"));
    assert!(matches!(toks[3].kind, TokenKind::Ident(ref s) if s == "Ω"));
    assert!(matches!(toks[4].kind, TokenKind::Ident(ref s) if s == "ohm"));
}


