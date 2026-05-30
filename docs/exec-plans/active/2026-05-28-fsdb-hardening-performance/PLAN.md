# Harden and speed up FSDB command execution

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill. A contributor must be able to start from only this file, a clean checkout, and the local Verdi-enabled development environment, then deliver the behavior described here without relying on conversation history.

## Purpose / Big Picture

After this work, a Verdi-enabled build of `wavepeek` should run the already-supported FSDB commands reliably on larger files, with repeatable benchmarks that exercise the same command matrix as the existing FST benchmark suite. Users should be able to prepare generated FSDB fixtures, run FSDB regression tests, run FSDB end-to-end benchmarks against converted RTL artifacts under the resolved RTL artifact directory, normally `/opt/rtl-artifacts`, and see that the FSDB command outputs match the existing FST benchmark outputs for the same designs.

The implementation also reduces avoidable FSDB Reader churn inside one command. Today the portable FSDB `change` and `property` path is correct, but each uncached sampling or event operation can reload signal lists and recreate native value-change traversal handles. This plan replaces that with a command-scoped native signal session that owns loaded signals and reusable traversal handles for the duration of one `Waveform` backend instance. In plain terms: one command should do less repetitive setup work while still closing and unloading everything when the command ends. Glamour is not required; boring resource ownership is.

## Non-Goals

This plan does not add Verdi or FSDB as a default dependency. Default `wavepeek` builds, default CI, and default pre-commit remain VCD/FST-only unless a target explicitly opts into `--features fsdb` and a local Verdi SDK.

This plan does not change public command names, flags, JSON schema, or human output shape. FSDB remains hidden behind the existing `--waves <FILE>` waveform abstraction.

This plan does not commit generated `.fsdb` files or proprietary Verdi payloads. Generated FSDB files are binary artifacts and must remain ignored.

This plan does not implement analog, real, string, sparse array, multidimensional array, or arbitrary rich datatype value decoding unless a deterministic fixture exists and the work is explicitly added to this living plan. The required scope is digital bit-vector, integer-like, enum-core, and raw-event behavior already covered by the portable FSDB command path.

This plan does not create durable file, module, function, target, test, or benchmark names that include roadmap labels. Use descriptive names such as `fsdb_signal_session`, `tests_fsdb.json`, and `bench-e2e-fsdb-run`; do not create names with milestone prefixes or suffixes.

## Progress

- [x] (2026-05-28 00:00Z) Read the FSDB architecture milestone, current FSDB code, FSDB fixture script, root Makefile targets, benchmark harness, development docs, and the completed portable `change`/`property` plan.
- [x] (2026-05-28 00:00Z) Drafted this execution plan under `docs/exec-plans/active/2026-05-28-fsdb-hardening-performance/PLAN.md` with descriptive names and no roadmap-label prefixes or suffixes.
- [x] (2026-05-28 00:00Z) Ran focused read-only review lanes for plan completeness, FSDB architecture/resource safety, benchmark/test feasibility, and an independent control pass; incorporated substantive findings into this plan.
- [x] (2026-05-30 11:35Z) Implemented FSDB benchmark fixture preparation and catalog support: `scripts/prepare_fsdb_fixtures.sh` now converts checked-in VCD fixtures and resolved RTL `*.fst` artifacts, `bench/e2e/tests_fsdb.json` mirrors the FST catalog with `.fsdb` paths, `bench/e2e/perf.py` supports artifact-root remapping, `report --tests`, and strict `compare --functional-only`, Makefile FSDB benchmark targets exist, and Python auxiliary tests pass.
- [ ] Implement command-scoped FSDB signal sessions and switch the backend to use them.
- [ ] Add hardening tests for missing, truncated, non-FSDB, repeated-use, and unsupported-datatype behavior.
- [ ] Run the required default and FSDB validation gates, plus the FSDB benchmark smoke and parity checks.
- [ ] Update documentation and retrospective notes, then archive or keep this plan active according to repository practice at handoff.

## Surprises & Discoveries

- Observation: `docs/fsdb/arch.md` defines this hardening/performance milestone as large-file measurement, in-command index caching, memory unload/free verification, graceful missing/truncated/non-FSDB errors, and fixture-backed datatype expansion only after coverage exists.
  Evidence: the hardening/performance architecture section lists exactly those bullets; the preceding completed milestone note says persistent native sampler/session reuse and large-window FSDB performance tuning were intentionally deferred.

- Observation: the current FSDB backend already caches metadata, hierarchy, expression sampled values, and raw-event occurrence results, but native sampling and candidate operations still use one-shot Reader calls.
  Evidence: `src/waveform/fsdb_backend.rs` stores `raw_metadata`, `hierarchy`, `expr_sample_cache`, and `event_occurrence_cache`; `sample_resolved_optional`, `collect_signal_change_times`, and `signal_event_occurred` call through `FsdbReader` one operation at a time.

- Observation: the native shim currently loads a Reader signal list and creates value-change traversal handles inside each one-shot sampling, candidate, or event call.
  Evidence: `native/fsdb/wavepeek_fsdb_shim.cpp` has `signal_list_guard::load`, `fill_sample_record`, `append_change_times_for_signal`, and `read_exact_event_occurrence`; each public operation owns its own guard and frees traversal handles before returning.

- Observation: the existing fixture preparation target converts only checked-in VCD fixtures, not RTL FST artifacts.
  Evidence: `scripts/prepare_fsdb_fixtures.sh` scans `tests/fixtures/hand` for `*.vcd` and writes ignored files under `tests/fixtures/fsdb`; it does not inspect `/opt/rtl-artifacts` or any resolved RTL artifact directory.

- Observation: the end-to-end benchmark harness already supports custom catalogs for `list` and `run`, but `report` still reads `bench/e2e/tests.json` unconditionally, and `compare` always requires a timing threshold.
  Evidence: `bench/e2e/perf.py` accepts `--tests` on `list` and `run`; `cmd_report` currently calls `load_tests(TESTS_PATH)`, and the compare parser marks `--max-negative-delta-pct` as required.

- Observation: the local development container currently has eight FST artifacts under `/opt/rtl-artifacts` that the existing FST benchmark catalog references.
  Evidence: a binary-safe `find /opt/rtl-artifacts -maxdepth 1 -type f -printf '%f %s\n'` listed `chipyard_ClusteredRocketConfig_dhrystone.fst`, `chipyard_ClusteredRocketConfig_mt-memcpy.fst`, `chipyard_DualRocketConfig_dhrystone.fst`, `picorv32_test_ez_vcd.fst`, `picorv32_test_vcd.fst`, `scr1_max_axi_coremark.fst`, `scr1_max_axi_isr_sample.fst`, and `scr1_max_axi_riscv_compliance.fst`.

- Observation: the currently running container image still has `/opt/rtl-artifacts` owned by `root`, so direct RTL FSDB conversion there fails before conversion by design until the devcontainer image is rebuilt or `RTL_ARTIFACTS_DIR` is overridden to a writable copy.
  Evidence: `ls -ld /opt/rtl-artifacts` reported `root root`; a validation run used `WAVEPEEK_RTL_ARTIFACTS_DIR=$PWD/tmp/fsdb-script-test` and successfully converted a copied `picorv32_test_ez_vcd.fst` artifact without reading the binary fixture as text.

- Observation: focused review found that the first draft was too optimistic about writable `/opt/rtl-artifacts`, benchmark path resolution, functional-only parity strictness, C ABI spelling, session lifetime safety, and native traversal-handle retention.
  Evidence: review lanes reported that `/opt/rtl-artifacts` is root-owned in the current container, `bench/e2e/tests_fsdb.json` would otherwise hard-code paths that can diverge from `WAVEPEEK_RTL_ARTIFACTS_DIR`, `perf.py run` alone does not fail functional mismatches, timeout artifacts `{}` are currently non-blocking, `std::size_t` is not valid in the public C-compatible header, a session wrapper without a reader lifetime tie can outlive the reader, and caching every traversal handle in a wildcard command can retain excessive native resources.

## Decision Log

- Decision: make benchmark FSDB inputs live next to the corresponding FST files in the resolved RTL artifact directory, normally `/opt/rtl-artifacts`, using the same basename with `.fsdb` extension, and make the default devcontainer artifact directory writable by the `ubuntu` user.
  Rationale: the user explicitly requested that the root Make target that prepares FSDB fixtures also walk `/opt/rtl-artifacts` and convert the FST files there. Keeping converted files next to their source keeps `bench/e2e/tests_fsdb.json` mechanically identical to `bench/e2e/tests.json` except for `.fst` to `.fsdb` path changes. The current container copy is root-owned, so the plan must either make the image writable for this generated-local-artifact use case or fail with a useful override message instead of face-planting halfway through conversion.
  Date/Author: 2026-05-28 / Grin

- Decision: keep `bench/e2e/tests_fsdb.json` as a committed explicit catalog, and add a Python unit test that verifies it stays in lockstep with `bench/e2e/tests.json` except for waveform path extensions and FSDB-specific metadata.
  Rationale: the benchmark harness uses explicit JSON catalogs. A generated-at-runtime catalog would hide drift, while a committed catalog plus a lockstep test makes review diffs boring and catches stale entries. Boring wins again, despite protests from the machinery spirits.
  Date/Author: 2026-05-28 / Grin

- Decision: preserve benchmark test names between `tests.json` and `tests_fsdb.json`.
  Rationale: identical names let `bench/e2e/perf.py compare` check functional parity between an FSDB run and the existing FST baseline. Distinguishing the format belongs in the catalog path, run directory, and optional metadata, not in the test identity.
  Date/Author: 2026-05-28 / Grin

- Decision: implement performance reuse as a command-scoped native `wp_fsdb_signal_session` rather than a global cache, cross-command daemon, or Rust-only memoization layer.
  Rationale: FSDB Reader loaded signal lists and traversal handles are native resources tied to an open Reader object. A session owned by one `FsdbBackend` keeps the lifetime bounded to one CLI command, avoids global state, and gives native C++ code responsibility for freeing native handles it creates.
  Date/Author: 2026-05-28 / Grin

- Decision: keep the existing one-shot native operations available as compatibility wrappers, but switch the Rust FSDB backend to session methods.
  Rationale: compatibility wrappers keep current smoke tests and narrow callers simple, while the backend path that matters for large commands gets reuse. The wrappers must not invalidate an active session; if the native design cannot safely support both at once, wrappers should create a short-lived temporary session only when no persistent session is active.
  Date/Author: 2026-05-28 / Grin

- Decision: make FSDB benchmark parity support a first-class harness feature by adding `report --tests` and strict `compare --functional-only` behavior, with an explicit `--allow-golden-extra` escape hatch only for filtered smoke runs.
  Rationale: `run --tests bench/e2e/tests_fsdb.json` already writes a correct report immediately, but regenerating reports and validating FSDB-vs-FST output parity should not require pretending FSDB timing must beat the FST baseline. Timing is measured and reported; functional parity is a separate gate. Functional-only compare must fail missing revised artifacts, revised-only tests, functional mismatches, and timeout payloads by default; otherwise a half-run benchmark could stroll through the gate wearing a fake moustache.
  Date/Author: 2026-05-28 / Grin

- Decision: do not add rich datatype decoding in this slice unless a deterministic fixture is added first.
  Rationale: the architecture explicitly says richer datatypes come after fixture coverage. Guessing at proprietary datatype behavior without a fixture is how one builds a haunted adapter.
  Date/Author: 2026-05-28 / Grin

- Decision: tie every safe Rust `FsdbSignalSession` to the lifetime of its owning reader through an owned inner handle, and close sessions before the underlying native reader can be closed.
  Rationale: a raw native session pointer is tied to `wp_fsdb_reader`. Safe Rust must not allow a reader to drop while a session can still call `wp_fsdb_close_signal_session`. An `Arc`-owned reader inner, or an equally strict closure API, makes the lifetime rule enforceable instead of depending on field-drop folklore and crossed fingers.
  Date/Author: 2026-05-28 / Grin

- Decision: make native traversal-handle reuse lazy and bounded rather than caching one handle for every loaded idcode.
  Rationale: wildcard `change` and `property` commands can involve very large signal sets. Loading the signal list once is useful, but retaining thousands of native traversal handles for the whole command risks trading CPU churn for memory pressure. Lazy handles with a cap, plus temporary handles for large candidate sweeps, gives reuse on hot sampled/event paths without building a native handle barn.
  Date/Author: 2026-05-28 / Grin

## Outcomes & Retrospective

Milestone 1 is implemented. The repository now has a committed FSDB benchmark catalog, fixture preparation that can convert both hand-authored VCD fixtures and external RTL FST artifacts, FSDB benchmark Make targets, a writable devcontainer artifact-copy fix for future rebuilt images, catalog path remapping for nonstandard artifact roots, and functional-only FSDB-vs-FST compare behavior that treats timeout artifacts as failures. Validation so far: `python3 -B -m unittest discover -s bench/e2e -p 'test_*.py'`, `python3 -m py_compile bench/e2e/perf.py bench/e2e/test_perf.py scripts/check_fsdb_bench_artifacts.py`, `WAVEPEEK_IN_CONTAINER=1 make test-aux`, `make -n check-fsdb-rtl-artifacts`, and a direct fixture-preparation run against a writable temporary RTL artifact directory. The remaining work is the native command-scoped signal session, hardening tests, full default/FSDB gates, benchmark smoke/parity measurements, focused review, and final handoff.

## Context and Orientation

`wavepeek` is a Rust CLI for inspecting digital waveform dumps. A waveform dump is a file that records signal values over simulation time. The default backend uses the `wellen` Rust crate for VCD and FST files. VCD is a text waveform format; FST is a compact binary waveform format. FSDB is Synopsys Verdi's proprietary binary waveform format. The repository must not vendor Verdi headers, libraries, docs, generated bindings, or FSDB files. FSDB support is optional and compiled only with the Cargo feature named `fsdb`.

The public command layer lives under `src/engine/`. The important commands for this plan are `info`, `scope`, `signal`, `value`, `change`, and `property`. The command layer talks to `src/waveform/mod.rs`, whose `Waveform` facade chooses either `Backend::Wellen` or, when compiled with `--features fsdb`, `Backend::Fsdb`. The Wellen backend in `src/waveform/wellen_backend.rs` has indexed fast paths for VCD/FST. The FSDB backend in `src/waveform/fsdb_backend.rs` currently implements the backend-neutral operations needed by the commands using a native C++ shim.

The native FSDB shim lives in `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp`. Rust FFI wrappers live in `src/waveform/fsdb_native.rs`. The shim links against the local Verdi FSDB Reader SDK through build logic that is already guarded by the `fsdb` feature. Native code uses C-compatible structs and functions so Rust never depends on proprietary C++ types directly.

The completed portable command work is recorded in `docs/exec-plans/completed/2026-05-27-fsdb-change-property-portable/PLAN.md`. That work made FSDB `change` and `property` correct by implementing candidate timestamp collection, sampled digital expression values, exact raw-event occurrence checks, and nonzero-start previous timestamp handling. It deliberately deferred persistent native sampler/session reuse and large-window performance tuning to this plan.

A candidate timestamp is a raw dump tick where at least one signal relevant to a command changed or where a raw event occurred. `change` and `property` use candidate timestamps to decide when to sample outputs and evaluate expressions. A sampled value at or before time `t` is the last value change for a signal whose raw time is less than or equal to `t`. An exact raw-event occurrence is true only when the raw event variable has a value-change record exactly at `t`.

The current FSDB one-shot native operations are correct but repetitive. For each sample or candidate call they reset the FSDB Reader signal list, add selected idcodes, load signals, create value-change traversal handles, perform the query, free handles, unload signals, and reset the list. This is acceptable for correctness fixtures. It is suspect on large windows where the same set of signals is sampled many times. Suspiciously expensive machinery should be measured before and after changing it; the benchmark work below exists to keep us honest.

FSDB regression tests live in `tests/fsdb_cli.rs`. Generated FSDB fixtures are created from committed VCD fixtures in `tests/fixtures/hand/` and written to ignored paths under `tests/fixtures/fsdb/`. The root target `make prepare-fsdb-fixtures` runs `scripts/prepare_fsdb_fixtures.sh` after Verdi availability checks. It currently only needs `vcd2fsdb`; after this plan it must also use `fst2vcd` to convert RTL FST artifacts before converting the temporary VCD to FSDB.

The end-to-end benchmark harness lives in `bench/e2e/perf.py`. Its default catalog is `bench/e2e/tests.json`; a smaller pre-commit catalog is `bench/e2e/tests_commit.json`; committed baseline output lives under `bench/e2e/runs/baseline/`. Benchmarks use `hyperfine` for timing and also capture each command's JSON `data` and `warnings` into `<test_name>.wavepeek.json`. Functional compare ignores warning text but fails when `data` differs. That behavior is useful for FSDB-vs-FST parity. The current catalogs use canonical `/opt/rtl-artifacts` paths; this plan requires the harness to remap that prefix to the resolved artifact root from `WAVEPEEK_RTL_ARTIFACTS_DIR` or `RTL_ARTIFACTS_DIR` when those variables point elsewhere.

The repository is container-first. Most Make targets require `WAVEPEEK_IN_CONTAINER=1`. FSDB targets also require a Verdi SDK that `.devcontainer/resolve_verdi_home.sh` can find. The default development docs are in `docs/DEVELOPMENT.md`. The root `AGENTS.md` and local breadcrumb files are navigation maps; they do not replace the contracts in this plan.

Critical safety rule: treat `.fst` and `.fsdb` files as binary files. Do not read them with text file tools or the agent Read tool. Use binary-safe metadata commands such as `find`, `ls`, `stat`, `du`, checksums, or Verdi/wellen tooling through the intended CLI paths.

## Open Questions

There are no blocking open questions for the required implementation. The default devcontainer path is still `/opt/rtl-artifacts`, and this plan requires the image to make that directory writable by the `ubuntu` user. In any other local environment where the resolved artifact directory is not writable, use `WAVEPEEK_RTL_ARTIFACTS_DIR` or `RTL_ARTIFACTS_DIR` to point at a writable complete copy of the RTL artifacts, then rerun `make prepare-fsdb-fixtures`; the benchmark harness path remapping keeps the committed catalogs usable with that override.

Rich datatype expansion is intentionally deferred unless this plan is revised with named fixtures and expected output. Do not smuggle it in while chasing performance numbers. The goat will not cooperate.

## Plan of Work

The work proceeds in five milestones. Each milestone ends with an observable behavior or a concrete validation command so the next contributor can tell whether the floor is still underneath them.

### Milestone 1: FSDB benchmark fixture and catalog infrastructure

At the end of this milestone, the repository can generate FSDB versions of both checked-in VCD fixtures and the large RTL FST artifacts, and the benchmark harness can run the existing end-to-end benchmark matrix against the FSDB copies.

Update `scripts/prepare_fsdb_fixtures.sh`. Keep the existing VCD-to-FSDB behavior for `tests/fixtures/hand/*.vcd` into `tests/fixtures/fsdb/*.fsdb`. Factor the script into small shell functions with descriptive names: `require_tool`, `convert_vcd_fixtures`, `resolve_rtl_artifacts_dir`, `convert_rtl_fst_artifacts`, `convert_vcd_to_fsdb`, and `convert_fst_to_fsdb`. Add `fst2vcd` as a required tool only for the RTL artifact conversion step. The script should resolve the RTL artifact directory by invoking `.devcontainer/resolve_rtl_artifacts_dir.sh`, not by open-coding a partial fallback list. That helper already prefers `WAVEPEEK_RTL_ARTIFACTS_DIR`, then `RTL_ARTIFACTS_DIR`, then readable `/opt/rtl-artifacts`, then `~/.cache/wavepeek/rtl-artifacts`.

Update `.devcontainer/Dockerfile` so the copied `/opt/rtl-artifacts` tree is writable by the `ubuntu` user in the default dev and CI images, for example by using `COPY --from=rtl_artifacts --chown=ubuntu:ubuntu /opt/rtl-artifacts /opt/rtl-artifacts` or an equivalent `chown` immediately after the copy. Because `.devcontainer/AGENTS.md` treats Dockerfile and environment-helper changes as part of the shared devcontainer/Codex contract, inspect `scripts/codex_setup.sh`, `scripts/codex_resume.sh`, and `scripts/codex_env_common.sh`; update them only if this ownership change or any helper change alters their assumptions. Do not make default non-FSDB targets generate FSDB files.

For every `*.fst` directly under the resolved artifact directory, write an FSDB file next to it with the same basename and `.fsdb` extension. For example, `/opt/rtl-artifacts/scr1_max_axi_coremark.fst` becomes `/opt/rtl-artifacts/scr1_max_axi_coremark.fsdb` when the resolved directory is `/opt/rtl-artifacts`. Use an intermediate VCD under `tmp/fsdb-fixtures/rtl-artifacts/` or a `mktemp -d` directory under repository-root `tmp/`, then run `vcd2fsdb` from that temporary VCD into a temporary FSDB path next to the final output, and finally `mv` the temporary FSDB into place. Skip conversion when the `.fsdb` output exists and is newer than its `.fst` input. If the resolved artifact directory is not writable, fail before conversion with a clear message that names the directory and tells the developer to use a writable `WAVEPEEK_RTL_ARTIFACTS_DIR` or `RTL_ARTIFACTS_DIR`; do not silently copy a partial benchmark corpus somewhere else. Print stable progress lines such as `info: fsdb fixture: up to date <path>` and `info: fsdb fixture: converted <source> -> <output>`. Remove `vcd2fsdbLog` and temporary logs on success and failure. Never print proprietary logs unless conversion fails, and even then print only the converter stderr/stdout captured from the local tool invocation.

Update `Makefile`. Keep `prepare-fsdb-fixtures` as the root target that invokes the script; its behavior expands to include RTL FST artifact conversion. Add variables for the FSDB benchmark catalog, FSDB benchmark baseline directory, and FSDB feature binary:

    BENCH_E2E_FSDB_TESTS := bench/e2e/tests_fsdb.json
    BENCH_E2E_FSDB_BASELINE_DIR := bench/e2e/runs/fsdb-baseline
    WAVEPEEK_FSDB_RELEASE_BIN := ./target/fsdb/release/wavepeek

Add `build-release-fsdb`, `check-fsdb-rtl-artifacts`, `bench-e2e-fsdb-update-baseline`, `bench-e2e-fsdb-run`, and `bench-e2e-fsdb-smoke-commit`. `build-release-fsdb` must be build-only: it requires Verdi and runs `VERDI_HOME="$$verdi_home" CARGO_TARGET_DIR=target/fsdb cargo build --release --features fsdb`, but it must not prepare fixtures or mutate artifact directories as a side effect. The FSDB test and benchmark targets must depend on `check-rtl-artifacts`, `prepare-fsdb-fixtures`, and `check-fsdb-rtl-artifacts`; `check-fsdb-rtl-artifacts` verifies that every FST required by the benchmark catalog has a neighboring `.fsdb` file in the resolved artifact directory. The benchmark targets must set `WAVEPEEK_BIN` to `$(WAVEPEEK_FSDB_RELEASE_BIN)` and pass `--tests $(BENCH_E2E_FSDB_TESTS)` to `bench/e2e/perf.py run`.

`bench-e2e-fsdb-smoke-commit` must be a real gate, not just a timing capture with a hopeful smile. It should run a representative filtered subset, such as one `info`, one `scope`, one `signal`, one `value`, and one `change` benchmark, into a temporary directory and then call `bench/e2e/perf.py compare --functional-only --allow-golden-extra --revised "$$tmp_revised" --golden "$(BENCH_E2E_BASELINE_DIR)"`. The `--allow-golden-extra` option is only for filtered smoke runs where the golden baseline intentionally contains tests that were not run in the revised directory. Full FSDB benchmark comparisons must not use it.

Create `bench/e2e/tests_fsdb.json` by copying `bench/e2e/tests.json` and replacing every waveform path ending in `.fst` with the corresponding `.fsdb` path under the same canonical directory. Preserve each test `name`, `category`, `runs`, `warmup`, command shape, and non-path metadata. Where a test has `meta.waves`, replace that path too. Add `meta.format: "fsdb"` only if the existing metadata pattern can tolerate the extra field in reports; otherwise keep metadata identical except paths. Do not rename tests with an FSDB suffix because identical names are what make functional parity compare possible.

Update `bench/e2e/perf.py` so catalog loading resolves canonical `/opt/rtl-artifacts/...` paths through the same artifact-root environment contract used by Make. In practice, when `WAVEPEEK_RTL_ARTIFACTS_DIR` or `RTL_ARTIFACTS_DIR` points somewhere other than `/opt/rtl-artifacts`, the harness should rewrite command tokens and `meta.waves` values under `/opt/rtl-artifacts` to the resolved directory before running commands or rendering reports. This keeps `tests.json` and `tests_fsdb.json` stable in git while still allowing cache-backed or override-backed artifact directories. Then make `report` accept `--tests` just like `list` and `run`.

Update `compare` with `--functional-only` and `--allow-golden-extra`. In functional-only mode it still loads and validates both sides' `<test_name>.wavepeek.json` artifacts, still reports missing or mismatched functional payloads, but skips timing threshold checks and does not require `--max-negative-delta-pct`. Functional-only mode must fail timeout payloads `{}` on either side, revised-only tests, missing matched artifacts, invalid JSON, and golden-only tests unless `--allow-golden-extra` is present. Keep the existing timing behavior unchanged when `--functional-only` is absent.

Update `bench/e2e/test_perf.py` with unit tests for artifact-root path remapping, `report --tests`, strict `compare --functional-only`, `compare --functional-only --allow-golden-extra`, timeout artifacts failing in functional-only mode, and catalog lockstep. The lockstep test should load both `tests.json` and `tests_fsdb.json`, normalize FSDB paths back to FST paths, remove any explicitly allowed `meta.format` difference, and compare the full normalized catalog objects. This catches drift in runs, warmups, command flags, signal lists, and non-path metadata instead of merely checking names and categories. This is a small guardrail, yes, but guardrails are cheaper than archaeology.

Update `bench/e2e/runs/.gitignore` only if the implementation commits a maintained `fsdb-baseline` run. The safer default is to leave generated FSDB runs ignored and treat `bench-e2e-fsdb-update-baseline` as a local maintainer command until the team explicitly decides to track a full FSDB baseline. If a tracked baseline is added, unignore only `fsdb-baseline/` and its contents, mirroring the existing `baseline/` exception.

Validation for this milestone:

    WAVEPEEK_IN_CONTAINER=1 make test-aux
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-smoke-commit

Expected observations are: `make test-aux` passes the Python unit tests; `make prepare-fsdb-fixtures` creates or skips generated FSDB files under `tests/fixtures/fsdb` and next to every FST under the resolved RTL artifact directory, normally `/opt/rtl-artifacts`; `check-fsdb-rtl-artifacts` confirms the required FSDB benchmark inputs exist; the smoke benchmark writes a temporary run directory and the functional-only compare reports no mismatches or timeout artifacts.

### Milestone 2: Command-scoped native signal session

At the end of this milestone, FSDB backend sampling, candidate collection, and raw-event occurrence reuse loaded signals and traversal handles within one command instead of repeating all native setup for every call.

Extend `native/fsdb/wavepeek_fsdb_shim.h` with an opaque signal session type and C ABI functions. Use these exact durable names unless a compile-time conflict forces a documented alternative:

    typedef struct wp_fsdb_signal_session wp_fsdb_signal_session;

    wp_fsdb_status wp_fsdb_open_signal_session(
        wp_fsdb_reader *reader,
        const uint64_t *idcodes,
        size_t count,
        wp_fsdb_signal_session **out,
        char **error_message
    );

    wp_fsdb_status wp_fsdb_signal_session_sample(
        wp_fsdb_signal_session *session,
        const uint64_t *idcodes,
        size_t count,
        uint64_t query_time_raw,
        wp_fsdb_sample_record **out,
        char **error_message
    );

    wp_fsdb_status wp_fsdb_signal_session_collect_change_times(
        wp_fsdb_signal_session *session,
        const uint64_t *idcodes,
        size_t count,
        uint64_t from_raw,
        uint64_t to_raw,
        wp_fsdb_time_list *out,
        char **error_message
    );

    wp_fsdb_status wp_fsdb_signal_session_event_occurred(
        wp_fsdb_signal_session *session,
        uint64_t idcode,
        uint64_t query_time_raw,
        int *occurred,
        char **error_message
    );

    void wp_fsdb_close_signal_session(wp_fsdb_signal_session *session);

Implement the session in `native/fsdb/wavepeek_fsdb_shim.cpp`. The session should be tied to one `wp_fsdb_reader`. It should own the unique idcode set and the loaded Reader signal list. Traversal handles must be created lazily, not eagerly for every loaded idcode. Cache handles only for hot sampled/event paths and keep the cache bounded with a descriptive constant such as `MAX_CACHED_TRAVERSE_HANDLES`; for large candidate sweeps, use temporary per-signal handles or evict old handles instead of retaining a handle for every wildcard source. Use RAII classes so every opened handle is freed, every loaded signal list is unloaded, and the Reader signal list is reset when the session closes or when session creation fails halfway. Keep the existing global Reader mutex and output suppressor around all Reader calls. Initialize every output parameter before doing work, including error paths.

The session must reject an idcode that was not part of the session's loaded idcode set. This prevents accidental use after a Rust-side union calculation bug. The error should be a normal `WavepeekError::File` through the existing native error path and should mention that the idcode is not loaded in the FSDB signal session.

Prevent multiple active sessions from corrupting one Reader's signal list. The preferred implementation is to add an `active_signal_session` pointer or counter inside the private `wp_fsdb_reader` struct, set it when a session opens, clear it when the matching session closes, and return a clear error if another session is opened while one is active. Public one-shot native operations must also check for an active session and return a normal `WP_FSDB_STATUS_ERROR` with a useful message if a persistent session owns the signal list; do not use C or C++ `assert` for this because an assertion failure would abort the CLI instead of producing a stable `error: file:` diagnostic. Do not let a one-shot operation reset a signal list that a persistent session owns.

Refactor existing helper code instead of duplicating it. `fill_sample_record`, `append_change_times_for_signal`, and `read_exact_event_occurrence` can accept cached traversal handles or a lookup helper. Preserve the existing final-value semantics at a raw timestamp: if multiple value changes share one raw timestamp, candidate deduplication may keep one schedule entry, but sampling must still return the final value according to the Reader's traversal behavior.

Extend `src/waveform/fsdb_native.rs` with a safe `FsdbSignalSession` wrapper. The wrapper must not be able to outlive its owning reader. The preferred shape is to make `FsdbReader` hold an `Arc<FsdbReaderInner>` where `FsdbReaderInner::drop` calls `wp_fsdb_close`, and make `FsdbSignalSession` hold both `NonNull<ffi::wp_fsdb_signal_session>` and an `Arc<FsdbReaderInner>` clone. Its `Drop` closes the session while the owner clone still keeps the native reader alive. An equally strict lifetime-parameter or closure-based API is acceptable if it prevents reader-before-session drop in safe Rust. The wrapper should expose methods named `sample_signal_values`, `collect_signal_change_times`, and `signal_event_occurred`, and convert all native arrays to owned Rust values exactly as the existing `FsdbReader` methods do. Keep the existing `FsdbReader` one-shot methods for smokes or compatibility, but implement the backend path through sessions.

Update `src/waveform/fsdb_backend.rs`. Add a command-scoped session cache with names such as `signal_session` and `loaded_session_idcodes`. Use a sorted or hash set of `SignalId` values to track what the current session has loaded. Add a helper such as `ensure_signal_session(&mut self, idcodes: &[u64]) -> Result<&FsdbSignalSession, WavepeekError>` that opens a new session when no session exists or when the requested idcodes are not a subset of the loaded set. When a superset is needed, close the old session before opening the replacement because the native Reader allows only one active signal list owner. Make the state update atomic from Rust's perspective: clear `signal_session` and `loaded_session_idcodes` before attempting the new open, and leave them as `None` plus an empty set if native creation fails; only install the new session and loaded set after success.

Switch `sample_resolved_optional`, `collect_change_times_with_mode`, `collect_expr_candidate_times_with_mode`, `sample_expr_value_uncached`, and `expr_event_occurred` to call session-backed native methods. Existing `expr_sample_cache` and `event_occurrence_cache` remain useful; keep them. Do not keep a session beyond the `FsdbBackend` lifetime. Do not add cross-command global caches.

Add targeted tests in `tests/fsdb_cli.rs` that exercise repeated sampling and candidate collection on generated fixtures. A useful pattern is one test that runs `value`, `change`, and `property` repeatedly in child processes against the generated FSDB fixtures and asserts successful JSON shape. Another useful pattern is a direct Rust integration path, if accessible, that makes one command sample the same signal at multiple times and verifies the output remains stable. The primary acceptance is no crash, no stderr from native Reader chatter, stable output, and the existing parity tests still passing.

Validation for this milestone:

    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Expected observations are: feature-enabled clippy passes; FSDB CLI tests still pass; repeated-use tests do not leak Reader messages into stdout/stderr; no test fails because an active session invalidated a one-shot operation.

### Milestone 3: Graceful error and resource hardening

At the end of this milestone, the FSDB feature build has explicit tests for common file failure modes and clearer resource cleanup behavior.

Add integration tests in `tests/fsdb_cli.rs` for missing FSDB-looking paths, empty or truncated FSDB-looking files, and clearly non-FSDB files with `.fsdb` extension. These tests should run only in the FSDB-gated suite. They should create temporary files under a test temp directory, never under `tests/fixtures`, and should assert: exit code is nonzero; stdout is empty; stderr begins with the repository's stable `error: <category>:` shape; stderr does not include raw Verdi banners or unrelated converter logs; and the process does not panic or abort. Match stable fragments such as `error: file:` and `FSDB Reader` only where the implementation actually routes through the Reader. If Wellen rejects a clearly non-FSDB file before FSDB open, accept the existing stable Wellen file error category instead of forcing FSDB wording.

Review native output suppression and error ownership. Every public native function must clear incoming error output, initialize outputs before work, free allocated native strings exactly once, and leave caller-owned output either null/empty or fully initialized on error. `wp_fsdb_free_samples`, `wp_fsdb_free_time_list`, `wp_fsdb_free_string`, and `wp_fsdb_free_error` must remain null-safe.

Verify unload/free paths by code structure and by process-level exercise. The native session destructor should free every traversal handle before unloading and resetting the signal list. It should tolerate partial construction failure by cleaning up handles and temporary allocations. The Rust `Drop` implementations for `FsdbReader`, `FsdbSignalSession`, `NativeSampleArray`, and `NativeTimeList` should be the only places that close or free their corresponding native resources. Add comments only where ownership would not be obvious to a future maintainer; do not wallpaper the code with apology notes.

If `valgrind` or a sanitizer-enabled local environment is available, run a focused smoke over `wavepeek value` and `wavepeek change` against a generated FSDB fixture and record the command and result in this plan's `Artifacts and Notes`. This is optional because the default devcontainer may not include those tools. The required validation remains the repeated subprocess tests and feature-enabled clippy/test gates.

Validation for this milestone:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Optional local memory probe if tools are available:

    WAVEPEEK_IN_CONTAINER=1 make build-release-fsdb
    VERDI_HOME="$(.devcontainer/resolve_verdi_home.sh)" valgrind --leak-check=full target/fsdb/release/wavepeek value --waves tests/fixtures/fsdb/change_property_core.fsdb --scope top --signals data --at 10ns --json

Expected observations are: the required FSDB test suite passes; failure-mode tests report ordinary CLI errors rather than crashes; optional memory tooling reports no definitely-lost allocations attributable to the project-owned session or sample/time-list objects.

### Milestone 4: Measurement and performance acceptance

At the end of this milestone, maintainers can run before/after FSDB benchmarks on the same files and see whether session reuse improved large-window behavior without changing command outputs.

Before changing performance-sensitive code, capture a baseline from the current implementation if this plan is being executed from an unoptimized tree. If session code is already implemented by the time a contributor reaches this step, capture the baseline from the parent commit or from a throwaway branch before the session commit. Keep raw benchmark runs under ignored directories unless explicitly asked to commit them.

Run the FSDB benchmark smoke first:

    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-smoke-commit

Then run a full FSDB benchmark capture when time permits:

    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-run

For functional parity against the existing FST baseline, run an FSDB benchmark into a temporary directory and compare only functional payloads:

    tmp_revised="$(mktemp -d)"
    WAVEPEEK_BIN=./target/fsdb/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests_fsdb.json --run-dir "$tmp_revised"
    python3 bench/e2e/perf.py compare --functional-only --revised "$tmp_revised" --golden bench/e2e/runs/baseline --verbose

This full functional-only compare must be strict: no timeout artifacts, no missing revised artifacts, no revised-only tests, and no golden-only tests. Filtered smoke runs may use `--allow-golden-extra` because the golden baseline intentionally contains tests outside the filter.

If a full FSDB baseline is refreshed locally, use:

    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-update-baseline

Do not commit generated benchmark run artifacts unless the task explicitly expands to maintaining a committed FSDB baseline. If committed, update `bench/e2e/runs/.gitignore` deliberately and mention the size impact in `Outcomes & Retrospective`.

Performance acceptance is deliberately comparative rather than magical. The FSDB run does not have to be faster than the FST baseline because the formats and libraries differ. It must produce matching functional `data` for matched benchmark names. Relative to the pre-session FSDB baseline, the session-backed implementation should improve or hold steady on repeated-sampling `change` and `property` cases, with no more than a 15% median regression on any existing FSDB benchmark unless the plan records a measured reason. If a benchmark regresses because conversion artifacts or Reader behavior differ, record the evidence and decide whether to fix, defer, or adjust the benchmark.

### Milestone 5: Documentation, gates, review, and handoff

At the end of this milestone, the implementation is documented, validated, reviewed, and ready to merge or continue from this plan.

Update `docs/DEVELOPMENT.md` in the optional FSDB and benchmark sections. Mention that `make prepare-fsdb-fixtures` now also converts every FST under the resolved RTL artifact directory into neighboring FSDB files, that `bench/e2e/tests_fsdb.json` is the FSDB benchmark catalog, and that FSDB benchmark targets require Verdi and use `target/fsdb`.

Update `CHANGELOG.md` under `## [Unreleased]` if the implementation changes maintainer-visible behavior, such as adding FSDB benchmark targets, fixture conversion, or improved FSDB command performance. Do not claim public end-user FSDB release support unless release policy has changed.

Run the default docs/code gates:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

Run the FSDB-specific gates on a Verdi-equipped machine:

    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb
    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-smoke-commit

Request focused read-only review lanes before final handoff: native/FFI resource safety, Rust backend correctness, benchmark/test infrastructure, and documentation/plan completeness. Apply fixes in the main session only. After fixes, rerun impacted gates and run one independent control review over the consolidated diff. Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` after each review loop.

## Concrete Steps

Start from the repository root:

    cd /workspaces/wavepeek-fsdb
    git status --short --branch

Expected observation is a clean branch or only changes intentionally made for this plan.

Confirm the default and FSDB development environment:

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env
    .devcontainer/resolve_verdi_home.sh
    command -v vcd2fsdb
    command -v fst2vcd

`make check-fsdb-env` exits successfully even when Verdi is absent, printing a skip-style diagnostic. The other commands are required only for executing FSDB implementation and benchmarks.

Implement Milestone 1 edits:

    $EDITOR scripts/prepare_fsdb_fixtures.sh
    $EDITOR .devcontainer/Dockerfile
    $EDITOR Makefile
    cp bench/e2e/tests.json bench/e2e/tests_fsdb.json
    python3 - <<'PY'
    import json
    from pathlib import Path
    path = Path('bench/e2e/tests_fsdb.json')
    data = json.loads(path.read_text())
    text = json.dumps(data, indent=2) + '\n'
    text = text.replace('.fst', '.fsdb')
    path.write_text(text)
    PY
    $EDITOR bench/e2e/perf.py
    $EDITOR bench/e2e/test_perf.py

The small Python snippet is only a starting point. Inspect the diff afterward; do not trust a blind extension replacement if future catalog fields contain prose that happens to mention `.fst`.

Implement Milestone 2 edits:

    $EDITOR native/fsdb/wavepeek_fsdb_shim.h
    $EDITOR native/fsdb/wavepeek_fsdb_shim.cpp
    $EDITOR src/waveform/fsdb_native.rs
    $EDITOR src/waveform/fsdb_backend.rs

Keep native declarations project-owned and minimal. Do not paste Verdi header declarations into repository files beyond the already-used type names and function calls needed by the shim.

Implement Milestone 3 tests and docs:

    $EDITOR tests/fsdb_cli.rs
    $EDITOR docs/DEVELOPMENT.md
    $EDITOR CHANGELOG.md

Run formatting and fast checks during iteration:

    cargo fmt
    WAVEPEEK_IN_CONTAINER=1 make test-aux
    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

Run benchmark and final gates when implementation is stable:

    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-smoke-commit
    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

At every stopping point, update this plan's `Progress` with completed and remaining work, add any unexpected behavior to `Surprises & Discoveries`, and add decisions to `Decision Log`.

## Validation and Acceptance

Acceptance is behavior-first.

Fixture preparation passes when:

    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures

creates or reports up-to-date FSDB files for all committed hand VCD fixtures and for every `*.fst` under `/opt/rtl-artifacts` or the resolved RTL artifact directory. A successful rerun should skip up-to-date outputs instead of doing expensive work again.

Benchmark catalog support passes when:

    WAVEPEEK_IN_CONTAINER=1 make test-aux
    python3 bench/e2e/perf.py list --tests bench/e2e/tests_fsdb.json

passes unit tests and lists the same benchmark names as `python3 bench/e2e/perf.py list --tests bench/e2e/tests.json`, with commands pointing at `.fsdb` files after artifact-root remapping is applied.

FSDB command correctness passes when:

    WAVEPEEK_IN_CONTAINER=1 make test-fsdb

passes all FSDB integration tests, including `info`, `scope`, `signal`, `value`, `change`, `property`, failure-mode tests, repeated-use tests, and generated fixture parity tests.

Performance acceptance passes when:

    WAVEPEEK_IN_CONTAINER=1 make bench-e2e-fsdb-smoke-commit

completes without functional mismatches, timeout artifacts, or missing revised artifacts, and a full local FSDB run, when performed, produces matching functional `data` against `bench/e2e/runs/baseline` under strict `compare --functional-only`. The implementation should improve or hold steady against a pre-session FSDB baseline on repeated-sampling `change` and `property` cases, allowing at most 15% median regression unless this plan records a measured exception.

Default repository health passes when:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

succeed without requiring Verdi for the default feature set.

Resource hardening passes when failure-mode tests return normal CLI errors instead of crashes, native Reader chatter does not appear in command stdout, repeated FSDB subprocess tests succeed, and optional memory tooling, if run, reports no project-owned definitely-lost leaks.

## Idempotence and Recovery

`make prepare-fsdb-fixtures` must be safe to rerun. It should skip generated outputs that are newer than their inputs, write through temporary files, and replace final outputs atomically with `mv`. If conversion fails, it should remove partial temporary outputs and leave the previous good `.fsdb` file in place. If the resolved RTL artifact directory is not writable, the script should fail before conversion with a clear message that names the directory and suggests setting `WAVEPEEK_RTL_ARTIFACTS_DIR` or `RTL_ARTIFACTS_DIR` to a writable artifact cache. The default devcontainer should avoid this failure by making `/opt/rtl-artifacts` writable for the `ubuntu` user.

Benchmark runs should be written to ignored temporary or run directories. If a benchmark is interrupted, rerun with `--run-dir <same-dir> --missing-only` where appropriate. Do not delete the committed FST baseline unless running the explicit existing `bench-e2e-update-baseline` target for that purpose.

Native session creation must be exception-safe. If any Reader call fails after some handles are opened, destructors must free those handles and reset/unload the signal list before returning an error. Rust must not assume a failed native call left partially valid output. The wrapper should free only outputs whose status and pointer invariants say they are owned by Rust.

If session-backed operations introduce instability, revert the backend to one-shot `FsdbReader` methods while keeping the benchmark and hardening infrastructure. Record the rollback in `Decision Log` and keep tests passing. A slower correct path is acceptable while investigating; a fast haunted path is not.

## Artifacts and Notes

Initial research evidence from plan authoring:

    docs/fsdb/arch.md says the hardening/performance work is to measure large FSDB files, optimize index caching within one command, verify memory unload/free paths, verify graceful errors for missing/truncated/non-FSDB files, and add richer datatypes only after fixture coverage exists.

    Current generated fixture script only scans tests/fixtures/hand/*.vcd and writes tests/fixtures/fsdb/*.fsdb.

    Current benchmark harness accepts --tests for list/run, but report uses the default tests.json and compare requires a timing threshold.

    Current FSDB backend caches metadata, hierarchy, expression samples, and event occurrence results, but native one-shot operations still load signal lists and create traversal handles inside each operation.

Record future implementation evidence here. Useful snippets include concise `make prepare-fsdb-fixtures` progress output, `test-aux` parser test output, `make test-fsdb` summary, FSDB benchmark smoke output, functional-only compare output, before/after benchmark deltas for representative `change` and `property` cases, and review outcomes. Do not paste proprietary Verdi documentation, header excerpts, or full benchmark dumps.

## Interfaces and Dependencies

The C ABI exposed by `native/fsdb/wavepeek_fsdb_shim.h` must include the existing reader, metadata, hierarchy, sample, candidate, event, and free functions, plus the new `wp_fsdb_signal_session` operations listed in Milestone 2. The C ABI owns native allocations until Rust calls the matching free or close function.

`src/waveform/fsdb_native.rs` must expose a safe Rust wrapper whose session cannot outlive the reader. One acceptable shape is:

    pub(super) struct FsdbReader {
        inner: Arc<FsdbReaderInner>,
    }

    struct FsdbReaderInner {
        raw: NonNull<ffi::wp_fsdb_reader>,
    }

    pub(super) struct FsdbSignalSession {
        raw: NonNull<ffi::wp_fsdb_signal_session>,
        owner: Arc<FsdbReaderInner>,
    }

    impl FsdbReader {
        pub(super) fn open_signal_session(&self, idcodes: &[u64]) -> Result<FsdbSignalSession, WavepeekError>;
    }

    impl FsdbSignalSession {
        pub(super) fn sample_signal_values(&self, idcodes: &[u64], query_time_raw: u64) -> Result<Vec<FsdbNativeSample>, WavepeekError>;
        pub(super) fn collect_signal_change_times(&self, idcodes: &[u64], from_raw: u64, to_raw: u64) -> Result<Vec<u64>, WavepeekError>;
        pub(super) fn signal_event_occurred(&self, idcode: u64, query_time_raw: u64) -> Result<bool, WavepeekError>;
    }

`FsdbSignalSession::drop` must close the native session before dropping `owner`; `FsdbReaderInner::drop` must close the native reader only after all session owners have gone away.

`src/waveform/fsdb_backend.rs` must keep session ownership inside `FsdbBackend`. A suitable shape is:

    signal_session: Option<FsdbSignalSession>,
    loaded_session_idcodes: HashSet<u64>,

    fn ensure_signal_session(&mut self, idcodes: &[u64]) -> Result<&FsdbSignalSession, WavepeekError>;

If the borrow checker makes returning `&FsdbSignalSession` awkward, use a helper that accepts a closure and performs the operation inside the borrow. Do not use unsafe Rust to dodge ordinary ownership; the C++ shim already provides enough sharp edges for one feature.

`bench/e2e/perf.py` must keep the existing public commands and add:

    python3 bench/e2e/perf.py report --tests bench/e2e/tests_fsdb.json --run-dir <dir>
    python3 bench/e2e/perf.py compare --functional-only --revised <dir> --golden <dir>
    python3 bench/e2e/perf.py compare --functional-only --allow-golden-extra --revised <filtered-dir> --golden <full-baseline-dir>

Existing invocations of `compare --max-negative-delta-pct <N>` must continue to work unchanged.

The root `Makefile` must provide these maintainer targets:

    build-release-fsdb
    prepare-fsdb-fixtures
    check-fsdb-rtl-artifacts
    bench-e2e-fsdb-update-baseline
    bench-e2e-fsdb-run
    bench-e2e-fsdb-smoke-commit

All FSDB Make targets must resolve `VERDI_HOME` through the existing `.devcontainer/resolve_verdi_home.sh` helper and use `CARGO_TARGET_DIR=target/fsdb` for feature-enabled Cargo builds. FSDB benchmark targets must also use the existing RTL artifact environment export so `bench/e2e/perf.py` remaps canonical `/opt/rtl-artifacts` catalog paths to the same resolved artifact directory that `prepare-fsdb-fixtures` populated.

Required external tools for FSDB work are the local Verdi FSDB Reader SDK, `vcd2fsdb`, and `fst2vcd`. Required external tool for benchmark timing is `hyperfine`, already used by the existing e2e benchmark harness. Default non-FSDB development must not require any of those Verdi tools.

## Revision Notes

- 2026-05-28 / Grin: Initial plan drafted from `docs/fsdb/arch.md`, current FSDB implementation files, Makefile targets, fixture scripts, benchmark harness, development docs, and the completed portable FSDB `change`/`property` plan. The plan intentionally uses descriptive FSDB names and avoids roadmap-label prefixes or suffixes in entities to be created.
- 2026-05-28 / Grin: Incorporated focused review findings. The plan now specifies full RTL artifact fallback resolution, default devcontainer artifact writability, benchmark path remapping, strict functional-only compare semantics, build-only release target behavior, complete catalog lockstep tests, `size_t` C ABI signatures, safe reader-owned session lifetimes, normal errors instead of native assertions, atomic Rust session cache failure state, bounded lazy traversal-handle caching, and the correct `value --at` memory-probe flag.
- 2026-05-28 / Grin: Incorporated replacement control review finding by changing the optional memory probe to use the generated `tests/fixtures/fsdb/change_property_core.fsdb` fixture.
- 2026-05-30 / Grin: Completed Milestone 1 implementation notes after adding RTL FSDB fixture conversion, FSDB benchmark catalog/targets, artifact-root remapping, `report --tests`, strict `compare --functional-only`, and the associated Python/catalog tests. This note records that the old container image remains root-owned at `/opt/rtl-artifacts`; the Dockerfile fix applies after rebuild, while current-session validation uses an override directory.
