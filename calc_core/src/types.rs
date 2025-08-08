use num_complex::Complex64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueKind {
    Scalar,
    ComplexScalar,
    Quantity,
    Bool,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dim {
    // L, M, T, I, Θ, N, J exponents
    pub exponents: [i8; 7],
}

impl Dim {
    pub const fn zero() -> Self { Self { exponents: [0;7] } }
    pub fn add(&self, other: &Dim) -> Dim { let mut e = [0i8;7]; for i in 0..7 { e[i] = self.exponents[i] + other.exponents[i]; } Dim { exponents: e } }
    pub fn sub(&self, other: &Dim) -> Dim { let mut e = [0i8;7]; for i in 0..7 { e[i] = self.exponents[i] - other.exponents[i]; } Dim { exponents: e } }
    pub fn mul_scalar(&self, n: i8) -> Dim { let mut e = [0i8;7]; for i in 0..7 { e[i] = self.exponents[i] * n; } Dim { exponents: e } }
    pub fn is_compatible(&self, other: &Dim) -> bool { self.exponents == other.exponents }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    Complex(Complex64),
    Quantity { value: f64, dim: Dim, unit: String },
    ComplexQuantity { value: Complex64, dim: Dim, unit: String },
    Bool(bool),
    String(String),
    #[serde(skip)]
    Function(UserFunction),
}

impl Value {
    pub fn display(&self) -> String {
        fn normalize_negative_zero(n: f64) -> f64 {
            if n == 0.0 { 0.0 } else { n }
        }

        fn format_number(n: f64) -> String {
            let mut v = normalize_negative_zero(n);
            // Snap very-close-to-integer values exactly to that integer to stabilize output
            let nearest = v.round();
            if (v - nearest).abs() < 1e-12 {
                v = nearest;
            }
            // Fixed precision, then trim trailing zeros and decimal point
            let mut s = format!("{:.12}", v);
            if s.contains('.') {
                while s.ends_with('0') { s.pop(); }
                if s.ends_with('.') { s.pop(); }
            }
            if s == "-0" { s = "0".to_string(); }
            s
        }
        match self {
            Value::Number(n) => format_number(*n),
            Value::Complex(c) => format!("({}+{}i)", format_number(c.re), format_number(c.im)),
            Value::Quantity{ value, unit, .. } => format!("{} {}", format_number(*value), unit),
            Value::ComplexQuantity{ value, unit, .. } => format!("({}+{}i) {}", format_number(value.re), format_number(value.im), unit),
            Value::Bool(b) => format!("{}", b),
            Value::String(s) => s.clone(),
            Value::Function(_) => "<fn>".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserFunction {
    pub params: Vec<String>,
    pub body: crate::ast::Expr,
}

