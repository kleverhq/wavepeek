# Quality Gates

`wavepeek` quality gates are owned by the root `justfile`. Run them from the host through `./dev`; most recipes intentionally fail outside the shared development container so local and CI results do not drift.

## Standard Gates

Use `./dev just check` before handing off local work. It runs formatting checks, clippy, GitHub Actions linting, benchmark catalog checks, packaged-skill and docs-site checks, native/browser Playground parity, and `cargo check`. Native build checks run when their SDK is configured.

Run `./dev just ci` locally for the checks used by GitHub Actions. The recipe runs formatting, lint, GitHub Actions checks, auxiliary Python unit tests, `cargo check`, packaged-skill and docs-site checks, native/browser Playground parity, and FST benchmark smoke. `cargo llvm-cov` runs the Rust unit and integration tests. `cargo test --doc` runs any doctests. The recipe includes native lint, tests, and smoke for each configured SDK.

## Focused Recipes

- `just format` and `just format-check` run rustfmt and the justfile formatter.
- `just lint` runs clippy with warnings denied and includes SDK-enabled targets when configured. `just lint-fix` applies safe clippy suggestions.
- `just check-build` runs `cargo check`.
- `just check-actions` runs `actionlint` for `.github/workflows/`.
- `just docs-site-build` stages packaged skill references under `tmp/docs-site/`, generates a MkDocs config, and builds the site in strict mode.
- `just docs-site-check` runs the full local docs and current-Playground publication check without touching `gh-pages`.
- `just playground-build` builds the current root app; `just playground-preview-build`, `just playground-test`, and `just playground-serve` compose it with current documentation, test it, or serve it locally.
- `just bench-gate`, `just bench-capture`, and `just bench-compare` support manual performance review and are not part of the standard pre-handoff gates.
- `just coverage-src` reports source coverage without enforcing thresholds.
- `just coverage-src-check` enforces the current `90%` minimum on lines, regions, and functions for `src/**`.

## Native SDK Checks Before Opening a PR

The public GitHub CI profile skips FSDB and SHM lint, tests, and smoke. Run these checks locally before opening each PR. Run `./dev just ci` with the required SDKs configured, or run the focused suites in their private profiles.

```sh
./dev just lint-fsdb test-fsdb bench-e2e-fsdb-smoke
./dev just lint-shm test-shm bench-e2e-shm-smoke
```

The focused recipes fail if the required SDK is unavailable. Each smoke run checks CLI execution and saves artifacts under `tmp/bench-smoke/`. See [FSDB setup](fsdb.md) and [SHM setup](shm.md).

## Interpreting Failures

Address coverage failures by adding meaningful tests. If a gate reports unstable output, investigate the product contract.
