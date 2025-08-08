use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<(usize, usize)>,
    pub kind: DiagnosticKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticKind {
    Parse,
    UnknownSymbol,
    Type,
    UnitMismatch,
    Domain,
    Overflow,
    Circular,
    Internal,
}

impl Diagnostic {
    pub fn parse(message: impl Into<String>, span: Option<(usize, usize)>) -> Self {
        Self { message: message.into(), span, kind: DiagnosticKind::Parse }
    }
}

