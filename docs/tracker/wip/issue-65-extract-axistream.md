# Add AXI4-Stream and AXI5-Stream transfer extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. Implementation follows test-driven development: user-visible behavior is expressed in focused tests before or alongside the smallest implementation, then the complete repository gates and independent review validate the consolidated result.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract axistream` on VCD, FST, or supported FSDB waveforms and receive one deterministic row for every completed AXI4-Stream or AXI5-Stream transfer. The adapter supplies protocol names, transfer predicates, source files, typed human/JSON/JSONL output, and exact schemas while reusing the existing protocol-neutral `extract generic` runtime.

The behavior is observable with `wavepeek extract axistream --profile axi4-stream --waves <dump> ...`; the output reports Issue B, the effective `tready_mode`, resolved mappings, event and pre-edge sample timestamps, and mapped payload values. Source-backed fixtures prove VCD/FST parity and both explicit and include-based mapping.

## Non-Goals

This work does not reconstruct packets or interleaved streams, interpret `tkeep` or `tstrb`, decode application payloads, validate signal widths or interface properties, check protocol timing and stability, or extract multiple stream interfaces in one invocation. It does not expose AXI5-Stream wake-up activity or parity/check validation. It does not synthesize omitted payload defaults. It does not change `extract axi` behavior.

## Progress

- [x] (2026-07-20 08:30Z) Read issue #65, repository guidance, current generic and AXI extraction paths, schema ownership, fixture policy, public contracts, prior feature plans, and Arm IHI 0051B source clauses.
- [x] (2026-07-20 08:30Z) Confirm the clean branch `feat/extract-axistream` is at `caea6c3`, identical to `main` and `origin/main`.
- [x] (2026-07-20 09:10Z) Milestone 1: added a failing CLI test, dedicated AXI-Stream runtime adapter, CLI, shared standard-name matching helper, output DTOs, human/JSONL rendering, and source-backed VCD/FST fixture.
- [x] (2026-07-20 09:24Z) Milestone 2: added exact input/output/stream contracts, regenerated schemas, and updated schema and deployed-endpoint checks.
- [x] (2026-07-20 09:38Z) Milestone 3: updated help, public docs, packaged skill, architecture, changelog, and executable documentation contracts.
- [ ] Milestone 4 (completed: implementation and all review fixes committed; final `just check` and `just ci` passed; four focused lanes and two fresh control passes completed, with the second control pass clean; remaining: remove this WIP plan, push, and open the issue-closing PR).

## Surprises & Discoveries

- Observation: `src/engine/extract.rs` already exposes the complete runtime needed by a protocol adapter, including `ExtractPlan`, `ExtractSource`, `ExtractRunArgs`, `ExtractRowSink`, and `run_plan_with_waveform_sink`.
  Evidence: `src/engine/axi.rs` constructs generic sources and converts rows without owning waveform backends or event evaluation.

- Observation: AXI-Stream has one ready/valid channel per selected interface, unlike `extract axi`, so its rows need no synthetic channel field.
  Evidence: Issue #65 requires one invocation to map one interface and defines output rows with only timestamps, profile, and payload.

- Observation: both requested profiles intentionally share the same functional transfer signals.
  Evidence: Arm IHI 0051B Table B-1 lists common functional signals; AXI5-only `TWAKEUP` and Table B-2 check signals do not define transfer completion and are excluded.

- Observation: the current schema family preserves extension fields on transfer objects, so the exact AXI-Stream contract can close mapping and payload key sets but cannot reject an arbitrary extension field named `channel`.
  Evidence: output and stream schema tests accept outer transfer extensions while runtime DTOs never emit `channel`; this preserves the documented v2 additive extension contract.

- Observation: deployed schema checks previously verified endpoint availability but not feature content.
  Evidence: `tools/docs/check_deploy.py` discarded fetched schema bodies. The v2.2 check now parses all three schema families and requires the AXI-Stream command and source kind.

- Observation: generated JSONL begin records originally allowed an absent, null, or cross-command protocol context because the derived `command` and optional `context` properties were independent.
  Evidence: the contract review reproduced invalid `extract axistream` begins accepted by `schema/stream.json`; command-aware begin branches now require the matching AXI or AXI-Stream context and reject context on other commands.

- Observation: generic source binding validated a scope by enumerating and sorting every direct signal even when the protocol adapter had already enumerated include candidates.
  Evidence: the performance review identified the duplicate scan in `bind_extract_sources`; a lightweight backend `validate_scope` path now preserves the same missing-scope error without allocating signal entries.

- Observation: exact AXI-Stream output mapping schemas listed allowed names but initially did not require the resolved handshake keys that runtime always emits.
  Evidence: the fresh control review showed impossible mapped contexts without `aclk`, `tvalid`, or `tready` validating successfully; mode-specific mapping schemas now require `aclk`/`tvalid` and additionally require `tready` in mapped mode.

## Decision Log

- Decision: Implement AXI-Stream in new `src/engine/axistream.rs` and `src/contract/axistream_schema.rs` modules rather than extending the large AXI modules.
  Rationale: Issue #65 explicitly requires a dedicated adapter; the command has a different one-channel model, TREADY mode, source kind, and output shape.
  Date/Author: 2026-07-20 / coding agent

- Decision: Keep auto-mapping behavior identical to AXI by extracting only the smallest protocol-neutral matching helpers if direct reuse requires it.
  Rationale: The new command must preserve deterministic token matching and existing diagnostics without changing AXI behavior.
  Date/Author: 2026-07-20 / coding agent

- Decision: Keep schema family version 2.2.
  Rationale: Issue #65 is in the v2.2 milestone and extends the current unreleased exact schemas rather than replacing a released historical contract.
  Date/Author: 2026-07-20 / coding agent

- Decision: Treat missing `tready` as HIGH only when `tready_mode` is explicitly `implicit-high`.
  Rationale: Arm IHI 0051B defines HIGH as the default for a physically omitted TREADY, while an absent waveform mapping alone cannot prove physical omission.
  Date/Author: 2026-07-20 / coding agent

- Decision: Make JSONL begin context constraints command-aware for all stream commands.
  Rationale: exact protocol branch isolation requires AXI and AXI-Stream begin records to carry their own matching context; non-protocol runtime begins carry no context and should not validate with one.
  Date/Author: 2026-07-20 / coding agent after contract review

- Decision: Add `Waveform::validate_scope` rather than retaining signal enumeration as an existence check.
  Rationale: both backends already index scopes directly, so validation can preserve diagnostics while avoiding a second hierarchy scan and allocation in protocol extraction.
  Date/Author: 2026-07-20 / coding agent after performance review

- Decision: Require runtime-mandatory handshake mappings in resolved output and JSONL context schemas, but not in source-input `maps`.
  Rationale: output contexts are post-resolution and always contain the required keys, while source documents can supply them through include-based auto-mapping rather than explicit `maps`.
  Date/Author: 2026-07-20 / coding agent after control review

## Outcomes & Retrospective

The command, runtime adapter, exact contracts, fixtures, schemas, checks, help, docs, skill, architecture, and changelog are complete. Runtime/protocol and docs/architecture reviews found no issues. Contract and performance reviews found two medium issues; command-aware JSONL begin schemas and lightweight scope validation resolved them and both reviewers rechecked clean. The first fresh control pass found one medium exact-schema issue; requiring resolved handshake mappings fixed it. The second fresh control pass found no substantive issues.

Final `just check` and `just ci` both pass after every review fix, including FSDB gates and source coverage of 94.18% regions, 93.53% functions, and 94.65% lines. All local acceptance criteria are met. Only mechanical delivery remains: remove this WIP file, push the branch, and create the PR with `Closes #65`.

## Context and Orientation

`wavepeek` is a Rust command-line waveform inspector. All work runs from `/workspaces/wavepeek/.worktrees/feat-extract-axistream` in the project devcontainer. Root `justfile` recipes own formatting, lint, fixtures, schemas, tests, and handoff gates. `just check` is the local static handoff gate; `just ci` includes tests and coverage.

The CLI hierarchy is defined in `src/cli/mod.rs` and `src/cli/extract.rs`. `src/engine/mod.rs` dispatches commands, renders human output, and writes JSON or JSONL. `src/engine/extract.rs` owns protocol-neutral execution: an `ExtractPlan` contains one or more `ExtractSource` values, each source selects an edge event, evaluates a Boolean predicate at the pre-edge sample point, and captures ordered payload signals. `src/engine/axi.rs` is the existing protocol-adapter pattern: it resolves explicit mappings and include candidates, validates a profile, builds generic sources, invokes `run_plan_with_waveform_sink`, and converts generic rows into protocol-specific transfer rows.

Machine DTOs and JSON Schema generation live under `src/contract/`. `src/contract/output.rs` defines JSON envelope data and transfer types; `src/contract/stream.rs` defines JSONL records; `src/contract/input.rs` defines source documents; and `src/contract/axi_schema.rs` replaces broad generated definitions with exact profile-aware AXI branches. A new AXI-Stream schema module must do the analogous replacement without coupling AXI and AXI-Stream profile branches. `tools/schema-gen` writes canonical `schema/output.json`, `schema/stream.json`, `schema/input.json`, and `schema/catalog.json`; generated snapshots must be updated with `just update-schema`, never hand-edited.

Integration tests live under `tests/`. `tests/extract_axi_cli.rs` demonstrates protocol-adapter human, JSON, JSONL, source, mapping, ambiguity, and schema validation patterns. The new command belongs in a focused `tests/extract_axistream_cli.rs`. Verilog fixture sources belong in `tests/fixtures/source/`; generated VCD/FST outputs are ignored under `tests/fixtures/generated/`; every output must be registered in `tests/fixtures/waveform_policy.json`.

The protocol authority is Arm IHI 0051B Issue B. Table 2-1 in §2.1 defines `aclk`, `aresetn`, `tvalid`, optional `tready`, and optional transfer payloads `tdata`, `tstrb`, `tkeep`, `tlast`, `tid`, `tdest`, and `tuser`. Section 2.2 says a transfer occurs when both `tvalid` and `tready` are asserted and signal information remains stable until handshake. Section 2.8 defines rising-edge sampling and active-low reset. Table B-1 defines omitted TREADY's default as HIGH. AXI5-only `twakeup` in §2.3 and check signals in Table B-2 are outside transfer extraction. Both `axi4-stream` and `axi5-stream` use Issue B and the same functional mapping set.

`mapped` TREADY mode requires a mapped `tready` and builds `aresetn && tvalid && tready` when reset is mapped, otherwise `tvalid && tready`. `implicit-high` forbids a mapped `tready` and builds `aresetn && tvalid` or `tvalid`. In every case, the event is `posedge aclk`, event bounds apply to edge time, and predicate/payload values use the runtime's existing pre-edge sample point.

## Open Questions

There are no blocking product or protocol questions. The issue fixes command spelling, profiles, modes, mappings, source and output shapes, schema strictness, docs, tests, validation, and delivery acceptance.

## Plan of Work

### Milestone 1: Runtime adapter and observable extraction

Add `AxiStream` to the `ExtractCommand` enum in `src/cli/extract.rs` with explicit clap name `axistream`. Define clap enums for canonical profiles and TREADY modes, aliases, defaults, source conflicts, shared waveform/time/scope/mapping/output options, and standalone long help. Wire dispatch in `src/cli/mod.rs` and exports in `src/engine/mod.rs`.

Create `src/engine/axistream.rs`. Define profile metadata, canonical/alias parsing, source loading, map/include resolution, deterministic auto-mapping, validation, generic-plan construction, sink conversion, and unit tests. Reuse existing generic execution types. If AXI auto-mapping internals cannot be called, move only tokenization and candidate resolution into a private sibling module used by both adapters, preserving exact existing diagnostics and tests. The final mapping order is `aclk`, `aresetn`, `tvalid`, `tready`, then `tdata`, `tstrb`, `tkeep`, `tlast`, `tid`, `tdest`, `tuser`; omitted entries are absent.

Add AXI-Stream output DTOs in `src/contract/output.rs` and JSONL context in `src/contract/stream.rs`. Add human rendering in `src/output.rs` and mode dispatch in `src/engine/mod.rs`. Human rows contain no channel marker; handshake-only rows contain timestamps and no fabricated payload.

Create `tests/fixtures/source/extract_axistream.v`, register VCD and FST outputs in `tests/fixtures/waveform_policy.json`, and add `tests/extract_axistream_cli.rs`. Cover explicit and automatic mapping, compact and split signal forms, explicit precedence, ambiguity, check/wakeup decoys, both profiles, both TREADY modes, reset gating, pre-edge sampling, handshake-only and repeated identical transfers, human/JSON/JSONL output, source parsing, conflicts, invalid names, and VCD/FST parity.

### Milestone 2: Exact machine contracts

Add `ExtractAxiStreamSourceInput` in `src/contract/input.rs` with exact current `$schema`, kind `extract.axistream.source`, canonical profile/mode enums, optional fields with runtime defaults, and null rejection. Add dedicated profile-driven exact schema replacement in `src/contract/axistream_schema.rs`; integrate it with output, stream, and input schema generation. Keep AXI and AXI-Stream branches isolated and mapping/payload objects closed to unknown standard names while outer current-v2 extension behavior remains unchanged.

Update `tests/schema_cli.rs`, `tests/cli_contract.rs`, `tools/schema/check_schema_contract.py`, `tools/schema/test_check_schema_contract.py`, `tools/docs/check_deploy.py`, and `tools/docs/test_check_deploy.py` for the new command and source kind. Regenerate all schema artifacts with `just update-schema`, inspect changes, and verify `schema/catalog.json` remains semantically unchanged except regeneration determinism.

### Milestone 3: User and maintainer guidance

Update `src/cli/extract.rs` long help, `docs/public/commands/extract.md`, `docs/public/commands/overview.md`, `docs/public/reference/machine-output.md`, `docs/public/workflows/extract-handshake.md`, `docs/skills/wavepeek.md`, `docs/dev/architecture.md`, and `CHANGELOG.md`. Describe both Issue B profiles, explicit TREADY modes, one-interface scope, mappings, transfer-only behavior, exclusions, output forms, and source mode without duplicating generated flag tables. Update `tests/docs_cli.rs` and `tests/skill_cli.rs` so embedded docs and packaged skill claims remain executable contracts.

Commit runtime, contracts, fixtures, generated schemas, tests, checkers, help, docs, skill, architecture, changelog, and the updated plan as coherent reviewed slices using normal hooks. Never bypass hooks.

### Milestone 4: Validation, review, and delivery

Run focused tests throughout development, then `just check` and `just ci`, saving long logs under repository-root `tmp/` without deleting unrelated files. Run independent read-only review lanes for runtime/protocol correctness, contracts/schema/tests, and docs/help/architecture. Reproduce substantive findings with tests where practical, fix in the main session, and rerun relevant tests and full gates. Run one fresh independent control review over the consolidated `main...HEAD` diff. If it finds a substantive issue, fix it, rerun both full gates, and perform one final control pass.

When all acceptance criteria and review findings are closed, complete this plan's retrospective, remove this WIP file while preserving `docs/tracker/wip/AGENTS.md`, and commit cleanup. Push `feat/extract-axistream` to `origin` without force. Open a GitHub pull request against `main` whose body includes `Closes #65`, then verify its URL, base/head refs, and open state.

### Concrete Steps

All commands run from `/workspaces/wavepeek/.worktrees/feat-extract-axistream`.

Focused development commands are:

    just prepare-waveform-fixtures
    cargo test --lib engine::axistream::tests
    cargo test --test extract_axistream_cli
    cargo test --test cli_contract
    cargo test --test schema_cli
    cargo test --test docs_cli
    cargo test --test skill_cli
    python3 -B -m unittest tools.schema.test_check_schema_contract
    python3 -B -m unittest tools.docs.test_check_deploy

Generate and validate exact schemas with:

    just update-schema
    just check-schema

Run full validation with:

    just check 2>&1 | tee tmp/issue-65-just-check.log
    just ci 2>&1 | tee tmp/issue-65-just-ci.log

Before each commit inspect and stage only owned paths:

    git status --short
    git diff --check
    git add <reviewed paths>
    git diff --cached --check
    git commit -m '<conventional subject>'

Delivery uses:

    git push -u origin feat/extract-axistream
    gh pr create -R kleverhq/wavepeek --base main --head feat/extract-axistream \
      --title 'feat(extract): add AXI-Stream transfer extraction' \
      --body-file tmp/issue-65-pr.md

The PR body must summarize runtime, contracts, docs, and validation, and end with `Closes #65`.

### Validation and Acceptance

Acceptance requires `wavepeek extract axistream --help` to expose exactly `axi4-stream` and `axi5-stream`, default to `axi4-stream`, and expose `mapped` and `implicit-high` modes. Runtime parsing accepts case-insensitive underscore aliases, but generated schemas accept only canonical hyphenated profile and mode values.

Both profiles must report Issue B and map only `aclk`, `aresetn`, `tvalid`, `tready`, `tdata`, `tstrb`, `tkeep`, `tlast`, `tid`, `tdest`, and `tuser`. `aclk` and `tvalid` are required. `mapped` requires `tready`; `implicit-high` forbids it. Missing reset and payload mappings remain valid. `twakeup` and every `*chk` standard name are rejected or ignored as unmatched include candidates.

Human, JSON, and JSONL output must include canonical name/profile/issue/mode context, resolved mappings, event and sample times, and mapped payloads without a channel field. Handshake-only and repeated-identical transfers must be preserved. JSON uses `command: "extract axistream"`; JSONL begins with matching context, emits independently typed transfer items, and ends with an accurate summary.

Source documents with exact current `$schema` and kind `extract.axistream.source` must load; defaults are `axi4-stream`, `mapped`, and `axistream`; wrong schema/kind and explicit null profile/mode/name must fail. Exact input, output, and stream schemas accept all valid forms, reject aliases and unknown mapping/payload keys, and remain isolated from `extract axi` branches.

The source-backed fixture must produce equivalent AXI-Stream data and diagnostics from generated VCD and FST. Existing AXI and generic extraction tests must remain green. Final `just check` and `just ci` must both exit zero after review fixes.

### Idempotence and Recovery

Fixture and schema generation are deterministic and safe to rerun. Generated VCD/FST files remain ignored; commit only source fixtures and policy metadata. Regenerate schema JSON from Rust code rather than editing snapshots. Keep disposable logs and PR bodies under `tmp/` and never delete unrelated existing files there.

Use milestone commits as recovery points. Do not bypass hooks or force-push. If a normal push is rejected, fetch and inspect `origin/feat-extract-axistream...HEAD`; only retry when the remote tip is an ancestor, otherwise reconcile unexpected commits. If PR creation reports an existing PR, locate and verify it instead of creating a duplicate.

### Artifacts and Notes

Initial state:

    branch: feat/extract-axistream
    base: caea6c3
    main/origin-main relation: identical
    tracked working tree: clean

Protocol evidence checked directly in Arm IHI 0051B includes §2.1/Table 2-1 for signals, §2.2 for handshake, §2.3 for AXI5 wake-up, §2.8 for clock/reset, and Tables B-1 through B-3 for defaults, profile applicability, checks, and properties.

### Interfaces and Dependencies

No new dependency is required. Continue using clap for CLI parsing, serde for runtime input/output, schemars for base schema derivation, serde_json for exact schema replacement, regex for includes, and the existing waveform backends through generic extraction.

At completion, `src/engine/axistream.rs` must expose the dispatch types/functions needed by `src/engine/mod.rs`, construct one generic `ExtractSource`, and return AXI-Stream-specific context and rows. `src/contract/output.rs` must define AXI-Stream context and transfer DTOs. `src/contract/stream.rs` must define AXI-Stream begin context. `src/contract/input.rs` must define the singular source DTO. `src/contract/axistream_schema.rs` must derive exact branches from one runtime profile source of truth rather than duplicate profile signal inventories across several modules.

Plan revision note (2026-07-20): Created the initial self-contained plan after repository and protocol investigation. It records the one-channel extraction model, explicit TREADY omission contract, module boundaries, exact schema scope, fixture strategy, review requirements, and PR delivery linked to issue #65.

Plan revision note (2026-07-20 09:40Z): Marked implementation, contracts, fixtures, generated schemas, checks, and documentation complete after focused tests and `just check`. Recorded the extension-field and deployed-schema-check discoveries; delivery work remains in Milestone 4.

Plan revision note (2026-07-20 10:02Z): Recorded four independent review lanes, the two medium findings and fixes, clean focused rechecks, and passing post-review `just check`/`just ci`. Only the control pass and delivery cleanup remain.

Plan revision note (2026-07-20 10:12Z): Recorded the first control pass's required-mapping schema finding and its mode-specific fix. Focused output/stream schema tests and `just check-schema` pass; full gates and the second control pass remain.

Plan revision note (2026-07-20 10:22Z): Recorded passing final full gates and a clean second control review. The implementation is complete and locally audited; this plan is ready for required WIP cleanup before branch delivery.
