mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use crate::support::{
    BenchHost, BenchSignal, bit_ty, criterion_config, event_ty, integral_signal, real_ty, string_ty,
};
use wavepeek::expr::{
    EventEvalFrame, SampledValue, bind_event_expr_ast, event_matches_at, parse_event_expr_ast,
};

fn event_host() -> BenchHost {
    let mut host = BenchHost::new();
    host.insert_signal(
        "clk",
        1,
        integral_signal(
            bit_ty(1, true, false),
            vec![
                (0, "0"),
                (5, "1"),
                (8, "0"),
                (10, "1"),
                (12, "0"),
                (15, "1"),
            ],
        ),
    );
    host.insert_signal(
        "rstn",
        2,
        integral_signal(bit_ty(1, true, false), vec![(0, "1"), (10, "0")]),
    );
    host.insert_signal(
        "count",
        3,
        integral_signal(
            bit_ty(8, true, false),
            vec![(10, "00000010"), (15, "00000011")],
        ),
    );
    host.insert_signal(
        "flag",
        4,
        integral_signal(bit_ty(4, true, false), vec![(10, "0000"), (15, "0001")]),
    );
    host.insert_signal(
        "ready",
        5,
        integral_signal(bit_ty(1, true, false), vec![(0, "0"), (7, "1")]),
    );
    host.insert_signal(
        "data",
        6,
        integral_signal(
            bit_ty(8, true, false),
            vec![(0, "00000000"), (10, "00001111")],
        ),
    );
    host.insert_signal(
        "x_sig",
        7,
        integral_signal(bit_ty(1, true, false), vec![(0, "x")]),
    );
    host.insert_signal(
        "a",
        8,
        integral_signal(
            bit_ty(8, true, false),
            vec![(0, "11001010"), (30, "11110000")],
        ),
    );
    host.insert_signal(
        "temp",
        9,
        BenchSignal::new(
            real_ty(),
            vec![(0, SampledValue::Real { value: Some(1.5) })],
        ),
    );
    host.insert_signal(
        "msg",
        10,
        BenchSignal::new(
            string_ty(),
            vec![(
                0,
                SampledValue::String {
                    value: Some("go".to_string()),
                },
            )],
        ),
    );
    host.insert_signal(
        "ev",
        11,
        BenchSignal::with_events(event_ty(), vec![], vec![10]),
    );
    host
}

fn bench_bind_event_union_iff(c: &mut Criterion) {
    let source = "posedge clk iff (!rstn && count >= 8'd3) or edge data";
    let ast = parse_event_expr_ast(source).expect("bind bench source should parse");
    let host = event_host();

    c.bench_function("event__bind_event_union_iff", |b| {
        b.iter(|| {
            let bound = bind_event_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_event_union_iff must bind");
            black_box(bound);
        })
    });
}

fn bench_eval_event_union_iff_true(c: &mut Criterion) {
    let source = "posedge clk iff (!rstn && (count >= 8'd3) || flag == 'h1) or ready";
    let host = event_host();
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

    c.bench_function("event__eval_event_union_iff_true", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_union_iff_true must evaluate");
            black_box(matched);
        })
    });
}

fn bench_eval_event_union_iff_unknown(c: &mut Criterion) {
    let source = "posedge clk iff (x_sig && 1)";
    let host = event_host();
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

    c.bench_function("event__eval_event_union_iff_unknown", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_union_iff_unknown must evaluate");
            black_box(matched);
        })
    });
}

fn bench_eval_event_iff_core_integral(c: &mut Criterion) {
    let source = "posedge clk iff a[1]";
    let host = event_host();
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

    c.bench_function("event__eval_event_iff_core_integral", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_iff_core_integral must evaluate");
            black_box(matched);
        })
    });
}

fn bench_eval_event_iff_triggered_rich(c: &mut Criterion) {
    let source = "posedge clk iff (ev.triggered() && (temp > 1.0) && (msg == \"go\"))";
    let host = event_host();
    let ast = parse_event_expr_ast(source).expect("event bench source should parse");
    let bound = bind_event_expr_ast(&ast, &host).expect("event bench source should bind");
    let tracked = [host.handle("clk")];
    let frame = EventEvalFrame {
        timestamp: 10,
        previous_timestamp: Some(0),
        tracked_signals: tracked.as_slice(),
    };
    assert!(event_matches_at(&bound, &host, &frame).expect("event bench setup must evaluate"));

    c.bench_function("event__eval_event_iff_triggered_rich", |b| {
        b.iter(|| {
            let matched = event_matches_at(black_box(&bound), black_box(&host), black_box(&frame))
                .expect("eval_event_iff_triggered_rich must evaluate");
            black_box(matched);
        })
    });
}

criterion_group!(
    name = benches;
    config = criterion_config();
    targets =
        bench_bind_event_union_iff,
        bench_eval_event_union_iff_true,
        bench_eval_event_union_iff_unknown,
        bench_eval_event_iff_core_integral,
        bench_eval_event_iff_triggered_rich
);
criterion_main!(benches);
