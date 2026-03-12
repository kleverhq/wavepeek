use std::collections::HashMap;
use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use wavepeek::expr::{
    EventEvalFrame, ExprDiagnostic, ExprStorage, ExprType, ExprTypeKind, ExpressionHost,
    SampledValue, SignalHandle, bind_event_expr_ast, bind_logical_expr_ast, eval_logical_expr_at,
    event_matches_at, parse_event_expr_ast, parse_logical_expr_ast,
};

#[derive(Clone)]
struct BenchSignal {
    ty: ExprType,
    samples: Vec<(u64, String)>,
}

struct BenchHost {
    handles: HashMap<&'static str, SignalHandle>,
    signals: HashMap<SignalHandle, BenchSignal>,
}

impl BenchHost {
    fn c3() -> Self {
        let mut handles = HashMap::new();
        let mut signals = HashMap::new();

        let mut insert =
            |name: &'static str, handle: u32, ty: ExprType, samples: Vec<(u64, &'static str)>| {
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
                        ty,
                        samples: normalized_samples,
                    },
                );
            };

        insert(
            "clk",
            1,
            bit_ty(1, true, false),
            vec![
                (0, "0"),
                (10, "1"),
                (20, "0"),
                (30, "1"),
                (40, "0"),
                (50, "1"),
            ],
        );
        insert(
            "a",
            2,
            bit_ty(8, true, false),
            vec![(0, "11001010"), (30, "11110000")],
        );
        insert(
            "b",
            3,
            bit_ty(8, true, false),
            vec![(0, "00110101"), (30, "00001111")],
        );
        insert(
            "idx",
            4,
            bit_ty(3, true, false),
            vec![(0, "010"), (30, "011")],
        );
        insert("sel", 5, bit_ty(1, true, false), vec![(0, "1"), (30, "0")]);
        insert("one", 6, bit_ty(1, true, false), vec![(0, "1")]);
        insert(
            "x_cond",
            7,
            bit_ty(1, true, false),
            vec![(0, "x"), (30, "x")],
        );
        insert(
            "x_rhs",
            8,
            bit_ty(4, true, false),
            vec![(0, "10x1"), (30, "001x")],
        );

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
        let sampled = if insertion_index == 0 {
            None
        } else {
            Some(signal.samples[insertion_index - 1].1.clone())
        };
        Ok(SampledValue::Integral {
            bits: sampled,
            label: None,
        })
    }

    fn event_occurred(
        &self,
        _handle: SignalHandle,
        _timestamp: u64,
    ) -> Result<bool, ExprDiagnostic> {
        Ok(false)
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

fn bench_bind_logical_core_integral(c: &mut Criterion) {
    let source = "(a[1] == 1'b1) && (b[0] == 1'b1)";
    let ast = parse_logical_expr_ast(source).expect("bind bench source should parse");
    let host = BenchHost::c3();

    c.bench_function("bind_logical_core_integral", |b| {
        b.iter(|| {
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_logical_core_integral must bind");
            black_box(bound);
        })
    });
}

fn bench_eval_logical_core_integral_true(c: &mut Criterion) {
    let source = "(a[1] == 1'b1) && (b[0] == 1'b1)";
    let host = BenchHost::c3();
    let ast = parse_logical_expr_ast(source).expect("eval true source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("eval true source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("eval true setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("eval_logical_core_integral_true", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_core_integral_true must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_core_integral_unknown(c: &mut Criterion) {
    let source = "x_cond ? {4{one}} : x_rhs";
    let host = BenchHost::c3();
    let ast = parse_logical_expr_ast(source).expect("eval unknown source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("eval unknown source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("eval unknown setup must evaluate");
    assert!(matches!(
        value.payload,
        wavepeek::expr::ExprValuePayload::Integral { ref bits, .. } if bits == "1xx1"
    ));

    c.bench_function("eval_logical_core_integral_unknown", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_core_integral_unknown must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_event_iff_core_integral(c: &mut Criterion) {
    let source = "posedge clk iff a[1]";
    let host = BenchHost::c3();
    let ast = parse_event_expr_ast(source).expect("event iff source should parse");
    let bound = bind_event_expr_ast(&ast, &host).expect("event iff source should bind");
    let tracked = [host.handle("clk")];
    let frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: Some(0),
        tracked_signals: tracked.as_slice(),
    };

    assert!(
        event_matches_at(&bound, &host, &frame).expect("event iff setup must evaluate"),
        "eval_event_iff_core_integral should match"
    );

    c.bench_function("eval_event_iff_core_integral", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_iff_core_integral must evaluate");
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
    targets =
        bench_bind_logical_core_integral,
        bench_eval_logical_core_integral_true,
        bench_eval_logical_core_integral_unknown,
        bench_eval_event_iff_core_integral
);
criterion_main!(benches);
