use crate::expr::diagnostic::Span;
use crate::expr::host::IntegerLikeKind;

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
pub struct LogicalExprAst {
    pub root: LogicalExprNode,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalExprNode {
    OperandRef {
        name: String,
        span: Span,
    },
    IntegralLiteral {
        literal: IntegralLiteral,
        span: Span,
    },
    RealLiteral {
        literal: RealLiteral,
        span: Span,
    },
    StringLiteral {
        literal: StringLiteral,
        span: Span,
    },
    EnumLabel {
        operand: String,
        operand_span: Span,
        label: String,
        label_span: Span,
        span: Span,
    },
    Parenthesized {
        expr: Box<LogicalExprNode>,
        span: Span,
    },
    Cast {
        target: CastTargetAst,
        expr: Box<LogicalExprNode>,
        span: Span,
    },
    Selection {
        base: Box<LogicalExprNode>,
        selection: SelectionKindAst,
        span: Span,
    },
    Unary {
        op: UnaryOpAst,
        expr: Box<LogicalExprNode>,
        span: Span,
    },
    Binary {
        op: BinaryOpAst,
        left: Box<LogicalExprNode>,
        right: Box<LogicalExprNode>,
        span: Span,
    },
    Conditional {
        condition: Box<LogicalExprNode>,
        when_true: Box<LogicalExprNode>,
        when_false: Box<LogicalExprNode>,
        span: Span,
    },
    Inside {
        expr: Box<LogicalExprNode>,
        set: Vec<InsideItemAst>,
        span: Span,
    },
    Concatenation {
        items: Vec<LogicalExprNode>,
        span: Span,
    },
    Replication {
        count: Box<LogicalExprNode>,
        expr: Box<LogicalExprNode>,
        span: Span,
    },
    Triggered {
        expr: Box<LogicalExprNode>,
        span: Span,
    },
}

impl LogicalExprNode {
    pub fn span(&self) -> Span {
        match self {
            Self::OperandRef { span, .. }
            | Self::IntegralLiteral { span, .. }
            | Self::RealLiteral { span, .. }
            | Self::StringLiteral { span, .. }
            | Self::EnumLabel { span, .. }
            | Self::Parenthesized { span, .. }
            | Self::Cast { span, .. }
            | Self::Selection { span, .. }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Conditional { span, .. }
            | Self::Inside { span, .. }
            | Self::Concatenation { span, .. }
            | Self::Replication { span, .. }
            | Self::Triggered { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastTargetAst {
    Signed,
    Unsigned,
    BitVector {
        width: u32,
        is_four_state: bool,
        is_signed: bool,
    },
    IntegerLike(IntegerLikeKind),
    Real,
    String,
    RecoveredType {
        name: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionKindAst {
    Bit {
        index: Box<LogicalExprNode>,
    },
    Part {
        msb: Box<LogicalExprNode>,
        lsb: Box<LogicalExprNode>,
    },
    IndexedUp {
        base: Box<LogicalExprNode>,
        width: Box<LogicalExprNode>,
    },
    IndexedDown {
        base: Box<LogicalExprNode>,
        width: Box<LogicalExprNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpAst {
    Plus,
    Minus,
    LogicalNot,
    BitNot,
    ReduceAnd,
    ReduceNand,
    ReduceOr,
    ReduceNor,
    ReduceXor,
    ReduceXnor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpAst {
    Power,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    ShiftArithLeft,
    ShiftArithRight,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    CaseEq,
    CaseNe,
    WildEq,
    WildNe,
    BitAnd,
    BitXor,
    BitXnor,
    BitOr,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsideItemAst {
    Expr(LogicalExprNode),
    Range {
        low: LogicalExprNode,
        high: LogicalExprNode,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegralLiteral {
    pub width: Option<u32>,
    pub signed: bool,
    pub base: IntegralBase,
    pub digits: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealLiteral {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegralBase {
    Binary,
    Decimal,
    Hex,
}
