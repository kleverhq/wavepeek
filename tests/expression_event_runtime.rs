use serde::Deserialize;
use serde_json::Value;
use wavepeek::expr::{
    EventEvalFrame, ExprDiagnostic, ExpressionHost, bind_event_expr_ast, event_matches_at,
    parse_event_expr_ast,
};

mod common;
use common::expr_cases::{SpanRecord, expected_layer, load_expr_manifest};
use common::expr_runtime::{
    InMemoryExprHost, SignalFixture as RuntimeSignalFixture, SignalSample as RuntimeSignalSample,
    TypeFixture,
};
use common::{fixture_path, wavepeek_cmd};

#[derive(Debug, Deserialize)]
struct PositiveManifest {
    cases: Vec<PositiveCase>,
}

#[derive(Debug, Deserialize)]
struct PositiveCase {
    name: String,
    source: String,
    tracked_signals: Vec<String>,
    signals: Vec<EventRuntimeSignalFixture>,
    probes: Vec<u64>,
    matches: Vec<u64>,
}

#[derive(Debug, Deserialize, Clone)]
struct EventRuntimeSignalFixture {
    name: String,
    width: u32,
    is_four_state: bool,
    is_signed: bool,
    samples: Vec<EventRuntimeSignalSample>,
}

#[derive(Debug, Deserialize, Clone)]
struct EventRuntimeSignalSample {
    timestamp: u64,
    bits: String,
}

#[derive(Debug, Deserialize)]
struct NegativeManifest {
    cases: Vec<NegativeCase>,
}

#[derive(Debug, Deserialize)]
struct NegativeCase {
    name: String,
    source: String,
    layer: String,
    code: String,
    span: SpanRecord,
    snapshot: Option<String>,
}

fn load_positive_manifest() -> PositiveManifest {
    load_expr_manifest("event_runtime_positive_manifest.json")
}

fn load_negative_manifest() -> NegativeManifest {
    load_expr_manifest("event_runtime_negative_manifest.json")
}

fn to_runtime_signal_fixture(signal: &EventRuntimeSignalFixture) -> RuntimeSignalFixture {
    RuntimeSignalFixture {
        name: signal.name.clone(),
        ty: TypeFixture {
            kind: "bit_vector".to_string(),
            integer_like_kind: None,
            storage: if signal.width > 1 {
                "packed_vector".to_string()
            } else {
                "scalar".to_string()
            },
            width: signal.width,
            is_four_state: signal.is_four_state,
            is_signed: signal.is_signed,
            enum_type_id: None,
            enum_labels: None,
        },
        samples: signal
            .samples
            .iter()
            .map(|sample| RuntimeSignalSample {
                timestamp: sample.timestamp,
                bits: Some(sample.bits.clone()),
                label: None,
                real: None,
                string: None,
            })
            .collect(),
        event_timestamps: vec![],
    }
}

fn event_runtime_host(signals: &[EventRuntimeSignalFixture]) -> InMemoryExprHost {
    let fixtures = signals
        .iter()
        .map(to_runtime_signal_fixture)
        .collect::<Vec<_>>();
    InMemoryExprHost::from_fixtures(fixtures.as_slice())
}

fn bit_signal(name: &str, width: u32, samples: &[(u64, &str)]) -> EventRuntimeSignalFixture {
    EventRuntimeSignalFixture {
        name: name.to_string(),
        width,
        is_four_state: true,
        is_signed: false,
        samples: samples
            .iter()
            .map(|(timestamp, bits)| EventRuntimeSignalSample {
                timestamp: *timestamp,
                bits: (*bits).to_string(),
            })
            .collect(),
    }
}

fn bind_for_host(
    source: &str,
    host: &dyn ExpressionHost,
) -> Result<wavepeek::expr::BoundEventExpr, ExprDiagnostic> {
    let ast = parse_event_expr_ast(source)?;
    bind_event_expr_ast(&ast, host)
}

#[test]
fn event_runtime_positive_manifest_matches() {
    let manifest = load_positive_manifest();
    for case in manifest.cases {
        let host = event_runtime_host(case.signals.as_slice());
        let expr = bind_for_host(case.source.as_str(), &host)
            .unwrap_or_else(|error| panic!("{} should bind: {error:?}", case.name));
        let tracked_handles = host.tracked_handles(case.tracked_signals.as_slice());

        let mut previous_timestamp = None;
        let mut matches = Vec::new();
        for probe in &case.probes {
            let frame = EventEvalFrame {
                timestamp: *probe,
                previous_timestamp,
                tracked_signals: tracked_handles.as_slice(),
            };
            let matched = event_matches_at(&expr, &host, &frame)
                .unwrap_or_else(|error| panic!("{} should evaluate: {error:?}", case.name));
            if matched {
                matches.push(*probe);
            }
            previous_timestamp = Some(*probe);
        }

        assert_eq!(matches, case.matches, "case '{}'", case.name);
    }
}

#[test]
fn event_runtime_negative_manifest_matches_snapshots() {
    let manifest = load_negative_manifest();
    let host = event_runtime_host(
        [
            bit_signal("clk", 1, &[(0, "0")]),
            bit_signal("a", 1, &[(0, "0")]),
            bit_signal("b", 1, &[(0, "1")]),
            bit_signal("c", 1, &[(0, "1")]),
            bit_signal("data", 8, &[(0, "00000000")]),
            bit_signal("ev", 1, &[(0, "0")]),
            bit_signal("state", 2, &[(0, "00")]),
        ]
        .as_slice(),
    );

    for case in manifest.cases {
        let diagnostic = match parse_event_expr_ast(case.source.as_str()) {
            Ok(ast) => {
                bind_event_expr_ast(&ast, &host).expect_err(&format!("{} should fail", case.name))
            }
            Err(diagnostic) => diagnostic,
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
fn event_runtime_short_circuit_holds() {
    let mut host = event_runtime_host(
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
        bind_for_host("posedge clk iff (0 && rhs_sig)", &host).expect("0 && rhs_sig should bind");
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
        bind_for_host("posedge clk iff (1 || rhs_sig)", &host).expect("1 || rhs_sig should bind");
    let matched = event_matches_at(&short_or, &host, &frame).expect("1 || rhs_sig should evaluate");
    let after_or = host.sample_count("rhs_sig");
    assert!(matched, "1 || rhs_sig must gate event to true");
    assert_eq!(
        after_or, before_or,
        "rhs_sig must not be sampled for 1 || rhs_sig"
    );

    let x_and_zero =
        bind_for_host("posedge clk iff (x_sig && 0)", &host).expect("x_sig && 0 should bind");
    let matched = event_matches_at(&x_and_zero, &host, &frame).expect("x_sig && 0 should evaluate");
    assert!(!matched, "x && 0 must evaluate to 0 and suppress event");

    let x_or_one =
        bind_for_host("posedge clk iff (x_sig || 1)", &host).expect("x_sig || 1 should bind");
    let matched = event_matches_at(&x_or_one, &host, &frame).expect("x_sig || 1 should evaluate");
    assert!(matched, "x || 1 must evaluate to 1 and allow event");

    let x_and_one =
        bind_for_host("posedge clk iff (x_sig && 1)", &host).expect("x_sig && 1 should bind");
    let matched = event_matches_at(&x_and_one, &host, &frame).expect("x_sig && 1 should evaluate");
    assert!(!matched, "x && 1 must evaluate to x and suppress event");
}

#[test]
fn event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let host = event_runtime_host(
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
        let typed = bind_for_host(source, &host).expect("typed source should bind");
        let tracked_handles = host.tracked_handles(tracked.as_slice());
        let mut previous = None;
        let mut typed_matches = Vec::new();
        for probe in probes {
            let frame = EventEvalFrame {
                timestamp: probe,
                previous_timestamp: previous,
                tracked_signals: tracked_handles.as_slice(),
            };
            if event_matches_at(&typed, &host, &frame).expect("typed evaluation should succeed") {
                typed_matches.push(probe);
            }
            previous = Some(probe);
        }

        let legacy_matches = legacy_change_matches(&fixture, source, cli_signals);
        assert_eq!(typed_matches, legacy_matches, "source '{source}'");
    }
}

fn legacy_change_matches(fixture: &str, source: &str, signals: &str) -> Vec<u64> {
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
        .expect("legacy change command should execute");

    assert!(
        output.status.success(),
        "legacy change command must succeed for source '{source}'"
    );

    let payload: Value = serde_json::from_slice(output.stdout.as_slice())
        .expect("legacy output should be valid JSON");
    payload["data"]
        .as_array()
        .expect("legacy data must be an array")
        .iter()
        .map(|row| {
            let token = row["time"]
                .as_str()
                .expect("legacy row time must be string");
            token
                .strip_suffix("ns")
                .unwrap_or(token)
                .parse::<u64>()
                .expect("legacy row time must be integer dump ticks")
        })
        .collect()
}
