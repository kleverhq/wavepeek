use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::ast::{BinaryOpAst, UnaryOpAst};
use crate::expr::diagnostic::ExprDiagnostic;
use crate::expr::host::{
    EventEvalFrame, ExprType, ExprTypeKind, ExpressionHost, SampledValue, SignalHandle,
};
use crate::expr::sema::{
    BoundBit, BoundCastKind, BoundEventExpr, BoundEventKind, BoundInsideItem, BoundLogicalExpr,
    BoundLogicalKind, BoundLogicalNode, BoundSelection,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprValuePayload {
    Integral { bits: String, label: Option<String> },
    Real { value: f64 },
    String { value: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprValue {
    pub ty: ExprType,
    pub payload: ExprValuePayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TruthValue {
    Zero,
    One,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeValue {
    ty: ExprType,
    payload: RuntimeValuePayload,
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeValuePayload {
    Integral {
        bits: Vec<BoundBit>,
        label: Option<String>,
    },
    Real {
        value: f64,
    },
    String {
        value: String,
    },
}

#[derive(Default)]
struct EvalCache {
    samples: HashMap<(SignalHandle, u64), CachedSample>,
    event_occurrences: HashMap<(SignalHandle, u64), bool>,
    signal_types: HashMap<SignalHandle, ExprType>,
}

#[derive(Debug, Clone, PartialEq)]
enum CachedSample {
    Integral {
        bits: Option<Rc<[BoundBit]>>,
        label: Option<Rc<str>>,
    },
    Real {
        value: Option<f64>,
    },
    String {
        value: Option<Rc<str>>,
    },
}

impl EvalCache {
    fn sample_value(
        &mut self,
        host: &dyn ExpressionHost,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<CachedSample, ExprDiagnostic> {
        let key = (handle, timestamp);
        if let Some(value) = self.samples.get(&key) {
            return Ok(value.clone());
        }

        let sampled = cache_sample(host.sample_value(handle, timestamp)?);
        self.samples.insert(key, sampled.clone());
        Ok(sampled)
    }

    fn signal_type(
        &mut self,
        host: &dyn ExpressionHost,
        handle: SignalHandle,
    ) -> Result<ExprType, ExprDiagnostic> {
        if let Some(ty) = self.signal_types.get(&handle) {
            return Ok(ty.clone());
        }

        let ty = host.signal_type(handle)?;
        self.signal_types.insert(handle, ty.clone());
        Ok(ty)
    }

    fn event_occurred(
        &mut self,
        host: &dyn ExpressionHost,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<bool, ExprDiagnostic> {
        let key = (handle, timestamp);
        if let Some(value) = self.event_occurrences.get(&key) {
            return Ok(*value);
        }

        let occurred = host.event_occurred(handle, timestamp)?;
        self.event_occurrences.insert(key, occurred);
        Ok(occurred)
    }
}

fn cache_sample(sample: SampledValue) -> CachedSample {
    match sample {
        SampledValue::Integral { bits, label } => CachedSample::Integral {
            bits: bits.map(|bits| Rc::<[BoundBit]>::from(bits_from_sample(bits.as_str()))),
            label: label.map(Rc::<str>::from),
        },
        SampledValue::Real { value } => CachedSample::Real { value },
        SampledValue::String { value } => CachedSample::String {
            value: value.map(Rc::<str>::from),
        },
    }
}

pub fn eval_logical_expr_at(
    expr: &BoundLogicalExpr,
    host: &dyn ExpressionHost,
    timestamp: u64,
) -> Result<ExprValue, ExprDiagnostic> {
    let mut cache = EvalCache::default();
    let value = eval_node(&expr.root, host, timestamp, &mut cache)?;
    let value = coerce_runtime_to_type(value, &expr.root.ty)?;
    runtime_value_to_public(value)
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
            if truthiness(&iff_value) == TruthValue::One {
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
    coerce_runtime_to_type(value, &expr.root.ty)
}

fn eval_node(
    node: &BoundLogicalNode,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let value = match &node.kind {
        BoundLogicalKind::SignalRef { handle } => {
            eval_signal_ref(node, *handle, host, timestamp, cache)?
        }
        BoundLogicalKind::IntegralLiteral { value, .. } => RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Integral {
                bits: value.bits.clone(),
                label: None,
            },
        },
        BoundLogicalKind::RealLiteral { value } => RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Real { value: *value },
        },
        BoundLogicalKind::StringLiteral { value } => RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::String {
                value: value.clone(),
            },
        },
        BoundLogicalKind::EnumLabel { value, label } => RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Integral {
                bits: value.bits.clone(),
                label: Some(label.clone()),
            },
        },
        BoundLogicalKind::Parenthesized { expr } => eval_node(expr, host, timestamp, cache)?,
        BoundLogicalKind::Cast { kind, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            eval_cast(*kind, value, &node.ty)?
        }
        BoundLogicalKind::Selection { base, selection } => {
            let base_value = eval_node(base, host, timestamp, cache)?;
            eval_selection(base_value, selection, host, timestamp, cache, &node.ty)?
        }
        BoundLogicalKind::Unary { op, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            eval_unary(*op, value, &node.ty)?
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
                let item_value = coerce_runtime_to_type(item_value, &item.ty)?;
                bits.extend(expect_integral_bits(&item_value)?.0);
            }
            RuntimeValue {
                ty: node.ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
        BoundLogicalKind::Replication { count, expr } => {
            let value = eval_node(expr, host, timestamp, cache)?;
            let value = coerce_runtime_to_type(value, &expr.ty)?;
            let (value_bits, _) = expect_integral_bits(&value)?;
            let mut bits = Vec::with_capacity(value_bits.len() * *count);
            for _ in 0..*count {
                bits.extend(value_bits.iter().copied());
            }
            RuntimeValue {
                ty: node.ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
        BoundLogicalKind::Triggered { handle } => RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Integral {
                bits: vec![if cache.event_occurred(host, *handle, timestamp)? {
                    BoundBit::One
                } else {
                    BoundBit::Zero
                }],
                label: None,
            },
        },
    };

    coerce_runtime_to_type(value, &node.ty)
}

fn runtime_value_to_public(value: RuntimeValue) -> Result<ExprValue, ExprDiagnostic> {
    let payload = match value.payload {
        RuntimeValuePayload::Integral { bits, label } => ExprValuePayload::Integral {
            bits: bits_to_string(bits.as_slice()),
            label,
        },
        RuntimeValuePayload::Real { value } => ExprValuePayload::Real { value },
        RuntimeValuePayload::String { value } => ExprValuePayload::String { value },
    };
    Ok(ExprValue {
        ty: value.ty,
        payload,
    })
}

fn eval_signal_ref(
    node: &BoundLogicalNode,
    handle: SignalHandle,
    host: &dyn ExpressionHost,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let sampled = cache.sample_value(host, handle, timestamp)?;
    match (&node.ty.kind, sampled) {
        (
            ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore,
            CachedSample::Integral { bits, label },
        ) => Ok(RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Integral {
                bits: bits
                    .map(|raw| raw.as_ref().to_vec())
                    .unwrap_or_else(|| vec![BoundBit::X; node.ty.width.max(1) as usize]),
                label: label.map(|label| label.to_string()),
            },
        }),
        (ExprTypeKind::Real, CachedSample::Real { value: Some(value) }) => Ok(RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::Real { value },
        }),
        (ExprTypeKind::String, CachedSample::String { value: Some(value) }) => Ok(RuntimeValue {
            ty: node.ty.clone(),
            payload: RuntimeValuePayload::String {
                value: value.to_string(),
            },
        }),
        (ExprTypeKind::Real, CachedSample::Real { value: None }) => Err(runtime_diag(
            "C4-RUNTIME-MISSING-SAMPLE",
            "real operand has no sampled value at or before the requested timestamp",
        )),
        (ExprTypeKind::String, CachedSample::String { value: None }) => Err(runtime_diag(
            "C4-RUNTIME-MISSING-SAMPLE",
            "string operand has no sampled value at or before the requested timestamp",
        )),
        _ => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "host returned a sampled value shape that does not match the bound operand type",
        )),
    }
}

fn eval_cast(
    _kind: BoundCastKind,
    value: RuntimeValue,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    coerce_runtime_to_type(value, result_ty)
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
    let base = coerce_runtime_to_type(base, &base_ty)?;
    let (base_bits, _) = expect_integral_bits(&base)?;
    let width = base_bits.len();
    let result = match selection {
        BoundSelection::Bit { index } => {
            let index = eval_node(index, host, timestamp, cache)?;
            let index_ty = index.ty.clone();
            let index = coerce_runtime_to_type(index, &index_ty)?;
            let (index_bits, _) = expect_integral_bits(&index)?;
            let bits = if let Some(bit_index) = runtime_index_to_usize(index_bits) {
                if bit_index < width {
                    vec![base_bits[width - 1 - bit_index]]
                } else {
                    vec![if base.ty.is_four_state {
                        BoundBit::X
                    } else {
                        BoundBit::Zero
                    }]
                }
            } else {
                vec![if base.ty.is_four_state {
                    BoundBit::X
                } else {
                    BoundBit::Zero
                }]
            };
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
        BoundSelection::Part { msb, lsb } => {
            let bits = part_select_bits(base_bits, *msb, *lsb, base.ty.is_four_state);
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
        BoundSelection::IndexedUp {
            base: index_base,
            width,
        } => {
            let index_base = eval_node(index_base, host, timestamp, cache)?;
            let index_base_ty = index_base.ty.clone();
            let index_base = coerce_runtime_to_type(index_base, &index_base_ty)?;
            let (index_bits, _) = expect_integral_bits(&index_base)?;
            let bits = if let Some(start) = runtime_index_to_i64(index_bits) {
                indexed_part_select_bits(base_bits, start, *width, true, base.ty.is_four_state)
            } else {
                vec![BoundBit::X; *width]
            };
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
        BoundSelection::IndexedDown {
            base: index_base,
            width,
        } => {
            let index_base = eval_node(index_base, host, timestamp, cache)?;
            let index_base_ty = index_base.ty.clone();
            let index_base = coerce_runtime_to_type(index_base, &index_base_ty)?;
            let (index_bits, _) = expect_integral_bits(&index_base)?;
            let bits = if let Some(start) = runtime_index_to_i64(index_bits) {
                indexed_part_select_bits(base_bits, start, *width, false, base.ty.is_four_state)
            } else {
                vec![BoundBit::X; *width]
            };
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral { bits, label: None },
            }
        }
    };
    Ok(result)
}

fn eval_unary(
    op: UnaryOpAst,
    value: RuntimeValue,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let value_ty = value.ty.clone();
    let value = coerce_runtime_to_type(value, &value_ty)?;
    Ok(match op {
        UnaryOpAst::Plus => coerce_runtime_to_type(value, result_ty)?,
        UnaryOpAst::Minus => {
            if matches!(&result_ty.kind, ExprTypeKind::Real) {
                let real = coerce_runtime_to_type(value, &real_type())?;
                let RuntimeValuePayload::Real { value } = real.payload else {
                    unreachable!();
                };
                RuntimeValue {
                    ty: result_ty.clone(),
                    payload: RuntimeValuePayload::Real { value: -value },
                }
            } else {
                let value = coerce_runtime_to_type(value, result_ty)?;
                let (bits, _) = expect_integral_bits(&value)?;
                if bits
                    .iter()
                    .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
                {
                    all_x(result_ty)
                } else {
                    match bits_to_u128(bits) {
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
                                    payload: RuntimeValuePayload::Integral {
                                        bits: unsigned_to_bits(negated, result_ty.width),
                                        label: None,
                                    },
                                },
                                result_ty,
                            )?
                        }
                        None => all_x(result_ty),
                    }
                }
            }
        }
        UnaryOpAst::LogicalNot => RuntimeValue {
            ty: result_ty.clone(),
            payload: RuntimeValuePayload::Integral {
                bits: vec![match truthiness(&value) {
                    TruthValue::Zero => BoundBit::One,
                    TruthValue::One => BoundBit::Zero,
                    TruthValue::Unknown => BoundBit::X,
                }],
                label: None,
            },
        },
        UnaryOpAst::BitNot => {
            let value = coerce_runtime_to_type(value, result_ty)?;
            let (bits, _) = expect_integral_bits(&value)?;
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: bits
                        .iter()
                        .map(|bit| match bit {
                            BoundBit::Zero => BoundBit::One,
                            BoundBit::One => BoundBit::Zero,
                            BoundBit::X | BoundBit::Z => BoundBit::X,
                        })
                        .collect(),
                    label: None,
                },
            }
        }
        UnaryOpAst::ReduceAnd
        | UnaryOpAst::ReduceNand
        | UnaryOpAst::ReduceOr
        | UnaryOpAst::ReduceNor
        | UnaryOpAst::ReduceXor
        | UnaryOpAst::ReduceXnor => {
            let value_ty = value.ty.clone();
            let value = coerce_runtime_to_type(value, &value_ty)?;
            let (bits, _) = expect_integral_bits(&value)?;
            let reduced = match op {
                UnaryOpAst::ReduceAnd => reduce_and(bits),
                UnaryOpAst::ReduceNand => invert_bit(reduce_and(bits)),
                UnaryOpAst::ReduceOr => reduce_or(bits),
                UnaryOpAst::ReduceNor => invert_bit(reduce_or(bits)),
                UnaryOpAst::ReduceXor => reduce_xor(bits),
                UnaryOpAst::ReduceXnor => invert_bit(reduce_xor(bits)),
                _ => BoundBit::X,
            };
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![reduced],
                    label: None,
                },
            }
        }
    })
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
            let lhs_truth = truthiness(&lhs);
            let truth = match lhs_truth {
                TruthValue::Zero => TruthValue::Zero,
                TruthValue::One => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    truthiness(&rhs)
                }
                TruthValue::Unknown => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    let rhs_truth = truthiness(&rhs);
                    if rhs_truth == TruthValue::Zero {
                        TruthValue::Zero
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            return Ok(RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![truth_to_bit(truth)],
                    label: None,
                },
            });
        }
        BinaryOpAst::LogicalOr => {
            let lhs = eval_node(left, host, timestamp, cache)?;
            let lhs_truth = truthiness(&lhs);
            let truth = match lhs_truth {
                TruthValue::One => TruthValue::One,
                TruthValue::Zero => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    truthiness(&rhs)
                }
                TruthValue::Unknown => {
                    let rhs = eval_node(right, host, timestamp, cache)?;
                    let rhs_truth = truthiness(&rhs);
                    if rhs_truth == TruthValue::One {
                        TruthValue::One
                    } else {
                        TruthValue::Unknown
                    }
                }
            };
            return Ok(RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![truth_to_bit(truth)],
                    label: None,
                },
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
            let truth = compare_unknown_sensitive(op, lhs, rhs)?;
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![truth_to_bit(truth)],
                    label: None,
                },
            }
        }
        BinaryOpAst::CaseEq | BinaryOpAst::CaseNe => {
            let truth = compare_case(op, lhs, rhs)?;
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![truth_to_bit(truth)],
                    label: None,
                },
            }
        }
        BinaryOpAst::WildEq | BinaryOpAst::WildNe => {
            let truth = compare_wild(op, lhs, rhs)?;
            RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![truth_to_bit(truth)],
                    label: None,
                },
            }
        }
        BinaryOpAst::BitAnd | BinaryOpAst::BitOr | BinaryOpAst::BitXor | BinaryOpAst::BitXnor => {
            eval_bitwise(op, lhs, rhs, result_ty)?
        }
        BinaryOpAst::ShiftLeft
        | BinaryOpAst::ShiftRight
        | BinaryOpAst::ShiftArithLeft
        | BinaryOpAst::ShiftArithRight => eval_shift(op, lhs, rhs, result_ty)?,
        BinaryOpAst::Add
        | BinaryOpAst::Subtract
        | BinaryOpAst::Multiply
        | BinaryOpAst::Divide
        | BinaryOpAst::Modulo
        | BinaryOpAst::Power => eval_arithmetic(op, lhs, rhs, result_ty)?,
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
    let cond_truth = truthiness(&cond);

    match cond_truth {
        TruthValue::One => {
            let value = eval_node(when_true, host, timestamp, cache)?;
            coerce_runtime_to_type(value, result_ty)
        }
        TruthValue::Zero => {
            let value = eval_node(when_false, host, timestamp, cache)?;
            coerce_runtime_to_type(value, result_ty)
        }
        TruthValue::Unknown => {
            let lhs = eval_node(when_true, host, timestamp, cache)?;
            let rhs = eval_node(when_false, host, timestamp, cache)?;
            let lhs = coerce_runtime_to_type(lhs, result_ty)?;
            let rhs = coerce_runtime_to_type(rhs, result_ty)?;
            if matches!(
                result_ty.kind,
                ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore
            ) {
                let (lhs_bits, _) = expect_integral_bits(&lhs)?;
                let (rhs_bits, _) = expect_integral_bits(&rhs)?;
                let bits = lhs_bits
                    .iter()
                    .zip(rhs_bits.iter())
                    .map(|(lhs, rhs)| if lhs == rhs { *lhs } else { BoundBit::X })
                    .collect::<Vec<_>>();
                coerce_runtime_to_type(
                    RuntimeValue {
                        ty: result_ty.clone(),
                        payload: RuntimeValuePayload::Integral { bits, label: None },
                    },
                    result_ty,
                )
            } else if lhs.payload == rhs.payload {
                Ok(lhs)
            } else {
                Err(runtime_diag(
                    "C4-RUNTIME-CONDITIONAL-UNKNOWN",
                    "unknown conditional selection requires matching non-integral result arms",
                ))
            }
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
                compare_wild(BinaryOpAst::WildEq, lhs.clone(), rhs)?
            }
            BoundInsideItem::Range { low, high } => {
                let low = eval_node(low, host, timestamp, cache)?;
                let high = eval_node(high, host, timestamp, cache)?;
                let ge_low = compare_unknown_sensitive(BinaryOpAst::Ge, lhs.clone(), low)?;
                let le_high = compare_unknown_sensitive(BinaryOpAst::Le, lhs.clone(), high)?;
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
                    payload: RuntimeValuePayload::Integral {
                        bits: vec![BoundBit::One],
                        label: None,
                    },
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
        payload: RuntimeValuePayload::Integral {
            bits: vec![bit],
            label: None,
        },
    })
}

fn eval_bitwise(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let left = coerce_runtime_to_type(left, result_ty)?;
    let right = coerce_runtime_to_type(right, result_ty)?;
    let (left_bits, _) = expect_integral_bits(&left)?;
    let (right_bits, _) = expect_integral_bits(&right)?;

    let bits = left_bits
        .iter()
        .zip(right_bits.iter())
        .map(|(lhs, rhs)| match op {
            BinaryOpAst::BitAnd => bitwise_and(*lhs, *rhs),
            BinaryOpAst::BitOr => bitwise_or(*lhs, *rhs),
            BinaryOpAst::BitXor => bitwise_xor(*lhs, *rhs),
            BinaryOpAst::BitXnor => invert_bit(bitwise_xor(*lhs, *rhs)),
            _ => BoundBit::X,
        })
        .collect();

    Ok(RuntimeValue {
        ty: result_ty.clone(),
        payload: RuntimeValuePayload::Integral { bits, label: None },
    })
}

fn eval_shift(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    let left = coerce_runtime_to_type(left, result_ty)?;
    let right_ty = right.ty.clone();
    let right = coerce_runtime_to_type(right, &right_ty)?;
    let (left_bits, _) = expect_integral_bits(&left)?;
    let (right_bits, _) = expect_integral_bits(&right)?;
    if right_bits
        .iter()
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return Ok(all_x(result_ty));
    }

    let shift = bits_to_u128(right_bits)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    if shift >= left_bits.len() {
        return Ok(match op {
            BinaryOpAst::ShiftArithRight if result_ty.is_signed => RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![
                        left_bits.first().copied().unwrap_or(BoundBit::Zero);
                        left_bits.len()
                    ],
                    label: None,
                },
            },
            _ => RuntimeValue {
                ty: result_ty.clone(),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![BoundBit::Zero; left_bits.len()],
                    label: None,
                },
            },
        });
    }

    let bits = match op {
        BinaryOpAst::ShiftLeft | BinaryOpAst::ShiftArithLeft => {
            let mut bits = left_bits[shift..].to_vec();
            bits.extend(std::iter::repeat_n(BoundBit::Zero, shift));
            bits
        }
        BinaryOpAst::ShiftRight => {
            let mut bits = vec![BoundBit::Zero; shift];
            bits.extend(left_bits[..left_bits.len() - shift].iter().copied());
            bits
        }
        BinaryOpAst::ShiftArithRight => {
            let fill = if result_ty.is_signed {
                left_bits.first().copied().unwrap_or(BoundBit::Zero)
            } else {
                BoundBit::Zero
            };
            let mut bits = vec![fill; shift];
            bits.extend(left_bits[..left_bits.len() - shift].iter().copied());
            bits
        }
        _ => vec![BoundBit::X; left_bits.len()],
    };

    Ok(RuntimeValue {
        ty: result_ty.clone(),
        payload: RuntimeValuePayload::Integral { bits, label: None },
    })
}

fn eval_arithmetic(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
    result_ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    if matches!(&result_ty.kind, ExprTypeKind::Real) {
        let left = coerce_runtime_to_type(left, &real_type())?;
        let right = coerce_runtime_to_type(right, &real_type())?;
        let RuntimeValuePayload::Real { value: lhs } = left.payload else {
            unreachable!();
        };
        let RuntimeValuePayload::Real { value: rhs } = right.payload else {
            unreachable!();
        };
        let value = match op {
            BinaryOpAst::Add => lhs + rhs,
            BinaryOpAst::Subtract => lhs - rhs,
            BinaryOpAst::Multiply => lhs * rhs,
            BinaryOpAst::Divide => lhs / rhs,
            BinaryOpAst::Modulo => lhs % rhs,
            BinaryOpAst::Power => lhs.powf(rhs),
            _ => unreachable!(),
        };
        return Ok(RuntimeValue {
            ty: result_ty.clone(),
            payload: RuntimeValuePayload::Real { value },
        });
    }

    let left = coerce_runtime_to_type(left, result_ty)?;
    let exponent_signed = right.ty.is_signed;
    let right_ty = right.ty.clone();
    let right = if op == BinaryOpAst::Power {
        coerce_runtime_to_type(right, &right_ty)?
    } else {
        coerce_runtime_to_type(right, result_ty)?
    };
    let (left_bits, _) = expect_integral_bits(&left)?;
    let (right_bits, _) = expect_integral_bits(&right)?;

    if left_bits
        .iter()
        .chain(right_bits.iter())
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return Ok(all_x(result_ty));
    }

    let Some(lhs_unsigned) = bits_to_u128(left_bits) else {
        return Ok(all_x(result_ty));
    };
    let Some(rhs_unsigned) = bits_to_u128(right_bits) else {
        return Ok(all_x(result_ty));
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
                return Ok(all_x(result_ty));
            }
            if result_ty.is_signed {
                let Some(lhs_signed) = bits_to_i128(left_bits, true) else {
                    return Ok(all_x(result_ty));
                };
                let Some(rhs_signed) = bits_to_i128(right_bits, true) else {
                    return Ok(all_x(result_ty));
                };
                if rhs_signed == 0 {
                    return Ok(all_x(result_ty));
                }
                signed_to_bits(lhs_signed.wrapping_div(rhs_signed), result_ty.width)
            } else {
                unsigned_to_bits(lhs_unsigned / rhs_unsigned, result_ty.width)
            }
        }
        BinaryOpAst::Modulo => {
            if rhs_unsigned == 0 {
                return Ok(all_x(result_ty));
            }
            if result_ty.is_signed {
                let Some(lhs_signed) = bits_to_i128(left_bits, true) else {
                    return Ok(all_x(result_ty));
                };
                let Some(rhs_signed) = bits_to_i128(right_bits, true) else {
                    return Ok(all_x(result_ty));
                };
                if rhs_signed == 0 {
                    return Ok(all_x(result_ty));
                }
                signed_to_bits(lhs_signed.wrapping_rem(rhs_signed), result_ty.width)
            } else {
                unsigned_to_bits(lhs_unsigned % rhs_unsigned, result_ty.width)
            }
        }
        BinaryOpAst::Power => {
            if exponent_signed {
                let Some(exp_signed) = bits_to_i128(right_bits, true) else {
                    return Ok(all_x(result_ty));
                };
                if exp_signed < 0 {
                    if lhs_unsigned == 0 {
                        return Ok(all_x(result_ty));
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
        _ => return Ok(all_x(result_ty)),
    };

    Ok(RuntimeValue {
        ty: result_ty.clone(),
        payload: RuntimeValuePayload::Integral { bits, label: None },
    })
}

fn compare_unknown_sensitive(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
) -> Result<TruthValue, ExprDiagnostic> {
    if matches!(&left.ty.kind, ExprTypeKind::Real) || matches!(&right.ty.kind, ExprTypeKind::Real) {
        let left = coerce_runtime_to_type(left, &real_type())?;
        let right = coerce_runtime_to_type(right, &real_type())?;
        let RuntimeValuePayload::Real { value: lhs } = left.payload else {
            unreachable!();
        };
        let RuntimeValuePayload::Real { value: rhs } = right.payload else {
            unreachable!();
        };
        return Ok(match op {
            BinaryOpAst::Lt => ordering_truth(lhs < rhs),
            BinaryOpAst::Le => ordering_truth(lhs <= rhs),
            BinaryOpAst::Gt => ordering_truth(lhs > rhs),
            BinaryOpAst::Ge => ordering_truth(lhs >= rhs),
            BinaryOpAst::Eq => ordering_truth(lhs == rhs),
            BinaryOpAst::Ne => ordering_truth(lhs != rhs),
            _ => TruthValue::Unknown,
        });
    }

    if matches!(&left.ty.kind, ExprTypeKind::String)
        || matches!(&right.ty.kind, ExprTypeKind::String)
    {
        let left = coerce_runtime_to_type(left, &string_type())?;
        let right = coerce_runtime_to_type(right, &string_type())?;
        let RuntimeValuePayload::String { value: lhs } = left.payload else {
            unreachable!();
        };
        let RuntimeValuePayload::String { value: rhs } = right.payload else {
            unreachable!();
        };
        return Ok(match op {
            BinaryOpAst::Eq => ordering_truth(lhs == rhs),
            BinaryOpAst::Ne => ordering_truth(lhs != rhs),
            _ => TruthValue::Unknown,
        });
    }

    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common)?;
    let right = coerce_runtime_to_type(right, &common)?;
    let (left_bits, _) = expect_integral_bits(&left)?;
    let (right_bits, _) = expect_integral_bits(&right)?;

    if left_bits
        .iter()
        .chain(right_bits.iter())
        .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
    {
        return Ok(TruthValue::Unknown);
    }

    let ord = compare_ordering(
        left_bits,
        right_bits,
        left.ty.is_signed && right.ty.is_signed,
    );
    Ok(match op {
        BinaryOpAst::Lt => ordering_truth(ord.is_lt()),
        BinaryOpAst::Le => ordering_truth(ord.is_le()),
        BinaryOpAst::Gt => ordering_truth(ord.is_gt()),
        BinaryOpAst::Ge => ordering_truth(ord.is_ge()),
        BinaryOpAst::Eq => ordering_truth(ord.is_eq()),
        BinaryOpAst::Ne => ordering_truth(!ord.is_eq()),
        _ => TruthValue::Unknown,
    })
}

fn compare_case(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
) -> Result<TruthValue, ExprDiagnostic> {
    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common)?;
    let right = coerce_runtime_to_type(right, &common)?;
    let equal = expect_integral_bits(&left)?.0 == expect_integral_bits(&right)?.0;
    Ok(match op {
        BinaryOpAst::CaseEq => ordering_truth(equal),
        BinaryOpAst::CaseNe => ordering_truth(!equal),
        _ => TruthValue::Unknown,
    })
}

fn compare_wild(
    op: BinaryOpAst,
    left: RuntimeValue,
    right: RuntimeValue,
) -> Result<TruthValue, ExprDiagnostic> {
    let common = common_integral_type_for_eval(&left.ty, &right.ty);
    let left = coerce_runtime_to_type(left, &common)?;
    let right = coerce_runtime_to_type(right, &common)?;
    let (left_bits, _) = expect_integral_bits(&left)?;
    let (right_bits, _) = expect_integral_bits(&right)?;

    let mut saw_unknown = false;
    for (lhs, rhs) in left_bits.iter().zip(right_bits.iter()) {
        if matches!(rhs, BoundBit::X | BoundBit::Z) {
            continue;
        }
        if matches!(lhs, BoundBit::X | BoundBit::Z) {
            saw_unknown = true;
            continue;
        }
        if lhs != rhs {
            return Ok(match op {
                BinaryOpAst::WildEq => TruthValue::Zero,
                BinaryOpAst::WildNe => TruthValue::One,
                _ => TruthValue::Unknown,
            });
        }
    }

    let eq = if saw_unknown {
        TruthValue::Unknown
    } else {
        TruthValue::One
    };
    Ok(match op {
        BinaryOpAst::WildEq => eq,
        BinaryOpAst::WildNe => match eq {
            TruthValue::Zero => TruthValue::One,
            TruthValue::One => TruthValue::Zero,
            TruthValue::Unknown => TruthValue::Unknown,
        },
        _ => TruthValue::Unknown,
    })
}

fn compare_ordering(left: &[BoundBit], right: &[BoundBit], signed: bool) -> Ordering {
    if !signed {
        return compare_unsigned(left, right);
    }

    let left_negative = left.first().copied() == Some(BoundBit::One);
    let right_negative = right.first().copied() == Some(BoundBit::One);
    match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => compare_unsigned(left, right),
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

fn coerce_runtime_to_type(
    value: RuntimeValue,
    ty: &ExprType,
) -> Result<RuntimeValue, ExprDiagnostic> {
    if &value.ty == ty {
        return Ok(value);
    }

    let RuntimeValue {
        ty: source_ty,
        payload,
    } = value;
    match &ty.kind {
        ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore => {
            match payload {
                RuntimeValuePayload::Integral { bits, .. } => {
                    let mut bits = resize_bits(bits, ty.width, source_ty.is_signed);
                    if !ty.is_four_state {
                        for bit in &mut bits {
                            if matches!(bit, BoundBit::X | BoundBit::Z) {
                                *bit = BoundBit::Zero;
                            }
                        }
                    }
                    let label = if matches!(&ty.kind, ExprTypeKind::EnumCore) {
                        enum_label_for_bits(ty, bits.as_slice())
                    } else {
                        None
                    };
                    Ok(RuntimeValue {
                        ty: ty.clone(),
                        payload: RuntimeValuePayload::Integral { bits, label },
                    })
                }
                RuntimeValuePayload::Real { value } => {
                    let bits = if ty.is_signed {
                        signed_to_bits(value.trunc() as i128, ty.width)
                    } else {
                        unsigned_to_bits((value.trunc() as i128) as u128, ty.width)
                    };
                    let label = if matches!(&ty.kind, ExprTypeKind::EnumCore) {
                        enum_label_for_bits(ty, bits.as_slice())
                    } else {
                        None
                    };
                    Ok(RuntimeValue {
                        ty: ty.clone(),
                        payload: RuntimeValuePayload::Integral { bits, label },
                    })
                }
                RuntimeValuePayload::String { .. } => Err(runtime_diag(
                    "C4-RUNTIME-CAST",
                    "string values cannot be coerced to integral types",
                )),
            }
        }
        ExprTypeKind::Real => match payload {
            RuntimeValuePayload::Real { value } => Ok(RuntimeValue {
                ty: ty.clone(),
                payload: RuntimeValuePayload::Real { value },
            }),
            RuntimeValuePayload::Integral { bits, .. } => {
                if bits
                    .iter()
                    .any(|bit| matches!(bit, BoundBit::X | BoundBit::Z))
                {
                    return Err(runtime_diag(
                        "C4-RUNTIME-REAL-CAST",
                        "integral-to-real conversion is invalid when the source contains x or z",
                    ));
                }
                let value = if source_ty.is_signed {
                    bits_to_i128(bits.as_slice(), true)
                        .map(|value| value as f64)
                        .ok_or_else(|| {
                            runtime_diag(
                                "C4-RUNTIME-REAL-CAST",
                                "integral-to-real conversion overflowed the supported range",
                            )
                        })?
                } else {
                    bits_to_u128(bits.as_slice())
                        .map(|value| value as f64)
                        .ok_or_else(|| {
                            runtime_diag(
                                "C4-RUNTIME-REAL-CAST",
                                "integral-to-real conversion overflowed the supported range",
                            )
                        })?
                };
                Ok(RuntimeValue {
                    ty: ty.clone(),
                    payload: RuntimeValuePayload::Real { value },
                })
            }
            RuntimeValuePayload::String { .. } => Err(runtime_diag(
                "C4-RUNTIME-CAST",
                "string values cannot be coerced to real",
            )),
        },
        ExprTypeKind::String => match payload {
            RuntimeValuePayload::String { value } => Ok(RuntimeValue {
                ty: ty.clone(),
                payload: RuntimeValuePayload::String { value },
            }),
            _ => Err(runtime_diag(
                "C4-RUNTIME-CAST",
                "string casts are supported only as string identity",
            )),
        },
        ExprTypeKind::Event => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "raw event values cannot be coerced as ordinary expression values",
        )),
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
        enum_labels: None,
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
        payload: RuntimeValuePayload::Integral {
            bits: vec![BoundBit::X; ty.width.max(1) as usize],
            label: None,
        },
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

fn truthiness(value: &RuntimeValue) -> TruthValue {
    match &value.payload {
        RuntimeValuePayload::Integral { bits, .. } => {
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
        RuntimeValuePayload::Real { value } => {
            if *value == 0.0 {
                TruthValue::Zero
            } else {
                TruthValue::One
            }
        }
        RuntimeValuePayload::String { .. } => TruthValue::Unknown,
    }
}

fn expect_integral_bits(
    value: &RuntimeValue,
) -> Result<(&[BoundBit], Option<&str>), ExprDiagnostic> {
    match &value.payload {
        RuntimeValuePayload::Integral { bits, label } => Ok((bits.as_slice(), label.as_deref())),
        _ => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "integral runtime payload was required but a non-integral value was produced",
        )),
    }
}

fn runtime_diag(code: &'static str, message: &str) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: crate::expr::DiagnosticLayer::Runtime,
        code,
        message: message.to_string(),
        primary_span: crate::expr::Span::new(0, 0),
        notes: vec![],
    }
}

fn enum_label_for_bits(ty: &ExprType, bits: &[BoundBit]) -> Option<String> {
    let raw = bits_to_string(bits);
    ty.enum_labels
        .as_ref()?
        .iter()
        .find(|entry| entry.bits == raw)
        .map(|entry| entry.name.clone())
}

fn real_type() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Real,
        storage: crate::expr::ExprStorage::Scalar,
        width: 64,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn string_type() -> ExprType {
    ExprType {
        kind: ExprTypeKind::String,
        storage: crate::expr::ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn any_tracked_matches(
    host: &dyn ExpressionHost,
    frame: &EventEvalFrame<'_>,
    cache: &mut EvalCache,
) -> Result<bool, ExprDiagnostic> {
    for &handle in frame.tracked_signals {
        if signal_changed(
            host,
            handle,
            frame.previous_timestamp,
            frame.timestamp,
            cache,
        )? {
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
    signal_changed(
        host,
        handle,
        frame.previous_timestamp,
        frame.timestamp,
        cache,
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

    let ty = cache.signal_type(host, handle)?;
    if !matches!(
        ty.kind,
        ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore
    ) {
        return Err(runtime_diag(
            "C4-SEMANTIC-EVENT-EDGE",
            "edge event terms require integral operands",
        ));
    }

    let previous_bits = sample_signal_bits(host, handle, previous_timestamp, cache)?;
    let current_bits = sample_signal_bits(host, handle, frame.timestamp, cache)?;
    let (Some(previous_bits), Some(current_bits)) = (previous_bits, current_bits) else {
        return Ok((false, false));
    };

    Ok(classify_edge_bits(
        previous_bits.as_ref(),
        current_bits.as_ref(),
    ))
}

fn signal_changed(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    previous_timestamp: Option<u64>,
    current_timestamp: u64,
    cache: &mut EvalCache,
) -> Result<bool, ExprDiagnostic> {
    let ty = cache.signal_type(host, handle)?;
    match ty.kind {
        ExprTypeKind::BitVector | ExprTypeKind::IntegerLike(_) | ExprTypeKind::EnumCore => {
            let Some(previous_timestamp) = previous_timestamp else {
                return Ok(false);
            };
            let previous_bits = sample_signal_bits(host, handle, previous_timestamp, cache)?;
            let current_bits = sample_signal_bits(host, handle, current_timestamp, cache)?;
            Ok(
                matches!((previous_bits, current_bits), (Some(previous), Some(current)) if previous != current),
            )
        }
        ExprTypeKind::Real => {
            let Some(previous_timestamp) = previous_timestamp else {
                return Ok(false);
            };
            let previous = sample_real_value(host, handle, previous_timestamp, cache)?;
            let current = sample_real_value(host, handle, current_timestamp, cache)?;
            Ok(
                matches!((previous, current), (Some(previous), Some(current)) if previous != current),
            )
        }
        ExprTypeKind::String => {
            let Some(previous_timestamp) = previous_timestamp else {
                return Ok(false);
            };
            let previous = sample_string_value(host, handle, previous_timestamp, cache)?;
            let current = sample_string_value(host, handle, current_timestamp, cache)?;
            Ok(
                matches!((previous, current), (Some(previous), Some(current)) if previous != current),
            )
        }
        ExprTypeKind::Event => cache.event_occurred(host, handle, current_timestamp),
    }
}

fn sample_signal_bits(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<Option<Rc<[BoundBit]>>, ExprDiagnostic> {
    match cache.sample_value(host, handle, timestamp)? {
        CachedSample::Integral { bits, .. } => Ok(bits),
        _ => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "event matching requires integral sampled values for edge and change detection",
        )),
    }
}

fn sample_real_value(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<Option<f64>, ExprDiagnostic> {
    match cache.sample_value(host, handle, timestamp)? {
        CachedSample::Real { value } => Ok(value),
        _ => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "event matching expected a real sampled value",
        )),
    }
}

fn sample_string_value(
    host: &dyn ExpressionHost,
    handle: SignalHandle,
    timestamp: u64,
    cache: &mut EvalCache,
) -> Result<Option<String>, ExprDiagnostic> {
    match cache.sample_value(host, handle, timestamp)? {
        CachedSample::String { value } => Ok(value.map(|value| value.to_string())),
        _ => Err(runtime_diag(
            "HOST-TYPE-MISMATCH",
            "event matching expected a string sampled value",
        )),
    }
}

fn classify_edge_bits(previous_bits: &[BoundBit], current_bits: &[BoundBit]) -> (bool, bool) {
    let Some(previous_lsb) = previous_bits.last().copied() else {
        return (false, false);
    };
    let Some(current_lsb) = current_bits.last().copied() else {
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

fn normalize_edge_bit(bit: BoundBit) -> char {
    match bit {
        BoundBit::Zero => '0',
        BoundBit::One => '1',
        BoundBit::Z => 'z',
        BoundBit::X => 'x',
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
                enum_labels: None,
            })
        }

        fn sample_value(
            &self,
            handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<SampledValue, ExprDiagnostic> {
            self.sampled.borrow_mut().push(handle);
            Ok(SampledValue::Integral {
                bits: Some("1".to_string()),
                label: None,
            })
        }

        fn event_occurred(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<bool, ExprDiagnostic> {
            Ok(false)
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
                enum_labels: None,
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
                    enum_labels: None,
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
                            enum_labels: None,
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
        assert!(matches!(
            value.payload,
            ExprValuePayload::Integral { ref bits, .. } if bits == "0"
        ));
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
                    enum_labels: None,
                },
                payload: RuntimeValuePayload::Integral {
                    bits: vec![BoundBit::X, BoundBit::One],
                    label: None,
                },
            },
            RuntimeValue {
                ty: ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: ExprStorage::PackedVector,
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                payload: RuntimeValuePayload::Integral {
                    bits: vec![BoundBit::Zero, BoundBit::One],
                    label: None,
                },
            },
        )
        .expect("wild equality should evaluate");
        assert_eq!(truth, TruthValue::Unknown);
    }
}
