# Expression Bench Run: baseline

- Generated at (UTC): 2026-03-19T22:09:20Z
- Catalog: `bench/expr/suites.json`
- Catalog fingerprint: `6a02342ef1b6f7e299d2764f5c628b42fa39c79359e5e416c7d24bec4621703d`
- Selected suites: `syntax, logical, event, waveform_host`
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `b5aa5d13b1d3bf80baae11819f334487d5d916b7`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

## syntax

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| tokenize_union_iff | 216.989016 | 214.133989 |
| parse_event_union_iff | 298.007322 | 294.087266 |
| parse_event_malformed | 158.650526 | 157.685449 |

## logical

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_logical_core_integral | 543.377313 | 543.275643 |
| eval_logical_core_integral_true | 777.188284 | 771.734220 |
| eval_logical_core_integral_unknown | 502.240156 | 502.337135 |
| bind_logical_rich_types | 1030.547536 | 1026.252784 |
| eval_logical_real_mixed_numeric | 420.964336 | 420.684620 |
| eval_logical_string_equality | 256.310974 | 255.674725 |
| eval_logical_enum_label_preservation | 319.179283 | 317.454422 |

## event

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_event_union_iff | 1920.031305 | 1913.945858 |
| eval_event_union_iff_true | 889.545935 | 888.801582 |
| eval_event_union_iff_unknown | 496.070716 | 495.664970 |
| eval_event_iff_core_integral | 525.453606 | 524.348127 |
| eval_event_iff_triggered_rich | 1008.055869 | 1006.578745 |

## waveform_host

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_waveform_host_metadata_path | 22598.160752 | 22404.278327 |
| eval_waveform_host_metadata_path | 212.458328 | 211.300480 |

