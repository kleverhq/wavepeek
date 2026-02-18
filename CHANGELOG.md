# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Keep a Changelog-based release notes workflow and release runbook updates.
- Added canonical `modules` command with optional `--tree` human renderer for hierarchy inspection.
- Added `signals --abs` for full-path human output while keeping concise short-name default.
- Added dedicated devcontainer fixture provisioning under `/opt/rtl-artifacts` pinned by release version in Dockerfile.
- Added fixture-backed regression coverage for realistic external FST artifacts alongside hand-crafted fixtures.
- Expanded the devcontainer toolset with Surfer (with X11/Mesa runtime support).

### Changed
- Switched CLI contract to default human output with explicit `--json` machine contract mode.
- Renamed command surface from `tree` to `modules` without compatibility alias and removed `--human` from the CLI surface.
- Updated CLI entry/help/version behavior: no-args now prints top-level help to stdout with exit code `0`, and both short/long help/version flags are covered by integration tests.
- Improved `error: args:` diagnostics to preserve actionable clap context and append deterministic help hints (`wavepeek --help` or `wavepeek <cmd> --help`).
- Removed `time_precision` from `info` output contract; metadata now exposes `time_unit`, `time_start`, and `time_end`.
- Enforced container-only `make ci` and `make pre-commit` execution with explicit fail-fast guard when container marker is absent.
- Reworked devcontainer image builds into explicit `base`/`ci`/`dev` Docker targets and switched GitHub Actions to `.devcontainer/devcontainer.ci.json` so CI skips dev-only tools (`opencode`, `surfer`, GUI dependencies) while keeping local development unchanged.
- Removed extra HDL tooling from local devcontainer/bootstrap requirements.

### Fixed
- Standardized runtime error categories and exit-code mapping: `args`/`scope`/`signal` errors exit with code `1`, while file open/parse errors exit with code `2`.
- Stabilized `signals.kind` fallback mapping to `unknown` for unmapped parser kinds across both VCD and FST inputs.
- Corrected module hierarchy listing semantics on realistic FST data to emit module instances (not signal leaves).

## [0.1.0] - 2026-02-08

### Added
- Initial public release of `wavepeek` CLI.

[Unreleased]: https://github.com/kleverhq/wavepeek/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.1.0
