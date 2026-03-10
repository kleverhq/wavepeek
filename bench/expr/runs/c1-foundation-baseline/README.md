# Expression C1 run: c1-foundation-baseline

- Benchmark command: `cargo bench --bench expr_c1 -- --save-baseline c1-foundation-baseline --noplot`
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `2305a2381e02b6ec6cf6426bd82f3dc8e9be4c65`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| parse_event_malformed | 100 | 151.241229 | 150.852312 | parse_event_malformed.raw.csv |
| parse_event_union_iff | 100 | 277.879253 | 273.956196 | parse_event_union_iff.raw.csv |
| tokenize_union_iff | 100 | 192.336017 | 192.207946 | tokenize_union_iff.raw.csv |
