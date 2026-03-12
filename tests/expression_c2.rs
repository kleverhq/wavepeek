use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use wavepeek::expr::{
    DiagnosticLayer, EventEvalFrame, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind,
    ExpressionHost, SampledValue, SignalHandle, bind_event_expr_ast, event_matches_at,
    parse_event_expr_ast,
};

mod common;
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
    signals: Vec<SignalFixture>,
    probes: Vec<u64>,
    matches: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct SignalFixture {
    name: String,
    width: u32,
    is_four_state: bool,
    is_signed: bool,
    samples: Vec<SignalSample>,
}

#[derive(Debug, Deserialize)]
struct SignalSample {
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

#[derive(Debug, Deserialize)]
struct SpanRecord {
    start: usize,
    end: usize,
}

fn fixture_expr_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expr")
        .join(file_name)
}

fn load_positive_manifest() -> PositiveManifest {
    let payload = fs::read_to_string(fixture_expr_path("c2_positive_manifest.json"))
        .expect("c2 positive manifest should be readable");
    serde_json::from_str(&payload).expect("c2 positive manifest should be valid JSON")
}

fn load_negative_manifest() -> NegativeManifest {
    let payload = fs::read_to_string(fixture_expr_path("c2_negative_manifest.json"))
        .expect("c2 negative manifest should be readable");
    serde_json::from_str(&payload).expect("c2 negative manifest should be valid JSON")
}

fn expected_layer(raw: &str) -> DiagnosticLayer {
    match raw {
        "parse" => DiagnosticLayer::Parse,
        "semantic" => DiagnosticLayer::Semantic,
        "runtime" => DiagnosticLayer::Runtime,
        other => panic!("unsupported manifest layer '{other}'"),
    }
}

#[derive(Default)]
struct InMemoryHost {
    handles_by_name: HashMap<String, SignalHandle>,
    types_by_handle: HashMap<SignalHandle, ExprType>,
    timelines_by_handle: HashMap<SignalHandle, Vec<(u64, String)>>,
    trap_handles: HashSet<SignalHandle>,
    sample_counts: RefCell<HashMap<SignalHandle, usize>>,
}

impl InMemoryHost {
    fn from_fixtures(signals: &[SignalFixture]) -> Self {
        let mut host = Self::default();
        for (index, signal) in signals.iter().enumerate() {
            let handle = SignalHandle((index + 1) as u32);
            host.handles_by_name.insert(signal.name.clone(), handle);
            host.types_by_handle.insert(
                handle,
                ExprType {
                    kind: ExprTypeKind::BitVector,
                    storage: if signal.width > 1 {
                        ExprStorage::PackedVector
                    } else {
                        ExprStorage::Scalar
                    },
                    width: signal.width,
                    is_four_state: signal.is_four_state,
                    is_signed: signal.is_signed,
                    enum_type_id: None,
                },
            );
            host.timelines_by_handle.insert(
                handle,
                signal
                    .samples
                    .iter()
                    .map(|sample| (sample.timestamp, sample.bits.clone()))
                    .collect(),
            );
        }
        host
    }

    fn tracked_handles(&self, names: &[String]) -> Vec<SignalHandle> {
        names
            .iter()
            .map(|name| {
                *self
                    .handles_by_name
                    .get(name)
                    .unwrap_or_else(|| panic!("tracked signal '{name}' must exist in host"))
            })
            .collect()
    }

    fn handle(&self, name: &str) -> SignalHandle {
        *self
            .handles_by_name
            .get(name)
            .unwrap_or_else(|| panic!("signal '{name}' must exist in host"))
    }

    fn enable_sample_trap(&mut self, name: &str) {
        self.trap_handles.insert(self.handle(name));
    }

    fn sample_count(&self, name: &str) -> usize {
        let handle = self.handle(name);
        self.sample_counts
            .borrow()
            .get(&handle)
            .copied()
            .unwrap_or(0)
    }
}

impl ExpressionHost for InMemoryHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        self.handles_by_name
            .get(name)
            .copied()
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: format!("unknown signal '{name}'"),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec![],
            })
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        self.types_by_handle
            .get(&handle)
            .cloned()
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-TYPE",
                message: format!("unknown signal type for handle {}", handle.0),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec![],
            })
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        *self.sample_counts.borrow_mut().entry(handle).or_insert(0) += 1;

        if self.trap_handles.contains(&handle) {
            return Err(ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "C2-RUNTIME-UNEXPECTED-SAMPLE",
                message: format!("signal {} was sampled unexpectedly", handle.0),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec!["short-circuit branch must not sample this signal".to_string()],
            });
        }

        let timeline = self
            .timelines_by_handle
            .get(&handle)
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-MISSING-TIMELINE",
                message: format!("missing timeline for signal handle {}", handle.0),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec![],
            })?;

        let sampled = timeline
            .iter()
            .rev()
            .find(|(sample_time, _)| *sample_time <= timestamp)
            .map(|(_, bits)| bits.clone());
        Ok(SampledValue { bits: sampled })
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
fn c2_positive_manifest_matches() {
    let manifest = load_positive_manifest();
    for case in manifest.cases {
        let host = InMemoryHost::from_fixtures(case.signals.as_slice());
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
fn c2_negative_manifest_matches_snapshots() {
    let manifest = load_negative_manifest();
    let host = InMemoryHost::from_fixtures(
        [
            SignalFixture {
                name: "clk".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "0".to_string(),
                }],
            },
            SignalFixture {
                name: "a".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "0".to_string(),
                }],
            },
            SignalFixture {
                name: "b".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "1".to_string(),
                }],
            },
            SignalFixture {
                name: "c".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "1".to_string(),
                }],
            },
            SignalFixture {
                name: "data".to_string(),
                width: 8,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "00000000".to_string(),
                }],
            },
            SignalFixture {
                name: "ev".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "0".to_string(),
                }],
            },
            SignalFixture {
                name: "state".to_string(),
                width: 2,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "00".to_string(),
                }],
            },
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
fn c2_short_circuit_subset_holds() {
    let mut host = InMemoryHost::from_fixtures(
        [
            SignalFixture {
                name: "clk".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![
                    SignalSample {
                        timestamp: 0,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 5,
                        bits: "1".to_string(),
                    },
                ],
            },
            SignalFixture {
                name: "rhs_sig".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "1".to_string(),
                }],
            },
            SignalFixture {
                name: "x_sig".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![SignalSample {
                    timestamp: 0,
                    bits: "x".to_string(),
                }],
            },
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
fn c2_shadow_parity_matches_legacy_event_matches_for_non_iff_subset() {
    let fixture = fixture_path("change_edge_cases.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let host = InMemoryHost::from_fixtures(
        [
            SignalFixture {
                name: "clk".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![
                    SignalSample {
                        timestamp: 0,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 5,
                        bits: "1".to_string(),
                    },
                    SignalSample {
                        timestamp: 10,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 15,
                        bits: "1".to_string(),
                    },
                    SignalSample {
                        timestamp: 20,
                        bits: "x".to_string(),
                    },
                    SignalSample {
                        timestamp: 25,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 30,
                        bits: "1".to_string(),
                    },
                ],
            },
            SignalFixture {
                name: "clk1".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![
                    SignalSample {
                        timestamp: 0,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 10,
                        bits: "1".to_string(),
                    },
                    SignalSample {
                        timestamp: 20,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 30,
                        bits: "1".to_string(),
                    },
                ],
            },
            SignalFixture {
                name: "clk2".to_string(),
                width: 1,
                is_four_state: true,
                is_signed: false,
                samples: vec![
                    SignalSample {
                        timestamp: 0,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 15,
                        bits: "1".to_string(),
                    },
                    SignalSample {
                        timestamp: 25,
                        bits: "0".to_string(),
                    },
                    SignalSample {
                        timestamp: 30,
                        bits: "1".to_string(),
                    },
                ],
            },
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
