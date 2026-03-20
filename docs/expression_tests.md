# Expression Test Coverage Map

This document maps the current expression tests to the checklist in
`docs/expression_cov.md`.

It is a snapshot of what exists today, not a future plan. Historical numbering
gaps from the checklist are preserved here as-is.

## Legend

- `covered`: there is a direct test for the checklist point, or a very close
  equivalent that exercises the intended contract surface.
- `partial`: nearby behavior is tested, but the full matrix, negative space, or
  a key variant is still missing.
- `none`: no current test directly covers the point.
- `infra`: harness or implementation guardrail only; useful, but not a direct
  checklist-coverage point.
- `done`: the row has been checked and its current state is accepted.

## Test Inventory

Manifest-backed suites:

- `tests/expression_parse.rs`
  - runners: `parse_positive_manifest_parses`,
    `parse_negative_manifest_matches_snapshots`
  - code-only: `parse_no_panic_corpus_holds`
  - manifests: `PP` = `parse_positive_manifest.json` (6 cases), `PN` =
    `parse_negative_manifest.json` (12 cases)
- `tests/expression_event_runtime.rs`
  - runners: `event_runtime_positive_manifest_matches`,
    `event_runtime_negative_manifest_matches_snapshots`
  - code-only: `event_runtime_short_circuit_holds`,
    `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface`
  - manifests: `ERP` = `event_runtime_positive_manifest.json` (9 cases), `ERN`
    = `event_runtime_negative_manifest.json` (2 cases)
- `tests/expression_integral_boolean.rs`
  - runners: `integral_boolean_positive_manifest_matches`,
    `integral_boolean_negative_manifest_matches_snapshots`
  - code-only: `integral_boolean_short_circuit_preservation_holds`
  - manifests: `IBP` = `integral_boolean_positive_manifest.json` (25 cases),
    `IBN` = `integral_boolean_negative_manifest.json` (10 cases)
- `tests/expression_rich_types.rs`
  - runners: `rich_types_positive_manifest_matches`,
    `rich_types_negative_manifest_matches_snapshots`
  - manifests: `RTP` = `rich_types_positive_manifest.json` (22 cases), `RTN` =
    `rich_types_negative_manifest.json` (15 cases)

Rust unit / regression tests under `src/expr/`:

- `src/expr/parser.rs`: `typed_parser_rejects_unmatched_open_parenthesis`,
  `typed_parser_rejects_broken_union_segmentation`,
  `typed_parser_preserves_iff_binding_to_single_term`,
  `logical_parser_accepts_integral_boolean_surface_sample`,
  `logical_parser_accepts_rich_type_surface_sample`
- `src/expr/lexer.rs`: `lex_event_expr_tracks_keywords_and_spans`,
  `lex_logical_expr_accepts_integral_boolean_operators`,
  `lex_logical_expr_reserves_triggered_suffix`,
  `lex_logical_expr_accepts_real_and_string_literals`
- `src/expr/sema.rs`: `binder_rejects_unsized_concat_literal`,
  `binder_rejects_non_constant_replication_multiplier`,
  `binder_preserves_enum_identity_in_conditional_arms`
- `src/expr/eval.rs`: `logical_and_short_circuits_rhs`,
  `wildcard_equality_preserves_unknown_from_lhs`
- `src/expr/mod.rs`: `parse_event_expr_wrapper_supports_any_tracked`
- `src/expr/legacy.rs`: `event_expr_iff_binding_with_union`,
  `event_expr_iff_capture_parenthesized_logical_payload`,
  `event_expr_accepts_comma_union`

Harness / manifest-contract guardrails:

- `tests/expression_fixture_contract.rs`
  - `shared_positive_manifest_contract_accepts_tagged_cases_and_rejects_legacy_shapes`
  - `shared_negative_manifest_contract_enforces_host_context_and_runtime_timestamp`
  - `all_expression_manifests_deserialize_through_the_shared_contract`
  - `negative_manifest_snapshots_exist_and_no_expression_snapshots_are_orphaned`

Unless noted otherwise, manifest case IDs below are exercised through the suite
runner for the corresponding file.

## Checklist Matrix

### Event Expressions (`1`-`35`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 1 | `[1.1] Wildcard event surface form` | done | `PP:wildcard_any_tracked`, `ERP:wildcard_any_tracked_same_timestamp_dedup`, `RTP:wildcard_tracks_string_and_event_changes`, `parse_event_expr_wrapper_supports_any_tracked` | - |
| 2 | `[1.1] Named event surface form` | done | `PP:named_any_change`, `ERP:named_change_requires_previous_sample`, `RTP:named_string_event_matches_signal_change`, `RTP:named_event_operand_matches_raw_event_timestamp` | - |
| 3 | `[1.1] Edge event surface forms` | done | `PP:edge_keywords`, `ERP:edge_surface_form_posedge`, `ERP:edge_surface_form_negedge`, `ERP:edge_surface_form_edge` | - |
| 4 | `[1.1] Union surface forms` | covered | `PP:comma_union`, `ERP:wildcard_any_tracked_same_timestamp_dedup`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface`, `event_expr_accepts_comma_union` | - |
| 5 | `[1.1] Gated event surface form` | covered | `PP:iff_binds_to_preceding_term_only`, `PP:iff_parenthesized_payload`, `ERP:iff_bounded_ops_and_literals`, `RTP:rich_iff_with_triggered_real_and_string` | - |
| 6 | `[1.1] Wildcard tracked-set binding comes from the host context` | partial | `ERP:wildcard_any_tracked_same_timestamp_dedup`, `RTP:wildcard_tracks_string_and_event_changes` | No test reuses the same parsed `*` against two different tracked sets. |
| 8 | `[1.2] Simple signal name resolution` | covered | `PP:named_any_change`, `ERP:named_change_requires_previous_sample`, `RTP:named_string_event_matches_signal_change` | - |
| 9 | `[1.2] Hierarchical name resolution` | none | - | No hierarchical event-reference case. |
| 10 | `[1.2] Canonical dump-token resolution` | none | - | No non-simple canonical dump-token event case. |
| 11 | `[1.2] Unresolved or non-signal names are errors` | partial | `ERN:unknown_signal_inside_iff`, `IBN:unresolved_signal_in_event_iff` | Unknown-name paths exist, but there is no direct unknown whole-event operand or non-signal target case. |
| 12 | `[1.3] Named event means any value change` | covered | `ERP:named_change_requires_previous_sample`, `RTP:named_string_event_matches_signal_change`, `RTP:named_event_operand_matches_raw_event_timestamp` | - |
| 13 | `[1.3] Wildcard event means any tracked-set change` | covered | `ERP:wildcard_any_tracked_same_timestamp_dedup`, `RTP:wildcard_tracks_string_and_event_changes` | - |
| 14 | `[1.3] Edge sampling uses the previous sample strictly before the candidate timestamp` | partial | `ERP:edge_terms_use_lsb_and_xz_normalization` | Previous-sample behavior is exercised, but not isolated as its own rule. |
| 15 | `[1.3] Only the least-significant bit participates in edge classification` | partial | `ERP:edge_terms_use_lsb_and_xz_normalization` | LSB-only behavior is exercised, but there is no direct named-vs-edge contrast test. |
| 16 | `[1.3] Nine-state h/u/w/l/- normalization before edge classification` | partial | `ERP:edge_terms_use_lsb_and_xz_normalization` | Covers `h`; no direct `u`, `w`, `l`, `-` matrix. |
| 17 | `[1.3] posedge classification matrix` | partial | `ERP:edge_terms_use_lsb_and_xz_normalization`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface` | Some transitions are covered, but not the full matrix. |
| 18 | `[1.3] negedge classification matrix` | partial | `ERP:edge_terms_use_lsb_and_xz_normalization`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface` | Some transitions are covered, but not the full matrix. |
| 19 | `[1.3] edge is the union of posedge and negedge` | partial | `PP:edge_keywords`, `ERP:edge_terms_use_lsb_and_xz_normalization`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface` | The relation is implied, but not proven by a direct exact-match comparison. |
| 20 | `[1.3] No previous sample means no edge` | partial | `ERP:named_change_requires_previous_sample`, `ERP:edge_terms_use_lsb_and_xz_normalization` | First-sample no-edge behavior is not isolated per edge operator. |
| 21 | `[1.4] Union is logical OR over event terms` | covered | `ERP:wildcard_any_tracked_same_timestamp_dedup`, `ERP:iff_binds_only_to_preceding_term`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface` | - |
| 22 | `[1.4] or and , are exact semantic synonyms` | covered | `ERP:wildcard_any_tracked_same_timestamp_dedup`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface`, `event_expr_accepts_comma_union` | - |
| 23 | `[1.4] Same-timestamp union matches are deduplicated` | covered | `ERP:wildcard_any_tracked_same_timestamp_dedup` | - |
| 24 | `[1.4] iff gates only the immediately preceding event term` | covered | `PP:iff_binds_to_preceding_term_only`, `ERP:iff_binds_only_to_preceding_term`, `typed_parser_preserves_iff_binding_to_single_term`, `event_expr_iff_binding_with_union` | - |
| 25 | `[1.4] Example grouping for negedge clk iff rstn or ready` | covered | `PP:iff_binds_to_preceding_term_only`, `ERP:iff_binds_only_to_preceding_term`, `typed_parser_preserves_iff_binding_to_single_term`, `event_expr_iff_binding_with_union` | The runtime example uses `posedge`, but the same grouping rule is exercised. |
| 26 | `[1.4] iff uses the boolean expression language from section 2` | covered | `ERP:iff_bounded_ops_and_literals`, `IBP:iff_arithmetic_and_cast_in_event_engine`, `IBP:iff_inside_and_concatenation`, `IBP:iff_dynamic_selection_and_conditional`, `RTP:rich_iff_with_triggered_real_and_string` | - |
| 27 | `[1.4] Event expressions have no independent parenthesized grouping form` | covered | `PP:iff_parenthesized_payload`, `ERN:event_grouping_remains_invalid`, `IBN:event_level_grouping_remains_invalid`, `event_expr_iff_capture_parenthesized_logical_payload` | - |
| 28 | `[1.5] Union is the only event-composition operator` | partial | `ERN:event_grouping_remains_invalid`, `IBN:event_level_grouping_remains_invalid` | Grouping is rejected, but other non-whitelisted event operators are not sampled. |
| 29 | `[1.5] iff binds tighter than union` | covered | `PP:iff_binds_to_preceding_term_only`, `ERP:iff_binds_only_to_preceding_term`, `typed_parser_preserves_iff_binding_to_single_term` | - |
| 30 | `[1.5] or and , have equal precedence and left-to-right association` | partial | `PP:comma_union`, `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface`, `event_expr_accepts_comma_union` | No mixed long-chain parse or AST-shape test. |
| 31 | `[1.5] Event precedence order is basic event, then gated term, then union` | covered | `PP:iff_binds_to_preceding_term_only`, `ERP:iff_binds_only_to_preceding_term`, `typed_parser_preserves_iff_binding_to_single_term` | - |
| 32 | `[1.6] Event grammar accepts repeated unions of event terms` | partial | `PP:edge_keywords`, `PP:comma_union`, `PP:iff_binds_to_preceding_term_only` | No four-term mixed-spelling parse case. |
| 33 | `[1.6] event_term is limited to basic_event and basic_event iff logical_expr` | partial | `PP:iff_binds_to_preceding_term_only`, `ERN:event_grouping_remains_invalid` | Accepted gated terms and rejected grouped union are covered, but plain `a iff cond` is only indirect. |
| 34 | `[1.6] basic_event grammar matrix` | covered | `PP:wildcard_any_tracked`, `PP:named_any_change`, `PP:edge_keywords` | - |
| 35 | `[1.6] Edge keywords require operand references, not arbitrary expressions` | none | - | No direct `posedge 1'b1` or `posedge (a)` rejection test. |

### Types, Primaries, and Casts (`36`-`92`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 36 | `[2.0] Inline vector cast targets are supported` | partial | `IBP:integral_cast_bit_logic_variants`, `IBP:integral_cast_integer_like_target` | Many inline target spellings are still missing. |
| 37 | `[2.0] Dump-derived operand-type casts are supported` | covered | `RTP:operand_type_cast_bit_vector`, `RTP:operand_type_cast_integer_like`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | - |
| 38 | `[2.0] Dump-derived enum label references are supported` | covered | `RTP:enum_label_reference`, `RTN:invalid_enum_label_on_non_enum`, `RTN:invalid_missing_enum_label` | - |
| 39 | `[2.1] Scalar bit and logic metadata` | partial | `RTP:triggered_logical_primary`, `IBP:integral_cast_bit_logic_variants` | Bit-vs-logic metadata is not isolated directly. |
| 40 | `[2.1] Packed bit-vector family metadata matrix` | partial | `IBP:integral_cast_bit_logic_variants` | Only `bit[4]` and `logic[4]` are exercised; signed/unsigned family coverage is missing. |
| 41 | `[2.1] Integer-like builtin metadata matrix` | partial | `IBP:integral_cast_integer_like_target`, `RTP:operand_type_cast_integer_like`, `RTP:real_mixed_numeric_result` | Full builtin keyword matrix is missing. |
| 42 | `[2.1] Enum type metadata and label carriage` | partial | `RTP:enum_label_reference`, `RTP:enum_cast_reacquires_label`, `IBP:enum_conditional_preserves_identity` | Unlabeled and 2-state enum cases are missing. |
| 43 | `[2.1] Real type metadata` | covered | `RTP:real_mixed_numeric_result`, `RTP:real_power_result`, `RTP:operand_type_cast_real`, `RTP:real_cast_truncates_to_int` | - |
| 44 | `[2.1] String type metadata` | covered | `RTP:string_conditional_result`, `RTP:string_equality_exact`, `RTN:invalid_string_numeric_add`, `RTN:invalid_string_boolean_context` | - |
| 45 | `[2.1] Event operand and .triggered` | covered | `RTP:triggered_logical_primary`, `RTN:invalid_raw_event_value_use`, `RTN:invalid_event_cast` | - |
| 46 | `[2.1] Raw event is excluded from ordinary value operations` | partial | `RTN:invalid_raw_event_value_use`, `RTN:invalid_event_cast`, `RTN:invalid_triggered_on_non_event` | Forbidden categories are not sampled broadly. |
| 47 | `[2.2.1] Integral literal surface-form matrix` | partial | `ERP:iff_bounded_ops_and_literals`, `IBP:precedence_and_associativity_core`, `IBP:wildcard_equality_matches_unknown_bits` | The literal-shape matrix is incomplete. |
| 48 | `[2.2.1] Integral literal sign and width defaults` | partial | `IBP:integral_cast_integer_like_target` | Default width/sign rules are only indirect today. |
| 49 | `[2.2.1] Based integral literals may contain x and z` | covered | `IBP:integral_cast_bit_logic_variants`, `ERP:edge_terms_use_lsb_and_xz_normalization`, `IBP:inside_returns_unknown_for_unknown_membership` | - |
| 50 | `[2.2.1] Integral literals are self-determined` | partial | `IBP:integral_cast_integer_like_target`, `IBP:signed_division_preserves_twos_complement_result` | Self-determined literal behavior is not isolated. |
| 51 | `[2.2.1] Unary minus is an operator, not part of the literal token` | none | - | No direct parser/AST check. |
| 52 | `[2.2.1] Real literals produce self-determined real` | covered | `RTP:real_mixed_numeric_result`, `RTP:real_power_result` | - |
| 53 | `[2.2.1] String literals produce self-determined string` | covered | `RTP:string_conditional_result`, `RTP:string_equality_exact` | - |
| 54 | `[2.2.2] Operand-reference primaries use dump-recovered operand types` | covered | `RTP:operand_type_cast_bit_vector`, `RTP:operand_type_cast_integer_like`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string`, `RTP:enum_label_reference` | - |
| 55 | `[2.2.2] Literals are valid primary expressions` | covered | `IBP:precedence_and_associativity_core`, `RTP:real_power_result`, `RTP:string_equality_exact` | - |
| 56 | `[2.2.2] Parenthesized expressions preserve inner value and type` | partial | `IBP:precedence_and_associativity_core`, `PP:iff_parenthesized_payload` | Parentheses-as-grouping is covered; type/value preservation is not isolated. |
| 57 | `[2.2.2] Static casts are valid primaries` | covered | `IBP:integral_cast_integer_like_target`, `RTP:real_cast_truncates_to_int`, `RTP:operand_type_cast_real` | - |
| 58 | `[2.2.2] Enum label reference primary semantics` | covered | `RTP:enum_label_reference` | - |
| 59 | `[2.2.2] Enum label reference invalid cases` | covered | `RTN:invalid_enum_label_on_non_enum`, `RTN:invalid_missing_enum_label`, `RTN:invalid_missing_enum_metadata` | - |
| 60 | `[2.2.2] Selection primary syntax matrix` | partial | `IBP:dynamic_bit_select_valid`, `IBP:part_select_and_indexed_part_select` | `base -: width` does not have a positive case. |
| 61 | `[2.2.2] Concatenation primary syntax` | covered | `IBP:concat_replication_inside_conditional`, `IBP:concatenation_select_uses_derived_width`, `IBP:iff_inside_and_concatenation` | - |
| 62 | `[2.2.2] Replication primary syntax` | covered | `IBP:concat_replication_inside_conditional`, `IBP:replication_select_indexes_replicated_bits`, `IBN:invalid_const_replication_multiplier` | Zero-replication is not covered, but the primary surface is. |
| 63 | `[2.2.2] .triggered is the only supported member-like primary` | partial | `RTP:triggered_logical_primary`, `RTN:invalid_triggered_on_non_event` | There is no direct `.foo` rejection case. |
| 64 | `[2.2.2] .triggered is restricted to raw event references and may not chain` | covered | `RTP:triggered_logical_primary`, `RTN:invalid_triggered_on_non_event`, `RTN:invalid_chained_triggered` | - |
| 65 | `[2.3.1] Cast syntax` | covered | `IBP:integral_cast_integer_like_target`, `IBP:integral_cast_bit_logic_variants`, `RTP:real_cast_truncates_to_int`, `RTP:operand_type_cast_bit_vector` | - |
| 66 | `[2.3.2] Supported explicit cast-target families` | partial | `IBP:integral_cast_bit_logic_variants`, `IBP:integral_cast_integer_like_target`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | Explicit `unsigned` target coverage is still missing. |
| 67 | `[2.3.3] Casts may change width, signedness, state domain, and operand kind` | partial | `IBP:integral_cast_integer_like_target`, `IBP:integral_cast_bit_logic_variants`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | Broad behavior is covered, but not as a full matrix. |
| 68 | `[2.3.3] Integral casts use deterministic bit-level conversion` | partial | `IBP:integral_cast_integer_like_target`, `IBP:integral_cast_bit_logic_variants` | Not exhaustively matrixed. |
| 69 | `[2.3.4] Signedness-only casts are width-preserving reinterpretations` | partial | `IBP:integral_cast_integer_like_target`, `IBP:signed_division_preserves_twos_complement_result`, `IBP:signed_modulo_preserves_twos_complement_result` | Width-preserving reinterpretation is implied, not directly isolated. |
| 70 | `[2.3.4] Signedness-only casts do not change the state domain` | partial | `IBP:integral_cast_integer_like_target`, `IBP:integral_cast_bit_logic_variants` | State-domain preservation for signed/unsigned-only casts is not isolated. |
| 71 | `[2.3.4] Signedness-only casts are valid only for integral operands` | partial | `IBP:integral_cast_integer_like_target`, `RTP:enum_cast_reacquires_label` | Integral-family sampling is partial. |
| 72 | `[2.3.4] Signedness-only casts reject real, string, and event` | none | - | No explicit rejection tests for `signed'`/`unsigned'` on non-integral kinds. |
| 73 | `[2.3.5] Widening resize extension rules` | partial | `IBP:integral_cast_integer_like_target` | Sign extension is covered; zero-extension matrix is missing. |
| 74 | `[2.3.5] Narrowing integral casts keep least-significant bits` | partial | `IBP:integral_cast_bit_logic_variants`, `RTP:operand_type_cast_bit_vector` | Narrowing LSB retention is not isolated directly. |
| 75 | `[2.3.5] Post-resize signedness comes from the target type` | partial | `IBP:signed_division_preserves_twos_complement_result`, `IBP:signed_modulo_preserves_twos_complement_result` | Target-signedness-after-resize is not isolated directly. |
| 76 | `[2.3.6] Integral state-domain conversion rules` | partial | `IBP:integral_cast_bit_logic_variants` | 4-state to 2-state zeroing is covered; explicit 4-state-target preservation is not isolated. |
| 77 | `[2.3.7] Inline vector cast-target semantics matrix` | partial | `IBP:integral_cast_bit_logic_variants` | Full inline target matrix is missing. |
| 78 | `[2.3.7] bit and logic are width-1 aliases` | none | - | No `bit'(x)` vs `bit[1]'(x)` / `logic` alias equivalence test. |
| 79 | `[2.3.8] Operand-type casts use recovered operand metadata` | covered | `RTP:operand_type_cast_bit_vector`, `RTP:operand_type_cast_integer_like`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | - |
| 80 | `[2.3.8] Operand-type casts support non-enum recovered types` | covered | `RTP:operand_type_cast_bit_vector`, `RTP:operand_type_cast_integer_like`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | - |
| 81 | `[2.3.8] Operand-type casts are the primary route into dump-derived enum types` | covered | `RTP:enum_cast_reacquires_label` | - |
| 82 | `[2.3.9] Enum label references are primaries, not cast targets` | partial | `RTP:enum_label_reference`, `RTN:invalid_enum_label_on_non_enum` | Primary use is covered, but cast-target misuse is not sampled. |
| 83 | `[2.3.10] Enum casts use underlying bit-vector values` | covered | `RTP:enum_cast_reacquires_label` | - |
| 84 | `[2.3.10] Enum casts apply target enum width and label set` | partial | `RTP:enum_cast_reacquires_label` | No comparison across distinct enum types. |
| 85 | `[2.3.10] Enum casts keep labels on exact bit-pattern matches` | covered | `RTP:enum_cast_reacquires_label` | - |
| 86 | `[2.3.10] Enum casts yield unlabeled enum values on non-matching patterns` | none | - | No unlabeled enum-cast result case. |
| 87 | `[2.3.11] real'(integral) is numeric conversion from the source's self-determined value` | partial | `RTP:operand_type_cast_real`, `RTP:real_mixed_numeric_result` | Self-determined-source nuance is not isolated. |
| 88 | `[2.3.11] real'(integral) rejects x and z` | partial | `RTN:invalid_real_cast_from_unknown_bits` | Covers `x`; there is no explicit `z` source case. |
| 89 | `[2.3.11] Real-to-integral casts truncate toward zero before target rules` | partial | `RTP:real_cast_truncates_to_int` | Positive truncation is covered; negative truncation is missing. |
| 90 | `[2.3.12] string'(string) identity cast` | covered | `RTP:string_conditional_result`, `RTP:operand_type_cast_string` | - |
| 91 | `[2.3.12] Other explicit string casts are invalid` | partial | `RTN:invalid_string_cast_from_integral` | Only integral-to-string rejection is direct; other directions are missing. |
| 92 | `[2.3.13] Raw event is not castable, but .triggered follows ordinary integral cast rules` | partial | `RTN:invalid_event_cast`, `RTP:triggered_logical_primary` | Raw-event rejection is covered; `.triggered` cast success is not direct. |

### Conversion and Typing Rules (`93`-`108`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 93 | `[2.4] Common-type conversion is the default outside self-determined operands` | partial | `IBP:precedence_and_associativity_core`, `RTP:real_mixed_numeric_result`, `IBP:concat_replication_inside_conditional` | Common-type default is exercised, but not isolated broadly. |
| 94 | `[2.4.1] Boolean-context consumers` | covered | `ERP:iff_bounded_ops_and_literals`, `event_runtime_short_circuit_holds`, `integral_boolean_short_circuit_preservation_holds`, `RTP:real_condition_in_ternary` | - |
| 95 | `[2.4.1] Integral boolean-context truth table` | partial | `ERP:iff_unknown_suppresses_event`, `event_runtime_short_circuit_holds` | The full integral truth table is not parameterized. |
| 96 | `[2.4.1] Real boolean-context truth table` | covered | `RTP:real_truthiness_not`, `RTP:real_logical_and_zero`, `RTP:real_logical_or_value`, `RTP:real_condition_in_ternary` | - |
| 97 | `[2.4.1] Strings do not implicitly convert in boolean context` | covered | `RTN:invalid_string_boolean_context` | - |
| 98 | `[2.4.1] Boolean conversion is temporary 1-bit truth only` | partial | `RTP:real_condition_in_ternary`, `ERP:iff_bounded_ops_and_literals` | Temporary-truth-only behavior is not isolated. |
| 99 | `[2.4.2] Integral common-type family and default width rule` | partial | `IBP:dynamic_bit_select_valid`, `IBP:part_select_and_indexed_part_select`, `IBP:concat_replication_inside_conditional` | The default common-width rule is not isolated. |
| 100 | `[2.4.2] Integral common sign and state rules` | partial | `IBP:integral_cast_integer_like_target`, `IBP:integral_cast_bit_logic_variants` | Mixed implicit sign/state matrix is missing. |
| 101 | `[2.4.2] Integral extension and domain-conversion rules` | partial | `IBP:integral_cast_bit_logic_variants`, `IBP:dynamic_bit_select_unknown_index` | Implicit pre-operator conversion is not isolated directly. |
| 102 | `[2.4.3] Enum participation and label-loss rules` | partial | `IBP:enum_conditional_preserves_identity`, `binder_preserves_enum_identity_in_conditional_arms` | Enum participation outside ternary is not direct. |
| 103 | `[2.4.3] ?: preserves enum type only for matching enum arms` | covered | `IBP:enum_conditional_preserves_identity`, `IBP:conditional_unknown_condition_merges_bits`, `RTP:enum_conditional_merges_unknown_identity`, `binder_preserves_enum_identity_in_conditional_arms` | - |
| 104 | `[2.4.4] Real-family common-type rules` | partial | `RTP:real_mixed_numeric_result`, `RTP:real_power_result` | Mixed arithmetic/power is covered; comparison/equality examples are still missing. |
| 105 | `[2.4.4] Implicit integral-to-real conversion rejects x and z` | none | - | No mixed operator with integral `x`/`z` rejected during implicit promotion. |
| 106 | `[2.4.4] No implicit real/string cross-conversion` | none | - | No direct real/string mixed operator rejection test. |
| 107 | `[2.4.5] String common-type restrictions` | partial | `RTP:string_conditional_result`, `RTP:string_equality_exact`, `RTN:invalid_string_numeric_add`, `RTN:invalid_string_boolean_context` | Same-type positive cases exist, but mixed string/non-string ternary invalidity is missing. |
| 108 | `[2.4.6] Result-type guidance matrix` | partial | `ERP:iff_bounded_ops_and_literals`, `IBP:integral_cast_integer_like_target`, `RTP:real_mixed_numeric_result`, `RTP:string_conditional_result` | Result typing is sampled, not matrixed. |

### Selection, Operators, and Expression Evaluation (`109`-`138`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 109 | `[2.5.3] Selection works only on packed integral values, including derived packed values` | covered | `IBP:dynamic_bit_select_valid`, `IBP:part_select_and_indexed_part_select`, `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits` | - |
| 110 | `[2.5.3] Selection rejects scalar integral, real, and string` | partial | `IBN:invalid_selection_base_scalar_integer_like` | Only scalar-integral rejection is direct; real/string selection negatives are missing. |
| 111 | `[2.5.3] Selection bound and runtime rules matrix` | partial | `IBP:part_select_and_indexed_part_select`, `IBN:invalid_const_part_select_bound`, `IBN:invalid_const_indexed_width` | `-:` and some runtime-invalid variants are missing. |
| 112 | `[2.5.3] Selection result type and identity rules` | partial | `IBP:dynamic_bit_select_valid`, `IBP:concatenation_select_uses_derived_width` | Enum/integer-like identity loss is not checked directly. |
| 113 | `[2.5.3] Invalid bit-select behavior matrix` | partial | `IBP:dynamic_bit_select_out_of_range`, `IBP:dynamic_bit_select_unknown_index` | Only 4-state invalid-index behavior is direct; 2-state branch is missing. |
| 114 | `[2.5.3] Invalid part-select behavior matrix` | none | - | No invalid part-select matrix case. |
| 115 | `[2.5.4] Logical operator acceptance, result type, and short-circuit matrix` | covered | `event_runtime_short_circuit_holds`, `integral_boolean_short_circuit_preservation_holds`, `RTP:real_truthiness_not`, `RTP:real_logical_and_zero`, `RTP:real_logical_or_value`, `RTN:invalid_string_boolean_context` | - |
| 116 | `[2.5.5] Bitwise operator acceptance and common-type rules` | none | - | No dedicated bitwise operator suite yet. |
| 117 | `[2.5.5] Bitwise truth-table controlling-value cases` | none | - | No direct bitwise truth-table cases. |
| 118 | `[2.5.6] Reduction operator rules matrix` | none | - | No reduction operator tests. |
| 119 | `[2.5.7] Exponentiation acceptance and result-family rules` | partial | `RTP:real_power_result`, `IBP:zero_to_negative_power_returns_unknown`, `IBP:nonzero_to_negative_power_rounds_to_zero` | Result-family and self-determined exponent details are incomplete. |
| 120 | `[2.5.7] Exponentiation special-case matrix` | partial | `IBP:zero_to_negative_power_returns_unknown`, `IBP:nonzero_to_negative_power_rounds_to_zero` | Negative-exponent special cases are covered; exponent-zero is missing. |
| 121 | `[2.5.8] Arithmetic supported-operator family` | partial | `IBP:precedence_and_associativity_core`, `IBP:signed_division_preserves_twos_complement_result`, `IBP:signed_modulo_preserves_twos_complement_result` | Unary `+`, unary `-`, binary `-`, and `*` are still missing. |
| 122 | `[2.5.8] Arithmetic common-type and promotion rules` | partial | `RTP:real_mixed_numeric_result`, `IBP:precedence_and_associativity_core` | Mixed/pure arithmetic is sampled lightly, not as a matrix. |
| 123 | `[2.5.8] Integral arithmetic x/z contagion and zero-divisor matrix` | partial | `IBP:unknown_sample_defaults_to_all_x` | x/z contagion and zero-divisor coverage is very incomplete. |
| 124 | `[2.5.8] Integral division truncates toward zero` | none | - | No non-integer quotient case. |
| 125 | `[2.5.9] Shift acceptance and width/count rules` | none | - | No shift suite. |
| 126 | `[2.5.9] Shift fill-rule matrix` | none | - | No shift suite. |
| 127 | `[2.5.9] Unknown shift counts yield all-x` | none | - | No shift suite. |
| 128 | `[2.5.10] Comparison acceptance, promotion, and result rules` | partial | `ERP:iff_bounded_ops_and_literals`, `RTP:rich_iff_with_triggered_real_and_string` | Comparison coverage is light. |
| 129 | `[2.5.11] Equality operator acceptance matrix` | partial | `IBP:wildcard_and_case_equality`, `IBP:wildcard_equality_matches_unknown_bits`, `IBP:case_equality_distinguishes_unknown_bits`, `RTP:string_equality_exact` | Full operator-family acceptance matrix is missing. |
| 130 | `[2.5.11] Integral equality result semantics matrix` | partial | `IBP:wildcard_and_case_equality`, `IBP:wildcard_equality_matches_unknown_bits`, `IBP:case_equality_distinguishes_unknown_bits`, `wildcard_equality_preserves_unknown_from_lhs` | No full equality-family matrix or mixed integral/real case. |
| 131 | `[2.5.11] String equality exact-match semantics matrix` | partial | `RTP:string_equality_exact` | Only a direct exact-match positive case exists. |
| 132 | `[2.5.12] Conditional operator condition and arm-resolution rules` | covered | `RTP:real_condition_in_ternary`, `IBP:enum_conditional_preserves_identity`, `RTP:string_conditional_result` | - |
| 133 | `[2.5.12] Conditional x/z merge rules matrix` | covered | `IBP:conditional_unknown_condition_merges_bits`, `RTP:enum_conditional_merges_unknown_identity`, `IBP:enum_conditional_preserves_identity` | - |
| 134 | `[2.5.13] inside accepted narrowed form and item kinds` | covered | `IBP:concat_replication_inside_conditional`, `IBP:inside_returns_unknown_for_unknown_membership`, `IBP:iff_inside_and_concatenation` | - |
| 135 | `[2.5.13] inside matching semantics and x result rule` | partial | `IBP:wildcard_and_case_equality`, `IBP:inside_returns_unknown_for_unknown_membership`, `IBP:concat_replication_inside_conditional` | Matching and ambiguous-`x` are covered, but the direct-miss matrix is not isolated. |
| 136 | `[2.5.13] Unsupported advanced inside forms matrix` | none | - | No advanced `inside` negative suite. |
| 137 | `[2.5.14] Concatenation and replication acceptance and identity rules` | covered | `IBP:concat_replication_inside_conditional`, `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits`, `binder_rejects_unsized_concat_literal` | - |
| 138 | `[2.5.14] Concatenation and replication width/state/value rules matrix` | partial | `IBP:concat_replication_inside_conditional`, `IBN:invalid_const_replication_multiplier`, `binder_rejects_unsized_concat_literal` | Width/state/value matrix is incomplete. |

### Precedence and Grammar (`139`-`168`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 139 | `[2.6] Primary and postfix forms bind tighter than infix operators` | covered | `IBP:precedence_and_associativity_core`, `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits`, `RTP:enum_label_reference` | - |
| 140 | `[2.6] Concatenation and replication behave as primary expressions in mixed expressions` | covered | `IBP:concat_replication_inside_conditional`, `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits` | - |
| 141 | `[2.6] .triggered has postfix binding like selection` | partial | `RTP:triggered_logical_primary`, `RTP:rich_iff_with_triggered_real_and_string` | Postfix binding relative to neighboring operators is not isolated. |
| 142 | `[2.6] Postfix forms chain left-to-right` | partial | `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits` | No repeated postfix chain like `expr[a][b]`. |
| 143 | `[2.6] Prefix unary operators bind to the following operand` | partial | `RTP:real_truthiness_not`, `IBP:integral_cast_integer_like_target` | Prefix grouping is not isolated. |
| 144 | `[2.6] Exponentiation precedence relative to unary and multiplicative operators` | none | - | No precedence test around `**`. |
| 145 | `[2.6] Multiplicative, additive, and shift precedence chain` | partial | `IBP:precedence_and_associativity_core` | Multiplicative-vs-additive is direct; the shift tier is missing. |
| 146 | `[2.6] Relational and inside precedence relative to shifts and equality` | partial | `IBP:concat_replication_inside_conditional`, `IBP:iff_inside_and_concatenation` | Mixed shift/relational/equality precedence is not isolated. |
| 147 | `[2.6] Equality and bitwise precedence chain` | none | - | No direct equality-vs-bitwise chain test. |
| 148 | `[2.6] Logical and conditional precedence chain` | none | - | No direct unparenthesized logical-vs-ternary precedence test. |
| 149 | `[2.6] Binary infix operators associate left-to-right` | partial | `typed_parser_preserves_iff_binding_to_single_term`, `IBP:precedence_and_associativity_core` | Broad binary left-associativity is not isolated. |
| 150 | `[2.6] Conditional operator associates right-to-left` | none | - | No nested ternary associativity test. |
| 151 | `[2.6] Casts and enum-label primaries bind as primary expressions` | partial | `RTP:enum_label_reference`, `RTP:string_conditional_result`, `logical_parser_accepts_rich_type_surface_sample` | Primary-vs-infix ambiguity is not isolated directly. |
| 152 | `[2.7] Start symbol and ternary grammar` | partial | `logical_parser_accepts_integral_boolean_surface_sample`, `logical_parser_accepts_rich_type_surface_sample` | Bare logical-or and nested ternary grammar are not isolated directly. |
| 153 | `[2.7] Logical-chain grammar for || and &&` | partial | `ERP:iff_bounded_ops_and_literals`, `event_runtime_short_circuit_holds`, `RTP:rich_iff_with_triggered_real_and_string` | Repeated `&&` chains exist; repeated `||` chains are light. |
| 154 | `[2.7] Bitwise, equality, and relational chain grammar` | partial | `IBP:wildcard_and_case_equality`, `IBP:inside_returns_unknown_for_unknown_membership` | Repeated bitwise/equality/relational chains are not isolated. |
| 155 | `[2.7] Shift, additive, multiplicative, and power chain grammar` | partial | `IBP:precedence_and_associativity_core`, `RTP:real_power_result` | Repeated chains at each level are not covered. |
| 156 | `[2.7] Recursive unary grammar` | none | - | No stacked unary-chain test. |
| 157 | `[2.7] Repeated postfix grammar` | partial | `IBP:concatenation_select_uses_derived_width`, `IBP:replication_select_indexes_replicated_bits` | No true repeated postfix sequence. |
| 158 | `[2.7] Postfix suffix grammar matrix` | partial | `RTP:triggered_logical_primary`, `IBP:dynamic_bit_select_valid`, `IBP:part_select_and_indexed_part_select` | `-:` is still missing. |
| 159 | `[2.7] Primary-expression grammar matrix` | covered | `logical_parser_accepts_integral_boolean_surface_sample`, `logical_parser_accepts_rich_type_surface_sample`, `IBP:concat_replication_inside_conditional`, `RTP:enum_label_reference` | - |
| 160 | `[2.7] Concatenation and replication grammar details` | partial | `IBP:concat_replication_inside_conditional`, `IBP:replication_select_indexes_replicated_bits`, `binder_rejects_non_constant_replication_multiplier` | Grammar details are not isolated as their own suite. |
| 161 | `[2.7] inside set grammar details` | covered | `IBP:concat_replication_inside_conditional`, `IBP:inside_returns_unknown_for_unknown_membership`, `IBP:iff_inside_and_concatenation` | - |
| 162 | `[2.7] Literal grammar matrix` | covered | `logical_parser_accepts_integral_boolean_surface_sample`, `lex_logical_expr_accepts_real_and_string_literals` | - |
| 163 | `[2.7] type_reference grammar matrix` | partial | `IBP:integral_cast_bit_logic_variants`, `IBP:integral_cast_integer_like_target`, `RTP:operand_type_cast_real`, `RTP:operand_type_cast_string` | Explicit `unsigned` target coverage is still missing. |
| 164 | `[2.7] bit_vector_type grammar details` | partial | `IBP:integral_cast_bit_logic_variants`, `IBN:invalid_cast_target_width` | Signed/unsigned and bare-form matrix is incomplete. |
| 165 | `[2.7] integer_like_type grammar keyword matrix` | partial | `IBP:integral_cast_integer_like_target`, `RTP:operand_type_cast_integer_like` | All six keywords are not sampled. |
| 166 | `[2.7] Hierarchical operand references are part of the grammar surface` | none | - | No hierarchical operand-reference test. |
| 167 | `[2.7] .triggered is a reserved postfix form with semantic restrictions` | covered | `lex_logical_expr_reserves_triggered_suffix`, `RTP:triggered_logical_primary`, `RTN:invalid_triggered_on_non_event`, `RTN:invalid_chained_triggered` | - |
| 168 | `[2.7] Semantic validation for enum labels, constant expressions, and inclusive ranges` | partial | `RTP:enum_label_reference`, `RTN:invalid_enum_label_on_non_enum`, `RTN:invalid_missing_enum_label`, `IBN:invalid_const_part_select_bound` | Inclusive range semantics are not isolated. |

### Late Checklist Additions (`169`-`214`)

| ID | Checklist point | Status | Current tests | Notes |
| --- | --- | --- | --- | --- |
| 169 | `[1] Event syntax rejects the outer @(...) wrapper` | none | - | No `@(...)` wrapper rejection case. |
| 170 | `[2.3.13] .triggered follows ordinary integral cast semantics` | none | - | No cast-equivalence test comparing `.triggered` to an ordinary bit. |
| 171 | `[2.4.4] Implicit integral-to-real promotion uses the source's self-determined value` | none | - | No direct self-determined implicit-promotion test. |
| 172 | `[2.5.1] Non-whitelisted operator syntax is rejected` | none | - | Unsupported operators like `++`, `~&`, and `~|` are not tested. |
| 173 | `[2.5.3] Bit-select unknown indices use the invalid-index rule` | partial | `IBP:dynamic_bit_select_unknown_index` | Only the 4-state branch is covered. |
| 174 | `[2.6] Exponentiation chains associate left-to-right` | none | - | No `a ** b ** c` associativity test. |
| 175 | `[2.5.3] Valid selection read semantics matrix` | partial | `IBP:dynamic_bit_select_valid`, `IBP:part_select_and_indexed_part_select` | Valid `-:` and state-domain preservation are not direct. |
| 176 | `[2.4.3] Mixed enum implicit conversions drop labels outside same-enum ternary` | none | - | No mixed enum/non-enum implicit-conversion label-drop test. |
| 177 | `[2.7] type_reference rejects unsupported cast-target syntax` | none | - | Unsupported targets like `event'`, `logic[7:0]'`, and `type(a + b)` are not tested. |
| 178 | `[2.7] Only exact type(enum_operand_reference)::LABEL scope syntax is accepted` | none | - | No `pkg::ID` / `enum_t::LABEL` rejection tests. |
| 179 | `[2.7] Event-expression-only syntax is rejected in boolean expressions` | none | - | No direct logical-expression rejection of event-only syntax. |
| 181 | `[1.4] iff gates event selection by boolean truth` | covered | `ERP:iff_unknown_suppresses_event`, `ERP:iff_bounded_ops_and_literals`, `event_runtime_short_circuit_holds` | - |
| 182 | `[2.2.1] Plain unsized decimal integers reject x and z` | none | - | No decimal-literal-with-`x`/`z` rejection test. |
| 183 | `[1.4][2.7] iff guard rejects event-expression-only syntax` | none | - | No event-only syntax rejection inside an `iff` guard. |
| 184 | `[1.4][2.4.1] iff guard enforces boolean-context operand restrictions` | partial | `RTN:invalid_string_boolean_context`, `RTP:rich_iff_with_triggered_real_and_string` | Valid boolean-context-rich guards exist, but guard-specific invalid cases are missing. |
| 185 | `[1.2][1.6] Edge-event operand-reference resolution matrix` | none | - | No edge-event simple/hierarchical/canonical resolution matrix. |
| 186 | `[1.4][1.5] Mixed union spellings remain semantically identical in multi-term chains` | partial | `event_runtime_shadow_parity_matches_legacy_event_matches_for_non_iff_surface` | No mixed multi-term chain equivalence case. |
| 187 | `[2.3.8][2.3.12] Recovered string-type casts obey string-only identity rules` | partial | `RTP:operand_type_cast_string`, `RTN:invalid_string_cast_from_integral` | Positive recovered-string identity exists, but recovered-string negative loopholes are missing. |
| 188 | `[2.5.13] inside rejects non-integral lhs and set items` | none | - | No non-integral `inside` lhs/item rejection cases. |
| 189 | `[2.6] Bitwise OR binds tighter than logical AND` | none | - | No `a | b && c` precedence test. |
| 190 | `[2.7] Semantic validation enforces packed-only selection and operator type restrictions` | partial | `IBN:invalid_selection_base_scalar_integer_like`, `RTN:invalid_string_numeric_add`, `RTN:invalid_string_boolean_context` | Broader semantic-vs-grammar distinction is still thin. |
| 191 | `[2.7] Brace-form grammars reject empty and malformed concatenation, replication, and inside sets` | partial | `IBN:malformed_inside_set`, `IBN:malformed_concatenation` | Malformed replication forms are still missing. |
| 192 | `[1.4][2.1][2.2.2][2.4.1] iff guards reject raw events but accept .triggered` | partial | `RTP:rich_iff_with_triggered_real_and_string`, `RTP:triggered_logical_primary` | `.triggered` acceptance is covered; raw-event guard rejection is missing. |
| 193 | `[2.2.2][2.7] .triggered rejects parenthesized raw-event forms` | none | - | No `(ev).triggered` rejection test. |
| 194 | `[2.2.2] Boolean-expression operand references must resolve to supported operands` | partial | `IBN:unresolved_signal_logical`, `RTN:invalid_recovered_type_unknown_signal` | Unknown identifiers are covered; non-value target cases are missing. |
| 195 | `[2.2.1] Unsupported unbased unsized integral literals are rejected` | none | - | No `'0`, `'1`, `'x`, `'z` rejection tests. |
| 196 | `[2.3.8][2.3.9] type(operand_reference) requires a resolvable operand reference` | partial | `RTN:invalid_recovered_type_unknown_signal` | Cast unknown-source is covered; enum-label unknown-source is missing. |
| 197 | `[2.3.8][2.3.11] Recovered real-type casts obey ordinary real-cast x/z rejection` | none | - | No recovered-real cast with `x`/`z` rejection case. |
| 198 | `[2.5.14][2.7] Unsupported brace-based expression syntax is rejected` | none | - | Unsupported brace syntaxes like streaming concatenation are not tested. |
| 199 | `[2.5.13][2.7] inside requires the exact braced set surface` | none | - | Non-braced `inside` RHS rejection is not tested. |
| 200 | `[2.5.7][2.5.8] Arithmetic and exponentiation reject non-numeric operands` | partial | `RTN:invalid_string_numeric_add` | String arithmetic rejection exists, but unary and exponentiation negatives are missing. |
| 201 | `[2.6][2.7][2.5.3] Mixed postfix .triggered and selection chains obey left-to-right parsing and semantic restrictions` | none | - | No mixed `.triggered` / selection chain tests. |
| 202 | `[2.5.3][2.5.14][2.7] Integer-only constant-expression positions reject constant non-integer operands` | none | - | Only non-constant variable negatives exist today. |
| 203 | `[1.2][1.6] Event operand references use host-provided resolution` | none | - | No host-resolution variation test for event refs. |
| 204 | `[2.3.8][2.3.13] Recovered raw event types are invalid cast targets` | none | - | No `type(event_ref)'(...)` rejection case. |
| 206 | `[2.2.2][2.7] Non-whitelisted call-like primary syntax is rejected` | none | - | No call-like primary rejection tests. |
| 207 | `[1.1][1.6] Wildcard gated-event form * iff logical_expr` | none | - | No `* iff ...` case. |
| 208 | `[2.3.6][2.3.8][2.3.10] Recovered enum casts apply state-domain rules before label resolution` | none | - | No recovered-enum cast state-domain-before-label test. |
| 209 | `[2.5.3][2.7] Malformed selection postfix syntax is rejected` | none | - | Malformed selection postfix syntax is not tested. |
| 210 | `[2.5.13][2.7] Malformed inside range-item syntax is rejected` | none | - | Malformed `inside` range items are not tested. |
| 211 | `[1.6][2.2.2][2.7] Section-2 operand references use shared host resolution` | none | - | No shared-host-resolution equivalence test for section-2 refs. |
| 212 | `[1.2][1.6][2.2.2][2.7] Section-2 operand references accept canonical dump-derived signal tokens` | none | - | No canonical dump-token logical operand case. |
| 213 | `[2.5.2][2.5.14] Concatenation operands stay self-determined before packing` | none | - | No concatenation self-determined-before-packing contrast test. |
| 214 | `[2.5.3] Indexed part-select base is evaluated at the current sample` | none | - | No time-varying indexed part-select base case. |

## Infra-Only / Harness-Only Coverage

These tests are important guardrails, but they do not map cleanly to a single
checklist point:

- `tests/expression_parse.rs::parse_no_panic_corpus_holds`: parser robustness
  against malformed event-input fuzz seeds.
- `tests/expression_fixture_contract.rs::*`: manifest schema, suite ownership,
  runtime-timestamp requirements, and snapshot hygiene.
- `src/expr/lexer.rs::lex_event_expr_tracks_keywords_and_spans`: lexer token/span
  regression coverage.
- `src/expr/parser.rs::typed_parser_rejects_unmatched_open_parenthesis` and
  `src/expr/parser.rs::typed_parser_rejects_broken_union_segmentation`: direct
  parser diagnostic regressions already reinforced by manifest negatives.
- `src/expr/eval.rs::logical_and_short_circuits_rhs`: evaluator-level
  short-circuit regression on a hand-built bound tree.
