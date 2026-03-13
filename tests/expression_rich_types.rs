use serde::Deserialize;
use wavepeek::expr::{
    DiagnosticLayer, EventEvalFrame, ExprDiagnostic, ExprType, ExprValue, ExprValuePayload,
    bind_event_expr_ast, bind_logical_expr_ast, eval_logical_expr_at, event_matches_at,
    parse_event_expr_ast, parse_logical_expr_ast,
};

mod common;
use common::expr_cases::{SpanRecord, expected_layer, load_expr_manifest};
use common::expr_runtime::{
    InMemoryExprHost, SignalFixture, SignalSample, TypeFixture, expr_type_from_fixture,
};

#[derive(Debug, Deserialize)]
struct PositiveManifest {
    logical_cases: Vec<LogicalCase>,
    event_cases: Vec<EventCase>,
}

#[derive(Debug, Deserialize)]
struct LogicalCase {
    name: String,
    source: String,
    signals: Vec<SignalFixture>,
    timestamp: u64,
    expected_type: TypeFixture,
    expected_result: ExpectedResult,
}

#[derive(Debug, Deserialize)]
struct EventCase {
    name: String,
    source: String,
    tracked_signals: Vec<String>,
    signals: Vec<SignalFixture>,
    probes: Vec<u64>,
    matches: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct ExpectedResult {
    kind: String,
    bits: Option<String>,
    label: Option<String>,
    real: Option<f64>,
    string: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NegativeManifest {
    cases: Vec<NegativeCase>,
}

#[derive(Debug, Deserialize)]
struct NegativeCase {
    name: String,
    entrypoint: String,
    source: String,
    layer: String,
    code: String,
    span: SpanRecord,
    snapshot: Option<String>,
    #[serde(default = "default_negative_signals")]
    signals: Vec<SignalFixture>,
    #[serde(default)]
    timestamp: Option<u64>,
}

fn load_positive_manifest() -> PositiveManifest {
    load_expr_manifest("rich_types_positive_manifest.json")
}

fn load_negative_manifest() -> NegativeManifest {
    load_expr_manifest("rich_types_negative_manifest.json")
}

#[test]
fn rich_types_positive_manifest_matches() {
    let manifest = load_positive_manifest();

    for case in manifest.logical_cases {
        let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
        let ast = parse_logical_expr_ast(case.source.as_str())
            .unwrap_or_else(|error| panic!("{} should parse: {error:?}", case.name));
        let bound = bind_logical_expr_ast(&ast, &host)
            .unwrap_or_else(|error| panic!("{} should bind: {error:?}", case.name));
        let value = eval_logical_expr_at(&bound, &host, case.timestamp)
            .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));

        assert_expr_type_eq(
            &value.ty,
            &expr_type_from_fixture(&case.expected_type),
            case.name.as_str(),
        );
        assert_result_eq(&value, &case.expected_result, case.name.as_str());
    }

    for case in manifest.event_cases {
        let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
        let ast = parse_event_expr_ast(case.source.as_str())
            .unwrap_or_else(|error| panic!("{} should parse: {error:?}", case.name));
        let expr = bind_event_expr_ast(&ast, &host)
            .unwrap_or_else(|error| panic!("{} should bind: {error:?}", case.name));
        let tracked_handles = host.tracked_handles(case.tracked_signals.as_slice());

        let mut previous = None;
        let mut matches = Vec::new();
        for probe in &case.probes {
            let frame = EventEvalFrame {
                timestamp: *probe,
                previous_timestamp: previous,
                tracked_signals: tracked_handles.as_slice(),
            };
            let matched = event_matches_at(&expr, &host, &frame)
                .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));
            if matched {
                matches.push(*probe);
            }
            previous = Some(*probe);
        }

        assert_eq!(matches, case.matches, "case '{}'", case.name);
    }
}

#[test]
fn rich_types_negative_manifest_matches_snapshots() {
    let manifest = load_negative_manifest();

    for case in manifest.cases {
        let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
        let diagnostic = match case.entrypoint.as_str() {
            "logical" => match parse_logical_expr_ast(case.source.as_str()) {
                Ok(ast) => match bind_logical_expr_ast(&ast, &host) {
                    Ok(bound) if case.layer == "runtime" => {
                        eval_logical_expr_at(&bound, &host, case.timestamp.unwrap_or(0))
                            .expect_err(&format!("{} should fail at runtime", case.name))
                    }
                    Ok(_) => panic!("{} should fail", case.name),
                    Err(diagnostic) => diagnostic,
                },
                Err(diagnostic) => diagnostic,
            },
            "event" => match parse_event_expr_ast(case.source.as_str()) {
                Ok(ast) => bind_event_expr_ast(&ast, &host)
                    .expect_err(&format!("{} should fail", case.name)),
                Err(diagnostic) => diagnostic,
            },
            other => panic!("unsupported entrypoint '{other}'"),
        };

        assert_eq!(
            diagnostic.layer,
            expected_layer(case.layer.as_str()),
            "case '{}'",
            case.name
        );
        assert_eq!(diagnostic.code, case.code, "case '{}'", case.name);
        assert_eq!(
            diagnostic.primary_span.start, case.span.start,
            "case '{}'",
            case.name
        );
        assert_eq!(
            diagnostic.primary_span.end, case.span.end,
            "case '{}'",
            case.name
        );

        if let Some(snapshot_name) = case.snapshot.as_deref() {
            insta::assert_snapshot!(snapshot_name, diagnostic.render(case.source.as_str()));
        }
    }
}

#[test]
fn rich_type_and_triggered_regressions_hold() {
    let host = InMemoryExprHost::from_fixtures(default_positive_signals().as_slice());

    let not_temp = eval_expr_at("!temp", &host, 10).expect("real truthiness eval");
    assert_integral_bits(&not_temp, "0");

    let logical_and = eval_expr_at("temp && 0.0", &host, 10).expect("real && eval");
    let logical_or = eval_expr_at("0.0 || temp", &host, 10).expect("real || eval");
    assert_integral_bits(&logical_and, "0");
    assert_integral_bits(&logical_or, "1");

    let string_eq = eval_expr_at("msg == \"go\"", &host, 10).expect("string eq eval");
    assert_integral_bits(&string_eq, "1");

    let merged = eval_expr_at("xsel ? type(state)::BUSY : type(state)::IDLE", &host, 10)
        .expect("enum merge eval");
    assert_integral_bits(&merged, "0x");
    assert_integral_label(&merged, None);

    let triggered_now = eval_expr_at("ev.triggered", &host, 10).expect("triggered eval");
    let triggered_later = eval_expr_at("ev.triggered", &host, 11).expect("triggered eval");
    assert_integral_bits(&triggered_now, "1");
    assert_integral_bits(&triggered_later, "0");

    let error = eval_expr_at("real'(xbus)", &host, 10).expect_err("runtime cast should fail");
    assert_eq!(error.layer, DiagnosticLayer::Runtime);
    assert_eq!(error.code, "EXPR-RUNTIME-REAL-CAST");

    let nonfinite = eval_expr_at("1.0 / 0.0", &host, 10).expect_err("non-finite real should fail");
    assert_eq!(nonfinite.layer, DiagnosticLayer::Runtime);
    assert_eq!(nonfinite.code, "EXPR-RUNTIME-REAL-NONFINITE");
}

#[test]
fn rich_type_named_and_tracked_events_handle_signal_changes() {
    let host = InMemoryExprHost::from_fixtures(default_positive_signals().as_slice());

    let msg_ast = parse_event_expr_ast("msg").expect("named string event should parse");
    let msg_expr = bind_event_expr_ast(&msg_ast, &host).expect("named string event should bind");
    let msg_frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: Some(0),
        tracked_signals: &[],
    };
    assert!(event_matches_at(&msg_expr, &host, &msg_frame).expect("named string event eval"));

    let ev_ast = parse_event_expr_ast("ev").expect("named event should parse");
    let ev_expr = bind_event_expr_ast(&ev_ast, &host).expect("named event should bind");
    let ev_frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: None,
        tracked_signals: &[],
    };
    assert!(event_matches_at(&ev_expr, &host, &ev_frame).expect("named event eval"));

    let any_ast = parse_event_expr_ast("*").expect("wildcard event should parse");
    let any_expr = bind_event_expr_ast(&any_ast, &host).expect("wildcard event should bind");
    let tracked = host.tracked_handles(&["msg".to_string(), "ev".to_string()]);
    let any_frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: None,
        tracked_signals: tracked.as_slice(),
    };
    assert!(event_matches_at(&any_expr, &host, &any_frame).expect("wildcard event eval"));
}

fn eval_expr_at(
    source: &str,
    host: &dyn wavepeek::expr::ExpressionHost,
    timestamp: u64,
) -> Result<ExprValue, ExprDiagnostic> {
    let ast = parse_logical_expr_ast(source)?;
    let bound = bind_logical_expr_ast(&ast, host)?;
    eval_logical_expr_at(&bound, host, timestamp)
}

fn assert_expr_type_eq(actual: &ExprType, expected: &ExprType, case_name: &str) {
    assert_eq!(actual.kind, expected.kind, "case '{case_name}' kind");
    assert_eq!(
        actual.storage, expected.storage,
        "case '{case_name}' storage"
    );
    assert_eq!(actual.width, expected.width, "case '{case_name}' width");
    assert_eq!(
        actual.is_four_state, expected.is_four_state,
        "case '{case_name}' is_four_state"
    );
    assert_eq!(
        actual.is_signed, expected.is_signed,
        "case '{case_name}' is_signed"
    );
    assert_eq!(
        actual.enum_type_id, expected.enum_type_id,
        "case '{case_name}' enum_type_id"
    );
    assert_eq!(
        actual.enum_labels, expected.enum_labels,
        "case '{case_name}' enum_labels"
    );
}

fn assert_result_eq(actual: &ExprValue, expected: &ExpectedResult, case_name: &str) {
    match (&actual.payload, expected.kind.as_str()) {
        (ExprValuePayload::Integral { bits, label }, "integral") => {
            assert_eq!(
                Some(bits.as_str()),
                expected.bits.as_deref(),
                "case '{case_name}' bits"
            );
            assert_eq!(
                label.as_deref(),
                expected.label.as_deref(),
                "case '{case_name}' label"
            );
        }
        (ExprValuePayload::Real { value }, "real") => {
            assert_eq!(Some(*value), expected.real, "case '{case_name}' real");
        }
        (ExprValuePayload::String { value }, "string") => {
            assert_eq!(
                Some(value.as_str()),
                expected.string.as_deref(),
                "case '{case_name}' string"
            );
        }
        (other, kind) => panic!("case '{case_name}' expected {kind} payload, got {other:?}"),
    }
}

fn assert_integral_bits(value: &ExprValue, expected: &str) {
    match &value.payload {
        ExprValuePayload::Integral { bits, .. } => assert_eq!(bits, expected),
        other => panic!("expected integral payload, got {other:?}"),
    }
}

fn assert_integral_label(value: &ExprValue, expected: Option<&str>) {
    match &value.payload {
        ExprValuePayload::Integral { label, .. } => assert_eq!(label.as_deref(), expected),
        other => panic!("expected integral payload, got {other:?}"),
    }
}

fn default_negative_signals() -> Vec<SignalFixture> {
    default_positive_signals()
}

fn default_positive_signals() -> Vec<SignalFixture> {
    vec![
        SignalFixture {
            name: "data".to_string(),
            ty: TypeFixture {
                kind: "bit_vector".to_string(),
                integer_like_kind: None,
                storage: "packed_vector".to_string(),
                width: 8,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("00000011".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "count".to_string(),
            ty: TypeFixture {
                kind: "integer_like".to_string(),
                integer_like_kind: Some("int".to_string()),
                storage: "scalar".to_string(),
                width: 32,
                is_four_state: false,
                is_signed: true,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("00000000000000000000000000000010".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "temp".to_string(),
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
            samples: vec![
                SignalSample {
                    timestamp: 0,
                    bits: None,
                    label: None,
                    real: Some(0.25),
                    string: None,
                },
                SignalSample {
                    timestamp: 10,
                    bits: None,
                    label: None,
                    real: Some(1.5),
                    string: None,
                },
            ],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "msg".to_string(),
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
            samples: vec![
                SignalSample {
                    timestamp: 0,
                    bits: None,
                    label: None,
                    real: None,
                    string: Some("idle".to_string()),
                },
                SignalSample {
                    timestamp: 10,
                    bits: None,
                    label: None,
                    real: None,
                    string: Some("go".to_string()),
                },
            ],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "state".to_string(),
            ty: TypeFixture {
                kind: "enum_core".to_string(),
                integer_like_kind: None,
                storage: "scalar".to_string(),
                width: 2,
                is_four_state: true,
                is_signed: false,
                enum_type_id: Some("fsm_state".to_string()),
                enum_labels: Some(vec![
                    common::expr_runtime::EnumLabelFixture {
                        name: "IDLE".to_string(),
                        bits: "00".to_string(),
                    },
                    common::expr_runtime::EnumLabelFixture {
                        name: "BUSY".to_string(),
                        bits: "01".to_string(),
                    },
                    common::expr_runtime::EnumLabelFixture {
                        name: "DONE".to_string(),
                        bits: "10".to_string(),
                    },
                ]),
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("01".to_string()),
                label: Some("BUSY".to_string()),
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "state_no_labels".to_string(),
            ty: TypeFixture {
                kind: "enum_core".to_string(),
                integer_like_kind: None,
                storage: "scalar".to_string(),
                width: 2,
                is_four_state: true,
                is_signed: false,
                enum_type_id: Some("fsm_state".to_string()),
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("01".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "sel".to_string(),
            ty: TypeFixture {
                kind: "bit_vector".to_string(),
                integer_like_kind: None,
                storage: "scalar".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("1".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "xsel".to_string(),
            ty: TypeFixture {
                kind: "bit_vector".to_string(),
                integer_like_kind: None,
                storage: "scalar".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("x".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "xbus".to_string(),
            ty: TypeFixture {
                kind: "bit_vector".to_string(),
                integer_like_kind: None,
                storage: "packed_vector".to_string(),
                width: 4,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![SignalSample {
                timestamp: 0,
                bits: Some("x101".to_string()),
                label: None,
                real: None,
                string: None,
            }],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "clk".to_string(),
            ty: TypeFixture {
                kind: "bit_vector".to_string(),
                integer_like_kind: None,
                storage: "scalar".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                enum_type_id: None,
                enum_labels: None,
            },
            samples: vec![
                SignalSample {
                    timestamp: 0,
                    bits: Some("0".to_string()),
                    label: None,
                    real: None,
                    string: None,
                },
                SignalSample {
                    timestamp: 10,
                    bits: Some("1".to_string()),
                    label: None,
                    real: None,
                    string: None,
                },
                SignalSample {
                    timestamp: 20,
                    bits: Some("0".to_string()),
                    label: None,
                    real: None,
                    string: None,
                },
            ],
            event_timestamps: vec![],
        },
        SignalFixture {
            name: "ev".to_string(),
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
            event_timestamps: vec![10],
        },
    ]
}
