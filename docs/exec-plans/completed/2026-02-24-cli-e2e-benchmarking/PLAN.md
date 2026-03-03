# Add Hyperfine-Based CLI E2E Benchmark Runner

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

We implemented a reproducible benchmark harness for wavepeek CLI end-to-end scenarios so contributors can run the same `info`, `at`, and `change` matrix and compare revisions against a stable baseline. The shipped workflow is centered on `bench/e2e/perf.py` (Python stdlib + `argparse`) and `hyperfine`, with test definitions in `bench/e2e/tests.json`, run artifacts under `bench/e2e/runs/`, and run-local Markdown reporting.

User-visible outcome: from repository root, a user can list tests, run a filtered benchmark matrix, regenerate a report from saved JSON artifacts, and fail fast on regressions versus a golden run.

## Non-Goals

This plan does not optimize engine internals for `at`/`change`; it only provides measurement infrastructure and baseline evidence.

This plan does not replace the separate `change` performance optimization plan in `docs/exec-plans/completed/2026-02-24-change-performance-stateless/PLAN.md`.

This plan does not add third-party Python dependencies.

## Progress

- [x] (2026-02-24) Added `hyperfine` to devcontainer base image so benchmark runs work in standard container workflows.
- [x] (2026-02-24) Authored initial plan and delivered first harness iteration.
- [x] (2026-02-24) Implemented benchmark harness with `list`, `run`, `report`, `compare` and JSON result parsing.
- [x] (2026-02-24) Added full explicit benchmark catalog at `bench/e2e/tests.json` and switched harness location to `bench/e2e/perf.py`.
- [x] (2026-02-24..2026-02-27) Expanded fixture coverage (`info` and chipyard cases), tuned matrix scope, and moved `change` tests to real wavepeek commands.
- [x] (2026-02-27) Captured and committed golden baseline artifacts under `bench/e2e/runs/baseline/`.
- [x] (2026-02-27) Updated docs/changelog/backlog collateral and removed superseded prototype harness (`scripts/cli_e2e_bench.py`).
- [x] (2026-02-27) Reconciled this plan with current implementation and baseline state.

## Surprises & Discoveries

- Observation: the final implementation architecture diverged from the original prototype plan.
  Evidence: initial `scripts/cli_e2e_bench.py` and `scripts/tests/test_cli_e2e_bench.py` were removed; canonical harness is now `bench/e2e/perf.py` with explicit catalog in `bench/e2e/tests.json`.

- Observation: one JSON artifact per test is enough for current reporting/comparison needs.
  Evidence: committed baseline contains `<test_name>.json` hyperfine exports plus `README.md`; no separate `*.wavepeek.json` sidecar is required by current compare logic.

- Observation: practical runtime required matrix reduction for `at` while preserving broad fixture coverage.
  Evidence: matrix settled at 132 total tests (`info=8`, `at=16`, `change=108`) instead of the larger combinatorial draft.

- Observation: benchmark evidence exposed a meaningful `at` anomaly worth tracking separately.
  Evidence: backlog item `docs/BACKLOG.md` documents `at_picorv32_signals_1000` slowdown and links it to engine-level follow-up work.

- Observation: `compare` must tolerate revised-only tests while still enforcing regression guardrails.
  Evidence: `perf.py compare` warns for missing golden counterparts and checks threshold only for matching tests.

## Decision Log

- Decision: make `bench/e2e/perf.py` + `bench/e2e/tests.json` the long-term harness shape.
  Rationale: explicit JSON catalog is easier to evolve per fixture and per-test run budget than generated dataclass matrices.
  Date/Author: 2026-02-24 / OpenCode

- Decision: keep compare delta semantics as `((golden - revised) / golden) * 100` and fail when delta is below negative threshold.
  Rationale: negative values directly represent regressions for lower-is-better metrics.
  Date/Author: 2026-02-24 / OpenCode

- Decision: resolve target binary via `WAVEPEEK_BIN` (or PATH) instead of auto-building release binaries in the harness.
  Rationale: keeps benchmark runner focused on measurement and avoids hidden build-time variability.
  Date/Author: 2026-02-24 / OpenCode

- Decision: use category-grouped Markdown reports sorted by slowest mean first within each category.
  Rationale: this makes hotspots obvious in baseline/revision reviews.
  Date/Author: 2026-02-24 / OpenCode

- Decision: commit a repository baseline run (`bench/e2e/runs/baseline`) for reproducible comparisons.
  Rationale: gives contributors and CI a concrete golden dataset and report.
  Date/Author: 2026-02-27 / OpenCode

- Decision: move feature-follow-up concerns to backlog instead of expanding this plan scope.
  Rationale: keeps harness delivery complete while isolating optimization and JSONL-streaming work.
  Date/Author: 2026-02-27 / OpenCode

## Outcomes & Retrospective

Outcome: plan goals are implemented and operational. The repository now contains a working CLI E2E performance harness, explicit benchmark catalog, committed golden baseline, and supporting documentation.

What remains: engine-level performance improvements are intentionally out of scope and tracked in backlog/other plans.

Lesson learned: for large, fixture-heavy benchmark suites, explicit test catalogs and committed baseline artifacts are more maintainable than generated combinatorial matrices.

## Context and Orientation

The benchmark system is intentionally simple and repository-native.

Key files:

- `bench/e2e/perf.py`: benchmark CLI implementation.
- `bench/e2e/tests.json`: explicit test catalog with per-test `runs`, `warmup`, command tokens, and metadata.
- `bench/e2e/runs/baseline/`: committed golden run artifacts (132 JSON files + README report).
- `docs/DEVELOPMENT.md`: developer-facing usage section.
- `CHANGELOG.md`: unreleased entry for the harness.
- `docs/BACKLOG.md`: follow-up perf and output-mode items discovered during rollout.

Core behavior constraints:

- `run` creates a timestamped directory unless `--run-dir` is provided.
- Each executed test writes one hyperfine export JSON (`<test_name>.json`).
- `README.md` is regenerated from discovered run JSON files.
- `run --compare` and `report --compare` annotate mean values with delta percentages and emoji threshold markers (`abs(delta) >= 3%`).
- `compare` is revised-centric, warns on missing golden counterparts, and fails only on matched tests exceeding negative threshold.

## CLI Contract (Implemented)

- `python3 bench/e2e/perf.py list [--filter REGEX]`
- `python3 bench/e2e/perf.py run [--run-dir PATH] [--out-dir PATH] [--filter REGEX] [--compare PATH]`
- `python3 bench/e2e/perf.py report --run-dir PATH [--compare PATH]`
- `python3 bench/e2e/perf.py compare --revised PATH --golden PATH --max-negative-delta-pct FLOAT`

`WAVEPEEK_BIN` selects the binary used to render command templates from `tests.json`.

## Report Contract (Implemented)

`README.md` contains sections for categories present in discovered artifacts (typically `at`, `change`, `info`) with table columns `test`, `mean_s`, `meta`.

Rows are sorted by descending `mean_s` inside each category (slowest first). When compare data is available for a test, mean renders as `<value> (<delta%>)` with optional `🟢`/`🔴` marker.

## Matrix Contract (Implemented)

Current committed matrix is fixture-driven and explicit:

- `info`: 8 tests.
- `at`: 16 tests.
- `change`: 108 tests.
- Total: 132 tests.

Fixtures currently include picorv32, scr1, and chipyard (dualrocket + clusteredrocket) variants.

## Open Questions

No blocking open questions remain for this plan. Future capability work is tracked in backlog and separate exec plans.

## Validation and Acceptance

Acceptance criteria now verified by shipped artifacts and commands:

- `list` enumerates stable explicit catalog entries from `bench/e2e/tests.json`.
- `run` produces per-test JSON artifacts and regenerates run `README.md`.
- `report` rebuilds metric tables and ordering from artifact files, while intentionally refreshing the generated-at timestamp on each run.
- `compare` exits non-zero on threshold regressions and prints all violating tests; missing golden tests are warnings.
- Golden baseline exists at `bench/e2e/runs/baseline` and can be reused for local or CI comparison.

## Idempotence and Recovery

`list` is read-only. `report` is safe to rerun and only rewrites `README.md`.

`run` is idempotent per selected tests in a target directory: reruns overwrite only matching `<test_name>.json` files and final `README.md`.

If a run is interrupted, rerun with the same `--run-dir` to refresh incomplete artifacts and regenerate report output.

## Interfaces and Dependencies

Main Python entry points in `bench/e2e/perf.py` are `load_tests`, `select_tests`, `cmd_list`, `cmd_run`, `cmd_report`, `cmd_compare`, and report helpers.

External dependencies are command-line tools expected in container/runtime environment: `hyperfine` and a resolvable `wavepeek` binary (`WAVEPEEK_BIN` or PATH).

Revision Note: 2026-02-24 / OpenCode - Initial plan created for a standalone hyperfine-backed CLI E2E benchmark harness.
Revision Note: 2026-02-24..2026-02-27 / OpenCode - Implementation evolved to `bench/e2e/perf.py` with explicit `tests.json` catalog, real command coverage, and committed baseline artifacts.
Revision Note: 2026-02-27 / OpenCode - Reconciled plan with actual `main...HEAD` diff and removed stale prototype assumptions (dataclass matrix, scripts-based harness, dummy-only execution, paired wavepeek/hyperfine artifacts, release auto-build preflight).
