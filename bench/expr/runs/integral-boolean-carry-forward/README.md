# Expression run: integral-boolean-carry-forward

- Benchmark command: `cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-carry-forward --noplot`
- Bench target: `expr_integral_boolean`
- Scenario set: `integral_boolean` (bench/expr/scenarios/integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `ff7959845fd7a252851325af8cc4494e6de419d1`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 576.864543 | 576.130162 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 538.742975 | 539.265355 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 788.014923 | 792.165311 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 502.541665 | 501.825386 | eval_logical_core_integral_unknown.raw.csv |
