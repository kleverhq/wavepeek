mod common;

use common::expr_cases::{
    ManifestEntrypoint, NegativeCase, PositiveCase, assert_negative_diagnostic,
    load_negative_manifest, load_positive_manifest, run_negative_case,
};
use common::expr_runtime::{
    InMemoryExprHost, assert_expected_value_eq, assert_expr_type_eq, collect_event_matches,
    eval_logical_expr_source_at, expr_type_from_fixture,
};

#[test]
fn rich_types_positive_manifest_matches() {
    let manifest = load_positive_manifest("rich_types_positive_manifest.json");

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
                panic!("rich types suite does not support event_parse cases")
            }
        }
    }
}

fn load_negative_cases() -> Vec<NegativeCase> {
    load_negative_manifest("rich_types_negative_manifest.json").cases
}

#[test]
fn rich_types_negative_manifest_matches_snapshots() {
    for case in load_negative_cases() {
        if matches!(case.entrypoint, ManifestEntrypoint::Parse) {
            panic!("rich types negative suite does not support parse entrypoints");
        }
        let diagnostic = run_negative_case(&case);
        assert_negative_diagnostic("rich_types_negative_manifest.json", &case, &diagnostic);
    }
}
