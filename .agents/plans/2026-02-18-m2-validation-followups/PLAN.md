# M2 Post-Validation Corrective Plan (v0.2.x)

## Summary
- This plan addresses the 13 post-M2 validation findings and aligns the CLI behavior with the latest product direction for human-first defaults, clearer argument errors, and stronger regression coverage.
- The work is split into contract updates, CLI UX fixes, command-surface changes (`tree` -> `modules`), output-mode migration, and container-level fixture provisioning.
- The expected result is a more usable CLI for both humans and agents: readable defaults, explicit opt-in JSON contract, cleaner metadata, and reproducible tests against real-world waveform artifacts.
- This revision adds a follow-up breaking-contract scope focused on metadata fidelity (`VarType`/`ScopeType`) and singular command names (`scope`, `signal`, `change`).
- Tasks 1-11 are historical baseline checkpoints; Tasks 12-13 define final command contracts for this plan revision.

## Goals
- Fix CLI entry/help/version behavior and lock it with integration tests.
- Standardize `args` error messaging to include actionable help hints.
- Rename `tree` command to `modules` as a direct cutover (no compatibility alias).
- Fix module hierarchy listing on realistic FST data (regression-proofed).
- Remove redundant `time_precision` field and keep only `time_unit` in `info` output.
- Switch default output mode to human-readable and introduce explicit `--json` envelope mode as the only machine-output switch for all commands.
- Improve human output ergonomics (`scope --tree`, `signal` short names + `--abs`).
- Provision required large fixtures in devcontainer image layer at stable path `/opt/rtl-artifacts`.
- Preserve full parser-provided signal type fidelity in output (`VarType` aliases, no coarse collapsing).
- Extend hierarchy listing to all scopes with explicit `kind` from `ScopeType`.
- Singularize command surface as direct cutover: `modules` -> `scope`, `signals` -> `signal`, `changes` -> `change`.

## Non-Goals
- Implementing M3/M4 functional scope (`at`, `change`, `when`) beyond CLI contract touchpoints required for output-mode migration.
- Replacing `schema_version` with a hosted `$schema` URL in this iteration.
- Defining a long-term warning-code taxonomy (warnings remain free-form text).
- Supporting `make` execution outside containerized environments.

## Background / Context
- The M2 plan at `.agents/plans/2026-02-08-m2-core-cli/PLAN.md` is complete, but validation uncovered UX and contract gaps.
- Current PRD defaults to JSON envelope output and command name `tree`, while the new direction favors human-default output and clearer module-oriented naming.
- Current tests cover core M2 functionality but do not fully cover `-V`/`--version`, short help flag `-h`, no-args invocation behavior, or external fixture-backed regressions.
- Real-world reproduction data (`scr1_max_axi_coremark.fst`, `picorv32_test_vcd.fst`) must be available for regular quality gates; runtime test download logic is intentionally avoided.

## Problem Statement
- Basic CLI ergonomics are inconsistent: no-args invocation errors instead of showing help, and parse errors lose useful context/hints.
- Output contracts carry redundant/noisy fields for default usage, while human-readable workflows require extra flags.
- `tree` naming/behavior mismatches user expectation (`modules` hierarchy), and a reported real-world fixture may expose traversal/classification issues.
- Current fixture strategy is too narrow to confidently prevent regressions on realistic dumps.
- Signal and scope typing is currently lossy (`map_signal_kind` collapse, module-only hierarchy filter), reducing debug value and hiding parser-provided structure.

## Requirements
- **Functional:**
- `VF-R1` (issue 1): `wavepeek` without subcommand prints top-level help to stdout and exits `0`.
- `VF-R2` (issues 2-3): Add integration coverage for `-V`/`--version` and `-h`/`--help`.
- `VF-R3` (issues 4-5): All `error: args:` messages include specific failure context and a help hint (`See 'wavepeek <cmd> --help'` or `See 'wavepeek --help'`).
- `VF-R4` (issue 6): Convert long command help descriptions to multiline string literals in CLI definitions.
- `VF-R5` (issue 7): Remove `time_precision` from `info` data model/output/docs; keep `time_unit`, `time_start`, `time_end`.
- `VF-R6` (issue 8): Default output mode is human-readable for all commands; `--json` enables strict envelope output.
- `VF-R7` (issues 9, 11): Rename `tree` to canonical `modules`, remove `tree` from CLI surface, and ensure output is module hierarchy (not signals).
- `VF-R8` (issue 10): Add optional human renderer `--tree` for `modules`; in JSON mode, `--tree` is accepted but ignored (no extra warning).
- `VF-R9` (issue 12): In `signals` human mode, print names without scope by default; `--abs` prints full paths; JSON remains full-path.
- `VF-R10` (issue 13): Provision required fixtures (`picorv32_test_vcd.fst`, `scr1_max_axi_coremark.fst`) in devcontainer image build from release `v1.0.0` and expose them under `/opt/rtl-artifacts`.
- `VF-R11`: Expand integration matrix to lock all fixture-dependent corrected behaviors and regressions using fixtures available at `/opt/rtl-artifacts`.
- `VF-R12`: Remove `--human` flag from CLI surface; output mode control is default human + explicit `--json`.
- `VF-R23`: `signal` output `kind` uses full parser-native `VarType` aliases (explicit stable string mapping), without collapsing multiple kinds into coarse buckets.
- `VF-R24`: `scope` command replaces `modules`, lists all hierarchy scopes (not only modules), and includes `kind` derived from `ScopeType` for each entry.
- `VF-R25`: `signal` command replaces `signals` as canonical surface; existing listing semantics and flags (`--scope`, `--max`, `--filter`, `--abs`, `--json`) are preserved.
- `VF-R26`: `change` command replaces `changes` as canonical surface (execution may remain unimplemented in this milestone, but CLI/help/error contracts use singular form).
- `VF-R27`: Legacy predecessor commands (`modules`, `signals`, `changes`) are removed from CLI surface and rejected with `error: args:` plus contextual help hint.
- **Non-functional / constraints:**
- `VF-R13`: Preserve deterministic JSON output in `--json` mode (`schema_version`, `command`, `data`, `warnings`).
- `VF-R14`: Keep stderr error format stable: `error: <category>: <message>`.
- `VF-R15`: `make ci` and `make pre-commit` run only inside containerized environments (devcontainer/CI container).
- `VF-R16`: Docker image fixture layer is versioned by variable (`RTL_ARTIFACTS_VERSION`).
- `VF-R17`: Fixture install path is stable (`/opt/rtl-artifacts`) without embedding version in directory name.
- `VF-R18`: Fixture provisioning is implemented as a dedicated Dockerfile layer to maximize cache reuse.
- `VF-R19`: `make ci` and `make pre-commit` include tests depending on `/opt/rtl-artifacts`; no runtime fixture download in test execution.
- `VF-R20`: Both `ci` and `dev` image targets include the same `/opt/rtl-artifacts` fixture payload.
- `VF-R21`: Enforce container-only quality-gate execution via explicit guard (outside-container `make ci`/`make pre-commit` fails fast with clear message).
- `VF-R22`: Keep fixture provenance reproducible by pinning artifact version in Dockerfile and using stable artifact names/paths.
- `VF-R28`: Bump JSON envelope `schema_version` for the singular-command and payload-shape breaking changes introduced by this follow-up.
- `VF-R29`: Keep deterministic ordering guarantees unchanged after widening hierarchy scope coverage (stable DFS + lexical tie-breakers).

## Traceability Matrix
| Scope item | Requirement IDs | Tasks | DoD checks | Primary artifacts |
|---|---|---|---|---|
| Contract docs and migration policy | VF-R6, VF-R7, VF-R8, VF-R5, VF-R12 | Task 1 | D23 | `.agents/PRD.md` (M2 conventions sections), `CHANGELOG.md` (`## [Unreleased]`) |
| CLI entry/help/version contract | VF-R1, VF-R2, VF-R14 | Task 2 | D1, D2 | `src/cli/mod.rs`, `tests/cli_contract.rs` |
| Args error quality + help hints | VF-R3, VF-R14 | Task 3 | D3, D4 | `src/cli/mod.rs`, `tests/cli_contract.rs` |
| Help text formatting style | VF-R4 | Task 4 | D5 | `src/cli/mod.rs` |
| Output mode migration | VF-R6, VF-R12, VF-R13 | Task 5 | D6, D7, D8 | `src/cli/*`, `src/engine/*`, `src/output.rs`, integration tests |
| `tree` to `modules` + hierarchy correctness | VF-R7, VF-R11 | Task 6 | D9, D10, D11 | `src/cli/mod.rs`, `src/engine/mod.rs`, `src/engine/modules.rs`, `src/waveform/mod.rs`, tests |
| Human tree-style rendering | VF-R8 | Task 7 | D12, D13 | `src/cli/mod.rs`, `src/cli/modules.rs`, `src/output.rs`, tests |
| Remove `time_precision` | VF-R5 | Task 8 | D14 | `src/waveform/mod.rs`, `src/engine/info.rs`, `src/output.rs`, tests, PRD |
| Signals human format + `--abs` | VF-R9 | Task 9 | D15, D16 | `src/cli/signals.rs`, `src/engine/signals.rs`, `src/output.rs`, tests |
| Container fixture provisioning | VF-R10, VF-R16, VF-R17, VF-R18, VF-R20, VF-R22 | Task 10 | D17, D18, D19, D20, D22 | `.devcontainer/Dockerfile`, `.devcontainer/devcontainer.ci.json`, `.devcontainer/devcontainer.json` |
| Full regression + quality gates | VF-R11, VF-R15, VF-R19, VF-R21 | Task 11 | D21, D23, D24 | `tests/*.rs`, `Makefile` targets |
| Full-fidelity kinds + all-scope listing (`modules` -> `scope`) | VF-R23, VF-R24, VF-R28, VF-R29 | Task 12 | D25, D26, D27, D30, D31 | `src/waveform/mod.rs`, `src/engine/scope.rs` (or renamed `modules.rs`), `src/cli/scope.rs` (or renamed `modules.rs`), `src/output.rs`, tests |
| Singular CLI surface for listing/range commands | VF-R25, VF-R26, VF-R27, VF-R28 | Task 13 | D28, D29, D31, D32 | `src/cli/mod.rs`, `src/cli/signals.rs`, `src/cli/changes.rs`, `src/engine/mod.rs`, tests, PRD/changelog |

## Proposed Solution
- Apply a direct contract cutover for unreleased `v0.2.0`:
- Make human output the default for command execution and add explicit `--json` for strict machine contract output.
- Keep cutover strict: no command aliases, no output-default compatibility bridge, and no `--human` compatibility mode.
- Improve argument error UX by preserving the key clap diagnostic line and appending deterministic help hints instead of stripping all context.
- Refactor command naming and rendering in one pass:
- Promote singular command names as canonical (`scope`, `signal`, `change`) and remove plural predecessors.
- Keep JSON data shape deterministic and flat.
- Add optional human renderer `--tree` for visual hierarchy output.
- Simplify metadata schema by removing redundant `time_precision` output field and updating all dependent tests/docs.
- Replace coarse signal-kind collapsing with explicit parser-native `VarType` alias mapping.
- Expand hierarchy listing from module-only to all scopes and add explicit `kind` from `ScopeType` in scope entries.
- Bump JSON `schema_version` to reflect this breaking contract revision.
- Add a dedicated Dockerfile layer that downloads pinned fixture artifacts at image-build time (versioned by `RTL_ARTIFACTS_VERSION`) and installs them at `/opt/rtl-artifacts`.
- Ensure both dev and CI container targets include the same fixture payload so `make ci` and `make pre-commit` can execute full fixture-backed coverage without runtime downloads.
- Extend integration tests to include new CLI contracts and fixture-backed regressions as part of normal quality gates.

## Alternatives Considered
- **Keep default JSON and only tweak errors:** rejected because it leaves the primary UX complaint (default output noise) unresolved.
- **Retain compatibility alias/bridge for unreleased contracts:** rejected because `v0.2.0` is not released and direct cleanup is simpler.
- **Lazy runtime fixture download in tests:** rejected due added runtime complexity and network-dependent test execution.
- **Separate external-fixture test job outside `make ci`/`make pre-commit`:** rejected because target workflow requires full coverage in standard quality gates.
- **Commit large real-world fixtures into repository:** rejected due to repository bloat and slower clone/check cycles.
- **Keep coarse signal/module taxonomy for readability:** rejected because it hides parser-level type detail needed for debugging and root-cause analysis.
- **Keep plural command aliases (`modules`/`signals`/`changes`) during transition:** rejected because direct cutover is accepted and avoids long-term CLI surface drift.

## Risks and Mitigations
- **Risk:** Breaking CLI contracts during direct cutover (`tree`, `--human`). **Mitigation:** update PRD/changelog first and lock behavior with integration tests in the same change set.
- **Risk:** JSON/human mode confusion after cutover. **Mitigation:** single-mode rule (default human + explicit `--json`) and focused contract tests.
- **Risk:** `modules` hierarchy bug is fixture-specific and hard to generalize. **Mitigation:** codify regression tests against downloaded real fixtures plus existing small fixtures.
- **Risk:** Network flakiness shifts to container build stage. **Mitigation:** isolated fixture Docker layer with version pinning and layer caching.
- **Risk:** Container image size increase from bundled fixtures. **Mitigation:** keep fixture set minimal to required files and rely on Docker layer caching.
- **Risk:** Full-scope traversal may significantly increase entry count and trigger more truncation warnings. **Mitigation:** keep deterministic ordering, preserve `--max`/`--max-depth`, and add focused fixture regressions for bounded output.
- **Risk:** Implicit stringification of upstream enums may change across dependency updates. **Mitigation:** use explicit internal mapping tables for `VarType`/`ScopeType` alias strings and lock with tests.

## Rollout / Migration Plan
- Phase 1: Update PRD/changelog contract language and document direct cutover policy.
- Phase 2: Land CLI parser/help/error UX fixes with focused tests (issues 1-6).
- Phase 3: Migrate output-mode defaults and command naming (`modules`, `--json`) as direct cutover.
- Phase 4: Apply behavior refinements (`modules --tree`, `signals --abs`, `info` field simplification).
- Phase 5: Add container fixture layer at `/opt/rtl-artifacts` and run full quality gates.
- Phase 6: Land Task 12 follow-up (`modules` -> `scope`, full-fidelity `VarType`/`ScopeType`, all-scope traversal, `schema_version` bump).
- Phase 7: Land Task 13 follow-up (`signals` -> `signal`, `changes` -> `change`) and complete docs/tests cutover.
- Rollback:
- Unit A rollback (CLI/help/errors): revert Task 2-4 commit set only.
- Unit B rollback (output-mode migration): revert Task 5 commit set and restore prior mode behavior.
- Unit C rollback (command rename): revert Task 6 commit set and restore `tree` command surface.
- Unit D rollback (singularization + fidelity follow-up): revert Task 12-13 commit set together to avoid mixed command/type contracts.

## Observability
- Add integration assertions for all CLI entry/error/help/version behaviors.
- Emit clear container build logs for fixture download source and artifact version.
- Validate reproducible fixture provenance via pinned Dockerfile version and stable install path.
- Track fixture-layer build failures as infrastructure-classified in CI triage.
- Verify deterministic JSON stability by repeated-run assertions in `--json` mode.

## Resolved Decisions
- Baseline checkpoint (Tasks 6-11): `tree` was replaced by `modules` with no alias; superseded by follow-up singular cutover in Tasks 12-13 (`scope`, `signal`, `change`).
- Output mode cutover: default is human and explicit `--json` enables envelope mode; no `WAVEPEEK_OUTPUT_DEFAULT` bridge is used.
- `--human` flag handling: remove `--human` from CLI surface and rely on default human mode.
- `--json --tree` interaction: JSON output stays flat list; `--tree` is ignored without extra warning.
- Fixture integrity source: pin release version in Dockerfile for required artifacts.
- Fixture provisioning model: download artifacts in a dedicated Dockerfile layer, keyed by `RTL_ARTIFACTS_VERSION`, and install at stable path `/opt/rtl-artifacts` in both `ci` and `dev` targets.
- Quality-gate execution model: all `make` quality gates are container-only and include fixture-backed tests.
- Follow-up command policy: singular names are canonical (`scope`, `signal`, `change`) and plural predecessors are removed without aliases.
- Follow-up type policy: preserve parser-native type fidelity by exposing explicit string aliases for `VarType` and `ScopeType` rather than coarse categories.
- Follow-up hierarchy policy: scope listing prints all scope kinds, not only `module` scopes.
- Follow-up contract constants: JSON `schema_version` increments from `1` to `2` when Task 12 lands.
- Follow-up alias coverage: mappings are exhaustive for all `wellen::VarType` and `wellen::ScopeType` variants in the pinned dependency version; future upstream additions fall back to `unknown` until mapped explicitly.

## Open Questions
- Should `$schema` URL support replace or complement `schema_version` in a future milestone.

## Assumptions
- The team accepts default human output + explicit `--json` for strict contract mode.
- `schema_version` remains the JSON envelope version marker in this scope and is incremented for this breaking follow-up.
- The fix is shipped before `v0.2.0` release, so direct CLI contract cleanup is acceptable.
- Fixture downloads happen only during container image build; test/runtime execution does not perform network fixture fetches.
- Stable lowercase aliases (explicit mapping table) are acceptable wire format for `VarType` and `ScopeType` in human and JSON outputs.

## Definition of Done
### Baseline checkpoints (completed; may be superseded by follow-up)
- [x] `D1`: `wavepeek` with no arguments exits `0` and prints top-level help to stdout.
- [x] `D2`: Integration tests explicitly cover `-h`, `--help`, `-V`, and `--version`.
- [x] `D3`: Missing-required-argument errors include the missing flag names in stderr.
- [x] `D4`: `error: args:` help hints are context-correct: global parse failures suggest `wavepeek --help`, subcommand failures suggest `wavepeek <cmd> --help`.
- [x] `D5`: Command `long_about` strings are multiline literals in CLI definitions.
- [x] `D6`: Historical checkpoint: default output for implemented commands at Task 5 stage (`info`, `modules`, `signals`) is human-readable when no format flag is provided (superseded naming in D27-D29).
- [x] `D7`: `--json` output matches strict envelope keys and deterministic ordering/content.
- [x] `D8`: `--human` is removed from CLI surface and tests assert the expected args error/hint if passed.
- [x] `D9`: Historical checkpoint: `modules` command exists at Task 6 stage (superseded by canonical `scope` in D27).
- [x] `D10`: `tree` invocation is rejected (no alias) with `error: args:` and help hint.
- [x] `D11`: Regression test with `scr1_max_axi_coremark.fst` confirms module paths (not signal leaves) in module listing.
- [x] `D12`: Historical checkpoint: `modules --tree` renders visual hierarchy format at Task 7 stage (superseded by `scope --tree` in D30).
- [x] `D13`: Historical checkpoint: `modules --json --tree` keeps flat JSON list behavior at Task 7 stage.
- [x] `D14`: `info` output no longer includes `time_precision` in JSON or human mode.
- [x] `D15`: Historical checkpoint: `signals` human mode defaults to short names at Task 9 stage (superseded by canonical `signal` in D28).
- [x] `D16`: Historical checkpoint: `signals --json` includes full signal paths deterministically at Task 9 stage (superseded naming in D28).
- [x] `D17`: Integration tests for commands in scope use fixture files from `/opt/rtl-artifacts`.
- [x] `D18`: Dockerfile contains dedicated fixture layer that downloads required artifacts pinned by version.
- [x] `D19`: Fixture layer uses `RTL_ARTIFACTS_VERSION` variable and keeps install path stable as `/opt/rtl-artifacts`.
- [x] `D20`: Both `ci` and `dev` image targets include `/opt/rtl-artifacts` fixture payload.
- [x] `D21`: `make ci` and `make pre-commit` pass inside container with fixture-backed tests and no runtime fixture download.
- [x] `D22`: Fixture provenance is pinned by Dockerfile version and stable artifact paths.
- [x] `D23`: Historical checkpoint (Tasks 1-11): PRD/changelog reflected first-cutover decisions (`tree` -> `modules`, no aliases at that stage, default human + `--json`, container-baked fixtures, container-only make workflow); superseded for command naming by `D27`-`D32`.
- [x] `D24`: Container-only guard is enforced: `make ci`/`make pre-commit` fail fast with clear message when container marker is absent.

### Final follow-up checkpoints (pending)
- [ ] `D25`: `signal` output `kind` exposes parser-native type aliases (for example `parameter` is no longer emitted as `unknown`) in both human and JSON modes.
- [ ] `D26`: `scope` output includes `kind` for every entry and includes non-module scope kinds when present in fixture data.
- [ ] `D27`: `scope` is canonical command in help/dispatch/output; `modules` invocation is rejected with `error: args:` and context help hint.
- [ ] `D28`: `signal` is canonical command in help/dispatch/output; `signals` invocation is rejected with `error: args:` and context help hint.
- [ ] `D29`: `change` is canonical command in help/dispatch/output; `changes` invocation is rejected with `error: args:` and context help hint.
- [ ] `D30`: `scope --tree` remains deterministic and readable with mixed `ScopeType` entries and preserved sort order.
- [ ] `D31`: JSON envelope reflects follow-up break: singular `command` values and incremented `schema_version`.
- [ ] `D32`: PRD/changelog and CLI contract tests explicitly document and lock the singular command cutover and kind-fidelity contract.

## Implementation Plan (Task Breakdown)

### Task 1: Lock contract deltas in docs first (~2-3h)
- Goal: Align source-of-truth docs with the validated product direction before code changes.
- Inputs: `.agents/PRD.md`, this plan, user validation findings, `CHANGELOG.md` policy.
- Known-unknowns: none blocking after decisions in this plan.
- Steps:
1. Update PRD sections for output default mode, command naming (`modules`), and `info` field set.
2. Add migration notes and user-visible changes under `## [Unreleased]` in `CHANGELOG.md`.
3. Record direct cutover policy (no command aliases, no output-default bridge, no `--human`) in docs.
- Outputs: Documentation baseline that matches the intended implementation.

### Task 2: Fix top-level invocation/help/version contracts (~2-3h)
- Goal: Correct no-args/help/version behavior and close test gaps from issues 1-3.
- Inputs: `src/cli/mod.rs`, `tests/cli_contract.rs`.
- Known-unknowns: clap pattern choice for printing help on empty invocation while preserving parse-error normalization.
- Steps:
1. Adjust CLI run path so empty invocation prints help and exits successfully.
2. Add integration tests for `-h`, `--help`, `-V`, `--version`, and no-args behavior.
3. Keep stdout/stderr split deterministic across these paths.
- Outputs: Stable top-level UX behavior locked by integration coverage.

### Task 3: Improve args-error diagnostics with help guidance (~2-3h)
- Goal: Make `args` errors actionable without violating the stable error-prefix format.
- Inputs: `src/cli/mod.rs` parse normalization path, `tests/cli_contract.rs`.
- Known-unknowns: none blocking; help-hint routing policy is fixed by this plan.
- Steps:
1. Rework parse error normalization to preserve required-argument details.
2. Append deterministic help hints with command context (`wavepeek --help` for global failures, `wavepeek <cmd> --help` for subcommand failures).
3. Add/extend tests for missing `--waves`, unknown flags, positional args, and invalid option combinations, covering both hint variants.
- Outputs: Context-rich `args` diagnostics with stable formatting and test coverage.

### Task 4: Reformat long help descriptions as multiline literals (~2h)
- Goal: Improve maintainability/readability of command help text definitions (issue 6).
- Inputs: `src/cli/mod.rs` subcommand metadata.
- Known-unknowns: final wrapping strategy that keeps help output concise.
- Steps:
1. Convert `long_about` strings to multiline raw strings per command.
2. Verify rendered help remains correct and no regressions in existing help tests.
3. Add focused assertions for key multiline content markers if needed.
- Outputs: Human-readable multiline help definitions in source with preserved runtime output quality.

### Task 5: Migrate output mode to human-default + explicit `--json` (~3-4h)
- Goal: Resolve issue 8 by flipping default output and making machine contract opt-in.
- Inputs: `src/cli/*` args, `src/engine/mod.rs`, `src/output.rs`, integration suites.
- Known-unknowns: none blocking; direct cutover policy is pre-decided in this plan.
- Steps:
1. Introduce output-mode flags such that default is human and `--json` enforces envelope mode.
2. Remove `--human` flag handling and keep output selection as default human vs explicit `--json`.
3. Update integration tests to assert default human output, explicit JSON envelope output, and rejected `--human` behavior across command surface.
4. Keep JSON schema contract deterministic and unchanged in `--json` mode.
- Outputs: Migrated output behavior with explicit tests for both user-facing and machine-facing modes.

### Task 6: Rename `tree` to `modules` and fix hierarchy semantics (~3-4h)
- Goal: Align command semantics/naming and close issue 9 with real fixture regression.
- Inputs: `src/cli/mod.rs`, `src/engine/mod.rs`, `src/engine/modules.rs`, `src/waveform/mod.rs`, tests.
- Known-unknowns: parser nuances that may expose signal-like leaves as hierarchy scopes in specific FST dumps.
- Steps:
1. Add canonical `modules` command and wire command dispatch/output naming.
2. Remove `tree` command surface and update help/dispatch/tests accordingly.
3. Reproduce issue 9 using fixture path under `/opt/rtl-artifacts` and add regression tests.
4. Adjust hierarchy traversal/filtering to guarantee module-instance output only.
- Outputs: Canonical `modules` command with corrected hierarchy listing and no alias compatibility paths.

### Task 7: Add optional human tree renderer (`--tree`) for modules (~2-3h)
- Goal: Provide a visual hierarchy mode in human output while keeping JSON flat.
- Inputs: command args for modules/tree, `src/output.rs`, integration tests.
- Known-unknowns: exact glyph/indent style that is readable and stable across terminals.
- Steps:
1. Add `--tree` flag for modules human mode.
2. Implement visual renderer with deterministic ordering/indentation.
3. Ensure JSON mode ignores visual rendering and remains list-based without extra warning noise.
4. Add tests for list mode vs tree mode and JSON interaction.
- Outputs: Optional visual hierarchy output without breaking machine contracts.

### Task 8: Remove `time_precision` from metadata/output surface (~2-3h)
- Goal: Eliminate redundant metadata field and align output/docs with issue 7.
- Inputs: `src/waveform/mod.rs`, `src/engine/info.rs`, `src/output.rs`, `tests/info_cli.rs`, PRD docs.
- Known-unknowns: whether any downstream tooling currently depends on `time_precision` key.
- Steps:
1. Remove `time_precision` from metadata structs and `info` command outputs.
2. Update human renderer and JSON assertions accordingly.
3. Update PRD examples/field tables and changelog notes for this contract change.
- Outputs: Simplified `info` schema and synchronized tests/docs.

### Task 9: Improve `signals` human output and add `--abs` (~2-3h)
- Goal: Make human signal listings concise by default while preserving full-path machine output.
- Inputs: `src/cli/signals.rs`, `src/engine/signals.rs`, `src/output.rs`, `tests/signals_cli.rs`.
- Known-unknowns: preferred human line format when width/kind metadata is present.
- Steps:
1. Add `--abs` flag to `signals` command.
2. Render human mode with short names by default; include full path only with `--abs`.
3. Keep JSON behavior unchanged (full path always present).
4. Add regression tests for both human variants and JSON invariants.
- Outputs: More concise human signal output with explicit absolute-path opt-in.

### Task 10: Provision fixtures in Docker image layer (~3-4h)
- Goal: Make realistic fixtures always available at stable runtime path without test-time download logic.
- Inputs: `.devcontainer/Dockerfile`, `.devcontainer/devcontainer.ci.json`, `.devcontainer/devcontainer.json`, release URL.
- Known-unknowns: none blocking; artifact names and source release are fixed.
- Steps:
1. Add dedicated Dockerfile layer/stage that downloads required artifacts using variable `RTL_ARTIFACTS_VERSION`.
2. Pin artifact release version in Dockerfile (`RTL_ARTIFACTS_VERSION`).
3. Install artifacts into stable path `/opt/rtl-artifacts` (no version in directory name).
4. Ensure both `ci` and `dev` targets include `/opt/rtl-artifacts` payload.
5. Add container marker env (for example `WAVEPEEK_IN_CONTAINER=1`) in devcontainer/CI configs and enforce Makefile guard for `make ci`/`make pre-commit`.
6. Update test paths to consume `/opt/rtl-artifacts/*` directly and remove runtime fixture download logic.
7. Document fixture provisioning behavior and version bump workflow in docs.
- Outputs: Fixture-backed tests run without network access during `cargo test`/`make` execution.

### Task 11: Final regression sweep and quality gates (~2-3h)
- Goal: Prove end-to-end correctness and repository-quality compliance.
- Inputs: all prior tasks, Makefile gates, updated docs/tests.
- Known-unknowns: none blocking after fixture-layer provisioning.
- Steps:
1. Run focused CLI integration suites for each corrected issue area.
2. Run `make ci` and `make pre-commit` inside container and fix regressions until green.
3. Assert fixture payload availability/version pin assumptions during CI-quality-gate execution.
4. Verify container guard behavior by running gate commands with container marker unset and confirming fail-fast diagnostics.
5. Re-check cutover behavior (no `tree`, no `--human`, default human + `--json`, help hints).
6. Confirm DoD checklist completion and record evidence links.
- Outputs: Green gates and release-ready corrective scope with explicit verification trail.

### Task 12: Full-fidelity kinds + all-scope listing with `scope` cutover (~3-4h)
- Goal: Preserve parser-native type information and widen hierarchy visibility while migrating `modules` to canonical `scope` command.
- Inputs: `src/waveform/mod.rs`, `src/engine/modules.rs` (or renamed `scope.rs`), `src/cli/modules.rs` (or renamed `scope.rs`), `src/output.rs`, fixture-backed tests.
- Known-unknowns: final alias vocabulary for `VarType`/`ScopeType` strings (resolved by explicit mapping table + tests).
- Steps:
1. Define explicit stable mapping for all `wellen::VarType` variants in the pinned dependency version to lowercase aliases and remove coarse `SignalKind` collapsing.
2. Define explicit stable mapping for all `wellen::ScopeType` variants in the pinned dependency version and include `kind` in scope entry model/output.
3. Rework hierarchy traversal to emit all scopes (not only modules) with deterministic ordering and clear depth semantics.
4. Rename command surface from `modules` to `scope` across CLI dispatch, engine command naming, and output envelope `command` field.
5. Update/extend tests for mixed scope kinds, deterministic ordering, `--tree` rendering, and kind-fidelity assertions in human/JSON modes.
6. Increment JSON `schema_version` and lock migration behavior in contract tests.
- Outputs: Canonical `scope` contract with all-scope visibility and faithful `kind` metadata.

### Task 13: Singularize `signal`/`change` command surface and docs (~2-3h)
- Goal: Complete singular naming consistency for command surface and user-facing contracts.
- Inputs: `src/cli/mod.rs`, `src/cli/signals.rs`, `src/cli/changes.rs`, `src/engine/mod.rs`, `tests/cli_contract.rs`, `tests/signals_cli.rs`, docs.
- Known-unknowns: none blocking; direct cutover without aliases is pre-approved.
- Steps:
1. Rename CLI command nodes and dispatch wiring: `signals` -> `signal`, `changes` -> `change`.
2. Update parse-error contextual hints, help text examples, and unimplemented-command messages to singular forms.
3. Update JSON envelope command-name assertions and integration tests to singular values.
4. Add explicit rejection tests for removed plural commands with stable `error: args:` + help hints.
5. Update PRD and changelog migration notes with the singular command cutover and no-alias policy.
- Outputs: Consistent singular CLI surface with locked tests and synchronized documentation.

## Execution Log (Living Doc)

### 2026-02-18 - Task 1 (docs baseline)
- Completed PRD contract cutover updates:
  - `tree` renamed to `modules` in command contract sections.
  - Default mode documented as human for implemented scope (`info`, `modules`, `signals`) with explicit `--json` for strict envelope mode.
  - `info` output fields updated to remove `time_precision`.
  - External fixture provisioning model documented as container-baked payload at `/opt/rtl-artifacts`.
  - Container-only quality gate rule documented for `make ci` and `make pre-commit`.
- Completed changelog migration notes under `## [Unreleased]` to reflect direct cutover policy and user-visible behavior changes.
- Decision update: output contract (`default human` + `--json`) is now applied across all command docs.

### 2026-02-18 - Tasks 2-4 (CLI entry/help/errors/help text)
- `src/cli/mod.rs` updated to print top-level help on empty invocation and exit `0`.
- Added deterministic clap error normalization with contextual hints (`wavepeek --help` vs `wavepeek <cmd> --help`) while preserving missing-argument detail lines.
- Converted command `long_about` values to multiline raw literals.
- Coverage added/updated in `tests/cli_contract.rs` for no-args behavior, `-h`/`--help`, `-V`/`--version`, and help-hint assertions.

### 2026-02-18 - Tasks 5-9 (output migration + command surface changes)
- Removed `--human` from CLI args and applied explicit `--json` switch semantics across command surface.
- Renamed command surface from `tree` to `modules` (no alias), including dispatch and output command key.
- Added `modules --tree` visual renderer; confirmed `--json --tree` keeps flat JSON list with no extra warning.
- Removed `time_precision` from waveform metadata model, info engine/output, tests, and PRD/changelog contract text.
- Added `signals --abs`; human default now uses short names, JSON keeps full canonical paths.
- Updated hierarchy traversal to emit module scopes only and keep deterministic depth-first ordering.

### 2026-02-18 - Task 10 (fixture provisioning + container model)
- Added dedicated Docker fixture stage in `.devcontainer/Dockerfile` keyed by `RTL_ARTIFACTS_VERSION`.
- Added pinned downloads for `picorv32_test_vcd.fst` and `scr1_max_axi_coremark.fst` via `RTL_ARTIFACTS_VERSION`.
- Copied `/opt/rtl-artifacts` payload through shared `base` stage so both `ci` and `dev` inherit it.
- Added `WAVEPEEK_IN_CONTAINER=1` to both devcontainer configs and documented fixture workflow/version-bump process in `.devcontainer/AGENTS.md`.
- Added Makefile fixture checks for required files and simplified version-pinned provisioning workflow.

### 2026-02-18 - Task 11 (regression sweep + gates)
- Added external-fixture integration coverage in:
  - `tests/info_cli.rs` (`picorv32_test_vcd.fst`)
  - `tests/modules_cli.rs` (`scr1_max_axi_coremark.fst` regression semantics)
  - `tests/signals_cli.rs` (`picorv32_test_vcd.fst` short-name human output)
- Validation evidence:
  - `cargo test` passed with `WAVEPEEK_RTL_ARTIFACTS_DIR=/tmp/rtl-artifacts`.
  - `make ci` passed with `WAVEPEEK_IN_CONTAINER=1 RTL_ARTIFACTS_DIR=/tmp/rtl-artifacts WAVEPEEK_RTL_ARTIFACTS_DIR=/tmp/rtl-artifacts`.
  - `make pre-commit` passed with the same env override.
  - `make ci`/`make pre-commit` fail-fast confirmed when container marker is unset.

### 2026-02-18 - Post-implementation user-directed adjustments
- Removed `RTL_ARTIFACTS_VERSION` overrides from `.devcontainer/devcontainer.json` and `.devcontainer/devcontainer.ci.json`; version pin now lives only in `.devcontainer/Dockerfile` (same style as `SURFER_VERSION`).
- Dropped checksum/manifest workflow: removed Dockerfile SHA checks, removed manifest generation, and simplified Makefile fixture checks to required file presence.
- Expanded output-contract cutover to all command docs/CLI flags: default human + explicit `--json`, no `--human` surface.
- Strengthened container enforcement model by attaching `require-container` to leaf Makefile command targets (not only aggregate gates).
- Removed explicit wording about "human/human-readable mode" from CLI help text; help now focuses on behavior and `--json` strict-mode switch.

### 2026-02-18 - Plan follow-up extension (Tasks 12-13)
- Added follow-up requirements `VF-R23`-`VF-R29` for kind fidelity, all-scope visibility, singular command cutover, and schema-version bump.
- Added two new implementation tasks:
  - Task 12: full-fidelity `VarType`/`ScopeType`, all-scope traversal, `modules` -> `scope` cutover.
  - Task 13: singular command cutover for `signals`/`changes` -> `signal`/`change`, including docs/tests.
- Extended DoD with pending checks `D25`-`D32` and updated traceability matrix links.
- This update is planning-only; no implementation evidence is recorded yet for Tasks 12-13.
