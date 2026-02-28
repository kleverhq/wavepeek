# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added fully functional `at` command for deterministic point-in-time signal sampling on both VCD and FST dumps, including scope-relative and full-path signal resolution.
- Added `at --abs` for canonical-path rendering in human mode while keeping compact default display based on requested `--signals` tokens.
- Added strict `at` JSON contract in `schema/wavepeek.json` with compact `atData` payload (`time`, ordered `signals[{path,value}]`) and conditional command-to-data validation.
- Added fully functional `change` command with unified `--when` event expressions (`*`, named, `posedge`/`negedge`/`edge`, `or`/`,` unions), deterministic delta snapshots, and staged `iff` parsing with explicit deferred-runtime error.
- Added new change-focused fixtures and integration coverage for baseline checkpoint semantics, union deduplication, warning parity, duplicate signal ordering, and default `--max=50` truncation behavior.
- Added a new hyperfine-backed CLI E2E benchmark harness (`bench/e2e/perf.py`) with `run`/`list`/`report`/`compare` modes, a flat explicit test catalog (`bench/e2e/tests.json`), per-test JSON artifacts, and Markdown run reports.
- Added recursive signal listing via `wavepeek signal --recursive` with optional `--max-depth`, deterministic depth-first traversal, depth-0 parity with non-recursive mode, and relative-path human rendering scoped to `--scope`.
- Added recursive `signal` benchmark catalog coverage for SCR1 (`all`, `filter valid`, `max-depth 2`) in `bench/e2e/tests.json`.
- Added explicit `unlimited` literal support for `--max` (`scope`, `signal`, `change`, `when` parsing) and `--max-depth` (`scope`, recursive `signal`) with deterministic warning parity in human stderr and JSON envelopes.
- Added CLI help contract integration coverage for top-level/subcommand parity (`-h == --help`), no-args parity with `--help`, and per-command self-descriptive guidance markers.
- Added `tests/change_opt_equivalence.rs` and `tests/change_vcd_fst_parity.rs` to lock optimization invariants (strict previous-timestamp delta semantics) and VCD/FST parity for `change`.

### Changed
- Updated CLI help/contracts and JSON schema so `change` is no longer marked as unimplemented and now documents `--when`/`--abs` behavior; `when` remains explicitly unimplemented.
- Aligned design documentation wording from `time_precision` to `time_unit` for normalized timestamp fields.
- Simplified `at` human output to compact form: `@<time>` header and `<display> <value>` signal lines.
- Updated JSON envelope `$schema` URLs from GitHub blob pages to `raw.githubusercontent.com` so schema links resolve as directly consumable raw JSON.
- Unified help ergonomics so `-h` and `--help` both render detailed help across all shipped commands; `wavepeek` with no args now follows the same render path as `wavepeek --help` and is byte-identical.
- Expanded command help text to include semantics, defaults/requiredness, boundary rules, normalized error-guidance wording, and output-shape notes without requiring `docs/DESIGN.md` for day-to-day usage.
- Accelerated `change` stateless execution by switching to resolved-handle sampling, persistent signal-load caching, timestamp-slice iteration, and candidate-timestamp reduction with strict pre-candidate baseline semantics.
- Added an FST streaming-capable candidate-time path (`wellen::stream` filter pushdown with fallback heuristic) while preserving existing `change` contract behavior and output parity.

## [0.2.0] - 2026-02-20

### Added
- Keep a Changelog-based release notes workflow and release runbook updates.
- Added canonical `scope` command with optional `--tree` human renderer for hierarchy inspection.
- Added `signal --abs` for full-path human output while keeping concise short-name default.
- Added dedicated devcontainer fixture provisioning under `/opt/rtl-artifacts` pinned by release version in Dockerfile.
- Added fixture-backed regression coverage for realistic external FST artifacts alongside hand-crafted fixtures.
- Expanded the devcontainer toolset with Surfer (with X11/Mesa runtime support).
- Added canonical schema artifact at `schema/wavepeek.json` with deterministic `wavepeek schema` export and `make update-schema` regeneration workflow.

### Changed
- Switched CLI contract to default human output with explicit `--json` machine contract mode.
- Renamed command surface to singular canonical forms without compatibility aliases: `tree`/`modules` -> `scope`, `signals` -> `signal`, `changes` -> `change`; `--human` remains removed from the CLI surface.
- Updated CLI entry/help/version behavior: no-args now prints top-level help to stdout with exit code `0`, and both short/long help/version flags are covered by integration tests.
- Improved `error: args:` diagnostics to preserve actionable clap context and append deterministic help hints (`wavepeek --help` or `wavepeek <cmd> --help`).
- Removed `time_precision` from `info` output contract; metadata now exposes `time_unit`, `time_start`, and `time_end`.
- Expanded hierarchy output to include all scope kinds with explicit `kind` metadata.
- Enforced container-only `make ci` and `make pre-commit` execution with explicit fail-fast guard when container marker is absent.
- Reworked devcontainer image builds into explicit `base`/`ci`/`dev` Docker targets and switched GitHub Actions to `.devcontainer/.devcontainer.json` so CI skips dev-only tools (`opencode`, `surfer`, GUI dependencies) while keeping local development unchanged.
- Removed extra HDL tooling from local devcontainer/bootstrap requirements.
- **Breaking:** migrated JSON envelope metadata from `schema_version` to `$schema` with versioned URL format `https://github.com/kleverhq/wavepeek/blob/v<version>/schema/wavepeek.json`.
- Simplified `schema` command to an argument-free deterministic contract: `wavepeek schema` always prints exactly one JSON Schema document to stdout.
- Added schema freshness enforcement to quality gates (`make check`, `make ci`, pre-commit) with explicit remediation hint via `make update-schema`.
- Reworked README into a concise, task-oriented format with quick start, command status, agentic-flow roadmap notes, and development workflow sections.
- Moved `schema` to the end of top-level CLI help output to keep waveform commands first.

### Fixed
- Standardized runtime error categories and exit-code mapping: `args`/`scope`/`signal` errors exit with code `1`, while file open/parse errors exit with code `2`.
- Preserved parser-native `signal.kind` aliases across both VCD and FST inputs (for example, `parameter` is no longer collapsed to `unknown`).
- Corrected hierarchy listing semantics on realistic FST data to emit scope entries (not signal leaves).
- Fixed CI devcontainer workspace permissions by enabling remote UID remapping so `cargo clippy` can write target artifacts.
- Fixed CI devcontainer baseline image to include `python3` for schema contract validation during `make ci`.

## [0.1.0] - 2026-02-08

### Added
- Initial public release of `wavepeek` CLI.

[Unreleased]: https://github.com/kleverhq/wavepeek/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.2.0
[0.1.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.1.0
