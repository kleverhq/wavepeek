# Expression run: rich-types-baseline

- Benchmark command: `cargo bench --bench expr_rich_types -- --save-baseline rich-types-baseline --noplot`
- Bench target: `expr_rich_types`
- Scenario set: `rich_types` (bench/expr/scenarios/rich_types.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `ff7959845fd7a252851325af8cc4494e6de419d1`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_rich_types | 100 | 953.332744 | 951.856712 | bind_logical_rich_types.raw.csv |
| bind_waveform_host_metadata_path | 100 | 22598.505846 | 22518.098876 | bind_waveform_host_metadata_path.raw.csv |
| eval_event_iff_triggered_rich | 100 | 965.385259 | 963.666523 | eval_event_iff_triggered_rich.raw.csv |
| eval_logical_enum_label_preservation | 100 | 313.767123 | 313.594858 | eval_logical_enum_label_preservation.raw.csv |
| eval_logical_real_mixed_numeric | 100 | 415.116253 | 414.352180 | eval_logical_real_mixed_numeric.raw.csv |
| eval_logical_string_equality | 100 | 253.647147 | 252.750405 | eval_logical_string_equality.raw.csv |
| eval_waveform_host_metadata_path | 100 | 210.151763 | 210.082047 | eval_waveform_host_metadata_path.raw.csv |
