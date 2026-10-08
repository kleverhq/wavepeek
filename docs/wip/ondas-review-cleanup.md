# Correct confirmed Ondas adapter regressions

This ExecPlan is maintained according to the `exec-plan` skill. Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective current.

## Purpose / Big Picture


`info` must return the same metadata and exit status with and without `DEBUG=1`. Protocol extraction with a global `--include` must remain practical on hierarchies with thousands of scopes. A scope is a named level of the waveform hierarchy; global include searches signals across all scopes rather than one `--scope`.

Prove both behaviors before changing production code. Add an FSDB CLI regression test and a successful, source-backed extraction benchmark. Preserve command output and deterministic signal ordering while removing the repeated full-hierarchy scans responsible for the measured slowdown.

## Non-Goals


Do not change Ondas 1.0.2, expression semantics, trace preload policies, scope-only caching, SDK telemetry, or panic recovery. These review suggestions have no confirmed defect in the supported CLI. Limit local cleanup to redundant internal options, duplicate declaration lookup, a used parameter name, and incorrect benchmark-tool diagnostic labels.

## Progress


- [x] (2026-10-08) Inspect repository instructions, OCR findings, fixture preparation, and benchmark workflow.
- [x] (2026-10-08 06:07Z) Add and run failing FSDB info and benchmark-generator diagnostic tests.
- [x] (2026-10-08 06:12Z) Add a source-backed large hierarchy and global AXI include benchmark; capture pre-fix FST and FSDB slowdown.
- [x] (2026-10-08 07:14Z) Correct info and hierarchy listing; comparisons against the pre-fix branch pass in all formats and against main in FST/FSDB. Record the remaining Ondas VCD opening cost.
- [x] (2026-10-08 06:15Z) Simplify confirmed redundant internal code; waveform, CLI, and FSDB tests pass before and after cleanup.
- [x] (2026-10-08 07:18Z) Run quality gates, review the final diff, and prepare the completed changes for host commit with normal hooks. Git history records delivery.

## Surprises & Discoveries


The existing RTL benchmarks have 101–190 scopes. The saved diagnostic FSDB has 25,435 scopes without a depth bound (the earlier default-depth listing counted 15,834): baseline `main` completed global AXI include search in 5.808 seconds, while `c282036` exceeded 120 seconds. Existing protocol benchmarks specify `--scope` and do not cover global include. The saved command intentionally failed mapping, so the permanent benchmark must instead complete successfully.

The first grouped-declaration implementation preserves output but still fails timing: FST median 0.138135 seconds versus baseline 0.088485; FSDB median 0.238510 versus 0.138419. A direct FST comparison of the same event shows global discovery at 0.089869 seconds and scoped discovery at 0.071001 seconds. Index construction needs profiling before choosing any further correction. Temporary release-test instrumentation is kept out of the final source.

FSDB metadata reading intentionally skips scope-path collision validation. Full waveform opening rejects colliding canonical scope paths. `src/engine/info.rs` chooses between those paths based on DEBUG, causing exit 0 without DEBUG and exit 2 with DEBUG for one file.

Generated VCD/FST fixtures live under `tests/fixtures/generated/`; their FSDB derivatives live under `tests/fixtures/fsdb/`. Benchmark catalog generation must retain these existing locations when selecting a format.

## Decision Log


On 2026-10-08, choose one source-backed hierarchy fixture using the existing Icarus Verilog generator, with thousands of scopes and a small AXI interface. This keeps benchmarks reproducible without committing binary dumps or provisioning another external waveform.

On 2026-10-08, keep info on its metadata-only path regardless of DEBUG. Debug telemetry may retain event names without opening the hierarchy merely to report backend details.

On 2026-10-08, retain one lazy `HierarchyIndex` and add scope lookup and declaration grouping there. A declaration is a signal's hierarchy entry; grouping its position by owning scope removes repeated scans without introducing another cache or changing waveform reads.

On 2026-10-08 06:28Z, use one borrowed-entry scan for global include matching and clone only matched entries. This avoids constructing both scope-only and full indexes and avoids materializing every signal listing just to reject most entries. Release profiling measured full-index variable processing at 38–43ms and scope-only rebuilding at 9–12ms on the new fixture. Copy component lists only when a variable creates synthetic scopes. These corrections target the still-failing benchmark; introduce no additional cache, preload, or dependency.

On 2026-10-08 06:38Z, discard the direct-child constructor variant: Ondas `children_of` itself filters all scopes and variables, reintroducing repeated scans. Its FST median rose to 0.413452 seconds. Retain one flat variable pass and use borrowed `Cow<str>` components during path construction; allocate a component only when escaping or slash normalization changes it. Component keys exist only during construction, and synthetic components own their strings.

On 2026-10-08 07:00Z, limit metadata construction to canonical paths selected by global include. The full-index matcher still measured FST 0.138412 seconds versus baseline 0.087522. Debug phases placed about 40ms between opening and metadata completion, with sampling below 1ms. Existing scope/name order, split width, and FSDB cross-scope ambiguity tests pass before this refinement; retain them afterward. This addresses the measured discovery defect without trace-loading changes or a new persistent cache.

On 2026-10-08 07:12Z, retain only selected owning scopes in the temporary index and reuse the existing simple packed-signal resolution path for VCD as well as FST. VCD phase measurements exposed a second full index during binding; sharing that lookup removed it. Split vectors, ambiguous paths, and complex names still fall back to the full index. One test incorrectly passed a signal handle between independently opened waveforms; use one shared waveform for both hosts.

On 2026-10-08 07:12Z, stop performance changes at the adapter boundary. Ten warmed VCD phase samples measured Ondas opening at 50.821ms versus the old backend at 20.831ms, while adapter discovery improved to 11.306ms versus 28.686ms. VCD median improved from pre-fix 0.388149s to 0.138040s, but still fails the main comparison (0.088155s). Preserve this failed comparison. Require green comparisons against the pre-fix branch in every format and against main in FST/FSDB; report the VCD reader difference explicitly rather than alter Ondas or the benchmark.

## Outcomes & Retrospective


The red CLI test reproduces exit 2 only with DEBUG. Catalog diagnostic tests fail on all four VCD error paths. The permanent benchmark emits the same AXI address event in baseline and pre-fix binaries. FST medians are 0.087960 seconds for baseline and 0.289045 seconds before the fix; FSDB medians are 0.138560 and 0.388932 seconds. Both timing comparisons fail at the standard threshold. Green verification and quality gates remain.

The new info test passes after selecting metadata-only reads for both diagnostic settings. Auxiliary suites and the 68 waveform tests pass before indexed-access cleanup. VCD also fails the new timing comparison: medians 0.088316 seconds for baseline and 0.388149 seconds before the fix.

Final captures preserve functional parity and pass the standard 5%/5ms comparison against `c282036` in VCD, FST, and FSDB. Revised medians are FST 0.087469s, FSDB 0.137653s, and VCD 0.138040s. Pre-fix medians are 0.289045s, 0.388932s, and 0.388149s respectively. FST/FSDB also pass against main. The VCD main comparison remains failed for the reader-phase reason recorded above.

The original no-match FSDB query now completes with identical output: revised median 7.074924s versus main 5.866756s, replacing the pre-fix timeout above 120s. An unlimited-depth scope listing matches main exactly for all 25,435 entries.

`just ci` and `just check` pass, including FSDB, auxiliary suites, docs, and native/browser Playground parity. Coverage is regions 91.86%, functions 91.66%, and lines 93.01%, above the 90% gate. Final diff review found no additional actionable defect. The code and validation milestones are complete; normal commit hooks and host push deliver this branch.

## Context and Orientation


The worktree is `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/feat-ondas`, branch `feat/ondas`, initially at `c282036`. Baseline `main` is `883f96d`. `src/waveform/ondas_backend.rs` owns format adaptation and the lazy hierarchy index. `signals_in_scope_recursive_report` currently finds a scope linearly, searches its subtree end, and scans every declaration even for a direct-scope listing. Protocol engines call direct listings once per scope for a global include.

`src/engine/info.rs` formats metadata. `src/waveform/mod.rs` forwards adapter methods and retains obsolete outer `Option` values around indexed access; `src/engine/change.rs` checks those always-present values. Preserve the inner optional offset and optional signal bits, which represent genuinely missing values.

`tests/fsdb_cli.rs` runs the CLI and uses Verdi's `vcd2fsdb` for special fixtures. `tests/fixtures/waveform_policy.json` declares source-backed fixtures. `tools/waveform/prepare_fixtures.py` generates ignored VCD/FST dumps from Verilog sources. `bench/e2e/tests.json` is the FST catalog; `tools/fsdb/generate_bench_catalog.py` derives VCD and FSDB catalogs. `tools/bench/capture.py` runs release comparisons using current benchmark tooling. Generated runs and logs belong under a new `tmp/confirmed-ondas-regressions/` directory.

Run all Cargo, waveform tools, and repository quality commands through root `./dev` and `just`. This worktree container requires host `VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06`; its SDK is mounted at `/opt/verdi`. Git and hooks installation run on the host. Do not bypass hooks. Read applicable AGENTS.md before edits.

## Plan of Work


### Milestone 1: Establish failing behavior


In `tests/fsdb_cli.rs`, create a temporary VCD with escaped scopes `\\top/a` and `\\top.a`, convert it to FSDB, and invoke info with DEBUG removed and with DEBUG=1. Both invocations must exit 0 with identical metadata. Check debug stderr consists of JSON events. Also show scope still rejects the ambiguous hierarchy. Run this test before touching `src/engine/info.rs`; its DEBUG invocation must fail with exit 2.

In `tools/fsdb/test_generate_bench_catalog.py`, cover VCD-target failures for missing source, malformed JSON, missing FST suffix, and missing check output. Assert the diagnostic names VCD. Run before changing labels; current hardcoded FSDB labels must fail.

### Milestone 2: Add a performance regression benchmark


Add `tests/fixtures/source/extract_global_include.v` with 2,048 generated child instances, each containing multiple independent declarations, plus a small top-level AXI interface. Generate one accepted address event. Declare generated VCD/FST outputs in `tests/fixtures/waveform_policy.json`. Use existing FSDB fixture conversion for its FSDB derivative.

Add `extract_large_hierarchy_axi_global_include` to `bench/e2e/tests.json`, using `--include`, no `--scope`, bounded output, and JSON. Use at least ten measured runs and five warmups. Derive VCD/FSDB catalogs with root recipes. Update catalog generation to route generated FSDB fixtures to their existing `tests/fixtures/fsdb/` directory, with a tool test for this mapping. Ensure capture prepares source-backed fixtures before suites, including FSDB conversion.

Preserve copies of initial release binaries before edits. Run the new benchmark on baseline `883f96d` and pre-fix `c282036` in fresh artifact directories. Compare matching-format functional results and timing; require a timing failure at the standard 5%/5ms threshold. Adjust only fixture scale if it does not expose the known repeated-scan cost reliably. Record measured times.

### Milestone 3: Correct the two regressions


Make info always call `Waveform::read_metadata` and emit diagnostics without full waveform opening. In `HierarchyIndex`, map scope paths to scope positions and group declaration positions by owning scope. Direct listings must touch only that scope's declarations; recursive listings must preserve depth bounds, ambiguity omissions, and depth-first ordering. Keep full indexing lazy and retain split-vector and synthetic-scope behavior.

Global include collection in `src/engine/{axi,apb,ahb,atb,axistream}.rs` uses `Waveform::matching_signals`, a predicate over canonical path and leaf name that returns only matched, unambiguous entries in the existing scope/name order. Reuse an existing full index when available. Otherwise scan normalized names once to select canonical paths, then build a temporary index retaining every declaration at those paths. Retain selected owning scopes and their complete component keys for the existing ordering. Retaining all declarations at a selected path preserves ambiguity omissions and split-vector merging even when only one leaf spelling matches. An empty selection needs no index; the temporary index is not cached. Its constructor retains the flat declaration traversal and existing visibility/packed-scope handling. Scope components borrow unchanged names from the immutable hierarchy during construction; public entries still own their strings.

Use the existing direct packed-signal lookup for VCD and FST to avoid rebuilding full metadata during binding of simple selected signals. Preserve fallback behavior for split vectors, canonical collisions, and complex names. Run the failing info test again and all hierarchy tests. Rebuild release binaries and run the new benchmark on baseline and revised code in all three formats. Require identical functional results and no timing regression against the pre-fix branch in every format. Require no timing regression against main for FST and FSDB. Retain and report the VCD main comparison failure, attributed by warmed phase measurements to Ondas opening; changing the dependency is outside this plan. Compare revised with pre-fix captures to establish the correction. Re-run the saved 25,435-scope FSDB command to show it completes rather than timing out.

### Milestone 4: Local cleanup and handoff


Parameterize the catalog tool's error labels by its existing target. Remove only the always-Some outer options from indexed offset/decode methods and their callers; retain checks for a missing time table. Rename `_path` to `path`. Pass an already-resolved declaration into value validation rather than searching it twice. Existing value/error and change-equivalence tests establish unchanged behavior; do not add tests that merely mirror these refactors.

Update `CHANGELOG.md`, benchmark documentation, and packaged DEBUG guidance for current behavior. Run `just ci` and `just check`, inspect the final diff for scope and ordering, and commit logical completed milestones using host Git with normal hooks. Record all results and remaining limitations in this plan.

### Concrete Steps


All commands below start at the worktree root. Prefix container commands with the VERDI_HOME assignment shown here:

    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev --install-hooks
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just prepare-waveform-fixtures
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just --command env CARGO_TARGET_DIR=target/fsdb cargo test --features fsdb --test fsdb_cli fsdb_info_debug_preserves_metadata_only_result -- --exact
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just --command python3 -B -m unittest tools/fsdb/test_generate_bench_catalog.py
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just update-bench-e2e-fsdb-catalog update-bench-e2e-vcd-catalog
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just build-release build-release-fsdb

For each format, run `bench/e2e/perf.py run` through `./dev just --command python3 -B`, select the corresponding catalog with `--tests`, filter `^extract_large_hierarchy_axi_global_include$`, and pass explicit `--binary baseline=...` and `--binary revised=...` paths. Existing baseline binaries are under `tmp/backend-regressions.PJqhRt1m/main-source/target/{release,fsdb/release}/wavepeek`; rebuild from `883f96d` if absent. Use unique `--run-dir` values under `tmp/confirmed-ondas-regressions/`. Compare the two labeled directories with `perf.py compare --max-negative-delta-pct 5 --max-negative-delta-seconds 0.005 --result-json ...`. Save stdout/stderr logs there.

Finish with:

    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just ci
    VERDI_HOME=/home/esynr3z/tools/synopsys/verdi/X-2025.06 ./dev just check
    git diff --check

### Validation and Acceptance


The new FSDB info test must fail before the fix and pass afterward, with identical stdout regardless of DEBUG and a retained scope collision error. VCD-target diagnostic tests must fail before their label fix and pass afterward. The global include benchmark must return an accepted AXI event, preserve payload parity with baseline, expose a pre-fix timing failure, and pass comparisons against the pre-fix branch after correction. FST/FSDB must also pass against main; preserve the VCD main timing failure with reader-phase evidence. Existing direct/recursive/depth-bounded/split/ambiguous hierarchy cases and change/value tests must pass. Both full quality gates and commit hooks must pass.

### Idempotence and Recovery


Fixture and catalog recipes are repeatable and own only their generated outputs. Preserve unrelated files in tmp. Use fresh benchmark directories; do not overwrite captured red evidence. Changes affect local source and ignored dumps only. Fix failed gates and retry normal commits without bypassing hooks.

### Artifacts and Notes


Prior review evidence is `tmp/ocr-ondas.mTQvHKME/validated.md` and `kiss-yagni.md`. This plan embeds the actionable findings and does not require those ignored reports. Populate new red/green logs and timing evidence as milestones complete.

Red evidence lives under `tmp/confirmed-ondas-regressions/`: `info-red.log`, `catalog-red.log`, `red-fst-compare.json`, and `red-fsdb-compare.json`. Both benchmark comparisons preserve functional payload parity; each binary has five warmups and ten measured samples.

### Interfaces and Dependencies


Keep Ondas pinned to 1.0.2. Keep public CLI and result schemas unchanged. Indexed offset forwarding returns `Option<SignalOffsetData>`; indexed decode forwarding returns `Result<SampledSignalState, WavepeekError>`. The optional offset still denotes absent recorded data. Reuse the existing `HierarchyIndex`, Icarus, GTKWave converters, Verdi converter, stdlib Python harness, and hyperfine; introduce no dependency.

Revision 2026-10-08: initial plan records the confirmed defects, red-first validation, permanent performance coverage, and excluded speculative changes.

Revision 2026-10-08 06:12Z: record red evidence and fixture routing; the permanent benchmark confirms the slowdown on successful extraction rather than a failing query.

Revision 2026-10-08 06:15Z: record passing info/tool tests and VCD red evidence; simplify existing indexed-access tests to the single-backend signatures.

Revision 2026-10-08 06:25Z: retain failed intermediate timing evidence and profile remaining index cost before extending the correction.

Revision 2026-10-08 06:28Z: record measured constructor costs and refine global matching to reuse the full index and avoid rejected-entry copies; scope traversal remains uncached.

Revision 2026-10-08 06:38Z: discard the child-iterator experiment after benchmark failure and preserve flat traversal; document temporary borrowed component ownership.

Revision 2026-10-08 07:00Z: narrow global metadata construction to selected canonical paths after profiling the still-failing benchmark; preserve all declarations for ambiguity and split-vector handling.

Revision 2026-10-08 07:12Z: remove the measured second VCD index and restrict temporary scope metadata; record the remaining Ondas VCD opening cost and limit acceptance to confirmed adapter work.

Revision 2026-10-08 07:18Z: record final benchmark comparisons, full hierarchy parity, passing gates, and coverage. Correct the saved FSDB scope count to the unlimited-depth total.
