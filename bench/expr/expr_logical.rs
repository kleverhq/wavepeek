mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use crate::support::{
    BenchHost, BenchSignal, bit_ty, criterion_config, int_ty, integral_signal, real_ty, string_ty,
};
use wavepeek::expr::{
    EnumLabelInfo, ExprStorage, ExprType, ExprTypeKind, ExprValuePayload, IntegerLikeKind,
    SampledValue, bind_logical_expr_ast, eval_logical_expr_at, parse_logical_expr_ast,
};

fn logical_host() -> BenchHost {
    let mut host = BenchHost::new();
    host.insert_signal(
        "a",
        1,
        integral_signal(
            bit_ty(8, true, false),
            vec![(0, "11001010"), (30, "11110000")],
        ),
    );
    host.insert_signal(
        "b",
        2,
        integral_signal(
            bit_ty(8, true, false),
            vec![(0, "00110101"), (30, "00001111")],
        ),
    );
    host.insert_signal(
        "count",
        3,
        integral_signal(
            int_ty(IntegerLikeKind::Int),
            vec![(0, "00000000000000000000000000000010")],
        ),
    );
    host.insert_signal(
        "temp",
        4,
        BenchSignal::new(
            real_ty(),
            vec![(0, SampledValue::Real { value: Some(1.5) })],
        ),
    );
    host.insert_signal(
        "msg",
        5,
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
        "state",
        6,
        BenchSignal::new(
            ExprType {
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
            vec![(
                0,
                SampledValue::Integral {
                    bits: Some("01".to_string()),
                    label: Some("BUSY".to_string()),
                },
            )],
        ),
    );
    host.insert_signal(
        "sel",
        7,
        integral_signal(bit_ty(1, true, false), vec![(0, "1")]),
    );
    host.insert_signal(
        "one",
        8,
        integral_signal(bit_ty(1, true, false), vec![(0, "1")]),
    );
    host.insert_signal(
        "x_cond",
        9,
        integral_signal(bit_ty(1, true, false), vec![(0, "x"), (30, "x")]),
    );
    host.insert_signal(
        "x_rhs",
        10,
        integral_signal(bit_ty(4, true, false), vec![(0, "10x1"), (30, "001x")]),
    );
    host
}

fn bench_bind_logical_core_integral(c: &mut Criterion) {
    let source = "(a[1] == 1'b1) && (b[0] == 1'b1)";
    let ast = parse_logical_expr_ast(source).expect("bind bench source should parse");
    let host = logical_host();

    c.bench_function("logical__bind_logical_core_integral", |b| {
        b.iter(|| {
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_logical_core_integral must bind");
            black_box(bound);
        })
    });
}

fn bench_eval_logical_core_integral_true(c: &mut Criterion) {
    let source = "(a[1] == 1'b1) && (b[0] == 1'b1)";
    let host = logical_host();
    let ast = parse_logical_expr_ast(source).expect("eval true source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("eval true source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("eval true setup must evaluate");
    assert!(matches!(
        value.payload,
        ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("logical__eval_logical_core_integral_true", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_core_integral_true must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_core_integral_unknown(c: &mut Criterion) {
    let source = "x_cond ? {4{one}} : x_rhs";
    let host = logical_host();
    let ast = parse_logical_expr_ast(source).expect("eval unknown source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("eval unknown source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("eval unknown setup must evaluate");
    assert!(matches!(
        value.payload,
        ExprValuePayload::Integral { ref bits, .. } if bits == "1xx1"
    ));

    c.bench_function("logical__eval_logical_core_integral_unknown", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_core_integral_unknown must evaluate");
            black_box(value);
        })
    });
}

fn bench_bind_logical_rich_types(c: &mut Criterion) {
    let source =
        "(type(state)::BUSY == state) && (type(temp)'(count) > 1.0) && (type(msg)'(\"go\") == msg)";
    let ast = parse_logical_expr_ast(source).expect("bind bench source should parse");
    let host = logical_host();

    c.bench_function("logical__bind_logical_rich_types", |b| {
        b.iter(|| {
            let bound = bind_logical_expr_ast(black_box(&ast), black_box(&host))
                .expect("bind_logical_rich_types must bind");
            black_box(bound);
        })
    });
}

fn bench_eval_logical_real_mixed_numeric(c: &mut Criterion) {
    let source = "1.5 + count + temp";
    let host = logical_host();
    let ast = parse_logical_expr_ast(source).expect("real bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("real bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("real bench setup must evaluate");
    assert!(matches!(value.payload, ExprValuePayload::Real { .. }));

    c.bench_function("logical__eval_logical_real_mixed_numeric", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_real_mixed_numeric must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_string_equality(c: &mut Criterion) {
    let source = "msg == \"go\"";
    let host = logical_host();
    let ast = parse_logical_expr_ast(source).expect("string bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("string bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("string bench setup must evaluate");
    assert!(matches!(
        value.payload,
        ExprValuePayload::Integral { ref bits, .. } if bits == "1"
    ));

    c.bench_function("logical__eval_logical_string_equality", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_string_equality must evaluate");
            black_box(value);
        })
    });
}

fn bench_eval_logical_enum_label_preservation(c: &mut Criterion) {
    let source = "sel ? type(state)::BUSY : type(state)::IDLE";
    let host = logical_host();
    let ast = parse_logical_expr_ast(source).expect("enum bench source should parse");
    let bound = bind_logical_expr_ast(&ast, &host).expect("enum bench source should bind");

    let value = eval_logical_expr_at(&bound, &host, 0).expect("enum bench setup must evaluate");
    assert!(matches!(
        value.payload,
        ExprValuePayload::Integral { ref label, .. } if label.as_deref() == Some("BUSY")
    ));

    c.bench_function("logical__eval_logical_enum_label_preservation", |b| {
        b.iter(|| {
            let value = eval_logical_expr_at(black_box(&bound), black_box(&host), black_box(0))
                .expect("eval_logical_enum_label_preservation must evaluate");
            black_box(value);
        })
    });
}

criterion_group!(
    name = benches;
    config = criterion_config();
    targets =
        bench_bind_logical_core_integral,
        bench_eval_logical_core_integral_true,
        bench_eval_logical_core_integral_unknown,
        bench_bind_logical_rich_types,
        bench_eval_logical_real_mixed_numeric,
        bench_eval_logical_string_equality,
        bench_eval_logical_enum_label_preservation
);
criterion_main!(benches);
