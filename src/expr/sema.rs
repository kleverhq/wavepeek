use crate::expr::ast::{
    BasicEventAst, DeferredLogicalExpr, EventExprAst, IntegralBase, IntegralLiteral,
    LogicalBinaryOp, LogicalExprNode,
};
use crate::expr::diagnostic::{DiagnosticLayer, ExprDiagnostic, Span};
use crate::expr::host::{ExprType, ExpressionHost, SignalHandle};
use crate::expr::parser::parse_bounded_logical_expr;

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
pub(crate) struct BoundLogicalExpr {
    pub(crate) root: BoundLogicalNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundLogicalNode {
    SignalRef {
        handle: SignalHandle,
        ty: ExprType,
        span: Span,
    },
    IntegralLiteral {
        value: BoundIntegralValue,
        span: Span,
    },
    UnaryNot {
        expr: Box<BoundLogicalNode>,
        span: Span,
    },
    Binary {
        op: LogicalBinaryOp,
        left: Box<BoundLogicalNode>,
        right: Box<BoundLogicalNode>,
        span: Span,
    },
}

impl BoundLogicalNode {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::SignalRef { span, .. }
            | Self::IntegralLiteral { span, .. }
            | Self::UnaryNot { span, .. }
            | Self::Binary { span, .. } => *span,
        }
    }
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
    bind_event_expr(ast, host)
}

pub(crate) fn bind_event_expr(
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

        let iff = term
            .iff
            .as_ref()
            .map(|expr| bind_logical_expr(expr, host))
            .transpose()?;

        terms.push(BoundEventTerm { event, iff });
    }

    Ok(BoundEventExpr { terms })
}

pub(crate) fn bind_logical_expr(
    expr: &DeferredLogicalExpr,
    host: &dyn ExpressionHost,
) -> Result<BoundLogicalExpr, ExprDiagnostic> {
    let ast = parse_bounded_logical_expr(expr.source.as_str(), expr.span.start)?;
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
                code: "C2-SEMANTIC-UNKNOWN-SIGNAL",
                message: format!("unknown signal '{name}'"),
                primary_span: *span,
                notes: if inner.message.is_empty() {
                    vec![]
                } else {
                    vec![format!("host detail: {}", inner.message)]
                },
            })?;
            let ty = host.signal_type(handle)?;
            Ok(BoundLogicalNode::SignalRef {
                handle,
                ty,
                span: *span,
            })
        }
        LogicalExprNode::IntegralLiteral { literal, span } => {
            let value = decode_integral_literal(literal)?;
            Ok(BoundLogicalNode::IntegralLiteral { value, span: *span })
        }
        LogicalExprNode::UnaryNot { expr, span } => {
            let expr = bind_logical_node(expr, host)?;
            Ok(BoundLogicalNode::UnaryNot {
                expr: Box::new(expr),
                span: *span,
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
            Ok(BoundLogicalNode::Binary {
                op: *op,
                left: Box::new(left),
                right: Box::new(right),
                span: *span,
            })
        }
    }
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
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: "C2-PARSE-LOGICAL-LITERAL",
                message: "invalid binary integral literal".to_string(),
                primary_span: literal.span,
                notes: vec!["binary literals can use 0, 1, x, z".to_string()],
            })?,
        IntegralBase::Hex => {
            let mut raw = Vec::new();
            for ch in literal.digits.chars() {
                push_hex_nibble(ch, &mut raw).ok_or_else(|| ExprDiagnostic {
                    layer: DiagnosticLayer::Parse,
                    code: "C2-PARSE-LOGICAL-LITERAL",
                    message: "invalid hexadecimal integral literal".to_string(),
                    primary_span: literal.span,
                    notes: vec!["hex literals can use 0-9, a-f, x, z".to_string()],
                })?;
            }
            raw
        }
        IntegralBase::Decimal => {
            let value = literal.digits.parse::<u128>().map_err(|_| ExprDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: "C2-PARSE-LOGICAL-LITERAL",
                message: "invalid decimal integral literal".to_string(),
                primary_span: literal.span,
                notes: vec!["decimal literals must fit in the supported integer range".to_string()],
            })?;
            let width = if let Some(width) = literal.width {
                width
            } else if literal.signed {
                decimal_signed_width(value)
            } else {
                bit_length(value)
            };
            unsigned_to_bits(value, width)
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
    let base = bit_length(value);
    if value == 0 {
        1
    } else {
        base.saturating_add(1)
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
        if (value >> shift) & 1 == 1 {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::expr::host::SampledValue;
    use crate::expr::{DiagnosticLayer, parse_event_expr_ast};

    struct HostStub {
        handles: HashMap<String, SignalHandle>,
    }

    impl HostStub {
        fn new() -> Self {
            let mut handles = HashMap::new();
            handles.insert("clk".to_string(), SignalHandle(1));
            handles.insert("rstn".to_string(), SignalHandle(2));
            handles.insert("count".to_string(), SignalHandle(3));
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
            Ok(ExprType {
                width: 8,
                is_four_state: true,
                is_signed: false,
            })
        }

        fn sample_value(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            Ok(SampledValue {
                bits: Some("0".to_string()),
            })
        }
    }

    #[test]
    fn bind_event_expr_resolves_named_terms() {
        let ast = parse_event_expr_ast("posedge clk or rstn").expect("source should parse");
        let host = HostStub::new();

        let bound = bind_event_expr(&ast, &host).expect("binding should succeed");

        assert_eq!(bound.terms.len(), 2);
        assert_eq!(
            bound.terms[0].event,
            BoundEventKind::Posedge(SignalHandle(1))
        );
        assert_eq!(bound.terms[1].event, BoundEventKind::Named(SignalHandle(2)));
    }

    #[test]
    fn bind_logical_expr_rejects_unknown_signal_with_c2_diagnostic() {
        let source = "posedge clk iff missing";
        let ast = parse_event_expr_ast(source).expect("source should parse");
        let host = HostStub::new();
        let logical = ast.terms[0].iff.as_ref().expect("iff payload should exist");

        let diagnostic = bind_logical_expr(logical, &host).expect_err("binding should fail");

        assert_eq!(diagnostic.layer, DiagnosticLayer::Semantic);
        assert_eq!(diagnostic.code, "C2-SEMANTIC-UNKNOWN-SIGNAL");
        assert_eq!(diagnostic.primary_span, Span::new(16, 23));
    }
}
