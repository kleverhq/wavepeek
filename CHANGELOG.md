# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added layered help plus embedded local docs: `wavepeek -h` is now compact, `wavepeek --help` and `wavepeek help <command-path...>` provide the detailed reference layer, and `wavepeek docs` exposes packaged topics, search, export, and the shipped agent skill.

### Changed
<<<<<<< HEAD
- Renamed the CI devcontainer config to `.devcontainer/devcontainer.ci.json` for clearer intent.
- Expanded the devcontainer for multi-agent workflows with Codex, Claude Code, Pi, OpenCode, and host-mounted agent state.
=======
- Moved open design questions into `docs/BACKLOG.md` and removed the separate `docs/design/open_questions.md` entrypoint.
- Superseded the temporary universal `-h == --help` help model with layered help and a packaged narrative-doc surface.

### Removed
- Removed legacy documentation paths `docs/DESIGN.md` and `docs/expression_lang.md`; use `docs/design/index.md` and `docs/design/contracts/expression_lang.md` instead.
>>>>>>> fa84440 (docs: tighten changelog policy and retire legacy doc entrypoints)

## [0.4.0] - 2026-04-05

### Added
- Implemented the `property` command end to end, with `match`/`switch`/`assert`/`deassert` capture modes plus human and `--json` output.

### Changed
- `change --on` now supports typed `iff` expressions at runtime instead of rejecting them, enabling richer trigger conditions.

### Fixed
- Invalid `change` and `property` expressions now fail consistently with deterministic `error: expr:` diagnostics.

## [0.3.0] - 2026-03-07

### Added
- Added fully functional `value` command for deterministic point-in-time signal sampling on both VCD and FST dumps, including scope-relative and full-path signal resolution.
- Added `value --abs` for canonical-path rendering in human mode while keeping compact default display based on requested `--signals` tokens.
- Added strict `value` JSON contract in `schema/wavepeek.json` with compact `valueData` payload (`time`, ordered `signals[{path,value}]`) and conditional command-to-data validation.
- Added fully functional `change` command with unified `--on` event expressions (`*`, named, `posedge`/`negedge`/`edge`, `or`/`,` unions), deterministic delta snapshots, and staged `iff` parsing with explicit deferred-runtime error.
- Added new change-focused fixtures and integration coverage for baseline checkpoint semantics, union deduplication, warning parity, duplicate signal ordering, and default `--max=50` truncation behavior.
- Added a new hyperfine-backed CLI E2E benchmark harness (`bench/e2e/perf.py`) with `run`/`list`/`report`/`compare` modes, a flat explicit test catalog (`bench/e2e/tests.json`), per-test JSON artifacts, and Markdown run reports.
- Added recursive signal listing via `wavepeek signal --recursive` with optional `--max-depth`, deterministic depth-first traversal, depth-0 parity with non-recursive mode, and relative-path human rendering scoped to `--scope`.
- Added recursive `signal` benchmark catalog coverage for SCR1 (`all`, `filter valid`, `max-depth 2`) in `bench/e2e/tests.json`.
- Added explicit `unlimited` literal support for `--max` (`scope`, `signal`, `change`) and `--max-depth` (`scope`, recursive `signal`) with deterministic warning parity in human stderr and JSON envelopes.
- Added CLI help contract integration coverage for top-level/subcommand parity (`-h == --help`), no-args parity with `--help`, and per-command self-descriptive guidance markers.
- Added `tests/change_opt_equivalence.rs` and `tests/change_vcd_fst_parity.rs` to lock optimization invariants (strict previous-timestamp delta semantics) and VCD/FST parity for `change`.

### Changed
- Breaking rename for point-in-time sampling surface: removed `at` command and `--time` flag, and replaced them with `value` and `--at`.
- Breaking rename for event/property surface: removed `when` command and `change --when` flag, and replaced them with `property` and `change --on` (no compatibility aliases).
- Updated CLI help/contracts and JSON schema so `change` is no longer marked as unimplemented and now documents `--on`/`--abs` behavior; `property` remains explicitly unimplemented.
- Aligned design documentation wording from `time_precision` to `time_unit` for normalized timestamp fields.
- Simplified `value` human output to compact form: `@<time>` header and `<display> <value>` signal lines.
- Updated JSON envelope `$schema` URLs from GitHub blob pages to `raw.githubusercontent.com` so schema links resolve as directly consumable raw JSON.
- At this release, help ergonomics were temporarily unified so `-h` and `--help` both rendered the detailed layer everywhere; later work replaced that temporary model with layered help and embedded local docs.
- Expanded command help text to include semantics, defaults/requiredness, boundary rules, normalized error-guidance wording, and output-shape notes without requiring `docs/design/reference/cli.md` for day-to-day usage.
- Accelerated `change` stateless execution with resolved-handle sampling, persistent signal-load caching, timestamp-slice iteration, and candidate-timestamp reduction while preserving strict pre-candidate baseline semantics.
- Added a multi-engine dispatcher for dense and sparse `change` workloads so large dump/window scenarios now complete around ~0.5s in the benchmark matrix while keeping payload parity.
- Added an FST streaming-capable candidate-time path (`wellen::stream` filter pushdown with fallback heuristic) and dense edge-trigger fast path internals while preserving existing `change` contract behavior and output parity.

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

[Unreleased]: https://github.com/kleverhq/wavepeek/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.4.0
[0.3.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.3.0
[0.2.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.2.0
[0.1.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.1.0
