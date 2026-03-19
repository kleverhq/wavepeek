pub use wavepeek::expr;

pub mod error {
    pub use wavepeek::WavepeekError;
}

#[allow(dead_code, unused_imports)]
#[path = "../../src/waveform/mod.rs"]
mod waveform;

mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use crate::support::{criterion_config, fixture_path};
use waveform::expr_host::WaveformExprHost;
use wavepeek::expr::{
    ExprValuePayload, bind_logical_expr_ast, eval_logical_expr_at, parse_logical_expr_ast,
};

fn bench_bind_waveform_host_metadata_path(c: &mut Criterion) {
    let source = "type(top.data)'(3) == 8'h03";
    let ast = parse_logical_expr_ast(source).expect("waveform bind source should parse");
    let fixture = fixture_path("m2_core.vcd");

    c.bench_function("waveform_host__bind_waveform_host_metadata_path", |b| {
        b.iter(|| {
            let host = WaveformExprHost::open(black_box(fixture.as_path()))
                .expect("waveform fixture should open");
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("waveform bind must succeed");
            black_box(bound);
        })
    });
}

fn bench_eval_waveform_host_metadata_path(c: &mut Criterion) {
    let source = "type(top.data)'(3) == 8'h03";
    let fixture = fixture_path("m2_core.vcd");
    let host = WaveformExprHost::open(fixture.as_path()).expect("waveform fixture should open");
    let ast = parse_logical_expr_ast(source).expect("waveform eval source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("waveform eval source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("waveform eval setup must evaluate");
    assert!(matches!(
        value.payload,
        ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("waveform_host__eval_waveform_host_metadata_path", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_waveform_host_metadata_path must evaluate");
            black_box(value);
        })
    });
}

criterion_group!(
    name = benches;
    config = criterion_config();
    targets = bench_bind_waveform_host_metadata_path, bench_eval_waveform_host_metadata_path
);
criterion_main!(benches);
