# Add AXI5 and AXI5-Lite ready/valid extraction profiles

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. Implementation follows test-driven development: each behavior is first expressed by a failing test, then implemented minimally until the test passes, and finally refactored while tests remain green.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract axi --profile axi5` or `wavepeek extract axi --profile axi5-lite` against VCD, FST, or supported FSDB waveforms and receive one deterministic row for each mapped ready/valid channel transfer. The new profiles use Arm IHI 0022L Issue L signal definitions while the existing AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 profiles remain based on Issue H.c.

The extraction runtime remains unchanged. It selects rising `aclk` edges, samples predicates and payloads one dump tick before each edge, optionally gates transfers with active-low `aresetn`, and emits one row per complete ready/valid channel. AXI5 DVM message support adds optional `ac` and `cr` channels. The behavior is visible through source-backed waveform fixtures, human output, schema-validating JSON and JSONL output, and the generated input/output/stream schemas.

## Non-Goals

This work does not add credited AXI transport. Signals such as `*pending`, `*crdt`, `*crdtsh`, `*rp`, and `*sharedcrd` are not accepted by these ready/valid profiles. It does not extract transfer-level credit-control, interface gating, wakeup, QoS-accept, coherency-connection, broadcast, parity/check, or data-protection check signals. Functional request attributes such as `AxPROT`, `AxNSE`, `AxPAS`, `AxINST`, and `AxPRIV` remain valid payload signals. It does not decode or reconstruct bursts, atomics, cache-maintenance operations, memory tagging, Arm Compression Technology, DVM messages, untranslated transactions, protection domains, or outstanding dependencies. It does not add ACE5-Lite, ACE5-LiteDVM, or ACE5-LiteACP profiles, and it does not add a legacy ACE/ACE5 `cd` channel to AXI5.

## Progress

- [x] (2026-07-11 12:41Z) Read issue #58, current AXI/ACE extraction implementation, schema ownership, fixture policy, public contracts, repository workflow, and Arm IHI 0022L source sections.
- [x] (2026-07-11 12:41Z) Confirm the branch is clean at `961155e`, current `main`, after ACE profile support merged.
- [x] (2026-07-11 12:54Z) Review the initial ExecPlan through independent architecture, protocol-data, follow-up, and control passes; incorporate all findings.
- [x] (2026-07-11 13:03Z) Milestone 1 RED: add focused runtime, parsing, mapping, and source-backed fixture tests; observe unit/source failures for unsupported `axi5` and clap failure for the missing profile value.
- [x] (2026-07-11 13:16Z) Milestone 1 GREEN/review: add Issue L profile data and runtime-facing CLI support; pass 9 focused unit tests, 29 AXI CLI tests, VCD/FST parity, fixture generation/policy, formatting, and lint. Three independent read-only review lanes found fixture width, transfer-size, stalled-payload, timing, and direction issues; fix them and complete three clean follow-up passes.
- [x] (2026-07-11 13:27Z) Milestone 2 RED/GREEN/review: observe expected failures against stale output, stream, input, and deploy-checker profile contracts; generate exact Issue L profile branches and pass 31 AXI CLI tests, 30 schema CLI tests, 50 CLI contract tests, 32 auxiliary tests, and `just check-schema`. Independent contract/schema review was clean; test review found and resolved parent/row, cross-profile mapping, input, and direct stream isolation gaps, followed by a clean re-review.
- [x] (2026-07-11 13:33Z) Milestone 3 RED/GREEN/review: observe expected failures against stale help, embedded docs, and packaged skill; publish mixed H.c/L ownership, full profile/channel guidance, ready/valid-only boundaries, and issue #58 changelog linkage. Pass 50 CLI contract tests, 25 docs tests, 3 skill tests, and `just check-schema`; complete a clean independent read-only user-contract review.
- [x] (2026-07-11 13:34Z) Commit the coherent runtime, fixture, schema, test, help, docs, skill, changelog, and plan slice as `79bae7b` after all installed hooks pass.
- [ ] Milestone 4 (completed: `just check` and `just ci` pass at `79bae7b`; three parallel final review lanes completed; runtime/protocol and deploy-checker findings fixed through regression tests and clean follow-up reviews; remaining: hooked fix commit, repeat both full gates, control review, WIP cleanup, push, and PR).

## Surprises & Discoveries

- Observation: Issue #58 predates the merged ACE work and refers to a `standard_parts` helper that no longer exists.
  Evidence: `src/engine/axi.rs` now uses token-based `standard_suffix_start` matching, and existing ACE tests cover split `ac`, `cr`, and `cd` forms. AXI5 `ac` and `cr` mapping should reuse that implementation without a new prefix-specific path.

- Observation: Arm IHI 0022L Table B2.2 appears to attach `AXI_Transport == Credited` to `ACADDR`, which conflicts with the detailed DVM message definition.
  Evidence: Issue L Table A15.17 defines `ACADDR` with presence condition `DVM_Message_Support`, and Table B1.6 describes it as the DVM request payload. This plan follows the detailed definition and issue #58 by accepting `acaddr` for the ready/valid AXI5 profile.

- Observation: Profile specs already drive most generated schema detail.
  Evidence: `src/contract/axi_schema.rs` derives profile mappings, issue constants, channel branches, and payload properties from `profile_specs()`, `standard_signals()`, and `channel_payload_signals()`.

- Observation: The waveform fixture policy test discovers source files through the Git index rather than only the filesystem.
  Evidence: `cargo test --test waveform_fixture_policy` initially omitted the two untracked Verilog sources from its tracked-source set and passed after only those files were staged with `git add`. The files remain staged but uncommitted until the coherent feature commit.

- Observation: A single-edge all-channel fixture was too weak and initially encoded several illegal widths and stalled-payload changes.
  Evidence: Milestone 1 reviews identified 4-bit MECID, 4-bit BTAGMATCH, undersized ACADDR, illegal AXI5-Lite transfer sizes, and payload changes after VALID assertion. The revised fixtures use legal widths and values, stable payloads, distinct READY windows, ordered request/response transfers at 5ns intervals, exact timestamp assertions, and unit checks for every channel's configured valid/ready names.

- Observation: Output-envelope schemas can couple `data.profile` to each nested transfer profile, but JSONL stream schemas cannot couple independent records.
  Evidence: A proposed stream test that rejected a valid ACE5 item after an AXI5 begin failed because each JSONL line is validated independently. Parent/row coupling is tested in `schema_output_validator_enforces_axi5_profile_channels_and_payloads`; stream tests instead enforce each record's internal issue, mapping, channel, payload, and profile branch.

- Observation: Individually legal optional AXI5 payload signals can form an illegal interface/transaction combination in one fixture.
  Evidence: Final protocol review found that 4-bit WTAGUPDATE and 5-bit RCHUNKNUM implied inconsistent DATA_WIDTH values and that active tag-update, MMU, and ACT attributes conflicted. The fixture now represents a 1024-bit-data configuration with 8-bit WTAGUPDATE and keeps optional conflicting attributes inactive while proving their extraction.

- Observation: Deployment checks validated the new input profile enum but did not initially reject stale output or stream profile branches.
  Evidence: New auxiliary regressions showed pre-AXI5 profile enums, wrong Issue L constants, and incomplete transfer branches passed `validate_schema_json` and `validate_stream_schema_json`. The v2.2 deploy checker now validates exact AXI profile and representative mapping/channel/payload branches in all three schema artifacts.

## Decision Log

- Decision: Keep the default profile as `axi4`.
  Rationale: Issue #58 explicitly preserves the current default, and adding a profile must not silently reinterpret existing commands or source files.
  Date/Author: 2026-07-11 / coding agent

- Decision: Insert canonical profile ordering as `axi3`, `axi4`, `axi4-lite`, `axi5`, `axi5-lite`, `ace`, `ace-lite`, `ace5`.
  Rationale: This groups AXI interface classes before ACE interface classes while preserving revision order inside each family. The order is deterministic and is reflected consistently in help, diagnostics, and schemas.
  Date/Author: 2026-07-11 / coding agent

- Decision: Accept case-insensitive hyphen and underscore spellings at runtime, but publish only canonical kebab-case values in JSON Schema.
  Rationale: This preserves existing `axi4-lite` and `ace-lite` behavior. Machine contracts remain deterministic while CLI and source parsing remain convenient.
  Date/Author: 2026-07-11 / coding agent

- Decision: Treat AXI5 profiles as property-agnostic legal signal supersets for waveform extraction.
  Rationale: Many Issue L signals are conditional on interface properties. The extractor accepts optional mappings and does not claim to validate interface configuration, so all legal functional ready/valid payload signals can be mapped while unmapped signals remain absent.
  Date/Author: 2026-07-11 / coding agent

- Decision: Keep the current schema family at version 2.2.
  Rationale: Issue #58 belongs to the v2.2 milestone, current schema URLs already identify 2.2, and adding profile branches extends the unreleased contract rather than migrating a published historical artifact.
  Date/Author: 2026-07-11 / coding agent

## Outcomes & Retrospective

The initial plan review completed with no unresolved findings after two focused lanes, follow-up checks, and a fresh control pass. Milestone 1 delivered a focused, reviewed runtime implementation and protocol-realistic generated VCD/FST fixtures. Milestone 2 exposes reviewed exact input/output/stream contracts and checker expectations. Milestone 3 makes help, embedded docs, packaged skill, and changelog coherent and passed independent read-only review. The three implementation milestones were committed coherently as `79bae7b` after all installed hooks passed. This section will be updated after every major milestone and at completion.

## Context and Orientation

`wavepeek` is a Rust command-line waveform inspector. All commands in this plan run from `/workspaces/wavepeek/.worktrees/feat-extract-axi5` inside the project devcontainer. Repository tasks are exposed through the root `justfile`; `just check` is the local handoff gate and `just ci` is the full CI gate.

A ready/valid channel transfers on a rising clock edge when both its `valid` and `ready` signals are high at the associated pre-edge sample point. `src/engine/axi.rs` maps lowercase standard names such as `awvalid` to waveform paths, validates that a mapped channel has a complete ready/valid pair, creates one generic extraction source per complete channel, and converts generic rows into AXI transfer rows. `AxiProfileSpec` owns a profile name, Arm specification issue, ordered channels, and accepted standard signal names. `AxiChannelSpec` owns each channel's `valid`, `ready`, and payload inventory.

`src/cli/extract.rs` exposes profiles through clap's `ValueEnum`. `src/contract/input.rs` contains a baseline hard-coded profile enum. `src/contract/axi_schema.rs` replaces broad AXI definitions with exact branches derived from runtime profile specs. `src/contract/output.rs` contains human-readable schema descriptions. Canonical generated artifacts are `schema/input.json`, `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`; they are regenerated with `just update-schema` and never edited manually.

`tests/extract_axi_cli.rs` invokes the CLI and validates human, JSON, and JSONL behavior. Source-backed Verilog fixtures live under `tests/fixtures/source/`; `just prepare-waveform-fixtures` creates ignored VCD/FST files under `tests/fixtures/generated/`. Every generated fixture must be declared in `tests/fixtures/waveform_policy.json`. `tests/schema_cli.rs` validates exact machine-contract behavior. CLI, embedded-doc, and packaged-skill contracts live in `tests/cli_contract.rs`, `tests/docs_cli.rs`, and `tests/skill_cli.rs`.

The Issue L profile inventories are as follows. Every profile also accepts required `aclk` and optional `aresetn`; payload signals are optional and are omitted when unmapped.

For `axi5`, channel order is `aw`, `w`, `b`, `ar`, `r`, `ac`, `cr`. AW accepts `awid`, `awaddr`, `awregion`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awnse`, `awpas`, `awinst`, `awpriv`, `awqos`, `awuser`, `awdomain`, `awsnoop`, `awstashnid`, `awstashniden`, `awstashlpid`, `awstashlpiden`, `awtrace`, `awloop`, `awmmuvalid`, `awmmusecsid`, `awmmusid`, `awmmussidv`, `awmmussid`, `awmmuatst`, `awmmuflow`, `awmmupasunknown`, `awmmupm`, `awpbha`, `awmecid`, `awnsaid`, `awsubsysid`, `awatop`, `awmpam`, `awidunq`, `awcmo`, `awtagop`, `awact`, `awactv`, `awvalid`, and `awready`. W accepts `wdata`, `wstrb`, `wtag`, `wtagupdate`, `wlast`, `wuser`, `wpoison`, `wtrace`, `wvalid`, and `wready`. B accepts `bid`, `bidunq`, `bresp`, `bcomp`, `bpersist`, `btagmatch`, `buser`, `btrace`, `bloop`, `bbusy`, `bvalid`, and `bready`. AR accepts `arid`, `araddr`, `arregion`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arnse`, `arpas`, `arinst`, `arpriv`, `arqos`, `aruser`, `ardomain`, `arsnoop`, `artrace`, `arloop`, `armmuvalid`, `armmusecsid`, `armmusid`, `armmussidv`, `armmussid`, `armmuatst`, `armmuflow`, `armmupasunknown`, `armmupm`, `arpbha`, `armecid`, `arnsaid`, `arsubsysid`, `armpam`, `archunken`, `aridunq`, `artagop`, `aract`, `aractv`, `arvalid`, and `arready`. R accepts `rid`, `ridunq`, `rdata`, `rtag`, `rresp`, `rlast`, `ruser`, `rpoison`, `rtrace`, `rloop`, `rchunkv`, `rchunknum`, `rchunkstrb`, `rbusy`, `rvalid`, and `rready`. AC accepts `acaddr`, `acvmidext`, `actrace`, `acvalid`, and `acready`. CR accepts `crtrace`, `crvalid`, and `crready`.

For `axi5-lite`, channel order is `aw`, `w`, `b`, `ar`, `r`. AW accepts `awid`, `awaddr`, `awsize`, `awprot`, `awuser`, `awtrace`, `awsubsysid`, `awidunq`, `awvalid`, and `awready`. W accepts `wdata`, `wstrb`, `wuser`, `wpoison`, `wtrace`, `wvalid`, and `wready`. B accepts `bid`, `bidunq`, `bresp`, `buser`, `btrace`, `bvalid`, and `bready`. AR accepts `arid`, `araddr`, `arsize`, `arprot`, `aruser`, `artrace`, `arsubsysid`, `aridunq`, `arvalid`, and `arready`. R accepts `rid`, `ridunq`, `rdata`, `rresp`, `ruser`, `rpoison`, `rtrace`, `rvalid`, and `rready`.

Protocol authority is Arm IHI 0022L Issue L. Valid-ready transport is defined in §A2.3 and Table A2.2; credited transport is separated in §A2.4 and Tables A2.3/A2.9; signal inventories are in §B1.1–B1.3 and Tables B1.1–B1.7; AXI5 and AXI5-Lite interface classes are in §B2.1.1 and §B2.1.5; signal presence is in §B2.2 Table B2.2; parity/check signals excluded by this work are in §B2.3 Table B2.3; DVM request and response signaling is detailed in §A15.4.1 Tables A15.17 and A15.18.

## Open Questions

There are no blocking product or protocol questions. The implementation will follow the issue's exact profile inventories, preserve current runtime semantics, and use the decisions recorded above.

## Plan of Work

### Milestone 1: Prove and implement runtime profiles

Create deterministic Verilog sources `tests/fixtures/source/extract_axi5.v` and `tests/fixtures/source/extract_axi5_lite.v`, register their generated VCD and FST outputs in `tests/fixtures/waveform_policy.json`, and generate them with `just prepare-waveform-fixtures`. The AXI5 fixture must produce transfers in all seven channels and expose representative optional payloads including `awmmuvalid`, `awmecid`, `awactv`, `wtagupdate`, `btagmatch`, `armecid`, `archunken`, `rchunknum`, `acaddr`, `acvmidext`, and `crtrace`. It must use split waveform names such as `axi_aw_mmuvalid_o`, `axi_aw_actv_o`, `axi_ar_mecid_o`, `axi_r_chunknum_i`, `axi_ac_addr_i`, `axi_ac_valid_i`, and `axi_cr_ready_o`. It should include credited-transport, parity/check, `cd`, wakeup, QoS-accept, coherency-connection, broadcast, and credit-control decoys that auto-mapping must ignore. The AXI5-Lite fixture must produce all five base-channel transfers and demonstrate single-transfer payloads without `wlast` or `rlast`. Add VCD/FST parity assertions for both profiles. No new AXI5 FSDB artifact is required because the AXI adapter is backend-neutral and existing conditional FSDB suites exercise the shared generic extraction runtime; `just ci` remains the FSDB regression gate when the SDK is available.

Before implementing profiles, add unit tests in `src/engine/axi.rs` and integration tests in `tests/extract_axi_cli.rs`. Cover canonical and case-insensitive underscore parsing, exact channel order, representative accepted signals, representative profile-specific rejections, explicit maps, auto-mapping, source-file aliases, Issue L human output, DVM `ac`/`cr` rows, AXI5-Lite's five channels, and credited/check/CD exclusions. Confirm focused tests fail because `axi5` and `axi5-lite` are unsupported.

Then add `Axi5` and `Axi5Lite` variants to `AxiProfileArg` in `src/cli/extract.rs`; define the exact profile constants and channel tables in `src/engine/axi.rs`; expose them through `profile_specs()` with `issue: "L"`; extend `parse_profile()` and unsupported-profile diagnostics; and preserve `DEFAULT_PROFILE`. Reuse the existing token matcher rather than adding AXI5-specific parsing. Run focused runtime tests, formatting, lint, and fixture-policy tests until green. Request independent read-only runtime, protocol-data, and test-fixture reviews; reviewers must not edit the working tree. Fix findings through regression tests. Do not commit this slice: `profile_specs()` intentionally makes generated schemas stale and the clap variants intentionally make exact help assertions stale. Record that state in this plan and proceed directly to Milestone 2; mandatory hooks must not be bypassed.

### Milestone 2: Prove exact machine contracts

Add failing JSON and JSONL integration assertions in `tests/extract_axi_cli.rs` and exact schema tests in `tests/schema_cli.rs`. Require canonical profile enum values, `issue: "L"`, profile-specific mapping keys, exact channel branches, representative payload keys, and rejection of credited transport, check/parity, `cd`, and cross-profile signals. Validate canonical source documents for both new profiles while continuing to reject noncanonical underscore spellings in JSON Schema.

Update the baseline enum in `src/contract/input.rs`, profile/channel descriptions in `src/contract/output.rs`, the profile description in `src/contract/axi_schema.rs`, exact checker expectations in `tools/schema/check_schema_contract.py` and `tools/docs/check_deploy.py`, corresponding Python tests, and the exact clap possible-values assertion in `tests/cli_contract.rs`. Run `just update-schema`, inspect all generated diffs, and run `just check-schema`; `schema/catalog.json` should remain unchanged. Run focused Rust and auxiliary tests until green. Request independent read-only contract/schema and test reviews; reviewers must not edit the working tree. Fix findings through tests, and update this plan. Do not commit yet because narrative help, public docs, the packaged skill, and the changelog would still contradict the exposed feature; proceed directly to Milestone 3.

### Milestone 3: Publish accurate guidance

First add failing assertions to `tests/cli_contract.rs`, `tests/docs_cli.rs`, and `tests/skill_cli.rs` for the new profile names and mixed H.c/L source wording. Then update the long help and profile flag description in `src/cli/extract.rs`; public guidance in `docs/public/commands/extract.md`, `docs/public/commands/overview.md`, `docs/public/workflows/extract-handshake.md`, and `docs/public/reference/machine-output.md`; routing text in `docs/skills/wavepeek.md`; and the unreleased entry in `CHANGELOG.md`. State that AXI5 and AXI5-Lite use Issue L, DVM `ac`/`cr` channels are optional for AXI5, and only ready/valid transport is extracted. Avoid duplicating generated flag tables.

Run focused help, docs, skill, schema, and auxiliary tests until green. Request an independent read-only documentation/help/changelog review; the reviewer must not edit the working tree. Fix findings through contract tests where possible, update this plan, and run the normal commit path so installed hooks validate the complete slice. Commit runtime profiles, fixtures, generated contracts, checker expectations, narrative help, public docs, packaged skill, changelog, tests, and the updated plan together. The exact clap possible-values assertion was already updated in Milestone 2; this milestone makes every remaining user-facing claim coherent.

### Milestone 4: Full validation, final review, and delivery

Run `just check` and `just ci`, saving long logs under repository-root `tmp/` without deleting unrelated files. Confirm only intentional changes remain. Run parallel read-only final reviews for runtime/protocol correctness, contract/schema/test consistency, and docs/help/changelog consistency over `main...HEAD`. Resolve findings with regression tests, commit fixes through normal hooks as a coherent conventional fix commit, and rerun both `just check` and `just ci` on the resulting `HEAD`. Request a fresh independent read-only control review of that consolidated diff. If the control review finds a substantive issue, repeat the regression-test, hooked-fix-commit, and both-full-gates sequence before one final control pass.

Only after the final control pass is clean, complete `Outcomes & Retrospective`, remove `docs/tracker/wip/issue-58-extract-axi5.md` as required by WIP policy, and commit the cleanup. Push `feat/extract-axi5` to `origin` and open a GitHub pull request against `main` with a concise summary, validation list, and `Closes #58`.

### Concrete Steps

All commands run from `/workspaces/wavepeek/.worktrees/feat-extract-axi5`.

Inspect state and keep the plan current:

    git status --short --branch
    git diff --check

Run focused Rust tests during RED/GREEN loops:

    cargo test --lib engine::axi::tests
    cargo test --test extract_axi_cli
    cargo test --test schema_cli
    cargo test --test cli_contract
    cargo test --test docs_cli
    cargo test --test skill_cli

Generate and validate source-backed fixtures and schemas:

    just prepare-waveform-fixtures
    just update-schema
    just check-schema

Run focused auxiliary checks when checker code changes:

    python3 -B -m unittest tools.schema.test_check_schema_contract
    python3 -B -m unittest tools.docs.test_check_deploy

Run milestone static checks and final gates:

    just format
    just lint
    just test
    just check
    just ci

The expected focused-test transcript ends with all selected tests passing and no failed tests. During each RED phase, newly added tests must fail for the expected missing behavior rather than fixture, syntax, or environment errors. `just check-schema`, `just check`, and `just ci` must exit zero.

Commit reviewed milestones with conventional messages. Suitable subjects are:

    docs(plan): add AXI5 extraction exec plan
    feat(axi): add AXI5 extraction profiles
    docs(axi): document Issue L profiles
    chore(plan): remove completed AXI5 exec plan

Before each commit, inspect and stage only owned files:

    git diff --check
    git status --short
    git add <reviewed-paths>
    git diff --cached --check
    git diff --cached --stat
    git commit -m '<conventional subject>'

Do not bypass hooks. Runtime profile data, generated schemas, checker expectations, exact clap value assertions, narrative help, public docs, packaged skill, and changelog form one coherent feature commit because intermediate commits would expose contradictory contracts.

After final validation and WIP cleanup, create and inspect the PR body, then deliver and verify the branch with repository-scoped commands:

    mkdir -p tmp
    printf '%s\n' \
      '## Summary' \
      '' \
      '- add AXI5 and AXI5-Lite ready/valid extraction profiles from Arm IHI 0022L' \
      '- support AXI5 DVM AC/CR channels and profile-specific auto-mapping' \
      '- publish exact input, output, and stream schema branches and user guidance' \
      '' \
      '## Validation' \
      '' \
      '- `just check`' \
      '- `just ci`' \
      '' \
      'Closes #58' > tmp/issue-58-pr.md
    cat tmp/issue-58-pr.md
    git push -u origin feat/extract-axi5
    gh pr create -R kleverhq/wavepeek --base main --head feat/extract-axi5 \
      --title 'feat(axi): add AXI5 extraction profiles' \
      --body-file tmp/issue-58-pr.md
    gh pr view -R kleverhq/wavepeek --json number,state,title,url,headRefName,baseRefName

If push is rejected, run `git fetch origin` and inspect `git log --oneline --left-right origin/feat/extract-axi5...HEAD`. Retry a normal push only when the remote tip is an ancestor of local `HEAD`; otherwise stop and reconcile the unexpected remote commits without force-pushing. If PR creation reports that a PR already exists, locate it with `gh pr list -R kleverhq/wavepeek --head feat/extract-axi5 --state all` and verify or update that PR instead of creating a duplicate.

### Validation and Acceptance

The feature is accepted when all of the following behavior is observable:

- `wavepeek extract axi --help` lists `axi5` and `axi5-lite` with the existing canonical profiles and describes H.c/L source ownership accurately.
- CLI and source parsing accept `axi5`, `axi5-lite`, `AXI5_LITE`, and `axi5_lite`, while output normalizes to canonical names.
- `axi5` output reports `issue: "L"` and emits mapped transfers in `aw`, `w`, `b`, `ar`, `r`, `ac`, `cr` order.
- `axi5-lite` output reports `issue: "L"`, emits only `aw`, `w`, `b`, `ar`, and `r`, and rejects burst, coherency, last-beat, and DVM channel signals.
- Auto-mapping recognizes compact and split Issue L names, including `acaddr`, without truncating compound suffixes or matching check/parity/credit/interface-control decoys.
- Explicit mappings accept all issue-required representative signals and reject `awbar`, `awunique`, `arbar`, and `cdvalid` for AXI5; reject `awlen`, `awburst`, `awcache`, `wlast`, `rlast`, `arsnoop`, and `acvalid` for AXI5-Lite; reject credited transport signals for both profiles; and reject representative interface-level signals such as `awakeup`, `varqosaccept`, `syscoreq`, `broadcastatomic`, and `activatereq`.
- Human, JSON, and JSONL outputs carry canonical profile names and Issue L metadata, and machine outputs validate against current schemas.
- Canonical input schemas accept the new profiles and exact mappings while rejecting noncanonical aliases and cross-profile keys.
- Existing AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 behavior remains green.
- `just check` and `just ci` both succeed.

### Idempotence and Recovery

Fixture and schema generation are deterministic and safe to repeat. Generated VCD/FST files remain ignored; only Verilog sources and waveform policy entries are committed. Schema snapshots are regenerated from Rust contracts with `just update-schema`; interrupted generation is recovered by rerunning the command rather than editing JSON manually. Milestone commits provide recovery points. Review findings are reproduced in tests before fixes where practical, and hooks are never bypassed.

Disposable source files, logs, and ad hoc output belong under repository-root `tmp/`. Existing files there may belong to another agent or the user and must not be deleted arbitrarily.

### Artifacts and Notes

Initial repository state:

    branch: feat/extract-axi5
    base commit: 961155e
    base relation: identical to origin/main and main
    working tree before plan: clean

Issue authority is GitHub issue #58. Protocol authority is Arm IHI 0022L Issue L, using the sections listed in `Context and Orientation`. Existing profiles continue to use Arm IHI 0022H.c.

### Interfaces and Dependencies

No new third-party dependency is required. `clap::ValueEnum` remains the CLI parser, `serde` remains the runtime source parser, `schemars` and `src/contract/axi_schema.rs` remain the schema generators, and the existing generic extraction engine remains the only runtime.

At completion, `src/cli/extract.rs` contains `AxiProfileArg::{Axi3, Axi4, Axi4Lite, Axi5, Axi5Lite, Ace, AceLite, Ace5}`. `src/engine/axi.rs` returns matching profile specs from `profile_specs()`, recognizes canonical and underscore aliases through `parse_profile()`, and associates the two new profiles with Issue L. No new extraction trait, waveform backend API, or transport abstraction is introduced.

Plan revision note (2026-07-11): Created the initial self-contained plan after repository and specification investigation. It records exact Issue L signal inventories, the ready/valid-only boundary, TDD milestones, per-milestone review requirements, generated-contract ownership, and PR delivery steps.

Plan revision note (2026-07-11): Incorporated independent plan and protocol-data review. The runtime slice now explicitly remains uncommitted until schemas and exact help contracts are coherent, fixtures require VCD/FST parity and `acaddr` coverage, protection-attribute wording is precise, representative interface-control exclusions are tested, and delivery commands are reproducible.

Plan revision note (2026-07-11): Incorporated follow-up plan review. Runtime, contracts, narrative guidance, and changelog now form one coherent feature commit after all three reviewed implementation milestones, and delivery creates the PR body explicitly and handles remote divergence without unsafe force-push guidance.

Plan revision note (2026-07-11): Incorporated fresh control review. Every milestone review is explicitly read-only, and any final-review fix must be committed through hooks and pass both full gates before control review, WIP cleanup, push, and PR creation.

Plan revision note (2026-07-11): Recorded Milestone 1 RED/GREEN evidence and the Git-index dependency of fixture-policy validation. Runtime profile and fixture changes remain uncommitted until machine contracts and user guidance are coherent.

Plan revision note (2026-07-11): Recorded Milestone 1 review findings and fixes. The fixture now proves channel-specific staggered transfers with stable legal payloads and passes clean runtime, protocol-data, and test-fixture follow-up reviews.

Plan revision note (2026-07-11): Recorded Milestone 2 RED/GREEN evidence. Canonical schema artifacts now contain exact AXI5 and AXI5-Lite mapping, issue, channel, and payload branches while `schema/catalog.json` remains unchanged.

Plan revision note (2026-07-11): Recorded Milestone 2 reviews and fixes. Contract tests now isolate parent/row profiles, cross-profile mappings, canonical input branches, and AXI5 stream record behavior without claiming cross-record JSONL coupling.

Plan revision note (2026-07-11): Recorded Milestone 3 RED/GREEN evidence. Generated help and embedded guidance now distinguish Issue H.c and Issue L profiles, describe AXI5 DVM channels and ready/valid-only extraction, and link issue #58 in the unreleased changelog.

Plan revision note (2026-07-11): Recorded the clean Milestone 3 documentation/help/changelog review. All user-facing and machine-facing surfaces are coherent for the feature commit.

Plan revision note (2026-07-11): Recorded the coherent feature commit and successful installed hooks before full handoff gates.

Plan revision note (2026-07-11): Recorded successful `just check` and `just ci` gates at feature commit `79bae7b`; full logs are stored under ignored `tmp/` paths.

Plan revision note (2026-07-11): Recorded parallel final-review findings and clean follow-up reviews. The AXI5 fixture now uses a legal combined optional-feature configuration, and deployment validation rejects stale v2.2 output and stream AXI profile branches.
