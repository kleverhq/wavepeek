# Bench E2E Scope Benchmarks

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

After this change, the benchmark catalog under `bench/e2e/` will measure `wavepeek scope` directly instead of inferring hierarchy performance only from `signal` and `change` scenarios. Operators will be able to compare small, medium, and large scope-listing workloads across pinned waveform fixtures and immediately see whether a scope-walk regression came from deep traversal, regex filtering, or large JSON payload generation.

The result is observable by running `python3 bench/e2e/perf.py list --filter '^scope_'` and `python3 bench/e2e/perf.py run --filter '^scope_'`: the catalog will include dedicated `scope_*` entries, the run directory will produce `.wavepeek.json` artifacts for them, and the grouped benchmark report will render a new `## scope` section.

## Non-Goals

This plan does not change Rust runtime behavior for the `scope` command. This plan does not add human-mode `--tree` benchmarks because the harness captures machine output and relies on deterministic JSON artifacts. This plan does not alter compare thresholds, benchmark math, or fixture provisioning. This plan does not broaden pre-commit smoke beyond the minimal cross-category subset already enforced by `bench/e2e/tests_commit.json`.

## Progress

- [x] (2026-03-08 11:08Z) Explored `bench/e2e` harness structure, current catalog categories, smoke subset rules, and Make/pre-commit wiring.
- [x] (2026-03-08 11:08Z) Manually inspected candidate fixtures with `./target/release/wavepeek scope` to measure hierarchy size, maximum depth, and stable filter candidates.
- [x] (2026-03-08 11:08Z) Chose the minimal benchmark set and documented exact commands, fixture paths, and acceptance targets in this active ExecPlan.
- [x] (2026-03-08 11:12Z) Added TDD coverage in `bench/e2e/test_perf.py` for the expanded smoke catalog contract and exact `scope` benchmark set; confirmed red before catalog edits and green after.
- [x] (2026-03-08 11:12Z) Added `scope` benchmark entries to `bench/e2e/tests.json` and `bench/e2e/tests_commit.json` and verified `perf.py list/run` for `^scope_` workloads.
- [ ] Refresh committed baseline artifacts for the new `scope` scenarios and verify report grouping.
- [ ] Run validation, execute mandatory review passes, apply fixes, and move the finished plan to `docs/exec-plans/completed/`.

## Surprises & Discoveries

- Observation: the current `scope` benchmark surface is completely absent; only `signal` indirectly exercises hierarchy listing today.
  Evidence: `bench/e2e/tests.json` contains `info`, `signal`, `value`, and `change` categories, but no `scope` category entries.

- Observation: exact numeric `--max-depth` values avoid `limit disabled` warnings while still reaching full hierarchy depth on the pinned fixtures.
  Evidence: manual checks with `./target/release/wavepeek scope --json` showed maximum depths of 7 for `scr1_max_axi_riscv_compliance.fst`, 12 for `chipyard_DualRocketConfig_dhrystone.fst`, and 13 for `chipyard_ClusteredRocketConfig_mt-memcpy.fst`; using those values returned full counts with empty `warnings` arrays.

- Observation: `wavepeek scope` has no `--scope` selector; it always starts at the dump roots and uses `--filter` plus `--max-depth` to shape work.
  Evidence: `docs/DESIGN.md` defines `scope --waves <file> [--max ...] [--max-depth ...] [--filter <regex>] [--tree] [--json]`, and the built CLI accepted the planned commands only after removing the nonexistent `--scope` flag from the draft.

- Observation: large Chipyard fixtures expose stable full-path substrings such as `frontend` and `tile_prci_domain` that produce deterministic filtered subsets without relying on fragile exact leaf names.
  Evidence: manual scope scans returned 118 `frontend` matches on `chipyard_DualRocketConfig_dhrystone.fst` and 2512 `tile_prci_domain` matches on `chipyard_ClusteredRocketConfig_mt-memcpy.fst`.

## Decision Log

- Decision: add exactly three full-catalog `scope` benchmarks.
  Rationale: three cases are enough to cover the important performance shapes without bloating the catalog: a small full traversal, a medium filtered deep traversal, and a large full traversal.
  Date/Author: 2026-03-08 / OpenCode

- Decision: use existing required fixtures only: `scr1_max_axi_riscv_compliance.fst`, `chipyard_DualRocketConfig_dhrystone.fst`, and `chipyard_ClusteredRocketConfig_mt-memcpy.fst`.
  Rationale: these binaries are already pinned in `Makefile`, already available in the devcontainer image, and span small, medium, and large hierarchy sizes.
  Date/Author: 2026-03-08 / OpenCode

- Decision: prefer JSON-mode scope benchmarks and skip `--tree` human-output scenarios.
  Rationale: `bench/e2e/perf.py` captures functional payloads as JSON and compares `data` only, so JSON scenarios fit the harness and keep artifacts deterministic.
  Date/Author: 2026-03-08 / OpenCode

- Decision: name the full-traversal cases with `all` rather than `top`.
  Rationale: the command does not accept a root-scope selector, so `all` more accurately describes the actual workload while still matching the dump-root traversal behavior.
  Date/Author: 2026-03-08 / OpenCode

- Decision: extend `tests_commit.json` with two scope smoke cases, keeping the existing two-per-category pattern.
  Rationale: smoke coverage should detect scope-catalog drift in pre-commit without turning the lightweight catalog into a second full suite.
  Date/Author: 2026-03-08 / OpenCode

## Outcomes & Retrospective

Current status: planning and fixture reconnaissance are complete; implementation, validation, and review remain.

Planned outcome: `bench/e2e/tests.json` will gain a dedicated `scope` category, `bench/e2e/tests_commit.json` will exercise two representative scope scenarios in smoke mode, and `bench/e2e/runs/baseline/README.md` plus new baseline artifacts will expose the new category in committed benchmark evidence.

Residual risk at this stage: low. The chosen fixtures and filters have already been checked manually with the built release binary, so implementation risk is mostly catalog consistency and baseline refresh work.

## Context and Orientation

The benchmark harness lives in `bench/e2e/perf.py`. A benchmark “catalog” is the JSON file listing named benchmark cases with their category, run counts, warmup count, command vector, and metadata. The full catalog is `bench/e2e/tests.json`; the lightweight pre-commit catalog is `bench/e2e/tests_commit.json`. `perf.py run` benchmarks each command through `hyperfine`, then reruns the same command in JSON mode to save a deterministic functional artifact as `<test>.wavepeek.json`.

The harness unit tests live in `bench/e2e/test_perf.py`. These tests already lock the current smoke subset contract: exact test names, per-category counts, `runs=1`, `warmup=0`, and name-to-command parity against the full catalog. Adding scope smoke scenarios requires updating those expectations first so the contract change is explicit and test-driven.

Committed baseline data lives in `bench/e2e/runs/baseline/`. Each benchmark case has a `.hyperfine.json` timing artifact and a `.wavepeek.json` functional artifact. `bench/e2e/runs/baseline/README.md` is regenerated from those artifacts and groups entries by `category`, so adding a new `scope` category requires refreshing baseline artifacts and the report.

The chosen scope benchmarks are:

1. `scope_scr1_all_depth7_json`: small full traversal on `/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst` with `--max 200000 --max-depth 7 --json`, returning 136 scope rows.
2. `scope_dualrocket_filter_frontend_depth12_json`: medium filtered deep traversal on `/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst` with `--filter '.*frontend.*' --max 200000 --max-depth 12 --json`, returning 118 rows.
3. `scope_clustered_all_depth13_json`: large full traversal on `/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst` with `--max 200000 --max-depth 13 --json`, returning 4625 rows.

The chosen scope smoke subset is:

1. `scope_scr1_all_depth7_json` for a low-cost basic hierarchy walk.
2. `scope_clustered_all_depth13_json` for a high-fanout large hierarchy walk.

## Open Questions

No blocking questions remain. If review reveals the clustered full traversal is too expensive for smoke, the fallback is to swap the smoke case from `scope_clustered_all_depth13_json` to `scope_dualrocket_filter_frontend_depth12_json` while keeping the full catalog unchanged.

## Plan of Work

Milestone 1 adds tests first. Update `bench/e2e/test_perf.py` so the smoke-catalog contract expects ten total tests instead of eight, introduces the two new `scope_*` names, and verifies the new category distribution is `2` each for `change`, `info`, `scope`, `signal`, and `value`. Add one more test that proves the full catalog contains the exact three planned `scope` benchmarks. These tests should fail before the catalog files are edited.

Milestone 2 adds the benchmark definitions. Update `bench/e2e/tests.json` with the three scope cases listed in Context and Orientation, keeping command vectors deterministic and metadata compact but useful (`waves`, `scope`, `filter`, `max_depth`). Update `bench/e2e/tests_commit.json` with the two smoke cases and `runs=1`, `warmup=0`. After this milestone, `python3 bench/e2e/perf.py list --filter '^scope_'` should print the three full-catalog scope benchmark names, and `python3 bench/e2e/perf.py list --tests bench/e2e/tests_commit.json --filter '^scope_'` should print the two smoke names.

Milestone 3 refreshes baseline evidence. Run the harness only for the new scope tests against `bench/e2e/runs/baseline` so existing artifacts remain intact and the README is regenerated with a `## scope` section. The committed baseline should include new `.hyperfine.json` and `.wavepeek.json` files for each added scope case and for the smoke subset cases that compare against baseline.

Milestone 4 validates and closes the work. Run targeted harness unit tests, a scope-only benchmark run against a temporary directory, the pre-commit smoke path, and the repo CI gate if time permits. Then execute the mandatory `ask-review` workflow: focused code lane for correctness/contracts, focused performance/docs lane for benchmark choice and metadata, followed by one fresh control pass on the consolidated diff. Resolve any findings, commit the fixes, move the plan from `docs/exec-plans/active/` to `docs/exec-plans/completed/`, and record the final outcome in this document.

### Concrete Steps

Run all commands from `/workspaces/perf-scope-tests`.

1. TDD red phase for smoke catalog expectations.

   Edit `bench/e2e/test_perf.py` to:

   - add `scope_scr1_all_depth7_json` and `scope_clustered_all_depth13_json` to the expected smoke test-name set;
   - change the expected smoke test count from `8` to `10`;
   - change the expected category distribution from four categories to five categories with `2` entries each.

   Red-phase command:

       WAVEPEEK_IN_CONTAINER=1 python3 -m unittest bench.e2e.test_perf

   Expected red evidence:

       FAIL: test_tests_commit_catalog_exact_subset_and_distribution (...)
       AssertionError: Items in the second set but not the first: 'scope_scr1_all_depth7_json'

2. Add scope benchmark definitions.

   Update `bench/e2e/tests.json` by inserting the three `scope` entries near the existing list-style commands (`info` and `signal` are the closest neighbors). Use these exact command vectors:

       ["{wavepeek_bin}", "scope", "--waves", "/opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst", "--max", "200000", "--max-depth", "7", "--json"]

       ["{wavepeek_bin}", "scope", "--waves", "/opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst", "--filter", ".*frontend.*", "--max", "200000", "--max-depth", "12", "--json"]

       ["{wavepeek_bin}", "scope", "--waves", "/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst", "--max", "200000", "--max-depth", "13", "--json"]

   Use metadata that records the waveform path plus the workload-shaping knobs. For the filtered case, include `filter: ".*frontend.*"`; for the full traversals, include `filter: ".*"`.

   Update `bench/e2e/tests_commit.json` by adding the two smoke scope cases with commands exactly matching the same entries in `tests.json`, but with `runs=1` and `warmup=0` like the rest of the smoke catalog.

3. Green-phase tests and targeted scope run.

       WAVEPEEK_IN_CONTAINER=1 python3 -m unittest bench.e2e.test_perf
       WAVEPEEK_IN_CONTAINER=1 python3 bench/e2e/perf.py list --filter '^scope_'
       WAVEPEEK_IN_CONTAINER=1 python3 bench/e2e/perf.py list --tests bench/e2e/tests_commit.json --filter '^scope_'
       WAVEPEEK_BIN=./target/release/wavepeek WAVEPEEK_IN_CONTAINER=1 python3 bench/e2e/perf.py run --filter '^scope_' --run-dir /tmp/wavepeek-scope-run

   Expected green evidence:

       scope_clustered_all_depth13_json
       scope_dualrocket_filter_frontend_depth12_json
       scope_scr1_all_depth7_json

4. Refresh committed baseline artifacts for the new scope cases.

       rm -f bench/e2e/runs/baseline/scope_*.hyperfine.json bench/e2e/runs/baseline/scope_*.wavepeek.json
       WAVEPEEK_BIN=./target/release/wavepeek WAVEPEEK_IN_CONTAINER=1 python3 bench/e2e/perf.py run --filter '^scope_' --run-dir bench/e2e/runs/baseline

   This is safe to rerun. It only rewrites the scope-related artifacts and regenerates `bench/e2e/runs/baseline/README.md` from the full artifact directory.

5. Validation gates.

       WAVEPEEK_IN_CONTAINER=1 make test-bench-e2e
       WAVEPEEK_IN_CONTAINER=1 make bench-e2e-smoke-commit
       WAVEPEEK_IN_CONTAINER=1 make ci

6. Mandatory review cycle and closeout.

   - Run focused review lanes in parallel:
     - code lane for `bench/e2e/test_perf.py`, `bench/e2e/tests.json`, `bench/e2e/tests_commit.json`;
     - performance/docs lane for benchmark choice, metadata clarity, and baseline refresh artifacts.
   - Apply fixes and commit them as a separate atomic unit if needed.
   - Run one fresh control review pass on the consolidated diff.
   - Move this plan to `docs/exec-plans/completed/2026-03-08-bench-e2e-scope-tests/PLAN.md` and update `Progress` plus `Outcomes & Retrospective` to reflect final validation and review results.

### Validation and Acceptance

Acceptance is complete only when all conditions below are true:

- `bench/e2e/tests.json` contains exactly three new `scope_*` entries with deterministic commands, pinned fixture paths, and metadata describing the workload knobs.
- `bench/e2e/tests_commit.json` contains exactly two `scope_*` entries, and `bench/e2e/test_perf.py` locks the new 10-test smoke contract and 2-per-category distribution.
- `python3 bench/e2e/perf.py list --filter '^scope_'` prints exactly the three scope benchmark names.
- `python3 bench/e2e/perf.py list --tests bench/e2e/tests_commit.json --filter '^scope_'` prints exactly the two smoke scope names.
- A scope-only harness run produces valid `.hyperfine.json` and `.wavepeek.json` artifacts for the new tests.
- `bench/e2e/runs/baseline/README.md` contains a `## scope` section and the corresponding committed baseline artifact files exist.
- `make test-bench-e2e` passes.
- `make bench-e2e-smoke-commit` passes.
- `make ci` passes.
- Mandatory review passes are complete: lane reviews are addressed, the fresh control pass is clean or fully resolved, and the finished plan is moved to `docs/exec-plans/completed/`.

### Idempotence and Recovery

The catalog edits are additive and safe to rerun. The baseline refresh step deletes only `scope_*` baseline artifacts before regenerating them, so it can be retried without disturbing other categories. If a benchmark run is interrupted, rerun the same `perf.py run --filter '^scope_'` command; the harness rewrites the affected scope artifacts and regenerates the report.

If review or validation shows the clustered full traversal is too expensive for smoke, recover by editing only `bench/e2e/tests_commit.json` and `bench/e2e/test_perf.py` to swap the smoke case to `scope_dualrocket_filter_frontend_depth12_json`, then rerun the validation and smoke commands.

### Artifacts and Notes

Manual fixture reconnaissance that drove the benchmark selection:

    ./target/release/wavepeek scope --waves /opt/rtl-artifacts/scr1_max_axi_riscv_compliance.fst --max 200000 --max-depth 7 --json
    -> 136 rows, warnings=[]

    ./target/release/wavepeek scope --waves /opt/rtl-artifacts/chipyard_DualRocketConfig_dhrystone.fst --filter '.*frontend.*' --max 200000 --max-depth 12 --json
    -> 118 rows, warnings=[]

    ./target/release/wavepeek scope --waves /opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst --max 200000 --max-depth 13 --json
    -> 4625 rows, warnings=[]

Expected example entries in the regenerated baseline report:

    ## scope

    | test | mean_s | meta |
    | --- | --- | --- |
     | scope_clustered_all_depth13_json | ... | waves=/opt/rtl-artifacts/chipyard_ClusteredRocketConfig_mt-memcpy.fst filter=.* max_depth=13 |

### Interfaces and Dependencies

No new Python modules or Rust interfaces are required. This work depends on the existing `bench/e2e/perf.py` catalog schema, `hyperfine` availability in the devcontainer, and the pre-provisioned fixtures listed in `Makefile` under `REQUIRED_RTL_ARTIFACTS`. The changed files are expected to be:

- `bench/e2e/tests.json`
- `bench/e2e/tests_commit.json`
- `bench/e2e/test_perf.py`
- `bench/e2e/runs/baseline/README.md`
- new baseline artifacts under `bench/e2e/runs/baseline/` named `scope_*.hyperfine.json` and `scope_*.wavepeek.json`
- this plan file, later moved from `docs/exec-plans/active/` to `docs/exec-plans/completed/`

Revision note: created the initial plan after repository exploration and manual fixture inspection so implementation can proceed autonomously with a fixed minimal benchmark set.
