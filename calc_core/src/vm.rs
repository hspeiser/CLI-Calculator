use std::collections::HashMap;

use anyhow::Result;

use crate::ast::BinaryOp;
use crate::ir::{Chunk, OpCode};
use crate::registry::FunctionRegistry;
use crate::types::{Dim, Value, UserFunction};
use crate::units::{lookup_unit, resolve_prefixed_unit};

#[derive(Default)]
pub struct VmState {
    pub symbols: HashMap<String, Value>,
}

pub fn eval_chunk(chunk: &Chunk, registry: &FunctionRegistry, state: &mut VmState) -> Result<Value> {
    let mut stack: Vec<Value> = Vec::new();
    for op in &chunk.ops {
        match op {
            OpCode::Const(v) => stack.push(v.clone()),
            OpCode::LoadSym(name) => stack.push(state.symbols.get(name).cloned().unwrap_or(Value::String(format!("<unknown:{}>", name)))),
            OpCode::StoreSym(name) => {
                let v = stack.pop().unwrap_or(Value::Number(0.0));
                state.symbols.insert(name.clone(), v.clone());
                stack.push(v);
            }
            OpCode::Unary(u) => {
                let v = stack.pop().unwrap_or(Value::Number(0.0));
                match (u, v) {
                    (crate::ast::UnaryOp::Plus, x) => stack.push(x),
                    (crate::ast::UnaryOp::Minus, Value::Number(n)) => stack.push(Value::Number(-n)),
                    (crate::ast::UnaryOp::Minus, Value::Complex(c)) => stack.push(Value::Complex(-c)),
                    _ => stack.push(Value::String("<bad-unary>".into())),
                }
            }
            OpCode::Binary(b) => {
                let right = stack.pop().unwrap_or(Value::Number(0.0));
                let left = stack.pop().unwrap_or(Value::Number(0.0));
                stack.push(apply_binary(*b, left, right)?);
            }
            OpCode::LoadUnit(u) => {
                stack.push(Value::String(u.clone()));
            }
            OpCode::CallName(name, argc) => {
                let mut args = Vec::with_capacity(*argc);
                for _ in 0..*argc { args.push(stack.pop().unwrap_or(Value::Number(0.0))); }
                args.reverse();
                if let Some(meta) = registry.get(name) {
                    let v = (meta.func)(&args)?;
                    stack.push(v);
                } else if let Some(Value::Function(f)) = state.symbols.get(name) {
                    let mut local = VmState { symbols: state.symbols.clone() };
                    for (i, p) in f.params.iter().enumerate() { if let Some(v) = args.get(i) { local.symbols.insert(p.clone(), v.clone()); } }
                    let chunk = crate::ir::lower_expr(&f.body);
                    let v = eval_chunk(&chunk, registry, &mut local)?;
                    stack.push(v);
                } else {
                    stack.push(Value::String(format!("<unknown-fn:{}>", name)));
                }
            }
            OpCode::Invoke(argc) => {
                let mut args = Vec::with_capacity(*argc);
                for _ in 0..*argc { args.push(stack.pop().unwrap_or(Value::Number(0.0))); }
                args.reverse();
                let callee = stack.pop().unwrap_or(Value::String("<no callee>".into()));
                match callee {
                    Value::Function(f) => {
                        let mut local = VmState { symbols: state.symbols.clone() };
                        for (i, p) in f.params.iter().enumerate() { if let Some(v) = args.get(i) { local.symbols.insert(p.clone(), v.clone()); } }
                        let chunk = crate::ir::lower_expr(&f.body);
                        let v = eval_chunk(&chunk, registry, &mut local)?;
                        stack.push(v);
                    }
                    Value::String(name) => {
                        if let Some(meta) = registry.get(&name) { let v = (meta.func)(&args)?; stack.push(v); }
                        else { stack.push(Value::String(format!("<unknown-fn:{}>", name))); }
                    }
                    _ => stack.push(Value::String("<not-callable>".into())),
                }
            }
            OpCode::Convert(target) => {
                let v = stack.pop().unwrap_or(Value::Number(0.0));
                let target_unit = target.clone();
                let to_u = if let Some((_d,_s,name)) = resolve_prefixed_unit(&target_unit) { name.to_string() } else { target_unit };
                match v {
                    Value::Quantity { value, dim, unit } => {
                        let from = lookup_unit(&unit);
                        let to = lookup_unit(&to_u);
                        if let (Some(f), Some(t)) = (from, to) {
                            if f.dim.is_compatible(&t.dim) {
                                // convert value to canonical base (by dividing scale), then to target (by multiplying target scale)
                                let base_value = value * f.scale; // since our scales are to canonical, multiplying suffices
                                let new_value = base_value / t.scale;
                                stack.push(Value::Quantity { value: new_value, dim: t.dim.clone(), unit: t.name.to_string() });
                                continue;
                            }
                        }
                        stack.push(Value::String("<unit-convert-error>".into()));
                    }
                    _ => stack.push(Value::String("<convert-non-quantity>".into())),
                }
            }
        }
    }
    Ok(stack.pop().unwrap_or(Value::Number(0.0)))
}

fn apply_binary(op: BinaryOp, left: Value, right: Value) -> Result<Value> {
    use BinaryOp::*;
    Ok(match op {
        Add => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Number(a), Value::Complex(b)) => Value::Complex(num_complex::Complex64::new(a, 0.0) + b),
            (Value::Complex(a), Value::Number(b)) => Value::Complex(a + num_complex::Complex64::new(b, 0.0)),
            (Value::Complex(a), Value::Complex(b)) => Value::Complex(a + b),
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Quantity{ value:b, unit:ub, dim:db }) if da.is_compatible(&db) && ua==ub => Value::Quantity{ value: a+b, unit: ua, dim: da },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::ComplexQuantity{ value:b, unit:ub, dim:db }) if da.is_compatible(&db) && ua==ub => Value::ComplexQuantity{ value: a+b, unit: ua, dim: da },
            _ => Value::String("<type-error:+>".into()),
        },
        Sub => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (Value::Number(a), Value::Complex(b)) => Value::Complex(num_complex::Complex64::new(a, 0.0) - b),
            (Value::Complex(a), Value::Number(b)) => Value::Complex(a - num_complex::Complex64::new(b, 0.0)),
            (Value::Complex(a), Value::Complex(b)) => Value::Complex(a - b),
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Quantity{ value:b, unit:ub, dim:db }) if da.is_compatible(&db) && ua==ub => Value::Quantity{ value: a-b, unit: ua, dim: da },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::ComplexQuantity{ value:b, unit:ub, dim:db }) if da.is_compatible(&db) && ua==ub => Value::ComplexQuantity{ value: a-b, unit: ua, dim: da },
            _ => Value::String("<type-error:->".into()),
        },
        Mul => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (Value::Complex(a), Value::Complex(b)) => Value::Complex(a * b),
            (Value::Complex(a), Value::Number(b)) => Value::Complex(a * b),
            (Value::Number(a), Value::Complex(b)) => Value::Complex(a * b),
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Quantity{ value:b, unit:ub, dim:db }) => Value::Quantity{ value: a*b, unit: format!("{}*{}", ua, ub), dim: da.add(&db) },
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Number(b)) => Value::Quantity{ value: a*b, unit: ua, dim: da },
            (Value::Number(a), Value::Quantity{ value:b, unit:ub, dim:db }) => Value::Quantity{ value: a*b, unit: ub, dim: db },
            (Value::Complex(a), Value::Quantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: a*b, unit: ub, dim: db },
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Complex(b)) => Value::ComplexQuantity{ value: b*a, unit: ua, dim: da },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::Number(b)) => Value::ComplexQuantity{ value: a*b, unit: ua, dim: da },
            (Value::Number(a), Value::ComplexQuantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: a*b, unit: ub, dim: db },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::Complex(b)) => Value::ComplexQuantity{ value: a*b, unit: ua, dim: da },
            (Value::Complex(a), Value::ComplexQuantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: a*b, unit: ub, dim: db },
            _ => Value::String("<type-error:*>".into()),
        },
        Div => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            (Value::Complex(a), Value::Complex(b)) => Value::Complex(a / b),
            (Value::Complex(a), Value::Number(b)) => Value::Complex(a / b),
            (Value::Number(a), Value::Complex(b)) => Value::Complex(num_complex::Complex64::new(a, 0.0) / b),
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Quantity{ value:b, unit:ub, dim:db }) => Value::Quantity{ value: a/b, unit: format!("{}/{}", ua, ub), dim: da.sub(&db) },
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Number(b)) => Value::Quantity{ value: a/b, unit: ua, dim: da },
            (Value::Number(a), Value::Quantity{ value:b, unit:ub, dim:db }) => Value::Quantity{ value: a/b, unit: format!("1/{}", ub), dim: Dim{ exponents: db.exponents.map(|e| -e) } },
            (Value::Complex(a), Value::Quantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: a/b, unit: format!("1/{}", ub), dim: Dim{ exponents: db.exponents.map(|e| -e) } },
            (Value::Quantity{ value:a, unit:ua, dim:da }, Value::Complex(b)) => Value::ComplexQuantity{ value: num_complex::Complex64::new(a,0.0)/b, unit: ua, dim: da },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::Number(b)) => Value::ComplexQuantity{ value: a/b, unit: ua, dim: da },
            (Value::Number(a), Value::ComplexQuantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: num_complex::Complex64::new(a,0.0)/b, unit: ub, dim: db },
            (Value::ComplexQuantity{ value:a, unit:ua, dim:da }, Value::Complex(b)) => Value::ComplexQuantity{ value: a/b, unit: ua, dim: da },
            (Value::Complex(a), Value::ComplexQuantity{ value:b, unit:ub, dim:db }) => Value::ComplexQuantity{ value: a/b, unit: format!("1/{}", ub), dim: Dim{ exponents: db.exponents.map(|e| -e) } },
            _ => Value::String("<type-error:/>".into()),
        },
        Mod => match (left, right) { (Value::Number(a), Value::Number(b)) => Value::Number(a % b), _ => Value::String("<type-error:%>".into()) },
        Pow => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),
            _ => Value::String("<type-error:^>".into()),
        },
        Parallel => match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number((a * b) / (a + b)),
            (Value::Quantity { value: a, unit: ua, dim: da }, Value::Quantity { value: b, unit: ub, dim: db }) if da.is_compatible(&db) => {
                Value::Quantity { value: (a * b) / (a + b), unit: ua, dim: da }
            }
            (Value::Quantity { value: a, unit: ua, dim: da }, Value::ComplexQuantity { value: b, unit: ub, dim: db }) if da.is_compatible(&db) => {
                let a_c = num_complex::Complex64::new(a, 0.0);
                Value::ComplexQuantity { value: (a_c * b) / (a_c + b), unit: ua, dim: da }
            }
            (Value::ComplexQuantity { value: a, unit: ua, dim: da }, Value::Quantity { value: b, unit: ub, dim: db }) if da.is_compatible(&db) => {
                let b_c = num_complex::Complex64::new(b, 0.0);
                Value::ComplexQuantity { value: (a * b_c) / (a + b_c), unit: ua, dim: da }
            }
            (Value::Complex(a), Value::Complex(b)) => Value::Complex((a * b) / (a + b)),
            (Value::ComplexQuantity { value: a, unit: ua, dim: da }, Value::ComplexQuantity { value: b, unit: ub, dim: db }) if da.is_compatible(&db) => {
                Value::ComplexQuantity { value: (a * b) / (a + b), unit: ua, dim: da }
            }
            _ => Value::String("<type-error://>".into()),
        },
        BinaryOp::Convert => Value::String("<unexpected-convert-binop>".into()),
    })
}

