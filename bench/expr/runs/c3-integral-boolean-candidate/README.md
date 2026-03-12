# Expression run: c3-integral-boolean-candidate

- Benchmark command: `cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-candidate --noplot`
- Bench target: `expr_c3`
- Scenario set: `c3_integral_boolean` (bench/expr/scenarios/c3_integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `834bd054cae493510334244f2535982274506b5e`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 578.957287 | 576.928855 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 737.050621 | 734.842367 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 585.624219 | 584.157344 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 371.915424 | 371.442341 | eval_logical_core_integral_unknown.raw.csv |
