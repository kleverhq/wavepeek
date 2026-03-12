use crate::expr::ast::{
    BasicEventAst, BinaryOpAst, CastTargetAst, EventExprAst, InsideItemAst, IntegralBase,
    IntegralLiteral, LogicalExprAst, LogicalExprNode, SelectionKindAst, UnaryOpAst,
};
use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
use crate::expr::host::{
    ExprStorage, ExprType, ExprTypeKind, ExpressionHost, IntegerLikeKind, SignalHandle,
};
use crate::expr::parser::parse_logical_expr_with_offset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundEventExpr {
    pub(crate) terms: Vec<BoundEventTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundEventTerm {
    pub event: BoundEventKind,
    pub iff: Option<BoundLogicalExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundEventKind {
    AnyTracked,
    Named(SignalHandle),
    Posedge(SignalHandle),
    Negedge(SignalHandle),
    Edge(SignalHandle),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundLogicalExpr {
    pub(crate) root: BoundLogicalNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundLogicalNode {
    pub ty: ExprType,
    pub span: Span,
    pub kind: BoundLogicalKind,
}

impl BoundLogicalNode {
    pub(crate) fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundLogicalKind {
    SignalRef {
        handle: SignalHandle,
    },
    IntegralLiteral {
        value: BoundIntegralValue,
        is_unsized: bool,
    },
    Parenthesized {
        expr: Box<BoundLogicalNode>,
    },
    Cast {
        target: CastTargetAst,
        expr: Box<BoundLogicalNode>,
    },
    Selection {
        base: Box<BoundLogicalNode>,
        selection: BoundSelection,
    },
    Unary {
        op: UnaryOpAst,
        expr: Box<BoundLogicalNode>,
    },
    Binary {
        op: BinaryOpAst,
        left: Box<BoundLogicalNode>,
        right: Box<BoundLogicalNode>,
    },
    Conditional {
        condition: Box<BoundLogicalNode>,
        when_true: Box<BoundLogicalNode>,
        when_false: Box<BoundLogicalNode>,
    },
    Inside {
        expr: Box<BoundLogicalNode>,
        set: Vec<BoundInsideItem>,
    },
    Concatenation {
        items: Vec<BoundLogicalNode>,
    },
    Replication {
        count: usize,
        expr: Box<BoundLogicalNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundSelection {
    Bit {
        index: Box<BoundLogicalNode>,
    },
    Part {
        msb: i64,
        lsb: i64,
    },
    IndexedUp {
        base: Box<BoundLogicalNode>,
        width: usize,
    },
    IndexedDown {
        base: Box<BoundLogicalNode>,
        width: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundInsideItem {
    Expr(BoundLogicalNode),
    Range {
        low: BoundLogicalNode,
        high: BoundLogicalNode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundIntegralValue {
    pub bits: Vec<BoundBit>,
    pub signed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundBit {
    Zero,
    One,
    X,
    Z,
}

pub fn bind_event_expr_ast(
    ast: &EventExprAst,
    host: &dyn ExpressionHost,
) -> Result<BoundEventExpr, ExprDiagnostic> {
    let mut terms = Vec::with_capacity(ast.terms.len());
    for term in &ast.terms {
        let event = match &term.event {
            BasicEventAst::AnyTracked { .. } => BoundEventKind::AnyTracked,
            BasicEventAst::Named { name, .. } => BoundEventKind::Named(host.resolve_signal(name)?),
            BasicEventAst::Posedge { name, .. } => {
                BoundEventKind::Posedge(host.resolve_signal(name)?)
            }
            BasicEventAst::Negedge { name, .. } => {
                BoundEventKind::Negedge(host.resolve_signal(name)?)
            }
            BasicEventAst::Edge { name, .. } => BoundEventKind::Edge(host.resolve_signal(name)?),
        };

        let iff = if let Some(iff) = &term.iff {
            let logical_ast = parse_logical_expr_with_offset(iff.source.as_str(), iff.span.start)?;
            Some(bind_logical_expr_ast(&logical_ast, host)?)
        } else {
            None
        };

        terms.push(BoundEventTerm { event, iff });
    }
    Ok(BoundEventExpr { terms })
}

pub fn bind_logical_expr_ast(
    ast: &LogicalExprAst,
    host: &dyn ExpressionHost,
) -> Result<BoundLogicalExpr, ExprDiagnostic> {
    let root = bind_logical_node(&ast.root, host)?;
    Ok(BoundLogicalExpr { root })
}

fn bind_logical_node(
    node: &LogicalExprNode,
    host: &dyn ExpressionHost,
) -> Result<BoundLogicalNode, ExprDiagnostic> {
    match node {
        LogicalExprNode::OperandRef { name, span } => {
            let handle = host.resolve_signal(name).map_err(|inner| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "C3-SEMANTIC-UNKNOWN-SIGNAL",
                message: format!("unknown signal '{name}'"),
                primary_span: *span,
                notes: if inner.message.is_empty() {
                    vec![]
                } else {
                    vec![format!("host detail: {}", inner.message)]
                },
            })?;
            let ty = host.signal_type(handle)?;
            if !is_integral_type(&ty) {
                return Err(sema_diag(
                    "C3-SEMANTIC-EXPECTED-INTEGRAL",
                    "logical expressions in C3 require integral operands",
                    *span,
                    &["non-integral operand types are deferred to C4"],
                ));
            }
            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::SignalRef { handle },
            })
        }
        LogicalExprNode::IntegralLiteral { literal, span } => {
            let value = decode_integral_literal(literal)?;
            let is_four_state = match literal.base {
                IntegralBase::Binary | IntegralBase::Hex => true,
                IntegralBase::Decimal => value
                    .bits
                    .iter()
                    .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z)),
            };
            let ty = bit_vector_type(
                value.bits.len() as u32,
                is_four_state,
                literal.signed,
                value.bits.len() > 1,
            );
            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::IntegralLiteral {
                    value,
                    is_unsized: literal.width.is_none(),
                },
            })
        }
        LogicalExprNode::Parenthesized { expr, span } => {
            let expr = bind_logical_node(expr, host)?;
            Ok(BoundLogicalNode {
                ty: expr.ty.clone(),
                span: *span,
                kind: BoundLogicalKind::Parenthesized {
                    expr: Box::new(expr),
                },
            })
        }
        LogicalExprNode::Cast { target, expr, span } => {
            let expr = bind_logical_node(expr, host)?;
            ensure_integral(&expr.ty, expr.span, "cast source")?;
            let ty = cast_target_type(target, &expr.ty, *span)?;
            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::Cast {
                    target: target.clone(),
                    expr: Box::new(expr),
                },
            })
        }
        LogicalExprNode::Selection {
            base,
            selection,
            span,
        } => {
            let base = bind_logical_node(base, host)?;
            ensure_integral(&base.ty, base.span, "selection base")?;
            if base.ty.storage != ExprStorage::PackedVector {
                return Err(sema_diag(
                    "C3-SEMANTIC-SELECTION-BASE",
                    "selection base must be a packed integral value",
                    base.span,
                    &["selection of scalar integer-like or enum-core values is invalid"],
                ));
            }

            let (selection, ty) = match selection {
                SelectionKindAst::Bit { index } => {
                    let index = bind_logical_node(index, host)?;
                    ensure_integral(&index.ty, index.span, "bit-select index")?;
                    (
                        BoundSelection::Bit {
                            index: Box::new(index),
                        },
                        bit_vector_type(1, base.ty.is_four_state, false, false),
                    )
                }
                SelectionKindAst::Part { msb, lsb } => {
                    let msb = bind_logical_node(msb, host)?;
                    let lsb = bind_logical_node(lsb, host)?;
                    let msb_value = eval_const_i64(&msb, "part-select msb", msb.span)?;
                    let lsb_value = eval_const_i64(&lsb, "part-select lsb", lsb.span)?;
                    let width = part_select_width(msb_value, lsb_value, *span)?;
                    (
                        BoundSelection::Part {
                            msb: msb_value,
                            lsb: lsb_value,
                        },
                        bit_vector_type(width as u32, base.ty.is_four_state, false, width > 1),
                    )
                }
                SelectionKindAst::IndexedUp {
                    base: index_base,
                    width,
                } => {
                    let index_base = bind_logical_node(index_base, host)?;
                    ensure_integral(&index_base.ty, index_base.span, "indexed part-select base")?;
                    let width = bind_logical_node(width, host)?;
                    let width_value =
                        eval_const_i64(&width, "indexed part-select width", width.span)?;
                    let width_value = usize::try_from(width_value).map_err(|_| {
                        sema_diag(
                            "C3-SEMANTIC-CONST-RANGE",
                            "indexed part-select width must be positive",
                            width.span,
                            &["width must be a positive constant integer expression"],
                        )
                    })?;
                    if width_value == 0 {
                        return Err(sema_diag(
                            "C3-SEMANTIC-CONST-RANGE",
                            "indexed part-select width must be positive",
                            width.span,
                            &["width must be a positive constant integer expression"],
                        ));
                    }
                    (
                        BoundSelection::IndexedUp {
                            base: Box::new(index_base),
                            width: width_value,
                        },
                        bit_vector_type(
                            width_value as u32,
                            base.ty.is_four_state,
                            false,
                            width_value > 1,
                        ),
                    )
                }
                SelectionKindAst::IndexedDown {
                    base: index_base,
                    width,
                } => {
                    let index_base = bind_logical_node(index_base, host)?;
                    ensure_integral(&index_base.ty, index_base.span, "indexed part-select base")?;
                    let width = bind_logical_node(width, host)?;
                    let width_value =
                        eval_const_i64(&width, "indexed part-select width", width.span)?;
                    let width_value = usize::try_from(width_value).map_err(|_| {
                        sema_diag(
                            "C3-SEMANTIC-CONST-RANGE",
                            "indexed part-select width must be positive",
                            width.span,
                            &["width must be a positive constant integer expression"],
                        )
                    })?;
                    if width_value == 0 {
                        return Err(sema_diag(
                            "C3-SEMANTIC-CONST-RANGE",
                            "indexed part-select width must be positive",
                            width.span,
                            &["width must be a positive constant integer expression"],
                        ));
                    }
                    (
                        BoundSelection::IndexedDown {
                            base: Box::new(index_base),
                            width: width_value,
                        },
                        bit_vector_type(
                            width_value as u32,
                            base.ty.is_four_state,
                            false,
                            width_value > 1,
                        ),
                    )
                }
            };

            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::Selection {
                    base: Box::new(base),
                    selection,
                },
            })
        }
        LogicalExprNode::Unary { op, expr, span } => {
            let expr = bind_logical_node(expr, host)?;
            ensure_integral(&expr.ty, expr.span, "unary operand")?;
            let ty = match op {
                UnaryOpAst::LogicalNot
                | UnaryOpAst::ReduceAnd
                | UnaryOpAst::ReduceNand
                | UnaryOpAst::ReduceOr
                | UnaryOpAst::ReduceNor
                | UnaryOpAst::ReduceXor
                | UnaryOpAst::ReduceXnor => bool_result_type(),
                UnaryOpAst::BitNot | UnaryOpAst::Plus | UnaryOpAst::Minus => {
                    non_enum_integral_type(&expr.ty)
                }
            };
            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::Unary {
                    op: *op,
                    expr: Box::new(expr),
                },
            })
        }
        LogicalExprNode::Binary {
            op,
            left,
            right,
            span,
        } => {
            let left = bind_logical_node(left, host)?;
            let right = bind_logical_node(right, host)?;
            ensure_integral(&left.ty, left.span, "binary lhs")?;
            ensure_integral(&right.ty, right.span, "binary rhs")?;

            let ty = match op {
                BinaryOpAst::LogicalAnd
                | BinaryOpAst::LogicalOr
                | BinaryOpAst::Lt
                | BinaryOpAst::Le
                | BinaryOpAst::Gt
                | BinaryOpAst::Ge
                | BinaryOpAst::Eq
                | BinaryOpAst::Ne
                | BinaryOpAst::CaseEq
                | BinaryOpAst::CaseNe
                | BinaryOpAst::WildEq
                | BinaryOpAst::WildNe => bool_result_type(),
                BinaryOpAst::ShiftLeft
                | BinaryOpAst::ShiftRight
                | BinaryOpAst::ShiftArithLeft
                | BinaryOpAst::ShiftArithRight => non_enum_integral_type(&left.ty),
                BinaryOpAst::BitAnd
                | BinaryOpAst::BitXor
                | BinaryOpAst::BitXnor
                | BinaryOpAst::BitOr
                | BinaryOpAst::Power
                | BinaryOpAst::Multiply
                | BinaryOpAst::Divide
                | BinaryOpAst::Modulo
                | BinaryOpAst::Add
                | BinaryOpAst::Subtract => common_integral_type(&left.ty, &right.ty),
            };

            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::Binary {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            })
        }
        LogicalExprNode::Conditional {
            condition,
            when_true,
            when_false,
            span,
        } => {
            let condition = bind_logical_node(condition, host)?;
            ensure_integral(&condition.ty, condition.span, "conditional condition")?;
            let when_true = bind_logical_node(when_true, host)?;
            let when_false = bind_logical_node(when_false, host)?;
            ensure_integral(&when_true.ty, when_true.span, "conditional true arm")?;
            ensure_integral(&when_false.ty, when_false.span, "conditional false arm")?;

            let ty = if matches!(when_true.ty.kind, ExprTypeKind::EnumCore)
                && matches!(when_false.ty.kind, ExprTypeKind::EnumCore)
                && when_true.ty.enum_type_id.is_some()
                && when_true.ty.enum_type_id == when_false.ty.enum_type_id
            {
                when_true.ty.clone()
            } else {
                common_integral_type(&when_true.ty, &when_false.ty)
            };

            Ok(BoundLogicalNode {
                ty,
                span: *span,
                kind: BoundLogicalKind::Conditional {
                    condition: Box::new(condition),
                    when_true: Box::new(when_true),
                    when_false: Box::new(when_false),
                },
            })
        }
        LogicalExprNode::Inside { expr, set, span } => {
            let expr = bind_logical_node(expr, host)?;
            ensure_integral(&expr.ty, expr.span, "inside lhs")?;

            let mut bound_set = Vec::with_capacity(set.len());
            for item in set {
                match item {
                    InsideItemAst::Expr(value) => {
                        let value = bind_logical_node(value, host)?;
                        ensure_integral(&value.ty, value.span, "inside set item")?;
                        bound_set.push(BoundInsideItem::Expr(value));
                    }
                    InsideItemAst::Range { low, high, .. } => {
                        let low = bind_logical_node(low, host)?;
                        let high = bind_logical_node(high, host)?;
                        ensure_integral(&low.ty, low.span, "inside range low")?;
                        ensure_integral(&high.ty, high.span, "inside range high")?;
                        bound_set.push(BoundInsideItem::Range { low, high });
                    }
                }
            }

            Ok(BoundLogicalNode {
                ty: bool_result_type(),
                span: *span,
                kind: BoundLogicalKind::Inside {
                    expr: Box::new(expr),
                    set: bound_set,
                },
            })
        }
        LogicalExprNode::Concatenation { items, span } => {
            let mut bound_items = Vec::with_capacity(items.len());
            let mut width_sum = 0u32;
            let mut is_four_state = false;
            for item in items {
                let bound = bind_logical_node(item, host)?;
                ensure_integral(&bound.ty, bound.span, "concatenation item")?;
                if matches!(
                    bound.kind,
                    BoundLogicalKind::IntegralLiteral {
                        is_unsized: true,
                        ..
                    }
                ) {
                    return Err(sema_diag(
                        "C3-SEMANTIC-CONCAT-UNSIZED",
                        "unsized constants are not allowed in concatenation",
                        bound.span,
                        &["cast or size constants before concatenating"],
                    ));
                }
                width_sum = width_sum.checked_add(bound.ty.width).ok_or_else(|| {
                    sema_diag(
                        "C3-SEMANTIC-CONST-RANGE",
                        "concatenation width exceeds supported range",
                        bound.span,
                        &["concatenation result width must fit in u32"],
                    )
                })?;
                is_four_state |= bound.ty.is_four_state;
                bound_items.push(bound);
            }
            if width_sum == 0 {
                return Err(sema_diag(
                    "C3-SEMANTIC-CONST-RANGE",
                    "concatenation width must be greater than zero",
                    *span,
                    &["concatenation cannot produce an empty value"],
                ));
            }

            Ok(BoundLogicalNode {
                ty: bit_vector_type(width_sum, is_four_state, false, true),
                span: *span,
                kind: BoundLogicalKind::Concatenation { items: bound_items },
            })
        }
        LogicalExprNode::Replication { count, expr, span } => {
            let count = bind_logical_node(count, host)?;
            let count_value = eval_const_i64(&count, "replication multiplier", count.span)?;
            let count_value = usize::try_from(count_value).map_err(|_| {
                sema_diag(
                    "C3-SEMANTIC-CONST-RANGE",
                    "replication multiplier must be non-negative",
                    count.span,
                    &["replication form is {N{expr}} and N must be >= 0"],
                )
            })?;
            if count_value == 0 {
                return Err(sema_diag(
                    "C3-SEMANTIC-CONST-RANGE",
                    "replication multiplier must be greater than zero",
                    count.span,
                    &["replication with zero multiplier is not supported in C3"],
                ));
            }

            let expr = bind_logical_node(expr, host)?;
            ensure_integral(&expr.ty, expr.span, "replication operand")?;
            let width = expr
                .ty
                .width
                .checked_mul(count_value as u32)
                .ok_or_else(|| {
                    sema_diag(
                        "C3-SEMANTIC-CONST-RANGE",
                        "replication width exceeds supported range",
                        *span,
                        &["replication result width must fit in u32"],
                    )
                })?;
            Ok(BoundLogicalNode {
                ty: bit_vector_type(width, expr.ty.is_four_state, false, true),
                span: *span,
                kind: BoundLogicalKind::Replication {
                    count: count_value,
                    expr: Box::new(expr),
                },
            })
        }
    }
}

fn ensure_integral(ty: &ExprType, span: Span, context: &str) -> Result<(), ExprDiagnostic> {
    if is_integral_type(ty) {
        return Ok(());
    }
    Err(sema_diag(
        "C3-SEMANTIC-EXPECTED-INTEGRAL",
        "integral operand is required in C3",
        span,
        &[context],
    ))
}

fn is_integral_type(ty: &ExprType) -> bool {
    matches!(
        ty.kind,
        ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore
    )
}

fn bool_result_type() -> ExprType {
    bit_vector_type(1, true, false, false)
}

fn bit_vector_type(width: u32, is_four_state: bool, is_signed: bool, packed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if packed {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width: width.max(1),
        is_four_state,
        is_signed,
        enum_type_id: None,
    }
}

fn integer_like_type(kind: IntegerLikeKind) -> ExprType {
    let (width, is_signed, is_four_state) = match kind {
        IntegerLikeKind::Byte => (8, true, false),
        IntegerLikeKind::Shortint => (16, true, false),
        IntegerLikeKind::Int => (32, true, false),
        IntegerLikeKind::Longint => (64, true, false),
        IntegerLikeKind::Integer => (32, true, true),
        IntegerLikeKind::Time => (64, false, true),
    };
    ExprType {
        kind: ExprTypeKind::IntegerLike(kind),
        storage: ExprStorage::Scalar,
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
    }
}

fn cast_target_type(
    target: &CastTargetAst,
    source: &ExprType,
    span: Span,
) -> Result<ExprType, ExprDiagnostic> {
    match target {
        CastTargetAst::Signed => {
            if !is_integral_type(source) {
                return Err(sema_diag(
                    "C3-SEMANTIC-CAST-TARGET",
                    "signed cast requires an integral source",
                    span,
                    &["signed'(expr) is valid only for integral expr"],
                ));
            }
            let mut ty = source.clone();
            ty.is_signed = true;
            Ok(ty)
        }
        CastTargetAst::Unsigned => {
            if !is_integral_type(source) {
                return Err(sema_diag(
                    "C3-SEMANTIC-CAST-TARGET",
                    "unsigned cast requires an integral source",
                    span,
                    &["unsigned'(expr) is valid only for integral expr"],
                ));
            }
            let mut ty = source.clone();
            ty.is_signed = false;
            Ok(ty)
        }
        CastTargetAst::BitVector {
            width,
            is_four_state,
            is_signed,
        } => Ok(bit_vector_type(
            *width,
            *is_four_state,
            *is_signed,
            *width > 1,
        )),
        CastTargetAst::IntegerLike(kind) => Ok(integer_like_type(*kind)),
    }
}

fn non_enum_integral_type(ty: &ExprType) -> ExprType {
    match ty.kind {
        ExprTypeKind::EnumCore => {
            bit_vector_type(ty.width, ty.is_four_state, ty.is_signed, ty.width > 1)
        }
        _ => ty.clone(),
    }
}

fn common_integral_type(left: &ExprType, right: &ExprType) -> ExprType {
    let width = left.width.max(right.width).max(1);
    let is_signed = left.is_signed && right.is_signed;
    let is_four_state = left.is_four_state || right.is_four_state;

    if let (ExprTypeKind::IntegerLike(lhs_kind), ExprTypeKind::IntegerLike(rhs_kind)) =
        (&left.kind, &right.kind)
        && lhs_kind == rhs_kind
    {
        let expected = integer_like_type(*lhs_kind);
        if expected.width == width
            && expected.is_signed == is_signed
            && expected.is_four_state == is_four_state
        {
            return expected;
        }
    }

    bit_vector_type(width, is_four_state, is_signed, width > 1)
}

fn decode_integral_literal(
    literal: &IntegralLiteral,
) -> Result<BoundIntegralValue, ExprDiagnostic> {
    let mut bits = match literal.base {
        IntegralBase::Binary => literal
            .digits
            .chars()
            .map(bit_from_char)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                sema_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "invalid binary integral literal",
                    literal.span,
                    &["binary literals may use 0, 1, x, z"],
                )
            })?,
        IntegralBase::Hex => {
            let mut raw = Vec::new();
            for ch in literal.digits.chars() {
                push_hex_nibble(ch, &mut raw).ok_or_else(|| {
                    sema_diag(
                        "C3-PARSE-LOGICAL-LITERAL",
                        "invalid hexadecimal integral literal",
                        literal.span,
                        &["hex literals may use 0-9, a-f, x, z"],
                    )
                })?;
            }
            raw
        }
        IntegralBase::Decimal => {
            if literal.digits.chars().all(|ch| ch.is_ascii_digit()) {
                let value = literal.digits.parse::<u128>().map_err(|_| {
                    sema_diag(
                        "C3-PARSE-LOGICAL-LITERAL",
                        "invalid decimal integral literal",
                        literal.span,
                        &["decimal literals must fit in the supported integer range"],
                    )
                })?;
                let width = if let Some(width) = literal.width {
                    width
                } else if literal.signed {
                    decimal_signed_width(value)
                } else {
                    bit_length(value)
                };
                unsigned_to_bits(value, width)
            } else {
                return Err(sema_diag(
                    "C3-PARSE-LOGICAL-LITERAL",
                    "decimal literals with x/z digits are not supported in C3",
                    literal.span,
                    &["use based literals for unknown digits"],
                ));
            }
        }
    };

    if bits.is_empty() {
        bits.push(BoundBit::Zero);
    }
    if let Some(width) = literal.width {
        bits = resize_bits(bits, width, literal.signed);
    }

    Ok(BoundIntegralValue {
        bits,
        signed: literal.signed,
    })
}

fn decimal_signed_width(value: u128) -> u32 {
    if value == 0 {
        1
    } else {
        bit_length(value).saturating_add(1)
    }
}

fn bit_length(value: u128) -> u32 {
    if value == 0 {
        1
    } else {
        u128::BITS - value.leading_zeros()
    }
}

fn unsigned_to_bits(value: u128, width: u32) -> Vec<BoundBit> {
    let width = width.max(1);
    let mut bits = Vec::with_capacity(width as usize);
    for shift in (0..width).rev() {
        if shift >= u128::BITS {
            bits.push(BoundBit::Zero);
        } else if (value >> shift) & 1 == 1 {
            bits.push(BoundBit::One);
        } else {
            bits.push(BoundBit::Zero);
        }
    }
    bits
}

fn resize_bits(mut bits: Vec<BoundBit>, width: u32, signed: bool) -> Vec<BoundBit> {
    let target = width.max(1) as usize;
    if bits.len() > target {
        bits = bits[bits.len() - target..].to_vec();
    } else if bits.len() < target {
        let fill = if signed {
            bits.first().copied().unwrap_or(BoundBit::Zero)
        } else {
            BoundBit::Zero
        };
        let mut extended = vec![fill; target - bits.len()];
        extended.extend(bits);
        bits = extended;
    }
    bits
}

fn bit_from_char(ch: char) -> Option<BoundBit> {
    match ch.to_ascii_lowercase() {
        '0' => Some(BoundBit::Zero),
        '1' => Some(BoundBit::One),
        'x' | 'h' | 'u' | 'w' | 'l' | '-' => Some(BoundBit::X),
        'z' => Some(BoundBit::Z),
        _ => None,
    }
}

fn push_hex_nibble(ch: char, out: &mut Vec<BoundBit>) -> Option<()> {
    match ch.to_ascii_lowercase() {
        '0' => out.extend([
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::Zero,
        ]),
        '1' => out.extend([
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::One,
        ]),
        '2' => out.extend([
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::One,
            BoundBit::Zero,
        ]),
        '3' => out.extend([BoundBit::Zero, BoundBit::Zero, BoundBit::One, BoundBit::One]),
        '4' => out.extend([
            BoundBit::Zero,
            BoundBit::One,
            BoundBit::Zero,
            BoundBit::Zero,
        ]),
        '5' => out.extend([BoundBit::Zero, BoundBit::One, BoundBit::Zero, BoundBit::One]),
        '6' => out.extend([BoundBit::Zero, BoundBit::One, BoundBit::One, BoundBit::Zero]),
        '7' => out.extend([BoundBit::Zero, BoundBit::One, BoundBit::One, BoundBit::One]),
        '8' => out.extend([
            BoundBit::One,
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::Zero,
        ]),
        '9' => out.extend([BoundBit::One, BoundBit::Zero, BoundBit::Zero, BoundBit::One]),
        'a' => out.extend([BoundBit::One, BoundBit::Zero, BoundBit::One, BoundBit::Zero]),
        'b' => out.extend([BoundBit::One, BoundBit::Zero, BoundBit::One, BoundBit::One]),
        'c' => out.extend([BoundBit::One, BoundBit::One, BoundBit::Zero, BoundBit::Zero]),
        'd' => out.extend([BoundBit::One, BoundBit::One, BoundBit::Zero, BoundBit::One]),
        'e' => out.extend([BoundBit::One, BoundBit::One, BoundBit::One, BoundBit::Zero]),
        'f' => out.extend([BoundBit::One, BoundBit::One, BoundBit::One, BoundBit::One]),
        'x' | 'h' | 'u' | 'w' | 'l' | '-' => {
            out.extend([BoundBit::X, BoundBit::X, BoundBit::X, BoundBit::X])
        }
        'z' => out.extend([BoundBit::Z, BoundBit::Z, BoundBit::Z, BoundBit::Z]),
        _ => return None,
    }
    Some(())
}

fn part_select_width(msb: i64, lsb: i64, span: Span) -> Result<usize, ExprDiagnostic> {
    let width = if msb >= lsb {
        msb.checked_sub(lsb)
    } else {
        lsb.checked_sub(msb)
    }
    .and_then(|delta| delta.checked_add(1))
    .ok_or_else(|| {
        sema_diag(
            "C3-SEMANTIC-CONST-RANGE",
            "part-select bounds overflow supported range",
            span,
            &["part-select bounds must stay within i64 arithmetic range"],
        )
    })?;

    usize::try_from(width).map_err(|_| {
        sema_diag(
            "C3-SEMANTIC-CONST-RANGE",
            "part-select width exceeds supported range",
            span,
            &["part-select width must fit in usize"],
        )
    })
}

fn eval_const_i64(
    node: &BoundLogicalNode,
    context: &str,
    span: Span,
) -> Result<i64, ExprDiagnostic> {
    let Some(value) = eval_const_node(node)? else {
        return Err(sema_diag(
            "C3-SEMANTIC-CONST-REQUIRED",
            "constant integer expression is required",
            span,
            &[context],
        ));
    };
    if value
        .bits
        .iter()
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return Err(sema_diag(
            "C3-SEMANTIC-CONST-REQUIRED",
            "constant integer expression must not contain x/z",
            span,
            &[context],
        ));
    }
    bits_to_i64(&value.bits, value.signed).ok_or_else(|| {
        sema_diag(
            "C3-SEMANTIC-CONST-RANGE",
            "constant integer expression is out of range",
            span,
            &["constant must fit in signed 64-bit range"],
        )
    })
}

fn eval_const_node(node: &BoundLogicalNode) -> Result<Option<BoundIntegralValue>, ExprDiagnostic> {
    let value = match &node.kind {
        BoundLogicalKind::SignalRef { .. } => return Ok(None),
        BoundLogicalKind::IntegralLiteral { value, .. } => value.clone(),
        BoundLogicalKind::Parenthesized { expr } => {
            let Some(value) = eval_const_node(expr)? else {
                return Ok(None);
            };
            value
        }
        BoundLogicalKind::Cast { target, expr } => {
            let Some(inner) = eval_const_node(expr)? else {
                return Ok(None);
            };
            apply_const_cast(target, inner, &node.ty)
        }
        BoundLogicalKind::Selection { .. } => return Ok(None),
        BoundLogicalKind::Unary { op, expr } => {
            let Some(inner) = eval_const_node(expr)? else {
                return Ok(None);
            };
            eval_const_unary(*op, inner, &node.ty)
        }
        BoundLogicalKind::Binary { op, left, right } => {
            let Some(lhs) = eval_const_node(left)? else {
                return Ok(None);
            };
            let Some(rhs) = eval_const_node(right)? else {
                return Ok(None);
            };
            eval_const_binary(*op, lhs, rhs, &node.ty)
        }
        BoundLogicalKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            let Some(cond) = eval_const_node(condition)? else {
                return Ok(None);
            };
            let Some(lhs) = eval_const_node(when_true)? else {
                return Ok(None);
            };
            let Some(rhs) = eval_const_node(when_false)? else {
                return Ok(None);
            };
            match truthiness_bits(&cond.bits) {
                ConstTruth::One => coerce_const_to_type(lhs, &node.ty),
                ConstTruth::Zero => coerce_const_to_type(rhs, &node.ty),
                ConstTruth::Unknown => {
                    let lhs = coerce_const_to_type(lhs, &node.ty);
                    let rhs = coerce_const_to_type(rhs, &node.ty);
                    let bits = lhs
                        .bits
                        .iter()
                        .zip(rhs.bits.iter())
                        .map(|(a, b)| if a == b { *a } else { BoundBit::X })
                        .collect();
                    BoundIntegralValue {
                        bits,
                        signed: node.ty.is_signed,
                    }
                }
            }
        }
        BoundLogicalKind::Inside { .. } => return Ok(None),
        BoundLogicalKind::Concatenation { items } => {
            let mut bits = Vec::new();
            for item in items {
                let Some(value) = eval_const_node(item)? else {
                    return Ok(None);
                };
                bits.extend(coerce_const_to_type(value, &item.ty).bits);
            }
            BoundIntegralValue {
                bits,
                signed: false,
            }
        }
        BoundLogicalKind::Replication { count, expr } => {
            let Some(value) = eval_const_node(expr)? else {
                return Ok(None);
            };
            let value = coerce_const_to_type(value, &expr.ty);
            let mut bits = Vec::with_capacity(value.bits.len() * *count);
            for _ in 0..*count {
                bits.extend(value.bits.iter().copied());
            }
            BoundIntegralValue {
                bits,
                signed: false,
            }
        }
    };
    Ok(Some(coerce_const_to_type(value, &node.ty)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstTruth {
    Zero,
    One,
    Unknown,
}

fn truthiness_bits(bits: &[BoundBit]) -> ConstTruth {
    let mut unknown = false;
    for bit in bits {
        match bit {
            BoundBit::One => return ConstTruth::One,
            BoundBit::X | BoundBit::Z => unknown = true,
            BoundBit::Zero => {}
        }
    }
    if unknown {
        ConstTruth::Unknown
    } else {
        ConstTruth::Zero
    }
}

fn apply_const_cast(
    target: &CastTargetAst,
    value: BoundIntegralValue,
    result_ty: &ExprType,
) -> BoundIntegralValue {
    match target {
        CastTargetAst::Signed | CastTargetAst::Unsigned => {
            let mut value = value;
            value.signed = result_ty.is_signed;
            value
        }
        CastTargetAst::BitVector { .. } | CastTargetAst::IntegerLike(_) => {
            coerce_const_to_type(value, result_ty)
        }
    }
}

fn eval_const_unary(
    op: UnaryOpAst,
    value: BoundIntegralValue,
    ty: &ExprType,
) -> BoundIntegralValue {
    match op {
        UnaryOpAst::Plus => coerce_const_to_type(value, ty),
        UnaryOpAst::Minus => {
            let value = coerce_const_to_type(value, ty);
            if value
                .bits
                .iter()
                .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
            {
                return BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                };
            }
            if let Some(raw) = bits_to_u128(&value.bits) {
                let modulus = 1_u128.checked_shl(ty.width.min(127)).unwrap_or(0);
                let negated = if modulus == 0 {
                    (!raw).wrapping_add(1)
                } else {
                    modulus.wrapping_sub(raw) & (modulus - 1)
                };
                BoundIntegralValue {
                    bits: unsigned_to_bits(negated, ty.width),
                    signed: ty.is_signed,
                }
            } else {
                BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                }
            }
        }
        UnaryOpAst::LogicalNot => BoundIntegralValue {
            bits: vec![match truthiness_bits(&value.bits) {
                ConstTruth::Zero => BoundBit::One,
                ConstTruth::One => BoundBit::Zero,
                ConstTruth::Unknown => BoundBit::X,
            }],
            signed: false,
        },
        UnaryOpAst::BitNot => BoundIntegralValue {
            bits: coerce_const_to_type(value, ty)
                .bits
                .into_iter()
                .map(|bit| match bit {
                    BoundBit::Zero => BoundBit::One,
                    BoundBit::One => BoundBit::Zero,
                    BoundBit::X | BoundBit::Z => BoundBit::X,
                })
                .collect(),
            signed: ty.is_signed,
        },
        UnaryOpAst::ReduceAnd
        | UnaryOpAst::ReduceNand
        | UnaryOpAst::ReduceOr
        | UnaryOpAst::ReduceNor
        | UnaryOpAst::ReduceXor
        | UnaryOpAst::ReduceXnor => {
            let value = coerce_const_to_type(value, ty);
            let reduced = match op {
                UnaryOpAst::ReduceAnd => reduce_and(&value.bits),
                UnaryOpAst::ReduceNand => invert_reduce(reduce_and(&value.bits)),
                UnaryOpAst::ReduceOr => reduce_or(&value.bits),
                UnaryOpAst::ReduceNor => invert_reduce(reduce_or(&value.bits)),
                UnaryOpAst::ReduceXor => reduce_xor(&value.bits),
                UnaryOpAst::ReduceXnor => invert_reduce(reduce_xor(&value.bits)),
                _ => BoundBit::X,
            };
            BoundIntegralValue {
                bits: vec![reduced],
                signed: false,
            }
        }
    }
}

fn eval_const_binary(
    op: BinaryOpAst,
    left: BoundIntegralValue,
    right: BoundIntegralValue,
    ty: &ExprType,
) -> BoundIntegralValue {
    let left = coerce_const_to_type(left, ty);
    let right = coerce_const_to_type(right, ty);

    match op {
        BinaryOpAst::Add
        | BinaryOpAst::Subtract
        | BinaryOpAst::Multiply
        | BinaryOpAst::Divide
        | BinaryOpAst::Modulo
        | BinaryOpAst::Power
        | BinaryOpAst::ShiftLeft
        | BinaryOpAst::ShiftRight
        | BinaryOpAst::ShiftArithLeft
        | BinaryOpAst::ShiftArithRight
        | BinaryOpAst::BitAnd
        | BinaryOpAst::BitXor
        | BinaryOpAst::BitXnor
        | BinaryOpAst::BitOr => {
            if left
                .bits
                .iter()
                .chain(right.bits.iter())
                .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
            {
                return BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                };
            }
        }
        _ => {}
    }

    match op {
        BinaryOpAst::Add => numeric_binary(&left, &right, ty, |a, b| a.wrapping_add(b)),
        BinaryOpAst::Subtract => numeric_binary(&left, &right, ty, |a, b| a.wrapping_sub(b)),
        BinaryOpAst::Multiply => numeric_binary(&left, &right, ty, |a, b| a.wrapping_mul(b)),
        BinaryOpAst::Divide => {
            if bits_to_u128(&right.bits) == Some(0) {
                BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                }
            } else {
                numeric_binary(&left, &right, ty, |a, b| a / b)
            }
        }
        BinaryOpAst::Modulo => {
            if bits_to_u128(&right.bits) == Some(0) {
                BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                }
            } else {
                numeric_binary(&left, &right, ty, |a, b| a % b)
            }
        }
        BinaryOpAst::Power => {
            let Some(base) = bits_to_u128(&left.bits) else {
                return BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                };
            };
            let Some(exp) = bits_to_u128(&right.bits) else {
                return BoundIntegralValue {
                    bits: vec![BoundBit::X; ty.width as usize],
                    signed: ty.is_signed,
                };
            };
            let mut acc = 1_u128;
            for _ in 0..exp {
                acc = acc.wrapping_mul(base);
            }
            BoundIntegralValue {
                bits: unsigned_to_bits(acc, ty.width),
                signed: ty.is_signed,
            }
        }
        BinaryOpAst::ShiftLeft | BinaryOpAst::ShiftArithLeft => {
            let shift = bits_to_u128(&right.bits).unwrap_or(u128::MAX) as usize;
            if shift >= left.bits.len() {
                BoundIntegralValue {
                    bits: vec![BoundBit::Zero; left.bits.len()],
                    signed: left.signed,
                }
            } else {
                let mut bits = left.bits[shift..].to_vec();
                bits.extend(std::iter::repeat_n(BoundBit::Zero, shift));
                BoundIntegralValue {
                    bits,
                    signed: left.signed,
                }
            }
        }
        BinaryOpAst::ShiftRight => {
            let shift = bits_to_u128(&right.bits).unwrap_or(u128::MAX) as usize;
            if shift >= left.bits.len() {
                BoundIntegralValue {
                    bits: vec![BoundBit::Zero; left.bits.len()],
                    signed: left.signed,
                }
            } else {
                let mut bits = vec![BoundBit::Zero; shift];
                bits.extend(left.bits[..left.bits.len() - shift].iter().copied());
                BoundIntegralValue {
                    bits,
                    signed: left.signed,
                }
            }
        }
        BinaryOpAst::ShiftArithRight => {
            let shift = bits_to_u128(&right.bits).unwrap_or(u128::MAX) as usize;
            let fill = if left.signed {
                left.bits.first().copied().unwrap_or(BoundBit::Zero)
            } else {
                BoundBit::Zero
            };
            if shift >= left.bits.len() {
                BoundIntegralValue {
                    bits: vec![fill; left.bits.len()],
                    signed: left.signed,
                }
            } else {
                let mut bits = vec![fill; shift];
                bits.extend(left.bits[..left.bits.len() - shift].iter().copied());
                BoundIntegralValue {
                    bits,
                    signed: left.signed,
                }
            }
        }
        BinaryOpAst::BitAnd => BoundIntegralValue {
            bits: left
                .bits
                .iter()
                .zip(right.bits.iter())
                .map(|(lhs, rhs)| bitwise_and(*lhs, *rhs))
                .collect(),
            signed: ty.is_signed,
        },
        BinaryOpAst::BitOr => BoundIntegralValue {
            bits: left
                .bits
                .iter()
                .zip(right.bits.iter())
                .map(|(lhs, rhs)| bitwise_or(*lhs, *rhs))
                .collect(),
            signed: ty.is_signed,
        },
        BinaryOpAst::BitXor => BoundIntegralValue {
            bits: left
                .bits
                .iter()
                .zip(right.bits.iter())
                .map(|(lhs, rhs)| bitwise_xor(*lhs, *rhs))
                .collect(),
            signed: ty.is_signed,
        },
        BinaryOpAst::BitXnor => BoundIntegralValue {
            bits: left
                .bits
                .iter()
                .zip(right.bits.iter())
                .map(|(lhs, rhs)| invert_reduce(bitwise_xor(*lhs, *rhs)))
                .collect(),
            signed: ty.is_signed,
        },
        BinaryOpAst::Lt
        | BinaryOpAst::Le
        | BinaryOpAst::Gt
        | BinaryOpAst::Ge
        | BinaryOpAst::Eq
        | BinaryOpAst::Ne
        | BinaryOpAst::CaseEq
        | BinaryOpAst::CaseNe
        | BinaryOpAst::WildEq
        | BinaryOpAst::WildNe
        | BinaryOpAst::LogicalAnd
        | BinaryOpAst::LogicalOr => BoundIntegralValue {
            bits: vec![BoundBit::X],
            signed: false,
        },
    }
}

fn numeric_binary<F>(
    left: &BoundIntegralValue,
    right: &BoundIntegralValue,
    ty: &ExprType,
    op: F,
) -> BoundIntegralValue
where
    F: Fn(u128, u128) -> u128,
{
    let Some(lhs) = bits_to_u128(&left.bits) else {
        return BoundIntegralValue {
            bits: vec![BoundBit::X; ty.width as usize],
            signed: ty.is_signed,
        };
    };
    let Some(rhs) = bits_to_u128(&right.bits) else {
        return BoundIntegralValue {
            bits: vec![BoundBit::X; ty.width as usize],
            signed: ty.is_signed,
        };
    };
    BoundIntegralValue {
        bits: unsigned_to_bits(op(lhs, rhs), ty.width),
        signed: ty.is_signed,
    }
}

fn coerce_const_to_type(value: BoundIntegralValue, ty: &ExprType) -> BoundIntegralValue {
    let mut bits = resize_bits(value.bits, ty.width, value.signed);
    if !ty.is_four_state {
        for bit in &mut bits {
            if matches!(bit, BoundBit::X | BoundBit::Z) {
                *bit = BoundBit::Zero;
            }
        }
    }
    BoundIntegralValue {
        bits,
        signed: ty.is_signed,
    }
}

fn bits_to_u128(bits: &[BoundBit]) -> Option<u128> {
    let mut value = 0u128;
    for bit in bits {
        value <<= 1;
        match bit {
            BoundBit::Zero => {}
            BoundBit::One => value |= 1,
            BoundBit::X | BoundBit::Z => return None,
        }
    }
    Some(value)
}

fn bits_to_i64(bits: &[BoundBit], signed: bool) -> Option<i64> {
    let unsigned = bits_to_u128(bits)?;
    if !signed {
        return i64::try_from(unsigned).ok();
    }
    let width = bits.len().min(64);
    if width == 0 {
        return Some(0);
    }
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    };
    let narrowed = (unsigned as u64) & mask;
    let signed_value = if width == 64 {
        narrowed as i64
    } else {
        let sign_bit = 1_u64 << (width - 1);
        if narrowed & sign_bit == 0 {
            narrowed as i64
        } else {
            let magnitude = ((!narrowed).wrapping_add(1)) & mask;
            -(magnitude as i64)
        }
    };
    Some(signed_value)
}

fn reduce_and(bits: &[BoundBit]) -> BoundBit {
    let mut unknown = false;
    for bit in bits {
        match bit {
            BoundBit::Zero => return BoundBit::Zero,
            BoundBit::One => {}
            BoundBit::X | BoundBit::Z => unknown = true,
        }
    }
    if unknown { BoundBit::X } else { BoundBit::One }
}

fn reduce_or(bits: &[BoundBit]) -> BoundBit {
    let mut unknown = false;
    for bit in bits {
        match bit {
            BoundBit::One => return BoundBit::One,
            BoundBit::Zero => {}
            BoundBit::X | BoundBit::Z => unknown = true,
        }
    }
    if unknown { BoundBit::X } else { BoundBit::Zero }
}

fn reduce_xor(bits: &[BoundBit]) -> BoundBit {
    if bits
        .iter()
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return BoundBit::X;
    }
    let ones = bits.iter().filter(|bit| **bit == BoundBit::One).count();
    if ones % 2 == 0 {
        BoundBit::Zero
    } else {
        BoundBit::One
    }
}

fn invert_reduce(bit: BoundBit) -> BoundBit {
    match bit {
        BoundBit::Zero => BoundBit::One,
        BoundBit::One => BoundBit::Zero,
        BoundBit::X | BoundBit::Z => BoundBit::X,
    }
}

fn bitwise_and(lhs: BoundBit, rhs: BoundBit) -> BoundBit {
    match (lhs, rhs) {
        (BoundBit::Zero, _) | (_, BoundBit::Zero) => BoundBit::Zero,
        (BoundBit::One, BoundBit::One) => BoundBit::One,
        _ => BoundBit::X,
    }
}

fn bitwise_or(lhs: BoundBit, rhs: BoundBit) -> BoundBit {
    match (lhs, rhs) {
        (BoundBit::One, _) | (_, BoundBit::One) => BoundBit::One,
        (BoundBit::Zero, BoundBit::Zero) => BoundBit::Zero,
        _ => BoundBit::X,
    }
}

fn bitwise_xor(lhs: BoundBit, rhs: BoundBit) -> BoundBit {
    match (lhs, rhs) {
        (BoundBit::Zero, BoundBit::Zero) | (BoundBit::One, BoundBit::One) => BoundBit::Zero,
        (BoundBit::Zero, BoundBit::One) | (BoundBit::One, BoundBit::Zero) => BoundBit::One,
        _ => BoundBit::X,
    }
}

fn sema_diag(code: &'static str, message: &str, span: Span, notes: &[&str]) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: DiagnosticLayer::Semantic,
        code,
        message: message.to_string(),
        primary_span: span,
        notes: notes.iter().map(|note| (*note).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::expr::parser::parse_logical_expr_ast;

    #[derive(Default)]
    struct HostStub {
        handles: HashMap<String, SignalHandle>,
    }

    impl HostStub {
        fn with_defaults() -> Self {
            let mut handles = HashMap::new();
            handles.insert("a".to_string(), SignalHandle(1));
            handles.insert("b".to_string(), SignalHandle(2));
            handles.insert("idx".to_string(), SignalHandle(3));
            Self { handles }
        }
    }

    impl ExpressionHost for HostStub {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            self.handles
                .get(name)
                .copied()
                .ok_or_else(|| ExprDiagnostic {
                    layer: DiagnosticLayer::Semantic,
                    code: "HOST-UNKNOWN-SIGNAL",
                    message: format!("unknown signal '{name}'"),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                })
        }

        fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            Ok(bit_vector_type(8, true, false, true))
        }

        fn sample_value(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<crate::expr::SampledValue, ExprDiagnostic> {
            Ok(crate::expr::SampledValue {
                bits: Some("0".to_string()),
            })
        }
    }

    #[test]
    fn binder_rejects_unsized_concat_literal() {
        let ast = parse_logical_expr_ast("{1, a}").expect("parse");
        let host = HostStub::with_defaults();
        let error = bind_logical_expr_ast(&ast, &host).expect_err("bind should fail");

        assert_eq!(error.code, "C3-SEMANTIC-CONCAT-UNSIZED");
    }

    #[test]
    fn binder_rejects_non_constant_replication_multiplier() {
        let ast = parse_logical_expr_ast("{idx{a}}").expect("parse");
        let host = HostStub::with_defaults();
        let error = bind_logical_expr_ast(&ast, &host).expect_err("bind should fail");

        assert_eq!(error.code, "C3-SEMANTIC-CONST-REQUIRED");
    }

    #[test]
    fn binder_preserves_enum_identity_in_conditional_arms() {
        struct EnumHost;
        impl ExpressionHost for EnumHost {
            fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
                match name {
                    "cond" => Ok(SignalHandle(1)),
                    "lhs" => Ok(SignalHandle(2)),
                    "rhs" => Ok(SignalHandle(3)),
                    _ => Err(ExprDiagnostic {
                        layer: DiagnosticLayer::Semantic,
                        code: "HOST-UNKNOWN",
                        message: "unknown".to_string(),
                        primary_span: Span::new(0, 0),
                        notes: vec![],
                    }),
                }
            }

            fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
                if handle == SignalHandle(1) {
                    Ok(bit_vector_type(1, true, false, false))
                } else {
                    Ok(ExprType {
                        kind: ExprTypeKind::EnumCore,
                        storage: ExprStorage::Scalar,
                        width: 2,
                        is_four_state: true,
                        is_signed: false,
                        enum_type_id: Some("fsm_state".to_string()),
                    })
                }
            }

            fn sample_value(
                &self,
                _handle: SignalHandle,
                _timestamp: u64,
            ) -> Result<crate::expr::SampledValue, ExprDiagnostic> {
                Ok(crate::expr::SampledValue {
                    bits: Some("0".to_string()),
                })
            }
        }

        let ast = parse_logical_expr_ast("cond ? lhs : rhs").expect("parse");
        let bound = bind_logical_expr_ast(&ast, &EnumHost).expect("bind");
        assert!(matches!(bound.root.ty.kind, ExprTypeKind::EnumCore));
        assert_eq!(bound.root.ty.enum_type_id.as_deref(), Some("fsm_state"));
    }
}
