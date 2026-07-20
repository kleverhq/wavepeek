# Add end-to-end APB event extraction

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. All commands below run from `/workspaces/wavepeek/.worktrees/feat-extract-apb` on branch `feat/extract-apb`.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract apb` against VCD, FST, or supported FSDB waveforms and receive deterministic APB Setup, waited Access, and completed Access event rows without hand-writing generic expressions. The command will understand APB3, APB4, and APB5 signal sets; map canonical APB names to waveform paths; sample predicates and payload immediately before each rising `pclk` edge; and emit equivalent human, JSON, and streaming JSONL representations.

The feature is observable by generating the APB fixtures with `just prepare-waveform-fixtures`, running an invocation such as `cargo run -- extract apb --waves tests/fixtures/generated/extract_apb4.vcd --scope top --include '^apb_'`, and seeing `setup` and `access-complete` rows. Adding `--include-wait` must also show one `access-wait` row for each enabled waited Access cycle.

## Non-Goals

APB2 support is out of scope. This command will not assemble events into transactions, correlate Setup and Access rows, count waits, decode registers, interpret sparse writes, validate protocol sequencing or stability, validate parity/check signals, interpret RME or user attributes, or process `pwakeup`. It will map one concrete Completer select as canonical `psel`; multi-Completer selection and one-hot checks are out of scope.

## Progress

- [x] (2026-07-20 08:30Z) Read issue #66, repository guidance, APB source material, existing generic extraction runtime, AXI adapter, contracts, docs, fixture policy, and quality workflow.
- [x] (2026-07-20 08:30Z) Create this branch-local ExecPlan with the implementation path and acceptance evidence.
- [x] (2026-07-20 08:59Z) Add CLI, dispatch, APB engine adapter, mapping, event payload filtering, human output, and focused unit/integration tests.
- [x] (2026-07-20 08:59Z) Add exact input/output/stream contract branches and regenerate schema artifacts.
- [x] (2026-07-20 08:59Z) Add APB3/APB4/APB5 source-backed VCD/FST fixtures and parity/behavior tests.
- [x] (2026-07-20 09:05Z) Update public docs, packaged skill guidance, architecture module map, and changelog.
- [ ] Run focused tests and `just check`, commit coherent milestones, and resolve all failures. (Completed: 13 APB integration tests, 35 schema tests, focused help/docs/skill tests, `just check-schema`, and the runtime milestone commit; remaining: full local gate.)
- [ ] Perform a strict self-review against issue #66, run `just ci`, remove this WIP plan, and commit review fixes.
- [ ] Push `feat/extract-apb`, open a GitHub PR that closes issue #66, and verify the remote PR state.

## Surprises & Discoveries

- Observation: The existing generic extraction runtime requires every source to have at least one payload signal, but APB requires mapped `pwrite`, so every APB source can satisfy that invariant without changing the shared runtime.
  Evidence: `src/engine/extract.rs` resolves each `ExtractSource` payload and issue #66 requires `pwrite` for every APB profile.

- Observation: The existing AXI adapter already opens the waveform once, builds generic `ExtractSource` values, and adapts generic rows into typed protocol rows, which exactly matches the requested stateless APB design.
  Evidence: `src/engine/axi.rs::build_axi_plan`, `GenericToAxiSink`, and `extract::run_plan_with_waveform_sink`.

- Observation: Exact profile, mode, wait-setting, event, and direction cross-products substantially expand generated output and stream schemas, but localized generation keeps each branch machine-checkable and source code small.
  Evidence: `just check-schema` passes after `src/contract/apb_schema.rs` generates closed APB mapping and payload branches; focused output and stream validators reject cross-profile and event-inappropriate keys.

- Observation: Rust regex intentionally lacks look-around, so fixture include expressions must enumerate valid candidates when decoys share a broad prefix.
  Evidence: the APB5 and APB4 parity tests use anchored alternations; explicit broad-prefix tests still exercise warnings and ambiguity diagnostics.

## Decision Log

- Decision: Implement APB as `src/engine/apb.rs` over `src/engine/extract.rs`, with no stateful reducer and no changes to generic event ordering or sampling.
  Rationale: Issue #66 requires one public APB row per generic source row, and the existing runtime already owns pre-edge sampling, bounds, limits, diagnostics, and JSONL streaming.
  Date/Author: 2026-07-20 / pi

- Decision: Use Arm IHI 0024E Issue E as the runtime metadata source for all three profiles and expose `issue: "E"`.
  Rationale: Issue E provides the current APB3/APB4/APB5 compatibility matrix. Transfers and waits are in sections 3.1–3.3, response validity in 3.4, optional APB5 signaling in 3.6–3.8, state behavior in 4.1, validity in Appendix A, and profile signal presence in Tables B-1–B-3.
  Date/Author: 2026-07-20 / pi

- Decision: Keep APB mapping code local and leave AXI code unchanged.
  Rationale: The current AXI implementation has AXI-specific errors and channel assumptions embedded with matching. Extracting shared helpers would widen the risk and diff without changing required APB behavior.
  Date/Author: 2026-07-20 / pi

- Decision: Keep this ExecPlan committed during implementation and remove it before the final PR-ready commit.
  Rationale: `docs/tracker/wip/` is explicitly for restartable branch-local work and its guidance requires cleanup before merge unless maintainers request retention.
  Date/Author: 2026-07-20 / pi

## Outcomes & Retrospective

The runtime and contract milestones are complete. The command now classifies sampled APB events through the generic runtime, profile/mode-aware schemas validate machine output, and three source-backed fixtures prove VCD/FST parity. Documentation, full gates, self-review, and remote delivery remain.

## Context and Orientation

`wavepeek` is a Rust CLI. `src/cli/extract.rs` declares the `extract` subcommands and their arguments. `src/cli/mod.rs` converts parsed CLI commands into `src/engine/mod.rs::Command`, which dispatches normal and JSONL execution. `src/output.rs` renders human output and delegates machine serialization to contract data-transfer objects.

`src/engine/extract.rs` is the protocol-neutral extraction runtime. An `ExtractPlan` contains ordered `ExtractSource` values. Each source has an edge expression, a pre-edge predicate, and ordered payload signal names. The runtime merges candidate rising-edge timestamps, evaluates each predicate at one dump tick before the edge, enforces bounds and the row limit, and emits `ExtractGenericRow` values through an `ExtractRowSink`. APB must reuse this path rather than scan the waveform separately.

`src/engine/axi.rs` is the nearest profile-adapter example. It parses a profile and optional source JSON, combines explicit standard-name mappings with include-regex auto-mapping, creates generic sources, and adapts rows into typed AXI transfers. APB requires the same integration shape but different event predicates and event-specific payload filtering.

For APB, a Setup event means sampled `psel && !penable`. In mapped-PREADY mode, a waited Access event means `psel && penable && !pready`, and completion means `psel && penable && pready`. In implicit-HIGH mode, completion means `psel && penable` and waits cannot be requested. These predicates are independent and classify sampled bus states; they do not validate that a Setup event preceded an Access event. If `presetn` is mapped, known HIGH is an additional gate. Unknown control values do not make a logical predicate true.

The profiles are APB3 (`pclk`, `presetn`, `psel`, `penable`, `pready`, `pwrite`, `paddr`, `pwdata`, `prdata`, `pslverr`), APB4 (APB3 plus `pprot`, `pstrb`), and APB5 (APB4 plus `pnse`, `pauser`, `pwuser`, `pruser`, `pbuser`). `pclk`, `psel`, `penable`, and `pwrite` are always required. `pready` is required only in mapped mode and forbidden in implicit-HIGH mode. `presetn` and all payload fields other than required `pwrite` are optional. `pwakeup` and all APB5 check signals are not valid standard mapping keys.

Every APB row includes `pwrite` and derives `direction` as `read`, `write`, or `unknown`. Request fields are available on every event. Read/response fields are only emitted on `access-complete`. Known reads omit write data/user fields; known writes omit read data/user fields; unknown direction preserves mapped direction-specific observations allowed for that event. Full sampled vectors and X/Z literals remain unchanged.

`src/contract/input.rs`, `src/contract/output.rs`, `src/contract/stream.rs`, `src/contract/schema.rs`, and protocol-specific schema helpers define runtime JSON shapes and generated schemas. The current exact schema family is version 2.2. The new source kind is `extract.apb.source`; old published schema versions are not modified. Generated profile, mode, event, direction, mapping, and payload objects must be closed where issue #66 requires exact keys, while current outer objects remain extension-friendly. `just update-schema` regenerates `schema/output.json`, `schema/stream.json`, `schema/input.json`, and `schema/catalog.json`; these files must not be edited manually.

`tests/fixtures/source/` holds HDL source fixtures. `just prepare-waveform-fixtures` generates ignored VCD/FST artifacts under `tests/fixtures/generated/`. `tests/fixtures/waveform_policy.json` records every source-backed output and rationale. Integration tests must invoke the built binary against those generated files and compare behavior across VCD and FST. Existing AXI coverage in `tests/extract_axi_cli.rs` provides command, mapping, source, JSON, JSONL, limit, diagnostics, and schema patterns.

Public command guidance is in `docs/public/commands/extract.md`; cross-cutting machine rules are in `docs/public/reference/machine-output.md`; workflow examples are in `docs/public/workflows/extract-handshake.md`; routing guidance for agents is in `docs/skills/wavepeek.md`; command discovery is also covered by `docs/public/commands/overview.md`. `CHANGELOG.md` records the unreleased addition. Exact flag documentation belongs in Clap help in `src/cli/extract.rs`, not duplicated as a Markdown flag table.

## Open Questions

There are no blocking open questions. If implementation reveals a conflict between issue #66 and an existing public contract, record it in the Decision Log and choose the smallest behavior that satisfies the explicit issue without changing existing commands.

## Plan of Work

### Milestone 1: Establish the command and runtime behavior

Extend `src/cli/extract.rs` with an `Apb` variant, `ApbProfileArg`, `PreadyModeArg`, and `ApbArgs`. The argument surface must implement the defaults and conflicts from issue #66, including case-insensitive canonical values and a custom validation error for `--include-wait` with implicit-HIGH mode. Extend `src/cli/mod.rs` and `src/engine/mod.rs` with `ExtractApb` dispatch, output-mode handling, command naming, and JSONL execution.

Create `src/engine/apb.rs`. Define profile specifications, ordered standard names, source parsing, explicit and include-based auto-mapping, APB context/data/event types, collecting and JSONL sinks, and a generic-row adapter. Build one generic source for Setup, optionally one for waits, and one for completion. Put the full mapped candidate payload needed by each public event into each generic source, then filter the sampled payload in the sink according to event kind and sampled `pwrite`. The milestone is complete when focused integration tests show mapped and implicit-HIGH modes, wait inclusion, reset behavior, event ordering, and human output.

### Milestone 2: Stabilize machine contracts and source files

Add APB source DTOs and runtime source parsing for exact `$schema`, `kind: "extract.apb.source"`, canonical profile/mode values, null rejection, and mode/map validation. Add APB output and JSONL context DTOs. Add a dedicated `src/contract/apb_schema.rs` if profile/event cross-products are clearer there than in `schema.rs`, mirroring the localized AXI schema generator without coupling the two protocols.

Generate exact branches for APB3/APB4/APB5, mapped versus implicit-HIGH PREADY, wait inclusion, event kinds, directions, profile mappings, and event-appropriate payload keys. Extend the command enums and stream record wrappers. Run `just update-schema`, then `just check-schema`. This milestone is complete when source documents validate against `schema/input.json`, runtime JSON and every JSONL line validate against the generated schemas, forbidden combinations fail, and existing AXI contracts remain unchanged except for additive top-level APB branches.

### Milestone 3: Add cross-format fixtures and exhaustive integration coverage

Add concise source HDL fixtures for APB3, APB4, and APB5 under `tests/fixtures/source/`. Each must produce deterministic Setup, zero or multiple waits, completion, read/write payload selection, and reset/control edge cases needed by its tests. Add both VCD and FST outputs to `tests/fixtures/waveform_policy.json`, regenerate them, and add `tests/extract_apb_cli.rs` with reusable invocation helpers and semantic assertions rather than large snapshots.

Cover explicit mapping, auto-mapping normalized suffixes, unmatched warnings, ambiguity, indexed/literal `psel` requiring an explicit map, check-signal decoys, source mode, bounds, limits, empty/truncation diagnostics, JSONL summaries, APB3/APB4/APB5 signal sets, unknown direction, response payload restrictions, repeated waits/events, and VCD/FST equivalence. Add command/help/schema fixture assertions to the existing focused test files where those contracts already live.

### Milestone 4: Document, review, and deliver

Update the public extract command topic, command overview, extraction workflow, machine-output reference, and packaged skill with concise routing-oriented APB guidance. Add an Unreleased changelog bullet linked to issue #66. Update `docs/dev/architecture.md` only if the new protocol module changes the documented module map or ownership description.

Run formatting, focused tests, and `just check`; commit fixes. Then review every changed hunk against the issue’s acceptance criteria, inspect generated artifacts for closed payload/map behavior, and run `just ci`. Record evidence here, remove this WIP plan, and make the final review commit. Push the branch, open a PR whose body contains `Closes #66`, and inspect the PR URL, base/head refs, issue link, and checks state.

### Concrete Steps

From the repository root, use these commands incrementally:

    cargo fmt --all
    cargo test --test extract_apb_cli
    cargo test --test cli_contract
    cargo test --test schema_cli
    just prepare-waveform-fixtures
    just update-schema
    just check-schema
    just check
    just ci

Expected focused-test evidence after implementation resembles:

    test result: ok. <N> passed; 0 failed

Expected manual output includes context and rows such as:

    name: uart
    profile: apb4
    issue: E
    pready_mode: mapped
    include_wait: true
    events:
    @20ns sample@19ns [setup write] ...
    @30ns sample@29ns [access-wait write] ...
    @40ns sample@39ns [access-complete write] ...

Delivery commands, after all gates pass, are:

    git push -u origin feat/extract-apb
    gh pr create --base main --head feat/extract-apb --title "feat: add APB event extraction" --body-file tmp/pr-66.md
    gh pr view --json url,state,baseRefName,headRefName,body,statusCheckRollup

The PR body must summarize behavior and verification and include `Closes #66` on its own line.

### Validation and Acceptance

Acceptance requires observable APB3, APB4, and APB5 extraction with APB4 as default. Mapped mode must emit Setup and completion by default and add waited Access rows only when requested. Implicit-HIGH mode must forbid `pready` and wait capture. Optional sampled reset gating must suppress rows without adding state. Explicit maps, auto-maps, and source JSON must enforce profile and mode rules.

Human, JSON, and JSONL must agree on Issue E, profile, mode, effective wait setting, event kind, direction, mappings, payload selection, ordering, bounds, row limits, diagnostics, and summaries. Generated input/output/stream schemas must validate positive runtime artifacts and reject forbidden combinations. APB3/APB4/APB5 generated VCD and FST fixture invocations must produce equivalent semantic output. Existing tests must remain green. `just check` and `just ci` must both exit successfully.

The delivery is complete only when the branch is pushed and an open PR against `main` contains `Closes #66` and references the successful validation commands.

### Idempotence and Recovery

Fixture generation, schema generation, formatting, and tests are idempotent. Generated waveform files are ignored and can be recreated; do not delete unrelated files under `tmp/` or `tests/fixtures/generated/`. Schema artifacts must be regenerated through `just update-schema`, never manually edited. If a commit fails hooks, fix the reported issue and commit normally rather than bypassing hooks. If push or PR creation fails, preserve all local commits, diagnose authentication or remote state, retry the same command, and verify with `gh pr view`.

### Artifacts and Notes

The issue contract is available during implementation in ignored scratch file `tmp/issue-66.txt`. The source-of-truth protocol document inspected was `/home/ubuntu/.pi/agent/skills/amba-apb/references/IHI0024E_amba_apb_architecture_spec.pdf`.

Focused evidence at the runtime milestone:

    cargo test --test extract_apb_cli: 13 passed
    cargo test --test schema_cli: 35 passed
    cargo test --test cli_contract extract_apb_help_is_self_descriptive: 1 passed
    cargo test --test docs_cli public_extract_docs: 2 passed
    cargo test --test skill_cli: 3 passed
    just check-schema: schema contract OK
    7308b55 feat(extract): add APB event extraction

At completion, replace this evidence with final gate output, commit hashes, and the PR URL before removing the WIP file in the final cleanup commit; the committed history will retain the completed plan.

Revision note (2026-07-20): Marked runtime, contracts, and fixtures complete after focused tests and schema contract validation; recorded schema and regex discoveries.

Revision note (2026-07-20): Marked documentation complete after embedded docs and packaged skill contract tests passed; recorded the runtime milestone commit.

### Interfaces and Dependencies

No new crate dependency is required. Use Clap for parsing, Serde for source JSON and runtime serialization, Schemars plus localized JSON construction for schemas, Regex for include matching, and the existing waveform and expression runtime.

`src/engine/apb.rs` must expose:

    pub fn run(args: ApbArgs) -> Result<CommandResult, WavepeekError>;
    pub fn run_jsonl<W: std::io::Write>(
        args: ApbArgs,
        writer: &mut JsonlWriter<W>,
    ) -> Result<(), WavepeekError>;

It must define public engine DTOs equivalent to:

    pub struct ApbSignalMapping { standard, display, path }
    pub struct ApbEventPayload { standard, display, path, value }
    pub struct ApbEvent { time, sample_time, profile, event, direction, payload }
    pub struct ApbContext { name, profile, issue, pready_mode, include_wait, mappings }
    pub struct ApbData { context fields plus events }

Names may use the repository’s existing `Axi*` naming pattern, but machine field names must be exactly `name`, `profile`, `issue`, `pready_mode`, `include_wait`, `mappings`, `events`, `time`, `sample_time`, `event`, `direction`, and `payload`.

The APB adapter must call `extract::run_plan_with_waveform_sink` with `CommandName::ExtractApb` and adapt through `ExtractRowSink`. It must not introduce a reducer, pending transaction state, or a second waveform scan. Contract DTOs in `src/contract/` must convert these engine values without serializing human-only display fields. `src/output.rs` must render canonical mapping order and event payload order, using absolute paths only when `--abs` is active.

Revision note (2026-07-20): Initial plan created after repository and protocol research to make the complete issue #66 implementation restartable and evidence-driven.
