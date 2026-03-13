# Expression run: integral-boolean-candidate

- Benchmark command: `cargo bench --bench expr_integral_boolean -- --save-baseline integral-boolean-candidate --noplot`
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
| bind_logical_core_integral | 100 | 575.060019 | 572.766406 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 939.299842 | 942.909910 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 701.781315 | 699.763743 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 574.368982 | 573.325587 | eval_logical_core_integral_unknown.raw.csv |
