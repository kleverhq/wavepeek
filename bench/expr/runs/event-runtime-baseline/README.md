# Expression run: event-runtime-baseline

- Benchmark command: `cargo bench --bench expr_event_runtime -- --save-baseline event-runtime-baseline --noplot`
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
| bind_event_union_iff | 100 | 733.019925 | 729.404694 | bind_event_union_iff.raw.csv |
| eval_event_union_iff_true | 100 | 477.459663 | 476.587675 | eval_event_union_iff_true.raw.csv |
| eval_event_union_iff_unknown | 100 | 270.190418 | 269.613217 | eval_event_union_iff_unknown.raw.csv |
