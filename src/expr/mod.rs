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
