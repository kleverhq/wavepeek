# Performance Guidance

## Source of Truth

- Manual benchmark gate: `../docs/dev/benchmarking.md`
- Runtime behavior contracts: `../docs/public/reference/command-model.md`, `../docs/public/reference/machine-output.md`

## Local Guidance

- Run benchmarks in the devcontainer or CI image so fixture paths, tool versions, and environment variables match project gates.
- Keep benchmark harnesses deterministic and stdlib-only unless a maintainer accepts a dependency change.
- End-to-end CLI scenarios live under `e2e/`.
- Generated run artifacts are ignored local evidence; do not add committed baselines unless a maintainer explicitly changes the benchmark policy.
- Preserve functional payload parity while optimizing; faster incorrect output remains incorrect output.
