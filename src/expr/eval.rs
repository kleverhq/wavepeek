use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::ast::{BinaryOpAst, CastTargetAst, UnaryOpAst};
use crate::expr::diagnostic::ExprDiagnostic;
use crate::expr::host::{EventEvalFrame, ExprType, ExprTypeKind, ExpressionHost, SignalHandle};
use crate::expr::sema::{
    BoundBit, BoundEventExpr, BoundEventKind, BoundInsideItem, BoundLogicalExpr, BoundLogicalKind,
    BoundLogicalNode, BoundSelection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprValue {
    pub ty: ExprType,
    pub bits: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TruthValue {
    Zero,
    One,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeValue {
    ty: ExprType,
    bits: Vec<BoundBit>,
}

#[derive(Default)]
struct EvalCache {
    samples: HashMap<(SignalHandle, u64), Option<Rc<str>>>,
    decoded_samples: HashMap<(SignalHandle, u64), Option<Rc<[BoundBit]>>>,
}

impl EvalCache {
    fn sample_bits(
        &mut self,
        host: &dyn ExpressionHost,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<Option<Rc<str>>, ExprDiagnostic> {
        let key = (handle, timestamp);
        if let Some(value) = self.samples.get(&key) {
            return Ok(value.clone());
        }

        let sampled = host
            .sample_value(handle, timestamp)?
            .bits
            .map(Rc::<str>::from);
        self.samples.insert(key, sampled.clone());
        Ok(sampled)
    }

    fn sample_decoded_bits(
        &mut self,
        host: &dyn ExpressionHost,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<Option<Rc<[BoundBit]>>, ExprDiagnostic> {
        let key = (handle, timestamp);
        if let Some(value) = self.decoded_samples.get(&key) {
            return Ok(value.clone());
        }

        let decoded = self
            .sample_bits(host, handle, timestamp)?
            .as_deref()
            .map(|raw| Rc::<[BoundBit]>::from(bits_from_sample(raw)));
        self.decoded_samples.insert(key, decoded.clone());
        Ok(decoded)
    }
}

pub fn eval_logical_expr_at(
    expr: &BoundLogicalExpr,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<ExprValue, ExprDiagnostic> {
    let mut cache = EvalCache::default();
    let value = eval_node(&expr.root, host, timestamp, &mut cache)?;
    let value = coerce_runtime_to_type(value, &expr.root.ty);
    Ok(ExprValue {
        ty: value.ty,
        bits: bits_to_string(value.bits.as_slice()),
    })
}

pub fn event_matches_at(
    expr: &BoundEventExpr,
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
) -> Result<bool, ExprDiagnostic> {
    let mut cache = EvalCache::default();

    for term in &expr.terms {
        let event_matches = match term.event {
            BoundEventKind::AnyTracked => any_tracked_matches(host, frame, &mut cache)?,
            BoundEventKind::Named(handle) => named_event_matches(host, handle, frame, &mut cache)?,
            BoundEventKind::Posedge(handle) => {
                edge_event_matches(host, handle, frame, &mut cache)?.0
            }
            BoundEventKind::Negedge(handle) => {
                edge_event_matches(host, handle, frame, &mut cache)?.1
            }
            BoundEventKind::Edge(handle) => {
                let (posedge, negedge) = edge_event_matches(host, handle, frame, &mut cache)?;
                posedge || negedge
            }
        };

        if !event_matches {
            continue;
        }

        if let Some(iff) = &term.iff {
            let iff_value = eval_bound_logical_with_cache(iff, host, frame.timestamp, &mut cache)?;
            if truthiness(iff_value.bits.as_slice()) == TruthValue::One {
                return Ok(true);
            }
            continue;
        }

        return Ok(true);
    }

    Ok(false)
}

fn eval_bound_logical_with_cache(
    expr: &BoundLogicalExpr,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let value = eval_node(&expr.root, host, timestamp, cache)?;
    Ok(coerce_runtime_to_type(value, &expr.root.ty))
}

fn eval_node(
    node: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let value = match &node.kind {
        BoundLogicalKind::SignalRef { handle } => {
            let sampled = cache.sample_decoded_bits(host, *handle, timestamp)?;
            let bits = sampled
                .as_deref()
                .map(|value| value.to_vec())
                .unwrap_or_else(|| vec![BoundBit::X; node.ty.width.max(1) as usize]);
            RuntimeValue {
                ty: node.ty.clone(),
                bits,
            }
        }
        BoundLogicalKind::IntegralLiteral { value, .. } => RuntimeValue {
            ty: node.ty.clone(),
            bits: value.bits.clone(),
        },
        BoundLogicalKind::Parenthesized { expr } => eval_node(expr, host, timestamp, cache)?,
        BoundLogicalKind::Cast { target, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            eval_cast(target, value, &node.ty)
        }
        BoundLogicalKind::Selection { base, selection } => {
            let base_value = eval_node(base, host, timestamp, cache)?;
            eval_selection(base_value, selection, host, timestamp, cache, &node.ty)?
        }
        BoundLogicalKind::Unary { op, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            eval_unary(*op, value, &node.ty)
        }
        BoundLogicalKind::Binary { op, left, right } => {
            eval_binary(*op, left, right, host, timestamp, cache, &node.ty)?
        }
        BoundLogicalKind::Conditional {
            condition,
            when_true,
            when_false,
        } => eval_conditional(
            condition, when_true, when_false, host, timestamp, cache, &node.ty,
        )?,
        BoundLogicalKind::Inside { expr, set } => {
            eval_inside(expr, set.as_slice(), host, timestamp, cache, &node.ty)?
        }
        BoundLogicalKind::Concatenation { items } => {
            let mut bits = Vec::new();
            for item in items {
                let item_value = eval_node(item, host, timestamp, cache)?;
                let item_value = coerce_runtime_to_type(item_value, &item.ty);
                bits.extend(item_value.bits);
            }
            RuntimeValue {
                ty: node.ty.clone(),
                bits,
            }
        }
        BoundLogicalKind::Replication { count, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            let value = coerce_runtime_to_type(value, &expr.ty);
            let mut bits = Vec::with_capacity(value.bits.len() * *count);
            for _ in 0..*count {
                bits.extend(value.bits.iter().copied());
            }
            RuntimeValue {
                ty: node.ty.clone(),
                bits,
            }
        }
    };

    Ok(coerce_runtime_to_type(value, &node.ty))
}

fn eval_cast(target: &CastTargetAst, value: RuntimeValue, result_ty: &ExprType) -> RuntimeValue {
    match target {
        CastTargetAst::Signed | CastTargetAst::Unsigned => {
            let mut coerced = coerce_runtime_to_type(value, result_ty);
            coerced.ty.is_signed = result_ty.is_signed;
            coerced
        }
        CastTargetAst::BitVector { .. } | CastTargetAst::IntegerLike(_) => {
            coerce_runtime_to_type(value, result_ty)
        }
    }
}

fn eval_selection(
    base: RuntimeValue,
    selection: &BoundSelection,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let base_ty = base.ty.clone();
    let base = coerce_runtime_to_type(base, &base_ty);
    let width = base.bits.len();
    let result = match selection {
        BoundSelection::Bit { index } => {
            let index = eval_node(index, host, timestamp, cache)?;
            let index_ty = index.ty.clone();
            let index = coerce_runtime_to_type(index, &index_ty);
            if let Some(bit_index) = runtime_index_to_usize(index.bits.as_slice()) {
                if bit_index < width {
                    RuntimeValue {
                        ty: result_ty.clone(),
                        bits: vec![base.bits[width - 1 - bit_index]],
                    }
                } else {
                    RuntimeValue {
                        ty: result_ty.clone(),
                        bits: vec![if base.ty.is_four_state {
                            BoundBit::X
                        } else {
                            BoundBit::Zero
                        }],
                    }
                }
            } else {
                RuntimeValue {
                    ty: result_ty.clone(),
                    bits: vec![if base.ty.is_four_state {
                        BoundBit::X
                    } else {
                        BoundBit::Zero
                    }],
                }
            }
        }
        BoundSelection::Part { msb, lsb } => {
            let bits = part_select_bits(base.bits.as_slice(), *msb, *lsb, base.ty.is_four_state);
            RuntimeValue {
                ty: result_ty.clone(),
                bits,
            }
        }
        BoundSelection::IndexedUp {
            base: index_base,
            width,
        } => {
            let index_base = eval_node(index_base, host, timestamp, cache)?;
            let index_base_ty = index_base.ty.clone();
            let index_base = coerce_runtime_to_type(index_base, &index_base_ty);
            let bits = if let Some(start) = runtime_index_to_i64(index_base.bits.as_slice()) {
                indexed_part_select_bits(
                    base.bits.as_slice(),
                    start,
                    *width,
                    true,
                    base.ty.is_four_state,
                )
            } else {
                vec![BoundBit::X; *width]
            };
            RuntimeValue {
                ty: result_ty.clone(),
                bits,
            }
        }
        BoundSelection::IndexedDown {
            base: index_base,
            width,
        } => {
            let index_base = eval_node(index_base, host, timestamp, cache)?;
            let index_base_ty = index_base.ty.clone();
            let index_base = coerce_runtime_to_type(index_base, &index_base_ty);
            let bits = if let Some(start) = runtime_index_to_i64(index_base.bits.as_slice()) {
                indexed_part_select_bits(
                    base.bits.as_slice(),
                    start,
                    *width,
                    false,
                    base.ty.is_four_state,
                )
            } else {
                vec![BoundBit::X; *width]
            };
            RuntimeValue {
                ty: result_ty.clone(),
                bits,
            }
        }
    };
    Ok(result)
}

fn eval_unary(op: UnaryOpAst, value: RuntimeValue, result_ty: &ExprType) -> RuntimeValue {
    let value_ty = value.ty.clone();
    let value = coerce_runtime_to_type(value, &value_ty);
    match op {
        UnaryOpAst::Plus => coerce_runtime_to_type(value, result_ty),
        UnaryOpAst::Minus => {
            let value = coerce_runtime_to_type(value, result_ty);
            if value
                .bits
                .iter()
                .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
            {
                return all_x(result_ty);
            }
            match bits_to_u128(value.bits.as_slice()) {
                Some(raw) => {
                    let width = result_ty.width.min(128);
                    let modulus = 1_u128.checked_shl(width).unwrap_or(0);
                    let negated = if modulus == 0 {
                        (!raw).wrapping_add(1)
                    } else {
                        modulus.wrapping_sub(raw) & (modulus - 1)
                    };
                    coerce_runtime_to_type(
                        RuntimeValue {
                            ty: result_ty.clone(),
                            bits: unsigned_to_bits(negated, result_ty.width),
                        },
                        result_ty,
                    )
                }
                None => all_x(result_ty),
            }
        }
        UnaryOpAst::LogicalNot => RuntimeValue {
            ty: result_ty.clone(),
            bits: vec![match truthiness(value.bits.as_slice()) {
                TruthValue::Zero => BoundBit::One,
                TruthValue::One => BoundBit::Zero,
                TruthValue::Unknown => BoundBit::X,
            }],
        },
        UnaryOpAst::BitNot => {
            let value = coerce_runtime_to_type(value, result_ty);
            RuntimeValue {
                ty: result_ty.clone(),
                bits: value
                    .bits
                    .iter()
                    .map(|bit| match bit {
                        BoundBit::Zero => BoundBit::One,
                        BoundBit::One => BoundBit::Zero,
                        BoundBit::X | BoundBit::Z => BoundBit::X,
                    })
                    .collect(),
            }
        }
        UnaryOpAst::ReduceAnd
        | UnaryOpAst::ReduceNand
        | UnaryOpAst::ReduceOr
        | UnaryOpAst::ReduceNor
        | UnaryOpAst::ReduceXor
        | UnaryOpAst::ReduceXnor => {
            let value_ty = value.ty.clone();
            let value = coerce_runtime_to_type(value, &value_ty);
            let reduced = match op {
                UnaryOpAst::ReduceAnd => reduce_and(value.bits.as_slice()),
                UnaryOpAst::ReduceNand => invert_bit(reduce_and(value.bits.as_slice())),
                UnaryOpAst::ReduceOr => reduce_or(value.bits.as_slice()),
                UnaryOpAst::ReduceNor => invert_bit(reduce_or(value.bits.as_slice())),
                UnaryOpAst::ReduceXor => reduce_xor(value.bits.as_slice()),
                UnaryOpAst::ReduceXnor => invert_bit(reduce_xor(value.bits.as_slice())),
                _ => BoundBit::X,
            };
            RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![reduced],
            }
        }
    }
}

fn eval_binary(
    op: BinaryOpAst,
    left: &BoundLogicalNode,
    right: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    match op {
        BinaryOpAst::LogicalAnd => {
            let lhs = eval_node(left, host, timestamp, cache)?;
            let lhs_truth = truthiness(lhs.bits.as_slice());
            let truth = match lhs_truth {
                TruthValue::Zero => TruthValue::Zero,
                TruthValue::One => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    truthiness(rhs.bits.as_slice())
                }
                TruthValue::Unknown => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    let rhs_truth = truthiness(rhs.bits.as_slice());
                    if rhs_truth == TruthValue::Zero {
                        TruthValue::Zero
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            return Ok(RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![truth_to_bit(truth)],
            });
        }
        BinaryOpAst::LogicalOr => {
            let lhs = eval_node(left, host, timestamp, cache)?;
            let lhs_truth = truthiness(lhs.bits.as_slice());
            let truth = match lhs_truth {
                TruthValue::One => TruthValue::One,
                TruthValue::Zero => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    truthiness(rhs.bits.as_slice())
                }
                TruthValue::Unknown => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    let rhs_truth = truthiness(rhs.bits.as_slice());
                    if rhs_truth == TruthValue::One {
                        TruthValue::One
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            return Ok(RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![truth_to_bit(truth)],
            });
        }
        _ => {}
    }

    let lhs = eval_node(left, host, timestamp, cache)?;
    let rhs = eval_node(right, host, timestamp, cache)?;

    let out = match op {
        BinaryOpAst::Lt
        | BinaryOpAst::Le
        | BinaryOpAst::Gt
        | BinaryOpAst::Ge
        | BinaryOpAst::Eq
        | BinaryOpAst::Ne => {
            let truth = compare_unknown_sensitive(op, lhs, rhs);
            RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![truth_to_bit(truth)],
            }
        }
        BinaryOpAst::CaseEq | BinaryOpAst::CaseNe => {
            let truth = compare_case(op, lhs, rhs);
            RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![truth_to_bit(truth)],
            }
        }
        BinaryOpAst::WildEq | BinaryOpAst::WildNe => {
            let truth = compare_wild(op, lhs, rhs);
            RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![truth_to_bit(truth)],
            }
        }
        BinaryOpAst::BitAnd | BinaryOpAst::BitOr | BinaryOpAst::BitXor | BinaryOpAst::BitXnor => {
            eval_bitwise(op, lhs, rhs, result_ty)
        }
        BinaryOpAst::ShiftLeft
        | BinaryOpAst::ShiftRight
        | BinaryOpAst::ShiftArithLeft
        | BinaryOpAst::ShiftArithRight => eval_shift(op, lhs, rhs, result_ty),
        BinaryOpAst::Add
        | BinaryOpAst::Subtract
        | BinaryOpAst::Multiply
        | BinaryOpAst::Divide
        | BinaryOpAst::Modulo
        | BinaryOpAst::Power => eval_arithmetic(op, lhs, rhs, result_ty),
        BinaryOpAst::LogicalAnd | BinaryOpAst::LogicalOr => unreachable!(),
    };

    Ok(out)
}

fn eval_conditional(
    condition: &BoundLogicalNode,
    when_true: &BoundLogicalNode,
    when_false: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let cond = eval_node(condition, host, timestamp, cache)?;
    let cond_truth = truthiness(cond.bits.as_slice());

    match cond_truth {
        TruthValue::One => {
            let value = eval_node(when_true, host, timestamp, cache)?;
            Ok(coerce_runtime_to_type(value, result_ty))
        }
        TruthValue::Zero => {
            let value = eval_node(when_false, host, timestamp, cache)?;
            Ok(coerce_runtime_to_type(value, result_ty))
        }
        TruthValue::Unknown => {
            let lhs = eval_node(when_true, host, timestamp, cache)?;
            let rhs = eval_node(when_false, host, timestamp, cache)?;
            let lhs = coerce_runtime_to_type(lhs, result_ty);
            let rhs = coerce_runtime_to_type(rhs, result_ty);
            let bits = lhs
                .bits
                .iter()
                .zip(rhs.bits.iter())
                .map(|(lhs, rhs)| if lhs == rhs { *lhs } else { BoundBit::X })
                .collect::<Vec<_>>();
            Ok(coerce_runtime_to_type(
                RuntimeValue {
                    ty: result_ty.clone(),
                    bits,
                },
                result_ty,
            ))
        }
    }
}

fn eval_inside(
    expr: &BoundLogicalNode,
    set: &[BoundInsideItem],
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let lhs = eval_node(expr, host, timestamp, cache)?;

    let mut saw_unknown = false;
    for item in set {
        let item_truth = match item {
            BoundInsideItem::Expr(value) => {
                let rhs = eval_node(value, host, timestamp, cache)?;
                compare_wild(BinaryOpAst::WildEq, lhs.clone(), rhs)
            }
            BoundInsideItem::Range { low, high } => {
                let low = eval_node(low, host, timestamp, cache)?;
                let high = eval_node(high, host, timestamp, cache)?;
                let ge_low = compare_unknown_sensitive(BinaryOpAst::Ge, lhs.clone(), low);
                let le_high = compare_unknown_sensitive(BinaryOpAst::Le, lhs.clone(), high);
                match (ge_low, le_high) {
                    (TruthValue::One, TruthValue::One) => TruthValue::One,
                    (TruthValue::Zero, _) | (_, TruthValue::Zero) => TruthValue::Zero,
                    _ => TruthValue::Unknown,
                }
            }
        };

        match item_truth {
            TruthValue::One => {
                return Ok(RuntimeValue {
                    ty: result_ty.clone(),
                    bits: vec![BoundBit::One],
                });
            }
            TruthValue::Unknown => saw_unknown = true,
            TruthValue::Zero => {}
        }
    }

    let bit = if saw_unknown {
        BoundBit::X
    } else {
        BoundBit::Zero
    };
    Ok(RuntimeValue {
        ty: result_ty.clone(),
        bits: vec![bit],
    })
}

fn eval_bitwise(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> RuntimeValue {
    let left = coerce_runtime_to_type(left, result_ty);
    let right = coerce_runtime_to_type(right, result_ty);

    let bits = left
        .bits
        .iter()
        .zip(right.bits.iter())
        .map(|(lhs, rhs)| match op {
            BinaryOpAst::BitAnd => bitwise_and(*lhs, *rhs),
            BinaryOpAst::BitOr => bitwise_or(*lhs, *rhs),
            BinaryOpAst::BitXor => bitwise_xor(*lhs, *rhs),
            BinaryOpAst::BitXnor => invert_bit(bitwise_xor(*lhs, *rhs)),
            _ => BoundBit::X,
        })
        .collect();

    RuntimeValue {
        ty: result_ty.clone(),
        bits,
    }
}

fn eval_shift(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> RuntimeValue {
    let left = coerce_runtime_to_type(left, result_ty);
    let right_ty = right.ty.clone();
    let right = coerce_runtime_to_type(right, &right_ty);
    if right
        .bits
        .iter()
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return all_x(result_ty);
    }

    let shift = bits_to_u128(right.bits.as_slice())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    if shift >= left.bits.len() {
        return match op {
            BinaryOpAst::ShiftArithRight if result_ty.is_signed => RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![left.bits.first().copied().unwrap_or(BoundBit::Zero); left.bits.len()],
            },
            _ => RuntimeValue {
                ty: result_ty.clone(),
                bits: vec![BoundBit::Zero; left.bits.len()],
            },
        };
    }

    let bits = match op {
        BinaryOpAst::ShiftLeft | BinaryOpAst::ShiftArithLeft => {
            let mut bits = left.bits[shift..].to_vec();
            bits.extend(std::iter::repeat_n(BoundBit::Zero, shift));
            bits
        }
        BinaryOpAst::ShiftRight => {
            let mut bits = vec![BoundBit::Zero; shift];
            bits.extend(left.bits[..left.bits.len() - shift].iter().copied());
            bits
        }
        BinaryOpAst::ShiftArithRight => {
            let fill = if result_ty.is_signed {
                left.bits.first().copied().unwrap_or(BoundBit::Zero)
            } else {
                BoundBit::Zero
            };
            let mut bits = vec![fill; shift];
            bits.extend(left.bits[..left.bits.len() - shift].iter().copied());
            bits
        }
        _ => vec![BoundBit::X; left.bits.len()],
    };

    RuntimeValue {
        ty: result_ty.clone(),
        bits,
    }
}

fn eval_arithmetic(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> RuntimeValue {
    let left = coerce_runtime_to_type(left, result_ty);
    let exponent_signed = right.ty.is_signed;
    let right_ty = right.ty.clone();
    let right = if op == BinaryOpAst::Power {
        coerce_runtime_to_type(right, &right_ty)
    } else {
        coerce_runtime_to_type(right, result_ty)
    };

    if left
        .bits
        .iter()
        .chain(right.bits.iter())
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return all_x(result_ty);
    }

    let Some(lhs_unsigned) = bits_to_u128(left.bits.as_slice()) else {
        return all_x(result_ty);
    };
    let Some(rhs_unsigned) = bits_to_u128(right.bits.as_slice()) else {
        return all_x(result_ty);
    };

    let bits = match op {
        BinaryOpAst::Add => {
            unsigned_to_bits(lhs_unsigned.wrapping_add(rhs_unsigned), result_ty.width)
        }
        BinaryOpAst::Subtract => {
            unsigned_to_bits(lhs_unsigned.wrapping_sub(rhs_unsigned), result_ty.width)
        }
        BinaryOpAst::Multiply => {
            unsigned_to_bits(lhs_unsigned.wrapping_mul(rhs_unsigned), result_ty.width)
        }
        BinaryOpAst::Divide => {
            if rhs_unsigned == 0 {
                return all_x(result_ty);
            }
            if result_ty.is_signed {
                let Some(lhs_signed) = bits_to_i128(left.bits.as_slice(), true) else {
                    return all_x(result_ty);
                };
                let Some(rhs_signed) = bits_to_i128(right.bits.as_slice(), true) else {
                    return all_x(result_ty);
                };
                if rhs_signed == 0 {
                    return all_x(result_ty);
                }
                signed_to_bits(lhs_signed.wrapping_div(rhs_signed), result_ty.width)
            } else {
                unsigned_to_bits(lhs_unsigned / rhs_unsigned, result_ty.width)
            }
        }
        BinaryOpAst::Modulo => {
            if rhs_unsigned == 0 {
                return all_x(result_ty);
            }
            if result_ty.is_signed {
                let Some(lhs_signed) = bits_to_i128(left.bits.as_slice(), true) else {
                    return all_x(result_ty);
                };
                let Some(rhs_signed) = bits_to_i128(right.bits.as_slice(), true) else {
                    return all_x(result_ty);
                };
                if rhs_signed == 0 {
                    return all_x(result_ty);
                }
                signed_to_bits(lhs_signed.wrapping_rem(rhs_signed), result_ty.width)
            } else {
                unsigned_to_bits(lhs_unsigned % rhs_unsigned, result_ty.width)
            }
        }
        BinaryOpAst::Power => {
            if exponent_signed {
                let Some(exp_signed) = bits_to_i128(right.bits.as_slice(), true) else {
                    return all_x(result_ty);
                };
                if exp_signed < 0 {
                    if lhs_unsigned == 0 {
                        return all_x(result_ty);
                    }
                    vec![BoundBit::Zero; result_ty.width.max(1) as usize]
                } else {
                    unsigned_to_bits(
                        pow_wrapping_u128(lhs_unsigned, exp_signed as u128),
                        result_ty.width,
                    )
                }
            } else {
                unsigned_to_bits(
                    pow_wrapping_u128(lhs_unsigned, rhs_unsigned),
                    result_ty.width,
                )
            }
        }
        _ => return all_x(result_ty),
    };

    RuntimeValue {
        ty: result_ty.clone(),
        bits,
    }
}

fn compare_unknown_sensitive(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
) -> TruthValue {
    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common);
    let right = coerce_runtime_to_type(right, &common);

    if left
        .bits
        .iter()
        .chain(right.bits.iter())
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return TruthValue::Unknown;
    }

    let ord = compare_ordering(&left, &right);
    match op {
        BinaryOpAst::Lt => ordering_truth(ord.is_lt()),
        BinaryOpAst::Le => ordering_truth(ord.is_le()),
        BinaryOpAst::Gt => ordering_truth(ord.is_gt()),
        BinaryOpAst::Ge => ordering_truth(ord.is_ge()),
        BinaryOpAst::Eq => ordering_truth(ord.is_eq()),
        BinaryOpAst::Ne => ordering_truth(!ord.is_eq()),
        _ => TruthValue::Unknown,
    }
}

fn compare_case(op: BinaryOpAst, left: RuntimeValue, right: RuntimeValue) -> TruthValue {
    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common);
    let right = coerce_runtime_to_type(right, &common);
    let equal = left.bits == right.bits;
    match op {
        BinaryOpAst::CaseEq => ordering_truth(equal),
        BinaryOpAst::CaseNe => ordering_truth(!equal),
        _ => TruthValue::Unknown,
    }
}

fn compare_wild(op: BinaryOpAst, left: RuntimeValue, right: RuntimeValue) -> TruthValue {
    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common);
    let right = coerce_runtime_to_type(right, &common);

    let mut saw_unknown = false;
    for (lhs, rhs) in left.bits.iter().zip(right.bits.iter()) {
        if matches!(rhs, BoundBit::X | BoundBit::Z) {
            continue;
        }
        if matches!(lhs, BoundBit::X | BoundBit::Z) {
            saw_unknown = true;
            continue;
        }
        if lhs != rhs {
            return match op {
                BinaryOpAst::WildEq => TruthValue::Zero,
                BinaryOpAst::WildNe => TruthValue::One,
                _ => TruthValue::Unknown,
            };
        }
    }

    let eq = if saw_unknown {
        TruthValue::Unknown
    } else {
        TruthValue::One
    };
    match op {
        BinaryOpAst::WildEq => eq,
        BinaryOpAst::WildNe => match eq {
            TruthValue::Zero => TruthValue::One,
            TruthValue::One => TruthValue::Zero,
            TruthValue::Unknown => TruthValue::Unknown,
        },
        _ => TruthValue::Unknown,
    }
}

fn compare_ordering(left: &RuntimeValue, right: &RuntimeValue) -> Ordering {
    let signed = left.ty.is_signed && right.ty.is_signed;
    if !signed {
        return compare_unsigned(left.bits.as_slice(), right.bits.as_slice());
    }

    let left_negative = left.bits.first().copied() == Some(BoundBit::One);
    let right_negative = right.bits.first().copied() == Some(BoundBit::One);
    match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => compare_unsigned(left.bits.as_slice(), right.bits.as_slice()),
    }
}

fn compare_unsigned(left: &[BoundBit], right: &[BoundBit]) -> Ordering {
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        match (*lhs, *rhs) {
            (BoundBit::One, BoundBit::Zero) => return Ordering::Greater,
            (BoundBit::Zero, BoundBit::One) => return Ordering::Less,
            _ => {}
        }
    }
    Ordering::Equal
}

fn ordering_truth(value: bool) -> TruthValue {
    if value {
        TruthValue::One
    } else {
        TruthValue::Zero
    }
}

fn coerce_runtime_to_type(value: RuntimeValue, ty: &ExprType) -> RuntimeValue {
    let mut bits = resize_bits(value.bits, ty.width, value.ty.is_signed);
    if !ty.is_four_state {
        for bit in &mut bits {
            if matches!(bit, BoundBit::X | BoundBit::Z) {
                *bit = BoundBit::Zero;
            }
        }
    }
    RuntimeValue {
        ty: ty.clone(),
        bits,
    }
}

fn common_integral_type_for_eval(left: &ExprType, right: &ExprType) -> ExprType {
    let width = left.width.max(right.width).max(1);
    let is_signed = left.is_signed && right.is_signed;
    let is_four_state = left.is_four_state || right.is_four_state;
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            crate::expr::ExprStorage::PackedVector
        } else {
            crate::expr::ExprStorage::Scalar
        },
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
    }
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

fn runtime_index_to_usize(bits: &[BoundBit]) -> Option<usize> {
    bits_to_u128(bits).and_then(|value| usize::try_from(value).ok())
}

fn runtime_index_to_i64(bits: &[BoundBit]) -> Option<i64> {
    bits_to_u128(bits).and_then(|value| i64::try_from(value).ok())
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

fn bits_to_i128(bits: &[BoundBit], signed: bool) -> Option<i128> {
    let unsigned = bits_to_u128(bits)?;
    if !signed {
        return i128::try_from(unsigned).ok();
    }
    let width = bits.len().clamp(1, 128);
    if width == 128 {
        return Some(unsigned as i128);
    }
    let mask = (1_u128 << width) - 1;
    let narrowed = unsigned & mask;
    let sign_bit = 1_u128 << (width - 1);
    if narrowed & sign_bit == 0 {
        Some(narrowed as i128)
    } else {
        let magnitude = ((!narrowed).wrapping_add(1)) & mask;
        Some(-(magnitude as i128))
    }
}

fn signed_to_bits(value: i128, width: u32) -> Vec<BoundBit> {
    unsigned_to_bits(value as u128, width)
}

fn pow_wrapping_u128(mut base: u128, mut exp: u128) -> u128 {
    let mut acc = 1u128;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = acc.wrapping_mul(base);
        }
        exp >>= 1;
        if exp > 0 {
            base = base.wrapping_mul(base);
        }
    }
    acc
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

fn all_x(ty: &ExprType) -> RuntimeValue {
    RuntimeValue {
        ty: ty.clone(),
        bits: vec![BoundBit::X; ty.width.max(1) as usize],
    }
}

fn part_select_bits(base: &[BoundBit], msb: i64, lsb: i64, base_four_state: bool) -> Vec<BoundBit> {
    let width = msb.abs_diff(lsb) as usize + 1;
    let mut result = Vec::with_capacity(width);
    if msb >= lsb {
        for idx in (lsb..=msb).rev() {
            result.push(select_bit(base, idx, base_four_state, true));
        }
    } else {
        for idx in msb..=lsb {
            result.push(select_bit(base, idx, base_four_state, true));
        }
    }
    result
}

fn indexed_part_select_bits(
    base: &[BoundBit],
    start: i64,
    width: usize,
    up: bool,
    base_four_state: bool,
) -> Vec<BoundBit> {
    let mut result = Vec::with_capacity(width);
    for offset in 0..width {
        let idx = if up {
            start.saturating_add(offset as i64)
        } else {
            start.saturating_sub(offset as i64)
        };
        result.push(select_bit(base, idx, base_four_state, true));
    }
    result.reverse();
    result
}

fn select_bit(
    base: &[BoundBit],
    index: i64,
    base_four_state: bool,
    x_for_out_of_range: bool,
) -> BoundBit {
    if index < 0 {
        return if x_for_out_of_range || base_four_state {
            BoundBit::X
        } else {
            BoundBit::Zero
        };
    }
    let index = index as usize;
    if index >= base.len() {
        return if x_for_out_of_range || base_four_state {
            BoundBit::X
        } else {
            BoundBit::Zero
        };
    }
    base[base.len() - 1 - index]
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

fn invert_bit(bit: BoundBit) -> BoundBit {
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

fn truth_to_bit(truth: TruthValue) -> BoundBit {
    match truth {
        TruthValue::Zero => BoundBit::Zero,
        TruthValue::One => BoundBit::One,
        TruthValue::Unknown => BoundBit::X,
    }
}

fn truthiness(bits: &[BoundBit]) -> TruthValue {
    let mut has_unknown = false;
    for bit in bits {
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

fn any_tracked_matches(
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
    cache: &mut EvalCache,
) -> Result<bool, ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok(false);
    };

    for &handle in frame.tracked_signals {
        let previous_bits = sample_signal_bits(host, handle, previous_timestamp, cache)?;
        let current_bits = sample_signal_bits(host, handle, frame.timestamp, cache)?;
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
    cache: &mut EvalCache,
) -> Result<bool, ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok(false);
    };

    let previous_bits = sample_signal_bits(host, handle, previous_timestamp, cache)?;
    let current_bits = sample_signal_bits(host, handle, frame.timestamp, cache)?;
    Ok(
        matches!((previous_bits, current_bits), (Some(previous), Some(current)) if previous != current),
    )
}

fn edge_event_matches(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    frame: &EventEvalFrame<'_>,
    cache: &mut EvalCache,
) -> Result<(bool, bool), ExprDiagnostic> {
    let Some(previous_timestamp) = frame.previous_timestamp else {
        return Ok((false, false));
    };

    let previous_bits = sample_signal_bits(host, handle, previous_timestamp, cache)?;
    let current_bits = sample_signal_bits(host, handle, frame.timestamp, cache)?;
    let (Some(previous_bits), Some(current_bits)) = (previous_bits, current_bits) else {
        return Ok((false, false));
    };

    Ok(classify_edge(previous_bits.as_ref(), current_bits.as_ref()))
}

fn sample_signal_bits(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<Option<Rc<str>>, ExprDiagnostic> {
    cache.sample_bits(host, handle, timestamp)
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

fn bits_to_string(bits: &[BoundBit]) -> String {
    bits.iter()
        .map(|bit| match bit {
            BoundBit::Zero => '0',
            BoundBit::One => '1',
            BoundBit::X => 'x',
            BoundBit::Z => 'z',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::expr::ast::BinaryOpAst;
    use crate::expr::host::{ExprStorage, ExprType, ExprTypeKind, SampledValue};
    use crate::expr::sema::{BoundLogicalExpr, BoundLogicalKind, BoundLogicalNode};

    use super::*;

    #[derive(Default)]
    struct HostStub {
        sampled: RefCell<Vec<SignalHandle>>,
    }

    impl ExpressionHost for HostStub {
        fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            Ok(SignalHandle(1))
        }

        fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            Ok(ExprType {
                kind: ExprTypeKind::BitVector,
                storage: ExprStorage::Scalar,
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
            })
        }

        fn sample_value(
            &self,
            handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            self.sampled.borrow_mut().push(handle);
            Ok(SampledValue {
                bits: Some("1".to_string()),
            })
        }
    }

    fn literal(bits: &[BoundBit]) -> BoundLogicalNode {
        BoundLogicalNode {
            ty: ExprType {
                kind: ExprTypeKind::BitVector,
                storage: if bits.len() > 1 {
                    ExprStorage::PackedVector
                } else {
                    ExprStorage::Scalar
                },
                width: bits.len() as u32,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
            },
            span: crate::expr::Span::new(0, 0),
            kind: BoundLogicalKind::IntegralLiteral {
                value: crate::expr::sema::BoundIntegralValue {
                    bits: bits.to_vec(),
                    signed: false,
                },
                is_unsized: false,
            },
        }
    }

    #[test]
    fn logical_and_short_circuits_rhs() {
        let host = HostStub::default();
        let expr = BoundLogicalExpr {
            root: BoundLogicalNode {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::Scalar,
                    width: 1,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                },
                span: crate::expr::Span::new(0, 0),
                kind: BoundLogicalKind::Binary {
                    op: BinaryOpAst::LogicalAnd,
                    left: Box::new(literal(&[BoundBit::Zero])),
                    right: Box::new(BoundLogicalNode {
                        ty: ExprType {
                            kind: ExprTypeKind::BitVector,
                            storage: ExprStorage::Scalar,
                            width: 1,
                            is_four_state: true,
                            is_signed: false,
                            enum_type_id: None,
                        },
                        span: crate::expr::Span::new(0, 0),
                        kind: BoundLogicalKind::SignalRef {
                            handle: SignalHandle(9),
                        },
                    }),
                },
            },
        };

        let value = eval_logical_expr_at(&expr, &host, 10).expect("eval works");
        assert_eq!(value.bits, "0");
        assert!(host.sampled.borrow().is_empty());
    }

    #[test]
    fn wildcard_equality_preserves_unknown_from_lhs() {
        let truth = compare_wild(
            BinaryOpAst::WildEq,
            RuntimeValue {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::PackedVector,
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                },
                bits: vec![BoundBit::X, BoundBit::One],
            },
            RuntimeValue {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::PackedVector,
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                },
                bits: vec![BoundBit::Zero, BoundBit::One],
            },
        );
        assert_eq!(truth, TruthValue::Unknown);
    }
}
