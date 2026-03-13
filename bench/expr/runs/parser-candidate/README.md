# Expression run: parser-candidate

- Benchmark command: `cargo bench --bench expr_parser -- --save-baseline parser-candidate --noplot`
- Bench target: `expr_parser`
- Scenario set: `parser` (bench/expr/scenarios/parser.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `c541ac9baaff845b89773831715e64ff71d465c6`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| parse_event_malformed | 100 | 289.977075 | 284.522402 | parse_event_malformed.raw.csv |
| parse_event_union_iff | 100 | 777.974961 | 766.531600 | parse_event_union_iff.raw.csv |
| tokenize_union_iff | 100 | 277.129594 | 277.110814 | tokenize_union_iff.raw.csv |
