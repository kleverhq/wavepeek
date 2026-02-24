# Add Hyperfine-Based CLI E2E Benchmark Runner

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

We need a reproducible benchmark harness for wavepeek CLI end-to-end scenarios so contributors can run the same matrix of `info`, `at`, and `change` checks and compare performance across revisions. After this work, a single Python script (stdlib only, `argparse`) will orchestrate runs via `hyperfine`, store per-test artifacts (`*.wavepeek.json` and `*.hyperfine.json`), generate a run-local `README.md` report, and enforce regression budgets in CI via a `compare` mode.

User-visible outcome: from repository root, a user can run one command to execute filtered benchmark tests, one command to regenerate the report, and one command to fail fast on regressions versus a golden run.

## Non-Goals

This plan does not implement final real wavepeek benchmark arguments for production datasets. For this iteration, benchmarked command execution uses `echo "dummy args"` to debug harness plumbing.

This plan does not modify the active `change` performance optimization plan in `docs/exec-plans/active/2026-02-24-change-performance-stateless/PLAN.md`.

This plan does not add third-party Python dependencies (no `pytest`, no templating libs, no CLI frameworks beyond `argparse`).

## Progress

- [x] (2026-02-24 15:20Z) Explored repository conventions, script locations, and existing perf-plan context.
- [x] (2026-02-24 15:27Z) Authored this execution plan with explicit requirements, assumptions, and validation strategy.
- [x] (2026-02-24 15:36Z) Ran pre-implementation review and tightened CLI/report/compare contracts.
- [ ] Implement benchmark runner with `run`, `list`, `report`, and `compare` commands.
- [ ] Add Python unit tests first (TDD), then implement to green.
- [ ] Run validation commands and update docs/changelog collateral.
- [ ] Run mandatory review pass #1 (implementation), apply fixes, and commit.
- [ ] Run mandatory fresh review pass #2 (implementation), apply fixes if needed, and commit.

## Surprises & Discoveries

- Observation: repository has no existing benchmark harness directory (`benches/` or `tests/perf/`) for this use case, so a script-first addition under `scripts/` fits current conventions.
  Evidence: repository scan showed only `scripts/check_schema_contract.py` as existing Python utility.

- Observation: an active perf plan already exists for `change`, but this task is orthogonal and should avoid touching it.
  Evidence: `docs/exec-plans/active/2026-02-24-change-performance-stateless/PLAN.md` is present and explicitly scoped to engine optimization.

## Decision Log

- Decision: place the new tool at `scripts/cli_e2e_bench.py` and keep it fully standalone.
  Rationale: existing repository convention keeps maintenance scripts in `scripts/` and avoids packaging overhead.
  Date/Author: 2026-02-24 / OpenCode

- Decision: use deterministic test catalog generation from Python dataclasses embedded in code.
  Rationale: user asked for dataclass-based matrix in Python and this keeps future migration to real arguments straightforward.
  Date/Author: 2026-02-24 / OpenCode

- Decision: define compare percentage as `((golden - revised) / golden) * 100` so negative values mean revised is slower.
  Rationale: matches user requirement to fail on "negative direction" regressions and aligns with lower-is-better timing metrics.
  Date/Author: 2026-02-24 / OpenCode

- Decision: keep current benchmark execution in dummy mode (`echo`) while still enforcing release-binary existence as a mandatory preflight.
  Rationale: user explicitly requested dummy command execution for harness debugging and also requested release binary existence/build check for future-ready runs.
  Date/Author: 2026-02-24 / OpenCode

## Outcomes & Retrospective

Implementation outcome is pending. Expected completion outcome is a runnable benchmark harness that can populate run artifacts and enforce regression thresholds from CLI.

## Context and Orientation

The repository is Rust-first and currently keeps utility Python scripts under `scripts/`. Existing quality gates (`make check`, `make ci`) do not include Python test harnesses by default, so this plan will run Python unit tests explicitly and report that validation in delivery notes.

Key files to add/update:

- `scripts/cli_e2e_bench.py`: new benchmark orchestrator.
- `scripts/tests/test_cli_e2e_bench.py`: unit tests for catalog generation, report rendering, and compare logic.
- `docs/DEVELOPMENT.md`: short usage section for the new benchmark harness commands.
- `CHANGELOG.md`: unreleased entry for the new tooling.

Important behavior constraints:

- `run` creates a timestamped run directory when explicit run dir is not provided.
- Each test writes two artifacts: `<test_name>.wavepeek.json` and `<test_name>.hyperfine.json`.
- `README.md` in run directory is always regenerated from discovered JSON artifacts.
- `run --compare <golden_dir>` adds percentage deltas into README display only.
- `run` supports regex filtering and can rewrite selected tests inside a non-empty run directory.
- `list` prints all available tests.
- `report` regenerates run README from existing artifacts.
- `compare` checks revised vs golden, supports threshold regression failure, and can enforce equality for `data` / `warning` fields in wavepeek JSONs when corresponding flags are set.

CLI contract to implement:

- `python3 scripts/cli_e2e_bench.py list [--filter REGEX]`
- `python3 scripts/cli_e2e_bench.py run [--run-dir PATH] [--out-dir PATH] [--filter REGEX] [--compare PATH] [--runs N] [--warmup N] [--wavepeek-bin PATH]`
- `python3 scripts/cli_e2e_bench.py report --run-dir PATH [--compare PATH]`
- `python3 scripts/cli_e2e_bench.py compare --revised PATH --golden PATH --max-negative-delta-pct FLOAT [--metric median] [--require-equal-field data|warning ...]`

README table contract to implement:

- Table `Benchmark Results` with fixed columns: `test`, `command`, `dimensions`, `mean_s`, `stddev_s`, `median_s`, `min_s`, `max_s`.
- When compare run is provided and matching test exists, each seconds column renders as `<value> (<delta%>)` where delta is `((golden-current)/golden)*100`, rounded to two decimals with explicit sign.
- Missing compare counterpart renders without suffix.
- Rows sorted lexicographically by `test` for deterministic diffs.

Test-matrix contract to implement (minimum complete baseline):

- `info`: 3 dump-size variants (small, medium, large).
- `at`: 3 dump-size variants x 4 signal-count variants (1, 10, 100, 1000) x 3 time-point variants (5%, 50%, 95%).
- `change`: 3 dump-size variants x 3 signal-count variants (1, 10, 100) x 3 window-position variants (5%, 50%, 95%) x 3 window-size variants (2000, 4000, 8000) x 3 trigger variants (`posedge clk`, `*`, `signal`).

## Open Questions

No blocking open questions remain. Non-critical ambiguity (exact production waveform args and datasets) is intentionally deferred and represented as dummy args in this milestone.

## Plan of Work

First, add unit tests that encode expected behavior for the test catalog size, README report generation with compare deltas, and compare-mode failure conditions. This is the TDD gate and should fail before implementation.

Next, implement `scripts/cli_e2e_bench.py` with modular pure functions for loading artifacts, generating Markdown, selecting tests by regex, and comparing runs. Keep command wiring in `argparse` subcommands and isolate subprocess calls for `hyperfine`, `cargo build --release`, and dummy command execution.

Wavepeek artifact contract in dummy mode:

- `*.wavepeek.json` is generated by the harness (not by wavepeek stdout), and must include `test_name`, `command`, `dims`, `invoked_utc`, `exit_code`, `stdout`, `stderr`, `data`, and `warning`.
- In dummy mode, `data` is set to the trimmed stdout of dummy command, and `warning` is `null`.
- `compare` with `--require-equal-field` checks exact JSON equality for requested field names.

Then, update docs/changelog so the new script is discoverable and recorded in unreleased changes.

Finally, run tests and script smoke checks, then run two independent review passes with the review agent and apply fixes in separate commits when needed.

### Concrete Steps

Run all commands from `/workspaces/wavepeek`.

1. Add failing unit tests first.

       python3 -m unittest discover -s scripts/tests -p 'test_*.py'

   Expected pre-implementation behavior: import/attribute failures or assertion failures for missing runner features.

2. Implement `scripts/cli_e2e_bench.py` and rerun tests.

       python3 -m unittest discover -s scripts/tests -p 'test_*.py'

3. Smoke-test CLI commands with filtered run to keep execution small.

       python3 scripts/cli_e2e_bench.py list
       python3 scripts/cli_e2e_bench.py run --filter '^info_' --runs 2 --warmup 1
       python3 scripts/cli_e2e_bench.py report --run-dir <generated_run_dir>
       python3 scripts/cli_e2e_bench.py report --run-dir <generated_run_dir> --compare <generated_run_dir>

4. Validate compare behavior with synthetic or generated runs.

       python3 scripts/cli_e2e_bench.py compare --revised <revised_dir> --golden <golden_dir> --max-negative-delta-pct 5 --require-equal-field data --require-equal-field warning

5. Update `docs/DEVELOPMENT.md` and `CHANGELOG.md`, then run focused regression checks.

       python3 -m unittest discover -s scripts/tests -p 'test_*.py'

### Validation and Acceptance

Acceptance criteria:

- `list` shows complete dataclass-backed matrix (`info`, `at`, `change`) with stable test names.
- `run` writes paired JSON artifacts and regenerates run `README.md`.
- `run --filter` executes only matching tests; rerunning in same directory rewrites selected JSON files and README.
- `report` rebuilds README from files present in run directory.
- `report` and `run --compare` annotate timing cells with percentage deltas relative to compare run when matching tests exist.
- `compare` exits non-zero and prints all violating tests when regression exceeds threshold in negative direction.
- `compare` allows revised and golden to have different test sets; it evaluates all revised tests with available golden counterparts.
- `compare` enforces equality for requested wavepeek fields (`data`, `warning`).
- `run` always performs release-binary preflight: if binary path does not exist, run `cargo build --release` and fail with actionable error if build fails.

### Idempotence and Recovery

`list` and `report` are read/compute operations (except report writing `README.md`) and are safe to rerun.

`run` is idempotent per selected test names in a target run directory: reruns overwrite only the corresponding `*.wavepeek.json`, `*.hyperfine.json`, and final `README.md`.

If execution is interrupted, rerun `run` with the same `--run-dir` to refresh missing/incomplete test artifacts and regenerate a clean report.

### Artifacts and Notes

Expected artifact layout in a run directory:

    README.md
    info_size_small.wavepeek.json
    info_size_small.hyperfine.json
    at_size_small_signals_1_time_5.wavepeek.json
    at_size_small_signals_1_time_5.hyperfine.json
    ...

### Interfaces and Dependencies

Python interfaces to provide in `scripts/cli_e2e_bench.py`:

    @dataclass(frozen=True)
    class TestCase:
        name: str
        command: str
        wavepeek_args: tuple[str, ...]
        dims: dict[str, str]

    def build_test_catalog() -> list[TestCase]
    def select_tests(tests: list[TestCase], pattern: str | None) -> list[TestCase]
    def ensure_release_binary(binary_path: pathlib.Path) -> None
    def run_tests(...) -> pathlib.Path
    def render_report(run_dir: pathlib.Path, compare_dir: pathlib.Path | None = None) -> str
    def compare_runs(...) -> None

External dependencies are only command-line tools already expected in environment: `cargo` and `hyperfine`.

Revision Note: 2026-02-24 / OpenCode - Initial plan created for a standalone hyperfine-backed CLI E2E benchmark harness with TDD-first implementation and mandatory double review workflow.
Revision Note: 2026-02-24 / OpenCode - Incorporated pre-implementation review feedback: fixed CLI signatures, deterministic README table contract, explicit matrix coverage, dummy-mode wavepeek JSON contract, and release-binary preflight acceptance.
