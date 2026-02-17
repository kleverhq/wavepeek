# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Keep a Changelog-based release notes workflow and release runbook updates.
- Implemented M2 core CLI commands: `info`, `tree`, and `signals`, with default JSON envelope output and `--human` mode.
- Added a waveform adapter over `wellen` with VCD/FST support for metadata reads, deterministic hierarchy traversal, and scope-local signal listing.
- Added hand-crafted waveform fixtures in `tests/fixtures/hand/` and integration coverage for VCD/FST parity, deterministic ordering, warning behavior, and error paths.
- Expanded the devcontainer toolset with Surfer (with X11/Mesa runtime support).

### Changed
- Refactored command dispatch to pass typed CLI arguments through engine handlers, preserving per-command flags (`--human`, `--max`, `--max-depth`, `--filter`, `--scope`).
- Normalized CLI parse/validation failures to `error: args: ...` (except `--help`/`--version`) with stable stderr-only error reporting semantics.
- Reworked devcontainer image builds into explicit `base`/`ci`/`dev` Docker targets and switched GitHub Actions to `.devcontainer/devcontainer.ci.json` so CI skips dev-only tools (`opencode`, `surfer`, `slang-server`, GUI dependencies) while keeping local development unchanged.

### Fixed
- Standardized runtime error categories and exit-code mapping: `args`/`scope`/`signal` errors exit with code `1`, while file open/parse errors exit with code `2`.
- Stabilized `signals.kind` fallback mapping to `unknown` for unmapped parser kinds across both VCD and FST inputs.

## [0.1.0] - 2026-02-08

### Added
- Initial public release of `wavepeek` CLI.

[Unreleased]: https://github.com/kleverhq/wavepeek/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kleverhq/wavepeek/releases/tag/v0.1.0
