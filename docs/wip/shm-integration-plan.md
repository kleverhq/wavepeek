# Integrate Ondas SHM input

This ExecPlan follows the `exec-plan` skill. Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective current during implementation.

## Purpose / Big Picture

Enable native waveform commands to inspect a complete Cadence SHM database through the existing Ondas adapter. A locally built binary should accept `--waves waveform.shm` for metadata, hierarchy, values, changes, properties, and extraction. Existing output contracts and deterministic ordering apply.

SHM is a directory containing design (`.dsn`) and transition (`.trn`) components. Ondas reads that directory directly; conversion is used only to prepare test and benchmark inputs.

## Non-Goals

This work covers the native CLI and optional local Cadence integration. Browser SHM input, bundled Cadence libraries, and another SHM parser are outside its scope.

## Progress

- [x] (2026-10-16) Create `/home/esynr3z/projects/wavepeek-shm`, branch `feat/shm`, from `47c5ec5` and install worktree-specific hooks.
- [x] (2026-10-16) Inspect the waveform adapter, FSDB integration, fixtures, quality gates, benchmark capture, Ondas 1.1.0, and Xcelium conversion documentation.
- [x] (2026-10-16) Verify VCD-to-SHM conversion and direct reader launch with Xcelium 25.03.001 in a private devcontainer.
- [x] (2026-10-16) Integrate optional SHM input, path normalization, and disabled-feature diagnostics.
- [x] (2026-10-16) Add fixture preparation, 12 SHM command tests, disabled-feature tests, helper tests, and conditional quality gates.
- [x] (2026-10-16) Add the 156-case SHM catalog, focused E2E runner, and conditional smoke recipe.
- [x] (2026-10-16) Measure all 156 SHM cases and 144 FSDB cases; confirm the 12 FSDB failures match existing exclusions, and compare all 156 FST functional captures.
- [x] (2026-10-16) Update user/maintainer documentation; pass `just ci` with SDKs and with both SDK variables unset, plus combined `fsdb,shm` library/CLI tests.
- [x] (2026-10-16) Pass normal host hooks for `e317ec1`, including both native benchmark smokes, and pass `just check`; record the final benchmark report.

## Surprises & Discoveries

Ondas is already pinned to `=1.1.0` in `Cargo.toml`. That release includes `shm-lib`; no dependency update is needed. Both the local clean Ondas checkout and GitHub's `v1.1.0` Cargo manifest confirm the feature.

The Ondas reader requires Linux x86_64 GNU, `XCELIUM_HOME`, a compatible Xcelium reader from 2021 or later, a C++17 compiler, binutils, and zlib. It selects `tools/lib/64bit/libsst2.so`. Readers with an ELF SONAME, the library name used by the dynamic loader, need runtime lookup configured separately. A standalone binary launch must be tested because Cargo can supply a different loader environment.

SHM carries bits, real values, strings, and event occurrences. FSDB's real/string restrictions must not be applied to SHM. Ondas does not expose SHM signedness or enumeration labels. Conversion can change declaration metadata and paths; compare observable behavior and record any justified parity exceptions explicitly.

The new worktree initially inherited the main worktree's hooks path. Its local setting was corrected, then `./dev --install-hooks` succeeded. Main's hook configuration was preserved. The private container now mounts Cadence and Verdi; converter licensing is configured only in its ignored profile.

## Decision Log

Use Cargo feature `shm = ["ondas/shm-lib"]`, independently selectable with `fsdb`. Default builds keep their existing dependency requirements. This follows the current optional native backend contract. Date: 2026-10-16.

Generate ignored SHM inputs from project-owned VCD fixtures. Run SDK-dependent tests only after an environment probe succeeds; an explicitly configured but broken installation must fail. Default tests cover disabled support without Cadence. This provides meaningful adapter coverage while keeping public CI usable. Date: 2026-10-16.

Prepare benchmark inputs through FST-to-VCD-to-SHM conversion, reusing valid existing VCD artifacts when available. Use the current FST workload catalog as the source of SHM cases. Conversion stays outside timed queries. Date: 2026-10-16.

## Outcomes & Retrospective

Native integration and all regression suites are implemented. `just ci` passed with SDKs and without their environment variables; combined `fsdb,shm` tests passed. Source coverage: 93.01% lines, 91.85% regions, 91.66% functions. SHM completed 156/156 benchmark scenarios; FSDB completed 144/156 with only its 12 existing scalar-array exclusions. The 134 identical-payload pairs have a median SHM/FSDB ratio of 1.776. FST reference captures account for eight SHM differences in declaration kinds and initial bounds/rows. Normal host hooks and `just check` passed. The implementation is committed as `e317ec1`; `docs/wip/shm-benchmark-report.md` records the measured performance, exclusions, and functional differences. No additional optimization or multi-ref benchmark integration is planned in this slice. GitHub's open issues and milestones were inspected; no SHM integration issue was present. No issue or pull request was created.

## Context and Orientation

`src/waveform/mod.rs` is the engine-facing facade. `src/waveform/ondas_backend.rs` opens inputs, adapts hierarchy paths and declaration kinds, resolves signals, samples values, and caches traces. Its `format_name` lacks SHM, and several direct-lookup and preparation branches currently name only VCD/FST/FSDB. Audit those branches individually rather than assigning SHM another format's behavior.

`src/waveform/fsdb_disabled.rs` supplies actionable guidance for feature-disabled FSDB input. `src/cli/mod.rs` declares supported formats and help text; command argument structs under `src/cli/` also describe waveform paths. `src/browser.rs` uses shared bytes, which SHM does not support.

`tools/fsdb/`, `tests/fsdb_cli.rs`, and `src/waveform/ondas_fsdb_tests.rs` show optional SDK validation, conversion, and command parity patterns. `tools/waveform/prepare_fixtures.py` builds source-backed VCD/FST inputs. Generated SHM directories should live under ignored `tests/fixtures/shm/`; update `tests/waveform_fixture_policy.rs` to enforce the generated-artifact policy.

`bench/e2e/tests.json` is the FST workload catalog. `tools/fsdb/generate_bench_catalog.py` already derives VCD/FSDB catalogs. `tools/bench/capture.py`, `tools/bench/gate.py`, and `tools/bench/compare.py` build selected refs, capture runs, and compare timing and functional output. Root `justfile` owns development entrypoints and conditional gates.

Read applicable `AGENTS.md` files before edits. Run Git on the host and development tools through root `./dev`. Scratch and logs belong under this worktree's `tmp/`. The ignored worktree-local private profile mounts Cadence, Verdi, and the local converter license.

## Open Questions

The first prototype must determine which available Xcelium installation and private container mount provide a usable reader and converter. Inspect its runtime dependencies and loader behavior; do not expose the whole vendor library directory if that selects an incompatible bundled C++ runtime.

Verify converted names, declaration kinds, end time, raw events, real values, and strings before defining exact parity expectations. Verify how converter options preserve same-tick updates. Choose a small native simulator-generated case only if conversion cannot express an essential contract.

## Plan of Work

### Milestone 1: Prove reader and converter availability

Configure an ignored private devcontainer with `XCELIUM_HOME` and a read-only installation mount. Add `tools/shm/check_shm_env.py` and a `check-shm-env` recipe that distinguish absent, available, and broken installations, following the existing FSDB probe. Reader availability and converter availability are separate requirements: builds need the reader; fixture preparation also needs `simvisdbutil`.

Use a minimal existing VCD fixture to verify the proposed invocation `simvisdbutil -64bit -nocopyright -nolog -shm -output <output.shm> <input.vcd>` in a task-owned scratch directory. Confirm the actual output directory and DSN/TRN layout before encoding the command in helpers. Load it with Ondas and launch the built consumer outside Cargo with the intended runtime environment. Keep any prototype under ignored `tmp/`; promote only the validated helper behavior.

### Milestone 2: Integrate native SHM commands

Add the Cargo feature and SHM format reporting. Adapt `OndasBackend` paths, declaration kinds, exact and batched resolution, sampling, and candidate-time handling as evidence requires. Preserve format-specific handling for arrays, escaped names, real/string expressions, and events. Add actionable feature-disabled guidance while preserving ordinary missing-path, permission, and malformed-input errors. Test directory detection with and without `.shm`, individual component rejection, and clean stdout/stderr for text, JSON, and JSONL.

Acceptance is successful `info`, `scope`, `signal`, `value`, `change`, `property`, and extraction queries against the converted fixture with stable output and existing error categories. Build the combined `fsdb,shm` feature configuration when both installations are available.

### Milestone 3: Establish regression coverage

Add `tools/shm/prepare_shm_fixtures.py`, adjacent helper tests and README, and proposed `prepare-shm-test-fixtures` and `test-shm` recipes. Convert a selected small fixture set once during preparation, outside individual tests. Verify complete output directories and cache validity so interrupted conversion cannot leave reusable partial fixtures. Keep vendor payloads and generated databases outside Git.

Add `tests/shm_cli.rs` and focused adapter tests for hierarchy ordering and depth, escaped/indexed paths, projections and duplicates, X/Z values, exact transitions, absent initial state where conversion preserves it, same-tick final state, trigger evaluation, properties, extraction, and repeated queries. Exercise rich expression values only where the fixture and reader preserve them. Add SDK-free disabled-feature tests and tooling unit tests for skip/fail states and directory validation. Reuse shared assertions where useful without restructuring unrelated suites.

Wire conditional SHM lint, build, tests, and benchmark smoke into the corresponding `justfile` gates. Public CI must pass without Cadence. A configured broken SDK or a failed SHM command must fail the appropriate gate. Keep source coverage policy intact.

### Milestone 4: Add benchmark parity and documentation

Extend catalog generation to write `bench/e2e/tests_shm.json`. Map repository-generated inputs to `tests/fixtures/shm/` and representative Ondas inputs to neighboring `waveform.shm` directories. Add proposed `prepare-shm-fixtures`, `bench-e2e-shm-smoke-commit`, and focused SHM run recipes. Validate DSN/TRN directories instead of assuming every waveform is a regular file. Preserve filtered preparation for pre-commit smoke.

Use the existing E2E run/report harness to compare SHM and FSDB on identical named workloads, run counts, and warmups. Record functional differences separately from timings. Do not add SHM to multi-ref capture/gate helpers in this slice: the user requested KISS/YAGNI and an immediate format comparison. Measure larger fixtures before deciding whether different trace caching or preparation is necessary.

Update README, CLI help, relevant packaged skill references, `docs/environment.md`, `docs/testing.md`, `docs/quality.md`, and `docs/benchmarking.md`; add `docs/shm.md` for the installation, runtime, fixture, and support contract. Follow local changelog guidance when implementation is ready.

## Concrete Steps

Run commands from `/home/esynr3z/projects/wavepeek-shm`. The implemented SHM entrypoints are:

    ./dev just check-shm-env
    ./dev just prepare-shm-test-fixtures
    ./dev just test-shm
    ./dev just bench-e2e-shm-smoke-commit
    ./dev just check
    ./dev just ci

Use an explicit private `WAVEPEEK_DEV_CONFIG` for SDK runs and the public configuration to verify Cadence-free behavior. Start the selected container before any host commit; installed hooks do not start containers. Run focused helper and Rust tests during iteration, then the full gates once the feature and documentation are complete.

## Validation and Acceptance

`wavepeek info --waves <fixture.shm> --json` reports the source timescale and span. Hierarchy and signal queries produce deterministic paths. Value, change, property, and extraction results match equivalent VCD/FST fixture payloads after removing input-path fields and only explicitly justified format metadata differences. Truncated or incomplete directories produce file errors without native messages corrupting stdout.

A default binary provides clear `--features shm` and `XCELIUM_HOME` guidance for recognized SHM input. Valid VCD/FST behavior remains covered by the existing suites. SHM-enabled and combined-feature binaries must launch directly with their documented loader setup. Public `just check` and `just ci` pass without a vendor installation; private gates execute SHM regressions rather than silently skipping them. Benchmark preparation completes before timing and reports all omitted cases explicitly.

## Idempotence and Recovery

Generate each SHM database in a task-owned staging directory, validate it, then publish it to its ignored destination. Cache keys must reflect source contents and the relevant converter identity. Retry failed conversions without reusing partial output. Never globally remove `tmp/`, fixtures, or another worktree's container. Keep private mounts and runtime settings out of tracked public configuration.

## Artifacts and Notes

Public dependency evidence: `https://github.com/kleverhq/ondas/blob/v1.1.0/Cargo.toml` and `https://github.com/kleverhq/ondas/blob/v1.1.0/docs/shm-lib.md`.

Local Ondas source inspected: `/home/esynr3z/projects/ondas/ondas-lib`, clean at `1cf6a0e`; relevant files are `docs/shm-lib.md`, `src/waveform.rs`, `src/backends/shm.rs`, and `build.rs`.

Cadence reference inspected: Xcelium 22.09.013, Running SimVision, “Using the Batch Database Translation Utility” and “simvisdbutil Command Syntax”, `/home/esynr3z/.agents/skills/cadence-xcelium/references/22.09.013/doc/RunningSimVision/databases.html`. It documents VCD input, `-64BIT`, `-SHM`, `-OUTPUT`, and DSN/TRN directory output. This establishes documented syntax; successful execution remains a prototype acceptance condition. Do not copy vendor documentation or interfaces into the repository.

## Interfaces and Dependencies

The public CLI continues to take a path through `--waves`. Cargo exposes `shm = ["ondas/shm-lib"]`. The engine continues to use the existing `Waveform` facade and Ondas public `Format::Shm`, metadata, hierarchy, signal, trace, and selection APIs. No new native reader interface belongs in wavepeek. `XCELIUM_HOME` selects the local reader installation; the private environment owns runtime library lookup.

Revision 2026-10-16: Initial analysis and implementation plan. No runtime support has been changed or validated yet.

Revision 2026-10-16: Reduced benchmark scope to the existing E2E harness under the user's KISS/YAGNI constraint. VCD conversion loses string types and keeps escape markers; normalize the markers and record one native rich-value fixture from `tests/fixtures/source/shm_rich.sv`. Its string/event/initial syntax follows IEEE 1800-2017 clauses 6.16, 6.17, and 9.2.1; SHM recording follows Xcelium 22.09.013, Command-Line Debug, Managing Databases and the Tcl `database`/`probe` commands.

Revision 2026-10-16: Recorded successful private/default CI and combined-feature validation. Native recording uses `probe -all -dynamic` to retain strings. All benchmark inputs are prepared; sequential measurements use five samples and two warmups per scenario.

Revision 2026-10-16: Completed benchmark captures and FST functional verification. Preserve native reader metadata; document the observed 0ps versus 1ps start bounds and initial-row differences rather than adjusting SHM to another format. Keep the timing headline restricted to identical-payload pairs.

Revision 2026-10-16: Completed final hooks and `just check`; preserved measured binaries and raw results under `tmp/shm/comparison/`. The benchmark report records all validation, timing ratios, existing FSDB exclusions, and SHM/FST metadata and initial-row differences.

Revision 2026-10-16: Follow-up performance work is tracked in
`shm-performance-plan.md`; its per-scenario 2x target and common-API constraint
supersede the completed integration slice's optimization non-goal.
