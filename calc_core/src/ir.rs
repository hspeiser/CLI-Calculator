use serde::{Deserialize, Serialize};

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::types::{Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpCode {
    Const(Value),
    LoadSym(String),
    StoreSym(String),
    Unary(UnaryOp),
    Binary(BinaryOp),
    LoadUnit(String),
    CallName(String, usize),
    Invoke(usize),
    Convert(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub ops: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self { Self { ops: Vec::new() } }
}

pub fn lower_expr(expr: &Expr) -> Chunk {
    let mut c = Chunk::new();
    lower_into(expr, &mut c);
    c
}

fn lower_into(expr: &Expr, c: &mut Chunk) {
    match expr {
        Expr::Number(n) => c.ops.push(OpCode::Const(Value::Number(*n))),
        Expr::Bool(b) => c.ops.push(OpCode::Const(Value::Bool(*b))),
        Expr::String(s) => c.ops.push(OpCode::Const(Value::String(s.clone()))),
        Expr::Ident(name) => c.ops.push(OpCode::LoadSym(name.clone())),
        Expr::Unary { op, expr } => { lower_into(expr, c); c.ops.push(OpCode::Unary(*op)); }
        Expr::Binary { op, left, right } => {
            if matches!(op, BinaryOp::Convert) {
                // Lower: evaluate left then convert to named unit on right if ident
                lower_into(left, c);
                match &**right {
                    Expr::Ident(u) => c.ops.push(OpCode::Convert(u.clone())),
                    Expr::Quantity { unit, .. } => c.ops.push(OpCode::Convert(unit.clone())),
                    _ => c.ops.push(OpCode::Convert("".into())),
                }
            } else {
                lower_into(left, c); lower_into(right, c); c.ops.push(OpCode::Binary(*op));
            }
        }
        Expr::Assign { name, expr } => { lower_into(expr, c); c.ops.push(OpCode::StoreSym(name.clone())); }
        Expr::Call { callee, args } => {
            // If callee is an identifier, use CallName to reach registry or symbol name
            if let Expr::Ident(name) = &**callee {
                for a in args { lower_into(a, c); }
                c.ops.push(OpCode::CallName(name.clone(), args.len()));
            } else {
                // Lower dynamic: evaluate callee value then args, then invoke
                lower_into(callee, c);
                for a in args { lower_into(a, c); }
                c.ops.push(OpCode::Invoke(args.len()));
            }
        }
        Expr::Quantity { value, unit } => {
            // Attach dimension and canonical base unit name
            if let Some(u) = crate::units::lookup_unit(unit) {
                let dim = u.dim.clone();
                let canon = crate::units::canonical_unit_for_dim(&dim).unwrap_or(u.name);
                c.ops.push(OpCode::Const(Value::Quantity { value: *value, dim, unit: canon.to_string() }));
            } else {
                c.ops.push(OpCode::Const(Value::Quantity { value: *value, dim: crate::types::Dim::zero(), unit: unit.clone() }));
            }
        }
        Expr::Function { name, params, body } => {
            // Store a user function value under the name
            c.ops.push(OpCode::Const(Value::Function(crate::types::UserFunction { params: params.clone(), body: (*body.clone()) })));
            c.ops.push(OpCode::StoreSym(name.clone()));
        }
        _ => c.ops.push(OpCode::Const(Value::String("<unhandled>".into()))),
    }
}

