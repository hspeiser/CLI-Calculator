use calc_core::parser::parse_cell;
use calc_core::ast::{Expr, BinaryOp};

#[test]
fn parse_assignment() {
    let p = parse_cell("x = 10");
    match p.expr {
        Expr::Assign { name, .. } => assert_eq!(name, "x"),
        _ => panic!("expected assignment"),
    }
}

#[test]
fn parse_implicit_multiplication_number_ident() {
    let p = parse_cell("10x");
    match p.expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Mul),
        _ => panic!("expected binary mul"),
    }
}

#[test]
fn parse_implicit_multiplication_paren() {
    let p = parse_cell("2(3+4)");
    match p.expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Mul),
        _ => panic!("expected binary mul"),
    }
}

#[test]
fn parse_function_definition_requires_equals() {
    // Should parse as call, not function def
    let p = parse_cell("pi()");
    match p.expr {
        Expr::Call { .. } => {}
        _ => panic!("expected call, not function def"),
    }
}

#[test]
fn parse_conversion_operator() {
    let p = parse_cell("10 m to in");
    match p.expr {
        Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Convert),
        _ => panic!("expected convert"),
    }
}


