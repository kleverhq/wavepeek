use crate::error::WavepeekError;
use crate::expr::ast::LogicalBinaryOp;
use crate::expr::diagnostic::ExprDiagnostic;
use crate::expr::host::{EventEvalFrame, ExpressionHost, SignalHandle};
use crate::expr::sema::{
    BoundBit, BoundEventExpr, BoundEventKind, BoundLogicalExpr, BoundLogicalNode,
};

use super::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    Zero,
    One,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeValue {
    bits: Vec<BoundBit>,
    signed: bool,
}

impl RuntimeValue {
    fn truth(value: TruthValue) -> Self {
        let bit = match value {
            TruthValue::Zero => BoundBit::Zero,
            TruthValue::One => BoundBit::One,
            TruthValue::Unknown => BoundBit::X,
        };
        Self {
            bits: vec![bit],
            signed: false,
        }
    }

    fn with_width(mut self, width: usize) -> Self {
        if width == 0 {
            self.bits = vec![BoundBit::Zero];
            return self;
        }

        if self.bits.len() > width {
            self.bits = self.bits[self.bits.len() - width..].to_vec();
        } else if self.bits.len() < width {
            let mut extended = vec![BoundBit::Zero; width - self.bits.len()];
            extended.extend(self.bits);
            self.bits = extended;
        }
        self
    }
}

pub fn event_matches_at(
    expr: &BoundEventExpr,
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, ExprDiagnostic> {
    for term in &expr.terms {
        let event_matches = match term.event {
            BoundEventKind::AnyTracked => any_tracked_matches(host, frame)?,
            BoundEventKind::Named(handle) => named_event_matches(host, handle, frame)?,
            BoundEventKind::Posedge(handle) => edge_event_matches(host, handle, frame)?.0,
            BoundEventKind::Negedge(handle) => edge_event_matches(host, handle, frame)?.1,
            BoundEventKind::Edge(handle) => {
                let (posedge, negedge) = edge_event_matches(host, handle, frame)?;
                posedge || negedge
            }
        };

        if !event_matches {
            continue;
        }

        if let Some(iff) = &term.iff {
            if eval_logical_expr(iff, host, frame.timestamp)? == TruthValue::One {
                return Ok(true);
            }
            continue;
        }

        return Ok(true);
    }

    Ok(false)
}

pub(crate) fn eval_logical_expr(
    expr: &BoundLogicalExpr,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<TruthValue, ExprDiagnostic> {
    let value = eval_logical_node(&expr.root, host, timestamp)?;
    Ok(truthiness(&value))
}

fn eval_logical_node(
    node: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<RuntimeValue, ExprDiagnostic> {
    match node {
        BoundLogicalNode::SignalRef { handle, ty, .. } => {
            let sampled = host.sample_value(*handle, timestamp)?;
            let width = ty.width.max(1) as usize;
            let bits = sampled
                .bits
                .as_deref()
                .map(bits_from_sample)
                .unwrap_or_else(|| vec![BoundBit::X; width]);
            Ok(RuntimeValue {
                bits,
                signed: ty.is_signed,
            }
            .with_width(width))
        }
        BoundLogicalNode::IntegralLiteral { value, .. } => Ok(RuntimeValue {
            bits: value.bits.clone(),
            signed: value.signed,
        }),
        BoundLogicalNode::UnaryNot { expr, .. } => {
            let truth = truthiness(&eval_logical_node(expr, host, timestamp)?);
            Ok(RuntimeValue::truth(match truth {
                TruthValue::Zero => TruthValue::One,
                TruthValue::One => TruthValue::Zero,
                TruthValue::Unknown => TruthValue::Unknown,
            }))
        }
        BoundLogicalNode::Binary {
            op, left, right, ..
        } => eval_binary_node(*op, left, right, host, timestamp),
    }
}

fn eval_binary_node(
    op: LogicalBinaryOp,
    left: &BoundLogicalNode,
    right: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<RuntimeValue, ExprDiagnostic> {
    match op {
        LogicalBinaryOp::AndAnd => {
            let left_truth = truthiness(&eval_logical_node(left, host, timestamp)?);
            let result = match left_truth {
                TruthValue::Zero => TruthValue::Zero,
                TruthValue::One => truthiness(&eval_logical_node(right, host, timestamp)?),
                TruthValue::Unknown => {
                    let right_truth = truthiness(&eval_logical_node(right, host, timestamp)?);
                    if right_truth == TruthValue::Zero {
                        TruthValue::Zero
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            Ok(RuntimeValue::truth(result))
        }
        LogicalBinaryOp::OrOr => {
            let left_truth = truthiness(&eval_logical_node(left, host, timestamp)?);
            let result = match left_truth {
                TruthValue::One => TruthValue::One,
                TruthValue::Zero => truthiness(&eval_logical_node(right, host, timestamp)?),
                TruthValue::Unknown => {
                    let right_truth = truthiness(&eval_logical_node(right, host, timestamp)?);
                    if right_truth == TruthValue::One {
                        TruthValue::One
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            Ok(RuntimeValue::truth(result))
        }
        LogicalBinaryOp::Lt
        | LogicalBinaryOp::Le
        | LogicalBinaryOp::Gt
        | LogicalBinaryOp::Ge
        | LogicalBinaryOp::Eq
        | LogicalBinaryOp::Ne => {
            let left = eval_logical_node(left, host, timestamp)?;
            let right = eval_logical_node(right, host, timestamp)?;
            let truth = compare_runtime_values(op, left, right);
            Ok(RuntimeValue::truth(truth))
        }
    }
}

fn compare_runtime_values(
    op: LogicalBinaryOp,
    left: RuntimeValue,
    right: RuntimeValue,
) -> TruthValue {
    let width = left.bits.len().max(right.bits.len()).max(1);
    let left = extend_runtime_value(left, width);
    let right = extend_runtime_value(right, width);

    if has_unknown_bits(&left.bits) || has_unknown_bits(&right.bits) {
        return TruthValue::Unknown;
    }

    match op {
        LogicalBinaryOp::Eq => {
            if left.bits == right.bits {
                TruthValue::One
            } else {
                TruthValue::Zero
            }
        }
        LogicalBinaryOp::Ne => {
            if left.bits != right.bits {
                TruthValue::One
            } else {
                TruthValue::Zero
            }
        }
        LogicalBinaryOp::Lt | LogicalBinaryOp::Le | LogicalBinaryOp::Gt | LogicalBinaryOp::Ge => {
            let ordering = signed_ordering(left.signed && right.signed, &left.bits, &right.bits);
            let matched = match op {
                LogicalBinaryOp::Lt => ordering.is_lt(),
                LogicalBinaryOp::Le => ordering.is_le(),
                LogicalBinaryOp::Gt => ordering.is_gt(),
                LogicalBinaryOp::Ge => ordering.is_ge(),
                _ => false,
            };
            if matched {
                TruthValue::One
            } else {
                TruthValue::Zero
            }
        }
        LogicalBinaryOp::AndAnd | LogicalBinaryOp::OrOr => TruthValue::Unknown,
    }
}

fn extend_runtime_value(mut value: RuntimeValue, width: usize) -> RuntimeValue {
    if value.bits.len() > width {
        value.bits = value.bits[value.bits.len() - width..].to_vec();
        return value;
    }

    if value.bits.len() < width {
        let fill = if value.signed {
            value.bits.first().copied().unwrap_or(BoundBit::Zero)
        } else {
            BoundBit::Zero
        };
        let mut extended = vec![fill; width - value.bits.len()];
        extended.extend(value.bits);
        value.bits = extended;
    }
    value
}

fn signed_ordering(signed: bool, left: &[BoundBit], right: &[BoundBit]) -> std::cmp::Ordering {
    if !signed {
        return unsigned_ordering(left, right);
    }

    let left_negative = matches!(left.first(), Some(BoundBit::One));
    let right_negative = matches!(right.first(), Some(BoundBit::One));
    match (left_negative, right_negative) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => unsigned_ordering(left, right),
    }
}

fn unsigned_ordering(left: &[BoundBit], right: &[BoundBit]) -> std::cmp::Ordering {
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        match (lhs, rhs) {
            (BoundBit::One, BoundBit::Zero) => return std::cmp::Ordering::Greater,
            (BoundBit::Zero, BoundBit::One) => return std::cmp::Ordering::Less,
            _ => {}
        }
    }
    std::cmp::Ordering::Equal
}

fn any_tracked_matches(
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok(false);
    };

    for &handle in frame.tracked_signals {
        let previous_bits = sample_signal_bits(host, handle, previous_timestamp)?;
        let current_bits = sample_signal_bits(host, handle, frame.timestamp)?;
        if let (Some(previous), Some(current)) = (previous_bits, current_bits)
            && previous != current
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn named_event_matches(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok(false);
    };

    let previous_bits = sample_signal_bits(host, handle, previous_timestamp)?;
    let current_bits = sample_signal_bits(host, handle, frame.timestamp)?;
    Ok(
        matches!((previous_bits, current_bits), (Some(previous), Some(current)) if previous != current),
    )
}

fn edge_event_matches(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    frame: &EventEvalFrame<'_>,
) -> Result<(bool, bool), ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok((false, false));
    };

    let previous_bits = sample_signal_bits(host, handle, previous_timestamp)?;
    let current_bits = sample_signal_bits(host, handle, frame.timestamp)?;
    let (Some(previous_bits), Some(current_bits)) = (previous_bits, current_bits) else {
        return Ok((false, false));
    };

    Ok(classify_edge(previous_bits.as_str(), current_bits.as_str()))
}

fn sample_signal_bits(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    timestamp: u64,
) -> Result<Option<String>, ExprDiagnostic> {
    host.sample_value(handle, timestamp)
        .map(|sample| sample.bits)
}

fn classify_edge(previous_bits: &str, current_bits: &str) -> (bool, bool) {
    let Some(previous_lsb) = previous_bits.chars().last() else {
        return (false, false);
    };
    let Some(current_lsb) = current_bits.chars().last() else {
        return (false, false);
    };

    let previous = normalize_edge_bit(previous_lsb);
    let current = normalize_edge_bit(current_lsb);

    let posedge = matches!(
        (previous, current),
        ('0', '1' | 'x' | 'z') | ('x' | 'z', '1')
    );
    let negedge = matches!(
        (previous, current),
        ('1', '0' | 'x' | 'z') | ('x' | 'z', '0')
    );
    (posedge, negedge)
}

fn normalize_edge_bit(bit: char) -> char {
    match bit.to_ascii_lowercase() {
        '0' => '0',
        '1' => '1',
        'z' => 'z',
        'x' | 'h' | 'u' | 'w' | 'l' | '-' => 'x',
        _ => 'x',
    }
}

fn bits_from_sample(raw: &str) -> Vec<BoundBit> {
    raw.chars()
        .map(|bit| match bit.to_ascii_lowercase() {
            '0' => BoundBit::Zero,
            '1' => BoundBit::One,
            'z' => BoundBit::Z,
            'x' | 'h' | 'u' | 'w' | 'l' | '-' => BoundBit::X,
            _ => BoundBit::X,
        })
        .collect()
}

fn has_unknown_bits(bits: &[BoundBit]) -> bool {
    bits.iter()
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
}

fn truthiness(value: &RuntimeValue) -> TruthValue {
    let mut has_unknown = false;
    for bit in &value.bits {
        match bit {
            BoundBit::One => return TruthValue::One,
            BoundBit::X | BoundBit::Z => has_unknown = true,
            BoundBit::Zero => {}
        }
    }

    if has_unknown {
        TruthValue::Unknown
    } else {
        TruthValue::Zero
    }
}

pub fn eval(_expression: &Expression) -> Result<bool, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "expression evaluation is not implemented yet",
    ))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::expr::ast::LogicalBinaryOp;
    use crate::expr::diagnostic::{DiagnosticLayer, Span};
    use crate::expr::host::{ExprType, SampledValue};
    use crate::expr::sema::{BoundIntegralValue, BoundLogicalExpr, BoundLogicalNode};

    use super::*;

    #[derive(Default)]
    struct HostStub {
        values: HashMap<SignalHandle, String>,
        sampled: RefCell<Vec<SignalHandle>>,
    }

    impl ExpressionHost for HostStub {
        fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            match name {
                "rhs" => Ok(SignalHandle(2)),
                "xv" => Ok(SignalHandle(3)),
                _ => Ok(SignalHandle(1)),
            }
        }

        fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            Ok(ExprType {
                width: 1,
                is_four_state: true,
                is_signed: false,
            })
        }

        fn sample_value(
            &self,
            handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            self.sampled.borrow_mut().push(handle);
            if handle == SignalHandle(2) {
                return Err(ExprDiagnostic {
                    layer: DiagnosticLayer::Runtime,
                    code: "TEST-RHS-SAMPLED",
                    message: "rhs sampled".to_string(),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                });
            }
            Ok(SampledValue {
                bits: Some(
                    self.values
                        .get(&handle)
                        .cloned()
                        .unwrap_or_else(|| "0".to_string()),
                ),
            })
        }
    }

    fn literal(bits: &str) -> BoundLogicalNode {
        BoundLogicalNode::IntegralLiteral {
            value: BoundIntegralValue {
                bits: bits_from_sample(bits),
                signed: false,
            },
            span: Span::new(0, bits.len()),
        }
    }

    #[test]
    fn eval_logical_expr_short_circuits_rhs_sampling() {
        let host = HostStub::default();
        let expr = BoundLogicalExpr {
            root: BoundLogicalNode::Binary {
                op: LogicalBinaryOp::AndAnd,
                left: Box::new(literal("0")),
                right: Box::new(BoundLogicalNode::SignalRef {
                    handle: SignalHandle(2),
                    ty: ExprType {
                        width: 1,
                        is_four_state: true,
                        is_signed: false,
                    },
                    span: Span::new(0, 0),
                }),
                span: Span::new(0, 0),
            },
        };

        let truth = eval_logical_expr(&expr, &host, 10).expect("evaluation should succeed");
        assert_eq!(truth, TruthValue::Zero);
        assert!(host.sampled.borrow().is_empty());
    }

    #[test]
    fn truthiness_handles_unknown_and_known_non_zero() {
        let unknown = RuntimeValue {
            bits: vec![BoundBit::X],
            signed: false,
        };
        let known_non_zero = RuntimeValue {
            bits: vec![BoundBit::One, BoundBit::X],
            signed: false,
        };

        assert_eq!(truthiness(&unknown), TruthValue::Unknown);
        assert_eq!(truthiness(&known_non_zero), TruthValue::One);
    }
}
