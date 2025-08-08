use serde::{Deserialize, Serialize};

use crate::diag::Diagnostic;
use crate::ir::{lower_expr, Chunk};
use crate::binder::bind_cell;
use crate::parser::parse_cell;
use crate::registry::{default_registry, FunctionRegistry};
use crate::vm::{eval_chunk, VmState};
use crate::types::Value;
use crate::units::try_canonicalize;
use num_complex::Complex64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalOutput {
    pub value: Value,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Engine {
    registry: FunctionRegistry,
    state: VmState,
}

impl Engine {
    pub fn new() -> Self {
        let mut state = VmState::default();
        // Seed common constants as identifiers
        state.symbols.insert("i".into(), Value::Complex(Complex64::new(0.0, 1.0)));
        state.symbols.insert("j".into(), Value::Complex(Complex64::new(0.0, 1.0)));
        state.symbols.insert("pi".into(), Value::Number(std::f64::consts::PI));
        state.symbols.insert("π".into(), Value::Number(std::f64::consts::PI));
        Self { registry: default_registry(), state }
    }

    pub fn eval_cell(&mut self, text: &str) -> anyhow::Result<EvalOutput> {
        let parsed = parse_cell(text);
        let (_defines, _uses) = bind_cell(&parsed.expr);
        let chunk: Chunk = lower_expr(&parsed.expr);
        let mut value = eval_chunk(&chunk, &self.registry, &mut self.state)?;
        // If V/Ω, collapse to A for nicer display
        if let Value::Quantity{ value: v, dim, unit } = &value {
            if unit.contains("V/") && unit.contains("Ω") {
                value = Value::Quantity { value: *v, dim: dim.clone(), unit: "A".into() };
            }
        }
        Ok(EvalOutput { value, diagnostics: parsed.diagnostics })
    }
}

