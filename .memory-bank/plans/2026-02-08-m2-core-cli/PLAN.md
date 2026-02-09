# M2 Core CLI Delivery Plan (v0.2.0)

## Summary
- This plan delivers PRD M2 by implementing the first production command set: `info`, `tree`, and `signals`, with deterministic default JSON output and `--human` mode.
- The implementation is organized around shared foundations first (dispatch, output, errors, waveform adapter), then command-by-command delivery with test coverage.
- The outcome is a usable core CLI for waveform discovery on VCD/FST dumps, with stable machine-readable contracts and predictable failure behavior for automation.

## Goals
- Deliver fully functional `info`, `tree`, and `signals` commands per PRD sections 3.2.1-3.2.3.
- Support both VCD and FST input formats in the same command paths.
- Enforce default JSON envelope contract (`schema_version`, `command`, `data`, `warnings`) and `--human` output mode.
- Standardize error categories and exit codes per PRD 5.6 (`1` user/args/signal errors, `2` file/parse errors).
- Add hand-crafted waveform fixtures and integration tests that verify deterministic output, warnings, and error formatting.

## Non-Goals
- Implementing `at`, `changes`, `when`, or full expression engine behavior (M3/M4 scope).
- Implementing `schema` command export behavior (M5 scope).
- Adding waveform caching, daemon/session state, or background services.
- Introducing warning code taxonomy beyond free-form warning strings.

## Background / Context
- Repository status is M1 scaffold: CLI args are defined, but engine and waveform layers return `Unimplemented` placeholders.
- Core architecture and module boundaries already exist (`cli`, `engine`, `waveform`, `output`, `error`) and should be preserved.
- PRD requires deterministic and bounded output by default, strict stderr error format, and stateless single-command execution.
- Existing integration tests currently validate CLI contract only (`--help`, required flags, positional rejection), not command behavior.

## Problem Statement
- Without M2 implementation, wavepeek cannot inspect dumps, so LLM/automation workflows still have no machine-friendly waveform access.
- Placeholder execution paths hide important contracts (data schema, warnings, error categories), making downstream tooling impossible to stabilize.
- Missing fixtures and behavioral tests create high risk of regressions in determinism and output compatibility as the CLI grows.

## Requirements
- **Functional:**
- `M2-R1`: Implement waveform opening/parsing for VCD and FST in `src/waveform/mod.rs`.
- `M2-R2`: Implement `info` metadata extraction and output fields (`time_unit`, `time_precision`, `time_start`, `time_end`).
- `M2-R3`: Implement `tree` traversal with deterministic ordering, `--max`, `--max-depth`, and `--filter` behavior.
- `M2-R4`: Implement `signals` listing for exact scope path with alphabetical sorting, `--max`, and `--filter`.
- `M2-R5`: For implemented commands, emit default JSON envelope with `schema_version = 1` and command-specific `data`.
- `M2-R6`: For implemented commands, support `--human` mode with readable output and warnings on stderr.
- `M2-R7`: Enforce typed errors and exit codes with stderr format `error: <category>: <message>`.
- `M2-R8`: Add hand-crafted fixtures and `assert_cmd` integration tests for success, warning, and error paths.
- `M2-R13`: Normalize CLI parse/validation failures (excluding `--help`/`--version`) into `args` category with the same stderr format.
- **Non-functional / constraints:**
- `M2-R9`: Preserve deterministic output for identical input (ordering and formatting stability).
- `M2-R10`: Keep outputs bounded by defaults and emit warnings when truncation occurs.
- `M2-R11`: Avoid panics in production paths (`Result`-based failures only).
- `M2-R12`: Keep implementation aligned with Makefile/pre-commit quality gates.

## Traceability Matrix
| Scope item | Requirement IDs | Tasks | DoD checks | Primary artifacts |
|---|---|---|---|---|
| Dispatch/output/error foundation | M2-R5, M2-R6, M2-R7, M2-R11, M2-R13 | Task 1, Task 2, Task 3 | D1, D2, D9, D10, D15 | `src/cli/mod.rs`, `src/engine/mod.rs`, `src/output.rs`, `src/error.rs`, `src/main.rs` |
| Waveform VCD/FST adapter | M2-R1, M2-R11 | Task 4 | D3, D4, D16 | `src/waveform/mod.rs` |
| `info` command | M2-R2, M2-R5, M2-R6, M2-R7 | Task 5 | D5, D11 | `src/engine/info.rs`, integration tests |
| `tree` command | M2-R3, M2-R5, M2-R6, M2-R9, M2-R10 | Task 6 | D6, D7, D11 | `src/engine/tree.rs`, integration tests |
| `signals` command | M2-R4, M2-R5, M2-R6, M2-R9, M2-R10 | Task 7 | D8, D11, D16 | `src/engine/signals.rs`, integration tests |
| Fixtures + quality gates | M2-R8, M2-R12 | Task 8 | D12, D13, D14 | `tests/fixtures/`, `tests/`, `Makefile` commands |

## Proposed Solution
- Refactor command dispatch so engine handlers receive typed CLI args instead of only `CommandKind`, enabling command-specific execution and output.
- Build shared output path for JSON envelope emission and human renderer selection, with warning routing rules centralized.
- Expand `WavepeekError` coverage for runtime failure modes (file open/parse, invalid regex, scope/signal lookup) while preserving stable category-based formatting.
- Implement a thin waveform adapter over `wellen` that exposes metadata, deterministic hierarchy traversal primitives, and scope-local signal lookup.
- Deliver commands incrementally (`info` -> `tree` -> `signals`) with integration tests added in lockstep, then close with fixture coverage and quality-gate runs.

## Alternatives Considered
- **Implement commands first, then shared foundations:** rejected because it duplicates output/error logic and increases drift risk.
- **Expose `wellen` types directly to engine:** rejected to avoid leaking dependency-specific shapes into stable CLI contracts.
- **Delay FST validation to M3:** rejected because M2 explicitly commits to VCD+FST support.

## Risks and Mitigations
- **Risk:** Path canonicalization for escaped identifiers differs across formats. **Mitigation:** define one canonical emitted path policy in waveform adapter and lock with cross-format tests.
- **Risk:** `signals.kind` mapping is inconsistent or lossy. **Mitigation:** define stable enum mapping for M2, map unknown/unsupported cases to explicit fallback value.
- **Risk:** Regex failures are surfaced as opaque parser errors. **Mitigation:** normalize regex compile failures to `args` category with the original pattern context.
- **Risk:** Truncation/no-results warnings diverge between commands. **Mitigation:** centralize warning helper text patterns and assert them in integration tests.
- **Risk:** Human output changes destabilize tests. **Mitigation:** keep exact assertions on JSON mode only; use smoke assertions for human mode as PRD allows.

## Rollout / Migration Plan
- Phase 1: foundation (dispatch/output/error contracts).
- Phase 2: waveform adapter and `info` vertical slice.
- Phase 3: `tree` and `signals` implementation.
- Phase 4: fixture expansion, cross-format verification, and full quality gates.
- Rollback: if instability appears during implementation, revert the latest M2 merge unit and keep release candidates gated until all DoD checks are green (no released branch fallback to `Unimplemented` for M2 commands).

## Observability
- Integration suite reports command-level contract coverage (`info`, `tree`, `signals`) for both formats.
- CI/local logs include explicit checks for error category prefix and exit code expectations.
- Determinism is validated by repeated-command golden comparisons in integration tests.
- Warning behavior is observable via assertions on JSON `warnings` and human-mode stderr smoke checks.

## Open Questions (Deferred to M3+)
- Whether warning strings remain free-form in M2 or start introducing stable warning codes.
- Should `signals.kind` vocabulary be expanded in M3+ beyond the M2 baseline mapping.

## Assumptions
- In M2 roadmap text, "all commands" means all commands implemented in M2 (`info`, `tree`, `signals`), not M3/M4/M5 commands.
- `signals` metadata remains the PRD MVP shape: `name`, `path`, `kind`, optional `width`; no extra metadata fields in M2.
- Canonical path policy in M2 is dot-separated full path; escaped identifiers are preserved verbatim from parser output (no extra normalization pass).
- `signals.kind` uses a frozen M2 enum mapping with explicit fallback value `unknown` for unmapped parser kinds.
- Warnings remain free-form strings in JSON envelope for M2.
- A small committed FST fixture is acceptable for M2 integration tests; `make fixtures` automation stays in M3.

## Definition of Done
- Sign-off gate policy: `make ci` is the authoritative quality gate; `make pre-commit` is a parity check with local hooks and must also pass before merge.
- [ ] `D1`: `src/cli/mod.rs` dispatch passes typed args and no M2 subcommand returns `Unimplemented` in normal success paths.
- [ ] `D2`: JSON envelope (`schema_version`, `command`, `data`, `warnings`) is emitted for `info`, `tree`, and `signals` in default mode.
- [ ] `D3`: `Waveform::open` successfully reads at least one VCD fixture and one FST fixture.
- [ ] `D4`: File open/parse failures return `error: file: ...` with exit code `2`.
- [ ] `D5`: `wavepeek info --waves <fixture>` returns required metadata keys and normalized time strings.
- [ ] `D6`: `wavepeek tree` output is deterministic DFS + lexicographic child order, respects `--max-depth`, and emits truncation warning when bounded.
- [ ] `D7`: `wavepeek tree --filter <invalid regex>` fails with `error: args: ...` and exit code `1`.
- [ ] `D8`: `wavepeek signals` enforces exact scope lookup, returns sorted non-recursive signals, and emits truncation warning when bounded.
- [ ] `D9`: `--human` mode works for `info`, `tree`, and `signals`; warnings are printed to stderr instead of JSON envelope.
- [ ] `D10`: Error formatting is stable for runtime args/file/scope-or-signal failures and stdout is empty on errors.
- [ ] `D11`: Integration tests verify both VCD and FST behavior for at least one query per implemented command.
- [ ] `D12`: Hand-crafted fixtures exist under `tests/fixtures/hand/` and are used by tests.
- [ ] `D13`: `make ci` exits `0` on the M2 branch.
- [ ] `D14`: `make pre-commit` exits `0` on the M2 branch.
- [ ] `D15`: CLI parse/validation failures (for example unknown flag or forbidden `--waves` on `schema`) are emitted as `error: args: ...` with exit code `1`.
- [ ] `D16`: Tests lock canonical path emission and `signals.kind` fallback mapping (`unknown`) across VCD and FST fixtures.

## Implementation Plan (Task Breakdown)

### Task 1: Refactor command dispatch contracts (~3h)
- Goal: Make engine execution paths command-aware by passing typed arguments from CLI to command handlers.
- Inputs: `src/cli/mod.rs`, `src/engine/mod.rs`, current command arg structs in `src/cli/{info,tree,signals}.rs`.
- Known-unknowns: Preferred location for shared M2 command result types (`engine/mod.rs` vs per-command modules).
- Steps:
1. Replace `CommandKind`-only execution with per-command dispatch that forwards parsed args.
2. Update engine entrypoints for `info`, `tree`, and `signals` to accept typed args and return structured results.
3. Keep non-M2 commands explicitly unchanged in behavior to avoid accidental scope expansion.
4. Add/adjust tests to ensure dispatch routes `--human` and command args correctly.
- Outputs: Typed dispatch pipeline ready for command implementations.

### Task 2: Implement shared output pipeline (~3h)
- Goal: Centralize JSON envelope serialization and human-mode rendering behavior for implemented commands.
- Inputs: `src/output.rs`, `src/main.rs`, Task 1 dispatch outputs, PRD 3.2 general conventions.
- Known-unknowns: Final human output text layout per command (not strict contract).
- Steps:
1. Extend `src/output.rs` with helpers for envelope emission and warning routing.
2. Add command-specific human renderers for `info`, `tree`, and `signals`.
3. Ensure default mode always writes one JSON object to stdout and includes `schema_version = 1`.
4. Add tests for envelope shape and warning behavior in JSON mode.
- Outputs: Reusable output layer with PRD-compliant JSON default behavior.

### Task 3: Expand error taxonomy and validation surfaces (~3h)
- Goal: Make runtime failures PRD-compliant in category, message format, and exit code mapping.
- Inputs: `src/error.rs`, `src/main.rs`, PRD 5.6.
- Known-unknowns: Exact category naming for scope-not-found vs signal-not-found in M2.
- Steps:
1. Add error variants needed by M2 runtime paths (regex compile, missing scope/signal, parse/open failures).
2. Preserve/verify exit code mapping (`1` user-domain errors, `2` file-domain errors).
3. Normalize semantic validation failures (`--max`, `--max-depth`, regex) to `args` category.
4. Switch to `Cli::try_parse`-based error handling and map clap parse failures to `WavepeekError::Args` while preserving normal `--help`/`--version` behavior.
5. Add unit/integration assertions for stderr format, exit codes, and empty stdout on errors.
- Outputs: Stable typed error model for M2 command execution.

### Task 4: Implement waveform adapter over wellen (~4h)
- Goal: Provide format-agnostic primitives for metadata, hierarchy traversal, and scope-local signal access.
- Inputs: `src/waveform/mod.rs`, `wellen` docs/APIs, PRD 5.2/5.3.
- Known-unknowns: Minimal adapter API that supports M2 now without constraining M3 time-value APIs.
- Steps:
1. Implement `Waveform::open` with file existence/open/parse error mapping.
2. Add adapter methods for dump metadata (`time_unit`, `time_precision`, bounds).
3. Add deterministic hierarchy and signal query helpers used by `tree` and `signals`.
4. Define canonical path emission policy and document it in module-level docs/tests.
- Outputs: Working VCD/FST adapter with deterministic query primitives.

### Task 5: Implement `info` command end-to-end (~3h)
- Goal: Deliver metadata query behavior for both output modes and both formats.
- Inputs: Task 1-4 outputs, `src/engine/info.rs`, `src/cli/info.rs`.
- Known-unknowns: Exact time normalization representation details from adapter for edge fixtures.
- Steps:
1. Implement engine logic to load waveform and construct required metadata response.
2. Wire `info` command to shared output path (JSON + `--human`).
3. Add integration tests for success JSON contract on VCD and FST fixtures.
4. Add error-path tests for missing/invalid waveform file.
- Outputs: PRD-compliant `info` command with deterministic outputs.

### Task 6: Implement `tree` command end-to-end (~4h)
- Goal: Deliver deterministic hierarchy listing with filter/depth/max controls and warnings.
- Inputs: Task 1-4 outputs, `src/engine/tree.rs`, `src/cli/tree.rs`.
- Known-unknowns: Root scope naming nuances across VCD/FST input sources.
- Steps:
1. Implement DFS pre-order traversal with lexicographic child ordering and depth tracking.
2. Apply `--max-depth` and regex filter behavior on full path values.
3. Apply `--max` truncation and emit warning through shared warning path.
4. Add integration tests for ordering determinism, truncation warning, and invalid regex errors.
- Outputs: PRD-compliant `tree` command with bounded deterministic output.

### Task 7: Implement `signals` command end-to-end (~4h)
- Goal: Deliver scope-local signal listing with stable metadata shape and deterministic ordering.
- Inputs: Task 1-4 outputs, `src/engine/signals.rs`, `src/cli/signals.rs`.
- Known-unknowns: Whether `wellen` surfaces parser kinds that should be promoted from `unknown` in future milestones.
- Steps:
1. Implement exact scope resolution and non-recursive signal enumeration.
2. Map signal metadata to stable output fields (`name`, `path`, `kind`, optional `width`).
3. Apply alphabetical ordering by name, regex filtering, and `--max` truncation warnings.
4. Add integration tests for scope-not-found errors, ordering, filter behavior, and truncation.
- Outputs: PRD-compliant `signals` command for VCD/FST inputs.

### Task 8: Add fixtures, integration matrix, and quality-gate closure (~3h)
- Goal: Lock behavior with realistic tests and complete repository quality gates.
- Inputs: `tests/`, `tests/fixtures/`, Makefile targets, outputs of Tasks 5-7.
- Known-unknowns: Best small FST fixture source that is stable in-repo for CI.
- Steps:
1. Fill fixture gaps not already covered in Tasks 5-7 (especially canonical path and `signals.kind` fallback cases).
2. Add at least one FST fixture under `tests/fixtures/` for cross-format parity checks if not already added earlier.
3. Complete integration matrix checks across `info`, `tree`, and `signals` (JSON contract, warnings, and error categories).
4. Run `make ci` and `make pre-commit`, fixing failures until both pass.
- Outputs: M2 behavior covered by deterministic integration tests and green quality gates.

## Execution Notes

### 2026-02-09 (autonomous run)
- Task 1 completed.
  - Decision: replaced `CommandKind` with typed `engine::Command` variants so CLI forwards full subcommand args (`--human`, `--max`, `--filter`, etc.) into engine handlers.
  - Decision: threaded typed args through non-M2 stub handlers (`schema`, `at`, `changes`, `when`) as ignored parameters to keep behavior unchanged while avoiding dead-code warnings.
  - Added unit tests in `src/cli/mod.rs` to lock dispatch argument forwarding for `info`, `tree`, and `signals`.
  - Validation: `cargo test` passed.
- Task 2 completed.
  - Added shared `CommandResult`/`CommandData` engine response model and wired CLI dispatch to a shared output writer.
  - Implemented centralized JSON envelope emission in `src/output.rs` with fixed `schema_version = 1` and warning passthrough.
  - Implemented initial human renderers for `info`, `tree`, and `signals`, including stderr warning routing in human mode.
  - Added JSON-envelope unit tests for shape and warning preservation; validation via `cargo test` passed.
- Task 3 completed.
  - Expanded `WavepeekError` with explicit `scope` and `signal` categories and added unit tests to lock formatting plus exit-code mapping.
  - Switched CLI parsing to `Cli::try_parse()` and normalized clap failures to `error: args: ...` while preserving standard `--help` and `--version` behavior.
  - Added clap error normalization helper that strips usage/noise into a stable one-line args message.
  - Updated integration tests to assert args-category stderr prefix and empty stdout for parse/validation failures.
  - Validation: `cargo test` passed.
- Task 4 completed.
  - Implemented `src/waveform/mod.rs` adapter over `wellen::simple::Waveform` with file-open/parse error mapping into `error: file: ...`.
  - Added metadata extraction (`time_unit`, `time_precision`, normalized `time_start`/`time_end`) plus deterministic DFS scope traversal and scope-local signal lookup helpers.
  - Introduced canonical path policy docs in the module and encoded signal-kind mapping with explicit `unknown` fallback.
  - Added focused waveform adapter tests for metadata, DFS order, scope errors, missing-file/parse-file errors, and unknown kind fallback behavior.
  - Surprise: adapter is introduced ahead of command wiring, so temporary `dead_code` allowance is kept in this module until Tasks 5-7 consume it.
  - Validation: `cargo test` passed.
