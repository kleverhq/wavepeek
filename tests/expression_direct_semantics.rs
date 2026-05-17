mod common;

use std::collections::HashMap;

use wavepeek::expr::{
    EnumLabelInfo, EventEvalFrame, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind,
    ExprValuePayload, ExpressionHost, IntegerLikeKind, SampledValue, SignalHandle,
    bind_event_expr_ast, bind_logical_expr_ast, eval_logical_expr_at, event_matches_at,
    parse_event_expr_ast, parse_logical_expr_ast,
};

#[derive(Clone)]
struct SignalDef {
    handle: SignalHandle,
    ty: ExprType,
    sample: SampledValue,
    previous_sample: Option<SampledValue>,
    event_now: bool,
}

#[derive(Default)]
struct RichHost {
    signals: HashMap<String, SignalDef>,
    by_handle: HashMap<SignalHandle, SignalDef>,
}

impl RichHost {
    fn with_signal(mut self, name: &str, def: SignalDef) -> Self {
        self.by_handle.insert(def.handle, def.clone());
        self.signals.insert(name.to_string(), def);
        self
    }
}

impl ExpressionHost for RichHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        self.signals
            .get(name)
            .map(|def| def.handle)
            .ok_or_else(|| host_diag("HOST-UNKNOWN-SIGNAL", &format!("unknown signal '{name}'")))
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        self.by_handle
            .get(&handle)
            .map(|def| def.ty.clone())
            .ok_or_else(|| host_diag("HOST-UNKNOWN-SIGNAL", "unknown handle"))
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        let def = self
            .by_handle
            .get(&handle)
            .ok_or_else(|| host_diag("HOST-UNKNOWN-SIGNAL", "unknown handle"))?;
        if timestamp == 0 {
            Ok(def
                .previous_sample
                .clone()
                .unwrap_or_else(|| def.sample.clone()))
        } else {
            Ok(def.sample.clone())
        }
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        let def = self
            .by_handle
            .get(&handle)
            .ok_or_else(|| host_diag("HOST-UNKNOWN-SIGNAL", "unknown handle"))?;
        Ok(timestamp > 0 && def.event_now)
    }
}

#[test]
fn direct_binding_and_evaluation_cover_rich_success_paths() {
    let host = rich_host();

    let cases = [
        (
            "type(state)::BUSY == state",
            ExprValuePayload::Integral {
                bits: "1".into(),
                label: None,
            },
        ),
        (
            "string'(msg)",
            ExprValuePayload::String {
                value: "hello".into(),
            },
        ),
        (
            "bit[4]'(real_sig + 1.5)",
            ExprValuePayload::Integral {
                bits: "0100".into(),
                label: None,
            },
        ),
        (
            "{2{flag}}",
            ExprValuePayload::Integral {
                bits: "11".into(),
                label: None,
            },
        ),
        (
            "real_sig ? 1 : 0",
            ExprValuePayload::Integral {
                bits: "01".into(),
                label: None,
            },
        ),
        (
            "ev.triggered()",
            ExprValuePayload::Integral {
                bits: "1".into(),
                label: None,
            },
        ),
    ];

    for (source, expected) in cases {
        let ast = parse_logical_expr_ast(source).expect(source);
        let bound = bind_logical_expr_ast(&ast, &host).expect(source);
        let value = eval_logical_expr_at(&bound, &host, 1).expect(source);
        assert_eq!(value.payload, expected, "{source}");
    }
}

#[test]
fn direct_event_binding_and_matching_cover_success_paths() {
    let host = rich_host();
    let ast = parse_event_expr_ast("sig or posedge clk or negedge rst or edge bus iff real_sig")
        .expect("event parse should succeed");
    let bound = bind_event_expr_ast(&ast, &host).expect("event bind should succeed");

    let frame = EventEvalFrame {
        timestamp: 1,
        previous_timestamp: Some(0),
        tracked_signals: &[SignalHandle(1), SignalHandle(2)],
    };
    assert!(event_matches_at(&bound, &host, &frame).expect("event match should succeed"));
}

#[test]
fn direct_binding_reports_targeted_semantic_failures() {
    let host = rich_host();

    for (source, expected) in [
        ("1e309", "EXPR-SEMANTIC-REAL-LITERAL"),
        ("ev", "EXPR-SEMANTIC-EVENT-VALUE"),
        ("sig.triggered()", "EXPR-SEMANTIC-TRIGGERED"),
        ("type(state)::MISSING", "EXPR-SEMANTIC-ENUM-LABEL"),
        ("string'(real_sig)", "EXPR-SEMANTIC-CAST-TARGET"),
        ("{1, sig}", "EXPR-SEMANTIC-CONCAT-UNSIZED"),
        ("{idx{sig}}", "EXPR-SEMANTIC-CONST-REQUIRED"),
        ("bus[0 +: 0]", "EXPR-SEMANTIC-CONST-RANGE"),
    ] {
        let ast = parse_logical_expr_ast(source).expect(source);
        let error = bind_logical_expr_ast(&ast, &host).expect_err(source);
        assert_eq!(error.code, expected, "{source}");
    }
}

#[test]
fn direct_event_binding_reports_targeted_semantic_failures() {
    let host = rich_host();

    for (source, expected) in [
        ("posedge real_sig", "EXPR-SEMANTIC-INTEGRAL-REQUIRED"),
        ("* iff msg", "EXPR-SEMANTIC-BOOLEAN-CONTEXT"),
        ("missing", "EXPR-SEMANTIC-UNKNOWN-SIGNAL"),
    ] {
        let ast = parse_event_expr_ast(source).expect(source);
        let error = bind_event_expr_ast(&ast, &host).expect_err(source);
        assert_eq!(error.code, expected, "{source}");
    }
}

fn rich_host() -> RichHost {
    RichHost::default()
        .with_signal(
            "sig",
            SignalDef {
                handle: SignalHandle(1),
                ty: bitvec_type(1, true, false),
                sample: SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                }),
                event_now: true,
            },
        )
        .with_signal(
            "flag",
            SignalDef {
                handle: SignalHandle(2),
                ty: bitvec_type(1, true, false),
                sample: SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                }),
                event_now: true,
            },
        )
        .with_signal(
            "clk",
            SignalDef {
                handle: SignalHandle(3),
                ty: bitvec_type(1, true, false),
                sample: SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                }),
                event_now: true,
            },
        )
        .with_signal(
            "rst",
            SignalDef {
                handle: SignalHandle(4),
                ty: bitvec_type(1, true, false),
                sample: SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("1".to_string()),
                    label: None,
                }),
                event_now: true,
            },
        )
        .with_signal(
            "bus",
            SignalDef {
                handle: SignalHandle(5),
                ty: bitvec_type(4, true, false),
                sample: SampledValue::Integral {
                    bits: Some("1011".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("0011".to_string()),
                    label: None,
                }),
                event_now: true,
            },
        )
        .with_signal(
            "real_sig",
            SignalDef {
                handle: SignalHandle(6),
                ty: ExprType {
                    kind: ExprTypeKind::Real,
                    storage: ExprStorage::Scalar,
                    width: 64,
                    is_four_state: false,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                sample: SampledValue::Real { value: Some(2.5) },
                previous_sample: Some(SampledValue::Real { value: Some(0.0) }),
                event_now: false,
            },
        )
        .with_signal(
            "msg",
            SignalDef {
                handle: SignalHandle(7),
                ty: ExprType {
                    kind: ExprTypeKind::String,
                    storage: ExprStorage::Scalar,
                    width: 0,
                    is_four_state: false,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                sample: SampledValue::String {
                    value: Some("hello".to_string()),
                },
                previous_sample: Some(SampledValue::String {
                    value: Some("idle".to_string()),
                }),
                event_now: false,
            },
        )
        .with_signal(
            "ev",
            SignalDef {
                handle: SignalHandle(8),
                ty: ExprType {
                    kind: ExprTypeKind::Event,
                    storage: ExprStorage::Scalar,
                    width: 0,
                    is_four_state: false,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                sample: SampledValue::Integral {
                    bits: Some("0".to_string()),
                    label: None,
                },
                previous_sample: None,
                event_now: true,
            },
        )
        .with_signal(
            "idx",
            SignalDef {
                handle: SignalHandle(9),
                ty: ExprType {
                    kind: ExprTypeKind::IntegerLike(IntegerLikeKind::Int),
                    storage: ExprStorage::Scalar,
                    width: 32,
                    is_four_state: false,
                    is_signed: true,
                    enum_type_id: None,
                    enum_labels: None,
                },
                sample: SampledValue::Integral {
                    bits: Some("0001".to_string()),
                    label: None,
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("0001".to_string()),
                    label: None,
                }),
                event_now: false,
            },
        )
        .with_signal(
            "state",
            SignalDef {
                handle: SignalHandle(10),
                ty: ExprType {
                    kind: ExprTypeKind::EnumCore,
                    storage: ExprStorage::Scalar,
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: Some("state_t".to_string()),
                    enum_labels: Some(vec![
                        EnumLabelInfo {
                            name: "IDLE".to_string(),
                            bits: "00".to_string(),
                        },
                        EnumLabelInfo {
                            name: "BUSY".to_string(),
                            bits: "10".to_string(),
                        },
                    ]),
                },
                sample: SampledValue::Integral {
                    bits: Some("10".to_string()),
                    label: Some("BUSY".to_string()),
                },
                previous_sample: Some(SampledValue::Integral {
                    bits: Some("00".to_string()),
                    label: Some("IDLE".to_string()),
                }),
                event_now: false,
            },
        )
}

fn bitvec_type(width: u32, four_state: bool, signed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width,
        is_four_state: four_state,
        is_signed: signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn host_diag(code: &'static str, message: &str) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: wavepeek::expr::DiagnosticLayer::Semantic,
        code,
        message: message.to_string(),
        primary_span: wavepeek::expr::Span::new(0, 0),
        notes: vec![],
    }
}
