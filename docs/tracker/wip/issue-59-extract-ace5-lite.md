# Add ACE5-Lite family ready/valid extraction profiles

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. Implementation follows test-driven development: every new behavior is first expressed by a focused failing test, then implemented minimally until the test passes, and finally refactored while tests remain green. Each milestone ends with independent read-only review before work proceeds.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract axi` with the Arm AMBA 5 profiles `ace5-lite`, `ace5-lite-dvm`, or `ace5-lite-acp` against VCD, FST, or supported FSDB waveforms. The command emits deterministic rows for mapped ready/valid channel transfers, reports Arm specification issue `L`, and accepts only the functional signal inventory legal for the selected profile.

The existing extraction semantics remain unchanged. A transfer occurs on a rising `aclk` edge when a channel's `valid` and `ready` signals are high at the pre-edge sample point. Optional `aresetn` mapping suppresses transfers while reset is active. The behavior is observable through source-backed waveform fixtures, human output, schema-validating JSON and JSONL output, generated schemas, exact CLI help, embedded public documentation, and the packaged wavepeek skill.

## Non-Goals

This work does not implement credited transport. It excludes `*pending`, `*crdt`, `*crdtsh`, `*rp`, and `*sharedcrd` signals. It does not add interface-level clock/wakeup, QoS-accept, coherency-connection, broadcast, activation, parity, check, or protection-check signals beyond the existing common `aclk` and optional `aresetn` mappings. It does not decode DVM messages, bursts, cache maintenance, atomics, tags, or outstanding dependencies. It does not change the default `axi4` profile, alter output row semantics, add a `cd` channel, or validate profile property combinations. It does not accept arbitrary combinations of hyphen and underscore separators as aliases.

## Progress

- [x] (2026-07-11) Read issue #59, issues #57 and #58, merged PRs #61 and #62, current profile/runtime/schema/docs code, repository workflow, and Arm IHI 0022L sections B2.1-B2.4.
- [x] (2026-07-11) Fast-forward `feat/extract_ace5_lite` from AXI5 feature commit `d362626` to merged `origin/main` commit `903c4ab`; the worktree was clean before creating this plan.
- [x] (2026-07-11 20:31Z) Milestone 0: reviewed the ExecPlan through architecture and protocol-data lanes, incorporated all findings, and prepared the reviewed plan commit.
- [x] (2026-07-11 20:52Z) Milestone 1 RED: generated and staged three source-backed VCD/FST fixtures; added failing profile, alias, runtime, exact schema, help, docs, skill, and deployment-check tests; captured intended failures under `tmp/issue-59-red/`; resolved all findings from protocol, runtime/fixture, and contract-test review lanes.
- [x] (2026-07-11 21:14Z) Milestone 2 GREEN: implemented exact profiles and explicit aliases; regenerated output, stream, and input schemas; updated exact deployment checks, help, docs, skill, and changelog; passed focused suites, `just format`, `just lint`, `just check-schema`, and `just test`; resolved all findings from protocol/Rust, runtime/fixture, schema/checker, and docs/help review lanes.
- [ ] Milestone 3: run full gates, conduct parallel final reviews and a fresh control review, remove this WIP plan, rerun full gates on final `HEAD`, push the branch, and open a PR closing issue #59.

## Surprises & Discoveries

- Observation: Issue #59 refers to extending `standard_parts()`, but that helper no longer exists.
  Evidence: `src/engine/axi.rs` now uses token-based `standard_suffix_start()` and already maps split AC/CR names for AXI5. The new DVM profile can reuse this code without a profile-specific parser.

- Observation: `profile_specs()` drives most exact output and stream schema branches.
  Evidence: `src/contract/axi_schema.rs` derives profile mappings, issue constants, channel branches, and payload properties from runtime profile metadata. The RED schema tests reject all three absent profile branches in output and stream contracts.

- Observation: The complete RED suite fails at the intended product boundaries after fixture generation succeeds.
  Evidence: `tmp/issue-59-red/1.log` through `8.log` show unsupported runtime profiles, absent schema branches/enums, and stale help/docs/skill/deployment contracts; no failure is caused by missing or malformed fixture output.

- Observation: The generic token auto-mapper and extraction engine required no profile-specific runtime branch.
  Evidence: Adding exact `AxiProfileSpec` metadata plus CLI/source parsing made all seven new VCD/FST, explicit-map, auto-map, alias, partial-profile, JSON, and JSONL integration tests pass.

- Observation: Representative deployment checks were weaker than local generated-schema tests.
  Evidence: GREEN schema review showed that a corrupted remote schema could lose legal mappings, add forbidden payloads, or drop input `allOf` isolation while local generation stayed correct. The deployment checker now validates exact new-family mapping and per-channel payload sets for output, stream, and input artifacts.

- Observation: Arm IHI 0022L revised ACE5-LiteDVM relative to Issue H.
  Evidence: section B2.1.3 removes `ACSNOOP`, `ACPROT`, and `CRRESP`; Table B2.4 requires `DVM_Message_Support = Receiver`, making AC and CR channels mandatory at protocol level. Table B2.2 still permits `ARTAGOP` for ACE5-LiteDVM. The extractor follows existing partial-mapping behavior and emits only channels with complete mapped ready/valid pairs.

## Decision Log

- Decision: Use only Arm IHI 0022L Issue L for all three new profiles.
  Rationale: Issue #59 targets current ACE5-Lite interface classes, and Issue L sections B2.1-B2.4 define their signal and property constraints.
  Date/Author: 2026-07-11 / coding agent

- Decision: Preserve `axi4` as the default and place the new canonical names after `ace5` in deterministic profile ordering.
  Rationale: Existing commands must not change meaning, and the new entries extend the existing AXI/ACE family ordering without reordering shipped values.
  Date/Author: 2026-07-11 / coding agent

- Decision: Accept only the explicit case-insensitive spellings requested in issue #59 and confirmed by the user.
  Rationale: `ace5-lite` accepts `ace5_lite`; DVM accepts `ace5-lite-dvm`, `ace5-litedvm`, `ace5_litedvm`, and `ace5_lite_dvm`; ACP accepts `ace5-lite-acp`, `ace5-liteacp`, `ace5_liteacp`, and `ace5_lite_acp`. Other mixed separator combinations remain unsupported. JSON Schema publishes canonical kebab-case values only.
  Date/Author: 2026-07-11 / coding agent

- Decision: Treat each profile inventory as a legal functional ready/valid superset whose payload mappings are optional.
  Rationale: Signal presence can depend on interface properties and Manager/Subordinate direction. The extractor inspects incomplete waveform dumps and does not validate interface configuration; existing behavior accepts legal names and emits only completely mapped channels.
  Date/Author: 2026-07-11 / coding agent

- Decision: Keep schema version 2.2.
  Rationale: Issue #59 is in milestone v2.2 and extends the current unreleased contract rather than migrating a published schema generation.
  Date/Author: 2026-07-11 / coding agent

## Outcomes & Retrospective

Milestones 0-2 completed. Users can select all three Issue L ACE5-Lite profiles through canonical CLI/source names and explicit aliases, extract deterministic ready/valid rows from VCD/FST and available FSDB inputs, and validate JSON, JSONL, or source documents against exact profile-specific schemas. Reviewed tests cover exact signal inventories, legal and forbidden mappings, DVM AC/CR without CD, profile-prefix isolation, aliases, partial profile extraction, generated contracts, deployment drift, help, docs, skill guidance, and changelog. Full handoff gates and final control review remain.

## Context and Orientation

`wavepeek` is a Rust command-line waveform inspector. Commands in this plan run from `/workspaces/wavepeek/.worktrees/feat-extract_ace5_lite` inside the project devcontainer. Repository tasks are exposed through the root `justfile`; `just check` is the local handoff gate and `just ci` is the full CI gate.

`src/engine/axi.rs` owns `AxiProfileSpec`, `AxiChannelSpec`, the ordered profile inventories, runtime profile parsing, explicit mapping validation, auto-mapping, and conversion of generic ready/valid events into AXI transfer rows. `src/cli/extract.rs` exposes canonical profile values and aliases through clap's `ValueEnum`. `src/contract/input.rs` owns the baseline source-document profile enum. `src/contract/axi_schema.rs` converts runtime profile metadata into exact generated input, output, and stream contract branches. `src/contract/output.rs` contains user-facing schema descriptions.

Source-backed Verilog fixtures live in `tests/fixtures/source/`. Their ignored VCD/FST outputs are generated under `tests/fixtures/generated/` by `just prepare-waveform-fixtures`; every source and output is registered in `tests/fixtures/waveform_policy.json`. `tests/extract_axi_cli.rs` covers runtime CLI behavior. `tests/schema_cli.rs` covers exact generated contracts. `tests/cli_contract.rs`, `tests/docs_cli.rs`, and `tests/skill_cli.rs` protect help, embedded docs, and packaged skill wording. Generated schemas under `schema/` are produced by `just update-schema`, never edited manually. `tools/schema/check_schema_contract.py` and `tools/docs/check_deploy.py` contain exact deployment assertions and have Python unit tests beside them.

The exact Issue L signal inventory is part of the implementation contract. Every profile also accepts common `aclk` and optional `aresetn`; every listed payload mapping is optional.

`ace5-lite` has channels `aw`, `w`, `b`, `ar`, and `r`. AW accepts `awid`, `awaddr`, `awregion`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awnse`, `awqos`, `awuser`, `awdomain`, `awsnoop`, `awstashnid`, `awstashniden`, `awstashlpid`, `awstashlpiden`, `awtrace`, `awloop`, `awmmuvalid`, `awmmusecsid`, `awmmusid`, `awmmussidv`, `awmmussid`, `awmmuatst`, `awmmuflow`, `awpbha`, `awmecid`, `awnsaid`, `awsubsysid`, `awatop`, `awmpam`, `awidunq`, `awcmo`, `awtagop`, `awvalid`, and `awready`. W accepts `wdata`, `wstrb`, `wtag`, `wtagupdate`, `wlast`, `wuser`, `wpoison`, `wtrace`, `wvalid`, and `wready`. B accepts `bid`, `bidunq`, `bresp`, `bcomp`, `bpersist`, `btagmatch`, `buser`, `btrace`, `bloop`, `bbusy`, `bvalid`, and `bready`. AR accepts `arid`, `araddr`, `arregion`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arnse`, `arqos`, `aruser`, `ardomain`, `arsnoop`, `artrace`, `arloop`, `armmuvalid`, `armmusecsid`, `armmusid`, `armmussidv`, `armmussid`, `armmuatst`, `armmuflow`, `arpbha`, `armecid`, `arnsaid`, `arsubsysid`, `armpam`, `archunken`, `aridunq`, `artagop`, `arvalid`, and `arready`. R accepts `rid`, `ridunq`, `rdata`, `rtag`, `rresp`, `rlast`, `ruser`, `rpoison`, `rtrace`, `rloop`, `rchunkv`, `rchunknum`, `rchunkstrb`, `rbusy`, `rvalid`, and `rready`.

`ace5-lite-dvm` uses the same five channels with the same functional inventory except that all `AWMMU*` and `ARMMU*` signals and `BTAGMATCH` are absent. `ARTAGOP` remains legal on AR. It adds AC with `acaddr`, `acvmidext`, `actrace`, `acvalid`, and `acready`, and CR with `crtrace`, `crvalid`, and `crready`. It does not accept `acsnoop`, `acprot`, `crresp`, or any `cd*` signal.

`ace5-lite-acp` has only `aw`, `w`, `b`, `ar`, and `r`. AW accepts `awid`, `awaddr`, `awlen`, `awcache`, `awprot`, `awuser`, `awdomain`, `awsnoop`, `awstashnid`, `awstashniden`, `awstashlpid`, `awstashlpiden`, `awtrace`, `awmpam`, `awidunq`, `awvalid`, and `awready`. W accepts `wdata`, `wstrb`, `wlast`, `wuser`, `wpoison`, `wtrace`, `wvalid`, and `wready`. B accepts `bid`, `bidunq`, `bresp`, `buser`, `btrace`, `bvalid`, and `bready`. AR accepts `arid`, `araddr`, `arlen`, `arcache`, `arprot`, `aruser`, `ardomain`, `arsnoop`, `artrace`, `armpam`, `archunken`, `aridunq`, `arvalid`, and `arready`. R accepts `rid`, `ridunq`, `rdata`, `rresp`, `rlast`, `ruser`, `rpoison`, `rtrace`, `rchunkv`, `rchunknum`, `rchunkstrb`, `rvalid`, and `rready`.

Protocol authority is Arm IHI 0022L Issue L. Interface summaries are in sections B2.1.2-B2.1.4, signal presence is in section B2.2 Table B2.2, parity/check exclusions are in section B2.3 Table B2.3, and property constraints are in section B2.4 Table B2.4. Issue L section B2.1.3 explicitly removes `ACSNOOP`, `ACPROT`, and `CRRESP` from ACE5-LiteDVM.

## Open Questions

There are no blocking questions. Alias scope, branch synchronization, protocol authority, profile ordering, and ready/valid-only boundaries are resolved in the Decision Log.

## Plan of Work

### Milestone 0: Review and checkpoint the plan

Request a read-only architecture/plan review and a separate protocol-data review of this file against issue #59 and Arm IHI 0022L. Incorporate substantive findings, update the living sections and revision notes, then run `git diff --check` and commit the plan as `docs(plan): add ACE5-Lite extraction plan`. This commit is the restartable design checkpoint.

### Milestone 1: Specify the complete behavior in failing tests

Create minimal deterministic source fixtures for the three profiles under `tests/fixtures/source/`, register generated VCD and FST outputs in `tests/fixtures/waveform_policy.json`, and generate them before adding runtime tests. Fixtures must exercise every channel, representative profile-specific payloads including DVM `artagop`, split names such as `ac_addr`, and decoy signals excluded from the profile. ACE5-LiteDVM must expose AC/CR handshakes and CD decoys. ACE5-LiteACP must expose its reduced inventory. Generate both VCD and FST for each profile. Stage only the three new Verilog source files before running fixture-policy or full Rust tests because `tests/waveform_fixture_policy.rs` intentionally discovers fixture sources through `git ls-files`; generated VCD/FST files remain ignored.

Add focused unit tests in `src/engine/axi.rs` for canonical ordering, Issue L metadata, exact channel inventories, representative accepted and rejected signals, case-insensitive parsing, every explicit alias, `ARTAGOP` acceptance for DVM, and rejection of unlisted mixed separator forms. Add integration tests in `tests/extract_axi_cli.rs` for explicit maps, auto-mapping, source aliases, canonical output names, Issue L metadata, channel ordering, partial-channel behavior, representative exclusions, VCD/FST parity, and human/JSON/JSONL output. Add failing CLI assertions in `tests/cli_contract.rs`, exact synthetic contract tests in `tests/schema_cli.rs`, checker expectations and tests, docs/skill assertions, and source-schema alias rejection.

Run focused tests and record failures caused by unsupported profile names, absent runtime profile metadata, and missing exact schema/help/docs contracts. Fixture generation itself must succeed; missing-file and malformed-fixture failures do not count as RED evidence. Request parallel read-only protocol/test-inventory, runtime-fixture, and schema/docs-test reviews. Fix test-specification findings before implementation. This milestone is intentionally uncommitted because hooks correctly reject a RED tree; the reviewed plan commit is the recovery point, and Milestone 2 immediately turns this specification green.

### Milestone 2: Implement and publish the coherent feature

Add `Ace5Lite`, `Ace5LiteDvm`, and `Ace5LiteAcp` variants and explicit clap aliases in `src/cli/extract.rs`. Define shared or profile-specific channel constants in `src/engine/axi.rs` without weakening exact inventories, add profiles to `profile_specs()`, extend runtime parsing and diagnostics, and preserve `DEFAULT_PROFILE`. Implement only enough runtime behavior to satisfy the reviewed tests; reuse the existing token-based auto-mapper and generic ready/valid extraction engine.

Update `src/contract/input.rs`, descriptions in `src/contract/output.rs` and `src/contract/axi_schema.rs`, exact checks in `tools/schema/check_schema_contract.py` and `tools/docs/check_deploy.py`, and corresponding Python tests. Regenerate `schema/input.json`, `schema/output.json`, and `schema/stream.json` with `just update-schema`; `schema/catalog.json` must remain unchanged.

Update long help in `src/cli/extract.rs`, `docs/public/commands/extract.md`, `docs/public/commands/overview.md`, `docs/public/workflows/extract-handshake.md`, `docs/public/reference/machine-output.md`, `docs/skills/wavepeek.md`, and the `CHANGELOG.md` Unreleased entry. State that all three profiles use Issue L and ready/valid transport; mention DVM AC/CR support and canonical names without duplicating generated flag tables.

Run all focused unit, integration, CLI, schema, docs, skill, and Python checker tests green, then `just format`, `just lint`, `just check-schema`, and `just test`. Request parallel read-only protocol/Rust, runtime/fixture, schema/checker, and docs/help reviews. Fix findings through regression tests, update this plan, and commit the coherent implementation, fixtures, tests, generated contracts, checks, help, docs, skill, changelog, and current plan as `feat(axi): add ACE5-Lite extraction profiles`. Hooks must run normally. This milestone leaves every committed test and schema green and provides the full observable behavior.

### Milestone 3: Full validation, final review, and delivery

Run `just check` and `just ci`, saving logs under repository-root `tmp/`. Run parallel read-only final reviews for runtime/protocol correctness, schema/test consistency, and docs/help/changelog consistency over `origin/main...HEAD`. Resolve findings with regression tests and normal hooked commits, then rerun both full gates. Request a fresh independent control review of the consolidated diff; repeat the fix-and-gate loop for any substantive finding, with at most two control passes unless a critical issue requires more.

When clean, complete `Outcomes & Retrospective`, remove this WIP plan as required by local policy, and commit cleanup as `chore(plan): remove completed ACE5-Lite plan`. Rerun `just check` and `just ci` on that final cleanup commit so the pushed `HEAD` is exactly the validated revision. Create and inspect `tmp/issue-59-pr.md` using the commands below. Push `feat/extract_ace5_lite` to `origin` without force, create a PR against `main` titled `feat(axi): add ACE5-Lite extraction profiles`, and verify the resulting PR URL and state.

### Concrete Steps

All commands run from `/workspaces/wavepeek/.worktrees/feat-extract_ace5_lite`.

Inspect and maintain state:

    git status --short --branch
    git diff --check

Run focused RED/GREEN tests:

    cargo test --lib engine::axi::tests
    cargo test --test extract_axi_cli
    cargo test --test schema_cli
    cargo test --test cli_contract
    cargo test --test docs_cli
    cargo test --test skill_cli

Generate and validate fixtures and schemas:

    just prepare-waveform-fixtures
    just update-schema
    just check-schema
    python3 -B -m unittest tools.schema.test_check_schema_contract
    python3 -B -m unittest tools.docs.test_check_deploy

Run milestone and final gates:

    just format
    just lint
    just test
    just check
    just ci

Every RED test must fail for the intended missing profile, alias, extraction, or schema behavior rather than syntax, fixture, or environment errors. Every GREEN command must exit zero. Before each commit, inspect owned changes, run `git diff --check`, stage explicit paths, inspect `git diff --cached --check` and `git diff --cached --stat`, and commit without bypassing hooks.

Delivery commands are:

    mkdir -p tmp
    printf '%s\n' \
      '## Summary' \
      '' \
      '- add ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid extraction profiles from Arm IHI 0022L' \
      '- support exact profile-specific mappings, DVM AC/CR channels, and explicit aliases' \
      '- publish generated machine contracts, extraction fixtures, help, docs, and packaged skill guidance' \
      '' \
      '## Validation' \
      '' \
      '- `just check`' \
      '- `just ci`' \
      '' \
      'Closes #59' > tmp/issue-59-pr.md
    python3 -B -c 'from pathlib import Path; print(Path("tmp/issue-59-pr.md").read_text())'
    git push -u origin feat/extract_ace5_lite
    gh pr create -R kleverhq/wavepeek --base main --head feat/extract_ace5_lite \
      --title 'feat(axi): add ACE5-Lite extraction profiles' \
      --body-file tmp/issue-59-pr.md
    gh pr view -R kleverhq/wavepeek --json number,state,title,url,headRefName,baseRefName

If push is rejected, fetch and inspect divergence; never force-push over unknown remote work. If a PR already exists, locate and update it rather than creating a duplicate.

### Validation and Acceptance

The feature is accepted when CLI help lists canonical `ace5-lite`, `ace5-lite-dvm`, and `ace5-lite-acp`; runtime/CLI parsing accepts exactly the explicit aliases in the Decision Log case-insensitively; output always normalizes to canonical names and reports issue `L`; and source JSON Schema accepts only canonical values.

ACE5-Lite must emit only `aw`, `w`, `b`, `ar`, and `r`; ACE5-LiteDVM must additionally emit `ac` and `cr` while rejecting `acsnoop`, `acprot`, `crresp`, and `cd*`; ACE5-LiteACP must expose only its reduced five-channel inventory. All profiles must reject credited transport, parity/check, and interface-level signals. Partial mappings must continue to emit only complete ready/valid channels. Auto-mapping must recognize compact and split legal names without matching excluded decoys.

Human, JSON, and JSONL outputs must carry canonical names and Issue L metadata and validate against current schemas. Existing AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, and ACE5 tests must remain green. `just check` and `just ci` must both succeed on final `HEAD`.

### Idempotence and Recovery

Fixture and schema generation are deterministic and safe to repeat. Generated VCD/FST files remain ignored; only source fixtures and policy entries are committed. Schema JSON is regenerated from Rust contracts and never hand-edited. Interrupted generation is recovered by rerunning the command. Reviewed commits provide recovery points. Disposable logs and scratch files stay under `tmp/`; unrelated existing files there are not deleted.

### Artifacts and Notes

Initial state:

    branch: feat/extract_ace5_lite
    base commit: 903c4ab
    base relation: identical to origin/main
    working tree before plan: clean

Issue authority is GitHub issue #59. Protocol authority is Arm IHI 0022L Issue L. Existing older profiles retain their current Issue H.c or Issue L metadata unchanged.

### Interfaces and Dependencies

No new dependency is required. `clap::ValueEnum` remains the CLI parser, the existing lowercase-and-explicit-match runtime parser remains the source parser, `schemars` remains the schema generator, and the generic extraction engine remains the runtime. At completion, `AxiProfileArg` and `profile_specs()` include the three canonical profile variants. `parse_profile()` recognizes only canonical values and the explicit aliases in the Decision Log. No waveform backend, extraction trait, or transport abstraction changes.

Plan revision note (2026-07-11): Created the initial self-contained plan after issue, repository, prior-feature, and specification investigation. It records exact profile inventories, explicit alias scope, ready/valid boundaries, TDD sequencing, per-milestone reviews, generated-contract ownership, full gates, and PR delivery.

Plan revision note (2026-07-11): Incorporated independent plan and protocol-data review. `ARTAGOP` remains legal for ACE5-LiteDVM; final gates rerun after WIP cleanup; and delivery deterministically creates the PR body.

Plan revision note (2026-07-11): Incorporated follow-up architecture review. Fixtures are generated before runtime tests, the complete reviewed RED specification fails on unsupported behavior rather than missing files, and one coherent GREEN milestone publishes schemas and turns all tests green before the feature commit.

Plan revision note (2026-07-11): Incorporated final feasibility review. New Verilog sources are staged before fixture-policy tests because that contract deliberately enumerates tracked fixture sources through `git ls-files`.

Plan revision note (2026-07-11): Milestone 1 RED generated all fixtures successfully and captured focused failures at each missing runtime and public-contract boundary before implementation.

Plan revision note (2026-07-11): RED reviews strengthened legal explicit-map evidence, profile-prefix collision safety, canonical alias output, exact output/stream schema branches, cross-profile isolation, all-profile stream records, deployment stale-artifact mutations, and documentation profile-list assertions. Follow-up passes reported no substantive findings.

Plan revision note (2026-07-11): Milestone 2 implementation turned all reviewed RED tests green, regenerated all three schema artifacts without changing the catalog, and passed format, lint, schema, full Rust/Python, and available FSDB tests before GREEN review.

Plan revision note (2026-07-11): GREEN reviews reported clean protocol/runtime lanes and identified deployment-check and schema-description gaps. Exact output/stream/input deployment validation, mutation tests, complete schema descriptions, and regenerated artifacts resolved those findings; follow-up reviews reported no substantive findings.
