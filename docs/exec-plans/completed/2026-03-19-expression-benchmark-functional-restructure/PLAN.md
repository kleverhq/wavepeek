# Expression Benchmark Functional Restructure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this plan is implemented, a contributor will be able to run one expression benchmark workflow, produce one run directory, and read one report that covers the whole `src/expr` microbenchmark surface. The committed expression baseline will stop looking like preserved rollout archaeology. Instead of thirteen committed run directories that silently encode `C1` through `C4`, the repository will have one maintained baseline at `bench/expr/runs/baseline/` and one harness entry point, `bench/expr/perf.py`, that lists suites, captures all expression suites into one run, regenerates a grouped report, and compares a revised run against the baseline.

The benchmark suite layout itself will also become functional rather than historical. A contributor adding or updating a benchmark will choose among `syntax`, `logical`, `event`, and `waveform_host`, which describe real expression-engine responsibilities in this repository. `syntax` covers lexer and parser work, `logical` covers standalone logical bind and evaluation work on in-memory hosts, `event` covers standalone event bind and matching work, and `waveform_host` covers the adapter that binds and evaluates expressions against waveform-backed metadata. The result is observable in three ways: the Rust bench targets in `Cargo.toml` use those functional names, `python3 bench/expr/perf.py run --run-dir bench/expr/runs/baseline` writes one grouped `README.md` for all expression benchmarks, and the old per-phase run directories are gone.

This plan changes benchmark organization and benchmark tooling only. It must not change expression language semantics, command routing, or the roadmap phase boundaries documented in `docs/expression_roadmap.md`.

## Non-Goals

This plan does not change `src/expr/` parser, binder, evaluator, or waveform-host semantics. It does not merge the expression microbenchmark harness into `bench/e2e/perf.py`; the two harnesses should look and feel similar, but expression benchmarks still depend on Rust Criterion `raw.csv` artifacts while the CLI harness depends on `hyperfine` plus functional JSON captures. It does not preserve like-for-like timing continuity with the current committed `parser-*`, `event-runtime-*`, `integral-boolean-*`, and `rich-types-*` run directories, because the suite boundaries and report shape will intentionally change. It does not require one monolithic `cargo bench` invocation; one expression baseline means one committed run directory and one report, even if the harness reaches that result by running multiple bench targets sequentially. It also does not keep `candidate`, `verify`, or `carry-forward` as permanent committed directories after the migration. Temporary review or reproducibility runs may still exist locally or in CI, but the durable repository state must keep only the maintained baseline unless a later plan explicitly promotes another run directory.

## Progress

- [x] (2026-03-19 21:10Z) Reviewed `docs/expression_roadmap.md`, `docs/DEVELOPMENT.md`, `Cargo.toml`, `bench/expr/AGENTS.md`, `bench/expr/capture.py`, `bench/expr/compare.py`, the four current Rust bench targets, the committed expression run directories, `bench/e2e/perf.py`, `Makefile`, and `.pre-commit-config.yaml` to understand the current workflow and the repository's benchmark conventions.
- [x] (2026-03-19 21:10Z) Chose the end-state benchmark model for this cleanup: functional suite names (`syntax`, `logical`, `event`, `waveform_host`), one explicit expression benchmark catalog, one e2e-style `bench/expr/perf.py` harness, one committed baseline directory under `bench/expr/runs/baseline/`, and deletion of the current per-suite historical run directories after the new baseline is captured.
- [x] (2026-03-19 21:24Z) Completed focused architecture, docs, and performance review lanes on the draft plan, then revised the migration order, the meaning of “one baseline”, the temporary status of same-commit verify runs, the deletion rules for old run directories, the Criterion namespacing requirement, and the final validation steps so the plan is executable without relying on historical context.
- [x] (2026-03-19 21:31Z) Completed one fresh control-pass review on the revised plan; no additional substantive issues were reported.
- [x] (2026-03-19 21:43Z) Applied follow-up review fixes to make `run --run-dir` safety explicit, add container-first command guidance, require temporary baseline capture before replacing `bench/expr/runs/baseline/`, include `docs/DESIGN.md` in the live-doc sweep, and tighten the Criterion collision story from exported-file namespacing to suite-prefixed internal benchmark IDs.
- [x] (2026-03-19 21:49Z) Added the final compare-identity clarification after the control pass so subset-to-full comparisons are explicitly rejected unless both runs record the same selected-suite set.
- [x] (2026-03-20 00:18Z) Implemented Milestone 1 with TDD: added `bench/expr/suites.json`, introduced the new stdlib-only `bench/expr/perf.py` harness with `list` / `run` / `report` / `compare`, added `bench/expr/test_perf.py`, kept the legacy helpers in place, and validated both the new unit tests and one real aggregated run/report over the current four Rust bench targets.
- [x] (2026-03-20 01:25Z) Implemented Milestone 2: replaced the old phase-shaped Rust bench targets with `expr_syntax`, `expr_logical`, `expr_event`, and `expr_waveform_host`, added `bench/expr/support.rs` for shared Criterion and in-memory host helpers, rewired `bench/expr/suites.json` to the final functional suite split, and validated the renamed benches plus one real aggregated harness run over the new suite layout.
- [x] (2026-03-20 01:58Z) Implemented Milestone 3: removed the retired helper scripts/tests and scenario manifests, updated `docs/DEVELOPMENT.md`, `docs/DESIGN.md`, `Makefile`, `bench/expr/AGENTS.md`, `bench/expr/runs/AGENTS.md`, `bench/AGENTS.md`, and `bench/expr/test_perf.py` to the unified harness contract, added the new `make bench-expr-update-baseline` / `make bench-expr-run` targets, and validated `python3 -m unittest discover -s bench/expr -p "test_*.py"`, the renamed bench targets in test mode, `python3 bench/expr/perf.py list`, and `WAVEPEEK_IN_CONTAINER=1 make check`.
- [x] (2026-03-20 02:33Z) Implemented Milestone 4: proved the final suite split with temporary control runs, promoted one clean `bench/expr/runs/baseline/` capture from the unified harness, deleted the thirteen historical committed run directories, and revalidated the committed baseline with a fresh same-commit compare at the 10% gate.
- [x] (2026-03-20 03:07Z) Completed Milestone 5: ran focused code, architecture, docs, and performance review lanes; fixed the reported harness provenance, strict Criterion-ID, deterministic-report, doc-contract, and baseline-refresh issues; reran the impacted validation including `python3 -m unittest bench.expr.test_perf`, fresh revised-vs-baseline compare at the 10% gate, and `WAVEPEEK_IN_CONTAINER=1 make ci`; then finished with one fresh independent control-pass review that reported the consolidated diff clean.

## Surprises & Discoveries

- Observation: the live expression benchmark names are already neutral, but their boundaries still mirror the roadmap delivery sequence.
  Evidence: `Cargo.toml` registers `expr_parser`, `expr_event_runtime`, `expr_integral_boolean`, and `expr_rich_types`, which align directly with `docs/expression_roadmap.md` phases `C1` through `C4` even though the old `expr_c1` through `expr_c4` names were removed.

- Observation: `bench/expr/expr_rich_types.rs` currently mixes two different concerns that are easier to maintain separately: rich logical and event semantics on an in-memory host, and waveform-adapter metadata cost on a real dump-backed host.
  Evidence: the same file benchmarks `bind_logical_rich_types` and `eval_logical_enum_label_preservation`, but it also benchmarks `bind_waveform_host_metadata_path` and `eval_waveform_host_metadata_path`, which exercise `waveform::expr_host::WaveformExprHost` rather than only the logical engine.

- Observation: the current export helpers key scenarios only by scenario name, not by suite plus scenario, which is safe today only because the current scenario names happen to be globally unique.
  Evidence: `bench/expr/capture.py` infers each exported row from `path.parent.parent.name` and stores it in a dictionary keyed by that one scenario string. A functional restructure could naturally introduce repeated names such as `bind_simple` or `eval_true` across multiple suites unless the new harness namespaces artifacts explicitly.

- Observation: the repository already has an e2e benchmark model that matches the user's desired outcome more closely than the current expression workflow does.
  Evidence: `bench/e2e/perf.py` already owns a `list` / `run` / `report` / `compare` CLI, one explicit catalog in `bench/e2e/tests.json`, one committed `bench/e2e/runs/baseline/` directory, and one grouped `README.md` report for the whole benchmark surface.

- Observation: the repository already has CI coverage for `bench/expr/test_*.py`, even though the current expression workflow docs and hooks still emphasize the older helper split.
  Evidence: `Makefile` already defines `test-bench-expr` with `python3 -m unittest discover -s bench/expr -p "test_*.py"`, and `make ci` already runs that target.

- Observation: the current committed run directory is the main source of “historical feel,” not the current Rust file names alone.
  Evidence: `bench/expr/runs/` currently contains `parser-baseline`, `parser-candidate`, `parser-verify`, `event-runtime-baseline`, `event-runtime-candidate`, `event-runtime-verify`, `integral-boolean-baseline`, `integral-boolean-candidate`, `integral-boolean-verify`, `integral-boolean-carry-forward`, `rich-types-baseline`, `rich-types-candidate`, and `rich-types-verify`.

- Observation: safe `--missing-only` resume required the new harness to persist partial multi-suite state after each completed suite instead of only at the very end of a full run.
  Evidence: the resume contract in this plan requires `summary.json` to prove the catalog fingerprint and selected-suite set before any missing suites are rerun, so an interrupted aggregated capture would otherwise have nothing authoritative to validate against.

- Observation: the dump-backed waveform adapter cannot be hidden entirely inside `bench/expr/support.rs` without breaking the current private-module wiring from `src/waveform/expr_host.rs`.
  Evidence: the included waveform adapter code still references `crate::waveform`, `crate::expr`, and `crate::error` from the bench crate root, so the final `expr_waveform_host` bench has to keep the crate-root shim and `mod waveform;` locally even after the rest of the shared scaffolding moved into `support.rs`.

- Observation: the last stale-name grep cannot go completely green until Milestone 4 deletes the historical committed run directories, even after Milestone 3 cleans all live workflow files.
  Evidence: the old `parser-*`, `event-runtime-*`, `integral-boolean-*`, and `rich-types-*` run directories still contain committed `README.md` and `summary.json` files with legacy bench-target and scenario-manifest metadata, so a repository-wide search still sees those historical names until the baseline refresh lands.

- Observation: once the branch started promoting the new baseline inside the repository, provenance handling had to distinguish real source-tree dirtiness from the run directory that the harness itself was creating or resuming.
  Evidence: the first review pass caught that `--missing-only` resume could invalidate itself on `worktree_state` when the run directory lived under `bench/expr/runs/`, which led to the final fix that captures git cleanliness with the target run directory excluded.

## Decision Log

- Decision: the final functional suite split will be `syntax`, `logical`, `event`, and `waveform_host`.
  Rationale: these four names describe real ownership boundaries in this repository better than `parser`, `event runtime`, `integral boolean`, and `rich types`. `syntax` keeps lexer and parser cost together, `logical` groups standalone logical bind and eval behavior across both core and rich logical forms, `event` groups standalone event bind and event-match behavior including richer `iff` forms, and `waveform_host` isolates dump-backed host overhead from pure expression semantics.
  Date/Author: 2026-03-19 / OpenCode

- Decision: implement a new e2e-style harness at `bench/expr/perf.py` rather than extending the current `capture.py` and `compare.py` pair.
  Rationale: the user wants one maintained baseline and one grouped report for the whole expression surface. A single harness with `list`, `run`, `report`, and `compare` commands makes that workflow explicit, mirrors the already successful `bench/e2e/perf.py` model, and removes the need for users to remember a manual `cargo bench` plus `capture.py` plus `compare.py` sequence for each suite.
  Date/Author: 2026-03-19 / OpenCode

- Decision: define “one expression baseline” as one committed run directory and one grouped report, not as one monolithic Criterion process.
  Rationale: Rust Criterion still operates through separate bench targets in `Cargo.toml`, and keeping those binaries separate is useful for compile times, ownership, and targeted iteration. The harness will aggregate those suites into one run directory and one report, which gives the user the ergonomic outcome they asked for without forcing a less maintainable one-binary design.
  Date/Author: 2026-03-19 / OpenCode

- Decision: keep only `bench/expr/runs/baseline/` as the committed durable baseline after the migration, and treat same-commit verify runs as temporary artifacts.
  Rationale: the repository needs one stable golden run for future comparisons, but it does not need to keep every review-time capture permanently. Preserving `candidate`, `verify`, and `carry-forward` in the working tree would keep the “historical archaeology” feel that this cleanup is intended to remove.
  Date/Author: 2026-03-19 / OpenCode

- Decision: replace the per-suite scenario manifests under `bench/expr/scenarios/` with one explicit expression benchmark catalog file at `bench/expr/suites.json`.
  Rationale: one catalog is easier to browse, mirrors the `bench/e2e/tests.json` workflow, and gives the new harness one authoritative source for suite identity, bench target names, scenario membership, and report grouping. The old directory split was mainly an artifact of the one-suite-at-a-time export workflow.
  Date/Author: 2026-03-19 / OpenCode

- Decision: namespace both Criterion-internal benchmark IDs and exported raw CSV artifact names by suite as well as scenario.
  Rationale: the new functional split makes repeated labels across suites more likely over time. Prefixing each `bench_function` ID as `suite__scenario` prevents collisions under `target/criterion` before export, and using the same identity in exported file names keeps run directories and compare summaries unambiguous.
  Date/Author: 2026-03-19 / OpenCode

- Decision: keep the expression compare gate focused on timing regressions and metadata identity, without adding a functional parity column like the e2e harness uses.
  Rationale: expression microbenchmarks already prove correctness through their Rust assertions before timing is recorded. Unlike the CLI e2e harness, there is no second functional artifact that needs to be compared separately. The grouped report should therefore mirror the e2e ergonomics while staying honest about what these benchmarks actually validate.
  Date/Author: 2026-03-19 / OpenCode

- Decision: `bench/expr/perf.py run --run-dir <dir>` must fail on any non-empty directory unless the user passes an explicit resume flag, and resume mode must accept only an exact catalog fingerprint plus exact selected-suite match.
  Rationale: once the repository keeps one shared baseline directory, accidental reuse of a partially populated run directory becomes the easiest way to publish misleading aggregated results. An explicit resume contract is safer and mirrors the guarded `--missing-only` behavior already used in the e2e harness.
  Date/Author: 2026-03-19 / OpenCode

- Decision: same-commit reproducibility compares will use a 10% negative-delta gate unless implementation evidence shows the new `waveform_host` suite is stable enough for a tighter threshold.
  Rationale: the unified expression surface includes noisier dump-backed host work. A blanket 5% same-commit threshold risks false failures that encourage rerun-until-green behavior, which would weaken the credibility of the committed baseline.
  Date/Author: 2026-03-19 / OpenCode

- Decision: do not broaden the pre-commit hooks in this plan.
  Rationale: `make ci` already runs `test-bench-expr`, so the new harness will be covered in the repository's main quality gate. Adding another pre-commit hook would increase local hook cost without being required for the user-visible workflow change requested here.
  Date/Author: 2026-03-19 / OpenCode

- Decision: `bench/expr/perf.py run` will rewrite `summary.json` and `README.md` after each completed suite capture, not only once at the end.
  Rationale: resume mode is explicitly keyed off existing `summary.json` metadata. Persisting partial state after each suite makes interrupted multi-suite runs resumable without inventing another checkpoint format, while the final successful run still ends with one authoritative summary and one grouped report.
  Date/Author: 2026-03-20 / OpenCode

- Decision: keep waveform-module inclusion local to `bench/expr/expr_waveform_host.rs` instead of moving it into `bench/expr/support.rs`.
  Rationale: the reused adapter source under `src/waveform/` still expects crate-root `waveform`, `expr`, and `error` modules. Localizing that shim to the waveform-host bench preserves the private-module wiring that already works today and keeps the new shared support focused on truly generic helpers.
  Date/Author: 2026-03-20 / OpenCode

- Decision: treat the final stale-reference grep as a Milestone 4 closeout gate rather than a Milestone 3 blocker.
  Rationale: once the live docs, helpers, tests, and make targets were migrated, the remaining old-name matches came only from the still-committed historical run artifacts that Milestone 4 is already responsible for replacing. Deferring the repository-wide grep until after those directories are deleted keeps the validation aligned with the actual migration order.
  Date/Author: 2026-03-20 / OpenCode

- Decision: keep the general `compare` subcommand flexible across code revisions, but require toolchain and environment metadata in the default `make bench-expr-run` path and document the stricter same-commit provenance flags separately.
  Rationale: contributors still need to compare changed code against the maintained baseline, so `source_commit` cannot be a mandatory default compare key. Tightening the default make target around `cargo_version`, `rustc_version`, `criterion_version`, and `environment_note`, while documenting the stricter same-commit flag set for verification workflows, preserves that flexibility without leaving the common path provenance-blind.
  Date/Author: 2026-03-20 / OpenCode

## Outcomes & Retrospective

Current status: all five milestones are complete. The repository now has one expression benchmark harness, one functional suite catalog, one maintained committed baseline under `bench/expr/runs/baseline/`, updated docs and make targets, and no remaining live helper workflow from the retired per-suite export path.

The main lesson from the full implementation is that the current pain was still mostly tool and artifact shape, not missing benchmark coverage. Once the harness, suite ownership, and artifact model were aligned, the remaining work collapsed to provenance hardening and baseline promotion mechanics rather than new benchmark logic.

The review cycle also confirmed that benchmark tooling needs provenance semantics just as much as benchmark scenarios do. The last real defects were not in the expression cases themselves; they were in how resume, report regeneration, and baseline promotion described what had been captured. Tightening those contracts made the final baseline and compare workflow trustworthy enough to keep as the new durable surface.

## Context and Orientation

`wavepeek` is a single-crate Rust repository. The expression engine lives in `src/expr/`, which exposes the public standalone entry points used by the benchmark files: `lex_event_expr`, `parse_event_expr_ast`, `bind_logical_expr_ast`, `bind_event_expr_ast`, `eval_logical_expr_at`, and `event_matches_at`. Those functions are re-exported from `src/expr/mod.rs`, and the current Rust benchmark files in `bench/expr/` call them directly. The benchmark files are not CLI integration tests. They are microbenchmarks that measure parser, binder, evaluator, and host-adapter hot paths inside the library.

For this plan, a “suite” means one Rust Criterion bench target in `Cargo.toml` together with the scenario IDs it owns. A “scenario” means one `c.bench_function("...")` identifier inside a Rust bench file. A “run directory” means a filesystem directory under `bench/expr/runs/` or an explicitly chosen temporary directory that contains exported `*.raw.csv` files, one run-level `summary.json`, and one human-readable `README.md`. A “baseline” means the committed golden run directory that future revised runs compare against. A “Criterion baseline name” is the internal label that Criterion stores under `target/criterion/`; it is an implementation detail of the harness and must not be the main user-facing workflow once this plan lands.

The current workflow is spread across several surfaces. `Cargo.toml` defines four bench binaries: `expr_parser`, `expr_event_runtime`, `expr_integral_boolean`, and `expr_rich_types`. The files `bench/expr/expr_parser.rs`, `bench/expr/expr_event_runtime.rs`, `bench/expr/expr_integral_boolean.rs`, and `bench/expr/expr_rich_types.rs` each set up a local `Criterion` configuration, build a suite-specific in-memory host, and register a small set of scenario IDs. The manifests under `bench/expr/scenarios/` declare one suite at a time. `bench/expr/capture.py` exports one saved Criterion baseline for one bench target into one run directory, and `bench/expr/compare.py` compares two exported run directories for one matching suite identity. This means a contributor who wants to refresh all expression baselines must repeat the same manual sequence four times and then manage thirteen committed run directories.

The e2e benchmark surface shows the desired direction. `bench/e2e/perf.py` reads one explicit catalog, owns one `list` / `run` / `report` / `compare` CLI, writes one grouped `README.md`, and treats `bench/e2e/runs/baseline/` as the shared golden run. The expression harness should adopt that ergonomic model while still reading Criterion `raw.csv` rather than `hyperfine` JSON.

The current scenario inventory is already useful and should mostly be carried forward rather than reinvented. `bench/expr/expr_parser.rs` owns the parser-only scenarios `tokenize_union_iff`, `parse_event_union_iff`, and `parse_event_malformed`. `bench/expr/expr_event_runtime.rs` owns `bind_event_union_iff`, `eval_event_union_iff_true`, and `eval_event_union_iff_unknown`. `bench/expr/expr_integral_boolean.rs` owns `bind_logical_core_integral`, `eval_logical_core_integral_true`, `eval_logical_core_integral_unknown`, and `eval_event_iff_core_integral`. `bench/expr/expr_rich_types.rs` owns `bind_logical_rich_types`, `bind_waveform_host_metadata_path`, `eval_logical_real_mixed_numeric`, `eval_logical_string_equality`, `eval_logical_enum_label_preservation`, `eval_event_iff_triggered_rich`, and `eval_waveform_host_metadata_path`.

The new functional suite mapping should be explicit from the start of implementation. `expr_syntax` should keep the three parser scenarios. `expr_logical` should own `bind_logical_core_integral`, `eval_logical_core_integral_true`, `eval_logical_core_integral_unknown`, `bind_logical_rich_types`, `eval_logical_real_mixed_numeric`, `eval_logical_string_equality`, and `eval_logical_enum_label_preservation`. `expr_event` should own `bind_event_union_iff`, `eval_event_union_iff_true`, `eval_event_union_iff_unknown`, `eval_event_iff_core_integral`, and `eval_event_iff_triggered_rich`. `expr_waveform_host` should own `bind_waveform_host_metadata_path` and `eval_waveform_host_metadata_path`. This mapping keeps existing scenario coverage while changing only ownership and report grouping. In the final Rust bench files, every Criterion benchmark ID should be prefixed as `suite__scenario`, for example `syntax__tokenize_union_iff` and `waveform_host__bind_waveform_host_metadata_path`, so the harness can reason about one collision-free namespace.

The current committed run directory is the cleanup target. `bench/expr/runs/` contains one `AGENTS.md` file plus thirteen phase-history run directories: three parser runs, three event-runtime runs, four integral-boolean runs, and three rich-types runs. After this plan lands, the durable committed state in that directory should be `AGENTS.md` plus `baseline/` only.

## Open Questions

There are no blocking product questions. The user-visible target state is clear.

One implementation-time question should be answered conservatively during Milestone 1: whether the new harness should print only suite IDs in `list`, or also print scenario counts and scenario names. The safe default is to print one line per suite with the suite ID, the Rust bench target, and the scenario count, because suite-level selection is the important workflow and the full per-scenario inventory already appears in `README.md` reports.

## Plan of Work

Milestone 1 adds the new workflow before any suite rename occurs. Create a new explicit catalog at `bench/expr/suites.json`, a new Python harness at `bench/expr/perf.py`, and a new test file at `bench/expr/test_perf.py`. The first version of the harness should aggregate the current four Rust bench targets exactly as they exist today. That keeps the migration safe: a contributor can prove that one run directory and one grouped report are possible before changing the Rust suite boundaries. The harness should expose four subcommands analogous to the e2e harness. `list` reads the catalog and prints the available suites. `run` iterates through the selected suites, runs `cargo bench --bench <target> -- --save-baseline <internal-name> --noplot`, exports stable `raw.csv` files into one run directory, and writes `summary.json` plus `README.md`. `report` regenerates `README.md` from an existing run directory and an optional compare directory. `compare` loads two run directories, validates identity, and fails when mean or median regression exceeds the requested threshold.

The catalog file should be the one source of truth for suite membership. Use one object with a `suites` array. Each suite row should declare the final stable fields `id`, `bench_target`, `description`, and `scenarios`. The first implementation can point at the current targets (`expr_parser`, `expr_event_runtime`, `expr_integral_boolean`, `expr_rich_types`), because the catalog will be updated to the final functional names in Milestone 2. `bench/expr/perf.py` should reuse the safe parts of the current helper logic rather than starting from nothing. Port the `raw.csv` parsing rules from `bench/expr/capture.py`, the metadata checks from `bench/expr/compare.py`, and the report ergonomics from `bench/e2e/perf.py`. Keep the new file self-contained and Python-stdlib-only. In the new harness, `run --run-dir <dir>` must fail if `<dir>` already exists and is non-empty unless the user passes an explicit `--missing-only` resume flag. Resume mode must load the existing `summary.json`, verify an exact catalog fingerprint match, verify the exact selected-suite list match, and then run only the missing suites before regenerating `summary.json` and `README.md` from scratch. The `compare` command must likewise require an exact catalog fingerprint and exact selected-suite match between revised and golden runs; subset-to-full compares should fail fast instead of being silently treated as partial success.

Milestone 2 performs the functional restructure in Rust while preserving the new aggregated workflow. Update `Cargo.toml` so the four bench targets become `expr_syntax`, `expr_logical`, `expr_event`, and `expr_waveform_host`. Rename or replace the Rust bench files in `bench/expr/` accordingly. The point of this milestone is not merely renaming files; it is moving scenarios into the right ownership buckets. `bench/expr/expr_syntax.rs` should continue to own the lexer and parser cases. `bench/expr/expr_logical.rs` should own all standalone logical bind and eval cases, including the current rich-type logical scenarios. `bench/expr/expr_event.rs` should own all standalone event bind and match cases, including richer `iff` scenarios. `bench/expr/expr_waveform_host.rs` should isolate waveform-backed host binding and evaluation.

This milestone should also remove the repeated Rust benchmark scaffolding that makes the current suite files harder to maintain. Add one shared module, `bench/expr/support.rs`, and move into it only the code that is genuinely reused across suites: the common `Criterion` configuration builder, the shared `ExprType` constructor helpers, the reusable in-memory `ExpressionHost` timeline lookup behavior, and the waveform fixture path helper. Keep suite-specific signal timelines and benchmark assertions in the suite files so the measured behavior remains easy to read. After the Rust files are in their final shape, update `bench/expr/suites.json` to the final suite IDs and scenario mapping described in the Context section, and rename every `c.bench_function(...)` ID to the final `suite__scenario` form so the new harness can validate Criterion-side identity without ambiguity.

Milestone 3 removes the legacy workflow and makes the new one the only documented path. Delete `bench/expr/capture.py`, `bench/expr/compare.py`, `bench/expr/test_capture.py`, `bench/expr/test_compare.py`, `bench/expr/scenarios/AGENTS.md`, and the four scenario manifest files under `bench/expr/scenarios/`. Update `bench/expr/AGENTS.md` so it names `bench/expr/suites.json`, `bench/expr/perf.py`, the final Rust bench target files, and `bench/expr/runs/baseline/` as the durable workflow. Update `bench/expr/runs/AGENTS.md` so it explains that `baseline/` is the maintained committed golden run and that other run directories are temporary unless explicitly promoted by a later plan. Update `docs/DEVELOPMENT.md` and `docs/DESIGN.md` so the live documentation no longer names the retired bench targets or the retired per-suite run workflow. Add matching convenience targets in `Makefile`, analogous to `bench-e2e-update-baseline` and `bench-e2e-run`, so the new expression workflow is easy to discover.

Milestone 4 captures the new baseline and deletes the historical artifacts. First, use the new harness to generate a temporary run directory and compare it to itself via a second temporary run so the final report shape and compare contract are proven on the final functional suite split. Then replace the committed contents of `bench/expr/runs/` with the new durable baseline. Capture `bench/expr/runs/baseline/` using the new harness, regenerate its `README.md`, and run one fresh temporary same-commit verify capture outside the repository baseline directory. Compare that temporary run against `bench/expr/runs/baseline/` with a tighter reproducibility threshold before accepting the baseline refresh. Once the baseline exists and the compare passes, delete the current committed directories `parser-baseline`, `parser-candidate`, `parser-verify`, `event-runtime-baseline`, `event-runtime-candidate`, `event-runtime-verify`, `integral-boolean-baseline`, `integral-boolean-candidate`, `integral-boolean-carry-forward`, `integral-boolean-verify`, `rich-types-baseline`, `rich-types-candidate`, and `rich-types-verify`.

Milestone 5 closes the branch through validation and review. Run the harness unit tests, the four Rust bench targets in test mode, the new baseline capture plus temporary verify compare, and then `make ci`. After the branch is green, run the required review workflow by loading the `ask-review` skill and requesting at least an architecture lane for suite boundaries and benchmark ownership, a docs lane for the workflow contract in the plan and the docs, and a performance lane for Criterion capture and compare semantics. Resolve any findings, rerun the affected validation, and then perform one final fresh control-pass review on the consolidated diff before handing the branch back.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-property` inside the wavepeek devcontainer or CI image. This repository is container-first, and the direct `cargo`, `python3`, and `make` commands below assume the toolchain and benchmark fixtures provided there.

1. Confirm the current baseline and current benchmark helper tests before editing anything.

       cargo test --bench expr_parser --bench expr_event_runtime --bench expr_integral_boolean --bench expr_rich_types
       python3 -m unittest bench.expr.test_capture bench.expr.test_compare

   Expect both commands to pass. If the existing helper tests or current bench targets are already broken, stop and record that discovery in this plan before changing any workflow files.

2. Add the new catalog and the new harness first, without deleting the old helpers yet.

   Create `bench/expr/suites.json`, `bench/expr/perf.py`, and `bench/expr/test_perf.py`. The first catalog version should still describe the current four bench targets. The first test file should cover catalog validation, run-directory path handling, non-empty-directory failure by default, explicit `--missing-only` resume behavior, stale-catalog rejection, artifact namespacing, summary generation, report generation, compare failures on suite or scenario mismatches, duplicate suite-plus-scenario detection, and metadata-key matching rules. Keep `bench/expr/capture.py` and `bench/expr/compare.py` in place during this milestone so the new harness can be developed against a known-good reference.

   Red-phase command:

       python3 -m unittest bench.expr.test_perf

   Expected red evidence: the new tests fail because `bench/expr/perf.py` does not exist yet or because it cannot yet aggregate multiple suites into one run directory.

   Green-phase commands after implementation:

       python3 -m unittest bench.expr.test_perf
       tmp_run="$(mktemp -d)" && python3 bench/expr/perf.py run --run-dir "$tmp_run" && python3 bench/expr/perf.py report --run-dir "$tmp_run"

   Acceptance for this milestone: the temporary run directory contains one `summary.json`, one `README.md`, and namespaced `*.raw.csv` files for all currently cataloged suites, and the report groups rows by suite instead of requiring one directory per suite.

3. Recut the Rust benchmarks into the final functional suites and wire the catalog to them.

   Update `Cargo.toml`, create or rename the four Rust bench files to `bench/expr/expr_syntax.rs`, `bench/expr/expr_logical.rs`, `bench/expr/expr_event.rs`, and `bench/expr/expr_waveform_host.rs`, and add `bench/expr/support.rs` for truly shared setup logic. Update `bench/expr/suites.json` so its suite IDs and bench targets match the new names. Move the scenario ownership exactly as described in the Context section; do not invent extra scenarios unless a rename or split makes a small helper case necessary to preserve the same coverage. Rename each Criterion benchmark ID to `suite__scenario` in the same milestone so the final suite split never depends on global scenario uniqueness by accident.

   Commands after this step:

       cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host
       python3 -m unittest bench.expr.test_perf
       tmp_run="$(mktemp -d)" && python3 bench/expr/perf.py run --run-dir "$tmp_run"

   Acceptance for this milestone: the new bench targets compile and execute in test mode, the temporary aggregated run uses the final suite names, and `expr_rich_types` no longer exists as a mixed semantic-plus-host bucket.

4. Remove the old helpers and switch repository docs to the new workflow.

   Delete `bench/expr/capture.py`, `bench/expr/compare.py`, `bench/expr/test_capture.py`, `bench/expr/test_compare.py`, and the `bench/expr/scenarios/` directory. Update `bench/expr/AGENTS.md`, `bench/expr/runs/AGENTS.md`, `docs/DEVELOPMENT.md`, `docs/DESIGN.md`, and `Makefile`. The new documentation should show the commands a contributor actually needs to run, for example:

       python3 bench/expr/perf.py list
       python3 bench/expr/perf.py run --run-dir bench/expr/runs/baseline
       python3 bench/expr/perf.py report --run-dir bench/expr/runs/baseline
       python3 bench/expr/perf.py compare --revised <dir> --golden bench/expr/runs/baseline --max-negative-delta-pct 15

   Add Make targets similar in spirit to:

       make bench-expr-update-baseline
       make bench-expr-run

   Keep `make ci` unchanged except for picking up the renamed bench targets and the existing `test-bench-expr` coverage.

   Commands after this step:

       python3 -m unittest discover -s bench/expr -p "test_*.py"
       cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host
       rg -n "expr_parser|expr_event_runtime|expr_integral_boolean|expr_rich_types|parser-baseline|event-runtime-baseline|integral-boolean-baseline|rich-types-baseline" Cargo.toml docs bench --glob '!docs/exec-plans/completed/**' --glob '!docs/expression_roadmap.md' --glob '!docs/ROADMAP.md'
       make check

   Acceptance for this milestone: no live docs or helper files mention the old `capture.py` / `compare.py` / per-suite-scenario workflow, and a contributor can learn the entire new workflow from `docs/DEVELOPMENT.md`, `bench/expr/AGENTS.md`, and `make help`.

5. Capture the new baseline and delete the historical run directories.

   Before deleting the old committed runs, validate the final harness and final suite split with one temporary end-to-end control pass.

       tmp_golden="$(mktemp -d)" && tmp_verify="$(mktemp -d)" && \
       python3 bench/expr/perf.py run --run-dir "$tmp_golden" && \
       python3 bench/expr/perf.py run --run-dir "$tmp_verify" && \
       python3 bench/expr/perf.py compare --revised "$tmp_verify" --golden "$tmp_golden" --max-negative-delta-pct 10

   Then capture a fresh temporary baseline candidate, validate it, and only after that replace the committed baseline directory.

       tmp_baseline="$(mktemp -d)" && tmp_verify="$(mktemp -d)" && \
       python3 bench/expr/perf.py run --run-dir "$tmp_baseline" && \
       python3 bench/expr/perf.py run --run-dir "$tmp_verify" && \
       python3 bench/expr/perf.py compare --revised "$tmp_verify" --golden "$tmp_baseline" --max-negative-delta-pct 10 && \
       rm -rf bench/expr/runs/baseline && mv "$tmp_baseline" bench/expr/runs/baseline

   After the baseline and temporary verify compare are green, delete the thirteen old committed run directories named in Milestone 4.

   Acceptance for this milestone: `bench/expr/runs/` contains `AGENTS.md` plus `baseline/` only, `bench/expr/runs/baseline/README.md` reports all expression suites in one document, and the temporary same-commit compare passes against that baseline.

6. Run the final repository validation and the mandatory review workflow.

       make ci

   Then run the required review lanes with `ask-review`: architecture, docs, and performance. Resolve findings, rerun the affected commands, and finish with one fresh control pass review on the consolidated diff.

### Validation and Acceptance

The implementation is complete when a contributor can verify all of the following without consulting the old phase-oriented benchmark history.

First, `python3 bench/expr/perf.py list` prints the functional expression suite inventory from `bench/expr/suites.json`. Second, `python3 bench/expr/perf.py run --run-dir bench/expr/runs/baseline` produces one committed run directory with suite-prefixed Criterion identities, namespaced exported `*.raw.csv`, one aggregated `summary.json`, and one `README.md` that groups all scenarios under `syntax`, `logical`, `event`, and `waveform_host`. Third, `python3 bench/expr/perf.py compare --revised <tmp-dir> --golden bench/expr/runs/baseline --max-negative-delta-pct 15` works against the single committed baseline rather than requiring four per-suite baselines. Fourth, `bench/expr/perf.py run --run-dir <existing-dir>` fails on a non-empty directory unless `--missing-only` is passed and the existing `summary.json` proves an exact catalog fingerprint plus selected-suite match. Fifth, `Cargo.toml`, `docs/DEVELOPMENT.md`, `docs/DESIGN.md`, `bench/expr/AGENTS.md`, and the bench file names mention only the new functional suite names and the new harness. Sixth, `bench/expr/runs/` no longer contains the thirteen historical per-suite run directories.

The final validation set should therefore include `python3 -m unittest discover -s bench/expr -p "test_*.py"`, `cargo test --bench expr_syntax --bench expr_logical --bench expr_event --bench expr_waveform_host`, one temporary `perf.py run` plus `perf.py compare` against `bench/expr/runs/baseline` with the 10% same-commit reproducibility gate, the stale-reference `rg` sweep from Milestone 3, and `make ci`.

### Idempotence and Recovery

The harness and the catalog changes are safe to iterate on repeatedly. `bench/expr/perf.py run --run-dir <dir>` must create a fresh empty directory by default and fail if `<dir>` already contains files. Resume behavior is allowed only through the explicit `--missing-only` path, and that path must first verify the catalog fingerprint and selected-suite list recorded in the existing `summary.json`; if either differs, the command must stop instead of trying to merge results. The implementation should treat generated Criterion output under `target/criterion/` as disposable build output. If a suite rename or scenario move leaves stale Criterion data that causes missing or extra scenario errors, delete the affected expression-generated contents under `target/criterion/` and rerun the same `perf.py run` command before changing code. The committed `bench/expr/runs/baseline/` directory should be refreshed only after a temporary control run and then a temporary same-commit verify run pass against the final suite split.

Because this plan intentionally resets the committed benchmark history, recovery from a failed migration means regenerating the new temporary run directory, not trying to keep the old per-suite run directories alive in parallel. The completed execution plans under `docs/exec-plans/completed/` remain the historical record for the old layout, so the working tree does not need to preserve those directories once the new baseline is accepted.

### Artifacts and Notes

The new aggregated `README.md` report should read more like the e2e report than the current per-suite README files. A concise example shape is:

    # Expression Bench Run: baseline

    - Generated at (UTC): 2026-03-19T22:00:00Z
    - cargo -V: cargo 1.93.0 (...)
    - rustc -V: rustc 1.93.0 (...)
    - criterion crate version: 0.8.2

    ## syntax

    | scenario | mean ns/iter | median ns/iter |
    | --- | ---: | ---: |
    | tokenize_union_iff | 192.33 | 192.20 |
    | parse_event_union_iff | 277.87 | 273.95 |

    ## logical

    | scenario | mean ns/iter | median ns/iter |
    | --- | ---: | ---: |
    | bind_logical_core_integral | 576.86 | 576.13 |

When `report` or `run` receives `--compare <dir>`, the same `README.md` should add compare context and annotate each row with timing deltas, but it should not invent a functional parity column because expression microbenchmarks do not emit separate functional artifacts.

The new `summary.json` should preserve the provenance that the current helper pair records: cargo version, rustc version, criterion crate version, source commit, worktree state, catalog path, suite identities, and per-scenario mean and median values. The report is for humans; the summary is the stable machine-readable compare input.

### Interfaces and Dependencies

Implement the new Python harness in `bench/expr/perf.py` as a Python-stdlib-only CLI with subcommands `list`, `run`, `report`, and `compare`. It should depend only on the catalog file, the repository's existing `cargo` and `rustc` commands, and stable Criterion `raw.csv` outputs under `target/criterion/`. Do not make it depend on Criterion's JSON analysis files.

Create `bench/expr/suites.json` with one root object containing a `suites` array. Each suite row must contain the non-empty string fields `id`, `bench_target`, and `description`, plus a non-empty `scenarios` array of non-empty unique strings. The final committed catalog should describe these four suites: `syntax`, `logical`, `event`, and `waveform_host`. The harness should derive and record a deterministic catalog fingerprint from this file so resume and compare flows can reject stale or mismatched run directories safely.

In `Cargo.toml`, the final four `[[bench]]` entries must be:

    name = "expr_syntax"
    path = "bench/expr/expr_syntax.rs"

    name = "expr_logical"
    path = "bench/expr/expr_logical.rs"

    name = "expr_event"
    path = "bench/expr/expr_event.rs"

    name = "expr_waveform_host"
    path = "bench/expr/expr_waveform_host.rs"

In `bench/expr/support.rs`, define the shared benchmark support that all four suite files should use. At minimum this module should expose one shared `Criterion` configuration builder, the small reusable `ExprType` constructor helpers that are currently duplicated across suite files, the reusable in-memory `ExpressionHost` sampling behavior, and the waveform fixture-path helper used by the dump-backed suite. Keep benchmark-specific source strings, signal timelines, and assertions in the suite files themselves so each measured scenario stays easy to understand. In the final Rust bench files, every `c.bench_function(...)` call must use the exact string form `suite__scenario` so Criterion-side identity and exported identity stay aligned.

Replace the old helper tests with `bench/expr/test_perf.py`. These tests should cover catalog loading, run-directory validation, README generation with and without compare context, failure on non-empty fresh run directories, guarded `--missing-only` resume, stale-catalog rejection, duplicate suite-plus-scenario rejection, compare failures on missing or mismatched suites, artifact-name namespacing, and metadata-match enforcement. The goal is to prove the new harness end-to-end contract in one place rather than preserving the older `capture.py` / `compare.py` split.

Revision note (2026-03-19 / OpenCode): initial plan authored from the current expression microbenchmark workflow, the existing grouped `bench/e2e/perf.py` model, and the user request to replace history-shaped benchmark artifacts with one functional baseline for the whole expression surface. Revised after focused architecture, docs, and performance review to stage the new harness before Rust suite renames, define the meaning of “one baseline” explicitly, require suite-prefixed Criterion and exported identities, make `run --run-dir` safety and resume rules explicit, update the live-doc sweep to include `docs/DESIGN.md`, keep same-commit verify runs temporary, and replace destructive baseline refresh with a temporary validated capture before the committed baseline directory is swapped.

Revision note (2026-03-20 / OpenCode): updated after Milestone 1 implementation to record the completed catalog + harness + tests, capture the partial-summary requirement that fell out of the `--missing-only` contract, and mark the branch as ready to proceed to the functional Rust suite restructure.

Revision note (2026-03-20 / OpenCode): updated after Milestone 2 implementation to record the final functional bench target split, the new shared bench support module, and the crate-root waveform-module constraint that kept `expr_waveform_host` slightly more specialized than the other three suites.

Revision note (2026-03-20 / OpenCode): updated after Milestone 3 implementation to record the completed live-doc/helper cleanup, the new make targets, and the fact that the last repository-wide stale-name sweep remains intentionally tied to Milestone 4 because the historical run directories are still committed at this point.

Revision note (2026-03-20 / OpenCode): updated after Milestone 4 implementation to record the promoted clean `baseline/` run, the deletion of the thirteen historical run directories, and the remaining review-only status for branch closeout.

Revision note (2026-03-20 / OpenCode): updated after Milestone 5 completion to record the review-lane findings, the provenance and determinism fixes applied in response, the fresh clean baseline recapture from the post-review code state, and the final clean control-pass review.
