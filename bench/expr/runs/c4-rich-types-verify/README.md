# Expression run: c4-rich-types-verify

- Benchmark command: `cargo bench --bench expr_c4 -- --save-baseline c4-rich-types-verify --noplot`
- Bench target: `expr_c4`
- Scenario set: `c4_rich_types` (bench/expr/scenarios/c4_rich_types.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `ff7959845fd7a252851325af8cc4494e6de419d1`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_rich_types | 100 | 967.420607 | 964.335423 | bind_logical_rich_types.raw.csv |
| bind_waveform_host_metadata_path | 100 | 22988.113599 | 22832.669931 | bind_waveform_host_metadata_path.raw.csv |
| eval_event_iff_triggered_rich | 100 | 964.562388 | 963.357175 | eval_event_iff_triggered_rich.raw.csv |
| eval_logical_enum_label_preservation | 100 | 317.927979 | 315.425829 | eval_logical_enum_label_preservation.raw.csv |
| eval_logical_real_mixed_numeric | 100 | 425.231999 | 425.697969 | eval_logical_real_mixed_numeric.raw.csv |
| eval_logical_string_equality | 100 | 255.493309 | 254.174399 | eval_logical_string_equality.raw.csv |
| eval_waveform_host_metadata_path | 100 | 211.698801 | 211.601750 | eval_waveform_host_metadata_path.raw.csv |
