use serde::Deserialize;
use wavepeek::expr::{
    EventEvalFrame, ExprDiagnostic, ExprType, ExprValue, ExprValuePayload, bind_event_expr_ast,
    bind_logical_expr_ast, eval_logical_expr_at, event_matches_at, parse_event_expr_ast,
    parse_logical_expr_ast,
};

mod common;
use common::expr_cases::{SpanRecord, expected_layer, load_expr_manifest};
use common::expr_runtime::{InMemoryExprHost, SignalFixture, TypeFixture, expr_type_from_fixture};

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
    expected_bits: String,
    expected_type: TypeFixture,
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
}

fn load_positive_manifest() -> PositiveManifest {
    load_expr_manifest("integral_boolean_positive_manifest.json")
}

fn load_negative_manifest() -> NegativeManifest {
    load_expr_manifest("integral_boolean_negative_manifest.json")
}

#[test]
fn integral_boolean_positive_manifest_matches() {
    let manifest = load_positive_manifest();

    for case in manifest.logical_cases {
        let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
        let ast = parse_logical_expr_ast(case.source.as_str())
            .unwrap_or_else(|error| panic!("{} should parse: {error:?}", case.name));
        let bound = bind_logical_expr_ast(&ast, &host)
            .unwrap_or_else(|error| panic!("{} should bind: {error:?}", case.name));
        let value = eval_logical_expr_at(&bound, &host, case.timestamp)
            .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));

        assert_eq!(
            integral_bits(&value),
            case.expected_bits,
            "case '{}'",
            case.name
        );
        assert_expr_type_eq(
            &value.ty,
            &expr_type_from_fixture(&case.expected_type),
            case.name.as_str(),
        );
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
fn integral_boolean_negative_manifest_matches_snapshots() {
    let manifest = load_negative_manifest();
    let host = InMemoryExprHost::from_fixtures(
        [
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("0".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "a".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("00000000".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "idx".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("0001".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("00000000000000000000000000000001".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
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
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("00".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "ev".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("0".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
        ]
        .as_slice(),
    );

    for case in manifest.cases {
        let diagnostic = match case.entrypoint.as_str() {
            "logical" => match parse_logical_expr_ast(case.source.as_str()) {
                Ok(ast) => bind_logical_expr_ast(&ast, &host)
                    .expect_err(&format!("{} should fail", case.name)),
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
fn integral_boolean_unknown_flow_regressions_hold() {
    let host = InMemoryExprHost::from_fixtures(
        [
            SignalFixture {
                name: "x_cond".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("x".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "lhs".to_string(),
                ty: TypeFixture {
                    kind: "bit_vector".to_string(),
                    integer_like_kind: None,
                    storage: "packed_vector".to_string(),
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("01".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "rhs".to_string(),
                ty: TypeFixture {
                    kind: "bit_vector".to_string(),
                    integer_like_kind: None,
                    storage: "packed_vector".to_string(),
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("x1".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
        ]
        .as_slice(),
    );

    let conditional = eval_expr_at("x_cond ? 4'b1010 : 4'b1001", &host, 0).expect("eval");
    assert_eq!(integral_bits(&conditional), "10xx");

    let wildcard = eval_expr_at("lhs ==? rhs", &host, 0).expect("eval");
    let case_eq = eval_expr_at("lhs === rhs", &host, 0).expect("eval");
    assert_eq!(integral_bits(&wildcard), "1");
    assert_eq!(integral_bits(&case_eq), "0");

    let inside = eval_expr_at("2'b1x inside {2'b10, 2'b11}", &host, 0).expect("eval");
    assert_eq!(integral_bits(&inside), "x");

    let signed_div = eval_expr_at("signed'(8'hfc) / signed'(8'h02)", &host, 0).expect("eval");
    let signed_mod = eval_expr_at("signed'(8'hfb) % signed'(8'h02)", &host, 0).expect("eval");
    assert_eq!(integral_bits(&signed_div), "11111110");
    assert_eq!(integral_bits(&signed_mod), "11111111");

    let zero_pow_negative =
        eval_expr_at("signed'(8'h00) ** signed'(8'hff)", &host, 0).expect("eval");
    let nonzero_pow_negative =
        eval_expr_at("signed'(8'h02) ** signed'(8'hff)", &host, 0).expect("eval");
    assert_eq!(integral_bits(&zero_pow_negative), "xxxxxxxx");
    assert_eq!(integral_bits(&nonzero_pow_negative), "00000000");
}

#[test]
fn integral_boolean_short_circuit_preservation_holds() {
    let mut host = InMemoryExprHost::from_fixtures(
        [
            SignalFixture {
                name: "trap".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("1".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "a".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("0001".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "b".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("0010".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
        ]
        .as_slice(),
    );
    host.enable_sample_trap("trap");

    let before = host.sample_count("trap");
    let value = eval_expr_at("0 && ((a + b) > trap)", &host, 0).expect("eval");
    let after = host.sample_count("trap");

    assert_eq!(integral_bits(&value), "0");
    assert_eq!(before, after, "rhs signal must not be sampled");
}

#[test]
fn integral_boolean_selection_and_missing_sample_regressions_hold() {
    let host = InMemoryExprHost::from_fixtures(
        [
            SignalFixture {
                name: "a".to_string(),
                ty: TypeFixture {
                    kind: "bit_vector".to_string(),
                    integer_like_kind: None,
                    storage: "packed_vector".to_string(),
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("10".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "b".to_string(),
                ty: TypeFixture {
                    kind: "bit_vector".to_string(),
                    integer_like_kind: None,
                    storage: "packed_vector".to_string(),
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("01".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "idx".to_string(),
                ty: TypeFixture {
                    kind: "bit_vector".to_string(),
                    integer_like_kind: None,
                    storage: "packed_vector".to_string(),
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: None,
                    enum_labels: None,
                },
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: Some("01".to_string()),
                    label: None,
                    real: None,
                    string: None,
                }],
            },
            SignalFixture {
                name: "data".to_string(),
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
                event_timestamps: vec![],
                samples: vec![common::expr_runtime::SignalSample {
                    timestamp: 0,
                    bits: None,
                    label: None,
                    real: None,
                    string: None,
                }],
            },
        ]
        .as_slice(),
    );

    let derived = eval_expr_at("{a,b}[3:1]", &host, 0).expect("eval");
    assert_eq!(integral_bits(&derived), "100");

    let replicated = eval_expr_at("{2{a}}[2]", &host, 0).expect("eval");
    assert_eq!(integral_bits(&replicated), "0");

    let missing = eval_expr_at("data[idx]", &host, 0).expect("eval");
    assert_eq!(integral_bits(&missing), "x");
}

fn eval_expr_at(
    source: &str,
    host: &dyn wavepeek::expr::ExpressionHost,
    timestamp: u64,
) -> Result<wavepeek::expr::ExprValue, ExprDiagnostic> {
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

fn integral_bits(value: &ExprValue) -> &str {
    match &value.payload {
        ExprValuePayload::Integral { bits, .. } => bits.as_str(),
        other => panic!("expected integral payload, got {other:?}"),
    }
}
