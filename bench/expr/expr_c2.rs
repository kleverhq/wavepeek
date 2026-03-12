use std::collections::HashMap;
use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use wavepeek::expr::{
    EventEvalFrame, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost,
    SampledValue, SignalHandle, bind_event_expr_ast, event_matches_at, parse_event_expr_ast,
};

#[derive(Clone)]
struct BenchSignal {
    width: u32,
    is_four_state: bool,
    is_signed: bool,
    samples: Vec<(u64, String)>,
}

struct BenchHost {
    handles: HashMap<&'static str, SignalHandle>,
    signals: HashMap<SignalHandle, BenchSignal>,
}

impl BenchHost {
    fn c2() -> Self {
        let mut handles = HashMap::new();
        let mut signals = HashMap::new();

        let mut insert = |name: &'static str,
                          handle: u32,
                          width: u32,
                          is_four_state: bool,
                          is_signed: bool,
                          samples: Vec<(u64, &'static str)>| {
            let signal_handle = SignalHandle(handle);
            handles.insert(name, signal_handle);
            let mut normalized_samples: Vec<(u64, String)> = samples
                .into_iter()
                .map(|(timestamp, bits)| (timestamp, bits.to_string()))
                .collect();
            normalized_samples.sort_by_key(|(timestamp, _)| *timestamp);
            signals.insert(
                signal_handle,
                BenchSignal {
                    width,
                    is_four_state,
                    is_signed,
                    samples: normalized_samples,
                },
            );
        };

        insert(
            "clk",
            1,
            1,
            true,
            false,
            vec![
                (0, "0"),
                (5, "1"),
                (8, "0"),
                (10, "1"),
                (12, "0"),
                (15, "1"),
            ],
        );
        insert("rstn", 2, 1, true, false, vec![(0, "1"), (10, "0")]);
        insert(
            "count",
            3,
            8,
            true,
            false,
            vec![(10, "00000010"), (15, "00000011")],
        );
        insert("flag", 4, 4, true, false, vec![(10, "0000"), (15, "0001")]);
        insert("ready", 5, 1, true, false, vec![(0, "0"), (7, "1")]);
        insert(
            "data",
            6,
            8,
            true,
            false,
            vec![(0, "00000000"), (10, "00001111")],
        );
        insert("x_sig", 7, 1, true, false, vec![(0, "x")]);

        Self { handles, signals }
    }

    fn handle(&self, name: &'static str) -> SignalHandle {
        *self.handles.get(name).expect("benchmark signal must exist")
    }
}

impl ExpressionHost for BenchHost {
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
        let signal = self.signals.get(&handle).ok_or_else(|| ExprDiagnostic {
            layer: wavepeek::expr::DiagnosticLayer::Semantic,
            code: "BENCH-UNKNOWN-TYPE",
            message: format!("unknown signal handle {}", handle.0),
            primary_span: wavepeek::expr::Span::new(0, 0),
            notes: vec![],
        })?;
        Ok(ExprType {
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
        let sampled = if insertion_index == 0 {
            None
        } else {
            Some(signal.samples[insertion_index - 1].1.clone())
        };
        Ok(SampledValue { bits: sampled })
    }
}

fn bench_bind_event_union_iff(c: &mut Criterion) {
    let source = "posedge clk iff (!rstn && count >= 8'd3) or edge data";
    let ast = parse_event_expr_ast(source).expect("bind bench source should parse");
    let host = BenchHost::c2();

    c.bench_function("bind_event_union_iff", |b| {
        b.iter(|| {
            let bound = bind_event_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_event_union_iff must bind");
            black_box(bound);
        })
    });
}

fn bench_eval_event_union_iff_true(c: &mut Criterion) {
    let source = "posedge clk iff (!rstn && (count >= 8'd3) || flag == 'h1) or ready";
    let host = BenchHost::c2();
    let ast = parse_event_expr_ast(source).expect("eval true source should parse");
    let bound = bind_event_expr_ast(&ast, &host).expect("eval true source should bind");
    let tracked = [host.handle("clk"), host.handle("ready")];
    let frame = EventEvalFrame {
        timestamp: 15,
        previous_timestamp: Some(12),
        tracked_signals: tracked.as_slice(),
    };
    assert!(
        event_matches_at(&bound, &host, &frame).expect("eval true setup must evaluate"),
        "eval_event_union_iff_true should match"
    );

    c.bench_function("eval_event_union_iff_true", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_union_iff_true must evaluate");
            black_box(matched);
        })
    });
}

fn bench_eval_event_union_iff_unknown(c: &mut Criterion) {
    let source = "posedge clk iff (x_sig && 1)";
    let host = BenchHost::c2();
    let ast = parse_event_expr_ast(source).expect("eval unknown source should parse");
    let bound = bind_event_expr_ast(&ast, &host).expect("eval unknown source should bind");
    let tracked = [host.handle("clk")];
    let frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: Some(8),
        tracked_signals: tracked.as_slice(),
    };
    assert!(
        !event_matches_at(&bound, &host, &frame).expect("eval unknown setup must evaluate"),
        "eval_event_union_iff_unknown should be suppressed by unknown iff result"
    );

    c.bench_function("eval_event_union_iff_unknown", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_union_iff_unknown must evaluate");
            black_box(matched);
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
    targets = bench_bind_event_union_iff, bench_eval_event_union_iff_true, bench_eval_event_union_iff_unknown
);
criterion_main!(benches);
