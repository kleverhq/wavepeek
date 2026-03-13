# Expression run: integral-boolean-verify

- Benchmark command: `cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-verify --noplot`
- Bench target: `expr_integral_boolean`
- Scenario set: `integral_boolean` (bench/expr/scenarios/integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `5ee64deca3ed8008ab6a6d0d835e2691a754814c`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 581.473446 | 579.382207 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 882.624029 | 878.174425 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 675.269561 | 672.089592 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 541.039612 | 540.385371 | eval_logical_core_integral_unknown.raw.csv |
