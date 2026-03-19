mod common;

use common::expr_cases::{
    ManifestEntrypoint, NegativeCase, PositiveCase, assert_negative_diagnostic,
    load_negative_manifest, load_positive_manifest, run_negative_case,
};
use common::expr_runtime::{
    InMemoryExprHost, SignalFixture, SignalSample, TypeFixture, assert_expected_value_eq,
    assert_expr_type_eq, collect_event_matches, eval_logical_expr_source_at,
    expr_type_from_fixture,
};

#[test]
fn integral_boolean_positive_manifest_matches() {
    let manifest = load_positive_manifest("integral_boolean_positive_manifest.json");

    for case in manifest.cases {
        match case {
            PositiveCase::LogicalEval(case) => {
                let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
                let value =
                    eval_logical_expr_source_at(case.source.as_str(), &host, case.timestamp)
                        .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));

                assert_expr_type_eq(
                    &value.ty,
                    &expr_type_from_fixture(&case.expected_type),
                    case.name.as_str(),
                );
                assert_expected_value_eq(&value, &case.expected_result, case.name.as_str());
            }
            PositiveCase::EventEval(case) => {
                let host = InMemoryExprHost::from_fixtures(case.signals.as_slice());
                let matches = collect_event_matches(
                    case.source.as_str(),
                    &host,
                    case.tracked_signals.as_slice(),
                    case.probes.as_slice(),
                )
                .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));
                assert_eq!(matches, case.matches, "case '{}'", case.name);
            }
            PositiveCase::EventParse(_) => {
                panic!("integral boolean suite does not support event_parse cases")
            }
        }
    }
}

fn load_negative_cases() -> Vec<NegativeCase> {
    load_negative_manifest("integral_boolean_negative_manifest.json").cases
}

#[test]
fn integral_boolean_negative_manifest_matches_snapshots() {
    for case in load_negative_cases() {
        if matches!(case.entrypoint, ManifestEntrypoint::Parse) {
            panic!("integral boolean negative suite does not support parse entrypoints");
        }
        let diagnostic = run_negative_case(&case);
        assert_negative_diagnostic(
            "integral_boolean_negative_manifest.json",
            &case,
            &diagnostic,
        );
    }
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
                samples: vec![SignalSample {
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
                samples: vec![SignalSample {
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
                samples: vec![SignalSample {
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
    let value = eval_logical_expr_source_at("0 && ((a + b) > trap)", &host, 0).expect("eval");
    let after = host.sample_count("trap");

    assert_expected_value_eq(
        &value,
        &common::expr_runtime::ExpectedValueFixture {
            kind: common::expr_runtime::ExpectedValueKind::Integral,
            bits: Some("0".to_string()),
            label: None,
            real: None,
            string: None,
        },
        "short_circuit_zero_and",
    );
    assert_eq!(before, after, "rhs signal must not be sampled");
}
