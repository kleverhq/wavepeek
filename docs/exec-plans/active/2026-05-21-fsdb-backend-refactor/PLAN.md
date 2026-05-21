# Refactor waveform backend boundary for FSDB support

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill. All new repository entities named by this plan intentionally use descriptive backend-refactor names rather than milestone labels; do not add milestone-label prefixes or suffixes to new files, modules, structs, functions, tests, make targets, benchmark run directories, or documentation anchors.

## Purpose / Big Picture

After this change, the waveform layer has a real backend boundary: the engine can ask for metadata, scopes, signals, sampled values, expression values, event occurrences, and candidate timestamps without knowing that VCD/FST currently come from the `wellen` crate. The user-visible behavior should not change at all. A user can run `wavepeek info`, `scope`, `signal`, `value`, `change`, and `property` on the existing VCD/FST fixtures and see byte-for-byte compatible JSON `data` output, while the code no longer leaks `wellen::SignalRef` into engine modules.

This matters because FSDB is planned as an optional native read backend. Trying to bolt FSDB onto the current all-in-one `src/waveform/mod.rs` would spread conditional code through the engine like glitter in a machine room. This refactor first moves the current Wellen implementation behind the same `Waveform` facade and introduces backend-neutral `SignalId` values, so later FSDB work can add a backend without rewriting command semantics.

## Non-Goals

This plan does not make `.fsdb` files open through the CLI. The clear FSDB-disabled user error and FSDB runtime format detection are separate work.

This plan does not implement an FSDB hierarchy reader, value sampler, candidate collector, or expression host. The existing feature-gated native build-smoke module `src/waveform/fsdb_native.rs` remains private and disconnected from `Waveform::open`.

This plan does not add FSDB-specific CLI flags, JSON schema fields, error categories, public command behavior, or public documentation promises. The schema artifact `schema/wavepeek.json` should remain unchanged.

This plan does not optimize the `change` engine beyond preserving the existing VCD/FST fast paths. Performance work here means proving the refactor did not regress hot paths; it is not a mandate to make them faster by accident. Accidental speedups are welcome, naturally, but do not sell them as architecture.

This plan does not create any repository entity whose name contains a milestone label. Use descriptive names like `SignalId`, `WellenBackend`, `backend-refactor-before`, and `backend-refactor-after` instead of milestone-branded names.

## Progress

- [x] (2026-05-21 11:23Z) Read `docs/fsdb/arch.md`, `docs/exec-plans/AGENTS.md`, the completed FSDB build-spike plan, `src/waveform/mod.rs`, `src/waveform/expr_host.rs`, `src/engine/change.rs`, `src/engine/property.rs`, `src/engine/expr_runtime.rs`, `Cargo.toml`, `Makefile`, `docs/DEVELOPMENT.md`, `bench/AGENTS.md`, `bench/expr/AGENTS.md`, and `tests/AGENTS.md`.
- [x] (2026-05-21 11:35Z) Drafted this active ExecPlan under `docs/exec-plans/active/2026-05-21-fsdb-backend-refactor/PLAN.md` with descriptive entity names and no milestone-label entity names.
- [x] (2026-05-21 11:52Z) Ran focused plan review lanes for exec-plan completeness, backend architecture/code feasibility, and validation/performance/coverage risk.
- [x] (2026-05-21 12:05Z) Applied review fixes: merged non-compiling extraction/engine milestones, made indexed fast paths an optional capability, tightened `SignalId` visibility and Wellen index conversion, preserved FST stream path re-resolution, switched blocking CLI benchmarks to the full catalog, made FSDB smoke conditional, and fixed scratch-directory setup.
- [x] (2026-05-21 12:24Z) Ran one final independent control review on the revised plan.
- [x] (2026-05-21 12:28Z) Applied the control-review fix by using the existing `ChangeCandidateCollectionMode::Stream` and `ChangeCandidateCollectionMode::Random` variants for the forced streaming regression test guidance.
- [x] (2026-05-21 12:31Z) Prepared the finalized plan for a conventional docs commit.
- [ ] Implementation not started. The first implementation step is to capture local coverage and performance baselines before editing Rust code.

## Surprises & Discoveries

- Observation: the existing `Waveform` facade is still directly a Wellen object.
  Evidence: `src/waveform/mod.rs` currently stores `simple::Waveform`, `wellen::FileFormat`, and `HashSet<wellen::SignalRef>` directly inside `pub struct Waveform`.

- Observation: engine code depends on Wellen-native signal handles today.
  Evidence: `src/engine/change.rs` imports waveform `ResolvedSignal` and currently keys `IndexDecodeCache` by `(wellen::SignalRef, u32)`. `src/engine/expr_runtime.rs`, `src/engine/property.rs`, and `src/engine/change.rs` deduplicate expression sources through `signal.signal_ref`.

- Observation: the repository already has explicit performance and coverage harnesses that this refactor can use as guardrails.
  Evidence: `Makefile` defines `coverage-src`, `bench-e2e-smoke-commit`, `bench-e2e-run`, and `bench-expr-run`; `docs/DEVELOPMENT.md` documents `bench/e2e/perf.py compare` and `bench/expr/perf.py compare`.

- Observation: the existing build spike has already added `fsdb` as a non-default Cargo feature plus a private native smoke module, but runtime waveform opening is still Wellen-only.
  Evidence: `Cargo.toml` has `[features] fsdb = []`, `build.rs` compiles the native shim only when `CARGO_FEATURE_FSDB` is present, and `src/waveform/mod.rs` only declares `#[cfg(feature = "fsdb")] mod fsdb_native;` without using it in `Waveform::open`.

- Observation: making `timestamps_raw_slice` an unconditional facade method would preserve a Wellen-shaped global time table requirement in the supposedly backend-neutral seam.
  Evidence: architecture review flagged that FSDB would otherwise have to fake indexed Wellen state. The plan now uses optional indexed capability methods for optimized VCD/FST paths and keeps portable scheduling on `previous_sample_time`.

- Observation: the commit-sized CLI benchmark catalog is not enough for this refactor because it has no `property` scenarios.
  Evidence: validation review checked `bench/e2e/tests_commit.json` and found `property count 0`. The blocking before/after CLI performance comparison now uses `bench/e2e/tests.json`, while the commit catalog remains only a convenience smoke.

- Observation: the current change candidate collection mode enum uses `Stream`, not a longer streaming-only variant name.
  Evidence: the final control review flagged the non-existent variant in test guidance. The plan now instructs implementers to compare `ChangeCandidateCollectionMode::Stream` against `ChangeCandidateCollectionMode::Random`.

## Decision Log

- Decision: keep `Waveform` as the engine-facing facade and introduce a private `Backend` enum with a `Wellen(WellenBackend)` variant instead of switching to `Box<dyn WaveformBackend>` in this slice.
  Rationale: an enum keeps the current fast paths explicit, avoids object-safety churn, and leaves later FSDB dispatch choices visible. There is only one runtime backend in this refactor, so a trait object would be ceremony wearing a hat.
  Date/Author: 2026-05-21 / Grin

- Decision: create `SignalId` as an opaque backend-neutral identifier and replace `ResolvedSignal.signal_ref` plus `ExprResolvedSignal.signal_ref` with `id`.
  Rationale: `wellen::SignalRef` must stop crossing from the waveform layer into the engine. `SignalId` gives the engine a stable key for caches and deduplication without knowing how a backend represents signals internally.
  Date/Author: 2026-05-21 / Grin

- Decision: make the Wellen backend derive `SignalId` from `wellen::SignalRef::index()` rather than maintaining mutable signal-id maps.
  Rationale: Wellen exposes a stable zero-based index for `SignalRef`, so no interior mutability is needed in `resolve_signals(&self)` or `resolve_expr_signal(&self)`. Later FSDB code can assign `SignalId` values using its own backend-local policy while still keeping the engine opaque.
  Date/Author: 2026-05-21 / Grin

- Decision: expose Wellen indexed fast paths as an optional facade capability, not as unconditional backend-neutral requirements.
  Rationale: the optimized `change` engines still need a global time table, per-index offsets, and per-index decoding for VCD/FST performance, but FSDB should not be forced to synthesize Wellen-shaped indexed state. This refactor converts the fast-path parameters from `SignalRef` to `SignalId`, returns `None` when a backend lacks indexed support, and keeps portable paths on sampling plus `previous_sample_time`.
  Date/Author: 2026-05-21 / Grin

- Decision: restrict `SignalId` raw construction and inspection to `crate::waveform`.
  Rationale: if `src/engine` can call `SignalId::from_backend_index` or `SignalId::as_u64`, the identifier is not actually opaque. Backend modules need conversion access; engine modules need only equality, hashing, ordering, and debug output.
  Date/Author: 2026-05-21 / Grin

- Decision: keep FST stream candidate collection path-based internally.
  Rationale: the existing streaming path re-resolves canonical paths against `streaming.hierarchy()` before building a stream filter. The implementation must not assume loaded-waveform `SignalId` values can be converted directly into streaming handles if those handle spaces ever differ.
  Date/Author: 2026-05-21 / Grin

- Decision: add and use `Waveform::previous_sample_time(raw_time)` for baseline event scheduling where possible.
  Rationale: edge semantics need the previous value strictly before the current timestamp. A backend-neutral helper reduces direct engine dependence on Wellen's global time table for baseline `change`/`property` scheduling, while preserving the indexed fast paths where they are intentionally Wellen-shaped.
  Date/Author: 2026-05-21 / Grin

- Decision: treat coverage and performance as blocking validation for this refactor.
  Rationale: a behavior-preserving backend boundary can still damage line coverage, cache behavior, or hot-path allocations. The plan therefore requires before/after local coverage and benchmark captures plus final comparisons; no known repeatable regression is accepted.
  Date/Author: 2026-05-21 / Grin

- Decision: use the full CLI benchmark catalog for the blocking before/after comparison and keep the smaller commit catalog as a secondary smoke.
  Rationale: the backend boundary touches `property` and broad waveform paths, while `bench/e2e/tests_commit.json` contains no `property` scenarios. The full `bench/e2e/tests.json` catalog is the appropriate no-regression gate for this refactor.
  Date/Author: 2026-05-21 / Grin

## Outcomes & Retrospective

Current status: this plan has been drafted, revised after focused review, corrected after a final control review, and prepared for commit. Implementation has not started. The intended implementation outcome is a behavior-preserving backend boundary with no public output changes, no schema changes, no Wellen-native handles in engine code, no coverage regression, and no repeatable performance regression.

Plan-authoring outcome: the review loop found and fixed non-verifiable intermediate milestones, unconditional indexed APIs that would force future FSDB code to fake Wellen state, overly broad `SignalId` raw access, unsafe Wellen index conversion guidance, missing property benchmark coverage, unconditional FSDB smoke commands, missing scratch-directory setup, and a non-existent change-candidate enum variant. After implementation, update this section with the actual files changed, validation commands run, benchmark/coverage comparison results, review findings, fixes, remaining risks, and any lessons learned. If the implementation discovers that the enum facade or `SignalId` mapping needs to change, record the reason in `Decision Log` before changing direction. If the benchmarks are noisy, record the raw run directories and rerun evidence rather than hand-waving at the smoke. The smoke does not care about our feelings.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting RTL waveform dumps. RTL means register-transfer-level hardware design data. The current public commands are implemented under `src/engine/`: `info`, `scope`, `signal`, `value`, `change`, and `property`. The command-line argument structs live under `src/cli/`, and rendering plus JSON schema validation live elsewhere. This plan touches the waveform adapter and the small engine call sites that currently depend on Wellen-native signal handles.

VCD and FST are waveform file formats currently read through the Rust crate `wellen`. FSDB is a proprietary waveform format that later work will read through a local Synopsys Verdi FSDB Reader SDK. The default build must remain VCD/FST-only and must not require Verdi. The optional `fsdb` feature already exists from the completed build spike, but it is a build/link smoke only; it must remain disconnected from runtime `Waveform::open` during this refactor.

The current file `src/waveform/mod.rs` does too much. It defines public waveform-facing types such as `WaveformMetadata`, `ScopeEntry`, `SignalEntry`, `ResolvedSignal`, and `ExprResolvedSignal`; it stores a `wellen::simple::Waveform`; it opens files; it maps Wellen scope and variable kinds; it samples values; it implements Wellen-specific FST streaming candidate collection; it exposes indexed fast paths used by `src/engine/change.rs`; and it contains unit tests. That was tolerable with one backend. With FSDB coming, it is an invitation for `#[cfg(feature = "fsdb")]` barnacles. We decline the barnacles.

The expression host bridge is `src/waveform/expr_host.rs`. It implements `crate::expr::ExpressionHost` by resolving names through `Waveform::resolve_expr_signal`, sampling through `Waveform::sample_expr_value`, and checking raw events through `Waveform::expr_event_occurred`. It stores `ExprResolvedSignal` by expression handle. After this refactor, it should keep the same behavior but the stored resolved signal contains a `SignalId`, not a Wellen handle.

The `change` engine is the most sensitive call site. `src/engine/change.rs` has three execution modes: a baseline path that samples candidate timestamps, a fused path that rolls values through the Wellen global time table, and an edge-fast path that decodes Wellen signal values by time-table index. These optimized paths are intentionally Wellen-shaped today. This plan keeps them for VCD/FST, changes their cache keys and method parameters to `SignalId`, and confines native Wellen conversion to `src/waveform/wellen_backend.rs`.

The `property` engine in `src/engine/property.rs` collects candidate timestamps and evaluates an event expression plus a logical expression. It currently builds previous-timestamp pairs from `Waveform::timestamps_raw_slice()`. This plan moves that scheduling to `Waveform::previous_sample_time()` so the baseline expression path no longer needs to know about the global time table.

The performance harnesses are under `bench/`. The CLI end-to-end harness in `bench/e2e/perf.py` runs release binaries through `hyperfine`, captures functional JSON output, and can compare timing plus functional payloads. The expression benchmark harness in `bench/expr/perf.py` runs Criterion benchmarks, including `expr_waveform_host`, which imports `src/waveform/mod.rs` directly through a `#[path]` module. That direct import is a useful tripwire: any module-path mistake in the refactor will break the benchmark build. Annoying, yes. Also useful.

The test suite is under `tests/`. The command fixtures and integration tests assert stable CLI behavior for all existing commands. Unit tests inside `src/waveform/mod.rs` currently cover VCD metadata, scope ordering, signal ordering, duplicate sampling, missing errors, rich-value rejection, indexed decoding, edge classification, and stable kind alias inventories. Move or split these tests with the code they validate; do not delete coverage just because the file got shorter.

## Open Questions

No product question blocks this refactor. The implementation must answer only mechanical questions: how much code remains in `src/waveform/mod.rs` after extraction, and whether any moved helper should be `pub(crate)` or private. Resolve those locally by choosing the least visible API that still lets `src/engine/` compile and keeps tests readable.

If benchmark tooling is unavailable in an execution environment, do not mark performance validation complete. Record the limitation in `Outcomes & Retrospective`, run the strongest available substitute, and leave the performance item unchecked. The acceptance target remains no known repeatable regression.

## Milestones

### Capture local guardrails before editing code

Before changing Rust source files, capture the current coverage and performance baselines in `tmp/`. This gives the refactor a same-machine comparison instead of relying only on committed benchmark artifacts, which may have been produced on different hardware. At the end of this milestone there are no source changes, but there are local disposable baseline artifacts for coverage, the full CLI benchmark catalog, and the waveform-host expression benchmark suite.

Run `make coverage-src` and copy `tmp/coverage/coverage-src-summary.txt` to a before file. Build a release binary and run `bench/e2e/tests.json` into a `tmp/perf/backend-refactor-before-e2e-*` directory. Run the expression benchmark harness with `--suite waveform_host` into a `tmp/perf/backend-refactor-before-expr-*` directory. Record the exact directories in `Progress`. If any baseline command fails on a clean tree, stop and fix the environment before refactoring; otherwise the later comparison is just performance astrology.

### Isolate Wellen behind the facade and convert engines to `SignalId`

Create `src/waveform/types.rs` for backend-neutral data structures and `src/waveform/wellen_backend.rs` for all Wellen-specific logic. Rewrite `src/waveform/mod.rs` into a small facade that declares modules, re-exports the backend-neutral types, stores a private `Backend` enum, and delegates methods to `WellenBackend`. In the same implementation slice, update `src/engine/change.rs`, `src/engine/property.rs`, `src/engine/expr_runtime.rs`, and `src/waveform/expr_host.rs` call sites so the crate compiles with `ResolvedSignal { id: SignalId, ... }` and `ExprResolvedSignal { id: SignalId, ... }`.

At the end of this milestone, `cargo test -q waveform` should pass, and `cargo check` should pass. `rg "wellen::|SignalRef" src/engine src/waveform/expr_host.rs` should find no matches. The optimized `change` modes should still work for Wellen because the facade exposes optional indexed capability methods; future non-indexed backends can return `None` instead of faking a global time table.

### Reduce global-time-table dependence in baseline scheduling

Add `Waveform::previous_sample_time(raw_time) -> Option<u64>` and use it in baseline event scheduling for `change` and `property`. For Wellen, implement it by searching the time table and returning the closest timestamp strictly less than `raw_time`. Keep the optional indexed fast-path methods for optimized `change` modes, and make the optimized dispatcher fall back to the baseline path when `Waveform::indexed_timestamps()` returns `None`.

At the end of this milestone, `property` and the baseline `change` schedule should no longer need to call an indexed time-table method merely to find the previous timestamp. This is a small future-proofing seam for FSDB, where a complete global time table should not be required by the portable path. Re-run `cargo test -q --test change_cli --test change_opt_equivalence --test property_cli` and expect success.

### Preserve and expand tests around identity and facade behavior

Move existing waveform unit tests to the module that owns the code they test, and add focused tests for backend-neutral identity. The new tests should prove that resolving the same canonical signal path through `resolve_signals` and `resolve_expr_signal` produces the same `SignalId`, that duplicate requested signals still preserve output order, that forced FST streaming candidate collection still re-resolves paths correctly, and that indexed decode/cache behavior remains equivalent to batch sampling.

At the end of this milestone, `cargo test -q waveform` and the command integration tests for `info`, `scope`, `signal`, `value`, `change`, and `property` pass. Existing snapshots should not change. If an output snapshot changes, treat that as a bug until proven otherwise.

### Run full validation, compare coverage and performance, and review

Run the default CI gate, coverage comparison, CLI benchmark comparison, expression benchmark comparison, optional FSDB build smoke when Verdi is available, focused review lanes, and a final control review. At the end of this milestone, the plan records actual evidence: test commands, coverage before/after totals, benchmark run directories, compare output, review findings, fixes, and remaining risks.

Acceptance is strict: default VCD/FST behavior unchanged, schema unchanged, no Wellen-native handles in engine code, source coverage not lower than the before baseline, and no repeatable performance regression in the benchmark scenarios. If any of those fail, keep the plan active and fix the refactor. No quietly shipping the dented part because the paint is dry.

## Plan of Work

Start from a clean tree on branch `feat/fsdb` or its implementation successor. All commands below assume the repository root `/workspaces/wavepeek-fsdb`.

First capture the local baselines. Create `tmp/perf/` if needed. Run `make coverage-src`, then copy `tmp/coverage/coverage-src-summary.txt` to a uniquely named before file such as `tmp/coverage/backend-refactor-before.txt`. Run `make build-release`. Run the full CLI benchmark catalog into a before directory under `tmp/perf/`. Run the waveform-host expression benchmark suite into a before directory under `tmp/perf/`. Record these paths in `Progress` so a later agent can compare against the same artifacts.

Next create `src/waveform/types.rs`. Move backend-neutral structs and enums from `src/waveform/mod.rs` into this file: `WaveformMetadata`, `ScopeEntry`, `SignalEntry`, `SampledSignal`, `SampledSignalState`, `ResolvedSignal`, `ExprResolvedSignal`, `ChangeCandidateCollectionMode`, `SignalOffsetData`, `STABLE_SCOPE_KIND_ALIASES`, `EXCLUDED_SCOPE_KIND_ALIASES`, `STABLE_SIGNAL_KIND_ALIASES`, and `EXCLUDED_SIGNAL_KIND_ALIASES`. Add `SignalId` in the same file. `SignalId` must be `Copy`, `Eq`, `Hash`, `Ord`, and `Debug` so the engine can use it in `HashMap`, `HashSet`, and deterministic comparisons. Its numeric value must stay private to ordinary engine code.

Define the key backend-neutral types with this final shape:

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct SignalId(u64);

    impl SignalId {
        pub(in crate::waveform) fn from_backend_index(index: u64) -> Self {
            Self(index)
        }

        pub(in crate::waveform) fn as_u64(self) -> u64 {
            self.0
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ResolvedSignal {
        pub path: String,
        pub id: SignalId,
        pub width: u32,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct ExprResolvedSignal {
        pub path: String,
        pub id: SignalId,
        pub expr_type: ExprType,
    }

The constructor and raw accessor are visible only inside `crate::waveform` so Wellen and later FSDB backends can convert native handles internally, while engine code treats `SignalId` as an opaque key. Do not make the tuple field public and do not widen these methods to `pub(crate)` merely for convenience.

Also give `SignalOffsetData` a constructor visible to waveform backends but not to engine modules, because `wellen_backend.rs` must build offset values while callers should keep treating them as opaque:

    impl SignalOffsetData {
        pub(in crate::waveform) fn new(start: usize, elements: u16) -> Self {
            Self { start, elements }
        }
    }

Create `src/waveform/wellen_backend.rs`. Move all Wellen-specific imports and helpers there: `BufReader`, `PathBuf`, `BTreeSet`, Wellen `ScopeRef`, `SignalRef`, `ScopeType`, `Timescale`, `TimescaleUnit`, `VarType`, `simple`, file-format detection, Wellen error mapping, scope/signal sorting, kind alias mapping from Wellen enums, timescale formatting, value decoding, expression type recovery from Wellen variables, FST streaming candidate collection, and signal loading. Define:

    #[derive(Debug)]
    pub(super) struct WellenBackend {
        inner: simple::Waveform,
        source_path: PathBuf,
        file_format: wellen::FileFormat,
        loaded_signals: HashSet<SignalRef>,
    }

Implement methods matching the current `Waveform` behavior, but use `SignalId` at the boundary. Convert Wellen signal refs with:

    fn signal_id(signal_ref: SignalRef) -> SignalId {
        SignalId::from_backend_index(signal_ref.index() as u64)
    }

    fn signal_ref(id: SignalId) -> Result<SignalRef, WavepeekError> {
        let raw = id.as_u64();
        if raw > u64::from(u32::MAX - 1) {
            return Err(WavepeekError::Internal(
                "signal id exceeds Wellen SignalRef index range".to_string(),
            ));
        }
        let index = usize::try_from(raw).map_err(|_| {
            WavepeekError::Internal("signal id exceeds platform index range".to_string())
        })?;
        SignalRef::from_index(index).ok_or_else(|| {
            WavepeekError::Internal("signal id cannot be represented as a Wellen SignalRef".to_string())
        })
    }

Use this conversion only inside `wellen_backend.rs`. Do not import `wellen::SignalRef` in engine modules or `src/waveform/expr_host.rs`. Preserve the existing FST streaming candidate collector behavior: when it opens a streaming reader, it must re-resolve canonical signal paths against `streaming.hierarchy()` before constructing the stream filter, rather than converting loaded-waveform `SignalId` values directly into stream handles.

Rewrite `src/waveform/mod.rs` as the facade. It should declare:

    #[allow(dead_code)]
    pub(crate) mod expr_host;
    #[cfg(feature = "fsdb")]
    mod fsdb_native;
    mod types;
    mod wellen_backend;

    pub(crate) use types::{...};

    #[derive(Debug)]
    pub struct Waveform {
        backend: Backend,
    }

    #[derive(Debug)]
    enum Backend {
        Wellen(wellen_backend::WellenBackend),
    }

Keep public facade method names stable where practical for user-facing behavior, but make indexed fast paths explicit optional capabilities. The ordinary backend-neutral methods are `open`, `metadata`, `scopes_depth_first`, `signals_in_scope`, `signals_in_scope_recursive`, `sample_signals_at_time`, `resolve_signals`, `resolve_expr_signal`, `resolve_expr_signals`, `sample_resolved_optional`, `sample_expr_value`, `expr_event_occurred`, `collect_change_times`, `collect_change_times_with_mode`, `collect_expr_candidate_times_with_mode`, `should_use_streaming_candidate_collection`, and `previous_sample_time`. The Wellen-oriented optional fast-path methods are `indexed_timestamps`, `indexed_signal_offset_at`, `decode_indexed_signal_at`, and `ensure_indexed_signals_loaded`. Each method should `match &self.backend` or `match &mut self.backend` and delegate to `WellenBackend`.

Keep `sample_signals_at_time` and `duplicate_preserving_projection` in the facade if convenient, because duplicate projection is backend-neutral and will be useful for FSDB. Keep generic edge helpers `EdgeClassification`, `classify_edge`, `should_emit_delta_and_update_baseline`, and `normalize_to_four_state` outside `WellenBackend`; they are waveform semantics, not Wellen adapter code.

Update `src/engine/change.rs`. Replace every `.signal_ref` access with `.id`. Replace `HashMap<(wellen::SignalRef, u32), Option<String>>` with `HashMap<(SignalId, u32), Option<String>>`. Rename local variables so they describe identifiers rather than native refs: `loaded_signal_ids`, `tracked_index_by_id`, `candidate_tracked_indices`, and similar. Build `ResolvedSignal` from `ExprResolvedSignal` with `id: signal.id`. Call `waveform.ensure_indexed_signals_loaded(&ids)` only inside indexed fast-path setup after confirming `indexed_timestamps()` is available. Call `waveform.indexed_signal_offset_at(resolved.id, idx)` and `waveform.decode_indexed_signal_at(resolved, idx)` from indexed code paths, and keep the baseline path on `sample_resolved_optional`.

While editing `change`, add a small helper for baseline candidate schedules that uses `Waveform::previous_sample_time` rather than requiring direct access to the time table. For example:

    fn build_candidate_schedule(
        waveform: &Waveform,
        candidate_times: &[u64],
    ) -> Vec<(u64, Option<u64>)> {
        candidate_times
            .iter()
            .copied()
            .map(|timestamp| (timestamp, waveform.previous_sample_time(timestamp)))
            .collect()
    }

If the optimized fused or edge-fast modes still need the current time-table validation helper, keep a separate indexed helper there. Gate optimized modes on `waveform.indexed_timestamps().is_some()` and fall back to the baseline path when it is `None`. Do not remove Wellen fast paths in this refactor.

Update `src/engine/property.rs` similarly. Deduplicate candidate sources by `signal.id`. Replace its `build_candidate_schedule(timestamps, candidate_times)` helper with one that accepts `&Waveform` or with direct use of `Waveform::previous_sample_time`. This preserves current Wellen semantics because all Wellen candidate times come from the Wellen time table, but it stops baseline property evaluation from knowing how previous timestamps are stored.

Update `src/engine/expr_runtime.rs` to deduplicate expression candidate sources by `resolved.id`. No expression parser or evaluator changes should be necessary.

Update tests. Move Wellen kind-alias tests into `wellen_backend.rs` because they depend on Wellen enum inventories. Keep schema inventory constants re-exported from `crate::waveform` so `src/schema_contract.rs` continues to compile. Add tests named descriptively, not with milestone labels, such as `resolved_signal_ids_are_stable_across_resolution_paths` and `resolved_signal_ids_can_drive_indexed_fast_paths`. Good tests are:

- resolve `top.clk` through `resolve_signals(&["top.clk".to_string()])` and through `resolve_expr_signal("top.clk")`, then assert the IDs are equal;
- resolve `top.clk` twice in one batch and assert the duplicate entries have equal IDs while output projection still preserves duplicate order;
- load signals with `ensure_indexed_signals_loaded(&[resolved[0].id])`, then verify `indexed_signal_offset_at(resolved[0].id, 0)` and `decode_indexed_signal_at(&resolved[0], 0)` behave like the old tests;
- force `ChangeCandidateCollectionMode::Stream` on an FST fixture and verify candidate timestamps match `ChangeCandidateCollectionMode::Random`, proving the streaming path still re-resolves canonical paths;
- run existing VCD/FST expression-host tests unchanged except for any imports required by the module split.

Run `rg` checks before validation. These are not tests, but they catch the important boundary leak:

    rg --line-number "wellen::|SignalRef" src/engine src/waveform/expr_host.rs

Expected result: no matches. Also inspect the waveform module boundary:

    rg --line-number "SignalRef|simple::Waveform|wellen::FileFormat" src/waveform

Expected result: matches are confined to `src/waveform/wellen_backend.rs` and its tests. `src/waveform/mod.rs` should not store Wellen-native fields anymore.

Finally run validation and compare with the before artifacts. The standard behavior gate is `make ci`. The optional FSDB build smoke is `make check-fsdb-build` only when `make check-fsdb-env` reports a usable SDK; skip it otherwise but record the skip. Coverage and performance are blocking for this refactor: run `make coverage-src` again, copy the summary to `tmp/coverage/backend-refactor-after.txt`, and compare the `TOTAL` row with the before summary. Rebuild release and run the same CLI and expression benchmark selections into after directories. Use the compare commands in the next section. If coverage decreases or a benchmark comparison reports a repeatable regression, add tests or fix the implementation before handoff.

## Concrete Steps

Run these from `/workspaces/wavepeek-fsdb`.

1. Confirm the clean starting state and record the baseline commit.

    mkdir -p tmp tmp/perf tmp/coverage
    git status --short --branch
    git rev-parse --short HEAD | tee tmp/backend-refactor-start-commit.txt

Expected output includes the current branch and no uncommitted Rust/source changes.

2. Capture coverage before editing.

    mkdir -p tmp/coverage
    make coverage-src
    cp tmp/coverage/coverage-src-summary.txt tmp/coverage/backend-refactor-before.txt

Expected result: `make coverage-src` exits 0 and the copied summary has a `TOTAL` row. If `cargo llvm-cov` is missing, install it through the documented development environment rather than skipping coverage.

3. Capture CLI and expression performance before editing.

    mkdir -p tmp/perf
    make build-release
    before_e2e="$(mktemp -d -p tmp/perf backend-refactor-before-e2e.XXXXXX)"
    WAVEPEEK_BIN=./target/release/wavepeek python3 -B bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir "$before_e2e"
    before_expr="$(mktemp -d -p tmp/perf backend-refactor-before-expr.XXXXXX)"
    python3 -B bench/expr/perf.py run --suite waveform_host --run-dir "$before_expr"
    printf 'before_e2e=%s\nbefore_expr=%s\n' "$before_e2e" "$before_expr" | tee tmp/perf/backend-refactor-before.env

Expected result: both harnesses finish successfully and write reports in their run directories. The CLI run should report functional payloads, not mismatches. If `hyperfine` is unavailable, fix the devcontainer/tooling before continuing.

4. Create `src/waveform/types.rs` and move backend-neutral declarations there. Update `src/waveform/mod.rs` to `mod types;` and re-export the moved types with `pub(crate) use` or `pub use` matching the current visibility needs. Run a focused check after this mechanical move:

    cargo fmt -- --check
    cargo check

At this intermediate point compile errors are acceptable only while the facade and engine conversion are still in progress. The milestone is not complete until `cargo check` and `cargo test -q waveform` pass; do not leave formatting broken.

5. Create `src/waveform/wellen_backend.rs` and move Wellen-specific implementation code into `WellenBackend`. Convert Wellen-native handles to and from `SignalId` only inside this file. Keep `fsdb_native` untouched except for any module-order formatting rustfmt applies.

6. Rewrite `src/waveform/mod.rs` into the facade with `Backend::Wellen`. Keep generic helpers and facade-level duplicate projection there. Add `previous_sample_time` and optional indexed capability methods: `indexed_timestamps`, `indexed_signal_offset_at`, `decode_indexed_signal_at`, and `ensure_indexed_signals_loaded`.

7. Update engine call sites.

    rg --line-number "signal_ref|SignalRef|wellen::" src/engine src/waveform/expr_host.rs

Use the search output as the to-do list. Replace each engine dependency on Wellen handles with `SignalId`, and gate optimized indexed `change` modes on `indexed_timestamps().is_some()` so future backends can fall back cleanly.

8. Update and add tests. Run focused tests while iterating:

    cargo test -q waveform
    cargo test -q --test info_cli --test scope_cli --test signal_cli --test value_cli
    cargo test -q --test change_cli --test change_opt_equivalence --test change_vcd_fst_parity
    cargo test -q --test property_cli --test property_vcd_fst_parity
    cargo test --bench expr_waveform_host

Expected result: all focused tests pass. The benchmark test command compiles and runs the Criterion target in test mode; it is not the timing capture.

9. Check the backend boundary explicitly.

    rg --line-number "wellen::|SignalRef" src/engine src/waveform/expr_host.rs
    rg --line-number "SignalRef|simple::Waveform|wellen::FileFormat" src/waveform

Expected result: the first command has no matches. The second command has matches only in `src/waveform/wellen_backend.rs` and tests inside that file.

10. Run full default validation.

    make ci

Expected result: formatting, clippy, schema check, GitHub Actions lint, Rust tests, auxiliary tests, and build check pass. `schema/wavepeek.json` should not change. If `make ci` fails because of missing external RTL artifacts, run `make codex-setup` or fix fixture provisioning; do not replace `make ci` with a weaker gate without recording the limitation.

11. Run optional FSDB build smoke if the SDK is available.

    make check-fsdb-env
    if make -s require-verdi >/dev/null 2>&1; then
        make check-fsdb-build
    else
        echo "Skipping make check-fsdb-build: Verdi FSDB SDK is unavailable"
    fi

Expected result on a Verdi-equipped machine: both commands pass. Expected result without Verdi: `make check-fsdb-env` exits 0 with a skip line and the shell prints the explicit skip. This refactor should not break the build-spike smoke.

12. Capture coverage after implementation and compare.

    make coverage-src
    cp tmp/coverage/coverage-src-summary.txt tmp/coverage/backend-refactor-after.txt
    grep '^TOTAL' tmp/coverage/backend-refactor-before.txt tmp/coverage/backend-refactor-after.txt

Expected result: the after `TOTAL` line has no lower line, region, or function coverage percentage than the before line. If coverage is lower, add tests around the moved backend/facade code until coverage recovers. Do not accept a coverage drop as “just a refactor”; that is how tests quietly evaporate.

13. Capture performance after implementation and compare against the local before runs.

    make build-release
    . tmp/perf/backend-refactor-before.env
    after_e2e="$(mktemp -d -p tmp/perf backend-refactor-after-e2e.XXXXXX)"
    WAVEPEEK_BIN=./target/release/wavepeek python3 -B bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir "$after_e2e"
    WAVEPEEK_BIN=./target/release/wavepeek python3 -B bench/e2e/perf.py compare --revised "$after_e2e" --golden "$before_e2e" --max-negative-delta-pct 5 --verbose
    after_expr="$(mktemp -d -p tmp/perf backend-refactor-after-expr.XXXXXX)"
    python3 -B bench/expr/perf.py run --suite waveform_host --run-dir "$after_expr"
    python3 -B bench/expr/perf.py compare --revised "$after_expr" --golden "$before_expr" --max-negative-delta-pct 10 --require-matching-metadata cargo_version rustc_version criterion_version environment_note
    printf 'after_e2e=%s\nafter_expr=%s\n' "$after_e2e" "$after_expr" | tee tmp/perf/backend-refactor-after.env

Expected result: compare commands pass. If either comparison reports a regression, rerun the failing benchmark subset once to filter one-off machine noise. If the regression repeats, fix it before handoff. A known repeatable regression is not accepted.

14. Run repository benchmark convenience gates for parity with existing workflow.

    make bench-e2e-smoke-commit
    make bench-expr-run

Expected result: both pass. `make bench-expr-run` compares against the maintained baseline with the repository threshold. The explicit before/after comparisons from step 13 are still the primary no-regression evidence for this refactor.

15. Run review lanes and apply fixes. Use read-only reviewers for at least architecture/code, validation/performance/coverage, and plan consistency. After fixes, run one independent control review. Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` after each review/fix loop.

16. Commit only after validation and review are complete.

    git status --short
    git diff --check
    git add src/waveform/mod.rs src/waveform/types.rs src/waveform/wellen_backend.rs src/engine/change.rs src/engine/property.rs src/engine/expr_runtime.rs docs/exec-plans/active/2026-05-21-fsdb-backend-refactor/PLAN.md
    git commit -m "refactor(waveform): isolate Wellen backend"

If the implementation only changes this plan and not source code, use a docs commit instead:

    git add docs/exec-plans/active/2026-05-21-fsdb-backend-refactor/PLAN.md
    git commit -m "docs(fsdb): plan waveform backend refactor"

## Validation and Acceptance

The refactor is accepted only when all of the following are true.

First, existing behavior is demonstrably unchanged for VCD/FST. Run `make ci` and expect success. Run the focused command tests listed above and expect success. Inspect `git diff -- schema/wavepeek.json tests/snapshots` and expect no schema or snapshot churn unless a test was intentionally reorganized without output change. CLI benchmark functional artifacts must compare without `data` mismatches.

Second, the backend boundary is real. `rg --line-number "wellen::|SignalRef" src/engine src/waveform/expr_host.rs` returns no matches. `ResolvedSignal` and `ExprResolvedSignal` contain `id: SignalId`, not `signal_ref`. Wellen-native imports are confined to `src/waveform/wellen_backend.rs` and tests that explicitly validate Wellen mapping.

Third, source coverage does not regress. Compare `tmp/coverage/backend-refactor-before.txt` and `tmp/coverage/backend-refactor-after.txt`; the after `TOTAL` row must not have lower line, region, or function coverage. If moving code changes the denominator, add tests until the total percentages recover or improve.

Fourth, performance does not regress. The local before/after CLI comparison must use the full `bench/e2e/tests.json` catalog and pass with `--max-negative-delta-pct 5`; the local before/after expression comparison must use `--suite waveform_host` and pass with `--max-negative-delta-pct 10` plus matching toolchain metadata. `make bench-e2e-smoke-commit` and `make bench-expr-run` must pass. If one benchmark run is noisy, rerun the failing subset and record both runs; a repeatable regression fails acceptance.

Fifth, optional FSDB build-smoke behavior is preserved. On a Verdi-equipped machine, `make check-fsdb-build` still passes. Without Verdi, `make check-fsdb-env` still exits 0 with its skip message and default `make ci` still avoids the `fsdb` feature.

Sixth, naming stays clean. New code files, modules, structs, functions, tests, benchmark run directory prefixes, and plan anchors use descriptive names and do not contain milestone-label prefixes or suffixes. Existing historical tags or architecture milestone text may still exist; do not propagate them into new entities.

## Idempotence and Recovery

The source edits are ordinary Rust refactors and can be retried safely. Keep extraction commits small enough to bisect: type extraction, Wellen backend move, engine `SignalId` conversion, scheduling helper, and tests can be separate commits during implementation if desired. If a move goes sideways, use `git restore -SW src/waveform src/engine/change.rs src/engine/property.rs src/engine/expr_runtime.rs` to return to the last committed state, then reapply in smaller pieces. Yes, smaller pieces. The ancient technology continues to work.

All scratch artifacts belong under `tmp/`, which is ignored by git. Benchmark and coverage before/after files should not be committed. If a benchmark run directory accidentally appears under `bench/e2e/runs/` or `bench/expr/runs/` and is not intentionally promoted, remove it before committing.

Do not use the `read` tool on `.fst` files. The repository safety rule treats `.fst` as binary. Tests and benchmarks may pass `.fst` paths to `wavepeek` or binary-safe tools through the existing harnesses, but do not inspect those files as text.

If `make ci` fails due external fixture provisioning, fix the container or run `make codex-setup`/`make codex-resume` as documented. If Verdi is unavailable, skip only `make check-fsdb-build` and record that `make check-fsdb-env` reported no SDK. Do not simulate FSDB runtime behavior in this refactor.

If coverage or performance comparisons fail, do not delete the guardrail. Add tests, reduce allocations, restore the old fast path, or split the refactor so the regression can be understood. A backend boundary that quietly slows `change` is just an expensive abstraction wearing a fake mustache.

## Artifacts and Notes

Important local artifacts produced during implementation should be recorded here as they are created:

    tmp/coverage/backend-refactor-before.txt
    tmp/coverage/backend-refactor-after.txt
    tmp/perf/backend-refactor-before.env
    tmp/perf/backend-refactor-after.env
    tmp/perf/backend-refactor-before-e2e.*
    tmp/perf/backend-refactor-after-e2e.*
    tmp/perf/backend-refactor-before-expr.*
    tmp/perf/backend-refactor-after-expr.*

Expected boundary search after implementation:

    $ rg --line-number "wellen::|SignalRef" src/engine src/waveform/expr_host.rs
    <no output>

Expected schema check after implementation:

    $ git diff -- schema/wavepeek.json
    <no output>

Expected coverage comparison shape:

    $ grep '^TOTAL' tmp/coverage/backend-refactor-before.txt tmp/coverage/backend-refactor-after.txt
    tmp/coverage/backend-refactor-before.txt:TOTAL ...
    tmp/coverage/backend-refactor-after.txt:TOTAL ...

Replace the ellipses with the actual `cargo llvm-cov` summary rows when implementation runs. If the after percentages are lower, this plan is not complete.

## Interfaces and Dependencies

The new module layout at the end of the refactor must be:

    src/waveform/mod.rs
    src/waveform/types.rs
    src/waveform/wellen_backend.rs
    src/waveform/expr_host.rs
    src/waveform/fsdb_native.rs

`src/waveform/fsdb_native.rs` remains behind `#[cfg(feature = "fsdb")]` and remains private build-smoke support. Do not add `fsdb_backend.rs` in this plan.

`src/waveform/types.rs` owns backend-neutral data types and stable kind inventories. It may depend on `crate::expr::ExprType` for `ExprResolvedSignal`, but it must not depend on `wellen` or native FSDB types.

`src/waveform/wellen_backend.rs` owns all Wellen-specific code. It may depend on `wellen::{simple, SignalRef, ScopeRef, ScopeType, Timescale, TimescaleUnit, VarType}` and `wellen::stream`, and it may know how to convert between `SignalId` and `SignalRef`. No other non-test module outside `src/waveform` should import Wellen-native signal handles after this refactor.

`src/waveform/mod.rs` owns the facade and generic waveform helpers. Its public-facing methods should keep these signatures or their obvious `SignalId` equivalents:

    impl Waveform {
        pub fn open(path: &Path) -> Result<Self, WavepeekError>;
        pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError>;
        pub fn scopes_depth_first(&self, max_depth: Option<usize>) -> Vec<ScopeEntry>;
        pub fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub fn sample_signals_at_time(&mut self, canonical_paths: &[String], query_time_raw: u64) -> Result<Vec<SampledSignal>, WavepeekError>;
        pub fn previous_sample_time(&self, raw_time: u64) -> Option<u64>;
        pub fn resolve_signals(&self, canonical_paths: &[String]) -> Result<Vec<ResolvedSignal>, WavepeekError>;
        pub(crate) fn resolve_expr_signal(&self, canonical_path: &str) -> Result<ExprResolvedSignal, WavepeekError>;
        pub(crate) fn resolve_expr_signals(&self, canonical_paths: &[String]) -> Result<Vec<ExprResolvedSignal>, WavepeekError>;
        pub fn sample_resolved_optional(&mut self, resolved: &[ResolvedSignal], query_time_raw: u64) -> Result<Vec<SampledSignalState>, WavepeekError>;
        pub(crate) fn sample_expr_value(&mut self, resolved: &ExprResolvedSignal, query_time_raw: u64) -> Result<SampledValue, WavepeekError>;
        pub(crate) fn expr_event_occurred(&mut self, resolved: &ExprResolvedSignal, query_time_raw: u64) -> Result<bool, WavepeekError>;
        pub(crate) fn indexed_timestamps(&self) -> Option<&[u64]>;
        pub(crate) fn indexed_signal_offset_at(&self, id: SignalId, time_table_idx: u32) -> Option<SignalOffsetData>;
        pub(crate) fn decode_indexed_signal_at(&self, resolved: &ResolvedSignal, time_table_idx: u32) -> Result<Option<SampledSignalState>, WavepeekError>;
        pub(crate) fn ensure_indexed_signals_loaded(&mut self, ids: &[SignalId]) -> bool;
        pub fn collect_change_times_with_mode(&mut self, resolved: &[ResolvedSignal], from_raw: u64, to_raw: u64, mode: ChangeCandidateCollectionMode) -> Result<Vec<u64>, WavepeekError>;
        pub(crate) fn collect_expr_candidate_times_with_mode(&mut self, resolved: &[ExprResolvedSignal], from_raw: u64, to_raw: u64, mode: ChangeCandidateCollectionMode) -> Result<Vec<u64>, WavepeekError>;
    }

`src/engine/change.rs` may use `SignalId` as a key but must not inspect its raw numeric value. It may use optional indexed capability methods only after checking `indexed_timestamps()`; otherwise it must fall back to sampling-based baseline logic. `src/engine/property.rs` and `src/engine/expr_runtime.rs` may deduplicate by `SignalId`. `src/waveform/expr_host.rs` should not need semantic changes; it stores `ExprResolvedSignal` and calls facade methods.

No new external Rust crates are required for this refactor. Do not change `Cargo.toml` unless an implementation discovery proves it is unavoidable, and if it does, record the reason in `Decision Log` first. The optional `fsdb` feature and `cc` build dependency already exist from the build spike and should remain unchanged.

## Change Notes

- 2026-05-21 / Grin: Initial plan created for the backend refactor described in `docs/fsdb/arch.md`. The plan is self-contained, names new entities descriptively without milestone labels, and includes explicit performance plus coverage guardrails because behavior-preserving refactors can still make the machine sulk.
- 2026-05-21 / Grin: Revised after focused review. The plan now keeps milestones independently verifiable, treats indexed fast paths as optional backend capabilities, restricts raw `SignalId` access to waveform modules, validates Wellen signal-id bounds, preserves FST streaming path re-resolution, uses the full CLI benchmark catalog for blocking performance control, makes FSDB build smoke conditional, and creates scratch directories before writing baseline files.
- 2026-05-21 / Grin: Revised after final control review. The forced streaming test guidance now names the existing `ChangeCandidateCollectionMode::Stream` and `ChangeCandidateCollectionMode::Random` variants instead of inventing a new enum variant. Shocking restraint from the machinery, eventually.
