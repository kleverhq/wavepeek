# Quality Gates

`wavepeek` quality gates are owned by the root `justfile`. Run them inside the devcontainer or CI image with `WAVEPEEK_IN_CONTAINER=1`; most recipes intentionally fail outside that environment so local and CI results do not drift.

## Standard Gates

Use `just check` before handing off local work. It runs formatting checks, clippy, schema contract validation, GitHub Actions linting, `cargo check`, and commit-message validation.

Use `just ci` when tests should be included. It runs the same static checks plus auxiliary Python unit tests, the source coverage gate, and `cargo check`. The Rust test execution used by CI is the `cargo llvm-cov` run behind `just coverage-src-check`; use `just test` when you want an explicit non-coverage Rust test pass while iterating.

Use `just pre-commit` to run all installed pre-commit hooks across the repository. Hooks are installed by `just dev-setup` and must not be bypassed unless a maintainer explicitly asks.

## Focused Recipes

- `just format` and `just format-check` run rustfmt and the justfile formatter.
- `just lint` runs clippy with warnings denied; `just lint-fix` applies safe clippy suggestions when useful.
- `just check-build` runs `cargo check`.
- `just check-schema` validates `schema/wavepeek.json` against the runtime schema contract.
- `just check-actions` runs `actionlint` for `.github/workflows/`.
- `just coverage-src` reports source coverage without enforcing thresholds.
- `just coverage-src-check` enforces the current `90%` minimum on lines, regions, and functions for `src/**`.
- `just check-commit` runs Commitizen against Git's current commit-message file.

## Optional FSDB Gates

The optional `fsdb` Cargo feature requires a local Synopsys Verdi FSDB Reader SDK and is intentionally excluded from default `just lint`, `just check`, `just ci`, and pre-commit paths. Use the Verdi-equipped recipes only on Linux x86_64 environments where `tools/fsdb/check_fsdb_env.py` can validate a usable SDK. FSDB binaries embed the selected Verdi library directory as an rpath/RUNPATH by default, so the Verdi-equipped recipes do not require `LD_LIBRARY_PATH` setup.

- `just check-fsdb-env` probes availability and treats missing Verdi as a skip.
- `just check-fsdb-build` validates the feature-enabled build/link path plus native Reader smokes.
- `just lint-fsdb` runs feature-enabled clippy under `CARGO_TARGET_DIR=target/fsdb`.
- `just prepare-fsdb-fixtures` generates ignored FSDB fixtures from committed VCD fixtures and RTL FST artifacts.
- `just test-fsdb` runs the supported FSDB regression path.
- `just bench-e2e-fsdb-smoke-commit` runs a lightweight functional-only FSDB benchmark smoke.

## Interpreting Failures

Schema freshness failures usually mean the runtime schema output changed; regenerate with `just update-schema` only if the schema change is intended. Coverage failures should be fixed by adding meaningful tests, not by lowering thresholds. If `just check-commit` fails before any commit exists because `.git/COMMIT_EDITMSG` is missing or stale, create the commit normally and let the commit-msg hook validate the actual message.

Keep stdout and stderr from project tools deterministic. If a gate reports unstable output, treat that as a product-contract problem rather than noisy tooling.
