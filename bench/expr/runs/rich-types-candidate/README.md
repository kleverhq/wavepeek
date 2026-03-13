# Expression run: rich-types-candidate

- Benchmark command: `cargo bench --bench expr_rich_types -- --save-baseline rich-types-candidate --noplot`
- Bench target: `expr_rich_types`
- Scenario set: `rich_types` (bench/expr/scenarios/rich_types.json)
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `c7cc5dfc14604041f04e7577fb86843dd2de1388`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

| scenario | samples | mean ns/iter | median ns/iter | raw csv |
| --- | ---: | ---: | ---: | --- |
| bind_logical_rich_types | 100 | 958.873131 | 955.165453 | bind_logical_rich_types.raw.csv |
| bind_waveform_host_metadata_path | 100 | 22439.135464 | 22306.462349 | bind_waveform_host_metadata_path.raw.csv |
| eval_event_iff_triggered_rich | 100 | 947.440622 | 945.692655 | eval_event_iff_triggered_rich.raw.csv |
| eval_logical_enum_label_preservation | 100 | 712.633889 | 707.168704 | eval_logical_enum_label_preservation.raw.csv |
| eval_logical_real_mixed_numeric | 100 | 461.063656 | 459.649055 | eval_logical_real_mixed_numeric.raw.csv |
| eval_logical_string_equality | 100 | 280.808694 | 280.372491 | eval_logical_string_equality.raw.csv |
| eval_waveform_host_metadata_path | 100 | 254.392968 | 254.308119 | eval_waveform_host_metadata_path.raw.csv |
