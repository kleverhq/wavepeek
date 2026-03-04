#![allow(dead_code)]

pub mod eval;
pub mod lexer;
pub mod parser;

use crate::error::WavepeekError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    source: String,
}

impl Expression {
    pub fn source(&self) -> &str {
        &self.source
    }
}

pub fn parse(source: &str) -> Result<Expression, WavepeekError> {
    parser::parse(source)
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

pub fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError> {
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
