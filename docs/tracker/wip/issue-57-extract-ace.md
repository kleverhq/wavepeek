# Add ACE-family ready/valid extraction profiles

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. Implementation follows test-driven development: each behavior is first expressed by a failing test, then implemented minimally until the test passes, and finally refactored while tests remain green.

## Purpose / Big Picture

After this change, a user can run `wavepeek extract axi --profile ace`, `--profile ace-lite`, or `--profile ace5` against VCD, FST, or supported FSDB waveforms and receive one deterministic row for each mapped ready/valid channel transfer. The new profiles preserve the existing `extract axi` model: they sample mapped payload values one dump tick before a rising `aclk` edge, optionally require active-low reset `aresetn` to be deasserted, and do not reconstruct bursts, ordering, or coherency state.

The behavior is visible by generating the source-backed waveform fixtures with `just prepare-waveform-fixtures`, running the new integration tests through `just test`, and invoking representative commands such as:

    wavepeek extract axi --waves tests/fixtures/generated/extract_ace.vcd \
      --profile ace --scope top --map aclk=clk \
      --include '^ace_.*' --json

The JSON result must report `profile: "ace"`, `issue: "H.c"`, mappings for recognized ACE signals, and transfer rows for the mapped base AXI and ACE coherency channels.

## Non-Goals

This work does not add ACE-LiteDVM, ACE5-Lite, ACE5-LiteDVM, or ACE5-LiteACP profiles. It does not join address and data beats, reconstruct bursts or complete transactions, validate ordering or coherency rules, track outstanding operations, infer interface capability properties, or interpret payload encodings. It does not extract standalone `rack` or `wack` acknowledgements because those signals are not ready/valid channels. It excludes check and parity signals, wakeup and coherency-connection signals, static interface controls, QoS-accept controls, and credited transport when credited transport replaces ready/valid flow control.

## Progress

- [x] (2026-07-09 20:01Z) Read issue #57, the existing AXI extraction implementation, public contracts, test policies, schema generation path, and Arm IHI 0022H.c signal definitions.
- [x] (2026-07-09 20:01Z) Resolve the ACE-Lite `awunique` omission with the maintainer: include it as a legal optional payload signal.
- [x] (2026-07-09 20:09Z) Review and correct the initial ExecPlan, including the exact ACE5 whitelist, TDD milestone boundaries, schema source ownership, and executable test commands.
- [x] (2026-07-09 20:10Z) Commit the reviewed initial ExecPlan as `2b0fa41`.
- [x] (2026-07-09 20:17Z) Milestone 1 RED: add focused unit and CLI parsing tests plus source-backed ACE-family fixtures and human-output end-to-end tests; four unit tests and five integration tests fail because the profiles and AC/CR/CD token decomposition do not exist.
- [x] (2026-07-09 20:26Z) Milestone 1 GREEN: implement profile metadata, runtime extraction, and AC/CR/CD auto-mapping tokenization; focused tests, all 669 library tests, fixture policy, formatting, and lint pass; code and protocol-data reviews plus a fresh control review report no remaining findings. Commit is deferred until Milestone 2 because profile specs immediately own generated schemas checked by mandatory hooks.
- [x] (2026-07-09 20:29Z) Milestone 2 RED: add JSON, JSONL, and generated-contract tests for canonical profile enums and exact profile/channel payload branches; three integration tests and four schema tests fail against the stale three-profile artifacts.
- [x] (2026-07-09 20:42Z) Milestone 2 GREEN: update contract sources, schema and deployment checker expectations, and generated output/input/stream artifacts; 28 schema tests, 24 AXI integration tests, schema drift checks, formatting, and lint pass; contract and test reviews plus a fresh control review report no remaining findings. `schema/catalog.json` remained unchanged. Commit is deferred until Milestone 3 because generated CLI help already invalidates the stale help contract test.
- [x] (2026-07-09 20:42Z) Milestone 3 RED: the existing help contract fails on the expanded clap value list; new packaged-skill and embedded-doc tests fail because ACE-family guidance is absent.
- [x] (2026-07-09 20:47Z) Milestone 3 GREEN: update help, public docs, packaged skill, changelog, and deployment-checker fixture; 50 CLI contract tests, 25 docs tests, 3 skill tests, 79 docs-tool tests, full auxiliary tests, formatting, and lint pass; docs review and a fresh control review report no findings.
- [x] (2026-07-09 21:02Z) Run `just check` and `just ci`; rerun both after final-review fixes. All checks pass, including FSDB-enabled tests and the coverage gate at 94.25% regions, 93.66% functions, and 94.66% lines.
- [x] (2026-07-09 21:02Z) Run parallel final code, contract/schema/test, and docs reviews. Fix the ACE-Lite fixture transaction controls and deployment-checker schema reference handling through failing tests; focused follow-ups and a fresh consolidated control review report no remaining findings.
- [ ] Remove this completed branch-local WIP plan in a cleanup commit, push `feat/extract_ace`, and open a GitHub pull request closing issue #57.

## Surprises & Discoveries

- Observation: Issue #57 omitted `awunique` from ACE-Lite even though it is a legal optional ACE-Lite write-address payload signal.
  Evidence: Arm IHI 0022H.c §D11.1 does not exclude `AWUNIQUE`; Table D2-2 defines it, and Table D3-9 permits it for write transaction types available to ACE-Lite. The maintainer approved including it.

- Observation: The existing implementation is profile-data-driven rather than split into separate engines.
  Evidence: `src/engine/axi.rs` represents each profile as an `AxiProfileSpec`, and `src/contract/axi_schema.rs` derives profile-specific output, stream, and input JSON Schema branches from `profile_specs()`.

- Observation: Independent plan review caught an initial draft ACE5 inventory that exceeded issue #57 and identified a milestone dependency between JSON integration tests and generated schemas.
  Evidence: The corrected plan now uses the issue's exact ACE5 lists and moves schema-validating JSON and JSONL tests to Milestone 2.

- Observation: The waveform fixture policy test inventories tracked Verilog sources with `git ls-files`, so a new source-backed fixture must be staged before that test can recognize it.
  Evidence: The test initially reported all three new source files missing from the tracked-source set, then passed unchanged after staging those files.

- Observation: Milestone review found protocol-invalid field widths in otherwise functional test fixtures.
  Evidence: `AWSNOOP`, `AxBAR`, `BIDUNQ`, and `AxVMIDEXT` were corrected to their H.c widths; the existing human-output assertions failed after fixture regeneration and were updated only after demonstrating the corrected payloads.

- Observation: The GitHub Pages deployment checker independently hard-coded the AXI input profile enum in addition to the canonical schema checker.
  Evidence: A repository-wide stale-profile search found `tools/docs/check_deploy.py` and its test fixture after schema review; both now require the six canonical values.

- Observation: Final review showed that the deployment checker test used an inline profile enum while the generated input schema uses a `$ref` to `$defs.axiProfile`.
  Evidence: The canonical-artifact test failed before the checker was corrected. The checker now validates both the reference and referenced enum, and its test suite loads `schema/input.json` directly.

- Observation: Protocol-realistic field widths do not guarantee a protocol-realistic transaction fixture.
  Evidence: Final review identified illegal simultaneous ACE-Lite snoop/barrier encodings. The AW row now represents WriteUnique and the AR row represents Barrier according to Arm IHI 0022H.c Tables D11-1 and D11-2.

## Decision Log

- Decision: Treat `ace`, `ace-lite`, and `ace5` as property-agnostic legal signal supersets for waveform extraction, not as interface conformance declarations.
  Rationale: Many ACE5 signals are optional or conditional on interface properties. The extractor already accepts optional mappings and omits unmapped payloads, so accepting all legal functional ready/valid signals supports real waveforms without pretending to validate the interface configuration.
  Date/Author: 2026-07-09 / coding agent

- Decision: Include `awunique` in both ACE and ACE-Lite AW signal lists.
  Rationale: Arm IHI 0022H.c permits `AWUNIQUE` in ACE-Lite even though issue #57 omitted it. Rejecting it would make the whitelist incomplete. The maintainer explicitly approved the correction.
  Date/Author: 2026-07-09 / maintainer and coding agent

- Decision: Preserve the existing extraction runtime and extend only profile data, token decomposition, fixtures, contracts, and user-facing descriptions.
  Rationale: ACE-family channels use the same rising-clock ready/valid handshake and pre-edge payload sampling model already implemented by `extract generic` and adapted by `src/engine/axi.rs`. A second runtime would duplicate semantics and increase drift risk.
  Date/Author: 2026-07-09 / coding agent

- Decision: Review every completed milestone before committing it, then run a multi-lane final review and a fresh control pass.
  Rationale: The change spans protocol data, matching behavior, generated machine contracts, fixtures, and documentation. Focused review at each boundary limits the size of defects carried into later generated artifacts.
  Date/Author: 2026-07-09 / maintainer and coding agent

- Decision: Package reviewed Milestone 1 runtime work with Milestone 2 generated contracts instead of forcing an intermediate commit.
  Rationale: `profile_specs()` immediately drives the canonical schemas, and mandatory commit hooks reject schema drift. Bypassing hooks is prohibited, so an independently reviewed runtime milestone remains a valid stopping point but not a valid commit boundary.
  Date/Author: 2026-07-09 / coding agent

- Decision: Validate the actual canonical input schema in deployment-checker tests in addition to a small synthetic fixture.
  Rationale: The synthetic fixture drifted from the generated `$ref` shape while all repository gates passed. Loading `schema/input.json` directly prevents that class of false confidence.
  Date/Author: 2026-07-09 / coding agent

## Outcomes & Retrospective

Implementation and validation are complete. The CLI and runtime recognize all three ACE-family profiles, exact profile data drives extraction without a new engine, AC/CR/CD auto-mapping is collision-tested, and source-backed fixtures demonstrate every profile channel with protocol-valid controls. Exact input/output/stream schema branches validate canonical profiles and profile-specific mappings, channels, and payloads. Help, embedded docs, packaged skill, changelog, schema checks, and deployment checks describe and enforce the expanded support.

TDD recorded expected failures before each behavior slice and before both final-review fixes. Reviews corrected fixture widths and transaction encodings, strengthened isolated schema rejection coverage, and found a synthetic deployment-checker fixture that had drifted from the canonical schema shape. `just check` and `just ci` pass after all fixes, and the final independent control review reports no substantive findings. Only removal of this WIP artifact, branch push, and PR creation remain.

## Context and Orientation

`wavepeek` is a Rust command-line waveform inspector. The repository root is `/workspaces/wavepeek/.worktrees/feat-extract_ace`, and all commands in this plan run there inside the project devcontainer. Repository tasks are exposed through the root `justfile`; `just check` is the local handoff gate and `just ci` is the full CI gate.

A ready/valid channel transfers when both its `valid` and `ready` signals are true at the sample point associated with a rising clock edge. `src/engine/axi.rs` maps lowercase standard protocol names such as `awvalid` to waveform paths, validates that a mapped channel has a complete ready/valid pair, builds one generic extraction source per complete channel, and converts generic rows back to AXI transfer rows. `AxiProfileSpec` owns each profile's name, Arm specification issue, ordered channels, and accepted standard signal names. `standard_parts` decomposes a standard name into matching tokens so waveform names such as `ace_ac_valid_o` can map to `acvalid` without substring collisions.

`src/cli/extract.rs` exposes the profile through clap's `ValueEnum`. Canonical names use lowercase kebab case, while parsing also accepts case-insensitive underscore aliases such as `ACE_LITE`. `src/contract/input.rs`, `src/contract/output.rs`, and `src/contract/stream.rs` contain serialization data transfer objects and human-readable schema descriptions. `src/contract/axi_schema.rs` replaces their broad generated AXI definitions with branches derived from `profile_specs()`, so every profile has exact channel and payload-key enums. The generated canonical artifacts are `schema/input.json`, `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`; they must be regenerated with `just update-schema`, never edited manually. Inspect all four and commit the catalog only if deterministic regeneration changes it.

`tests/extract_axi_cli.rs` contains AXI command integration tests. Source-backed Verilog fixtures live in `tests/fixtures/source/`; `tools/waveform/prepare_fixtures.py`, invoked by `just prepare-waveform-fixtures`, generates ignored VCD files under `tests/fixtures/generated/`. Every source-backed output must be declared in `tests/fixtures/waveform_policy.json`. `tests/schema_cli.rs` validates representative runtime objects against the generated output and stream schemas. `tests/cli_contract.rs` checks long help, and `tests/skill_cli.rs` checks the packaged skill at `docs/skills/wavepeek.md`.

The new profiles are based on Arm IHI 0022H.c. Every profile includes common `aclk` and optional `aresetn`. Existing AXI channels remain ordered `aw`, `w`, `b`, `ar`, `r`. ACE and ACE5 append coherency channels `ac`, `cr`, `cd`, yielding `aw`, `w`, `b`, `ar`, `r`, `ac`, `cr`, `cd`. ACE-Lite has only the five base AXI channels.

ACE uses these channel whitelists. AW accepts `awid`, `awaddr`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awqos`, `awregion`, `awuser`, `awsnoop`, `awdomain`, `awbar`, `awunique`, `awvalid`, and `awready`. W accepts AXI4 W signals. B accepts AXI4 B signals. AR accepts `arid`, `araddr`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arqos`, `arregion`, `aruser`, `arsnoop`, `ardomain`, `arbar`, `arvalid`, and `arready`. R accepts AXI4 R signals plus `rresp`. The whole sampled `rresp` vector carries any ACE response extension bits. AC accepts `acaddr`, `acsnoop`, `acprot`, `acvalid`, and `acready`. CR accepts `crresp`, `crvalid`, and `crready`. CD accepts `cddata`, `cdlast`, `cdvalid`, and `cdready`.

ACE-Lite starts from AXI4. AW additionally accepts `awsnoop`, `awdomain`, `awbar`, and `awunique`. AR additionally accepts `arsnoop`, `ardomain`, and `arbar`. It has no AC, CR, or CD channels and no standalone RACK or WACK extraction.

ACE5 uses the issue #57 functional signal whitelist and does not accept removed `awbar` or `arbar`. AW accepts `awid`, `awaddr`, `awlen`, `awsize`, `awburst`, `awlock`, `awcache`, `awprot`, `awqos`, `awregion`, `awuser`, `awdomain`, `awsnoop`, `awunique`, `awtrace`, `awloop`, `awmmusecsid`, `awmmusid`, `awmmussidv`, `awmmussid`, `awmmuatst`, `awnsaid`, `awmpam`, `awidunq`, `awvalid`, and `awready`. W accepts `wdata`, `wstrb`, `wlast`, `wuser`, `wpoison`, `wtrace`, `wvalid`, and `wready`. B accepts `bid`, `bresp`, `buser`, `btrace`, `bloop`, `bidunq`, `bvalid`, and `bready`. AR accepts `arid`, `araddr`, `arlen`, `arsize`, `arburst`, `arlock`, `arcache`, `arprot`, `arqos`, `arregion`, `aruser`, `ardomain`, `arsnoop`, `arvmidext`, `artrace`, `arloop`, `armmusecsid`, `armmusid`, `armmussidv`, `armmussid`, `armmuatst`, `arnsaid`, `armpam`, `aridunq`, `arvalid`, and `arready`. R accepts `rid`, `rdata`, `rresp`, `rlast`, `ruser`, `rpoison`, `rtrace`, `rloop`, `ridunq`, `rvalid`, and `rready`. AC accepts `acaddr`, `acsnoop`, `acprot`, `acvmidext`, `actrace`, `acvalid`, and `acready`. CR accepts `crresp`, `crtrace`, `crnsaid`, `crvalid`, and `crready`. CD accepts `cddata`, `cdlast`, `cdpoison`, `cdtrace`, `cdvalid`, and `cdready`.

## Open Questions

There are no open protocol or product questions. Optional and conditional ACE-family signals are accepted when mapped and are not required. The generated input schema advertises canonical profile enum values, while runtime parsing continues the existing behavior of accepting case-insensitive hyphen and underscore aliases.

## Plan of Work

### Milestone 1: Prove and implement profiles, runtime extraction, and collision-safe mapping

Create minimal deterministic Verilog fixtures under `tests/fixtures/source/` that produce representative base and coherency-channel transfers for ACE, ACE-Lite, and ACE5. Include enough signals to prove the three new channel prefixes, ACE-Lite's address additions including `awunique`, ACE5 representative optional payloads, and exclusion behavior. Register generated VCD outputs in `tests/fixtures/waveform_policy.json` and regenerate them with `just prepare-waveform-fixtures`.

Before adding any profile implementation, add tests in the unit-test module of `src/engine/axi.rs`, CLI contract tests, and integration tests in `tests/extract_axi_cli.rs`. Cover `ace`, `ace-lite`, `ace5`, `ACE_LITE`, and `ace_lite`; exact ordered channel lists; representative legal and illegal standard names; automatic and explicit mapping; source-file aliases; human extraction output; and extraction from every profile channel. Defer schema-validating JSON and JSONL integration assertions to Milestone 2 because the checked-in schemas intentionally remain unchanged during this milestone. Add tokenization and matching tests proving `acvalid`, `crready`, and `cddata` match channel-prefixed candidate names while `aclk` remains a common whole token and AW/W, AR/R, and AC/common collisions do not appear. Include the accepted ACE-Lite `awunique` and rejected ACE5 `awbar`/`arbar`. Run the focused tests and record that they fail because the profiles do not exist.

Then add `Ace`, `AceLite`, and `Ace5` variants to `AxiProfileArg` in `src/cli/extract.rs`; extend `Display`, conversion, and parsing in `src/engine/axi.rs`; define static profile/channel specs with the exact whitelists in this plan; expose them from `profile_specs()`; and extend `standard_parts` with `ac`, `cr`, and `cd` channel prefixes after common-name handling. Keep all payload signals optional and preserve the existing validation and extraction algorithms. Run focused tests until green, run formatting and lint checks relevant to the changed Rust files, request independent code and protocol-data reviews, fix findings through regression tests, and commit the milestone including fixture sources and policy but not ignored generated waveform dumps.

### Milestone 2: Prove exact machine contracts

After runtime support is green, add JSON and JSONL integration assertions in `tests/extract_axi_cli.rs` and `tests/schema_cli.rs` cases that require the new canonical profile enums and accept only profile-appropriate channel/payload combinations; assert ACE5 rejects `awbar` and `arbar`. Run the tests before changing contract code and capture failures caused by schemas that still describe only the original profiles.

Update the baseline input enum in `src/contract/input.rs`, stale transfer profile/channel descriptions in `src/contract/output.rs`, and the profile description generated by `src/contract/axi_schema.rs`. The exact output and stream branches remain owned by `src/contract/axi_schema.rs` and the profile specs. Run `just update-schema`, inspect the generated diff, run `just check-schema`, and run focused schema tests until green. Request a contract/schema/test review, address findings through tests, and commit the milestone including generated schema artifacts.

### Milestone 3: Publish accurate user guidance

First extend `tests/cli_contract.rs` and `tests/skill_cli.rs` with assertions requiring the new profile names and ACE-family scope. Run the focused tests and confirm stale help and skill text fail. Then update `src/cli/extract.rs` long help; `docs/public/commands/extract.md`; profile claims in `docs/public/commands/overview.md`, `docs/public/workflows/extract-handshake.md`, and `docs/public/reference/machine-output.md`; and routing guidance in `docs/skills/wavepeek.md`. Describe functional ready/valid extraction and preserve the non-goals without duplicating generated flag tables. Amend the unreleased AXI extraction changelog entry in `CHANGELOG.md` because the original capability has not shipped yet. Run focused help, docs, skill, and schema tests until green, request a docs/help/changelog review, address findings through contract tests where possible, and commit the milestone.

### Milestone 4: Full validation, final review, and delivery

Run `just check` and `just ci` from the repository root and save concise logs under ignored `tmp/` if output is long. Confirm `git status --short` contains only intentional tracked work. Run parallel read-only final reviews for code correctness, contract/schema/test consistency, and docs/help/changelog consistency over `main...HEAD`. Resolve findings, rerun affected tests, and then request a fresh independent control review of the consolidated diff. Complete `Outcomes & Retrospective` and the evidence sections of this plan, commit the updated plan if useful for restart safety, then remove `docs/tracker/wip/issue-57-extract-ace.md` before merge as required by the WIP tracker policy and commit that cleanup. Run the final gates again if cleanup or review fixes touched executable or contract files. Push the branch and open a PR with a concise summary, validation list, and `Closes #57`.

### Concrete Steps

All commands run from `/workspaces/wavepeek/.worktrees/feat-extract_ace`.

Inspect status and keep the plan current:

    git status --short --branch
    git diff --check

Run focused Rust integration tests during RED/GREEN cycles with container-local Cargo, as permitted by `docs/dev/testing.md`:

    cargo test --test extract_axi_cli
    cargo test --test schema_cli
    cargo test --test cli_contract
    cargo test --test skill_cli

The `just test` recipe intentionally runs the full suite and does not forward Cargo arguments. Use it at milestone or final boundaries when a full test run is required:

    just test

Regenerate source-backed waveforms and schemas:

    just prepare-waveform-fixtures
    just update-schema
    just check-schema

Run milestone-level static checks and final gates:

    just format
    just lint
    just check
    just ci

The expected focused-test transcript ends with all tests passing and no failed tests. `just check-schema` must report no generated schema drift. `just check` and `just ci` must exit zero.

Commit reviewed milestones with conventional messages. Suitable subjects are:

    feat(axi): add ACE profile definitions
    test(axi): cover ACE extraction contracts
    docs(axi): document ACE extraction profiles

The exact split may combine tests with the implementation that makes them pass, but every commit must leave the repository in a coherent reviewed state.

### Validation and Acceptance

The feature is accepted when all of the following behavior is observable:

- `wavepeek extract axi --help` lists canonical `ace`, `ace-lite`, and `ace5` profile values in addition to existing profiles.
- CLI and source JSON parsing accept canonical names case-insensitively and accept `ACE_LITE` and `ace_lite` aliases, while output normalizes to `ace-lite`.
- ACE extraction reports base AXI and `ac`, `cr`, and `cd` channel rows in profile order and includes mapped payload values.
- ACE-Lite accepts mapped address additions including `awunique`, emits only `aw`, `w`, `b`, `ar`, and `r`, and rejects ACE-only AC/CR/CD names as unsupported mappings.
- ACE5 accepts representative optional additions on every channel, reports coherency channels, and rejects removed `awbar` and `arbar` names.
- Auto-mapping recognizes `ac*`, `cr*`, and `cd*` token forms without regressing AW/W, AR/R, or `aclk` matching.
- JSON and JSONL outputs validate against their current generated schemas, and source documents validate against the input schema for canonical values.
- Existing AXI3, AXI4, and AXI4-Lite tests remain green.
- `just check` and `just ci` both succeed.

### Idempotence and Recovery

Fixture and schema generation are deterministic and safe to repeat. Generated VCD files remain ignored; only their Verilog sources and waveform policy entries are committed. Schema snapshots are regenerated from Rust contracts with `just update-schema`; if generation is interrupted, rerun the command rather than editing partial JSON by hand. Milestone commits provide recovery points. If a review fix breaks later work, reproduce the problem in a test, amend the current milestone with a new commit rather than rewriting reviewed history, and rerun the affected gate.

Disposable source JSON files, logs, and ad hoc command output belong under repository-root `tmp/`. Do not delete unrelated existing files there.

### Artifacts and Notes

Initial repository state:

    branch: feat/extract_ace
    base commit: 3fa8994
    working tree: clean

Protocol authority used for the profile data is Arm IHI 0022H.c. Relevant sections are AXI4 base channels in §A2.2–A2.6 and Tables A2-2–A2-6; ACE additions in §D1.3 and Tables D1-2–D1-4 plus §D2.1–D2.2 and Tables D2-1–D2-6; ACE-Lite exclusions in §D11.1; ACE5 additions in §F1.2.1 and Tables F1-2–F1-9; required/optional status in Appendix G2 Table G2-2; and ACE5 barrier removal in §F5.2.

### Interfaces and Dependencies

No new third-party dependency is required. `clap::ValueEnum` remains the CLI profile parser. `serde` and `schemars` continue to own runtime JSON and baseline schema generation. `src/contract/axi_schema.rs` continues to derive exact profile branches from `crate::engine::axi::profile_specs()`.

At completion, `src/cli/extract.rs` contains `AxiProfileArg::{Axi3, Axi4, Axi4Lite, Ace, AceLite, Ace5}`. `src/engine/axi.rs` recognizes the same canonical profiles through its internal `AxiProfile`, returns all six specs from `profile_specs()`, and parses case-insensitive hyphen/underscore aliases. Each new `AxiChannelSpec` uses existing fields `name`, `valid`, `ready`, and `signals`; no new extraction trait or runtime layer is introduced.

Plan revision note (2026-07-09): Created the initial self-contained plan after repository and specification investigation. It records the approved ACE-Lite `awunique` correction, TDD sequencing, per-milestone review gates, generated-contract ownership, and final PR delivery requirements.

Plan revision note (2026-07-09): Corrected the ACE5 whitelist to match issue #57 and Arm IHI 0022H.c, moved end-to-end runtime RED tests before profile implementation, identified the exact hand-written contract sources, and replaced invalid focused `just test` examples with supported Cargo commands after independent plan review.

Plan revision note (2026-07-09): Moved schema-validating JSON and JSONL tests to the contract milestone, removed a nonexistent `profile_spec()` helper from the implementation instructions, and documented all four generated schema artifacts after plan re-review.

Plan revision note (2026-07-09): Aligned the Progress checklist with the Milestone 1 prose by deferring JSON and JSONL end-to-end tests to the contract milestone after the final plan control review.

Plan revision note (2026-07-09): Recorded Milestone 1 RED/GREEN evidence, the tracked-fixture policy behavior, review-driven field-width corrections, successful focused/static validation, and clean follow-up and control reviews.

Plan revision note (2026-07-09): Deferred the Milestone 1 commit to the generated-contract milestone because mandatory schema hooks make profile data and schema snapshots one commit unit; no hook is bypassed.

Plan revision note (2026-07-09): Recorded Milestone 2 RED/GREEN evidence, isolated schema negative cases added after test review, clean follow-up/control reviews, the deployment-checker enum update, and the help/docs/skill RED state for Milestone 3.

Plan revision note (2026-07-09): Recorded Milestone 3 GREEN validation and clean documentation/control reviews; all feature, contract, and guidance milestones are ready for a coherent commit and final repository gates.

Plan revision note (2026-07-09): Completed the retrospective after two passing full-gate runs, recorded final-review deployment-checker and ACE-Lite fixture fixes, and documented clean focused follow-ups and consolidated control review. The plan is ready for required WIP cleanup before PR creation.
