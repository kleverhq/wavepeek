use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_tokenize_union_iff(c: &mut Criterion) {
    c.bench_function("tokenize_union_iff", |b| {
        b.iter(|| {
            let source = black_box("posedge clk iff (a || b) or negedge rstn");
            assert!(
                wavepeek::expr::lex_event_expr(source).is_ok(),
                "tokenize_union_iff unexpectedly failed"
            );
        })
    });
}

fn bench_parse_event_union_iff(c: &mut Criterion) {
    c.bench_function("parse_event_union_iff", |b| {
        b.iter(|| {
            let source = black_box("posedge clk iff (a || b) or negedge rstn");
            assert!(
                wavepeek::expr::parse_event_expr_ast(source).is_ok(),
                "parse_event_union_iff unexpectedly failed"
            );
        })
    });
}

fn bench_parse_event_malformed(c: &mut Criterion) {
    c.bench_function("parse_event_malformed", |b| {
        b.iter(|| {
            let source = black_box("posedge clk or , clk");
            assert!(
                wavepeek::expr::parse_event_expr_ast(source).is_err(),
                "parse_event_malformed unexpectedly parsed"
            );
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
    targets = bench_tokenize_union_iff, bench_parse_event_union_iff, bench_parse_event_malformed
);
criterion_main!(benches);
