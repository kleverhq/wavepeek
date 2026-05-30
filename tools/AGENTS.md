## Source of Truth

- Helper automation guidance: `../docs/dev/automation.md`
- Quality gates: `../docs/dev/quality.md`
- Stable task interface: `../justfile`
- CI and release workflows: `../.github/workflows/`

## Local Guidance

- Keep helper groups deterministic, non-interactive, and safe for CI.
- Prefer invoking helpers through `just` recipes or workflow steps; helper paths are implementation detail unless documented here or in `../docs/dev/automation.md`.
- Keep helper tests next to the helper group they cover, and update `just test-aux` when adding or moving helper tests.
- Avoid generated state in this tree. Python cache files, temporary logs, and ad hoc outputs belong in ignored locations such as `../tmp/`.
