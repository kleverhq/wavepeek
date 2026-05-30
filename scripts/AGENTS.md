# Helper Script Guidance

## Source of Truth

- Helper automation guidance: `../docs/dev/automation.md`
- Quality gates: `../docs/dev/quality.md`
- CI and release automation entrypoints: `../.github/workflows/`
- Script entrypoints and contracts: `../justfile`

## Local Guidance

This directory is scheduled to move into grouped `../tools/` directories. Until that migration lands, keep scripts deterministic and CI-safe: avoid interactive prompts, keep stdout/stderr stable, and return explicit non-zero exits on failure.
