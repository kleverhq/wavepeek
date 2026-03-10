use crate::expr::diagnostic::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventExprAst {
    pub terms: Vec<EventTermAst>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTermAst {
    pub event: BasicEventAst,
    pub iff: Option<DeferredLogicalExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicEventAst {
    AnyTracked { span: Span },
    Named { name: String, span: Span },
    Posedge { name: String, span: Span },
    Negedge { name: String, span: Span },
    Edge { name: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredLogicalExpr {
    pub source: String,
    pub span: Span,
}
