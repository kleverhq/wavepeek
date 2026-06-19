# Manual Performance Gate Without Committed Baselines

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, maintainers can screen for regressions on selected release benchmarks by comparing two explicit source revisions in the same local or controlled machine environment instead of trusting committed benchmark snapshots or noisy GitHub-hosted runners. A maintainer will run a command such as `just bench-gate v0.5.0 HEAD`, which will clone both revisions into `tmp/`, capture the end-to-end command benchmarks and expression microbenchmarks for each revision, optionally capture FSDB benchmarks when the Synopsys Verdi FSDB Reader SDK is available, compare revised results against the baseline thresholds, and write reviewable artifacts under `tmp/bench-gate/`.

The visible outcome is a smaller repository with benchmark run artifacts no longer tracked in git, plus three public benchmark entrypoints: one to capture performance for a ref, one to compare two captures, and one to perform the full baseline-ref versus revised-ref gate. The release runbook will require this manual gate for major and minor releases. For patch releases, maintainers must either run the gate when the patch is expected to affect performance or record why the gate was skipped for clearly non-performance changes such as documentation-only or metadata-only updates.

## Non-Goals

This work will not add a GitHub Actions performance workflow. This work will not make performance benchmarks part of default pre-merge CI. This work will not attempt to solve statistical noise with retries, self-hosted runners, or per-test tolerance bands. This work will not change the behavior of the `wavepeek` CLI itself. This work will not require Verdi; FSDB benchmark capture remains optional and is skipped when the local Verdi environment is unavailable.

## Progress

- [x] (2026-06-18 00:00Z) Rejected the original CI benchmark workflow direction in GitHub issue 24 and recorded the manual gate direction in an issue comment.
- [x] (2026-06-18 00:10Z) Read the benchmark harnesses, justfile recipes, release docs, and local breadcrumb guidance relevant to benchmark, docs, tools, workflow, and WIP files.
- [x] (2026-06-18 00:20Z) Drafted this self-contained ExecPlan for review.
- [x] (2026-06-18 00:35Z) Requested architecture/code and docs/release-process reviews of the plan, accepted findings, and revised the plan for artifact-set identity checks, FSDB pairing policy, dirty-worktree handling, and release documentation scope.
- [ ] Implement `tools/bench/gate.py` and its unit tests.
- [ ] Update `justfile` so public benchmark recipes are `bench-gate`, `bench-capture`, and `bench-compare`, while lower-level harness recipes are private and no longer refresh committed baselines.
- [ ] Remove tracked benchmark run baselines from `bench/e2e/runs/` and `bench/expr/runs/`, while preserving `.gitignore` files and any ignored local user runs.
- [ ] Update maintainer docs and benchmark breadcrumbs to describe the manual gate, optional FSDB behavior, patch-release skip recording, and the lack of committed baseline runs.
- [ ] Run focused tests for the new helper and benchmark harness tests, then run repository gates as far as practical.
- [ ] Request implementation review, apply fixes, and run a final control review.
- [ ] Commit logical milestones as they become stable.

## Surprises & Discoveries

- Observation: The repository currently tracks 591 files under benchmark run directories, including FST, FSDB, and expression baseline artifacts.
  Evidence: `git ls-files 'bench/e2e/runs/*' 'bench/expr/runs/*' | wc -l` printed `591`.
- Observation: `just bench-e2e-smoke-commit` currently depends on committed FST and FSDB baselines for comparison.
  Evidence: `justfile` invokes `bench/e2e/perf.py compare --golden "{{ bench_e2e_baseline_dir }}"` and FSDB functional-only compare against `{{ bench_e2e_fsdb_baseline_dir }}`.
- Observation: The expression benchmark harness already writes useful provenance into `summary.json`, including source commit, worktree state, Cargo version, Rust compiler version, Criterion version, and environment note.
  Evidence: `bench/expr/perf.py` function `build_summary` writes those fields.
- Observation: The end-to-end comparator accepts partial intersections and reports missing tests as warnings rather than identity failures.
  Evidence: Plan review found that `bench/e2e/perf.py compare` warns on revised-only or golden-only tests; the gate must add its own artifact-name equality check before treating a comparison as passing.
- Observation: FSDB capture needs more than Verdi detection.
  Evidence: Existing recipes prepare/check FSDB RTL artifacts, build with `--features fsdb`, and use `target/fsdb/release/wavepeek`; the new helper must mirror that flow rather than reusing the non-FSDB release binary.

## Decision Log

- Decision: Replace committed baseline artifacts with explicit baseline source revisions selected at gate time.
  Rationale: A baseline ref is smaller, reviewable, and naturally tied to releases. It avoids storing large generated JSON/CSV snapshots while still allowing maintainers to compare two revisions in one environment.
  Date/Author: 2026-06-18 / Grin
- Decision: Do not add a GitHub Actions benchmark workflow for this issue.
  Rationale: GitHub-hosted runners are noisy, and a heavy manual benchmark workflow would add friction for forks and contributors without producing authoritative performance decisions.
  Date/Author: 2026-06-18 / Grin
- Decision: Keep FSDB benchmark capture optional by default with an `auto` mode, but treat asymmetric support as a comparison failure.
  Rationale: FSDB requires a local Synopsys Verdi installation that many contributors and CI environments do not have. The gate should capture FSDB performance when possible and explicitly record a skip when Verdi is unavailable or when both compared refs lack FSDB support. If Verdi is available and only one ref supports FSDB, silently skipping would hide a performance or feature regression; that should fail unless the maintainer explicitly selects a skip mode.
  Date/Author: 2026-06-18 / Grin
- Decision: Implement the new orchestration in `tools/bench/gate.py` instead of embedding shell logic in the `justfile`.
  Rationale: The operation requires cloning refs, running multiple commands with logs, generating manifests, handling optional FSDB, and aggregating comparison results. That is too much control flow for a maintainable `justfile` recipe.
  Date/Author: 2026-06-18 / Grin
- Decision: The helper must fail by default when the source repository is dirty and the requested ref is `HEAD` or resolves to the current `HEAD`, with an explicit override for intentional committed-state comparisons.
  Rationale: The gate clones committed refs. Ignoring uncommitted local changes would give maintainers false confidence by benchmarking code that is not the code in their working tree.
  Date/Author: 2026-06-18 / Grin
- Decision: The gate must check end-to-end artifact set identity before running timing comparisons.
  Rationale: The existing end-to-end comparator can pass a partial overlap. Release gating needs to fail when golden and revised captures contain different test artifact names unless an explicit override is added later.
  Date/Author: 2026-06-18 / Grin

## Outcomes & Retrospective

No implementation outcome yet. The intended result is a manual gate that produces comparable artifacts outside git and keeps the repository free of generated baseline runs. This section must be updated after each implementation and review milestone.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. Performance coverage currently lives under `bench/`. The end-to-end benchmark harness is `bench/e2e/perf.py`; it reads JSON benchmark catalogs such as `bench/e2e/tests.json`, runs the `wavepeek` binary through `hyperfine`, captures functional `wavepeek --json` payloads, writes one `*.hyperfine.json` and one `*.wavepeek.json` per test, writes `README.md`, and can compare two run directories with a maximum allowed negative timing delta. A negative timing delta means the revised run is slower than the golden run. The default FST catalog contains 142 tests. The FSDB catalog is `bench/e2e/tests_fsdb.json`; FSDB means Fast Signal Database, a proprietary waveform format read through the optional Synopsys Verdi FSDB Reader SDK.

The expression microbenchmark harness is `bench/expr/perf.py`; it runs Rust Criterion benchmark targets registered in `Cargo.toml`, exports one raw CSV per scenario, writes `summary.json`, writes `README.md`, and can compare two expression run directories against a negative delta threshold. Criterion is a Rust benchmarking library. The expression catalog is `bench/expr/suites.json`; it currently contains four suites and seventeen scenarios.

The root `justfile` is the stable local automation interface. Today it exposes recipes such as `bench-e2e-update-baseline`, `bench-e2e-run`, `bench-e2e-fsdb-update-baseline`, `bench-e2e-fsdb-run`, `bench-expr-update-baseline`, and `bench-expr-run`. Some of those recipes refresh committed baseline directories. The new design should stop refreshing committed baseline directories and should leave public entrypoints at the higher manual-gate level.

Tracked generated benchmark artifacts currently live in `bench/e2e/runs/baseline_fst/`, `bench/e2e/runs/baseline_fsdb/`, and `bench/expr/runs/baseline/`. The `.gitignore` files under `bench/e2e/runs/` and `bench/expr/runs/` currently unignore those baseline directories. The new design should keep only `.gitignore` files in those run directories and should ignore generated run artifacts by default.

The new helper group will live under `tools/bench/`. Repository guidance says helper automation should be deterministic, non-interactive, safe for CI-like environments, tested next to the helper group, and invoked through `just` recipes where possible. Because this helper group has workflow-like behavior and more than one file, it should include a concise `tools/bench/README.md`.

The release runbook is `docs/dev/release.md`. It currently has no performance gate step. The benchmark guide is `docs/dev/benchmarking.md`. It currently describes committed baselines. The FSDB guide is `docs/dev/fsdb.md`; it currently references FSDB benchmark baselines. These docs must be updated so a future maintainer can run and interpret the manual gate without reading this plan.

## Open Questions

There are no blocking open questions. The implementation will use these defaults: FST end-to-end timing threshold defaults to 5 percent negative delta, expression timing threshold defaults to 15 percent negative delta, FSDB end-to-end timing threshold defaults to the same 5 percent negative delta as FST, and expression comparison will not require matching Criterion crate versions because comparing old and new source revisions may legitimately cross dependency updates. Catalog fingerprint mismatches should still fail expression comparison because changed benchmark scenarios are not apples-to-apples.

## Plan of Work

The first milestone is to review this design. The plan should be checked for missing release-policy concerns, misuse of existing benchmark harnesses, and accidental contributor friction. Any accepted finding should be recorded in the `Decision Log` and reflected throughout the plan.

The second milestone is to add `tools/bench/gate.py`. The helper will be Python standard-library only. It will provide three subcommands. `capture` resolves a ref in the current repository, refuses to benchmark `HEAD` from a dirty source repository unless an explicit `--allow-dirty-source` override is provided, clones that ref to a checkout under an output directory, runs FST end-to-end and expression benchmarks in that checkout, optionally runs FSDB end-to-end benchmarks if support files and Verdi are available, and writes a manifest and README summary. Capture subprocesses must run with `cwd` set to the cloned checkout and must invoke the benchmark harness files inside that checkout so the selected ref supplies the catalogs and benchmark source. `compare` compares two capture directories, checks end-to-end artifact set identity, and writes comparison logs and a comparison summary. `gate` resolves a baseline ref and revised ref, clones both, captures both, compares them, and exits non-zero when required benchmark comparisons fail.

The helper will never delete arbitrary existing paths. A requested output directory must be absent or empty. Default output goes under repository-root `tmp/bench-gate/`, which is already the approved scratch location. The helper will keep checkouts with the output so maintainers can inspect the exact source trees used for the run. It will write command logs under each capture directory and under the comparison directory.

The third milestone is to change automation. Public `just` recipes should become `bench-gate baseline_ref revised_ref="HEAD"`, `bench-capture ref="HEAD"`, and `bench-compare golden_dir revised_dir`. Existing low-level benchmark recipes may remain for development but should be private and should not mention committed baseline update flows. The pre-commit benchmark smoke recipe should still run selected benchmark commands and validate artifacts, but it must not compare against deleted baselines.

The fourth milestone is to remove committed benchmark baselines. Use `git rm -r bench/e2e/runs/baseline_fst bench/e2e/runs/baseline_fsdb bench/expr/runs/baseline`, then update `bench/e2e/runs/.gitignore` and `bench/expr/runs/.gitignore` so only `.gitignore` remains tracked under those directories. Do not remove ignored local run directories such as timestamped directories under `bench/e2e/runs/`; they may belong to the user or another agent.

The fifth milestone is documentation. Rewrite `docs/dev/benchmarking.md` so it explains the manual gate, the capture directory structure, thresholds, optional FSDB behavior, patch-release skip recording, and release baseline policy. Update `docs/dev/automation.md`, `docs/dev/quality.md`, `docs/dev/release.md`, `docs/dev/fsdb.md`, `bench/AGENTS.md`, and `bench/expr/AGENTS.md` to remove committed-baseline language and point to the manual gate.

The sixth milestone is validation and review. Run unit tests for `tools/bench`, `bench/e2e`, and `bench/expr`. Run `just format-justfile-check`, `just check-actions`, and a broader `just check` or `just ci` if time and environment allow. Request focused review of the implementation and docs, fix findings, then request one final control review of the consolidated diff.

### Concrete Steps

From repository root `/workspaces/wavepeek`, create `tools/bench/README.md` with a short description of the manual performance gate helper. Create `tools/bench/gate.py` with the following public command shapes:

    python3 -B tools/bench/gate.py capture --ref HEAD
    python3 -B tools/bench/gate.py compare --golden tmp/bench-gate/<run>/baseline --revised tmp/bench-gate/<run>/revised
    python3 -B tools/bench/gate.py gate --baseline-ref v0.5.0 --revised-ref HEAD

The command parser should also expose `--fsdb auto|always|never`, threshold flags for FST end-to-end, FSDB end-to-end, and expression benchmarks, and `--allow-dirty-source` for cases where a maintainer intentionally wants to benchmark committed `HEAD` while the working tree contains unrelated edits.

Add `tools/bench/test_gate.py` with mocked command execution tests for parser defaults, output directory safety, dirty-source enforcement, optional FSDB skip behavior, asymmetric FSDB comparison handling, end-to-end artifact identity checks, compare-suite selection, and manifest or summary writing. Update `just test-aux` to discover `tools/bench` tests.

Edit the `justfile` so public benchmark recipes call the helper:

    bench-gate baseline_ref revised_ref="HEAD": require-container
        {{ python }} tools/bench/gate.py gate --baseline-ref "{{ baseline_ref }}" --revised-ref "{{ revised_ref }}"

    bench-capture ref="HEAD": require-container
        {{ python }} tools/bench/gate.py capture --ref "{{ ref }}"

    bench-compare golden_dir revised_dir: require-container
        {{ python }} tools/bench/gate.py compare --golden "{{ golden_dir }}" --revised "{{ revised_dir }}"

Keep low-level benchmark recipes private or remove them when they only refreshed committed baselines. Update smoke recipes so they do not depend on `bench/e2e/runs/baseline_fst` or `bench/e2e/runs/baseline_fsdb`.

Remove tracked run baselines with git, not with broad filesystem deletion:

    git rm -r bench/e2e/runs/baseline_fst bench/e2e/runs/baseline_fsdb bench/expr/runs/baseline

Then edit `bench/e2e/runs/.gitignore` and `bench/expr/runs/.gitignore` to contain only:

    *
    !.gitignore

Run focused tests:

    python3 -B -m unittest discover -s tools/bench -p 'test_*.py'
    python3 -B -m unittest discover -s bench/e2e -p 'test_*.py'
    python3 -B -m unittest discover -s bench/expr -p 'test_*.py'
    just format-justfile-check

If the environment supports the full project gates, run:

    just check

### Validation and Acceptance

A maintainer can verify the new behavior without running full benchmarks by running the new unit tests and checking parser output. Full acceptance in a release-like environment is to run:

    just bench-gate <previous-release-tag> HEAD

The command should create a new directory under `tmp/bench-gate/`. Inside it, `baseline/` and `revised/` should each contain a `manifest.json`, a `README.md`, `logs/`, `e2e-fst/`, and `expr/`. If Verdi is available and the checked-out source supports FSDB, each capture should also contain `e2e-fsdb/`; otherwise the manifest and README should say FSDB was skipped. The gate-level directory should contain `compare/`, `manifest.json`, and `summary.md`. The command should exit zero only when all required comparisons pass within thresholds, and non-zero when any required comparison fails.

The repository should no longer track generated benchmark run artifacts. Running `git ls-files 'bench/e2e/runs/*' 'bench/expr/runs/*'` after the removal should list only the two `.gitignore` files.

The release runbook should tell maintainers to run the manual performance gate for major and minor releases. For patch releases, it should tell maintainers to either run the gate when the patch likely affects performance or record a skip rationale when the patch is clearly unrelated to performance, such as documentation-only or release-metadata-only changes. The benchmark guide should no longer describe committed baseline refresh recipes.

### Idempotence and Recovery

The helper must be safe to run repeatedly because each default run creates a unique timestamped output directory under `tmp/bench-gate/`. If a user passes an explicit output directory and it already contains files, the helper should fail before cloning or running benchmarks. The safe recovery is to choose a new output directory or manually remove the old one after confirming it is disposable. The implementation must not delete arbitrary paths automatically.

If benchmark capture fails halfway, the partial output directory and logs should remain for diagnosis. A maintainer can rerun the command with a fresh output directory after fixing the problem. Because tracked baseline directories are removed with `git rm`, rollback is simply `git restore --staged --worktree bench/e2e/runs bench/expr/runs` before committing if the change must be abandoned.

### Artifacts and Notes

Issue 24 has been updated with the rejected CI direction and selected manual gate direction. The comment URL is `https://github.com/kleverhq/wavepeek/issues/24#issuecomment-4748912184`.

A concise expected successful unit-test transcript after the helper tests exist is:

    $ python3 -B -m unittest discover -s tools/bench -p 'test_*.py'
    ........
    ----------------------------------------------------------------------
    Ran 8 tests in ...s
    OK

The exact number of tests may differ as implementation details settle, but failures in output directory safety, FSDB skip handling, or comparison aggregation must be treated as blocking.

### Interfaces and Dependencies

`tools/bench/gate.py` must use only Python standard-library modules such as `argparse`, `dataclasses`, `datetime`, `json`, `os`, `pathlib`, `shutil`, `subprocess`, `sys`, and `typing`. It must not add Python package dependencies.

The helper should define a `main(argv: list[str] | None = None) -> int` entrypoint and return process exit codes rather than calling `sys.exit` deep inside helper functions, except for top-level CLI wiring. Testable functions should include output directory preparation, git ref resolution, FSDB availability detection, capture orchestration, comparison orchestration, and summary rendering.

The helper should invoke existing project tools by path rather than duplicating benchmark logic. For FST capture it should build the release binary and run this command with `cwd` set to `<checkout>` so `bench/e2e/perf.py` comes from the selected source ref:

    WAVEPEEK_BIN=<checkout>/target/release/wavepeek python3 -B bench/e2e/perf.py run --run-dir <capture>/e2e-fst

For expression capture it should run this command with `cwd` set to `<checkout>` so `bench/expr/perf.py`, `Cargo.toml`, and Criterion benchmark sources come from the selected source ref:

    python3 -B bench/expr/perf.py run --run-dir <capture>/expr --environment-note "wavepeek manual performance gate"

For FSDB capture in auto or always mode it should first check whether `tools/fsdb/check_fsdb_env.py`, `tools/fsdb/generate_bench_catalog.py`, `tools/fsdb/prepare_fsdb_fixtures.sh`, `tools/fsdb/check_fsdb_bench_artifacts.py`, and `bench/e2e/tests_fsdb.json` exist in the checkout. If any are missing, auto mode should mark that side as FSDB-unsupported and always mode should fail. If files exist, it should run `python3 -B tools/fsdb/check_fsdb_env.py` in the checkout; exit code 0 means the Verdi environment is available, exit code 77 means Verdi is unavailable and auto mode should skip FSDB for the whole gate, and any other exit code should fail. When FSDB capture proceeds, the helper must prepare and validate FSDB RTL artifacts, build with `CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb`, and run:

    WAVEPEEK_BIN=<checkout>/target/fsdb/release/wavepeek python3 -B bench/e2e/perf.py run --tests bench/e2e/tests_fsdb.json --run-dir <capture>/e2e-fsdb

For FST and FSDB comparison it should first list matching `*.hyperfine.json` and `*.wavepeek.json` stems in both capture directories and fail if the golden and revised sets differ or if either side is empty. Only after that identity check should it invoke `bench/e2e/perf.py compare` from the current repository. For expression comparison it should invoke `bench/expr/perf.py compare` from the current repository. The current repository is the repository containing `tools/bench/gate.py`, not necessarily either checkout being compared. This keeps `just bench-compare` usable when only capture directories are provided.

### Revision Notes

- 2026-06-18 / Grin: Initial plan created after discussion switched issue 24 away from CI-collected benchmark artifacts and toward a manual source-ref comparison gate.
- 2026-06-18 / Grin: Revised plan after architecture/code and docs/release-process review. Added dirty-source handling, e2e artifact identity checks, explicit FSDB build/preparation flow, pairwise FSDB skip policy, `docs/dev/fsdb.md` updates, and patch-release skip recording.
