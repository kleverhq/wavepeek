# Add end-to-end AMBA ATB extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. Work from repository root `/workspaces/wavepeek/.worktrees/feat-extract-atb` on branch `feat/extract-atb`. The product requirement is GitHub issue `kleverhq/wavepeek#68`.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract atb` against VCD, FST, or optional FSDB waveforms and receive deterministic AMBA ATB interface events. The command reports accepted trace transfers, completed flush handshakes, and synchronization-request pulses without reconstructing packets or maintaining cross-cycle protocol state. Users can configure an interface with command-line mappings, automatic include-based mapping, or a JSON source file, and consume human, JSON, or JSONL output governed by generated schemas.

A working result is visible by running the command against the new source-backed fixture. With an ATB-C mapping, the output must identify profile `atb-c`, issue `C`, list canonical signal mappings, and emit events in deterministic source order: `transfer`, `flush`, then `sync-request` when events share a timestamp.

## Non-Goals

This work does not reconstruct CoreSight trace packets, infer trigger encodings, enforce ATB protocol legality, pair multi-cycle episodes, count synchronization requests, decode payload bytes, or implement wake-up events. It does not add reducer state to the generic extraction runtime. It does not refactor AXI extraction unless a helper can be reused without changing existing AXI behavior or public contracts. It does not keep this branch-local plan in the final pull-request diff.

## Progress

- [x] (2026-07-20 08:30Z) Read issue #68, repository guidance, current AXI and generic extraction paths, CLI dispatch, output contracts, schema generation, docs, fixtures, tests, and quality gates.
- [x] (2026-07-20 08:30Z) Verify the protocol facts used by the issue against Arm IHI 0032C Issue C, especially sections 3.2, 4.3, 5.1, 6.2, and Appendix A Table A-1.
- [x] (2026-07-20 08:30Z) Add the branch-local ExecPlan; commit is the next command.
- [x] (2026-07-20 08:58Z) Implement the ATB CLI, dedicated engine planning/runtime adapter, output models, and human/JSON/JSONL dispatch with focused unit tests; commit is the next milestone boundary.
- [x] (2026-07-20 08:58Z) Implement exact output, stream, input, and command-enum schema branches; regenerate artifacts; add focused schema validators; update the schema contract checker.
- [x] (2026-07-20 08:58Z) Add one source-backed ATB fixture with VCD/FST outputs and end-to-end tests for profiles, aliases, mapping, output modes, source files, ordering, reset, windows/limits, parity, independent channels, optional payload, and negative cases.
- [x] (2026-07-20 08:58Z) Update embedded public docs, packaged skill routing, architecture module map, and `CHANGELOG.md`; documentation commit remains.
- [ ] Run focused tests, `just test`, `just check`, and `just ci`; repair all failures without weakening gates or coverage.
- [ ] Run multi-lane read-only review and an independent control review; fix all substantive findings and rerun affected gates.
- [ ] Update this plan with evidence and retrospective, then remove it in a cleanup commit before handoff.
- [ ] Push `feat/extract-atb` to `origin`, open a pull request against `main` whose body closes issue #68, and verify the remote PR metadata.

## Surprises & Discoveries

- Observation: The repository already centralizes synchronous stateless event evaluation in `src/engine/extract.rs`, and AXI adapts its rows through a protocol-specific sink in `src/engine/axi.rs`.
  Evidence: `axi::run_with_sink` builds `ExtractSource` entries and delegates to `extract::run_plan_with_waveform_sink`; `GenericToAxiSink` converts generic payload rows to AXI transfers.

- Observation: AXI JSONL emits a context-bearing begin record, unlike ordinary commands, and therefore ATB needs the same special dispatch treatment.
  Evidence: `src/output.rs::write_jsonl_result` skips the ordinary begin record for `CommandData::ExtractAxi`, then calls `begin_context`.

- Observation: Arm IHI 0032C defines `ATWAKEUP` only for ATB-C when the `Wakeup_Signal` property applies, but issue #68 deliberately excludes wake-up extraction and mappings from every initial profile. `SYNCREQ` is optional for ATB-B and ATB-C and absent from ATB-A.
  Evidence: Appendix A Table A-1 marks `ATWAKEUP` as conditional for C only and `SYNCREQ` as optional for B/C and not present for A; issue #68 lines 109-123 explicitly keeps the ATB-B and ATB-C extraction-name sets identical and rejects `atwakeup`.

- Observation: The full issue contract permits transfer extraction without `ATBYTES`, `ATDATA`, or `ATID`, and requires a complete transfer or flush pair even when `SYNCREQ` is mapped.
  Evidence: The source-backed fixture passes handshake-only and 8-bit-`ATDATA`-without-`ATBYTES` integration tests; a synchronization-only configuration fails with the required handshake-channel diagnostic.

- Observation: Include-selected malformed and excluded names can be retained in one fixture without affecting mapping because normalized matching requires an exact standard suffix.
  Evidence: `trace_at_valid_chk_o`, `trace_at_ready_check_o`, `trace_at_clken_o`, and `trace_at_wakeup_o` all produce deterministic `WPK-W0004` diagnostics while canonical signals map successfully.

## Decision Log

- Decision: Implement a dedicated `src/engine/atb.rs` module and reuse only the generic extraction runtime, mirroring AXI's proven plan-and-sink pattern.
  Rationale: Issue #68 requires an ATB-specific module and stateless predicates. Keeping protocol tables and validation together avoids changing the existing AXI path and meets KISS/YAGNI guidance.
  Date/Author: 2026-07-20 / pi coding agent

- Decision: Treat all mapped values as formatted waveform values and leave payload interpretation to consumers.
  Rationale: The requested behavior is event extraction, not protocol decoding. This also preserves deterministic generic runtime behavior across waveform formats.
  Date/Author: 2026-07-20 / pi coding agent

- Decision: Use Arm IHI 0032C Issue C as the normative source for all three interface profiles.
  Rationale: Issue C defines current ATB-A, ATB-B, and ATB-C compatibility and supplies the current signal matrix. The required metadata explicitly says `issue: C`.
  Date/Author: 2026-07-20 / pi coding agent

- Decision: Preserve profile order and event source order as explicit static data rather than sorting user-visible fields alphabetically.
  Rationale: Issue #68 defines canonical mapping order and same-timestamp event order as public deterministic contracts.
  Date/Author: 2026-07-20 / pi coding agent

## Outcomes & Retrospective

The command, runtime adapter, exact schemas, fixture, integration tests, and documentation are implemented without a new dependency or generic-runtime change. Focused ATB, CLI-help, docs, schema, clippy, and schema-contract checks pass. Full repository gates and external review remain.

## Context and Orientation

`wavepeek` is a Rust CLI. `src/cli/extract.rs` defines `extract generic` and `extract axi` argument models. `src/cli/mod.rs` converts parsed CLI variants into engine commands. `src/engine/mod.rs` owns command names, dispatch, and the `CommandData` enum used by renderers. `src/engine/extract.rs` evaluates one or more synchronous sources. Each source has an `on` edge expression, a Boolean `when` predicate sampled immediately before that edge, and a list of payload waveforms. The runtime emits rows through a sink and already owns time windows, limits, reset-independent expression evaluation, diagnostics, and deterministic source ordering.

`src/engine/axi.rs` is the nearest implementation model. It defines profile signal lists, parses mappings, finds include candidates, validates complete ready/valid channels, turns each channel into a generic extraction source, and converts rows into AXI-specific DTOs. ATB must follow this shape but remain a separate module because its profiles and event predicates differ.

An ATB profile is the set of canonical standard signal names allowed for an interface class. All profiles use `atclk`, `atresetn`, `atvalid`, `atready`, `atbytes`, `atdata`, `atid`, `afvalid`, and `afready`. `atb-a` has only those signals. `atb-b` and `atb-c` add optional `syncreq`; neither initial extraction profile accepts `atwakeup`. CLI profile parsing is case-insensitive and accepts underscore aliases `atb_a`, `atb_b`, and `atb_c`, plus legacy `atbv1.0` and `atbv1.1` for ATB-A and ATB-B. Output and source schema enums accept canonical hyphenated names only.

The command accepts common waveform flags `--waves`, `--from`, `--to`, `--scope`, `--max`, `--abs`, `--json`, and `--jsonl`. Its protocol options are `--profile` with default `atb-c`, optional `--name` with default `atb`, repeatable `--map STD=WAVES`, repeatable `--include REGEX`, and optional `--source PATH`. Source-file mode excludes `--profile`, `--name`, `--map`, and `--include`. A source file has exact `$schema` equal to the current input schema URL, exact kind `extract.atb.source`, optional canonical profile, optional name, and optional `includes` and `maps` collections.

Mapping behavior follows AXI: canonical names are lowercase; explicit mappings override automatic candidates; includes select candidates, and matching tolerates case, separators, a leading interface prefix, and common direction suffixes. `--scope` makes mappings and output display names scope-relative and forbids dotted explicit waveform names. Without `--scope`, mappings resolve as absolute paths. Duplicate standard mappings, invalid regexes, unsupported profile signals, ambiguous candidates, and missing required mappings are argument errors. Unmatched include candidates produce stable warning `WPK-W0004`.

At least one complete handshake channel must be enabled. A transfer source requires the complete `atvalid`/`atready` pair; `atbytes`, `atdata`, and `atid` are optional raw payload mappings, which permits handshake-only extraction and an 8-bit `ATDATA` interface without `ATBYTES`. Its predicate is `atresetn && atvalid && atready` when reset is mapped, otherwise `atvalid && atready`, and its mapped payload order is `atbytes`, `atdata`, `atid`. A flush source requires the complete `afvalid`/`afready` pair; its predicate is reset gating plus both handshake signals, and it has an empty payload. A synchronization-request source exists whenever `syncreq` is mapped on ATB-B or ATB-C; its predicate is reset gating plus `syncreq`, and it has an empty payload, but it does not satisfy the requirement for a transfer or flush handshake channel. `atclk` is always required. Partial handshake pairs are errors. Transfer payload fields without the complete transfer handshake are errors. Sources are constructed in fixed order `transfer`, `flush`, `sync-request`, which determines same-timestamp output order.

`src/contract/output.rs` defines JSON DTOs; `src/contract/stream.rs` defines JSONL begin context and item DTOs; `src/contract/input.rs` defines source-file input schema models; `src/contract/schema.rs` combines branches and command enums; `src/contract/axi_schema.rs` manually creates AXI profile-exact schema branches. ATB needs exact profile-specific branches of the same kind. ATB-A schema mappings allow only the base signals and exclude `syncreq`, `atclken`, and `atwakeup`; ATB-B and ATB-C add `syncreq` but still exclude `atclken` and `atwakeup`. Transfer event payload schemas allow only optional `atbytes`, `atdata`, and `atid`; flush and sync-request events require empty payload objects. Output/context branches enforce a complete transfer or flush pair and mapping-backed events, while stream items permit each profile-legal event independently. Generated snapshots under `schema/` must only be updated through `just update-schema`.

`src/output.rs` renders human text and writes JSONL. Human ATB output has header lines `name`, `profile`, `issue`, then ordered `mappings`, then `events`. Each event is one line with `@time sample@sample_time [kind]` and canonical payload names unless `--abs` selects absolute waveform paths. JSON uses command `extract atb` and data object `{name, profile, issue, mappings, events}`. JSONL starts with a context-bearing begin record, streams one event per item, then diagnostics and an end summary.

Source-backed waveform fixtures are HDL files under `tests/fixtures/source/`, generated VCD/FST files are ignored under `tests/fixtures/generated/`, and `tests/fixtures/waveform_policy.json` declares generated outputs. `tools/waveform/prepare_fixtures.py` regenerates them through the root `just prepare-waveform-fixtures` recipe. Integration tests should live in a new `tests/extract_atb_cli.rs`, with schema assertions in `tests/schema_cli.rs` and help assertions in `tests/cli_contract.rs`.

Public command guidance lives in `docs/public/commands/extract.md`; stable command and machine-output contracts live in `docs/public/reference/command-model.md` and `docs/public/reference/machine-output.md`; agent routing lives in `docs/skills/wavepeek.md`; user-visible release history belongs under `CHANGELOG.md` `Unreleased`.

## Open Questions

There are no blocking product questions. If exact shared mapping code can be extracted from AXI without changing behavior and with a smaller total diff, it may be reused; otherwise ATB keeps small protocol-local helpers. Review must confirm that schema branches reject cross-profile fields and that event ordering remains deterministic.

## Plan of Work

The first milestone creates this committed living plan so every later commit is reproducible. Update `Progress`, discoveries, decisions, and evidence after each milestone. Commit the plan with a conventional documentation commit.

The second milestone adds runtime behavior. Extend `src/cli/extract.rs` with boxed `AtbArgs` and complete long help. Extend `src/cli/mod.rs` and `src/engine/mod.rs` with `ExtractAtb` variants and dispatch. Create `src/engine/atb.rs` with profile tables, source-file parsing, canonical mapping, include matching, validation, plan construction, generic-row adaptation, DTOs, and focused unit tests. Extend `src/output.rs` for human and JSONL context output. Verify with `cargo fmt`, `cargo check`, unit tests, and help output before committing.

The third milestone adds machine contracts. Define ATB engine-to-contract DTO conversions in `src/contract/output.rs`, context and item variants in `src/contract/stream.rs`, source input in `src/contract/input.rs`, and exact schema wiring in `src/contract/schema.rs`. Add a small `src/contract/atb_schema.rs` only if schemars cannot express the required exact profile-specific branches clearly; register it in `src/contract/mod.rs`. Add focused assertions to `tests/schema_cli.rs`, run `just update-schema`, inspect generated diffs, and run `just check-schema` before committing.

The fourth milestone demonstrates end-to-end behavior. Add a concise Verilog fixture that creates ATB transfer, flush, and synchronization-request events, reset-suppressed events, and same-timestamp events. Generate VCD and FST and register both generated paths in `tests/fixtures/waveform_policy.json`. Add `tests/extract_atb_cli.rs` for explicit and automatic mapping, aliases, three profiles, source-file mode, output modes, absolute display, deterministic ordering, reset gating, windows, maximum row truncation, VCD/FST parity, all validation errors, and accepted partial configurations. Extend CLI fixture contracts only where the behavior is durable. Run focused integration tests and commit.

The fifth milestone updates user guidance without duplicating generated help. Add ATB semantics and examples to `docs/public/commands/extract.md`, add only cross-cutting output notes where needed, add a short ATB routing note and stateless limitation to `docs/skills/wavepeek.md`, and add one `Unreleased / Added` changelog entry linked to issue #68. Update `docs/dev/architecture.md` only if module ownership descriptions enumerate protocol modules. Run docs tests and commit.

The sixth milestone validates and reviews the complete branch. Run all standard gates inside the devcontainer. Then launch read-only review lanes for correctness/tests, public contracts/docs, architecture, and performance. Apply fixes in the main worktree, rerun affected tests, and run a fresh independent control review. Record findings and fixes here. Remove this WIP plan in its own cleanup commit after the retrospective is complete, because repository policy excludes branch-local plans from merge.

The final milestone pushes the reviewed branch to `origin` and creates a GitHub pull request against `main`. The PR title uses the same conventional feature wording as the main implementation. The body summarizes behavior and validation and includes `Closes #68`, which links and closes the issue when merged. Verify the PR URL, base/head branches, issue-closing reference, and clean local worktree.

### Concrete Steps

Run commands from `/workspaces/wavepeek/.worktrees/feat-extract-atb`.

Commit the plan:

    git add docs/tracker/wip/issue-68-extract-atb.md
    git commit -m "docs(plan): add ATB extraction ExecPlan"

Iterate on runtime and focused tests:

    cargo fmt
    cargo check
    cargo test -q --lib engine::atb
    cargo test -q --test cli_contract
    cargo test -q --test extract_atb_cli

Regenerate fixtures and schemas through repository recipes:

    just prepare-waveform-fixtures
    just update-schema
    just check-schema

Run complete local and CI gates:

    just test
    just check
    just ci

Expected successful gates exit with status zero. Optional FSDB messages may report a documented skip when the local SDK is unavailable; this is not a failure.

Review and publish after all fixes:

    git diff --check main...HEAD
    git status --short
    git push -u origin feat/extract-atb
    gh pr create -R kleverhq/wavepeek --base main --head feat/extract-atb \
      --title "feat(extract): add ATB event extraction" \
      --body-file tmp/issue-68-pr.md
    gh pr view -R kleverhq/wavepeek --json url,baseRefName,headRefName,title,body,state

### Validation and Acceptance

The feature is accepted when `wavepeek extract atb --help` shows the requested options and profile/default semantics, and when the generated source fixture proves all three event kinds. Human output must show canonical ordered mappings and event lines; `--abs` must switch mapped payload display to absolute waveform paths. JSON must validate against `schema/output.json`. JSONL must validate begin, item, diagnostic, and end records against `schema/stream.json`. Source files must validate against `schema/input.json` and execute equivalently to CLI configuration.

ATB-A must reject `syncreq`; ATB-B and ATB-C must accept it. Every profile must reject `atclken` and `atwakeup`. CLI aliases with underscores, legacy ATB version spellings, and mixed case must normalize to canonical hyphenated output. The input schema must accept canonical values only. Mapping failures must be deterministic and actionable. Reset-low sampled cycles must emit no events. A complete transfer emits only when valid and ready are both true, a flush emits only when both flush signals are true, and every sampled high `syncreq` emits independently after at least one handshake channel enables the command. Same-timestamp events appear as transfer, flush, then sync-request. Window and maximum options must preserve existing generic extraction semantics and diagnostics.

`just test`, `just check`, and `just ci` must all exit zero. The final pull request must target `main`, originate from `feat/extract-atb`, contain the reviewed commits, and include `Closes #68`.

### Idempotence and Recovery

Fixture and schema generation recipes are deterministic and safe to rerun. Do not edit generated schema JSON manually. Do not delete unrelated files under `tmp/`; use unique issue-68 filenames. If a commit hook fails, fix the reported problem and commit again without bypassing hooks. If a milestone needs rollback, revert only that milestone commit rather than resetting unrelated branch work. If push succeeds but PR creation fails, rerun only `gh pr create`; first check `gh pr list --head feat/extract-atb` to avoid a duplicate. The plan file is removed only after its final state has been committed or its evidence has been reflected in permanent commits and PR text.

### Artifacts and Notes

Normative protocol evidence comes from Arm IHI 0032C Issue C. Sections 3.1-3.2 state that transfer occurs only when `ATVALID` and `ATREADY` are both high and that signals are sampled on the rising edge of `ATCLK`. Section 4.2 describes the `AFVALID`/`AFREADY` flush handshake. Section 4.4 states that `SYNCREQ` is a single-`ATCLK` synchronization request pulse independent of other ATB signals. Section 6.2 defines conditional ATB-C `ATWAKEUP`, which this initial command excludes. Appendix A Table A-1 gives the profile signal matrix.

The starting commit is `caea6c3` on branch `feat/extract-atb`, with a clean worktree. The nearest runtime model is `src/engine/axi.rs`; the nearest generic evaluator is `src/engine/extract.rs`.

### Interfaces and Dependencies

No new dependency is required. Continue using clap for CLI parsing, serde for runtime JSON source parsing and DTO serialization, schemars for generated schemas, regex for include matching, and the existing waveform readers and generic extraction runtime.

At completion, `src/cli/extract.rs` exposes `AtbArgs`; `src/engine/mod.rs` exposes and dispatches `CommandName::ExtractAtb`; and `src/engine/atb.rs` exposes `run(AtbArgs) -> Result<CommandResult, WavepeekError>` and `run_jsonl(AtbArgs, &mut JsonlWriter<_>) -> Result<(), WavepeekError>`. Its public engine DTOs represent `AtbData`, `AtbContext`, ordered `AtbSignalMapping` values, event kind, event payload values, and events.

The contract layer must serialize JSON command data under `extract atb`, serialize a context-bearing JSONL begin record and ATB event items, and expose exact input/output/stream schema branches for all three profiles. Engine DTOs remain internal behavior models; contract DTOs remain the sole owners of machine serialization and schema derivation.

Revision note: 2026-07-20 initial ExecPlan created after repository and protocol research. It resolves the implementation path, acceptance behavior, commit boundaries, validation gates, review workflow, and PR linkage required by issue #68. The progress entry was updated immediately before the plan commit so a fresh checkout starts from an accurate milestone boundary.

Revision note: 2026-07-20 corrected the profile, payload, handshake-channel, alias, and clause details after preserving and line-reading the full issue body locally. The initial summary had incorrectly treated conditional `ATWAKEUP` as an extraction field and transfer payload as required; issue #68 explicitly excludes wake-up mappings and permits handshake-only transfers.

Revision note: 2026-07-20 updated milestone progress, test evidence, and discoveries after implementing the complete local feature slice. The next boundary is the implementation commit, followed by full gates and review.
