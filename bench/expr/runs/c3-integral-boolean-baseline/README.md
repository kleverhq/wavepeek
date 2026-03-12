# Expression run: c3-integral-boolean-baseline

- Benchmark command: `cargo bench --bench expr_c3 -- --save-baseline c3-integral-boolean-baseline --noplot`
- Bench target: `expr_c3`
- Scenario set: `c3_integral_boolean` (bench/expr/scenarios/c3_integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `5ee64deca3ed8008ab6a6d0d835e2691a754814c`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 595.694341 | 593.441336 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 904.999303 | 904.297176 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 714.963041 | 713.211618 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 531.708428 | 530.309573 | eval_logical_core_integral_unknown.raw.csv |
