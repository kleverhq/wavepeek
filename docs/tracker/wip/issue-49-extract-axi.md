# Issue 49: Add `wavepeek extract axi`

This ExecPlan is a living document. It is maintained according to the `exec-plan` skill. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must remain current as work proceeds.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract axi --waves <FILE>` to extract AXI-family ready/valid transfer rows from a VCD, FST, or supported FSDB waveform without manually writing one generic extraction source per AXI channel. The command supports only the initial profiles requested by issue 49: `axi3`, `axi4`, and `axi4-lite`. The default profile is `axi4`.

The command maps standard AXI signal names such as `awvalid`, `awready`, and `awaddr` to waveform paths, samples every complete ready/valid channel on `posedge aclk`, and prints one row per completed transfer. A completed transfer means that the channel `VALID` signal and the channel `READY` signal are both true at the sample point. If `aresetn` is mapped, reset must also be true at that same sample point. The sample point is one dump tick before the clock edge, matching `extract generic` pre-edge semantics.

A user can see the feature working by running a command like:

    wavepeek extract axi --waves tests/fixtures/axi_lite.vcd --profile axi4-lite --map aclk=top.clk --include 'top.dut.axi_.*'

The expected human output starts with profile metadata and mappings, then transfer rows ordered by event time and channel order. JSON and JSONL output expose the same transfer rows in machine-readable form.

## Non-Goals

This plan does not add AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, or any credited-transport support. It does not reconstruct bursts, join read and write transactions, infer multi-port interfaces, or extract static interface-control signals that are not part of ready/valid transfer channels. It does not add implicit `ready = 1`; a channel must have both `VALID` and `READY` mapped to be extractable.

## Progress

- [x] (2026-07-04T21:15Z) Read issue 49 and repository guidance for source, tests, docs, and WIP tracker files.
- [x] (2026-07-04T21:15Z) Started subagent research for the existing `extract generic` implementation, test patterns, and AXI profile source verification.
- [x] (2026-07-04T21:15Z) Created this initial self-contained execution plan before implementation.
- [x] (2026-07-04T21:54Z) Incorporated subagent findings into this plan, including concrete generic-extraction reuse points and AXI source citations from Arm IHI 0022H.c.
- [x] (2026-07-04T21:54Z) Implemented the profile model for `axi3`, `axi4`, and `axi4-lite`, including standard signal lists, channel order, issue metadata, profile parsing, and unit tests.
- [x] (2026-07-04T21:54Z) Implemented CLI parsing for `wavepeek extract axi`, including `--waves`, `--profile`, `--source`, `--name`, `--from`, `--to`, `--scope`, `--map`, `--include`, `--max`, `--abs`, `--json`, and `--jsonl`.
- [x] (2026-07-04T21:54Z) Implemented source JSON loading for `kind: "extract.axi.source"` with optional `profile`, optional `name`, `includes`, and `maps`.
- [x] (2026-07-04T21:54Z) Implemented AXI signal mapping: regex include collection, token-based auto-mapping, explicit-map overrides, warnings for unmatched included candidates, ambiguity rejection, lowercase standard-name normalization, and validation.
- [x] (2026-07-04T21:54Z) Implemented extraction execution by constructing generic sources: one `posedge aclk` event per complete AXI channel, a pre-edge predicate using `aresetn` when present, and mapped non-ready/valid payload signals.
- [x] (2026-07-04T21:54Z) Implemented human, JSON, and JSONL AXI output contracts.
- [x] (2026-07-04T21:58Z) Added integration tests for default `axi4`, explicit `axi3` with `wid`, explicit `axi4-lite`, source JSON, mapping collisions, partial ready/valid validation, reset gating, JSON, JSONL, schema validation, and handshake-only payload `{}`.
- [x] (2026-07-04T21:54Z) Updated public docs, command help tests, schema snapshots, packaged skill, and changelog. Internal architecture docs did not need a module-boundary update because the AXI adapter is a new engine module over the existing extract runtime.
- [x] (2026-07-04T22:12Z) Ran staged subagent code reviews for code, contract/schema/tests, and docs/help lanes, then applied findings.
- [x] (2026-07-04T22:31Z) Ran focused tests and `just check`. Completed: `cargo check -q`, `cargo test -q --test extract_axi_cli`, full `cargo test -q --test cli_contract`, full `cargo test -q --test schema_cli`, `cargo test -q --test docs_cli`, `cargo test -q --test skill_cli`, `just check-schema`, and `just check`.
- [x] (2026-07-04T22:33Z) Ran `just ci`; it passed, including FSDB checks in this environment.
- [x] (2026-07-04T23:06Z) Ran final post-commit control review and fixed deploy-check support for schema v2.2 stream/input contracts.
- [ ] Commit coherent slices, push `feat/extract-axi`, and open a draft PR for issue 49.

## Surprises & Discoveries

- Observation: `extract generic` had the right runtime pieces but kept its plan and sink types private, so AXI needed a small visibility refactor rather than a second waveform traversal engine.
  Evidence: `src/engine/extract.rs` now exposes `ExtractPlan`, `ExtractSource::new`, `ExtractRunArgs`, `ExtractRowSink`, and `run_plan_with_sink` for crate-local reuse.

- Observation: AXI transfer rows can be handshake-only with an empty payload, while generic source JSON requires at least one payload entry.
  Evidence: `ExtractSource::new` accepts empty payload vectors for generated AXI sources; `tests/extract_axi_cli.rs` verifies JSON payload `{}` for a complete AW handshake with only ready/valid mapped.

- Observation: Issue 49's signal lists match Arm IHI 0022H.c for the requested initial profiles.
  Evidence: The AXI verification subagent cited section A1.2 and A1.2.1 for the five channels and channel VALID/READY model, Table A3-1 page A3-42 for handshake pairs, Tables A2-2 through A2-6 pages A2-33 through A2-37 for AXI3/AXI4 signal lists, and Table B1-1 page B1-120 for AXI4-Lite signals.

- Observation: A single include candidate can otherwise match multiple standard names, and common signal `aresetn` can be mis-tokenized as an AR-channel signal.
  Evidence: Code review flagged `axi_awvalid_awready` and `ar_esetn` cases. `src/engine/axi.rs` now rejects one candidate matching multiple unmapped standards and special-cases common signals before channel-prefix splitting.

- Observation: The input schema root changed from one generic source object to a `oneOf` over generic and AXI documents, so repository schema checks needed explicit updates.
  Evidence: Contract review flagged stale expectations in `tests/schema_cli.rs` and `tools/schema/check_schema_contract.py`; `just check-schema` now passes.

- Observation: Adding `extract axi` changes the public output, stream, and input schema families.
  Evidence: Control review flagged that keeping family version `2.1` would mutate an already published contract. The schema family versions, URLs, catalog, docs references, tests, benchmark source inputs, and checker expectations now use `2.2`.

- Observation: The docs deploy checker had separate schema-shape assumptions from the schema contract checker.
  Evidence: Final post-commit review flagged stale `tools/docs/check_deploy.py` checks for stream commands and input root shape. `tools/docs/test_check_deploy.py`, `just test-aux`, and `just check` now cover the v2.2 stream/input contracts.

## Decision Log

- Decision: Store this ExecPlan at `docs/tracker/wip/issue-49-extract-axi.md`.
  Rationale: The branch needs a tracked, restartable plan that survives a fresh checkout. The local WIP breadcrumb explicitly designates `docs/tracker/wip/` for active execution plans.
  Date/Author: 2026-07-04 / coding agent

- Decision: Limit the first implementation to `axi3`, `axi4`, and `axi4-lite` even though issue 49 mentions broader future profile references.
  Rationale: The user explicitly requested only these three profiles for this task, and adding more profiles would expand validation and source-reference work without improving the requested outcome.
  Date/Author: 2026-07-04 / coding agent

- Decision: Reuse the existing generic extraction semantics instead of creating a second waveform traversal engine.
  Rationale: Issue 49 requires `extract axi` to build an AXI-family plan on top of `extract generic` semantics: `posedge aclk`, pre-edge sampling, and logical ready/valid predicates. Reusing the existing runtime minimizes duplicate code and preserves deterministic output behavior.
  Date/Author: 2026-07-04 / coding agent

- Decision: Implement AXI as `src/engine/axi.rs`, backed by crate-local `src/engine/extract.rs` plan execution helpers.
  Rationale: Keeping AXI mapping/profile logic separate avoids bloating the protocol-neutral generic extractor, while the shared run-plan entrypoint keeps waveform scanning, pre-edge sampling, limits, diagnostics, and JSONL row emission behavior aligned.
  Date/Author: 2026-07-04 / coding agent

- Decision: Add diagnostic `WPK-W0004` for AXI include candidates that do not match any profile standard signal.
  Rationale: Issue 49 asks unmatched include candidates to be ignored with warnings. Existing warning codes covered disabled limits, truncation, and empty results only.
  Date/Author: 2026-07-04 / coding agent

- Decision: Allow generated AXI sources to have empty payload lists.
  Rationale: A ready/valid handshake is still a useful transfer row even when only the channel handshake signals are mapped. The generic source-file contract still requires non-empty payload lists for user-authored generic sources.
  Date/Author: 2026-07-04 / coding agent

- Decision: Bump output, stream, and input schema family versions from `2.1` to `2.2`.
  Rationale: The feature adds a new output command branch, a stream context/item branch, and a new input source document kind. Keeping the `2.1` schema URLs would silently change already published contracts.
  Date/Author: 2026-07-04 / coding agent

## Outcomes & Retrospective

Milestones 1 through 6 are implemented in the working tree. `extract axi` can extract AXI4-Lite rows from a hand-written VCD fixture, defaults to `axi4`, extracts AXI3 `wid`, emits JSON and JSONL validated by the current schema artifacts, supports source JSON, rejects ambiguous and partial mappings, and documents the new user-facing command. Review findings have been addressed, `just test-aux`, `just check`, and `just ci` pass. The remaining work is amend/push and draft PR.

## Context and Orientation

The repository root for this worktree is `/workspaces/wavepeek/.worktrees/feat-extract-axi`. All edits must stay in this worktree. `wavepeek` is a Rust CLI for deterministic VCD, FST, and optional FSDB waveform inspection.

The existing command family is split across these layers:

- `src/cli/` owns clap argument parsing and command dispatch structures.
- `src/engine/` owns command behavior and waveform/expression execution.
- `src/waveform/` hides VCD, FST, and optional FSDB backends behind a shared facade.
- `src/output.rs` owns human, JSON envelope, and JSONL rendering helpers.
- `src/contract/` and `schema/` own machine-output and input schema contracts.
- `docs/public/` contains packaged user docs served by `wavepeek docs`.
- `tests/` contains integration and contract tests.

`extract generic` is the existing protocol-neutral extraction command. It accepts event sources, logical predicates, payload signals, and source JSON, then emits sampled rows. `extract axi` should not reimplement low-level waveform scanning. It should build a small AXI-specific plan that can be executed with the same sampling and logical evaluation behavior.

AXI means Advanced eXtensible Interface, an Arm AMBA bus protocol family. A ready/valid channel transfers data only when the channel `VALID` and `READY` signals are both asserted. The five AXI full-interface transfer channels are address write (`aw`), write data (`w`), write response (`b`), address read (`ar`), and read data (`r`). AXI4-Lite uses the same channel names but has a reduced signal set for single-beat memory-mapped control-register access.

Issue 49 defines these standard names for the initial profiles:

AXI3 common clock/reset names are `aclk` and optional `aresetn`. AXI3 channels are:

- AW: `awid`, `awaddr`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awvalid`, `awready`.
- W: `wid`, `wdata`, `wstrb`, `wlast`, `wvalid`, `wready`.
- B: `bid`, `bresp`, `bvalid`, `bready`.
- AR: `arid`, `araddr`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arvalid`, `arready`.
- R: `rid`, `rdata`, `rresp`, `rlast`, `rvalid`, `rready`.

AXI4 common clock/reset names are `aclk` and optional `aresetn`. AXI4 channels are:

- AW: `awid`, `awaddr`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awqos`, `awregion`, `awuser`, `awvalid`, `awready`.
- W: `wdata`, `wstrb`, `wlast`, `wuser`, `wvalid`, `wready`.
- B: `bid`, `bresp`, `buser`, `bvalid`, `bready`.
- AR: `arid`, `araddr`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arqos`, `arregion`, `aruser`, `arvalid`, `arready`.
- R: `rid`, `rdata`, `rresp`, `rlast`, `ruser`, `rvalid`, `rready`.

AXI4-Lite common clock/reset names are `aclk` and optional `aresetn`. AXI4-Lite channels are:

- AW: `awaddr`, `awprot`, `awvalid`, `awready`.
- W: `wdata`, `wstrb`, `wvalid`, `wready`.
- B: `bresp`, `bvalid`, `bready`.
- AR: `araddr`, `arprot`, `arvalid`, `arready`.
- R: `rdata`, `rresp`, `rvalid`, `rready`.

These signal lists were verified against Arm IHI 0022H.c. Relevant citations are: section A1.2 page A1-27 for the five AXI channels; section A1.2.1 page A1-28 for each channel having VALID and READY plus information signals; section A3.2.1 page A3-41 for transfer occurring only when VALID and READY are HIGH; Table A3-1 page A3-42 for `AWVALID/AWREADY`, `WVALID/WREADY`, `BVALID/BREADY`, `ARVALID/ARREADY`, and `RVALID/RREADY`; Tables A2-2 through A2-6 pages A2-33 through A2-37 for AXI3 and AXI4 signal membership, including AXI3-only `WID`; and Table B1-1 page B1-120 for AXI4-Lite signal membership.

## Open Questions

No open design questions remain for the initial implementation. Resolved answers are:

- `src/engine/extract.rs` exposes a crate-local run-plan API reused by `src/engine/axi.rs`; AXI owns only profile, mapping, context, and row-conversion logic.
- Output and stream schemas support `extract axi` with new DTOs. The input schema root accepts both `extract.generic.sources` and `extract.axi.source` documents.
- A new warning code, `WPK-W0004`, covers unmatched AXI include candidates. Fatal mapping validation remains process-level argument errors.
- CLI help and docs expectations are captured in `tests/cli_contract.rs`, `docs/public/commands/extract.md`, `docs/public/reference/machine-output.md`, and `docs/skills/wavepeek.md`.

## Plan of Work

The work will proceed in small, verifiable milestones.

Milestone 1 is research and plan hardening. Incorporate subagent results into this file. Read the exact generic extraction entrypoints and tests they identify. Verify the AXI signal lists against Arm IHI 0022H.c and cite the relevant section, table, or page identifiers in this plan. Acceptance for this milestone is that this file names the concrete code touchpoints and no longer has unresolved questions about where the feature belongs.

Milestone 2 is the minimal AXI profile and mapping core. Add profile definitions for `axi3`, `axi4`, and `axi4-lite`; parse profile names case-insensitively; normalize standard signal names to lowercase; define per-channel ready/valid signal names and payload signal order; implement token-based auto-mapping and explicit mapping overrides; and validate that `aclk` exists, `aresetn` is optional, at least one complete ready/valid pair exists, partial ready/valid pairs are rejected, and payload signals are only allowed on channels with complete handshakes. Acceptance is focused unit tests for profile parsing, token matching, collision avoidance such as `awvalid` versus `wvalid`, ambiguity rejection, explicit override behavior, and validation failures.

Milestone 3 is CLI and runtime integration. Add the `axi` subcommand under `extract`, load CLI options or source JSON, build the AXI extraction plan, and execute it using the same event, sample-time, predicate, and payload semantics as `extract generic`. Output rows must be ordered by event time and profile channel order. Acceptance is at least one CLI integration test against a small hand-written VCD fixture that prints AW/W/B/AR/R rows for AXI4-Lite.

Milestone 4 is machine output and source JSON. Add JSON envelope and JSONL stream support with AXI metadata, mappings, and transfer rows. Add source JSON handling for `kind: "extract.axi.source"`, optional `profile`, `name`, `includes`, and `maps`, and enforce conflicts between `--source` and CLI mapping inputs: `--profile`, `--name`, `--map`, and `--include`. Acceptance is integration coverage for JSON and JSONL records plus source-file mode.

Milestone 5 is public documentation, schema, and release notes. Update `docs/public/commands/extract.md`, `docs/public/workflows/extract-handshake.md` if useful, command model or machine-output references as needed, CLI help contract tests, schema snapshots, and `CHANGELOG.md`. Update `docs/dev/architecture.md` only if module boundaries change. Acceptance is docs/catalog tests and schema checks passing.

Milestone 6 is review, cleanup, gates, commits, push, and draft PR. Run focused subagent reviews after milestones with meaningful code changes, apply findings, run targeted tests while iterating, then run `just check`. If feasible, run `just ci`. Commit coherent slices, push the branch, and open a draft PR against the repository default branch with a concise summary and test evidence.

## Concrete Steps

All commands in this plan run from `/workspaces/wavepeek/.worktrees/feat-extract-axi` unless explicitly stated otherwise.

Research commands completed:

    gh issue view 49 --repo kleverhq/wavepeek --json number,title,body,labels,state,url
    find . -name AGENTS.md -print

Implementation and focused validation commands completed so far:

    cargo fmt
    cargo check -q
    cargo test -q token_matching_avoids_channel_prefix_collisions
    cargo test -q --test extract_axi_cli
    just update-schema
    cargo test -q --test schema_cli
    cargo test -q --test docs_cli
    cargo test -q --test skill_cli
    cargo test -q --test cli_contract extract_axi_help_is_self_descriptive
    cargo test -q --test cli_contract extract_command_without_subcommand_prints_help
    cargo test -q --test cli_contract
    just check-schema
    just check

Review lanes completed:

    8ad9e783-0ccd-4d2: code review lane
    de270952-bf4a-4fd: contracts/schema/tests review lane
    9eca7249-c438-450: docs/help review lane
    cc754fdb-6b3e-437: final control review lane

Final validation commands completed:

    just ci
    just test-aux
    just check

If the environment and time allow the full test gate, run:

    just ci

## Validation and Acceptance

The feature is accepted when these observable behaviors hold:

1. `wavepeek extract axi --help` documents the AXI command, the default `--profile axi4`, source mode, mapping mode, and output mode.
2. Running `wavepeek extract axi --waves <fixture> --profile axi4-lite --map aclk=<clock> --include <regex>` prints profile metadata, mappings, and transfer rows for all mapped complete ready/valid channels.
3. Running the same command without `--profile` behaves as `--profile axi4`.
4. `--json` returns a success envelope whose `data` object has `name`, `profile`, `issue`, `mappings`, and `transfers` fields, omitting unmapped signals and using an empty payload object for handshake-only rows.
5. `--jsonl` emits a `begin` record with AXI context and mappings, one `item` record per transfer, and an `end` summary record.
6. Source JSON with `kind: "extract.axi.source"` can provide `profile`, `name`, `includes`, and `maps`; CLI mapping options conflict with `--source` as specified.
7. Auto-mapping matches accepted token forms such as `awvalid`, `aw_valid`, `axi_awvalid_o`, and `axi_aw_valid_o`, while preventing prefix collisions such as `awvalid` matching `wvalid`.
8. Invalid mappings fail with clear user-facing errors: missing `aclk`, no complete ready/valid pair, partial ready/valid pairs, payload on a channel without a complete pair, and ambiguous auto-matches.
9. If `aresetn` is mapped, transfers during reset are omitted because the reset signal is included in the channel predicate at the sample time.
10. Rows are ordered by event time, then channel order `aw`, `w`, `b`, `ar`, `r`.
11. `just check` passes before final handoff, or any environment failure is recorded exactly.

## Idempotence and Recovery

The implementation steps are additive and can be repeated. Running tests and schema checks is safe. Generated schema snapshots should only be committed when the schema change is intentional. Use repository-root `tmp/` for disposable scratch outputs and do not delete arbitrary existing files there.

If a code change goes in the wrong direction, use `git diff` to isolate the change and revert only the affected hunks. Do not clean the worktree globally because other agents or the user may own unrelated files. If a subagent review suggests a large redesign, first update the Decision Log here with the reason before changing direction.

## Artifacts and Notes

Issue 49 URL:

    https://github.com/kleverhq/wavepeek/issues/49

Current branch and worktree at plan creation:

    /workspaces/wavepeek/.worktrees/feat-extract-axi
    branch: feat/extract-axi

Initial background research lanes:

    eee21b4d-50f4-459: explore generic extract implementation
    dd5223f9-6a8f-465: verify AXI profiles against Arm IHI 0022H.c
    e5826bc6-d309-4fc: find integration and contract test patterns

## Interfaces and Dependencies

The implementation must use existing repository dependencies where possible. The `regex` crate is already available and should be used for `--include` matching. `serde` and `serde_json` are already available and should be used for source JSON and machine-output DTOs. No new dependency should be added unless all existing code and standard library options are insufficient.

At the end of the implementation, the codebase should have:

- A CLI subcommand representing `wavepeek extract axi` under `src/cli/extract.rs` or a closely related module.
- An engine entrypoint for AXI extraction under `src/engine/`, preferably reusing `src/engine/extract.rs` for low-level runtime behavior.
- Profile definitions and mapping helpers for `axi3`, `axi4`, and `axi4-lite` in the smallest module that keeps `src/engine/extract.rs` readable.
- Serializable DTOs for AXI JSON and JSONL output in the existing contract/output layer.
- Tests under `tests/` and, where practical, unit tests near profile/mapping helpers.
- Public documentation under `docs/public/` and a changelog entry under `CHANGELOG.md`.

## Revision Notes

- 2026-07-04: Initial plan created from issue 49, repository guidance, and user constraints before implementation. The plan deliberately leaves source-code touchpoints and AXI PDF citations to be filled after the already-started research subagents complete.
- 2026-07-04: Updated after implementing the main AXI feature slice, regenerating schemas, adding docs/tests, and launching focused subagent reviews. The update records source citations, resolved design questions, current test evidence, and remaining review/gate work.
- 2026-07-04: Updated after final review fixes, schema family version bump to 2.2, and successful `just check`/`just ci`. The plan now records completed validation and remaining repository publication work.
- 2026-07-04: Updated after post-commit control review fixed docs deploy schema validation for the v2.2 stream and input contracts. The plan now records the extra `just test-aux` and repeat `just check` evidence.
