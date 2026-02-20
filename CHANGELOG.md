# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

## [0.1.0] - 2026-02-08

### Added
- Initial public release of `wavepeek` CLI.

[Unreleased]: https://github.com/kleverhq/wavepeek/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.1.0
