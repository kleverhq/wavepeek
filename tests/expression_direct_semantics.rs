mod common;

use common::expr_runtime::{
    EnumLabelFixture, InMemoryExprHost, SignalFixture, SignalSample, TypeFixture, bind_event_expr,
    bind_logical_expr, eval_logical_expr_source_at,
};
use wavepeek::expr::{EventEvalFrame, ExprValuePayload, event_matches_at};

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
        let value = eval_logical_expr_source_at(source, &host, 1).expect(source);
        assert_eq!(value.payload, expected, "{source}");
    }
}

#[test]
fn direct_event_binding_and_matching_cover_success_paths() {
    let host = rich_host();
    let bound = bind_event_expr(
        "sig or posedge clk or negedge rst or edge bus iff real_sig",
        &host,
    )
    .expect("event bind should succeed");
    let tracked = host.tracked_handles(&["sig".to_string(), "flag".to_string()]);

    let frame = EventEvalFrame {
        timestamp: 1,
        previous_timestamp: Some(0),
        tracked_signals: &tracked,
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
        let error = bind_logical_expr(source, &host).expect_err(source);
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
        let error = bind_event_expr(source, &host).expect_err(source);
        assert_eq!(error.code, expected, "{source}");
    }
}

fn rich_host() -> InMemoryExprHost {
    InMemoryExprHost::from_fixtures(&[
        bit_signal("sig", 1, &[(0, "0"), (1, "1")]),
        bit_signal("flag", 1, &[(0, "1"), (1, "1")]),
        bit_signal("clk", 1, &[(0, "0"), (1, "1")]),
        bit_signal("rst", 1, &[(0, "1"), (1, "0")]),
        bit_signal("bus", 4, &[(0, "0011"), (1, "1011")]),
        real_signal("real_sig", &[(0, 0.0), (1, 2.5)]),
        string_signal("msg", &[(0, "idle"), (1, "hello")]),
        event_signal("ev", &[1]),
        integer_like_signal("idx", "int", 32, false, true, &[(0, "0001"), (1, "0001")]),
        enum_signal(
            "state",
            2,
            "state_t",
            &[(0, "00", Some("IDLE")), (1, "10", Some("BUSY"))],
            vec![
                EnumLabelFixture {
                    name: "IDLE".to_string(),
                    bits: "00".to_string(),
                },
                EnumLabelFixture {
                    name: "BUSY".to_string(),
                    bits: "10".to_string(),
                },
            ],
        ),
    ])
}

fn bit_signal(name: &str, width: u32, samples: &[(u64, &str)]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: bit_vector_type(width, true, false),
        samples: samples
            .iter()
            .map(|(timestamp, bits)| bits_sample(*timestamp, bits, None))
            .collect(),
        event_timestamps: vec![],
    }
}

fn integer_like_signal(
    name: &str,
    integer_like_kind: &str,
    width: u32,
    is_four_state: bool,
    is_signed: bool,
    samples: &[(u64, &str)],
) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "integer_like".to_string(),
            integer_like_kind: Some(integer_like_kind.to_string()),
            storage: "scalar".to_string(),
            width,
            is_four_state,
            is_signed,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: samples
            .iter()
            .map(|(timestamp, bits)| bits_sample(*timestamp, bits, None))
            .collect(),
        event_timestamps: vec![],
    }
}

fn enum_signal(
    name: &str,
    width: u32,
    enum_type_id: &str,
    samples: &[(u64, &str, Option<&str>)],
    labels: Vec<EnumLabelFixture>,
) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "enum_core".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width,
            is_four_state: true,
            is_signed: false,
            enum_type_id: Some(enum_type_id.to_string()),
            enum_labels: Some(labels),
        },
        samples: samples
            .iter()
            .map(|(timestamp, bits, label)| bits_sample(*timestamp, bits, *label))
            .collect(),
        event_timestamps: vec![],
    }
}

fn real_signal(name: &str, samples: &[(u64, f64)]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "real".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width: 64,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: samples
            .iter()
            .map(|(timestamp, value)| SignalSample {
                timestamp: *timestamp,
                bits: None,
                label: None,
                real: Some(*value),
                string: None,
            })
            .collect(),
        event_timestamps: vec![],
    }
}

fn string_signal(name: &str, samples: &[(u64, &str)]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "string".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width: 0,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: samples
            .iter()
            .map(|(timestamp, value)| SignalSample {
                timestamp: *timestamp,
                bits: None,
                label: None,
                real: None,
                string: Some((*value).to_string()),
            })
            .collect(),
        event_timestamps: vec![],
    }
}

fn event_signal(name: &str, event_timestamps: &[u64]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "event".to_string(),
            integer_like_kind: None,
            storage: "scalar".to_string(),
            width: 0,
            is_four_state: false,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: vec![],
        event_timestamps: event_timestamps.to_vec(),
    }
}

fn bit_vector_type(width: u32, is_four_state: bool, is_signed: bool) -> TypeFixture {
    TypeFixture {
        kind: "bit_vector".to_string(),
        integer_like_kind: None,
        storage: if width > 1 {
            "packed_vector".to_string()
        } else {
            "scalar".to_string()
        },
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn bits_sample(timestamp: u64, bits: &str, label: Option<&str>) -> SignalSample {
    SignalSample {
        timestamp,
        bits: Some(bits.to_string()),
        label: label.map(str::to_string),
        real: None,
        string: None,
    }
}
