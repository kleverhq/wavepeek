# Bench E2E Perf Harness QoL: Quiet Defaults, Stable Reports, and Pre-Commit Smoke Catalog

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, `bench/e2e/perf.py` becomes safer for agent-driven debugging sessions: `run` and `compare` are quiet by default, `list` prints names only, and operators can opt into detailed logs with `-v/--verbose` when needed. This prevents long benchmark logs from consuming context window budget while keeping a deterministic path to deeper diagnostics.

The generated per-run report in `README.md` drops the redundant "Run directory" line so reports remain stable across temporary worktree paths. This avoids stale path noise in committed artifacts.

Pre-commit also gains a dedicated lightweight benchmark catalog (`tests_commit.json`) that exercises a small cross-category subset of real perf scenarios. This closes the current gap where harness unit tests pass but integration behavior can still drift silently.

## Non-Goals

This plan does not change benchmark math, compare pass/fail policy, report table schema, or baseline artifact content beyond the removal of the "Run directory" metadata line. This plan does not add new benchmark categories. This plan does not change `report` command CLI flags unless required to keep behavior consistent with the `run` path. This plan does not tune performance thresholds or alter Rust runtime behavior.

## Progress

- [x] (2026-03-05 13:05Z) Mapped current `perf.py` command output behavior, parser flags, and report rendering structure.
- [x] (2026-03-05 13:12Z) Mapped pre-commit and Makefile integration points and identified missing pre-commit smoke coverage.
- [x] (2026-03-05 13:25Z) Drafted active ExecPlan with SDD scope, TDD-first implementation steps, and acceptance criteria.
- [x] (2026-03-05 13:35Z) Completed review pass #1 on plan quality; tightened smoke scope, determinism, and test coverage requirements.
- [x] (2026-03-05 13:47Z) Completed independent review pass #2 on plan quality; tightened binary/fixture pinning and `--tests` path semantics.
- [ ] Execute Milestone 1 (tests-first CLI surface for `--tests` and verbosity controls).
- [ ] Execute Milestone 2 (quiet-by-default logging behavior and report/list output changes).
- [ ] Execute Milestone 3 (pre-commit smoke catalog + Makefile + hook wiring).
- [ ] Run full validation, complete implementation review pass #1, apply fixes, commit.
- [ ] Run independent mandatory review pass #2 (fresh context), apply fixes, re-validate, commit.

## Surprises & Discoveries

- Observation: the "README.md" path concern maps to generated run reports, not the repository root `README.md`.
  Evidence: `bench/e2e/perf.py` writes report header lines in `render_report(...)`, including `- Run directory: ...`, and committed run report files under `bench/e2e/runs/*/README.md` contain that line.

- Observation: test catalog selection is currently hard-wired to `bench/e2e/tests.json`.
  Evidence: `TESTS_PATH` is a module constant and `load_tests()` has no path argument; `list` and `run` call `load_tests()` directly.

- Observation: pre-commit currently validates harness unit tests only, not an actual benchmark run/compare smoke path.
  Evidence: `.pre-commit-config.yaml` hook `bench-e2e-test` runs `make test-bench-e2e`, which executes Python unittest discovery in `bench/e2e`.

## Decision Log

- Decision: interpret "README.md report" as the generated run report (`bench/e2e/runs/.../README.md`) and remove only the run-directory header line there.
  Rationale: user motivation references path decay from temporary worktrees, which applies to generated run directories, not the project README.
  Date/Author: 2026-03-05 / OpenCode

- Decision: introduce `--tests` (path to JSON catalog) on `run` and `list`, defaulting to existing `bench/e2e/tests.json`.
  Rationale: satisfies explicit requirement while preserving backwards compatibility for existing workflows that do not pass a catalog path.
  Date/Author: 2026-03-05 / OpenCode

- Decision: make `run` and `compare` quiet by default and gate detailed progress/warning output behind `-v/--verbose`.
  Rationale: this directly addresses context-window pressure in agentic debugging while preserving a deterministic escape hatch for diagnostics.
  Date/Author: 2026-03-05 / OpenCode

- Decision: add a dedicated pre-commit benchmark smoke catalog with a few smallest tests per category, and wire it through a new Make target and pre-commit hook.
  Rationale: integration smoke coverage catches silent harness breakage with lower overhead than full benchmark sweeps.
  Date/Author: 2026-03-05 / OpenCode

- Decision: require pre-commit smoke to execute both `run` and `compare` (not `run` only).
  Rationale: user-requested QoL changes include `compare` behavior; run-only smoke can miss regressions in compare parsing and quiet/verbose output paths.
  Date/Author: 2026-03-05 / OpenCode

- Decision: lock `tests_commit.json` to an explicit 8-test subset (2 tests per category) with low-latency settings (`runs=1`, `warmup=0`) and a target runtime budget of <= 180 seconds in devcontainer.
  Rationale: deterministic selection and budget prevent drift and keep pre-commit overhead predictable.
  Date/Author: 2026-03-05 / OpenCode

- Decision: resolve `--tests` paths exactly like other path inputs by applying `normalize_path` to user input at command execution time, which interprets relative paths against the process current working directory.
  Rationale: explicit path semantics eliminate shell/CI/hook ambiguity and match existing harness path normalization style.
  Date/Author: 2026-03-05 / OpenCode

## Outcomes & Retrospective

Current status: planning complete, implementation not started.

Expected outcome after implementation: `perf.py list` prints test names only, `run` and `compare` stay silent during execution unless `--verbose` is provided, generated run reports omit the run-directory line, and pre-commit executes a lightweight real benchmark smoke path powered by `tests_commit.json`.

Primary risk to watch during implementation: over-suppressing error information in non-verbose mode. The acceptance contract below keeps one-line success/failure outcomes visible while reserving detailed diagnostics for verbose mode.

## Context and Orientation

The benchmark harness is implemented in `bench/e2e/perf.py`. Command wiring is centralized in `build_parser()` and dispatch in `main()`. Current command-output behavior is embedded directly in `cmd_list`, `cmd_run`, and `cmd_compare` via `print(...)` calls. The test catalog is currently loaded by `load_tests()` from a hard-coded constant `TESTS_PATH`.

Generated report markdown is produced by `render_report(...)` and written by `write_report(...)` into a per-run `README.md`. This file lives inside each run directory and is committed for baseline runs under `bench/e2e/runs/baseline/README.md`.

Harness tests live in `bench/e2e/test_perf.py` and currently focus on helper behavior and compare edge-cases; they do not yet lock the new CLI surface (`--tests`, `--verbose`) or output-contract reductions requested here.

Automation wiring lives in `.pre-commit-config.yaml` and `Makefile`. The existing pre-commit hook validates Python harness unit tests (`make test-bench-e2e`) but does not run a real `perf.py run/compare` smoke scenario.

In this plan, "quiet mode" means no progress logs while work is in flight, with only a short terminal outcome line for success/failure and a hint to use `--verbose` for detailed logs.

## Open Questions

No blocking questions remain. Scope and acceptance are explicit.

## Plan of Work

Milestone 1 establishes the CLI contract and parsing primitives with tests-first discipline. At the end of this milestone, parser and loader behavior support external test catalogs and verbosity toggles, and failing tests prove the new surface before implementation is completed.

Milestone 2 applies output behavior changes and report/list simplification. At the end of this milestone, `run` and `compare` are quiet by default, `list` emits names only, and generated reports no longer include the run-directory header line.

Milestone 3 introduces the pre-commit smoke catalog and automation integration. At the end of this milestone, a lightweight cross-category benchmark subset is executable through Make and pre-commit, and the smoke workflow always exercises both `run` and `compare`.

Milestone 4 closes with full validation and the mandatory two-pass independent review cycle.

### Concrete Steps

Run all commands from `/workspaces/fix-bench-e2e-qol`.

1. TDD red phase for new CLI contracts and output contracts.

   Update `bench/e2e/test_perf.py` first to add failing tests for:

   - parser accepts `run --tests <path>` and `list --tests <path>` with default still pointing to `bench/e2e/tests.json`;
   - parser accepts `run -v/--verbose` and `compare -v/--verbose` with default `False`;
   - `cmd_list(...)` emits only test names (one per line), no category/runs/warmup/preview command, no `total=` footer;
   - `render_report(...)` output omits the `- Run directory:` line;
   - `cmd_run(...)` and `cmd_compare(...)` in non-verbose mode suppress per-test/progress/detail logs and keep only concise terminal outcome messaging;
   - `--tests` invalid path and malformed JSON still fail deterministically with existing `error: tests:` diagnostics.
   - relative `--tests` paths resolve from the current working directory (including non-repo-root invocation contexts).

   Targeted red-phase command:

       python3 -m unittest bench.e2e.test_perf

   Expected red evidence: at least one of the new tests fails due to missing `--tests`/`--verbose` flags and legacy noisy output.

   Example red transcript:

       FAIL: test_run_parser_accepts_tests_flag (...)
       AssertionError: SystemExit not raised
       FAIL: test_cmd_list_outputs_names_only (...)

2. Implement catalog-path and verbosity surface in `bench/e2e/perf.py`.

   - Refactor `load_tests()` to accept a path argument (for example `tests_path: pathlib.Path`) and keep default behavior by passing `TESTS_PATH` from call sites.
   - Normalize `args.tests` with `normalize_path(...)` inside `cmd_list` and `cmd_run` before loading catalog content.
   - Extend parser:
     - `list --tests` (default `str(TESTS_PATH)`),
     - `run --tests` (default `str(TESTS_PATH)`),
     - `run -v/--verbose`,
     - `compare -v/--verbose`.
   - Route `cmd_list` and `cmd_run` through the provided tests path.
   - Keep `report` behavior unchanged unless a small follow-on adjustment is required to preserve internal consistency.

3. Implement output-contract changes in `bench/e2e/perf.py`.

   - In `cmd_list`, print only test names.
   - In `cmd_run`, gate existing progress and warning prints behind `args.verbose`.
   - In `cmd_compare`, gate detailed warning/error listings behind `args.verbose`.
   - Add concise non-verbose terminal outcome lines for `run` and `compare` success/failure that include a hint to rerun with `--verbose` for details.
   - Preserve exit codes and failure semantics.
   - In `render_report(...)`, remove the `- Run directory: ...` header line while keeping the rest of report metadata intact.

4. Add pre-commit smoke catalog and automation wiring.

   - Create `bench/e2e/tests_commit.json` with this explicit subset (2 tests per category):
     - `change_scr1_coremark_imem_axi_1sig_to_1000ps`
     - `change_scr1_signals_1_window_2ns_trigger_any`
     - `info_picorv32_ez`
     - `info_scr1_isr_sample`
     - `signal_scr1_top_recursive_depth2_json`
     - `signal_scr1_top_recursive_filter_valid_json`
     - `value_scr1_signals_1`
     - `value_scr1_signals_10`
   - Set `runs=1` and `warmup=0` for each `tests_commit.json` entry to bound pre-commit latency while preserving real end-to-end execution coverage.
   - Keep command vectors aligned with matching entries in `bench/e2e/tests.json`.
   - Add/adjust tests in `bench/e2e/test_perf.py` (or a new targeted harness test file) to lock deterministic catalog contract:
     - exact set of the 8 names listed above,
     - exactly 2 tests per category,
     - `runs=1` and `warmup=0` for every entry,
     - name+command parity against `tests.json`.
   - Add a new Make target (for example `bench-e2e-smoke-commit`) that:
     - runs `perf.py run --tests bench/e2e/tests_commit.json` into an isolated temporary revised directory;
     - then runs `perf.py compare --revised <tmp-revised-dir> --golden bench/e2e/runs/baseline --max-negative-delta-pct 100`.
   - Require the Make target to depend on `check-rtl-artifacts build-release` and run both commands with `WAVEPEEK_BIN="$(WAVEPEEK_RELEASE_BIN)"` to avoid PATH drift.
   - Do not add `--tests` to `compare`; compare consumes artifact directories, so catalog scoping is applied at run-generation time.
   - Keep threshold `100` intentionally non-gating for timing volatility; smoke objective is script/harness contract health, not performance regression policing.
   - Wire a new pre-commit hook in `.pre-commit-config.yaml` to execute this target.

5. Commit atomic units.

   Suggested commit split:

       git add bench/e2e/test_perf.py
       git commit -m "test(bench): lock perf cli quiet and catalog contracts"

       git add bench/e2e/perf.py bench/e2e/runs/baseline/README.md
       git commit -m "feat(bench): add quiet defaults and tests catalog flag"

       git add bench/e2e/tests_commit.json Makefile .pre-commit-config.yaml bench/e2e/test_perf.py
       git commit -m "chore(bench): add pre-commit perf smoke catalog"

6. Validation gates.

       make test-bench-e2e
       make bench-e2e-smoke-commit
       pre-commit run bench-e2e-smoke-commit --all-files
       make check

   If environment time budget permits, also run:

       make ci

   Example green transcript:

       make test-bench-e2e
       ...
       OK
       make bench-e2e-smoke-commit
       ok: run: completed successfully (use --verbose for detailed logs)
       ok: compare: all checks passed (use --verbose for detailed logs)

7. Mandatory review cycle.

   - Run review pass #1 (`review` agent), fix findings, commit fixes.
   - Run a fresh-context review pass #2 (`review` agent), fix findings if any, commit fixes.
   - Re-run `make test-bench-e2e` and `make bench-e2e-smoke-commit` after final review fixes.

### Validation and Acceptance

Acceptance is complete only when all conditions below are true:

- `python3 bench/e2e/perf.py list` prints only test names, one per line, with no summary/footer and no extra columns.
- `python3 bench/e2e/perf.py run` emits no progress/detail logs during execution in default mode; completion message is concise and points to `--verbose` for details.
- `python3 bench/e2e/perf.py run --verbose` preserves rich progress/detail output equivalent to the current behavior baseline.
- `python3 bench/e2e/perf.py compare` in default mode emits concise outcome-only messaging with `--verbose` hint, while preserving exit code behavior.
- `python3 bench/e2e/perf.py compare --verbose` prints detailed warnings/errors comparable to current behavior.
- `python3 bench/e2e/perf.py run --tests bench/e2e/tests_commit.json` and `python3 bench/e2e/perf.py list --tests bench/e2e/tests_commit.json` both work.
- `--tests` relative-path behavior is deterministic: path resolution uses current working directory semantics for both `list` and `run`.
- `python3 bench/e2e/perf.py run --tests /bad/path.json` and malformed-json catalogs fail with deterministic `error: tests:` diagnostics.
- Generated report markdown no longer contains `- Run directory:` and still contains the remaining metadata/table sections.
- `.pre-commit-config.yaml` includes a hook that executes the new Make target for the smoke catalog.
- New Make target executes both `run` and `compare` for `tests_commit.json` and fails fast with useful status on broken harness behavior.
- New Make target depends on `check-rtl-artifacts` and `build-release`, and pins `WAVEPEEK_BIN` to `$(WAVEPEEK_RELEASE_BIN)` for deterministic binary selection.
- `make bench-e2e-smoke-commit` completes in <= 180 seconds in the devcontainer reference environment.
- `tests_commit.json` contract tests enforce exact 8-name set, 2-per-category distribution, and `runs=1`/`warmup=0` invariants.
- `pre-commit run bench-e2e-smoke-commit --all-files` passes in the devcontainer.
- `make test-bench-e2e` passes, then review pass #1 and independent review pass #2 are both clean (or findings fixed and rechecked).

TDD acceptance requirement: at least one new/updated harness test fails before implementation and passes after implementation in the same branch history.

### Idempotence and Recovery

All edits are additive and safe to rerun. The pre-commit smoke target must create and clean temporary run directories so repeated invocations do not leave stateful artifacts. If a hook run fails after partial artifact generation, rerun after deleting the temporary directory or rely on trap-based cleanup in the Make target.

If noisy logging leaks into default mode, compare non-verbose and verbose outputs side-by-side and move remaining detail prints behind verbosity checks. Preserve one-line outcome messaging in all cases.

### Artifacts and Notes

Expected touched paths for implementation:

- `bench/e2e/perf.py`
- `bench/e2e/test_perf.py`
- `bench/e2e/tests_commit.json` (new)
- `.pre-commit-config.yaml`
- `Makefile`
- `bench/e2e/runs/baseline/README.md` (or whichever committed run report fixture is used as canonical expected output)

Capture closure evidence in implementation commits:

- Red-phase unittest failure snippets proving new contracts were initially unmet.
- Green-phase `make test-bench-e2e` output.
- Green-phase `make bench-e2e-smoke-commit` output.
- Review pass #1 findings and fixes.
- Independent review pass #2 findings and fixes.

### Interfaces and Dependencies

No new third-party dependencies are expected.

Interface changes after implementation:

- `bench/e2e/perf.py` parser adds:
  - `list --tests <json-path>`
  - `run --tests <json-path>`
  - `run -v/--verbose`
  - `compare -v/--verbose`

- `load_tests` accepts a path parameter instead of relying on only a module constant.

- `list` output contract becomes strictly `test_name` per line.

- `run` and `compare` output contracts become quiet-by-default with explicit verbose opt-in.

- Pre-commit interface gains a benchmark smoke hook backed by `tests_commit.json` and a dedicated Make target.

Revision Note: 2026-03-05 / OpenCode - Created active ExecPlan for bench/e2e perf QoL improvements; implementation intentionally deferred per request.
Revision Note: 2026-03-05 / OpenCode - Incorporated pass #1 review fixes: mandatory run+compare smoke flow, deterministic 8-test commit catalog, negative-path test coverage, and expected red/green transcripts.
Revision Note: 2026-03-05 / OpenCode - Incorporated independent pass #2 review fixes: explicit Make target prerequisites and binary pinning, explicit `--tests` path-resolution semantics, deterministic catalog invariants, and hook-level validation command.
