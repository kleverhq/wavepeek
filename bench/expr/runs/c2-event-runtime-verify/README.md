# Expression run: c2-event-runtime-verify

- Benchmark command: `cargo bench --bench expr_c2 -- --save-baseline c2-event-runtime-verify --noplot`
- Bench target: `expr_c2`
- Scenario set: `c2_event_runtime` (bench/expr/scenarios/c2_event_runtime.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `dc1d14160e20d69c278f3043aa59d98f1754280c`
- Worktree state: `dirty`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_event_union_iff | 100 | 750.247408 | 750.317570 | bind_event_union_iff.raw.csv |
| eval_event_union_iff_true | 100 | 492.034159 | 492.534928 | eval_event_union_iff_true.raw.csv |
| eval_event_union_iff_unknown | 100 | 270.134659 | 269.856264 | eval_event_union_iff_unknown.raw.csv |
