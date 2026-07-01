# Implement `wavepeek extract generic`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

This change adds `wavepeek extract generic`, a protocol-neutral command for row-oriented extraction of synchronous waveform events. After the change, a user can describe one or more event sources, select clock edges, filter each selected edge with a Boolean predicate, and emit payload signal values sampled from the same pre-edge point as deterministic human, JSON, or JSONL output. This replaces the current multi-step workflow of running `property --capture match`, running `value` at returned sample times, and joining the results outside `wavepeek`.

A visible success case is a ready/valid handshake query such as `wavepeek extract generic --waves dump.vcd --scope top.dut --on "posedge clk iff rst_n" --when "valid && ready" --payload data,last --jsonl`, which emits one stream item per matching clock edge, including repeated transfers with identical payload values.

## Non-Goals

This plan does not implement protocol-specific extractors such as AXI, AXI-Stream, APB, AHB, or Wishbone. It does not decode bursts, track outstanding transactions, reconstruct higher-level transactions, or validate protocol rules. It does not add native timestamp sampling, wildcard triggers, plain-signal triggers, mixed edge/plain triggers, or typed protocol-specific output rows. The only sampling semantics for `extract generic` are RTL-style pre-edge sampling.

## Progress

- [x] (2026-06-28T12:06Z) Read the local proposal at `tmp/extract-generic-proposal-v3.ru.md` and issue context for kleverhq/wavepeek#35.
- [x] (2026-06-28T12:06Z) Inspected existing command, expression, output, schema, docs, and test architecture relevant to `change`, `property`, `value`, JSONL streaming, and schema generation.
- [x] (2026-06-28T12:06Z) Created this ExecPlan as the branch-local implementation artifact in `docs/tracker/wip/extract-generic-execplan.md`.
- [x] (2026-06-28T12:12Z) Committed the initial ExecPlan as `1a83a76 docs(tracker): plan generic extraction`.
- [x] (2026-06-28T12:22Z) Ran focused read-only review lanes for engine architecture, schema/tooling, and docs/tests, then recorded required plan changes.
- [x] (2026-06-28T12:28Z) Committed the reviewed ExecPlan revision.
- [x] (2026-06-28T13:28Z) Implemented CLI, input parsing, engine execution, human output, JSON output, JSONL output, schema contracts, schema tooling, docs publication tooling, public docs, packaged skill guidance, and targeted tests.
- [x] (2026-06-28T13:30Z) Regenerated schemas and ran targeted validation: `just check-schema`, `cargo test --test docs_cli`, `cargo test --test skill_cli`, `cargo test --test schema_cli`, `cargo test --test cli_contract`, `cargo test --test extract_generic_cli`, `cargo test --test jsonl_cli`, `cargo test --test extract_generic_vcd_fst_parity`, `cargo test --test fsdb_disabled_cli`, `python3 -m unittest discover -s tools/docs -p 'test_*.py'`, and `python3 -m unittest discover -s tools/schema -p 'test_*.py'`.
- [x] (2026-06-28T14:53Z) Ran local pre-handoff gate `just check`; it passed, including schema checks, docs-site check, commit message check, and FSDB checks because Verdi was available.
- [x] (2026-06-28T13:51Z) Committed the implementation slice as `8c2a039 feat(extract): add generic waveform extraction`; commit hooks passed Rust format, clippy, build, schema contract, Rust tests, auxiliary tests, FSDB tests, benchmark smoke, and commit style.
- [x] (2026-06-28T14:24Z) Ran implementation review lanes for code/runtime, schema/tooling, and docs/tests; fixed findings in `ec1892e fix(extract): preflight generic extraction inputs`.
- [x] (2026-06-28T14:47Z) Ran follow-up review lanes for code/runtime and schema/docs collateral; both returned no substantive findings. Initial fresh control-pass attempts returned empty output, so they were not counted.
- [x] (2026-06-28T15:12Z) Re-ran final control review as two narrow independent lanes for runtime and schema/tooling; both returned no substantive findings.
- [x] (2026-06-28T14:58Z) Opened draft PR https://github.com/kleverhq/wavepeek/pull/46.
- [x] (2026-06-29T20:59Z) Collected maintainer PR review comments, renamed public extract docs from the concrete `commands/extract-generic` topic to the `commands/extract` command-family topic, renamed the handshake workflow to `workflows/extract-handshake`, updated README/help/docs/skill wording, and ran targeted docs/help validation.
- [x] (2026-06-29T21:08Z) Ran focused review for the PR-comment fixes, tightened CLI and skill tests against stale `commands/extract-generic` links, ran follow-up review with no substantive findings, and reran `just check` successfully.
- [x] (2026-06-29T21:18Z) Applied the remaining live CLI-help review suggestion for the `wavepeek schema --input` reference and updated the help contract test.
- [x] (2026-06-30T19:39Z) Fixed the `wavepeek extract` namespace invocation to print short help like `wavepeek docs` instead of returning a fatal missing-subcommand error, and added CLI contract coverage.
- [x] (2026-06-30T20:19Z) Profiled `extract generic` on the SCR1 AXI DMEM FST case, added DEBUG-only extract timing counters for time parsing, candidate collection, event matching, predicate evaluation, and row build/emit, and reviewed the instrumentation.
- [x] (2026-07-01T05:56Z) Add SCR1 AXI DMEM extract e2e benchmark scenarios for one CLI-defined channel, two JSON-defined channels, and five JSON-defined channels to the FST and FSDB catalogs. Validated with `python3 -m unittest bench.e2e.test_perf` and `just check-bench-e2e-fsdb-catalog`.
- [x] (2026-07-01T06:29Z) Run the new extract benchmark scenarios on the current implementation and confirm the expected poor scaling. FST medians were `1.445s` for 1ch, `2.548s` for 2ch, and `5.705s` for 5ch. FSDB medians were `25.123s` for 1ch, `29.984s` for 2ch, and `44.790s` for 5ch.
- [x] (2026-07-01T06:38Z) Implement event-expression grouping for `extract generic` without changing benchmark definitions or command output contracts. The implementation groups by original `on` text plus resolved event candidate signal IDs, collects candidate timestamps per group, and emits matching sources in declaration order.
- [x] (2026-07-01T07:00Z) Validate the grouping fix with DEBUG diagnostics and focused review. DEBUG diagnostics show one event group and `1,248,862` event checks for both 2-channel and 5-channel SCR1 AXI source-file runs. Initial code/performance review found DEBUG truncation counter semantics and three hot-path concerns; fixes were applied and both reviewers rechecked with no substantive findings. A fresh control pass over the consolidated grouping diff returned no substantive findings.
- [x] (2026-07-01T07:29Z) Run the extract e2e benchmark scenarios as the final performance gate and record the speedup evidence before handoff. FST grouped results showed functional matches and speedups of `1.47x` for 2ch and `2.33x` for 5ch. FSDB grouped results showed functional matches and speedups of `1.02x` for 2ch and `1.06x` for 5ch.
- [x] (2026-07-01T08:38Z) Investigate the current FSDB debug 2-channel critical path in enough detail to identify a concrete optimization path toward FST-comparable performance. Added DEBUG-only FSDB backend counters, ran the current 2-channel FSDB debug workload, and recorded the value-sampling bottleneck and proposed fix under `tmp/extract-fsdb-diagnostics/`.

## Surprises & Discoveries

- Observation: The local proposal is more specific than issue #35 and narrows the first implementation to `extract generic`, edge-only pre-edge semantics, and an input schema.
  Evidence: `tmp/extract-generic-proposal-v3.ru.md` states that `--sample-mode` is not exposed, wildcard/plain/mixed `on` expressions are rejected, and `schema/input.json` plus `wavepeek schema --input` are required.
- Observation: `property` is the closest execution base, while `change` must not be used as the row-emission model.
  Evidence: `property` already emits one selected predicate result per event, while `change` suppresses rows when displayed signal values do not change. `extract generic` must preserve repeated transfers with identical payloads.
- Observation: Schema tooling and docs publication currently assume output and stream schema families only.
  Evidence: `tools/schema-gen/src/main.rs`, `tools/schema/check_schema_contract.py`, `tools/docs/publish_docs.py`, `tools/docs/check_deploy.py`, and the `docs-site-check-deploy` recipe map only output and stream artifacts today.
- Observation: The plan review found that published schema artifact URLs must not be reused for a changed output/stream contract.
  Evidence: The repository has tag `v2.0.0` in history, and `schema/output.json` plus `schema/stream.json` currently publish v2.0 URLs. Adding `extract generic` changes both schema families.
- Observation: The plan review found that `--from` and `--to` must bound selected event times, not pre-edge sample times.
  Evidence: The proposal says a row is skipped when `time - 1` is outside the dump, not when it is before the user-specified `--from`. Existing pre-edge commands keep `time` and `sample_time` as distinct concepts.
- Observation: Manual `$ref` schemas in schemars do not cause referenced definitions to be generated automatically.
  Evidence: `tools/schema-gen --validate tmp/schema-try` initially failed with `Pointer '/$defs/extractGenericSource' does not exist`; explicitly asking `SchemaGenerator` for `ExtractGenericSource` fixed the generated input schema.
- Observation: Docs publication helpers had separate hard-coded paths for catalog copying, deploy validation, workflow metadata forwarding, allowed gh-pages paths, and release tests.
  Evidence: `tools/docs` tests failed until `schema-input-v2.1.json` was added to `publish_docs.py`, `check_deploy.py`, `workflow_docs.py`, and their tests.
- Observation: Implementation review found that JSONL preflight must include `--on ... iff` dependencies and payload encodings, not only `--when` dependencies.
  Evidence: The fix added `event_iff_handles` in `src/engine/expr_runtime.rs`, preflight validation in `src/engine/extract.rs`, and regression tests for event-time `iff` versus sample-time `when` evaluation.
- Observation: The SCR1 AXI DMEM performance baseline is dominated by repeated event matching when several sources share `posedge clk iff axi_rst_n`.
  Evidence: `docs/tracker/wip/extract_generic_perf_baseline.md` records the 2-channel run at `2,497,724` event checks and `23.078s` non-DEBUG wall time to `/dev/null`, while the 5-channel run performs `6,244,310` event checks and takes `56.607s`; the candidate timestamp count stays `1,248,862` in both runs.
- Observation: The FSDB e2e benchmark catalog is generated from the FST catalog by rewriting `/opt/rtl-artifacts/*.fst` strings to `.fsdb`.
  Evidence: `tools/fsdb/generate_bench_catalog.py` walks every JSON string in `bench/e2e/tests.json`; `bench/e2e/test_perf.py` asserts that `tests_fsdb.json` matches the FST catalog except for the artifact extension.
- Observation: The release FST benchmark is much faster than the earlier DEBUG baseline, but it still shows source-count scaling for shared-event source files.
  Evidence: `tmp/bench-extract-baseline-fst/baseline` records medians of `1.445s` for the 1-channel CLI case, `2.548s` for the 2-channel JSON case, and `5.705s` for the 5-channel JSON case.
- Observation: The FSDB benchmark has a large backend cost, but the extract workload still grows materially with channel count.
  Evidence: `tmp/bench-extract-baseline-fsdb/baseline` records medians of `25.123s` for 1 channel, `29.984s` for 2 channels, and `44.790s` for 5 channels.
- Observation: Grouping the shared SCR1 AXI `posedge clk iff axi_rst_n` event reduces event checks to the candidate timestamp count while preserving output.
  Evidence: After the grouping implementation, `DEBUG=1 target/release/wavepeek extract generic ... --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_5ch.json --jsonl` records `event_groups=1`, `candidate_times=1,248,862`, `event_checks=1,248,862`, and `event_match_ns=960,580,774`; `cmp` confirms the grouped 5-channel JSON output matches the pre-grouping release output byte-for-byte.
- Observation: Review found three avoidable costs in the first grouping implementation.
  Evidence: The performance review flagged duplicate candidate collection for same-signal event groups, unconditional pre-emit merged timestamp counting in non-DEBUG runs, and per-timestamp sorting for the common single-matching-group case. The follow-up patch caches candidate streams by sorted candidate `SignalId` set, only computes the merged timestamp count for DEBUG diagnostics, and skips source-index sorting unless more than one event group matches.
- Observation: The final FST benchmark shows the intended speedup on multi-source shared-event workloads, while the one-source CLI case remains effectively unchanged.
  Evidence: `tmp/bench-extract-grouped-fst/grouped` reports functional matches for all three cases. Relative to `tmp/bench-extract-baseline-fst/baseline`, the 2-channel JSON case improved from `2.543s` mean to `1.726s` mean (`1.47x` faster), and the 5-channel JSON case improved from `5.754s` mean to `2.467s` mean (`2.33x` faster). The 1-channel CLI case changed from `1.465s` mean to `1.495s` mean, within the expected overhead/noise for a workload with no grouping opportunity.
- Observation: The final FSDB benchmark also improves the shared-event source-file cases, but backend/system cost dominates absolute runtime.
  Evidence: `tmp/bench-extract-grouped-fsdb/grouped` reports functional matches for all three cases. Relative to `tmp/bench-extract-baseline-fsdb/baseline`, the 2-channel JSON case improved from `30.029s` mean to `29.437s` mean (`1.02x` faster), and the 5-channel JSON case improved from `44.757s` mean to `42.275s` mean (`1.06x` faster).
- Observation: The original 23s and 57s comparison points were `target/debug` JSONL runs, not release runs.
  Evidence: `docs/tracker/wip/extract_generic_perf_baseline.md` records `Binary: ./target/debug/wavepeek` and `Output mode: --jsonl`. Fresh current `target/debug` JSONL runs without `DEBUG` took `16.639s` for 2 channels and `23.247s` for 5 channels, compared with saved baseline files `tmp/axi_extract_nodebug_null.time.json` at `23.078s` and `tmp/axi_extract_5ch_nodebug_null.time.json` at `56.607s`. Fresh current `target/debug` runs with `DEBUG=1` took `18.398s` for 2 channels and `25.900s` for 5 channels.
- Observation: Functional output remained stable in direct JSON and JSONL validation.
  Evidence: Direct current JSON payloads matched saved FST bench baseline/grouped artifacts for 2 channels and 5 channels, with `9,878` and `20,242` rows respectively. Fresh current JSONL outputs from `target/debug` with `DEBUG=1` matched `tmp/axi_extract_profile2.jsonl` and `tmp/axi_extract_5ch_profile.jsonl` byte-for-byte.
- Observation: Direct FSDB diagnostic runs confirm event grouping, but release wall-clock improvement is muted by FSDB backend and system time.
  Evidence: A temporary detached worktree at baseline commit `550e058` was built with `--features fsdb` and compared with current commit `1e313b1`. For `target/fsdb/release` with `DEBUG=1`, 2 channels changed from `28.480s` wall (`10.906s` user, `17.576s` sys) to `25.830s` wall (`9.330s` user, `16.501s` sys), while `event_checks` dropped from `2,497,724` to `1,248,862`. The 5-channel release run changed from `42.787s` wall (`17.738s` user, `25.067s` sys) to `42.302s` wall (`16.613s` user, `25.739s` sys), while `event_checks` dropped from `6,244,310` to `1,248,862`. For `target/fsdb/debug` with `DEBUG=1`, 2 channels changed from `65.456s` to `59.810s`, and 5 channels changed from `110.096s` to `86.205s`. Baseline/current JSONL outputs matched byte-for-byte for all four pairs.
- Observation: The current FSDB 2-channel debug critical path is dominated by millions of one-signal native sample calls from expression evaluation.
  Evidence: With DEBUG-only FSDB backend counters, `target/fsdb/debug/wavepeek extract generic ... --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_2ch.json --jsonl` took `62.470s` wall (`42.061s` user, `20.407s` sys). The run recorded `4,390,756` `sample_resolved_calls`, `4,380,878` expression sample cache misses, only `1` expression sample cache hit, and `35.454s` in native sample calls. Candidate collection was only one call and `0.558s`; signal sessions were opened `8` times and reused `4,390,749` times.
- Observation: A current FST debug 2-channel run with the same source manifest is much faster because it does not pay the FSDB random native sampling cost.
  Evidence: `target/debug/wavepeek extract generic ... --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst ... --jsonl` took `18.443s` wall with `event_match_ns=9.593s` and `predicate_eval_ns=4.470s`, compared with the FSDB debug run at `62.470s` wall, `event_match_ns=39.693s`, and `predicate_eval_ns=19.262s`.

## Decision Log

- Decision: Treat `tmp/extract-generic-proposal-v3.ru.md` as the normative feature specification and issue #35 as background motivation.
  Rationale: The proposal captures updated scope decisions that are more precise than the issue text, including source JSON wrapper shape, pre-edge-only semantics, and input schema publication.
  Date/Author: 2026-06-28 / coding agent.
- Decision: Add `extract` as a top-level namespace with nested `generic`, and use the command string `"extract generic"` in JSON envelopes and JSONL stream records.
  Rationale: Existing nested commands use space-separated command names in machine output. The proposal describes future protocol-specific extractors under the same namespace.
  Date/Author: 2026-06-28 / coding agent.
- Decision: Implement a shared internal extraction plan in `src/engine/extract.rs` rather than encoding the generic command directly around CLI arguments.
  Rationale: The proposal explicitly calls for a reusable internal model that future protocol-specific extractors can build on, and a plan object separates source validation from waveform execution.
  Date/Author: 2026-06-28 / coding agent.
- Decision: Publish updated output, stream, and input schemas as the next schema family minor, using `https://kleverhq.github.io/wavepeek/schema-output-v2.1.json`, `https://kleverhq.github.io/wavepeek/schema-stream-v2.1.json`, and `https://kleverhq.github.io/wavepeek/schema-input-v2.1.json`.
  Rationale: `v2.0.0` exists in repository history, so reusing `schema-output-v2.0.json` or `schema-stream-v2.0.json` for a changed command/schema contract would mutate a published artifact. This intentionally differs from the local proposal's v2.0 example to preserve stable schema URLs.
  Date/Author: 2026-06-28 / coding agent.
- Decision: Add the SCR1 AXI extract performance scenarios to `bench/e2e/tests.json`, store reusable source manifests under `bench/e2e/inputs/`, and regenerate `bench/e2e/tests_fsdb.json` from the FST catalog. Do not add these long-running cases to `bench/e2e/tests_commit.json`.
  Rationale: The release benchmark catalog already carries long-running timing cases with at least ten measured runs and five warmups. The commit-smoke catalog is intentionally small and its tests assert an exact set of cases.
  Date/Author: 2026-07-01 / coding agent.
- Decision: Optimize `extract generic` by grouping sources that share the same resolved event expression, while keeping `when` predicates and payload sampling per source.
  Rationale: Grouping removes duplicate edge and `iff` evaluation for repeated `on` expressions without introducing protocol-specific AXI behavior or changing output ordering, JSON contracts, diagnostics, or `--max` semantics.
  Date/Author: 2026-07-01 / coding agent.
- Decision: Use a conservative event-group key made of the original `on` expression text and the resolved event candidate `SignalId` list.
  Rationale: Every `extract generic` invocation has one global scope, so identical `on` text in the same run resolves through the same namespace. Adding the resolved event candidate IDs prevents accidental grouping if local expression handles differ between hosts. Semantically equivalent but textually different expressions are intentionally not grouped.
  Date/Author: 2026-07-01 / coding agent.
- Decision: Keep the new FSDB backend counters DEBUG-only and surface them through existing extract DEBUG events.
  Rationale: The counters identify backend sampling behavior without changing normal output or non-DEBUG runtime. They expose whether future fixes eliminate native per-value sampling and whether fallback paths remain active.
  Date/Author: 2026-07-01 / coding agent.
- Decision: The likely FSDB performance fix should be a value-change timeline cache or bulk sequential query API, not another source grouping pass.
  Rationale: Source grouping already reduces event checks to one per candidate timestamp. The remaining cost comes from one native random sample call per expression value. FST-comparable performance requires moving repeated sampling out of the emit loop and replacing `ffrGotoXTag` per query with sequential value-change loading for the relevant signals.
  Date/Author: 2026-07-01 / coding agent.

## Outcomes & Retrospective

Implementation and review-fix slices committed. `wavepeek extract generic` now supports single-source CLI mode, source-file mode, pre-edge event-row extraction, human/JSON/JSONL output, v2.1 output and stream schemas, input schema publication, docs/tooling collateral, and targeted regression coverage. Review fixes added preflight validation for `iff` dependencies and payload encodings, independent-clock source-file coverage, corrected docs deploy artifact-family handling, and tightened packaged skill wording. Follow-up reviewers found no substantive issues. The local `just check` gate passed. Final narrow control review lanes for runtime and schema/tooling returned no substantive findings. Draft PR #46 was opened. Maintainer PR review comments then narrowed user-facing docs toward the `extract` command family rather than the first `extract generic` subcommand; docs topic IDs, help cross-links, README wording, packaged skill routing, and affected tests were updated accordingly. A focused review of those PR-comment fixes found only stale-link substring assertion gaps, which were fixed and re-reviewed cleanly. The performance follow-up added committed benchmark workloads for the SCR1 AXI DMEM extraction case, captured local FST/FSDB baseline runs, implemented event-expression grouping, fixed focused review findings, and completed a clean control review. Final e2e benchmarks show functional matches and the expected speedup: FST improves by `1.47x` for 2ch and `2.33x` for 5ch; FSDB improves by `1.02x` for 2ch and `1.06x` for 5ch because FSDB backend/system time dominates. The follow-up FSDB critical-path investigation added DEBUG-only backend counters and found that the remaining gap to FST is caused by millions of native random sample calls during expression evaluation; the next substantial fix should add FSDB value-change timeline caching or a bulk sequential query API.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. A waveform is a simulation dump file such as VCD, FST, or FSDB. A scope is a hierarchical design path, for example `top.dut`. A signal path is a signal name, either canonical such as `top.dut.data` or relative to a selected `--scope` such as `data`. An event expression is the syntax accepted by existing commands such as `property --on`, for example `posedge clk iff rst_n`. A logical expression is the Boolean syntax accepted by `property --eval`, for example `valid && ready`. Pre-edge sampling means the selected event time is kept as the row `time`, while values are read at the previous dump tick and printed as `sample_time`.

The command-line surface is defined under `src/cli/`. The top-level command enum is in `src/cli/mod.rs`. Existing leaf argument structs are in files such as `src/cli/change.rs`, `src/cli/property.rs`, and `src/cli/value.rs`. `src/cli/sampling.rs` defines existing sampling modes, but `extract generic` must not expose `--sample-mode`.

Command execution is routed by `src/engine/mod.rs`. Existing waveform command implementations live in `src/engine/change.rs`, `src/engine/property.rs`, and `src/engine/value.rs`. Expression binding and evaluation helpers live in `src/engine/expr_runtime.rs`. Time parsing and formatting helpers live in `src/engine/time.rs`. Waveform sampling primitives live under `src/waveform/`, especially `src/waveform/mod.rs` and `src/waveform/expr_host.rs`. Diagnostic types and standard warning/error codes are in `src/diagnostic.rs`.

Human output, JSON envelopes, and JSONL stream routing are handled by `src/output.rs` and the contract modules under `src/contract/`. `src/contract/output.rs` defines serializable command output data, `src/contract/stream.rs` defines JSONL records, and `src/contract/schema.rs` generates JSON Schema artifacts. `tools/schema-gen/src/main.rs` writes generated schemas. `tools/schema/check_schema_contract.py` checks committed schema snapshots against generated output and runtime `wavepeek schema` output. The committed generated artifacts are under `schema/` and must be regenerated, not edited manually.

Public docs live under `docs/public/`. Command topics are in `docs/public/commands/`, workflow topics in `docs/public/workflows/`, and stable semantics references in `docs/public/reference/`. The packaged agent skill is `docs/skills/wavepeek.md`. Maintainer architecture and automation docs are under `docs/dev/`.

Tests are in `tests/`. CLI/help coverage lives in `tests/cli_contract.rs`. Runtime command tests for the nearest existing commands are `tests/change_cli.rs`, `tests/property_cli.rs`, `tests/value_cli.rs`, and JSONL tests are in `tests/jsonl_cli.rs`. Schema command tests are in `tests/schema_cli.rs`. Tooling tests live near their helpers, for example `tools/schema/test_check_schema_contract.py` and `tools/docs/test_publish_docs.py`.

## Open Questions

There are no open product-shaping questions at plan creation. If implementation reveals an ambiguity, resolve it in this plan before changing behavior. The expected resolution policy is to match the local proposal unless it conflicts with existing stable repository contracts; in that case prefer the least breaking behavior and record the decision here.

## Plan of Work

The first milestone is to land the ExecPlan and review it before implementation. Commit this file, then run focused review lanes for design correctness, schema/tooling completeness, and test/doc coverage. Incorporate review findings into this plan and commit the revision.

The second milestone is to add the CLI and typed input model without executing waveform extraction yet. Create `src/cli/extract.rs` with an `ExtractCommand` subcommand enum and a `GenericArgs` leaf. Update `src/cli/mod.rs` to include the top-level `extract` namespace, dispatch `extract generic`, add help mutations for the nested leaf, and expose `--json`/`--jsonl` as mutually exclusive modes. `GenericArgs` must accept `--waves`, `--from`, `--to`, `--scope`, `--name`, `--on`, `--when`, `--payload`, `--source`, `--max`, `--abs`, `--json`, and `--jsonl`. The single-source mode requires `--on`, `--when`, and non-empty `--payload`, and defaults source `name` to `transfer` when `--name` is omitted. Source-file mode requires `--source` and rejects explicit `--name`, `--on`, `--when`, and `--payload`. Payload names must follow the same scoped-name discipline as expression names: when `--scope` is set, payload strings are relative to that scope and canonical or scope-prefixed names are rejected instead of silently accepted.

The third milestone is to implement input source parsing and validation. Add an internal input contract type for the source JSON wrapper, either in `src/engine/extract.rs` if it is runtime-only or in `src/contract/input.rs` if it is also used for schema generation. The parsed wrapper must require `$schema`, `kind`, and non-empty `sources`. The `kind` value must be `extract.generic.sources`. Source names must be unique within the file. Each source requires `name`, `on`, `when`, and a non-empty `payload` array. Duplicate payload strings within one source are rejected. The parser must produce a neutral internal `ExtractPlan` with `sources[]`, `declaration_index`, `name`, `on`, `when`, and `payload[]`, preserving source and payload order.

The fourth milestone is to implement the waveform execution engine. Add `src/engine/extract.rs` and wire it from `src/engine/mod.rs` with `Command::ExtractGeneric`, `CommandName::ExtractGeneric`, and `CommandData::ExtractGeneric`. Execution opens the waveform once, parses optional `--from` and `--to` inclusive bounds using the same user-facing time rules as `property` and `change`, binds every source `on` expression with the same scope rules as existing expression commands, rejects any `on` expression that is not edge-only under pre-edge semantics, binds every `when` expression, resolves payload signals fail-fast, validates expression value support for all `when` signal dependencies before any JSONL `begin` record is emitted, and collects candidate event times from all source event handles. User range bounds apply to selected event `time` values only. For each candidate time in ascending order and for each source in declaration order, it checks whether that source `on` matches at the event time. If it matches, it computes the previous dump tick as `sample_time`; if no previous sample time exists or that sample time is outside the dump bounds, the source produces no row for that event. A `sample_time` before `--from` is allowed when the selected event `time` is within range. It evaluates `when` at `sample_time` as a Boolean predicate; if true, it samples payload signals at `sample_time` and emits an `ExtractRow`. The output is sorted by ascending `time` and declaration order by construction. `--max` applies after this ordering across all sources. `--max 0` is an error, `--max unlimited` emits the existing limit-disabled warning, truncation emits the existing output-truncated diagnostic, and valid queries with no rows emit the existing empty-result diagnostic.

The fifth milestone is to add output and streaming contracts. Define an extract row with `time`, `sample_time`, `source`, and ordered `payload` entries. Each payload entry has canonical `path` and formatted Verilog literal `value`. Human output in `src/output.rs` must print a compact line per row: for one source, `@25ns sample@24999ps data=32'hdeadbeef last=1'h1`; for multiple sources, include `[source]` after `sample@...`; and when `--abs` is set, print canonical payload paths in human output. JSON output must use the standard envelope with `command: "extract generic"` and `data` as an array of extract rows. JSONL output must stream existing `begin`, `item`, `diagnostic`, and `end` records with `command: "extract generic"`, and item records must contain the same row shape as JSON output.

The sixth milestone is to add schema artifacts and schema command support. Add `src/contract/input.rs` if needed and update `src/contract/schema.rs` so it can generate `input_schema_json()` and catalog entries for `wavepeek.input`, `wavepeek.output`, and `wavepeek.stream-record` at schema family version 2.1. Add `wavepeek schema --input` in `src/cli/schema.rs` and `src/engine/schema.rs`, making it mutually exclusive with `--stream`; no selector continues to print the output schema for compatibility. Update `src/schema_contract.rs` to embed `CANONICAL_INPUT_SCHEMA_JSON` from `schema/input.json` so runtime schema output can be checked byte-for-byte against the committed snapshot. Update `tools/schema-gen/src/main.rs` to write and validate `input.json`, including semantic valid/invalid source-wrapper samples. Update `tools/schema/check_schema_contract.py`, `tools/schema/test_check_schema_contract.py`, and `tools/schema/README.md` to include the input family. Regenerate `schema/input.json`, `schema/output.json`, `schema/stream.json`, and `schema/catalog.json` with `just update-schema`, then validate with `just check-schema`.

The seventh milestone is to update docs publication and release collateral for the new schema family. Update `tools/docs/publish_docs.py`, `tools/docs/check_deploy.py`, `tools/docs/workflow_docs.py`, `tools/docs/test_publish_docs.py`, `tools/docs/test_check_deploy.py`, `tools/docs/test_workflow_docs.py`, the `docs-site-check-deploy` recipe in `justfile`, and `docs/dev/release.md` so input schema artifacts are copied, forwarded through workflow metadata, checked, and listed with output and stream artifacts. Keep legacy behavior for older releases intact.

The eighth milestone is to update user-facing docs and packaged guidance. Add a public command topic for the `extract` command family at `commands/extract`, update `docs/public/commands/overview.md`, `docs/public/commands/schema.md`, `docs/public/reference/command-model.md`, `docs/public/reference/machine-output.md`, `docs/public/reference/expression-language.md`, `docs/public/intro.md`, and `docs/public/workflows/` with a workflow that shows extracting synchronous events and handshakes. `reference/expression-language.md` must explicitly include `extract generic --on` and `extract generic --when` as current command examples, explain edge-only pre-edge restrictions, and distinguish `iff` event-time gating from sample-time `when` evaluation. Update `docs/skills/wavepeek.md` only as a short routing addition. Update `docs/dev/architecture.md`, `docs/dev/automation.md`, `docs/dev/quality.md`, and release/schema notes where they mention the set of current schemas or command families.

The ninth milestone is to add tests. Update `tests/cli_contract.rs` to include `extract` and nested help coverage. Add runtime tests, preferably in a new `tests/extract_generic_cli.rs`, for single-source CLI extraction, source-file multi-source extraction, repeated identical payload preservation, declaration-order tie-breaking, inclusive event-time bounds, `sample_time` before `--from` at a range-start edge, `iff` event-time gating versus `when` sample-time evaluation, `--abs` human output, JSON envelope shape, JSONL stream shape, invalid CLI/source combinations, invalid source JSON, duplicate source names, duplicate payload names, fail-fast unresolved payload, rejected wildcard/plain/mixed `on` expressions, `--max 0`, `--max unlimited`, truncation, and empty-result diagnostics. Update `tests/jsonl_cli.rs` and `tests/schema_cli.rs` for the new stream/output/input schema branches, including semantic input-schema validation cases for a valid wrapper, missing or wrong `kind`, empty `sources`, and empty payload. Update `tests/docs_cli.rs` topic lists if a new topic is added. Update `tests/skill_cli.rs` to cover packaged skill routing to `extract generic` and to prevent stale property-plus-value guidance from remaining as the primary transaction extraction path. Add VCD/FST parity coverage for the new command and extend FSDB disabled or feature-gated smoke lists if the repository has comparable waveform command coverage. Add or update tool tests for schema and docs publication helpers.

The final feature milestone is review, fixes, and PR creation. Run the validation commands below, commit coherent slices, run focused implementation review lanes, fix findings, run one fresh control review pass on the final diff, push the branch, and open a draft PR for issue #35.

The current performance follow-up has four milestones and does not push the branch. First, add benchmark coverage for the SCR1 AXI DMEM extraction workload: a one-channel CLI-argument case, a two-channel source-file case matching `tmp/axi_dmem_extract.json`, and a five-channel source-file case matching `tmp/axi_dmem_extract_5ch.json`. These cases belong in the FST release catalog and the generated FSDB catalog; their shared source JSON files belong under `bench/e2e/inputs/`. Validate that the new cases load and show the known poor scaling, then commit this benchmark baseline slice.

Second, implement the grouping fix described in `docs/tracker/wip/proposal_fix_grouping.md`. A grouped execution plan must evaluate each distinct resolved event expression once per relevant timestamp and then run the source-specific `when` predicates and payload sampling in source declaration order. A safe implementation may use a conservative group key; if semantic identity cannot be proven for a source, place it in a singleton group rather than risking an incorrect merge.

Third, validate the implementation through DEBUG diagnostics and focused review. Diagnostics should show the number of event-expression evaluations scaling with the number of event groups rather than with the number of sources for the SCR1 AXI shared-clock manifests. Output rows, JSON/JSONL payloads, diagnostics, and ordering must remain unchanged except for performance/debug counters.

Fourth, run the extract e2e benchmark scenarios as the final performance gate. The expected result is a visible runtime reduction for the two-channel and five-channel source-file cases, with the largest improvement in the five-channel case because all five sources share `posedge clk iff axi_rst_n`. Record the run directories, representative medians, and diagnostic counters in this plan before handoff.

The current FSDB follow-up is investigative. Use the current 2-channel FSDB source-file workload with `target/fsdb/debug/wavepeek` and `DEBUG=1` as the primary microscope because it exposes the expensive path clearly. Start from `src/engine/extract.rs`, `src/waveform/expr_host.rs`, `src/waveform/fsdb_backend.rs`, and `src/waveform/fsdb_native.rs`. If existing counters are too coarse, add DEBUG-only counters that count and time backend sample/event operations by call site. Preserve output semantics while investigating; temporary or permanent diagnostics must be inert when `DEBUG` is not set.

## Concrete Steps

Work from the repository root:

    cd /workspaces/wavepeek/.worktrees/feat-cmd-extract

Before implementation, commit the plan:

    git add docs/tracker/wip/extract-generic-execplan.md
    git commit -m "docs(tracker): plan generic extraction"

After plan review, commit any plan revisions:

    git add docs/tracker/wip/extract-generic-execplan.md
    git commit -m "docs(tracker): refine extraction plan"

During implementation, keep commits focused. Reasonable slice commits are:

    feat(extract): add generic extraction CLI
    feat(extract): execute generic extraction plans
    feat(schema): publish extract input schema
    docs(extract): document generic extraction
    test(extract): cover generic extraction contracts

Use actual commit subjects that match the final diff. Do not commit generated schema snapshots before the Rust contract code that generates them exists.

Run formatting and targeted tests as each slice becomes buildable:

    cargo fmt
    cargo test --test cli_contract
    cargo test --test extract_generic_cli
    cargo test --test schema_cli
    cargo test --test jsonl_cli
    python -m unittest discover -s tools/schema -p "test_*.py"
    python -m unittest discover -s tools/docs -p "test_*.py"

Regenerate and validate schemas after contract changes:

    just update-schema
    just check-schema

Run local handoff gates before review and before PR creation:

    just check

If `just check` fails because the environment is outside the devcontainer, rerun in the repository devcontainer or CI image. Do not bypass hooks unless the user explicitly asks.

Expected success markers include cargo tests reporting `ok`, schema checks reporting no diffs between committed and generated artifacts, and `just check` finishing successfully.

For the performance follow-up, add the benchmark source files and regenerate the FSDB catalog with:

    mkdir -p bench/e2e/inputs
    just update-bench-e2e-fsdb-catalog
    python3 -m unittest bench.e2e.test_perf

Build the release binary and run the new FST extract scenarios with a filter that selects only the new cases:

    cargo build --release
    python3 bench/e2e/perf.py run \
      --tests bench/e2e/tests.json \
      --filter 'extract_scr1_coremark_dmem_axi_(1ch_cli|2ch_source|5ch_source)$' \
      --binary baseline=target/release/wavepeek \
      --run-dir tmp/bench-extract-baseline-fst \
      --wavepeek-timeout-seconds 300 \
      --verbose

If Verdi and FSDB fixtures are available, run the matching FSDB scenarios from the generated catalog:

    python3 bench/e2e/perf.py run \
      --tests bench/e2e/tests_fsdb.json \
      --filter 'extract_scr1_coremark_dmem_axi_(1ch_cli|2ch_source|5ch_source)$' \
      --binary baseline=target/release/wavepeek \
      --run-dir tmp/bench-extract-baseline-fsdb \
      --wavepeek-timeout-seconds 300 \
      --verbose

During optimization, do not modify `bench/e2e/tests.json`, `bench/e2e/tests_fsdb.json`, or `bench/e2e/inputs/`; they are the fixed baseline workload. Use DEBUG runs against the same source files to inspect counters, for example:

    DEBUG=1 target/release/wavepeek extract generic \
      --waves /opt/rtl-artifacts/scr1_max_axi_coremark.fst \
      --scope TOP.scr1_top_tb_axi.i_top \
      --source bench/e2e/inputs/extract_scr1_coremark_dmem_axi_5ch.json \
      --max unlimited \
      --jsonl > tmp/extract-grouping-check.jsonl \
      2> tmp/extract-grouping-check.debug.jsonl

After the grouping fix, run the same e2e filter into a new run directory and compare against the baseline run:

    python3 bench/e2e/perf.py run \
      --tests bench/e2e/tests.json \
      --filter 'extract_scr1_coremark_dmem_axi_(1ch_cli|2ch_source|5ch_source)$' \
      --binary grouped=target/release/wavepeek \
      --run-dir tmp/bench-extract-grouped-fst \
      --compare tmp/bench-extract-baseline-fst/baseline \
      --wavepeek-timeout-seconds 300 \
      --verbose


## Validation and Acceptance

A human can verify the main behavior with a small VCD fixture or integration test. The command:

    wavepeek extract generic \
      --waves tests/fixtures/<small>.vcd \
      --scope top \
      --on "posedge clk iff rst_n" \
      --when "valid && ready" \
      --payload data,last \
      --json

must print a JSON envelope with `$schema` set to `https://kleverhq.github.io/wavepeek/schema-output-v2.1.json`, `command` set to `extract generic`, `diagnostics` present, and `data` containing rows shaped like:

    {
      "time": "25ns",
      "sample_time": "24999ps",
      "source": "transfer",
      "payload": [
        {"path": "top.data", "value": "32'hdeadbeef"},
        {"path": "top.last", "value": "1'h1"}
      ]
    }

A repeated ready/valid transfer with the same payload on consecutive clock edges must produce two rows, not one collapsed row. A multi-source source file must preserve declaration order when two sources match at the same event time. A row whose event `time` equals `--from` must still be emitted when its pre-edge `sample_time` is before `--from` but inside the dump. A wildcard trigger such as `--on '*'`, a plain trigger such as `--on valid`, and a mixed trigger such as `--on 'valid or posedge clk'` must fail before producing rows and explain that `extract generic` requires edge-only pre-edge triggers.

Machine-output acceptance requires all generated schema artifacts to validate. `wavepeek schema --input` must print exactly `schema/input.json`; `wavepeek schema --stream` and default `wavepeek schema` must continue printing the stream and output schemas respectively. The three runtime schema outputs must match the three committed schema snapshots byte-for-byte. JSON output and every JSONL line produced by the new command must validate against the current committed schemas. JSONL must start with a `begin` record, emit item records for rows, emit diagnostic records before the final `end` record, and report final item/diagnostic counts accurately.

Documentation acceptance requires `wavepeek docs topics` to include `commands/extract` and `workflows/extract-handshake`, `wavepeek docs show commands/extract` to explain when to use the command family and current `extract generic` subcommand, and the packaged skill to route event/transaction extraction tasks to `extract` without duplicating full command reference.

Repository acceptance requires `just check` to pass before handoff and `just ci` to pass or be run in CI if local runtime constraints make it impractical. The draft PR must reference issue #35 and state which validation commands passed. For the performance follow-up, acceptance also requires the new extract e2e cases to exist in both `bench/e2e/tests.json` and the generated `bench/e2e/tests_fsdb.json`, a local baseline commit containing only the benchmark additions and plan update, DEBUG evidence that grouped event evaluation reduced duplicate checks for shared `on` expressions, focused review with no unresolved findings, and final benchmark evidence showing improved runtime on the extract scenarios.

## Idempotence and Recovery

The plan and implementation steps are safe to rerun. `cargo fmt` is idempotent. `just update-schema` overwrites generated schema snapshots from Rust contract code; if the output is unexpected, inspect the Rust contract changes and rerun rather than editing `schema/*.json` manually. `just check-schema` writes disposable output under `tmp/schema-check` and removes/recreates only that owned path.

Use repository-root `tmp/` only for disposable scratch files and do not delete arbitrary existing files there. Treat waveform dumps such as `.fst` and `.fsdb` as binary and inspect them through `wavepeek`, fixtures, or purpose-built tests. If a commit slice fails validation, fix forward in a new commit or amend only before a published/pushed review point. Do not reset or delete user work without explicit permission.

If a subagent review is unavailable or fails without usable output, restart that review lane rather than marking review complete. If a generated schema, docs publication helper, or release doc change appears unrelated, keep it if and only if it is required for `just check`, `just ci`, or the new input schema family to work consistently.

## Artifacts and Notes

Important source material:

    tmp/extract-generic-proposal-v3.ru.md
    https://github.com/kleverhq/wavepeek/issues/35

Representative source JSON:

    {
      "$schema": "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json",
      "kind": "extract.generic.sources",
      "sources": [
        {
          "name": "fifo_write",
          "on": "posedge wclk iff wrst_n",
          "when": "wvalid && wready",
          "payload": ["wdata", "be"]
        },
        {
          "name": "fifo_read",
          "on": "posedge rclk iff rrst_n",
          "when": "rvalid && rready",
          "payload": ["rdata"]
        }
      ]
    }

Representative JSONL item record:

    {"type":"item","seq":1,"command":"extract generic","item":{"time":"25ns","sample_time":"24999ps","source":"rx.beat","payload":[{"path":"top.dut.data","value":"32'hdeadbeef"}]}}

## Interfaces and Dependencies

At the end of implementation, `src/cli/extract.rs` must define a nested command interface equivalent to:

    #[derive(Debug, Clone, clap::Subcommand)]
    pub enum ExtractCommand {
        Generic(GenericArgs),
    }

    #[derive(Debug, Clone, clap::Args)]
    pub struct GenericArgs {
        pub waves: PathBuf,
        pub from: Option<String>,
        pub to: Option<String>,
        pub scope: Option<String>,
        pub name: Option<String>,
        pub on: Option<String>,
        pub when: Option<String>,
        pub payload: Option<String>,
        pub source: Option<PathBuf>,
        pub max: MaxCount,
        pub abs: bool,
        pub json: bool,
        pub jsonl: bool,
    }

The concrete field types may differ to match existing CLI conventions, but the external flags and validation behavior must match this interface.

At the end of implementation, `src/engine/extract.rs` must expose engine-owned types equivalent to:

    pub struct ExtractPlan {
        pub sources: Vec<ExtractSource>,
        pub display_absolute: bool,
        pub max: MaxCount,
    }

    pub struct ExtractSource {
        pub declaration_index: usize,
        pub name: String,
        pub on: String,
        pub when: String,
        pub payload: Vec<String>,
    }

    pub struct ExtractRow {
        pub time: String,
        pub sample_time: String,
        pub source: String,
        pub payload: Vec<ExtractPayloadValue>,
    }

    pub struct ExtractPayloadValue {
        pub path: String,
        pub display_path: String,
        pub value: String,
    }

The contract-facing row type may omit `display_path` because JSON and JSONL always use canonical paths. Human output may use `display_path` or equivalent rendering metadata to support `--abs`.

`src/engine/mod.rs` must route the command through:

    Command::ExtractGeneric(extract::GenericArgs)
    CommandName::ExtractGeneric
    CommandData::ExtractGeneric(Vec<contract::output::ExtractGenericRow>)

The exact enum payload type can differ, but the command name string must be `extract generic`.

`src/contract/schema.rs` must define an input schema family with constants equivalent to:

    pub const INPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-input-v2.1.json";
    pub const OUTPUT_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-output-v2.1.json";
    pub const STREAM_SCHEMA_URL: &str = "https://kleverhq.github.io/wavepeek/schema-stream-v2.1.json";
    pub const INPUT_SCHEMA_FAMILY_ID: &str = "wavepeek.input";
    pub fn input_schema_json() -> String;

`wavepeek schema --input` must print `input_schema_json()`. `tools/schema-gen` must write `schema/input.json` from the same function.

## Revision Notes

- 2026-06-28: Initial ExecPlan created from the local v3 proposal, issue context, and repository inspection so implementation can proceed through reviewed milestones.
- 2026-06-28: Updated after focused plan review to bump changed schema families to v2.1, correct pre-edge range-bound semantics, require pre-output expression support validation, enforce scoped payload naming, and add missing schema/docs/tests/tooling collateral.
- 2026-06-28: Updated after implementation commit `8c2a039` to record completed feature surfaces, generated schemas, targeted validation, and passing commit-hook evidence.
- 2026-06-28: Updated after implementation review and fix commit `ec1892e` to record reviewer findings, runtime preflight fixes, tooling/docs corrections, and targeted validation.
- 2026-06-28: Updated after follow-up review and `just check` to record clean follow-up lanes, local gate success, initially unavailable broad control-pass output, and successful narrow control review lanes.
- 2026-06-28: Updated after opening draft PR #46.
- 2026-06-29: Updated after maintainer PR review comments to record extract-family docs renaming, README/help/skill wording fixes, targeted validation, focused review, follow-up review, and final `just check` success.
- 2026-06-30: Updated after fixing `wavepeek extract` no-subcommand help behavior.
- 2026-06-30: Updated after profiling the SCR1 AXI DMEM extraction run and adding DEBUG-only extract timing diagnostics.
- 2026-07-01: Added the performance follow-up plan for SCR1 AXI extract e2e benchmarks, baseline validation, event-expression grouping, review, and final performance gate.
- 2026-07-01: Updated after adding the extract e2e benchmark scenarios and collecting FST/FSDB baseline timings for the current implementation.
- 2026-07-01: Updated after implementing event-expression grouping and collecting DEBUG evidence on the SCR1 AXI 2-channel and 5-channel source-file runs.
- 2026-07-01: Updated after focused review, performance/correctness follow-up fixes, and reviewer rechecks with no substantive findings.
- 2026-07-01: Updated after the clean control review and final FST/FSDB extract e2e benchmark gate.
- 2026-07-01: Updated after direct current-vs-baseline output comparison and apples-to-apples `target/debug` JSONL timing validation against the original 23s/57s baseline.
- 2026-07-01: Updated after direct baseline/current FSDB diagnostic comparisons for release and debug builds on the 2-channel and 5-channel source-file workloads.
- 2026-07-01: Added the FSDB critical-path investigation milestone for current debug 2-channel source-file extraction.
- 2026-07-01: Updated after adding FSDB backend DEBUG counters and identifying native per-value sampling as the current critical path.
