# Expression run: event-runtime-candidate

- Benchmark command: `cargo bench --bench expr_event_runtime -- --save-baseline event-runtime-candidate --noplot`
- Bench target: `expr_event_runtime`
- Scenario set: `event_runtime` (bench/expr/scenarios/event_runtime.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `dc1d14160e20d69c278f3043aa59d98f1754280c`
- Worktree state: `dirty`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_event_union_iff | 100 | 721.204003 | 714.955243 | bind_event_union_iff.raw.csv |
| eval_event_union_iff_true | 100 | 477.674295 | 477.611421 | eval_event_union_iff_true.raw.csv |
| eval_event_union_iff_unknown | 100 | 268.427638 | 267.526452 | eval_event_union_iff_unknown.raw.csv |
