# Avoid a second full waveform index for targeted queries

This ExecPlan is a living document maintained according to the `exec-plan` skill.

## Purpose / Big Picture

Wavepeek reads VCD, FST, and FSDB through Ondas. For some queries, Wavepeek currently traverses the whole Ondas hierarchy again to build `HierarchyIndex`, even when the user requests one signal. A targeted query should return the same path, value, ambiguity error, and CLI JSON without constructing a second index for unrelated declarations. Observe the effect with the unchanged Chipyard FST/FSDB `value` scenarios in `bench/e2e/tests.json` and `bench/e2e/tests_fsdb.json`.

## Non-Goals

Do not modify Ondas or binary fixtures, alter benchmark catalogs or the `max(5%, 5ms)` gate, change public spelling/diagnostics, or claim that local indexing explains Ondas's independently measured FST bulk-sampling cost (#28). Do not weaken VCD, FST, FSDB, or browser tests.

## Progress

- [x] (2026-09-23 19:51Z) Confirm that current `9e044fe` builds an additional full adapter index and that FSDB Chipyard `value` spends about 240 ms selecting one signal after Ondas opens the file. Direct Ondas scanning of 101,329 FSDB variables for raw-name candidates takes 21–28 ms and finds 2,917 names containing `clock`.
- [x] (2026-09-23 19:53Z) Add a failing FSDB regression proving that one exact converted signal currently initializes the second index; strengthen the alias-collision test to resolve before any listing builds that index.
- [x] (2026-09-23 19:56Z) Extract the full builder's variable spelling, packed-range, and synthetic-scope normalization into `with_public_variable`; targeted FSDB collision and FST exact-path tests still pass. The new exact FSDB no-second-index test is deliberately red until the targeted path is implemented.
- [x] (2026-09-23 20:06Z) Select only public-path matches for one FSDB signal and prepare up to 128 requested FSDB values in one hierarchy pass; preserve ambiguous aliases by falling back to the existing full-index error path. Tests cover sampling before/after full indexing and collision both before listing and after batch preparation.
- [x] (2026-09-23 20:05Z) Focused FSDB Chipyard `value` on 1/10/100 signals passes the unchanged 5%/5ms comparison with equal functional JSON (`tmp/ondas-migration/fsdb-batch-focused-run/`).
- [x] (2026-09-23 20:17Z) `./dev just ci` passes, including browser smoke and 23 FSDB CLI tests. FST functional catalog remains 155/155 equal; the final FSDB catalog is 143 equal, 0 different, and the same 12 historical failures. Focused FSDB `value` on 1/10/100 Chipyard signals passes the unchanged threshold; focused FSDB `change` improves but remains slower with equal JSON.
- [x] (2026-09-23 20:37Z) Independent Sol/high FSDB and ordinary FST batch reviews found no substantive correctness defects; their focused tests passed. A later batch extension handles FST arrays and 1,000 selectors; review that final delta before the gate.
- [x] (2026-09-23 20:32Z) Pin upstream `646a536c8ee2793e40c656b43894be8ff4219803` (Ondas #28). Direct FST 70-handle sampling improves ~452→350 ms, but pin-only CLI still fails at ~541 ms versus ~490 ms baseline. One-pass FST selection cuts CLI to ~439 ms and passes the unchanged focused compare with equal JSON. Fresh catalogs match 155 FST and 143 comparable FSDB cases; the same 12 FSDB captures remain unavailable. `./dev just ci` passes. Ondas #28 remains open pending consumer gate verification.
- [x] (2026-09-23 20:34Z) With Ondas #28 pinned and both adapters updated, `./dev just ci`, `./dev just check`, and the 155 FST/143 comparable FSDB functional catalogs pass; focused FST and FSDB Chipyard `value` cases pass the unchanged comparison.
- [x] (2026-09-23 20:47Z) Batch-select FST public array elements through shared normalization, allowing the 1,000-path Chipyard case to bypass the second index. The focused CLI improves ~1043→842 ms versus ~792 ms baseline, but its median exceeds the unchanged limit; JSON matches.
- [x] (2026-09-23 20:56Z) The final FST Sol/high review found a genuine array alias collision: `top.memory[0]` and `top.memory.[0]` can share a public path. A generated FST regression failed before the fix and passes after scanning all plausible array leaves through shared normalization. `just ci` passes with 680 FSDB-enabled library tests and 23 FSDB CLI tests. Focused 100-selector FST comparison passes; the 1,000-selector median remains over threshold and must be judged by the unchanged full gate, including its best-sample confirmation.
- [ ] Commit with hooks and run the unmodified clean-ref `./dev just bench-gate v3.0.1 HEAD always` once after both changes; report failures by origin.

## Surprises & Discoveries

- The old Wellen path used Wellen's hierarchy directly, while the retired FSDB reader built one lazy index; the Ondas adapter can build a second full index after Ondas already materializes its hierarchy.
- `src/waveform/ondas_backend.rs` already avoids declaration construction for scope-only requests and caches ordinary FST direct handles. FST queries with complex names or more than 128 direct handles still use the full Wavepeek index.
- The full clean-ref gate on `9e044fe` has 27 FST and 120 FSDB best-sample timing failures, zero functional mismatches across 584 comparable tests. Direct Ondas FST bulk sampling of 70 unique Chipyard handles takes about 452 ms; it is separate from this adapter work.
- Repeating the 101,329-variable FSDB scan separately for 100 requested paths raised the Chipyard CLI to ~840 ms; preparing those paths in one pass brought it to ~438 ms, matching a ~439 ms baseline. The requested final name is a substring of the original reader spelling under the existing FSDB public-name transformations; every candidate still receives the shared full normalization before comparison.
- On Ondas #28, the FST Chipyard 100-selector CLI still spent ~130 ms selecting signals after opening, versus <1 ms in Wellen. Preparing ordinary FST paths in one variable pass reduces selection to ~6 ms and the full CLI from ~541 to ~439 ms. For 1,000 selectors including four unpacked array elements, the full-index fallback cost ~207 ms; one-pass structural array normalization reduces selection to ~25 ms, but the median complete CLI remains ~50 ms slower than baseline. An adversarial review exposed a second raw declaration with the same public array path; batch selection now detects this ambiguity and defers to the full-index error. Packed fragments and unsupported spellings retain the full-index fallback.

## Decision Log

- Decision: Keep `HierarchyIndex` as the fallback for full recursive listings and unsupported exact paths; do not replace the public contracts with Ondas's normalized names. Rationale: FSDB raw spellings create synthetic scopes and collisions; FST packed fragments can form one public vector. Date/Author: 2026-09-23, Wavepeek agent.
- Decision: Filter targeted declarations by raw-name candidates but compare their fully normalized public paths before selecting. Rationale: matching an Ondas SDK path alone can silently miss other declarations that alias the same Wavepeek path. Date/Author: 2026-09-23, Wavepeek agent.
- Decision: Prepare multiple `value` and `change` paths in one pass and retain disjoint direct IDs; build the old full index for selections above 128 paths or any unsupported/ambiguous path. Rationale: one scan per path regressed the existing 100-signal case, while one shared scan passes the unchanged focused `value` comparison. Date/Author: 2026-09-23, Wavepeek agent.
- Decision: Prepare up to 1,024 FST paths in one pass, including structurally normalized unpacked array elements, while retaining packed fragments and escaped paths on the full-index fallback. Rationale: Ondas #28 alone leaves the original 100-selector CLI over threshold, and the 1,000-selector case still built a redundant ~207 ms adapter index for four array elements. Date/Author: 2026-09-23, Wavepeek agent.
- Decision: Defer the next full gate until the Ondas #28 fix is pinned and its original FST workload passes focused verification. Rationale: the user asked for one full gate after both integrations; running it on the older pin cannot establish acceptance. Date/Author: 2026-09-23, Wavepeek agent.

## Outcomes & Retrospective

The FSDB targeted path eliminates the duplicate adapter index for ordinary exact `value` selections; the FST multi-signal path avoids repeated hierarchy scans. Targeted FSDB/FST regressions, full functional catalogs, and `just ci` pass with Ondas #28 pinned. End-to-end acceptance remains pending adversarial review and the full gate.

## Context and Orientation

`src/waveform/mod.rs` is the stable facade used by CLI engines. `src/waveform/ondas_backend.rs` owns Ondas opening, hierarchy adaptation, signal resolution, and sampling. `HierarchyIndex::new` currently converts every Ondas declaration to Wavepeek paths and IDs, including FSDB escaped names, dotted names, arrays, and FST split packed vectors. `OndasBackend::declaration` initializes that full index. `OndasBackend::direct_fst_signal` handles a safe subset without it. Tests in `src/waveform/ondas_tests.rs` and `src/waveform/ondas_fsdb_tests.rs` cover these public conversions; `tests/fsdb_cli.rs` exercises the CLI. A stable signal handle must continue to resolve to the same signal if another operation subsequently initializes the full index.

## Open Questions

The Sol/high review found no FSDB raw-name filtering counterexample; generated escaped-name, array, enum, and collision fixtures pass before any full index. Focused measurements show that one pass across 100 targets beats repeated per-path scans on Chipyard FSDB/FST. The full gate is still required for all workload sizes.

## Plan of Work

The shared `with_public_variable` conversion now serves both `HierarchyIndex::new` and FSDB targeted lookup in `src/waveform/ondas_backend.rs`. Keep the full index for hierarchy-wide listings and unsupported paths. Targeted FSDB lookups scan original reader names for plausible candidates, compare fully normalized public paths, and defer collisions to the unchanged full-index error path. `src/engine/value.rs` and `src/engine/change.rs` prepare multiple paths in one pass for both FSDB and FST; IDs stay disjoint from the full index. Preserve these contracts during review. Ondas #28 is pinned and its original Chipyard FST bulk sampling has been verified directly and in the focused CLI. Run the full gate only after review and commit.

### Concrete Steps

Run commands in `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/feat-ondas` through `./dev` for Cargo, fixtures, and benchmarks. Start with `./dev just test-fsdb` for feature-enabled library and CLI tests and `./dev just ci` for coverage and browser smoke. For focused comparisons, run `./dev python3 bench/e2e/perf.py run --help` and use the existing `value_chipyard_clusteredrocketconfig_dhrystone_signals_1` and `_signals_100` tests from both catalogs, saving scratch results under `tmp/ondas-migration/`; run `perf.py compare` with the unchanged 5% and 0.005-second limits. After a hook-validated commit run `./dev just check`, then `./dev just bench-gate v3.0.1 HEAD always`. The gate takes over an hour: preserve its output and inspect its final `summary.md`, both per-format comparison JSON files, and best-sample confirmation files.

### Validation and Acceptance

The new FSDB exact-path test should show that selecting and sampling a simple converted FSDB signal leaves `backend.index` empty. Existing collision, hidden-scope, synthetic-array, split-vector, VCD/FST parity, and browser tests must still pass. Focused CLI JSON must match baseline byte-for-byte. The clean-ref gate must be reported as passed only if both functional and timing comparisons pass; otherwise report the exact remaining counts and verified root attribution rather than waiving thresholds.

### Idempotence and Recovery

Scratch outputs have unique names under `tmp/ondas-migration/` and must not overwrite or delete other scratch artifacts. The Ondas checkout must remain unmodified. Keep all edits on the Wavepeek branch, commit only after `just ci`, and never bypass hooks. Rerun a failed focused test before rerunning the hour-long full gate.

### Artifacts and Notes

Baseline gate artifacts are in `tmp/bench-gate/gates/20260923T155421Z-883f96de05e3..9e044fe29dba/`. A direct current-pin FST probe and a small FSDB candidate scan are retained under `tmp/ondas-migration/`. No new dependency is required.

### Interfaces and Dependencies

Reuse Ondas `Hierarchy::variables`, `Variable::reader_name`, `Variable::parent`, and `Variable::signal` from the pinned Git SHA in `Cargo.toml`. Use the existing `DirectSignal` and `SignalId` mapping in `src/waveform/ondas_backend.rs` rather than adding a second general-purpose index library. Keep the public methods on `src/waveform/mod.rs` unchanged.
