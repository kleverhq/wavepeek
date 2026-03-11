#![allow(dead_code)]

pub mod ast;
pub mod diagnostic;
pub(crate) mod eval;
pub(crate) mod host;
mod legacy;
mod lexer;
mod parser;
pub(crate) mod sema;

use crate::error::WavepeekError;

pub use crate::expr::ast::{BasicEventAst, DeferredLogicalExpr, EventExprAst, EventTermAst};
pub use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
pub use crate::expr::host::{EventEvalFrame, ExprType, ExpressionHost, SampledValue, SignalHandle};
pub use crate::expr::lexer::{Token, TokenKind};
pub use crate::expr::sema::BoundEventExpr;

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

pub fn bind_event_expr_ast(
    ast: &EventExprAst,
    host: &dyn ExpressionHost,
) -> Result<BoundEventExpr, ExprDiagnostic> {
    sema::bind_event_expr_ast(ast, host)
}

pub fn event_matches_at(
    expr: &BoundEventExpr,
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, ExprDiagnostic> {
    eval::event_matches_at(expr, host, frame)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EventKind {
    AnyTracked,
    AnyChange(String),
    Posedge(String),
    Negedge(String),
    Edge(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventTerm {
    pub event: EventKind,
    pub iff_expr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventExpr {
    pub terms: Vec<EventTerm>,
}

pub(crate) fn parse(source: &str) -> Result<Expression, WavepeekError> {
    legacy::parse(source)
}

pub(crate) fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError> {
    legacy::parse_event_expr(source)
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
