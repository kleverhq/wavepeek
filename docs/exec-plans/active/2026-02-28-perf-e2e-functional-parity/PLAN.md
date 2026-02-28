# Align `bench/e2e` Harness with Functional Parity Checks

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Today the performance harness only measures runtime and does not verify that benchmarked commands still return the same functional result. After this plan is implemented, every benchmark run will produce two artifacts per test: Hyperfine timing JSON and Wavepeek output JSON. The harness will compare both performance and functional output (`data`) so regressions are visible immediately while ignoring free-form warning text churn.

This plan also aligns repository navigation: benchmark breadcrumbs move from `perf/` to `bench/` so docs match the executable harness location. A contributor should be able to run benchmarks from `bench/e2e/perf.py`, inspect `<test>.hyperfine.json` and `<test>.wavepeek.json`, and clearly see both timing deltas and functional mismatches.

## Non-Goals

This plan does not change Wavepeek CLI output schema or warning text contract.

This plan does not redesign benchmark scenarios or re-balance the full performance matrix beyond metadata additions required for functional validation.

This plan does not make `run` fail on functional mismatch by default; `run` will surface mismatches in the report, while `compare` remains the blocking gate.

This plan does not add a new benchmark framework. The canonical harness remains `bench/e2e/perf.py` + Hyperfine.

## Progress

- [x] (2026-02-28 12:20Z) Mapped current benchmark structure and confirmed that executable harness is in `bench/e2e`, while breadcrumb docs are still in `perf/`.
- [x] (2026-02-28 12:20Z) Captured baseline behavior of `bench/e2e/perf.py`: `run` writes `<test>.json` timing artifacts and `compare` checks only mean regression.
- [x] (2026-02-28 12:20Z) Drafted this ExecPlan with functional parity scope (`data` + `warnings`) and artifact naming migration.
- [x] (2026-02-28 12:46Z) Added benchmark-local breadcrumbs (`bench/AGENTS.md`, `bench/e2e/AGENTS.md`) and retired `perf/` breadcrumb files.
- [x] (2026-02-28 12:34Z) Captured user clarifications: `run` stays non-blocking, old-run compatibility is not required, and compare must tolerate tests present only on one side.
- [x] (2026-02-28 12:49Z) Extended harness artifact model and IO to split timing and functional outputs into `<test>.hyperfine.json` and `<test>.wavepeek.json` with explicit suffix parsing.
- [x] (2026-02-28 12:50Z) Implemented functional capture in `run`, functional marker column in `run --compare`/`report --compare`, and strict functional checks in `compare`.
- [x] (2026-02-28 13:03Z) Updated benchmark docs and passed validation gates (`make check`, `make ci`).
- [x] (2026-02-28 13:28Z) Applied updated product rule: removed warning-only inventory subcommand, compare parity by `data` only, keep emoji markers with `E`/`D` status suffixes, and stop forcing `data` to list type.

## Surprises & Discoveries

- Observation: `perf/` contains only breadcrumb docs and no runnable harness code.
  Evidence: `perf/AGENTS.md` and `perf/e2e/AGENTS.md` are the only tracked files under `perf/`.

- Observation: `bench/e2e/perf.py` currently loads `*.json` indiscriminately, so introducing Wavepeek JSON artifacts requires suffix-aware parsing.
  Evidence: `load_results()` iterates `run_dir.glob("*.json")` and assumes Hyperfine schema.

- Observation: `REPO_ROOT` computation in the moved harness still pointed at `/workspaces` (old `perf/e2e` depth) instead of repository root.
  Evidence: `bench/e2e/perf.py` used `SCRIPT_DIR.parents[2]`; for `bench/e2e`, repo root is `SCRIPT_DIR.parents[1]`.

- Observation: functional payload `data` is command-shaped by schema (`object` for scalar commands, `array` for list-like commands), so list-only validation in harness caused false failures.
  Evidence: `schema/wavepeek.json` defines `data` via `oneOf`; `info` resolves to object payload.

## Decision Log

- Decision: New artifact naming is `<test>.hyperfine.json` and `<test>.wavepeek.json` for every benchmark test.
  Rationale: Explicit suffixes prevent schema confusion and let report/compare logic parse each artifact type safely.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Functional parity compares only Wavepeek JSON field `data`; `warnings` are ignored in parity checks.
  Rationale: Warning strings are free-form and can change during refactors without functional behavior drift.
  Date/Author: 2026-02-28 / OpenCode

- Decision: `compare` becomes blocking for functional mismatch (non-zero exit), while `run --compare` surfaces mismatch with a high-visibility marker in the report table.
  Rationale: This preserves exploratory benchmark flow and provides a strict CI-style gate when explicitly comparing runs.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Do not keep backward compatibility with legacy `<test>.json` run directories; baseline will be regenerated after implementation.
  Rationale: User explicitly plans fresh baseline regeneration and prefers simpler artifact handling over transition complexity.
  Date/Author: 2026-02-28 / OpenCode

- Decision: Functional JSON capture command is deterministic: start from `test["command"]`, append `--json` only if missing, and run once outside Hyperfine timing loop.
  Rationale: Current catalog mixes commands with and without explicit `--json`; this rule removes ambiguity and keeps timing command unchanged.
  Date/Author: 2026-02-28 / OpenCode

- Decision: `compare` treats missing or invalid functional artifacts as hard failures only for tests present in both revised and golden timing artifacts.
  Rationale: Functional gate must stay strict for matched tests while remaining tolerant to newly added or removed benchmark tests.
  Date/Author: 2026-02-28 / OpenCode

## Outcomes & Retrospective

Planning outcome: scope is now explicit across repository structure, artifact format, functional diff semantics, and reporting behavior.

Implementation outcome: benchmark harness now emits dual artifacts per test (`.hyperfine.json`, `.wavepeek.json`), reports functional parity status in compare reports, and enforces functional parity in blocking `compare` for matched tests.

Validation outcome: targeted smoke (`run`, `run --compare`, `compare`) succeeded with release binary, and repository gates `make check` + `make ci` passed.

Retrospective: adding lightweight Python unit tests for artifact and functional helpers reduced risk in compare error handling and kept marker semantics (`✅`, `✅E`, `⚠️D`, `?`) explicit.

## Context and Orientation

The benchmark harness entry point is `bench/e2e/perf.py`. It exposes `list`, `run`, `report`, and `compare`. `run` currently invokes Hyperfine and writes one file per test (`<test>.json`) that stores timing metrics. `report` and `run --compare` render Markdown in `README.md` within a run directory. `compare` currently validates only mean timing deltas.

Benchmark definitions live in `bench/e2e/tests.json`. This is a JSON object with `tests`, where each test defines command tokens (including `{wavepeek_bin}` placeholder), run count, warmup count, and metadata.

Repository breadcrumb policy requires local `AGENTS.md` files near durable directories. Right now benchmark breadcrumbs are in `perf/` even though runnable code and artifacts are in `bench/e2e`.

Functional parity in this plan means comparing only Wavepeek JSON field `data` between revised and golden runs for each compared test name. `warnings` are intentionally ignored for parity because they are free-form text and may evolve without data-level behavior changes.

`compare` test universe has two sets: matched tests (present in both revised and golden timing artifacts) and unmatched tests (present only on one side). Timing and functional checks execute only for matched tests. Unmatched tests are reported as warnings and never fail compare.

Artifact resolution rules are strict. Timing loader reads only `*.hyperfine.json`. Functional loader reads only `*.wavepeek.json`. Test names are derived by stripping exact suffixes `.hyperfine.json` and `.wavepeek.json`, not by generic `Path.stem`.

## Open Questions

No open questions remain for this scope.

## Plan of Work

Milestone 1 realigns breadcrumbs to actual ownership. Copy the existing perf guide text into `bench/AGENTS.md` and `bench/e2e/AGENTS.md` with updated links, then remove `perf/AGENTS.md` and `perf/e2e/AGENTS.md` and delete empty directories.

Milestone 2 adds artifact model separation. In `bench/e2e/perf.py`, introduce explicit helper functions for artifact paths (`hyperfine` and `wavepeek`) and refactor loaders so timing metrics are read from `*.hyperfine.json` only. Keep report behavior stable except where functional markers are intentionally added.

Milestone 3 adds functional capture during `run`. For each selected benchmark test, keep Hyperfine execution unchanged for timing, then run a deterministic functional command once outside Hyperfine timing: clone `test["command"]`, append `--json` only if not already present, and execute. Parse JSON output and persist `<test>.wavepeek.json`. If output is invalid JSON or misses `data`/`warnings`, fail fast with a clear error naming the test.

Milestone 4 adds functional parity checks and surfacing. During `run --compare`, compare revised vs baseline Wavepeek payload (`data` only) per test and include a dedicated status column in README (`✅` for data match, `✅E` for data match with empty payload, `⚠️D` for data mismatch, `?` for missing counterpart). During `compare`, print all tests with functional mismatch and return exit code `1` if any mismatch exists, even when timing checks pass.

Milestone 4 also defines compare matrix behavior. `compare` warns for unmatched tests (test exists only in revised or only in golden). For matched tests, `compare` fails when any test has missing `*.wavepeek.json`, invalid Wavepeek JSON, or missing `data`/`warnings` keys in either revised or golden run. Output groups messages by category (unmatched tests warning, timing regression, functional mismatch, functional artifact error).

Milestone 5 updates docs and markers to reflect product rules: remove warning-only inventory workflow, ignore warnings in parity checks, and keep emoji markers with `E`/`D` suffixes.

## Concrete Steps

Run all commands from `/workspaces/perf-harness-func-comp`.

1. Preflight benchmark harness and enumerate change tests.

       python3 bench/e2e/perf.py list --filter '^change_'

2. Implement Milestones 1-2 (breadcrumbs + artifact model) and run static sanity.

       python3 bench/e2e/perf.py --help
       python3 bench/e2e/perf.py list --filter '^info_'

3. Implement Milestones 3-4 (functional capture + parity checks) and validate behavior on a narrow slice.

       cargo build --release
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/functional-parity-smoke --filter '^info_picorv32$'
       WAVEPEEK_BIN=./target/release/wavepeek python3 bench/e2e/perf.py run --run-dir bench/e2e/runs/functional-parity-smoke-rev --filter '^info_picorv32$' --compare bench/e2e/runs/functional-parity-smoke
       python3 bench/e2e/perf.py compare --revised bench/e2e/runs/functional-parity-smoke-rev --golden bench/e2e/runs/functional-parity-smoke --max-negative-delta-pct 100

4. Run repository gates after code and docs are updated.

       make check
       make ci

   Expected signal of success is zero exit code for both commands.

## Validation and Acceptance

Acceptance is behavioral and observable.

After Milestone 1, benchmark breadcrumbs exist at `bench/AGENTS.md` and `bench/e2e/AGENTS.md`, and `perf/` breadcrumb files are removed.

After Milestones 2-3, each test run directory contains two per-test files: `<test>.hyperfine.json` and `<test>.wavepeek.json`.

After Milestone 4, `run --compare` README includes a functional status marker column and legend (`✅`, `✅E`, `⚠️D`, `?`). A known data mismatch shows `⚠️D` in report output. `compare` exits with non-zero status if any `data` mismatch is present, and stderr lists all mismatched test names.

Negative-path acceptance is mandatory:

1. If functional capture output is not valid JSON, `run` fails for that test with non-zero exit and test-specific error text.
2. If functional capture JSON lacks `data` or `warnings`, `run` fails for that test with non-zero exit and missing-key error text.
3. If `compare` sees mismatched `data`, it exits `1` and prints all mismatched tests.
4. If `compare` cannot find or parse either side's `*.wavepeek.json` for a matched test, it exits `1` and prints all broken tests.
5. If a test exists only on one side (revised or golden), `compare` prints a warning and continues; this does not change exit code.

After Milestone 5, docs and harness help are consistent with data-only parity semantics and no warning-only inventory subcommand.

## Idempotence and Recovery

All harness commands should remain rerunnable. Re-running `run --run-dir <existing>` overwrites current test artifacts and updates README in place.

If functional capture fails for a test (invalid JSON, missing keys), fail that run early with a clear message and allow the user to rerun after fixing the benchmark command or binary.

If `compare` sees unmatched tests because the revised suite changed, accept warnings and continue; only matched-test regressions are blocking.

## Artifacts and Notes

Expected implementation touch points:

    bench/AGENTS.md
    bench/e2e/AGENTS.md
    bench/e2e/perf.py
    bench/e2e/tests.json
    docs/DEVELOPMENT.md
    docs/exec-plans/active/2026-02-28-perf-e2e-functional-parity/PLAN.md

Expected run-artifact shape after implementation:

    bench/e2e/runs/<run-name>/<test>.hyperfine.json
    bench/e2e/runs/<run-name>/<test>.wavepeek.json
    bench/e2e/runs/<run-name>/README.md

Minimal expected shape of functional artifact:

    {
      "data": {},
      "warnings": [...]
    }

## Interfaces and Dependencies

No new third-party dependencies are required.

`bench/e2e/perf.py` must expose stable helper interfaces for artifact handling. Exact function names may vary, but the final file should have clear separation between:

1. Hyperfine execution and parse (`run_test`, Hyperfine loader).
2. Wavepeek JSON capture and parse (new helper that executes command once and validates `data`/`warnings` keys).
3. Functional parity compare helper returning structured mismatch reasons per test based on `data` only.

`compare` command contract after implementation:

- Input: `--revised`, `--golden`, `--max-negative-delta-pct`.
- Output: unmatched test warnings, timing regression failures, functional mismatches, and functional artifact errors printed separately.
- Exit code: `1` if timing threshold violated, functional mismatch found, or functional artifact error detected for any matched test; `0` otherwise.

Revision Note: 2026-02-28 / OpenCode - Initial plan drafted to relocate benchmark breadcrumbs from `perf/` to `bench/`, add dual artifact export (`.hyperfine.json` + `.wavepeek.json`), enforce functional parity checks on `data`/`warnings`, and add warning-only `change` test inventory workflow.
Revision Note: 2026-02-28 / OpenCode - Incorporated review-pass fixes: deterministic `--json` capture rule, explicit artifact name parsing rules, strict compare behavior for missing/invalid functional artifacts on matched tests, and expanded negative-path acceptance criteria.
Revision Note: 2026-02-28 / OpenCode - Applied user clarifications: keep `run` non-blocking, drop legacy run compatibility (fresh baseline regeneration), and make compare tolerant to unmatched tests by warning instead of failing.
Revision Note: 2026-02-28 / OpenCode - Implemented all milestones end-to-end: moved benchmark breadcrumbs to `bench/`, split artifact IO into `.hyperfine.json`/`.wavepeek.json`, added functional capture + strict compare semantics, documented warning-only inventory (`total=27`), corrected repo root resolution after harness move, and validated with smoke runs plus `make check`/`make ci`.
Revision Note: 2026-02-28 / OpenCode - Updated scope per product decision: removed warning-only inventory subcommand from harness/docs, changed parity checks to compare `data` only (ignore `warnings`), kept emoji-based functional markers with `E`/`D` suffixes, and fixed payload validation to allow command-shaped `data` (object or array).
