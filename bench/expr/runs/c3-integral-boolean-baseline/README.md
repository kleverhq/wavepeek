# Expression run: c3-integral-boolean-baseline

- Benchmark command: `cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-baseline --noplot`
- Bench target: `expr_c3`
- Scenario set: `c3_integral_boolean` (bench/expr/scenarios/c3_integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `ff7959845fd7a252851325af8cc4494e6de419d1`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 566.138291 | 564.944571 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 508.316688 | 506.513088 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 765.517475 | 764.061014 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 499.455204 | 498.499357 | eval_logical_core_integral_unknown.raw.csv |
