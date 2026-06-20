# Manual Performance Gate Without Committed Baselines

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, maintainers can screen selected release benchmarks by comparing two explicit source revisions in one controlled local environment instead of trusting committed benchmark snapshots or noisy GitHub-hosted runners. A maintainer runs `just bench-gate <baseline-ref> <revised-ref>`, which clones both revisions under `tmp/bench-gate/`, builds both before measurement, prepares optional FSDB fixtures before FSDB measurement, captures FST end-to-end, FSDB end-to-end, and expression benchmark artifacts, then compares baseline versus revised results.

The gate compares like with like for timing: FST baseline versus FST revised, FSDB baseline versus FSDB revised, and expression baseline versus expression revised. It also performs cross-format FST-versus-FSDB functional checks inside each ref so FSDB payload parity is verified without pretending FST and FSDB timing are comparable. Generated benchmark artifacts are ignored local evidence, not repository source.

## Non-Goals

This work will not add a GitHub Actions performance workflow. This work will not make performance benchmarks part of default pre-merge CI. This work will not store benchmark baseline runs in git. This work will not compare FST timing against FSDB timing. This work will not solve noisy-machine benchmarking with retries, self-hosted runners, or statistical modeling. This work will not require Verdi; FSDB benchmark capture remains optional and is skipped when the local Verdi environment is unavailable.

## Progress

- [x] (2026-06-18 00:00Z) Rejected the original CI benchmark workflow direction in GitHub issue 24 and recorded the manual gate direction in an issue comment.
- [x] (2026-06-18 00:35Z) Reviewed the original plan and accepted findings for artifact-set identity checks, FSDB pairing policy, dirty-worktree handling, and release documentation scope.
- [x] (2026-06-18 03:30Z) Implemented and committed the first manual-gate version as `f3fd31c chore(bench): add manual performance gate`; it removed committed benchmark baselines and added public `just bench-gate`, `just bench-capture`, and `just bench-compare` entrypoints.
- [x] (2026-06-19 18:05Z) Reproduced a manual gate FSDB failure for `v1.0.0` on `change_scr1_signals_100_pos_50_window_2ns_trigger_any`, traced it to VCD-style scalar element names in the generated FSDB catalog, and added gate-local runnable FSDB catalog filtering.
- [x] (2026-06-19 22:50Z) Tested an interim follow-up that filtered FSDB scalar-element cases but over-corrected timing policy by making end-to-end comparison functional-only and broadening expression thresholds; this direction was superseded before final handoff.
- [x] (2026-06-19 23:20Z) Agreed on the corrected benchmark policy: build both refs first, prepare both fixture sets before measurement, run baseline/revised suites in same-format pairs, compare same-format timing at 5%, and run cross-format checks only as functional parity checks.
- [x] (2026-06-19 23:45Z) Split the overloaded helper into `tools/bench/common.py`, `tools/bench/capture.py`, `tools/bench/compare.py`, and a smaller `tools/bench/gate.py`; updated `justfile` to call the split scripts.
- [x] (2026-06-19 23:55Z) Replaced the old gate unit tests with module-focused tests covering parser defaults, dirty-HEAD rejection without override, FSDB filtering, same-format timing compare, cross-format functional compare, standalone capture behavior, and gate execution order.
- [x] (2026-06-20 00:10Z) Ran focused helper, e2e, expression, auxiliary, justfile-format, actionlint, and `just check` gates for the split helper; all passed.
- [x] (2026-06-20 00:20Z) Requested implementation and docs review for the refactor. Code review returned no substantive findings; docs review found one wording issue in `docs/dev/benchmarking.md`, which was fixed.
- [x] (2026-06-20 00:35Z) Re-ran `python3 -B -m unittest discover -s tools/bench -p 'test_*.py'` and `just check` after the docs wording fix; both passed.
- [x] (2026-06-20 00:40Z) Committed the refactor locally.

## Surprises & Discoveries

- Observation: The repository originally tracked 591 files under benchmark run directories, including FST, FSDB, and expression baseline artifacts.
  Evidence: `git ls-files 'bench/e2e/runs/*' 'bench/expr/runs/*' | wc -l` printed `591` before baseline removal.
- Observation: The end-to-end comparator can pass partial intersections if used directly.
  Evidence: Plan review found that `bench/e2e/perf.py compare` warns on revised-only or golden-only tests; the gate now checks end-to-end artifact sets before treating same-format comparisons as valid.
- Observation: FSDB capture needs more than Verdi detection.
  Evidence: Existing recipes prepare/check FSDB RTL artifacts, build with `--features fsdb`, and use `target/fsdb/release/wavepeek`; the gate mirrors that flow before running FSDB benchmarks.
- Observation: The full generated FSDB catalog contains VCD-style scalar element signal paths that converted RTL FSDB fixtures do not expose under the same canonical names.
  Evidence: Running the `v1.0.0` FSDB command for `change_scr1_signals_100_pos_50_window_2ns_trigger_any` failed with `fatal: signal: signal 'TOP.scr1_top_tb_axi.i_memory_tb.araddr.[0]' not found in dump`, while `wavepeek signal` listed `TOP.scr1_top_tb_axi.i_memory_tb.araddr[0] [31:0]`. Filtering tests with `.[` in command tokens skipped 12 of 142 FSDB tests and left 130 runnable FSDB tests.
- Observation: The previous full-gate timing failures happened while other repository work was running, so they are not reliable evidence for changing thresholds.
  Evidence: Logs from earlier attempts included cargo package-cache locking and concurrent work. The corrected policy returns to 5% same-format timing thresholds and treats noisy-machine failures as a reason to rerun in a quieter environment.

## Decision Log

- Decision: Replace committed baseline artifacts with explicit baseline source revisions selected at gate time.
  Rationale: A baseline ref is smaller, reviewable, and naturally tied to releases. It avoids storing generated JSON/CSV snapshots while still allowing maintainers to compare two revisions in one environment.
  Date/Author: 2026-06-18 / Grin
- Decision: Do not add a GitHub Actions benchmark workflow for this issue.
  Rationale: GitHub-hosted runners are noisy, and a heavy manual benchmark workflow would add friction for forks and contributors without producing authoritative performance decisions.
  Date/Author: 2026-06-18 / Grin
- Decision: Keep FSDB benchmark capture optional by default with an `auto` mode, but treat asymmetric support as a comparison failure.
  Rationale: FSDB requires a local Synopsys Verdi installation that many contributors and CI environments do not have. If Verdi is available and only one ref supports FSDB, silently skipping would hide a feature or performance regression; maintainers can explicitly choose `never` when they intentionally skip FSDB review.
  Date/Author: 2026-06-18 / Grin
- Decision: Split benchmark helper responsibilities across `common.py`, `capture.py`, `compare.py`, and `gate.py`.
  Rationale: The original single `gate.py` mixed capture, compare, gate orchestration, and shared utilities. Separate scripts keep the public `just` entrypoints simple while making each helper easier to test and review.
  Date/Author: 2026-06-19 / Grin
- Decision: Remove the dirty-source override.
  Rationale: The gate benchmarks committed refs. If a maintainer asks to benchmark `HEAD` while the source worktree is dirty, the helper must fail and ask for a commit or stash. Tags and explicit SHAs are unaffected by unrelated local edits.
  Date/Author: 2026-06-19 / Grin
- Decision: Build both refs and prepare both FSDB fixture sets before benchmark measurement, then run suites in baseline/revised pairs.
  Rationale: Build and fixture preparation are prerequisites, not performance signals. Pairing FST baseline/revised, FSDB baseline/revised, and expression baseline/revised reduces avoidable drift compared with a large all-baseline then all-revised block.
  Date/Author: 2026-06-19 / Grin
- Decision: Same-format FST, same-format FSDB, and expression comparisons use timing thresholds; cross-format FST-versus-FSDB checks are functional-only.
  Rationale: FST versus FSDB timing crosses file formats and readers and is not a meaningful performance comparison. Cross-format checks still matter for payload parity. Same-format comparisons remain the release timing screen and default to 5% maximum negative delta.
  Date/Author: 2026-06-19 / Grin
- Decision: FSDB gate capture writes and uses a gate-local runnable catalog that excludes tests with VCD-style scalar element paths containing `.[`.
  Rationale: The committed FSDB catalog is generated by path replacement from the FST catalog, but converted FSDB fixtures expose some array element signals under FSDB-specific names. Old release refs cannot be changed, so the gate filters non-runnable FSDB scenarios at capture time and records the skip list instead of aborting the entire release performance screen.
  Date/Author: 2026-06-19 / Grin

## Outcomes & Retrospective

The corrected design is complete and committed locally. It keeps committed benchmark baselines removed, preserves the public `just` interface, splits helper implementation by responsibility, restores 5% same-format timing thresholds, validates cross-format parity functionally, and records FSDB runnable-catalog skips in capture manifests. Focused tests, review, and `just check` pass. Full `just bench-gate` is intentionally not rerun during this refactor because the user asked not to run it while other work may add noise.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic waveform inspection. Performance coverage lives under `bench/`. The end-to-end benchmark harness is `bench/e2e/perf.py`; it reads JSON benchmark catalogs such as `bench/e2e/tests.json`, runs the `wavepeek` binary through `hyperfine`, captures functional `wavepeek --json` payloads, writes one `*.hyperfine.json` and one `*.wavepeek.json` per test, writes `README.md`, and can compare two run directories with a maximum allowed negative timing delta. A negative timing delta means the revised run is slower than the golden run. The FSDB catalog is `bench/e2e/tests_fsdb.json`; FSDB means Fast Signal Database, a proprietary waveform format read through the optional Synopsys Verdi FSDB Reader SDK.

The expression microbenchmark harness is `bench/expr/perf.py`; it runs Rust Criterion benchmark targets registered in `Cargo.toml`, exports one raw CSV per scenario, writes `summary.json`, writes `README.md`, and can compare two expression run directories against a negative delta threshold. Criterion is a Rust benchmarking library.

The root `justfile` is the stable local automation interface. Public benchmark recipes are `bench-gate`, `bench-capture`, and `bench-compare`. Low-level benchmark recipes remain private development helpers. Generated benchmark artifacts live under ignored directories such as `tmp/bench-gate/`, `bench/e2e/runs/`, and `bench/expr/runs/`.

The helper group under `tools/bench/` has four scripts. `common.py` owns shared subprocess, git, JSON, timestamp, and output-directory helpers. `capture.py` owns one-ref capture and FSDB runnable-catalog generation. `compare.py` owns same-format timing comparisons plus cross-format functional comparisons. `gate.py` owns the two-ref release orchestration.

## Open Questions

There are no blocking open questions. The implementation uses 5% as the default maximum negative delta for same-format FST, same-format FSDB, and expression timing comparisons. Cross-format FST-versus-FSDB comparisons are functional-only and allow FST-only extra tests when the FSDB runnable catalog is a subset.

## Plan of Work

The current milestone is a refactor of the already committed manual gate. First, keep the public `just` recipes stable while changing them to call the split helper scripts directly. Second, move shared code into `tools/bench/common.py`, one-ref capture into `tools/bench/capture.py`, comparison into `tools/bench/compare.py`, and two-ref orchestration into `tools/bench/gate.py`. Third, make the gate sequence explicit: clone both refs, assess FSDB support, initialize capture directories, build both release binaries, build both FSDB release binaries when enabled, prepare both FSDB fixture sets when enabled, run all FST end-to-end captures baseline then revised, run all FSDB end-to-end captures baseline then revised, run expression captures baseline then revised, finalize manifests, and compare all artifacts. Fourth, update docs and tests to match the corrected behavior.

### Concrete Steps

From repository root `/workspaces/wavepeek`, keep these stable user commands:

    just bench-gate <baseline-ref> [revised-ref] [fsdb-mode]
    just bench-capture [ref] [fsdb-mode]
    just bench-compare <golden-capture-dir> <revised-capture-dir>

The `justfile` should call:

    python3 -B tools/bench/gate.py --baseline-ref <baseline-ref> --revised-ref <revised-ref> --fsdb <mode>
    python3 -B tools/bench/capture.py --ref <ref> --fsdb <mode>
    python3 -B tools/bench/compare.py --golden <dir> --revised <dir>

`tools/bench/gate.py` should expose `--max-negative-delta-pct` with default 5.0 and should not expose `--allow-dirty-source`. If `HEAD` is requested from a dirty source worktree, the helper should fail before cloning. `tools/bench/compare.py` should use the same default threshold for FST, FSDB, and expression same-format timing. It should run cross-format FST-versus-FSDB checks with `bench/e2e/perf.py compare --functional-only --allow-golden-extra`.

Run focused tests after implementation:

    python3 -B -m unittest discover -s tools/bench -p 'test_*.py'
    python3 -B -m unittest discover -s bench/e2e -p 'test_*.py'
    python3 -B -m unittest discover -s bench/expr -p 'test_*.py'
    just test-aux
    just format-justfile-check
    just check-actions

If the environment supports the full project gates, run:

    just check

Do not run full `just bench-gate` during this refactor unless the user asks for it again.

### Validation and Acceptance

Unit tests should prove that parser defaults match the stable interface, dirty `HEAD` is rejected without an override, FSDB support policy behaves as expected, FSDB runnable-catalog filtering records skipped scalar-element tests, same-format FST/FSDB/expr comparisons include timing thresholds, cross-format checks are functional-only with FST extras allowed, and gate orchestration runs build and fixture preparation before paired measurement.

A full release-like acceptance run, when explicitly requested, is:

    just bench-gate <previous-release-tag> HEAD

A successful gate directory should contain `baseline/`, `revised/`, `compare/`, `checkouts/`, `manifest.json`, and `summary.md`. Each capture should contain `manifest.json`, `README.md`, `logs/`, `e2e-fst/`, `expr/`, and optionally `e2e-fsdb/`. If FSDB is captured, each capture manifest should include the generated runnable catalog path and skipped test names. The compare manifest should show same-format FST, same-format FSDB when present, expression, and cross-format functional checks.

The repository should no longer track generated benchmark run artifacts. Running `git ls-files 'bench/e2e/runs/*' 'bench/expr/runs/*'` should list only `bench/e2e/runs/.gitignore` and `bench/expr/runs/.gitignore`.

### Idempotence and Recovery

The helper must be safe to run repeatedly because each default run creates a unique timestamped output directory under `tmp/bench-gate/`. If a user passes an explicit output directory and it already contains files, the helper should fail before cloning or running benchmarks. The safe recovery is to choose a new output directory or manually remove the old one after confirming it is disposable. The implementation must not delete arbitrary paths automatically.

If benchmark capture fails halfway, the partial output directory and logs should remain for diagnosis. A maintainer can rerun the command with a fresh output directory after fixing the problem.

### Artifacts and Notes

Issue 24 has been updated with the rejected CI direction and selected manual gate direction. The comment URL is `https://github.com/kleverhq/wavepeek/issues/24#issuecomment-4748912184`.

The latest full gate artifact from before this refactor is `tmp/bench-gate/gates/20260619T211137Z-a93aad1db823..81777d4e939e/`. It is useful historical evidence but should not be treated as validation for the current refactor because the helper behavior has changed.

### Interfaces and Dependencies

The helper scripts must use only Python standard-library modules. Each script should expose `main(argv: Sequence[str] | None = None) -> int` and return process exit codes rather than calling `sys.exit` deep inside helper functions. The stable public interface is the root `justfile`, not direct helper paths.

`tools/bench/capture.py` must invoke benchmark harnesses from the selected checkout with `cwd` set to that checkout so catalogs and benchmark sources come from the selected ref. `tools/bench/compare.py` may invoke comparison harnesses from the current repository so standalone `just bench-compare` works with existing capture directories.

### Revision Notes

- 2026-06-18 / Grin: Initial plan created after discussion switched issue 24 away from CI-collected benchmark artifacts and toward a manual source-ref comparison gate.
- 2026-06-18 / Grin: Revised plan after architecture/code and docs/release-process review. Added dirty-source handling, e2e artifact identity checks, explicit FSDB build/preparation flow, pairwise FSDB skip policy, `docs/dev/fsdb.md` updates, and patch-release skip recording.
- 2026-06-19 / Grin: Revised plan after real local runs exposed FSDB runnable-catalog issues and after discussion corrected timing policy. The plan now requires split helper scripts, 5% same-format timing comparisons, cross-format functional-only checks, no dirty-state override, and no full gate run during this refactor.
