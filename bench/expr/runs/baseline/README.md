# Expression Bench Run: baseline

- Generated at (UTC): 2026-03-28T17:34:15Z
- Catalog: `bench/expr/suites.json`
- Catalog fingerprint: `6a02342ef1b6f7e299d2764f5c628b42fa39c79359e5e416c7d24bec4621703d`
- Selected suites: `syntax, logical, event, waveform_host`
- cargo -V: `cargo 1.93.0 (083ac5135 2025-12-15)`
- rustc -V: `rustc 1.93.0 (254b59607 2026-01-19)`
- criterion crate version: `0.8.2`
- Source commit: `ca8e0ad0e69a937a255693663650c04d24361079`
- Worktree state: `clean`
- Environment note: wavepeek devcontainer/CI image

## syntax

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| tokenize_union_iff | 215.012284 | 213.400975 |
| parse_event_union_iff | 304.252966 | 303.307486 |
| parse_event_malformed | 164.748081 | 165.005872 |

## logical

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_logical_core_integral | 709.255908 | 706.901844 |
| eval_logical_core_integral_true | 772.666056 | 771.896529 |
| eval_logical_core_integral_unknown | 503.305497 | 499.329267 |
| bind_logical_rich_types | 1140.394348 | 1136.689326 |
| eval_logical_real_mixed_numeric | 418.171745 | 416.093071 |
| eval_logical_string_equality | 255.063283 | 254.277813 |
| eval_logical_enum_label_preservation | 318.390979 | 318.196106 |

## event

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_event_union_iff | 1900.095102 | 1898.020893 |
| eval_event_union_iff_true | 868.273770 | 864.693973 |
| eval_event_union_iff_unknown | 487.866978 | 481.586496 |
| eval_event_iff_core_integral | 498.579842 | 498.254462 |
| eval_event_iff_triggered_rich | 958.687069 | 955.744510 |

## waveform_host

| scenario | mean ns/iter | median ns/iter |
| --- | --- | --- |
| bind_waveform_host_metadata_path | 22257.367823 | 22218.034776 |
| eval_waveform_host_metadata_path | 210.441338 | 210.242603 |

