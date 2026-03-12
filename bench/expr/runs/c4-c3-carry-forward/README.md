# Expression run: c4-c3-carry-forward

- Benchmark command: `cargo bench --bench expr_c3 -- --save-baseline c4-c3-carry-forward --noplot`
- Bench target: `expr_c3`
- Scenario set: `c3_integral_boolean` (bench/expr/scenarios/c3_integral_boolean.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `71dd01581296a225f5f8b1376c2c6826d6d5c255`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_core_integral | 100 | 587.027441 | 585.382121 | bind_logical_core_integral.raw.csv |
| eval_event_iff_core_integral | 100 | 513.833950 | 512.895920 | eval_event_iff_core_integral.raw.csv |
| eval_logical_core_integral_true | 100 | 801.543034 | 801.971901 | eval_logical_core_integral_true.raw.csv |
| eval_logical_core_integral_unknown | 100 | 512.687429 | 509.761168 | eval_logical_core_integral_unknown.raw.csv |
