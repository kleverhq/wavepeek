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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogicalExprAst {
    pub root: LogicalExprNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LogicalExprNode {
    OperandRef {
        name: String,
        span: Span,
    },
    IntegralLiteral {
        literal: IntegralLiteral,
        span: Span,
    },
    UnaryNot {
        expr: Box<LogicalExprNode>,
        span: Span,
    },
    Binary {
        op: LogicalBinaryOp,
        left: Box<LogicalExprNode>,
        right: Box<LogicalExprNode>,
        span: Span,
    },
}

impl LogicalExprNode {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::OperandRef { span, .. }
            | Self::IntegralLiteral { span, .. }
            | Self::UnaryNot { span, .. }
            | Self::Binary { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogicalBinaryOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    AndAnd,
    OrOr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegralLiteral {
    pub width: Option<u32>,
    pub signed: bool,
    pub base: IntegralBase,
    pub digits: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegralBase {
    Binary,
    Decimal,
    Hex,
}
