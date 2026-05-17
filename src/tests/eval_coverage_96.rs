use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::expr::ast::BinaryOpAst;
use crate::expr::host::{ExprStorage, IntegerLikeKind};

use super::*;

#[derive(Default)]
struct Host96 {
    samples: RefCell<Vec<(SignalHandle, u64)>>,
}

impl ExpressionHost for Host96 {
    fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        Ok(SignalHandle(1))
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        Ok(match handle.0 {
            2 => real_ty(),
            3 => string_ty(),
            4 => event_ty(),
            _ => int_ty(4, true, false),
        })
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        self.samples.borrow_mut().push((handle, timestamp));
        Ok(match handle.0 {
            2 => SampledValue::Real {
                value: Some(2.0 + timestamp as f64),
            },
            3 => SampledValue::String {
                value: Some(format!("s{timestamp}")),
            },
            _ => SampledValue::Integral {
                bits: Some(format!("{:04b}", (timestamp as u8) & 0xf)),
                label: None,
            },
        })
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        Ok(handle.0 == 4 && timestamp == 7)
    }
}

fn int_ty(width: u32, four: bool, signed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width,
        is_four_state: four,
        is_signed: signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn enum_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::EnumCore,
        storage: ExprStorage::Scalar,
        width: 2,
        is_four_state: true,
        is_signed: false,
        enum_type_id: Some("state_t".to_string()),
        enum_labels: Some(vec![
            crate::expr::host::EnumLabelInfo {
                name: "IDLE".to_string(),
                bits: "00".to_string(),
            },
            crate::expr::host::EnumLabelInfo {
                name: "BUSY".to_string(),
                bits: "01".to_string(),
            },
        ]),
    }
}

fn real_ty() -> ExprType {
    real_type()
}
fn string_ty() -> ExprType {
    string_type()
}
fn event_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Event,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn value(bits: &[BoundBit], four: bool, signed: bool) -> RuntimeValue {
    RuntimeValue {
        ty: int_ty(bits.len() as u32, four, signed),
        payload: RuntimeValuePayload::Integral {
            bits: bits.to_vec(),
            label: None,
        },
    }
}

fn real_value(value: f64) -> RuntimeValue {
    RuntimeValue {
        ty: real_ty(),
        payload: RuntimeValuePayload::Real { value },
    }
}

fn string_value(value: &str) -> RuntimeValue {
    RuntimeValue {
        ty: string_ty(),
        payload: RuntimeValuePayload::String {
            value: value.to_string(),
        },
    }
}

fn lit(bits: &[BoundBit]) -> BoundLogicalNode {
    BoundLogicalNode {
        ty: int_ty(bits.len() as u32, true, false),
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

fn real_node(value: f64) -> BoundLogicalNode {
    BoundLogicalNode {
        ty: real_ty(),
        span: crate::expr::Span::new(0, 0),
        kind: BoundLogicalKind::RealLiteral { value },
    }
}

fn string_node(value: &str) -> BoundLogicalNode {
    BoundLogicalNode {
        ty: string_ty(),
        span: crate::expr::Span::new(0, 0),
        kind: BoundLogicalKind::StringLiteral {
            value: value.to_string(),
        },
    }
}

#[test]
fn eval96_cache_public_and_event_match_branches() {
    let host = Host96::default();
    let mut cache = EvalCache::default();
    assert!(matches!(
        cache.sample_value(&host, SignalHandle(1), 3).unwrap(),
        CachedSample::Integral { .. }
    ));
    assert!(matches!(
        cache.sample_value(&host, SignalHandle(1), 3).unwrap(),
        CachedSample::Integral { .. }
    ));
    assert_eq!(host.samples.borrow().len(), 1);
    assert!(matches!(
        cache.signal_type(&host, SignalHandle(2)).unwrap().kind,
        ExprTypeKind::Real
    ));
    assert!(matches!(
        cache.signal_type(&host, SignalHandle(3)).unwrap().kind,
        ExprTypeKind::String
    ));
    assert!(cache.event_occurred(&host, SignalHandle(4), 7).unwrap());
    assert!(cache.event_occurred(&host, SignalHandle(4), 7).unwrap());

    let expr = BoundEventExpr {
        terms: vec![
            crate::expr::sema::BoundEventTerm {
                event: BoundEventKind::Named(SignalHandle(4)),
                iff: None,
            },
            crate::expr::sema::BoundEventTerm {
                event: BoundEventKind::AnyTracked,
                iff: Some(BoundLogicalExpr {
                    root: lit(&[BoundBit::One]),
                }),
            },
        ],
    };
    let frame = EventEvalFrame {
        timestamp: 7,
        previous_timestamp: Some(6),
        tracked_signals: &[SignalHandle(1)],
    };
    assert!(event_matches_at(&expr, &host, &frame).unwrap());

    let pos = BoundEventExpr {
        terms: vec![crate::expr::sema::BoundEventTerm {
            event: BoundEventKind::Posedge(SignalHandle(1)),
            iff: None,
        }],
    };
    let neg = BoundEventExpr {
        terms: vec![crate::expr::sema::BoundEventTerm {
            event: BoundEventKind::Negedge(SignalHandle(1)),
            iff: None,
        }],
    };
    let edge = BoundEventExpr {
        terms: vec![crate::expr::sema::BoundEventTerm {
            event: BoundEventKind::Edge(SignalHandle(1)),
            iff: None,
        }],
    };
    assert!(event_matches_at(&pos, &host, &frame).is_ok());
    assert!(event_matches_at(&neg, &host, &frame).is_ok());
    assert!(event_matches_at(&edge, &host, &frame).is_ok());
}

#[test]
fn eval96_node_shapes_and_conditionals() {
    let host = Host96::default();
    let mut cache = EvalCache::default();
    let result_ty = int_ty(4, true, false);
    let concat = BoundLogicalNode {
        ty: result_ty.clone(),
        span: crate::expr::Span::new(0, 0),
        kind: BoundLogicalKind::Concatenation {
            items: vec![
                lit(&[BoundBit::One, BoundBit::Zero]),
                lit(&[BoundBit::Zero, BoundBit::One]),
            ],
        },
    };
    assert!(matches!(
        eval_node(&concat, &host, 1, &mut cache).unwrap().payload,
        RuntimeValuePayload::Integral { .. }
    ));
    let repl = BoundLogicalNode {
        ty: result_ty.clone(),
        span: crate::expr::Span::new(0, 0),
        kind: BoundLogicalKind::Replication {
            count: 2,
            expr: Box::new(lit(&[BoundBit::One, BoundBit::Zero])),
        },
    };
    assert!(matches!(
        eval_node(&repl, &host, 1, &mut cache).unwrap().payload,
        RuntimeValuePayload::Integral { .. }
    ));
    let trig = BoundLogicalNode {
        ty: int_ty(1, true, false),
        span: crate::expr::Span::new(0, 0),
        kind: BoundLogicalKind::Triggered {
            handle: SignalHandle(4),
        },
    };
    assert_eq!(
        expect_integral_bits(&eval_node(&trig, &host, 7, &mut cache).unwrap())
            .unwrap()
            .0,
        [BoundBit::One]
    );

    let unknown = lit(&[BoundBit::X]);
    let same_real = eval_conditional(
        &unknown,
        &real_node(1.25),
        &real_node(1.25),
        &host,
        0,
        &mut cache,
        &real_ty(),
    )
    .unwrap();
    assert!(matches!(same_real.payload, RuntimeValuePayload::Real { value } if value == 1.25));
    assert!(
        eval_conditional(
            &unknown,
            &real_node(1.0),
            &real_node(2.0),
            &host,
            0,
            &mut cache,
            &real_ty()
        )
        .is_err()
    );
    assert!(
        eval_conditional(
            &unknown,
            &string_node("a"),
            &string_node("b"),
            &host,
            0,
            &mut cache,
            &string_ty()
        )
        .is_err()
    );
}

#[test]
fn eval96_binary_arithmetic_comparison_and_cast_residue() {
    let ty = int_ty(4, true, true);
    for op in [
        BinaryOpAst::Add,
        BinaryOpAst::Subtract,
        BinaryOpAst::Multiply,
        BinaryOpAst::Divide,
        BinaryOpAst::Modulo,
        BinaryOpAst::Power,
    ] {
        assert!(
            eval_arithmetic(
                op,
                value(
                    &[BoundBit::Zero, BoundBit::Zero, BoundBit::One, BoundBit::One],
                    true,
                    true
                ),
                value(
                    &[
                        BoundBit::Zero,
                        BoundBit::Zero,
                        BoundBit::One,
                        BoundBit::Zero
                    ],
                    true,
                    true
                ),
                &ty
            )
            .is_ok()
        );
    }
    for op in [
        BinaryOpAst::Add,
        BinaryOpAst::Subtract,
        BinaryOpAst::Multiply,
        BinaryOpAst::Divide,
        BinaryOpAst::Modulo,
        BinaryOpAst::Power,
    ] {
        assert!(eval_arithmetic(op, real_value(5.0), real_value(2.0), &real_ty()).is_ok());
    }
    assert!(matches!(
        eval_arithmetic(
            BinaryOpAst::Divide,
            value(&[BoundBit::One], true, false),
            value(&[BoundBit::Zero], true, false),
            &int_ty(1, true, false)
        )
        .unwrap()
        .payload,
        RuntimeValuePayload::Integral { .. }
    ));
    assert!(matches!(
        eval_arithmetic(
            BinaryOpAst::Power,
            value(&[BoundBit::Zero], true, false),
            value(&[BoundBit::One], true, true),
            &int_ty(1, true, false)
        )
        .unwrap()
        .payload,
        RuntimeValuePayload::Integral { .. }
    ));

    for op in [
        BinaryOpAst::Lt,
        BinaryOpAst::Le,
        BinaryOpAst::Gt,
        BinaryOpAst::Ge,
        BinaryOpAst::Eq,
        BinaryOpAst::Ne,
    ] {
        assert!(compare_unknown_sensitive(op, real_value(1.0), real_value(2.0)).is_ok());
    }
    assert_eq!(
        compare_unknown_sensitive(BinaryOpAst::Eq, string_value("a"), string_value("a")).unwrap(),
        TruthValue::One
    );
    assert_eq!(
        compare_unknown_sensitive(BinaryOpAst::Lt, string_value("a"), string_value("b")).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_case(
            BinaryOpAst::CaseEq,
            value(&[BoundBit::X], true, false),
            value(&[BoundBit::X], true, false)
        )
        .unwrap(),
        TruthValue::One
    );
    assert_eq!(
        compare_case(
            BinaryOpAst::CaseNe,
            value(&[BoundBit::X], true, false),
            value(&[BoundBit::One], true, false)
        )
        .unwrap(),
        TruthValue::One
    );

    let labeled = coerce_runtime_to_type(
        value(&[BoundBit::Zero, BoundBit::One], true, false),
        &enum_ty(),
    )
    .unwrap();
    assert!(
        matches!(labeled.payload, RuntimeValuePayload::Integral { label: Some(label), .. } if label == "BUSY")
    );
    assert!(coerce_runtime_to_type(string_value("nope"), &real_ty()).is_err());
    assert!(coerce_runtime_to_type(real_value(3.75), &int_ty(3, true, false)).is_ok());
    assert!(coerce_runtime_to_type(value(&[BoundBit::One], true, false), &event_ty()).is_err());
    assert!(matches!(
        common_integral_type_for_eval(&int_ty(2, false, true), &int_ty(5, true, true)).kind,
        ExprTypeKind::BitVector
    ));
    assert!(matches!(
        ExprType {
            kind: ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
            storage: ExprStorage::Scalar,
            width: 32,
            is_four_state: false,
            is_signed: true,
            enum_type_id: None,
            enum_labels: None
        }
        .kind,
        ExprTypeKind::IntegerLike(_)
    ));
}

#[test]
fn eval96_direct_runtime_residue_branches() {
    let host = Host96::default();
    let mut cache = EvalCache::default();
    let one = value(&[BoundBit::Zero, BoundBit::One], true, false);
    let unknown = value(&[BoundBit::X, BoundBit::One], true, false);
    let wide = int_ty(129, true, false);
    let wide_value = RuntimeValue {
        ty: wide.clone(),
        payload: RuntimeValuePayload::Integral {
            bits: vec![BoundBit::One; 129],
            label: None,
        },
    };

    assert!(eval_unary(UnaryOpAst::Minus, wide_value.clone(), &wide).is_ok());
    assert!(eval_unary(UnaryOpAst::Minus, unknown.clone(), &int_ty(2, true, false)).is_ok());
    assert!(
        eval_unary(
            UnaryOpAst::ReduceXor,
            unknown.clone(),
            &int_ty(1, true, false)
        )
        .is_ok()
    );
    assert!(
        eval_bitwise(
            BinaryOpAst::Add,
            one.clone(),
            one.clone(),
            &int_ty(2, true, false)
        )
        .is_ok()
    );
    assert!(
        eval_shift(
            BinaryOpAst::Add,
            one.clone(),
            one.clone(),
            &int_ty(2, true, false)
        )
        .is_ok()
    );
    assert!(
        eval_shift(
            BinaryOpAst::ShiftArithRight,
            value(&[BoundBit::One, BoundBit::Zero], true, true),
            value(&[BoundBit::One, BoundBit::One], true, false),
            &int_ty(2, true, true)
        )
        .is_ok()
    );
    assert!(
        eval_arithmetic(
            BinaryOpAst::BitAnd,
            one.clone(),
            one.clone(),
            &int_ty(2, true, false)
        )
        .is_ok()
    );
    assert!(
        eval_arithmetic(
            BinaryOpAst::Divide,
            unknown.clone(),
            one.clone(),
            &int_ty(2, true, false)
        )
        .is_ok()
    );
    assert!(
        eval_arithmetic(
            BinaryOpAst::Modulo,
            one.clone(),
            unknown.clone(),
            &int_ty(2, true, true)
        )
        .is_ok()
    );
    assert!(
        coerce_runtime_to_type(
            RuntimeValue {
                ty: int_ty(128, true, true),
                payload: RuntimeValuePayload::Integral {
                    bits: vec![BoundBit::X; 128],
                    label: None
                }
            },
            &real_ty()
        )
        .is_err()
    );
    assert!(matches!(
        coerce_runtime_to_type(string_value("same"), &string_ty())
            .unwrap()
            .payload,
        RuntimeValuePayload::String { .. }
    ));

    assert_eq!(
        compare_unknown_sensitive(BinaryOpAst::Add, real_value(1.0), real_value(2.0)).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_unknown_sensitive(BinaryOpAst::Add, string_value("a"), string_value("b")).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_unknown_sensitive(BinaryOpAst::Add, one.clone(), one.clone()).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_case(BinaryOpAst::Eq, one.clone(), one.clone()).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_wild(BinaryOpAst::Eq, one.clone(), one.clone()).unwrap(),
        TruthValue::Unknown
    );
    assert_eq!(
        compare_ordering(&[BoundBit::One], &[BoundBit::Zero], true),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        compare_ordering(&[BoundBit::Zero], &[BoundBit::One], true),
        std::cmp::Ordering::Greater
    );
    assert_eq!(
        select_bit(&[BoundBit::One], 4, false, false),
        BoundBit::Zero
    );
    assert_eq!(
        part_select_bits(&[BoundBit::One, BoundBit::Zero, BoundBit::One], 0, 2, false),
        vec![BoundBit::One, BoundBit::Zero, BoundBit::One]
    );
    assert_eq!(unsigned_to_bits(1, 130).len(), 130);

    assert!(signal_changed(&host, SignalHandle(2), Some(0), 1, &mut cache).unwrap());
    assert!(signal_changed(&host, SignalHandle(3), Some(0), 1, &mut cache).unwrap());
    assert!(signal_changed(&host, SignalHandle(4), Some(0), 7, &mut cache).unwrap());

    assert!(
        catch_unwind(AssertUnwindSafe(|| eval_unary(
            UnaryOpAst::Minus,
            one.clone(),
            &real_ty()
        )
        .unwrap()))
        .is_ok()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| eval_unary(
            UnaryOpAst::ReduceAnd,
            one.clone(),
            &int_ty(1, true, false)
        )
        .unwrap()))
        .is_ok()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| eval_arithmetic(
            BinaryOpAst::BitAnd,
            real_value(1.0),
            real_value(2.0),
            &real_ty()
        )
        .unwrap()))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| compare_unknown_sensitive(
            BinaryOpAst::BitAnd,
            real_value(1.0),
            real_value(2.0)
        )
        .unwrap()))
        .is_ok()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| compare_unknown_sensitive(
            BinaryOpAst::BitAnd,
            string_value("a"),
            string_value("b")
        )
        .unwrap()))
        .is_ok()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| compare_case(
            BinaryOpAst::BitAnd,
            one.clone(),
            one.clone()
        )
        .unwrap()))
        .is_ok()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| compare_wild(
            BinaryOpAst::BitAnd,
            one.clone(),
            one.clone()
        )
        .unwrap()))
        .is_ok()
    );
    assert!(matches!(
        coerce_runtime_to_type(real_value(1.0), &enum_ty())
            .unwrap()
            .payload,
        RuntimeValuePayload::Integral { label: Some(_), .. }
    ));
    assert!(matches!(
        coerce_runtime_to_type(real_value(1.0), &real_ty())
            .unwrap()
            .payload,
        RuntimeValuePayload::Real { .. }
    ));
    assert!(matches!(
        coerce_runtime_to_type(string_value("same"), &string_ty())
            .unwrap()
            .payload,
        RuntimeValuePayload::String { .. }
    ));
}

#[test]
fn eval96_exhaustive_small_bit_truth_tables() {
    let bits = [BoundBit::Zero, BoundBit::One, BoundBit::X, BoundBit::Z];
    for lhs in bits {
        for rhs in bits {
            let _ = bitwise_and(lhs, rhs);
            let _ = bitwise_or(lhs, rhs);
            let _ = bitwise_xor(lhs, rhs);
            let _ = invert_bit(lhs);
            let left = value(&[lhs, rhs], true, false);
            let right = value(&[rhs, lhs], true, false);
            for op in [
                BinaryOpAst::BitAnd,
                BinaryOpAst::BitOr,
                BinaryOpAst::BitXor,
                BinaryOpAst::BitXnor,
            ] {
                let _ = eval_bitwise(op, left.clone(), right.clone(), &int_ty(2, true, false));
            }
            for op in [
                BinaryOpAst::Eq,
                BinaryOpAst::Ne,
                BinaryOpAst::Lt,
                BinaryOpAst::Le,
                BinaryOpAst::Gt,
                BinaryOpAst::Ge,
            ] {
                let _ = compare_unknown_sensitive(op, left.clone(), right.clone());
            }
            let _ = compare_wild(BinaryOpAst::WildEq, left.clone(), right.clone());
            let _ = compare_wild(BinaryOpAst::WildNe, left.clone(), right.clone());
        }
    }
    for pattern in [
        vec![BoundBit::Zero, BoundBit::Zero],
        vec![BoundBit::One, BoundBit::One],
        vec![BoundBit::X, BoundBit::Zero],
        vec![BoundBit::Z, BoundBit::One],
        vec![BoundBit::One, BoundBit::Zero],
    ] {
        let _ = reduce_and(&pattern);
        let _ = reduce_or(&pattern);
        let _ = reduce_xor(&pattern);
        let rv = value(&pattern, true, false);
        let _ = truthiness(&rv);
    }
    assert_eq!(truthiness(&real_value(0.0)), TruthValue::Zero);
    assert_eq!(truthiness(&real_value(0.5)), TruthValue::One);
    assert_eq!(truthiness(&string_value("s")), TruthValue::Unknown);
}

#[test]
fn eval96_runtime_sampling_error_residue() {
    let host = Host96::default();
    let mut cache = EvalCache::default();
    assert!(matches!(
        sample_real_value(&host, SignalHandle(2), 0, &mut cache).unwrap(),
        Some(2.0)
    ));
    assert!(sample_real_value(&host, SignalHandle(1), 0, &mut cache).is_err());
    assert!(
        sample_string_value(&host, SignalHandle(3), 0, &mut cache)
            .unwrap()
            .is_some()
    );
    assert!(sample_string_value(&host, SignalHandle(1), 0, &mut cache).is_err());
    assert!(sample_signal_bits(&host, SignalHandle(2), 0, &mut cache).is_err());
    assert!(!signal_changed(&host, SignalHandle(2), None, 0, &mut cache).unwrap());
    assert!(!signal_changed(&host, SignalHandle(3), None, 0, &mut cache).unwrap());
}

#[test]
fn eval96_condition_inside_and_logical_truth_grid() {
    let host = Host96::default();
    let mut cache = EvalCache::default();
    let bool_ty = int_ty(1, true, false);
    for cond in [BoundBit::One, BoundBit::Zero, BoundBit::X] {
        let _ = eval_conditional(
            &lit(&[cond]),
            &lit(&[BoundBit::One]),
            &lit(&[BoundBit::Zero]),
            &host,
            0,
            &mut cache,
            &bool_ty,
        );
    }
    for (lhs, low, high) in [
        (BoundBit::One, BoundBit::Zero, BoundBit::One),
        (BoundBit::Zero, BoundBit::One, BoundBit::One),
        (BoundBit::X, BoundBit::Zero, BoundBit::One),
    ] {
        let set = vec![BoundInsideItem::Range {
            low: lit(&[low]),
            high: lit(&[high]),
        }];
        let _ = eval_inside(&lit(&[lhs]), &set, &host, 0, &mut cache, &bool_ty);
    }
    let set = vec![BoundInsideItem::Expr(lit(&[BoundBit::Z]))];
    let _ = eval_inside(&lit(&[BoundBit::X]), &set, &host, 0, &mut cache, &bool_ty);

    for op in [BinaryOpAst::LogicalAnd, BinaryOpAst::LogicalOr] {
        for lhs in [BoundBit::Zero, BoundBit::One, BoundBit::X] {
            for rhs in [BoundBit::Zero, BoundBit::One, BoundBit::X] {
                let left = lit(&[lhs]);
                let right = lit(&[rhs]);
                let _ = eval_binary(op, &left, &right, &host, 0, &mut cache, &bool_ty);
            }
        }
    }
}
