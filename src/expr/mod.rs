#![allow(dead_code)]

pub mod ast;
pub mod diagnostic;
pub mod eval;
pub mod host;
pub mod lexer;
pub mod parser;
pub(crate) mod sema;

use crate::error::WavepeekError;

pub use crate::expr::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst};
pub use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
pub use crate::expr::lexer::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    source: String,
}

impl Expression {
    pub fn source(&self) -> &str {
        &self.source
    }
}

pub fn lex_event_expr(source: &str) -> Result<Vec<Token>, ExprDiagnostic> {
    lexer::lex_event_expr(source)
}

pub fn parse_event_expr_ast(source: &str) -> Result<EventExprAst, ExprDiagnostic> {
    parser::parse_event_expr_ast(source)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    AnyTracked,
    AnyChange(String),
    Posedge(String),
    Negedge(String),
    Edge(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTerm {
    pub event: EventKind,
    pub iff_expr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventExpr {
    pub terms: Vec<EventTerm>,
}

pub(crate) fn parse(source: &str) -> Result<Expression, WavepeekError> {
    parser::parse(source)
}

pub(crate) fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError> {
    parser::parse_event_expr(source)
}

#[cfg(test)]
mod tests {
    use super::{EventKind, parse_event_expr};

    #[test]
    fn parse_event_expr_wrapper_supports_any_tracked() {
        let expr = parse_event_expr("*").expect("event expression should parse");

        assert_eq!(expr.terms.len(), 1);
        assert!(matches!(expr.terms[0].event, EventKind::AnyTracked));
    }
}
