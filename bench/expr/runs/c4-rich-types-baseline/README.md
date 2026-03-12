# Expression run: c4-rich-types-baseline

- Benchmark command: `cargo bench --bench expr_c4 -- --save-baseline c4-rich-types-baseline --noplot`
- Bench target: `expr_c4`
- Scenario set: `c4_rich_types` (bench/expr/scenarios/c4_rich_types.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `71dd01581296a225f5f8b1376c2c6826d6d5c255`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_rich_types | 100 | 966.581678 | 963.531797 | bind_logical_rich_types.raw.csv |
| bind_waveform_host_metadata_path | 100 | 22329.416075 | 22285.823111 | bind_waveform_host_metadata_path.raw.csv |
| eval_event_iff_triggered_rich | 100 | 959.473805 | 958.508278 | eval_event_iff_triggered_rich.raw.csv |
| eval_logical_enum_label_preservation | 100 | 317.490028 | 317.295973 | eval_logical_enum_label_preservation.raw.csv |
| eval_logical_real_mixed_numeric | 100 | 428.998544 | 428.433025 | eval_logical_real_mixed_numeric.raw.csv |
| eval_logical_string_equality | 100 | 258.611334 | 258.670932 | eval_logical_string_equality.raw.csv |
| eval_waveform_host_metadata_path | 100 | 213.964615 | 214.441368 | eval_waveform_host_metadata_path.raw.csv |
