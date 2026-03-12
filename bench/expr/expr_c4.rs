pub use wavepeek::expr;

pub mod error {
    pub use wavepeek::WavepeekError;
}

#[allow(dead_code, unused_imports)]
#[path = "../../src/waveform/mod.rs"]
mod waveform;

use std::collections::HashMap;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use waveform::expr_host::WaveformExprHost;
use wavepeek::expr::{
    EnumLabelInfo, EventEvalFrame, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind,
    ExpressionHost, IntegerLikeKind, SampledValue, SignalHandle, bind_event_expr_ast,
    bind_logical_expr_ast, eval_logical_expr_at, event_matches_at, parse_event_expr_ast,
    parse_logical_expr_ast,
};

#[derive(Clone)]
struct BenchSignal {
    ty: ExprType,
    samples: Vec<(u64, SampledValue)>,
    event_timestamps: Vec<u64>,
}

struct RichBenchHost {
    handles: HashMap<&'static str, SignalHandle>,
    signals: HashMap<SignalHandle, BenchSignal>,
}

impl RichBenchHost {
    fn c4() -> Self {
        let mut handles = HashMap::new();
        let mut signals = HashMap::new();

        let mut insert = |name: &'static str, handle: u32, signal: BenchSignal| {
            let signal_handle = SignalHandle(handle);
            handles.insert(name, signal_handle);
            signals.insert(signal_handle, signal);
        };

        insert(
            "clk",
            1,
            BenchSignal {
                ty: bit_ty(1, true, false),
                samples: vec![
                    (0, integral_sample("0")),
                    (10, integral_sample("1")),
                    (20, integral_sample("0")),
                ],
                event_timestamps: vec![],
            },
        );
        insert(
            "data",
            2,
            BenchSignal {
                ty: bit_ty(8, true, false),
                samples: vec![(0, integral_sample("00000011"))],
                event_timestamps: vec![],
            },
        );
        insert(
            "count",
            3,
            BenchSignal {
                ty: int_ty(IntegerLikeKind::Int),
                samples: vec![(0, integral_sample("00000000000000000000000000000010"))],
                event_timestamps: vec![],
            },
        );
        insert(
            "temp",
            4,
            BenchSignal {
                ty: real_ty(),
                samples: vec![(0, SampledValue::Real { value: Some(1.5) })],
                event_timestamps: vec![],
            },
        );
        insert(
            "msg",
            5,
            BenchSignal {
                ty: string_ty(),
                samples: vec![(
                    0,
                    SampledValue::String {
                        value: Some("go".to_string()),
                    },
                )],
                event_timestamps: vec![],
            },
        );
        insert(
            "state",
            6,
            BenchSignal {
                ty: ExprType {
                    kind: ExprTypeKind::EnumCore,
                    storage: ExprStorage::Scalar,
                    width: 2,
                    is_four_state: true,
                    is_signed: false,
                    enum_type_id: Some("fsm_state".to_string()),
                    enum_labels: Some(vec![
                        EnumLabelInfo {
                            name: "IDLE".to_string(),
                            bits: "00".to_string(),
                        },
                        EnumLabelInfo {
                            name: "BUSY".to_string(),
                            bits: "01".to_string(),
                        },
                    ]),
                },
                samples: vec![(
                    0,
                    SampledValue::Integral {
                        bits: Some("01".to_string()),
                        label: Some("BUSY".to_string()),
                    },
                )],
                event_timestamps: vec![],
            },
        );
        insert(
            "sel",
            7,
            BenchSignal {
                ty: bit_ty(1, true, false),
                samples: vec![(0, integral_sample("1"))],
                event_timestamps: vec![],
            },
        );
        insert(
            "ev",
            8,
            BenchSignal {
                ty: event_ty(),
                samples: vec![],
                event_timestamps: vec![10],
            },
        );

        Self { handles, signals }
    }

    fn handle(&self, name: &'static str) -> SignalHandle {
        *self.handles.get(name).expect("benchmark signal must exist")
    }
}

impl ExpressionHost for RichBenchHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        self.handles
            .get(name)
            .copied()
            .ok_or_else(|| ExprDiagnostic {
                layer: wavepeek::expr::DiagnosticLayer::Semantic,
                code: "BENCH-UNKNOWN-SIGNAL",
                message: format!("unknown signal '{name}'"),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec![],
            })
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        self.signals
            .get(&handle)
            .map(|signal| signal.ty.clone())
            .ok_or_else(|| ExprDiagnostic {
                layer: wavepeek::expr::DiagnosticLayer::Semantic,
                code: "BENCH-UNKNOWN-TYPE",
                message: format!("unknown signal handle {}", handle.0),
                primary_span: wavepeek::expr::Span::new(0, 0),
                notes: vec![],
            })
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        let signal = self.signals.get(&handle).ok_or_else(|| ExprDiagnostic {
            layer: wavepeek::expr::DiagnosticLayer::Runtime,
            code: "BENCH-MISSING-TIMELINE",
            message: format!("missing timeline for signal handle {}", handle.0),
            primary_span: wavepeek::expr::Span::new(0, 0),
            notes: vec![],
        })?;
        let insertion_index = signal
            .samples
            .partition_point(|(sample_time, _)| *sample_time <= timestamp);
        Ok(if insertion_index == 0 {
            empty_sample_for_type(&signal.ty)
        } else {
            signal.samples[insertion_index - 1].1.clone()
        })
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        let signal = self.signals.get(&handle).ok_or_else(|| ExprDiagnostic {
            layer: wavepeek::expr::DiagnosticLayer::Runtime,
            code: "BENCH-MISSING-EVENTS",
            message: format!("missing event set for signal handle {}", handle.0),
            primary_span: wavepeek::expr::Span::new(0, 0),
            notes: vec![],
        })?;
        Ok(signal.event_timestamps.contains(&timestamp))
    }
}

fn integral_sample(bits: &'static str) -> SampledValue {
    SampledValue::Integral {
        bits: Some(bits.to_string()),
        label: None,
    }
}

fn empty_sample_for_type(ty: &ExprType) -> SampledValue {
    match ty.kind {
        ExprTypeKind::Real => SampledValue::Real { value: None },
        ExprTypeKind::String => SampledValue::String { value: None },
        _ => SampledValue::Integral {
            bits: None,
            label: None,
        },
    }
}

fn bit_ty(width: u32, is_four_state: bool, is_signed: bool) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: if width > 1 {
            ExprStorage::PackedVector
        } else {
            ExprStorage::Scalar
        },
        width,
        is_four_state,
        is_signed,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn int_ty(kind: IntegerLikeKind) -> ExprType {
    let (width, is_four_state) = match kind {
        IntegerLikeKind::Byte => (8, false),
        IntegerLikeKind::Shortint => (16, false),
        IntegerLikeKind::Int => (32, false),
        IntegerLikeKind::Longint => (64, false),
        IntegerLikeKind::Integer => (32, true),
        IntegerLikeKind::Time => (64, true),
    };
    ExprType {
        kind: ExprTypeKind::IntegerLike(kind),
        storage: ExprStorage::Scalar,
        width,
        is_four_state,
        is_signed: kind != IntegerLikeKind::Time,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn real_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Real,
        storage: ExprStorage::Scalar,
        width: 64,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn string_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::String,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn event_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Event,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("hand")
        .join(filename)
}

fn bench_bind_logical_rich_types(c: &mut Criterion) {
    let source =
        "(type(state)::BUSY == state) && (type(temp)'(count) > 1.0) && (type(msg)'(\"go\") == msg)";
    let ast = parse_logical_expr_ast(source).expect("bind bench source should parse");
    let host = RichBenchHost::c4();

    c.bench_function("bind_logical_rich_types", |b| {
        b.iter(|| {
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_logical_rich_types must bind");
            black_box(bound);
        })
    });
}

fn bench_bind_waveform_host_metadata_path(c: &mut Criterion) {
    let source = "type(top.data)'(3) == 8'h03";
    let ast = parse_logical_expr_ast(source).expect("waveform bind source should parse");
    let fixture = fixture_path("m2_core.vcd");

    c.bench_function("bind_waveform_host_metadata_path", |b| {
        b.iter(|| {
            let host = WaveformExprHost::open(black_box(fixture.as_path()))
                .expect("waveform fixture should open");
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("waveform bind must succeed");
            black_box(bound);
        })
    });
}

fn bench_eval_logical_real_mixed_numeric(c: &mut Criterion) {
    let source = "1.5 + count + count + temp";
    let host = RichBenchHost::c4();
    let ast = parse_logical_expr_ast(source).expect("real bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("real bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("real bench setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Real { .. }
    ));

    c.bench_function("eval_logical_real_mixed_numeric", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_real_mixed_numeric must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_string_equality(c: &mut Criterion) {
    let source = "(msg == \"go\") && (msg == \"go\")";
    let host = RichBenchHost::c4();
    let ast = parse_logical_expr_ast(source).expect("string bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("string bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("string bench setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("eval_logical_string_equality", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_string_equality must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_enum_label_preservation(c: &mut Criterion) {
    let source = "sel ? type(state)::BUSY : type(state)::IDLE";
    let host = RichBenchHost::c4();
    let ast = parse_logical_expr_ast(source).expect("enum bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("enum bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("enum bench setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Integral { ref label, .. } if label.as_deref() == Some("BUSY")
    ));

    c.bench_function("eval_logical_enum_label_preservation", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_enum_label_preservation must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_event_iff_triggered_rich(c: &mut Criterion) {
    let source = "posedge clk iff (ev.triggered && (temp > 1.0) && (msg == \"go\"))";
    let host = RichBenchHost::c4();
    let ast = parse_event_expr_ast(source).expect("event bench source should parse");
    let bound = bind_event_expr_ast(&ast, &host).expect("event bench source should bind");
    let tracked = [host.handle("clk")];
    let frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: Some(0),
        tracked_signals: tracked.as_slice(),
    };

    assert!(event_matches_at(&bound, &host, &frame).expect("event bench setup must evaluate"));

    c.bench_function("eval_event_iff_triggered_rich", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_iff_triggered_rich must evaluate");
            black_box(matched);
        })
    });
}

fn bench_eval_waveform_host_metadata_path(c: &mut Criterion) {
    let source = "(type(top.data)'(3) == 8'h03) && (top.data == 8'h00) && (top.data != 8'hff)";
    let fixture = fixture_path("m2_core.vcd");
    let host = WaveformExprHost::open(fixture.as_path()).expect("waveform fixture should open");
    let ast = parse_logical_expr_ast(source).expect("waveform eval source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("waveform eval source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("waveform eval setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("eval_waveform_host_metadata_path", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_waveform_host_metadata_path must evaluate");
            black_box(value);
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .significance_level(0.05)
        .noise_threshold(0.01)
        .configure_from_args();
    targets =
        bench_bind_logical_rich_types,
        bench_bind_waveform_host_metadata_path,
        bench_eval_logical_real_mixed_numeric,
        bench_eval_logical_string_equality,
        bench_eval_logical_enum_label_preservation,
        bench_eval_event_iff_triggered_rich,
        bench_eval_waveform_host_metadata_path
);
criterion_main!(benches);
