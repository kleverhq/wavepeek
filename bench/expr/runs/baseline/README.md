# Expression Bench Run: baseline

- Generated at (UTC): 2026-03-19T21:48:39Z
- Catalog: `bench/expr/suites.json`
- Catalog fingerprint: `6a02342ef1b6f7e299d2764f5c628b42fa39c79359e5e416c7d24bec4621703d`
- Selected suites: `syntax, logical, event, waveform_host`
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `c83774c0889617a5f55157e6dfe4121780499dc9`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

## syntax

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| tokenize_union_iff | 208.575168 | 207.908818 |
| parse_event_union_iff | 298.436541 | 297.354259 |
| parse_event_malformed | 158.073094 | 157.593576 |

## logical

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_logical_core_integral | 543.561565 | 541.353426 |
| eval_logical_core_integral_true | 767.210212 | 765.092051 |
| eval_logical_core_integral_unknown | 502.821634 | 500.406254 |
| bind_logical_rich_types | 1124.436740 | 1117.914749 |
| eval_logical_real_mixed_numeric | 416.061654 | 413.864182 |
| eval_logical_string_equality | 253.985846 | 253.042029 |
| eval_logical_enum_label_preservation | 326.030488 | 324.389047 |

## event

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_event_union_iff | 1940.133167 | 1937.790204 |
| eval_event_union_iff_true | 872.338561 | 869.543715 |
| eval_event_union_iff_unknown | 490.548806 | 490.232190 |
| eval_event_iff_core_integral | 502.650967 | 501.720804 |
| eval_event_iff_triggered_rich | 976.245898 | 966.623768 |

## waveform_host

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_waveform_host_metadata_path | 22592.087255 | 22379.254985 |
| eval_waveform_host_metadata_path | 214.167659 | 212.334320 |

