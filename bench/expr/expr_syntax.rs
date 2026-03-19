mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use crate::support::criterion_config;

fn bench_tokenize_union_iff(c: &mut Criterion) {
    c.bench_function("syntax__tokenize_union_iff", |b| {
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
    c.bench_function("syntax__parse_event_union_iff", |b| {
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
    c.bench_function("syntax__parse_event_malformed", |b| {
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
    config = criterion_config();
    targets = bench_tokenize_union_iff, bench_parse_event_union_iff, bench_parse_event_malformed
);
criterion_main!(benches);
