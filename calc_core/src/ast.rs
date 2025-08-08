use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Complex { re: f64, im: f64 },
    Quantity { value: f64, unit: String },
    Bool(bool),
    String(String),
    Ident(String),
    Array(Vec<Expr>),
    Record(Vec<(String, Expr)>),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Assign { name: String, expr: Box<Expr> },
    Function { name: String, params: Vec<String>, body: Box<Expr> },
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp { Plus, Minus }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Parallel,
    Convert,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellAst {
    pub text: String,
    pub expr: Expr,
}

