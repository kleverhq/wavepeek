# Expression Bench Run: baseline

- Generated at (UTC): 2026-03-19T21:35:55Z
- Catalog: `bench/expr/suites.json`
- Catalog fingerprint: `6a02342ef1b6f7e299d2764f5c628b42fa39c79359e5e416c7d24bec4621703d`
- Selected suites: `syntax, logical, event, waveform_host`
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `cc4400144309545387e2f7444ff7ffc66bfdd9d3`
- Worktree state: `dirty`
- Environment note: wavepeek devcontainer/CI image

## syntax

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| tokenize_union_iff | 208.676272 | 206.544200 |
| parse_event_union_iff | 301.388392 | 293.489139 |
| parse_event_malformed | 158.782964 | 158.772894 |

## logical

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_logical_core_integral | 551.301559 | 546.880903 |
| eval_logical_core_integral_true | 778.552632 | 777.538181 |
| eval_logical_core_integral_unknown | 503.206384 | 501.231935 |
| bind_logical_rich_types | 1145.299775 | 1139.378441 |
| eval_logical_real_mixed_numeric | 412.406661 | 412.018633 |
| eval_logical_string_equality | 252.741910 | 253.930715 |
| eval_logical_enum_label_preservation | 316.305936 | 314.527828 |

## event

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_event_union_iff | 1963.193545 | 1961.044602 |
| eval_event_union_iff_true | 890.291221 | 889.577226 |
| eval_event_union_iff_unknown | 502.916253 | 499.122603 |
| eval_event_iff_core_integral | 526.724938 | 520.896580 |
| eval_event_iff_triggered_rich | 984.138145 | 981.674698 |

## waveform_host

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_waveform_host_metadata_path | 22527.428658 | 22444.622540 |
| eval_waveform_host_metadata_path | 211.461320 | 210.985881 |

