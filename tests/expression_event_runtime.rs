use serde_json::Value;
use wavepeek::expr::{EventEvalFrame, event_matches_at};

mod common;
use common::expr_cases::{
    ManifestEntrypoint, NegativeCase, PositiveCase, assert_negative_diagnostic,
    load_negative_manifest, load_positive_manifest, run_negative_case,
};
use common::expr_runtime::{
    InMemoryExprHost, SignalFixture, SignalSample, TypeFixture, bind_event_expr,
    collect_bound_event_matches, collect_event_matches,
};
use common::{fixture_path, wavepeek_cmd};

fn load_positive_cases() -> Vec<common::expr_cases::EventEvalCase> {
    load_positive_manifest("event_runtime_positive_manifest.json")
        .cases
        .into_iter()
        .map(|case| match case {
            PositiveCase::EventEval(case) => case,
            other => panic!("event runtime suite only supports event_eval cases, got {other:?}"),
        })
        .collect()
}

fn load_negative_cases() -> Vec<NegativeCase> {
    load_negative_manifest("event_runtime_negative_manifest.json").cases
}

fn bit_signal(name: &str, width: u32, samples: &[(u64, &str)]) -> SignalFixture {
    SignalFixture {
        name: name.to_string(),
        ty: TypeFixture {
            kind: "bit_vector".to_string(),
            integer_like_kind: None,
            storage: if width > 1 {
                "packed_vector".to_string()
            } else {
                "scalar".to_string()
            },
            width,
            is_four_state: true,
            is_signed: false,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: samples
            .iter()
            .map(|(timestamp, bits)| SignalSample {
                timestamp: *timestamp,
                bits: Some((*bits).to_string()),
                label: None,
                real: None,
                string: None,
            })
            .collect(),
        event_timestamps: vec![],
    }
}

#[test]
fn event_runtime_positive_manifest_matches() {
    for case in load_positive_cases() {
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
}

#[test]
fn wildcard_tracked_set_binding_comes_from_host_context() {
    let host = InMemoryExprHost::from_fixtures(
        [
            bit_signal("clk", 1, &[(0, "0"), (5, "1"), (10, "1"), (15, "0")]),
            bit_signal("data", 1, &[(0, "0"), (5, "0"), (10, "1"), (15, "1")]),
        ]
        .as_slice(),
    );
    let expr = bind_event_expr("*", &host).expect("wildcard should bind once");
    let probes = [0_u64, 5, 10, 15];

    let clk_only = host.tracked_handles(&["clk".to_string()]);
    let clk_matches = collect_bound_event_matches(&expr, &host, clk_only.as_slice(), &probes)
        .expect("wildcard should evaluate for clk-only tracked set");
    assert_eq!(
        clk_matches,
        vec![5, 15],
        "clk-only tracked set should see clk changes"
    );

    let data_only = host.tracked_handles(&["data".to_string()]);
    let data_matches = collect_bound_event_matches(&expr, &host, data_only.as_slice(), &probes)
        .expect("wildcard should evaluate for data-only tracked set");
    assert_eq!(
        data_matches,
        vec![10],
        "data-only tracked set should see data changes"
    );

    let both = host.tracked_handles(&["clk".to_string(), "data".to_string()]);
    let both_matches = collect_bound_event_matches(&expr, &host, both.as_slice(), &probes)
        .expect("wildcard should evaluate for combined tracked set");
    assert_eq!(
        both_matches,
        vec![5, 10, 15],
        "combined tracked set should see both changes"
    );
}

#[test]
fn event_runtime_negative_manifest_matches_snapshots() {
    for case in load_negative_cases() {
        if matches!(case.entrypoint, ManifestEntrypoint::Logical) {
            panic!("event runtime negative suite does not support logical entrypoints");
        };
        let diagnostic = run_negative_case(&case);

        assert_negative_diagnostic("event_runtime_negative_manifest.json", &case, &diagnostic);
    }
}

#[test]
fn event_runtime_short_circuit_holds() {
    let mut host = InMemoryExprHost::from_fixtures(
        [
            bit_signal("clk", 1, &[(0, "0"), (5, "1")]),
            bit_signal("rhs_sig", 1, &[(0, "1")]),
            bit_signal("x_sig", 1, &[(0, "x")]),
        ]
        .as_slice(),
    );
    host.enable_sample_trap("rhs_sig");
    let tracked = host.tracked_handles(&["clk".to_string()]);
    let frame = EventEvalFrame {
        timestamp: 5,
        previous_timestamp: Some(0),
        tracked_signals: tracked.as_slice(),
    };

    let before_and = host.sample_count("rhs_sig");
    let short_and =
        bind_event_expr("posedge clk iff (0 && rhs_sig)", &host).expect("0 && rhs_sig should bind");
    let matched =
        event_matches_at(&short_and, &host, &frame).expect("0 && rhs_sig should evaluate");
    let after_and = host.sample_count("rhs_sig");
    assert!(!matched, "0 && rhs_sig must gate event to false");
    assert_eq!(
        after_and, before_and,
        "rhs_sig must not be sampled for 0 && rhs_sig"
    );

    let before_or = host.sample_count("rhs_sig");
    let short_or =
        bind_event_expr("posedge clk iff (1 || rhs_sig)", &host).expect("1 || rhs_sig should bind");
    let matched = event_matches_at(&short_or, &host, &frame).expect("1 || rhs_sig should evaluate");
    let after_or = host.sample_count("rhs_sig");
    assert!(matched, "1 || rhs_sig must gate event to true");
    assert_eq!(
        after_or, before_or,
        "rhs_sig must not be sampled for 1 || rhs_sig"
    );

    let x_and_zero =
        bind_event_expr("posedge clk iff (x_sig && 0)", &host).expect("x_sig && 0 should bind");
    let matched = event_matches_at(&x_and_zero, &host, &frame).expect("x_sig && 0 should evaluate");
    assert!(!matched, "x && 0 must evaluate to 0 and suppress event");

    let x_or_one =
        bind_event_expr("posedge clk iff (x_sig || 1)", &host).expect("x_sig || 1 should bind");
    let matched = event_matches_at(&x_or_one, &host, &frame).expect("x_sig || 1 should evaluate");
    assert!(matched, "x || 1 must evaluate to 1 and allow event");

    let x_and_one =
        bind_event_expr("posedge clk iff (x_sig && 1)", &host).expect("x_sig && 1 should bind");
    let matched = event_matches_at(&x_and_one, &host, &frame).expect("x_sig && 1 should evaluate");
    assert!(!matched, "x && 1 must evaluate to x and suppress event");
}

#[test]
fn event_runtime_shadow_parity_matches_change_cli_for_non_iff_surface() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let host = InMemoryExprHost::from_fixtures(
        [
            bit_signal(
                "clk",
                1,
                &[
                    (0, "0"),
                    (5, "1"),
                    (10, "0"),
                    (15, "1"),
                    (20, "x"),
                    (25, "0"),
                    (30, "1"),
                ],
            ),
            bit_signal("clk1", 1, &[(0, "0"), (10, "1"), (20, "0"), (30, "1")]),
            bit_signal("clk2", 1, &[(0, "0"), (15, "1"), (25, "0"), (30, "1")]),
        ]
        .as_slice(),
    );

    let probes = [0_u64, 5, 10, 15, 20, 25, 30];
    for (source, cli_signals, tracked) in [
        (
            "*",
            "clk1,clk2",
            vec!["clk1".to_string(), "clk2".to_string()],
        ),
        ("clk", "clk", vec!["clk".to_string()]),
        ("posedge clk", "clk", vec!["clk".to_string()]),
        ("negedge clk", "clk", vec!["clk".to_string()]),
        ("edge clk", "clk", vec!["clk".to_string()]),
        (
            "posedge clk1, posedge clk2",
            "clk1,clk2",
            vec!["clk1".to_string(), "clk2".to_string()],
        ),
        (
            "posedge clk1 or posedge clk2",
            "clk1,clk2",
            vec!["clk1".to_string(), "clk2".to_string()],
        ),
    ] {
        let typed = bind_event_expr(source, &host).expect("typed source should bind");
        let tracked_handles = host.tracked_handles(tracked.as_slice());
        let typed_matches =
            collect_bound_event_matches(&typed, &host, tracked_handles.as_slice(), &probes)
                .expect("typed evaluation should succeed");

        let cli_matches = change_cli_matches(&fixture, source, cli_signals);
        assert_eq!(typed_matches, cli_matches, "source '{source}'");
    }
}

fn change_cli_matches(fixture: &str, source: &str, signals: &str) -> Vec<u64> {
    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture,
            "--from",
            "0ns",
            "--to",
            "30ns",
            "--scope",
            "top",
            "--signals",
            signals,
            "--on",
            source,
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("change command should execute");

    assert!(
        output.status.success(),
        "change command must succeed for source '{source}'"
    );

    let payload: Value = serde_json::from_slice(output.stdout.as_slice())
        .expect("change output should be valid JSON");
    payload["data"]
        .as_array()
        .expect("change data must be an array")
        .iter()
        .map(|row| {
            let token = row["time"]
                .as_str()
                .expect("change row time must be string");
            token
                .strip_suffix("ns")
                .unwrap_or(token)
                .parse::<u64>()
                .expect("change row time must be integer dump ticks")
        })
        .collect()
}
