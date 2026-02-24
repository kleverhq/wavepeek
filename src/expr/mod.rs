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
    fn event_expr_iff_binding_with_union() {
        let expr =
            parse_event_expr("negedge clk iff rstn or bar").expect("event expression should parse");

        assert_eq!(expr.terms.len(), 2);
        assert!(matches!(expr.terms[0].event, EventKind::Negedge(_)));
        assert_eq!(expr.terms[0].iff_expr.as_deref(), Some("rstn"));
        assert!(matches!(expr.terms[1].event, EventKind::AnyChange(_)));
        assert!(expr.terms[1].iff_expr.is_none());
    }

    #[test]
    fn event_expr_iff_capture_parenthesized_or() {
        let expr = parse_event_expr("posedge clk iff (a or b) or bar")
            .expect("event expression should parse");

        assert_eq!(expr.terms.len(), 2);
        assert!(matches!(expr.terms[0].event, EventKind::Posedge(_)));
        assert_eq!(expr.terms[0].iff_expr.as_deref(), Some("(a or b)"));
        assert!(matches!(expr.terms[1].event, EventKind::AnyChange(_)));
    }
}
