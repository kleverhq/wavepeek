## Source of Truth

- Helper automation guidance: `../docs/automation.md`
- Quality gates: `../docs/quality.md`
- Stable task interface: `../justfile`
- CI and release workflows: `../.github/workflows/`

## Local Guidance

- Keep helper groups deterministic, non-interactive, and safe for CI.
- Prefer invoking helpers through `just` recipes or workflow steps; helper paths are implementation detail unless documented here or in `../docs/automation.md`.
- Keep helper tests next to the helper group they cover, and update `just test-aux` when adding or moving helper tests.
- Each helper group must have a concise purpose/usage description either in script header comments or in a companion `README.md`; use a README when a group has multiple scripts or workflow-facing behavior.
- Keep generated helper state, including Python caches and temporary logs, in ignored locations such as `../tmp/`.
