use crate::expr::Span;
use crate::expr::ast::{BinaryOpAst, UnaryOpAst};
use crate::expr::host::{ExprStorage, ExprType, ExprTypeKind, ExpressionHost, SignalHandle};

use super::*;

fn ty(width: u32) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width,
        is_four_state: true,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn bits(value: &[BoundBit]) -> BoundIntegralValue {
    BoundIntegralValue {
        bits: value.to_vec(),
        signed: false,
    }
}

fn node(kind: BoundLogicalKind) -> BoundLogicalNode {
    BoundLogicalNode {
        ty: ty(1),
        span: Span::new(0, 1),
        kind,
    }
}

fn lit(bit: BoundBit) -> BoundLogicalNode {
    node(BoundLogicalKind::IntegralLiteral {
        value: bits(&[bit]),
        is_unsized: false,
    })
}

#[test]
fn sema96_bound_tree_derive_surface() {
    let event_kinds = [
        BoundEventKind::AnyTracked,
        BoundEventKind::Named(SignalHandle(1)),
        BoundEventKind::Posedge(SignalHandle(2)),
        BoundEventKind::Negedge(SignalHandle(3)),
        BoundEventKind::Edge(SignalHandle(4)),
    ];
    for kind in event_kinds {
        assert_eq!(kind.clone(), kind);
        assert!(format!("{kind:?}").len() > 3);
    }

    let term = BoundEventTerm {
        event: BoundEventKind::Named(SignalHandle(1)),
        iff: Some(BoundLogicalExpr {
            root: lit(BoundBit::One),
        }),
    };
    assert_eq!(term.clone(), term);
    assert!(format!("{term:?}").contains("Named"));
    let event = BoundEventExpr { terms: vec![term] };
    assert_eq!(event.clone(), event);
    assert!(format!("{event:?}").contains("terms"));

    for cast in [
        BoundCastKind::Signed,
        BoundCastKind::Unsigned,
        BoundCastKind::Static,
    ] {
        assert_eq!(cast.clone(), cast);
        assert!(format!("{cast:?}").len() > 3);
    }

    for bit in [BoundBit::Zero, BoundBit::One, BoundBit::X, BoundBit::Z] {
        assert_eq!(bit.clone(), bit);
        assert!(!format!("{bit:?}").is_empty());
    }
    let integral = bits(&[BoundBit::Zero, BoundBit::One]);
    assert_eq!(integral.clone(), integral);
    assert!(format!("{integral:?}").contains("signed"));

    let base = lit(BoundBit::One);
    let variants = vec![
        BoundLogicalKind::SignalRef {
            handle: SignalHandle(9),
        },
        BoundLogicalKind::RealLiteral { value: 1.25 },
        BoundLogicalKind::StringLiteral {
            value: "s".to_string(),
        },
        BoundLogicalKind::EnumLabel {
            value: bits(&[BoundBit::Zero]),
            label: "IDLE".to_string(),
        },
        BoundLogicalKind::Parenthesized {
            expr: Box::new(base.clone()),
        },
        BoundLogicalKind::Cast {
            kind: BoundCastKind::Signed,
            expr: Box::new(base.clone()),
        },
        BoundLogicalKind::Selection {
            base: Box::new(base.clone()),
            selection: BoundSelection::Bit {
                index: Box::new(lit(BoundBit::Zero)),
            },
        },
        BoundLogicalKind::Unary {
            op: UnaryOpAst::BitNot,
            expr: Box::new(base.clone()),
        },
        BoundLogicalKind::Binary {
            op: BinaryOpAst::BitAnd,
            left: Box::new(base.clone()),
            right: Box::new(base.clone()),
        },
        BoundLogicalKind::Conditional {
            condition: Box::new(base.clone()),
            when_true: Box::new(base.clone()),
            when_false: Box::new(base.clone()),
        },
        BoundLogicalKind::Inside {
            expr: Box::new(base.clone()),
            set: vec![
                BoundInsideItem::Expr(base.clone()),
                BoundInsideItem::Range {
                    low: lit(BoundBit::Zero),
                    high: lit(BoundBit::One),
                },
            ],
        },
        BoundLogicalKind::Concatenation {
            items: vec![base.clone()],
        },
        BoundLogicalKind::Replication {
            count: 2,
            expr: Box::new(base.clone()),
        },
        BoundLogicalKind::Triggered {
            handle: SignalHandle(5),
        },
    ];
    for kind in variants {
        assert_eq!(kind.clone(), kind);
        let n = node(kind);
        assert_eq!(n.clone(), n);
        assert_eq!(n.span(), Span::new(0, 1));
        assert!(format!("{n:?}").contains("ty"));
    }

    let selections = vec![
        BoundSelection::Part { msb: 3, lsb: 0 },
        BoundSelection::IndexedUp {
            base: Box::new(lit(BoundBit::Zero)),
            width: 2,
        },
        BoundSelection::IndexedDown {
            base: Box::new(lit(BoundBit::One)),
            width: 2,
        },
    ];
    for selection in selections {
        assert_eq!(selection.clone(), selection);
        assert!(format!("{selection:?}").len() > 3);
    }
}

#[test]
fn sema96_const_and_type_helper_residue_branches() {
    let unsigned4 = bit_vector_type(4, false, false, true);
    let signed4 = bit_vector_type(4, true, true, true);
    let one = BoundIntegralValue {
        bits: vec![
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::One,
        ],
        signed: false,
    };
    let two = BoundIntegralValue {
        bits: vec![
            BoundBit::Zero,
            BoundBit::Zero,
            BoundBit::One,
            BoundBit::Zero,
        ],
        signed: false,
    };
    let xval = BoundIntegralValue {
        bits: vec![BoundBit::X, BoundBit::One],
        signed: false,
    };

    assert!(part_select_width(i64::MAX, i64::MIN, Span::new(0, 1)).is_err());
    assert!(
        indexed_part_select_may_produce_x(&unsigned4, &lit(BoundBit::One), usize::MAX, true)
            .unwrap()
    );
    assert!(
        !indexed_part_select_may_produce_x(&unsigned4, &lit(BoundBit::One), usize::MAX, false)
            .unwrap()
    );
    assert!(!selection_range_is_in_bounds(4, -1, 0));

    assert!(
        eval_const_i64(
            &node(BoundLogicalKind::SignalRef {
                handle: SignalHandle(1)
            }),
            "ctx",
            Span::new(0, 1)
        )
        .is_err()
    );
    assert!(eval_const_i64(&lit(BoundBit::X), "ctx", Span::new(0, 1)).is_err());
    assert_eq!(try_eval_const_i64(&lit(BoundBit::X)).unwrap(), None);

    let cast_non_integral = BoundLogicalNode {
        ty: real_type(),
        span: Span::new(0, 1),
        kind: BoundLogicalKind::Cast {
            kind: BoundCastKind::Static,
            expr: Box::new(lit(BoundBit::One)),
        },
    };
    assert!(eval_const_node(&cast_non_integral).unwrap().is_none());
    let parent_non_const = node(BoundLogicalKind::Parenthesized {
        expr: Box::new(node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })),
    });
    assert!(eval_const_node(&parent_non_const).unwrap().is_none());
    let unary_non_const = node(BoundLogicalKind::Unary {
        op: UnaryOpAst::Plus,
        expr: Box::new(node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })),
    });
    assert!(eval_const_node(&unary_non_const).unwrap().is_none());
    let binary_non_const_left = node(BoundLogicalKind::Binary {
        op: BinaryOpAst::Add,
        left: Box::new(node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })),
        right: Box::new(lit(BoundBit::One)),
    });
    assert!(eval_const_node(&binary_non_const_left).unwrap().is_none());
    let binary_non_const_right = node(BoundLogicalKind::Binary {
        op: BinaryOpAst::Add,
        left: Box::new(lit(BoundBit::One)),
        right: Box::new(node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })),
    });
    assert!(eval_const_node(&binary_non_const_right).unwrap().is_none());
    let cond_unknown = node(BoundLogicalKind::Conditional {
        condition: Box::new(lit(BoundBit::X)),
        when_true: Box::new(lit(BoundBit::One)),
        when_false: Box::new(lit(BoundBit::Zero)),
    });
    assert!(matches!(
        eval_const_node(&cond_unknown).unwrap().unwrap().bits[0],
        BoundBit::X
    ));
    let concat_non_const = node(BoundLogicalKind::Concatenation {
        items: vec![node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })],
    });
    assert!(eval_const_node(&concat_non_const).unwrap().is_none());
    let repl_non_const = node(BoundLogicalKind::Replication {
        count: 3,
        expr: Box::new(node(BoundLogicalKind::SignalRef {
            handle: SignalHandle(1),
        })),
    });
    assert!(eval_const_node(&repl_non_const).unwrap().is_none());
    assert!(
        eval_const_node(&node(BoundLogicalKind::Inside {
            expr: Box::new(lit(BoundBit::One)),
            set: vec![]
        }))
        .unwrap()
        .is_none()
    );
    assert!(
        eval_const_node(&node(BoundLogicalKind::Triggered {
            handle: SignalHandle(1)
        }))
        .unwrap()
        .is_none()
    );

    assert!(
        eval_const_unary(UnaryOpAst::Minus, xval.clone(), &signed4)
            .bits
            .iter()
            .all(|bit| matches!(bit, BoundBit::X))
    );
    assert_eq!(
        eval_const_unary(
            UnaryOpAst::Minus,
            one.clone(),
            &bit_vector_type(129, true, false, true)
        )
        .bits
        .len(),
        129
    );
    assert!(
        eval_const_unary(UnaryOpAst::BitNot, one.clone(), &signed4)
            .bits
            .contains(&BoundBit::Zero)
    );
    for op in [
        UnaryOpAst::ReduceAnd,
        UnaryOpAst::ReduceNand,
        UnaryOpAst::ReduceOr,
        UnaryOpAst::ReduceNor,
        UnaryOpAst::ReduceXor,
        UnaryOpAst::ReduceXnor,
    ] {
        assert_eq!(eval_const_unary(op, one.clone(), &signed4).bits.len(), 1);
    }

    for op in [
        BinaryOpAst::Add,
        BinaryOpAst::Subtract,
        BinaryOpAst::Multiply,
        BinaryOpAst::Divide,
        BinaryOpAst::Modulo,
        BinaryOpAst::Power,
        BinaryOpAst::ShiftLeft,
        BinaryOpAst::ShiftRight,
        BinaryOpAst::ShiftArithLeft,
        BinaryOpAst::ShiftArithRight,
        BinaryOpAst::BitAnd,
        BinaryOpAst::BitOr,
        BinaryOpAst::BitXor,
        BinaryOpAst::BitXnor,
    ] {
        assert!(
            !eval_const_binary(op, one.clone(), two.clone(), &signed4)
                .bits
                .is_empty()
        );
    }
    assert!(
        eval_const_binary(
            BinaryOpAst::Divide,
            one.clone(),
            BoundIntegralValue {
                bits: vec![BoundBit::Zero; 4],
                signed: true
            },
            &signed4
        )
        .bits
        .iter()
        .all(|bit| matches!(bit, BoundBit::X))
    );
    assert!(
        eval_const_binary(
            BinaryOpAst::Modulo,
            one.clone(),
            BoundIntegralValue {
                bits: vec![BoundBit::Zero; 4],
                signed: true
            },
            &signed4
        )
        .bits
        .iter()
        .all(|bit| matches!(bit, BoundBit::X))
    );
    assert!(
        eval_const_binary(
            BinaryOpAst::Power,
            BoundIntegralValue {
                bits: vec![BoundBit::Zero; 4],
                signed: true
            },
            BoundIntegralValue {
                bits: vec![BoundBit::One; 4],
                signed: true
            },
            &signed4
        )
        .bits
        .iter()
        .all(|bit| matches!(bit, BoundBit::X))
    );
    assert_eq!(
        eval_const_binary(BinaryOpAst::Lt, one.clone(), two.clone(), &signed4)
            .bits
            .len(),
        1
    );

    assert!(
        ensure_cast_compatible(&string_type(), &ExprTypeKind::BitVector, Span::new(0, 1)).is_err()
    );
    assert!(ensure_real_cast_source(&string_type(), Span::new(0, 1)).is_err());
    assert!(ensure_string_cast_source(&real_type(), Span::new(0, 1)).is_err());
    assert!(matches!(
        common_integral_type(
            &integer_like_type(IntegerLikeKind::Int),
            &integer_like_type(IntegerLikeKind::Int)
        )
        .kind,
        ExprTypeKind::IntegerLike(IntegerLikeKind::Int)
    ));
    assert!(matches!(
        common_numeric_result_type(&real_type(), &unsigned4).kind,
        ExprTypeKind::Real
    ));
    assert!(matches!(
        binary_result_type(
            BinaryOpAst::CaseEq,
            &unsigned4,
            Span::new(0, 1),
            &unsigned4,
            Span::new(1, 2)
        )
        .unwrap()
        .kind,
        ExprTypeKind::BitVector
    ));
    assert!(matches!(
        binary_result_type(
            BinaryOpAst::BitAnd,
            &unsigned4,
            Span::new(0, 1),
            &signed4,
            Span::new(1, 2)
        )
        .unwrap()
        .kind,
        ExprTypeKind::BitVector
    ));
}

#[test]
fn sema96_more_error_and_const_region_residue() {
    struct EmptyUnknownHost;
    impl ExpressionHost for EmptyUnknownHost {
        fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
            Err(ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: String::new(),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
        }

        fn signal_type(&self, _handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
            Ok(bit_vector_type(4, true, false, true))
        }

        fn sample_value(
            &self,
            _handle: SignalHandle,
            _timestamp: u64,
        ) -> Result<crate::expr::SampledValue, ExprDiagnostic> {
            Ok(crate::expr::SampledValue::Integral {
                bits: Some("0".to_string()),
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

    let enum_error = bind_logical_expr_ast(
        &crate::expr::parse_logical_expr_ast("type(missing)::BUSY").expect("parse enum label"),
        &EmptyUnknownHost,
    )
    .expect_err("missing enum operand should fail");
    assert!(enum_error.notes.is_empty());

    let span = Span::new(0, 1);
    let unsigned2 = bit_vector_type(2, true, false, true);
    let signed4 = bit_vector_type(4, true, true, true);
    let one = BoundIntegralValue {
        bits: vec![BoundBit::One],
        signed: false,
    };
    assert!(apply_const_cast(BoundCastKind::Signed, one.clone(), &signed4).signed);
    assert!(!apply_const_cast(BoundCastKind::Unsigned, one.clone(), &unsigned2).signed);
    assert_eq!(
        apply_const_cast(BoundCastKind::Static, one.clone(), &signed4)
            .bits
            .len(),
        4
    );
    assert_eq!(truthiness_bits(&[BoundBit::Zero]), ConstTruth::Zero);

    for (condition, expected) in [
        (BoundBit::One, BoundBit::One),
        (BoundBit::Zero, BoundBit::Zero),
    ] {
        let node = BoundLogicalNode {
            ty: bit_vector_type(1, true, false, false),
            span,
            kind: BoundLogicalKind::Conditional {
                condition: Box::new(lit(condition)),
                when_true: Box::new(lit(BoundBit::One)),
                when_false: Box::new(lit(BoundBit::Zero)),
            },
        };
        assert_eq!(
            eval_const_node(&node).unwrap().unwrap().bits,
            vec![expected]
        );
    }

    assert_eq!(
        eval_const_node(&BoundLogicalNode {
            ty: real_type(),
            span,
            kind: BoundLogicalKind::RealLiteral { value: 1.0 }
        })
        .unwrap(),
        None
    );
    assert_eq!(
        eval_const_node(&BoundLogicalNode {
            ty: string_type(),
            span,
            kind: BoundLogicalKind::StringLiteral {
                value: "s".to_string()
            }
        })
        .unwrap(),
        None
    );
    assert!(part_select_width(i64::MIN, i64::MAX, span).is_err());
    assert!(
        indexed_part_select_may_produce_x(&unsigned2, &lit(BoundBit::One), usize::MAX, true)
            .unwrap()
    );
    assert!(
        indexed_part_select_may_produce_x(&unsigned2, &lit(BoundBit::One), usize::MAX, false)
            .unwrap()
    );
}
