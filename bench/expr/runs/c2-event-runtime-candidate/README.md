# Expression run: c2-event-runtime-candidate

- Benchmark command: `cargo bench --bench expr_c2 -- --save-baseline c2-event-runtime-candidate --noplot`
- Bench target: `expr_c2`
- Scenario set: `c2_event_runtime` (bench/expr/scenarios/c2_event_runtime.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `5eb4e10c02eb1d345c6826faa6502262d3c2bcd8`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_event_union_iff | 100 | 702.822856 | 700.882297 | bind_event_union_iff.raw.csv |
| eval_event_union_iff_true | 100 | 186.289195 | 185.028869 | eval_event_union_iff_true.raw.csv |
| eval_event_union_iff_unknown | 100 | 106.610784 | 106.197969 | eval_event_union_iff_unknown.raw.csv |
