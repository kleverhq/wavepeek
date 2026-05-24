# Implement FSDB metadata and hierarchy commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the repository `exec-plan` skill. It is intentionally self-contained: a contributor with only the current working tree and this file should be able to implement, validate, and review the change without prior conversation.

All repository entities created by this plan must use descriptive FSDB command names, not milestone labels. Do not add milestone-label prefixes, suffixes, directory names, module names, function names, test names, temporary run directories, documentation anchors, or commit subjects. Use names such as `fsdb_backend`, `fsdb_hierarchy`, `fsdb_time`, `fsdb_cli`, `FsdbBackend`, `FsdbHierarchyIndex`, and `fsdb_signal_json_is_sorted_and_stable`.

## Purpose / Big Picture

After this change, a `wavepeek` binary built with the optional Cargo feature `fsdb` and a local licensed Synopsys Verdi FSDB Reader SDK can run the existing `info`, `scope`, and `signal` commands directly on FSDB waveform files. Users do not learn new commands or pass FSDB-only flags: `wavepeek info --waves dump.fsdb --json`, `wavepeek scope --waves dump.fsdb --json`, and `wavepeek signal --waves dump.fsdb --scope <scope> --json` use the same JSON schema, human rendering, limits, sorting, and error categories as VCD/FST.

The visible proof is a Verdi-equipped `make test-fsdb` run: the Makefile generates ignored `.fsdb` files from the hand-written VCD fixtures, `info` / `scope` / `signal` JSON payloads match generated test expectations from those VCD sources, and a separate smoke pass against `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` prints non-empty deterministic structure using only existing stable kind aliases. Default builds without the `fsdb` feature continue to pass `make ci` without Verdi and keep the existing clear FSDB-disabled error. Later FSDB command work for `value`, `change`, and `property` remains unimplemented here; those commands must fail clearly on FSDB input rather than panic or silently produce nonsense. Nonsense is cheaper to generate than to debug, which is why we refuse the bargain.

## Non-Goals

This plan does not implement FSDB value sampling, event candidate collection, expression evaluation, `change`, or `property`. It may add clear unsupported-operation errors for those command paths when an FSDB-enabled binary opens an FSDB file, but it must not pretend those commands are complete.

This plan does not add public CLI flags, public JSON fields, JSON schema changes, public error categories, or new `scope.kind` / `signal.kind` strings. FSDB output must fit the existing `schema/wavepeek.json` contract.

This plan does not publish FSDB-enabled binaries and does not change default public CI into a Verdi-dependent workflow. The `fsdb` Cargo feature remains explicit and local to machines with a licensed Verdi installation.

This plan does not commit Verdi headers, libraries, documentation excerpts, generated bindings derived from proprietary headers, `.fsdb` fixtures, full golden outputs from Verdi-bundled FSDB files, native logs, or any copied vendor sample code. The Makefile may generate adjacent `.fsdb` files from checked-in VCD fixtures for local testing, but those files remain ignored local artifacts. Tests may reference bundled example FSDB files by path under `$VERDI_HOME`; they must not read FSDB files as text and must not check in the files or their proprietary design contents.

This plan does not add milestone-labelled entity names. If a new name would contain a milestone label, rename it before committing. A little naming hygiene now prevents fossilized archaeology later.

## Progress

- [x] (2026-05-23 21:03Z) Read `docs/fsdb/arch.md`, `docs/fsdb/cmd_info.md`, `docs/fsdb/cmd_scope.md`, `docs/fsdb/cmd_signal.md`, `docs/fsdb/verdi_home_map.md`, completed FSDB build/refactor/disabled plans, `src/waveform/mod.rs`, `src/waveform/types.rs`, `src/waveform/wellen_backend.rs`, `src/waveform/fsdb_native.rs`, `native/fsdb/wavepeek_fsdb_shim.{h,cpp}`, `Makefile`, `docs/DEVELOPMENT.md`, and relevant CLI tests.
- [x] (2026-05-23 21:03Z) Drafted this active ExecPlan under `docs/exec-plans/active/2026-05-23-fsdb-hierarchy-commands/PLAN.md` with descriptive names and no milestone-labelled created entities.
- [x] (2026-05-23 21:15Z) Ran focused read-only plan review lanes for exec-plan completeness, native/FFI feasibility, Rust backend architecture, and testing/licensing coverage.
- [x] (2026-05-23 21:25Z) Applied review fixes: strengthened datatype block traversal, required panic-safe Rust callbacks, serialized native output suppression, made metadata lazy, clarified FSDB probe fallback rules, expanded unsupported-command tests, fixed feature-test skip wording, switched integration kind validation to schema-derived enums, required non-empty signal discovery, and added gated FSDB lint guidance.
- [x] (2026-05-23 21:35Z) Ran one fresh independent control review on the revised plan; it found one contradictory `Waveform::open` precedence sentence.
- [x] (2026-05-23 21:38Z) Fixed the control-review finding by stating one explicit probe precedence rule: a positive FSDB probe opens `FsdbBackend` immediately, while only negative or failed probes fall through to Wellen.
- [x] (2026-05-23 21:42Z) Ran a targeted recheck of the probe precedence fix; it returned no substantive findings.
- [x] (2026-05-23 21:44Z) Committed the initial reviewed plan and architecture link as `f11cd66 docs(fsdb): plan hierarchy command support`.
- [x] (2026-05-24 10:32Z) Revisited the FSDB fixture strategy and chose a simpler generated-fixture path: a Makefile target generates adjacent ignored `.fsdb` files for every VCD fixture under `tests/fixtures/hand/`, and `test-fsdb` depends on that preparation.
- [x] (2026-05-24 10:41Z) Ran focused read-only review on the generated FSDB fixture plan update; review found masked converter failures, missing explicit `vcd2fsdb` validation, and a recursive-discovery mismatch.
- [x] (2026-05-24 10:43Z) Applied review fixes: `prepare-fsdb-fixtures` now validates `vcd2fsdb`, uses `set -eu`, avoids a piped loop, writes through a temporary FSDB file before atomic `mv`, and the parity tests use the same recursive fixture scope as the Make target.
- [x] (2026-05-24 10:47Z) Ran targeted recheck of the generated-fixture fixes; it found one stale non-recursive `docs/DEVELOPMENT.md` wording example, which was corrected to “all `.vcd` files under `tests/fixtures/hand/`”.
- [x] (2026-05-24 10:49Z) Ran a final tiny recheck of the recursive fixture-scope wording; it returned no substantive findings.
- [x] (2026-05-24 10:50Z) Prepared the updated plan for commit with a conventional commit message that does not contain milestone-labelled entity names.
- [x] (2026-05-24 12:32Z) Began implementation from clean branch `feat/fsdb` at `5ca48e1`; default `make check`, default `make ci`, `make check-fsdb-env`, and current `make check-fsdb-build` all passed before editing.
- [x] (2026-05-24 12:44Z) Added pure Rust `fsdb_time` and `fsdb_hierarchy` helpers with no-Verdi unit coverage for scale-unit normalization, raw time overflow, sorting, depth filtering, hidden subtree exclusion, deduplication, packed range normalization, direct/recursive signal listing, missing scope/signal errors, stable kind aliases, and enum datatype override.
- [x] (2026-05-24 12:44Z) Ran `cargo fmt -- --check`, `cargo test -q fsdb_time`, and `cargo test -q fsdb_hierarchy`; all passed after formatting.
- [x] (2026-05-24 13:08Z) Extended the wavepeek-owned native FSDB C ABI and Rust `fsdb_native` wrapper for datatype and scope/variable traversal, including serialized Reader calls and panic-safe Rust callback handling.
- [x] (2026-05-24 13:08Z) Ran `cargo fmt -- --check`, `VERDI_HOME=$(./.devcontainer/resolve_verdi_home.sh) cargo check --features fsdb`, `cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture`, and `cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture`; feature build and native smokes passed.
- [x] (2026-05-24 13:49Z) Wired `FsdbBackend` into `Waveform::open` with feature-enabled FSDB probing, Result-returning scope traversal, metadata normalization, hierarchy-backed scope/signal resolution, and explicit unsupported errors for `value`, `change`, and `property`.
- [x] (2026-05-24 13:49Z) Added generated FSDB fixtures, `scripts/prepare_fsdb_fixtures.sh`, Make targets `lint-fsdb`, `prepare-fsdb-fixtures`, and expanded `check-fsdb-build`/`test-fsdb`; added `tests/fsdb_cli.rs` covering generated fixture parity, bundled `cpu.fsdb` smoke behavior, missing-scope errors, and unsupported commands.
- [x] (2026-05-24 13:49Z) Ran `cargo fmt -- --check`, `cargo check`, `cargo check --features fsdb`, default and FSDB clippy, pure helper tests, and native metadata/hierarchy smokes; focused command probes against generated FSDB fixtures and bundled `cpu.fsdb` produced expected JSON and unsupported-command errors.
- [x] (2026-05-24 14:02Z) Updated `docs/DEVELOPMENT.md` for `lint-fsdb`, `prepare-fsdb-fixtures`, expanded `check-fsdb-build`, and `test-fsdb`; suppressed converter stdout/stderr in the fixture preparation script except on failure.
- [x] (2026-05-24 14:02Z) Ran final validation after commit `07ed4c1`: `WAVEPEEK_IN_CONTAINER=1 make check`, `WAVEPEEK_IN_CONTAINER=1 make ci`, `WAVEPEEK_IN_CONTAINER=1 make lint-fsdb`, `WAVEPEEK_IN_CONTAINER=1 make test-fsdb`, and a standalone `VERDI_HOME=$(./.devcontainer/resolve_verdi_home.sh) cargo test --features fsdb --test fsdb_cli`; all passed.

## Surprises & Discoveries

- Observation: the current FSDB native layer already opens a real FSDB and reads metadata through `wp_fsdb_open`, `wp_fsdb_read_metadata`, and the private Rust `FsdbReader` smoke test, but it is intentionally disconnected from `Waveform::open`.
  Evidence: `src/waveform/fsdb_native.rs` declares the current FFI wrapper behind `#[cfg(feature = "fsdb")]`, and `src/waveform/mod.rs` declares the module without using it in the `Backend` enum.

- Observation: `Waveform::scopes_depth_first` currently returns `Vec<ScopeEntry>` rather than `Result<Vec<ScopeEntry>, WavepeekError>`.
  Evidence: `src/waveform/mod.rs` delegates directly to Wellen, and `src/engine/scope.rs` calls `.scopes_depth_first(...).into_iter()`. An FSDB backend may fail while lazily traversing hierarchy callbacks, so this plan changes the facade method to return `Result` and updates the scope engine accordingly.

- Observation: the current backend boundary already hides Wellen-native signal handles from engine modules.
  Evidence: `src/waveform/types.rs` exposes opaque `SignalId`, while Wellen conversion functions remain in `src/waveform/wellen_backend.rs`. The FSDB backend should follow this pattern rather than leaking FSDB idcodes into `src/engine`.

- Observation: default automation already keeps the optional `fsdb` feature out of normal lint, coverage, pre-commit, and CI paths.
  Evidence: `docs/DEVELOPMENT.md` says the optional `fsdb` feature is intentionally excluded from default checks; `Makefile` has explicit `check-fsdb-env`, `require-verdi`, `check-fsdb-build`, and `test-fsdb` targets.

- Observation: the architecture explicitly permits using Verdi-bundled FSDB examples for gated tests, but not committing FSDB fixtures or full proprietary golden outputs.
  Evidence: `docs/fsdb/arch.md` names `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` as the first smoke fixture and says tests should use FSDB files only through the Reader API or binary-safe metadata operations.

- Observation: feature-enabled Cargo tests cannot provide a no-SDK skip path because `build.rs` runs before Rust test code.
  Evidence: `Cargo.toml` feature `fsdb` activates the native build script checks, and `Makefile` already gates explicit FSDB targets through `require-verdi` before invoking Cargo.

- Observation: integration tests cannot import `STABLE_SCOPE_KIND_ALIASES` or `STABLE_SIGNAL_KIND_ALIASES` directly because they are crate-private implementation constants.
  Evidence: `src/waveform/types.rs` declares those inventories as `pub(crate)`, so `tests/fsdb_cli.rs` must validate output kinds through the public schema artifact or a test-local allowlist rather than widening production visibility.

- Observation: in-process library tests are a poor place to assert that the native Reader wrote nothing to stdout/stderr.
  Evidence: current smoke commands run with `--nocapture`, and stdout/stderr are process-global file descriptors. Clean output is better asserted by CLI subprocess tests that capture child output.

- Observation: a plan can quietly contradict itself in one sentence while sounding terribly authoritative. Rude, but instructive.
  Evidence: final control review found the first revised `Waveform::open` guidance both opened FSDB immediately on a positive suffix probe and later implied Wellen could still win for the same path. The plan now has one precedence rule.

- Observation: the repository already ignores generated FSDB files and Verdi converter log directories.
  Evidence: `.gitignore` contains `*.fsdb`, `*.fsdb.*`, and `*Log/`, so adjacent generated FSDB fixtures and `vcd2fsdbLog/` will not be tracked unless someone goes out of their way to anger the tooling.

- Observation: all checked-in VCD fixtures currently live under `tests/fixtures/hand/`.
  Evidence: `find tests -name '*.vcd' -type f` lists only paths in that directory, including `m2_core.vcd`, `signal_recursive_depth.vcd`, `scope_mixed_kinds.vcd`, and change/expression fixtures that can still exercise `info`, `scope`, and `signal` parity.

- Observation: the simple Make loop still needed guardrails because shell pipelines and converter outputs are perfectly capable of lying by accident.
  Evidence: focused review noted that a piped `find | sort | while` loop can mask early failures, a direct `vcd2fsdb -o final.fsdb` can leave a partial output that appears up to date, and `require-verdi` does not by itself prove `vcd2fsdb` is on `PATH`.

- Observation: the public schema has `unknown` for `scopeKind` but not for `signalKind`.
  Evidence: the pure helper contract test failed when `RawSignalKind::Unknown` mapped to `unknown`; `schema/wavepeek.json` and `STABLE_SIGNAL_KIND_ALIASES` contain `bit_vector` but no signal-level `unknown`, so unknown FSDB signal variables now degrade to the stable `bit_vector` alias instead of emitting a schema-invalid kind.

- Observation: the local FSDB Reader datatype block API can read the full datatype definition range in one call starting at block index `0`.
  Evidence: `/opt/verdi/share/FsdbReader/ffrAPI.h` documents `ffrReadDataTypeDefByBlkIdx2` as reading from the supplied block to the last block and updating the biggest block read; the implementation calls it once with `block_index = 0` before `ffrReadScopeVarTree2`, and both `cargo check --features fsdb` and the `fsdb_reader_hierarchy_smoke` test passed against the bundled `cpu.fsdb`.

- Observation: the SDK callback examples use `TRUE` and `FALSE`, but this build environment does not expose those macros to the shim translation unit.
  Evidence: the first feature build failed with `TRUE`/`FALSE` undeclared in `native/fsdb/wavepeek_fsdb_shim.cpp`; returning `static_cast<bool_T>(1)` and `static_cast<bool_T>(0)` matches the callback ABI without depending on those convenience macros.

- Observation: clippy flags the feature-enabled backend enum as a large enum unless backend payloads use indirection.
  Evidence: `cargo clippy --features fsdb --all-targets -- -D warnings` reported `clippy::large-enum-variant` for `Backend::Wellen` versus `Backend::Fsdb`; the facade now stores both backend variants in `Box` values.

- Observation: `vcd2fsdb` writes a repository-root `vcd2fsdbLog` directory and a long proprietary banner unless it is isolated and captured.
  Evidence: the first `cargo test --features fsdb --test fsdb_cli -- --nocapture` and `make test-fsdb` runs printed Synopsys banner text and left ignored generated files/logs. The integration tests now run converter subprocesses in test temp directories with captured stdout/stderr, and `scripts/prepare_fsdb_fixtures.sh` captures converter output unless conversion fails.

## Decision Log

- Decision: implement the feature behind `FsdbBackend` plus helper modules named `fsdb_backend`, `fsdb_native`, `fsdb_hierarchy`, and `fsdb_time`, with no milestone-labelled names.
  Rationale: these names describe long-lived responsibilities: command backend dispatch, native Reader FFI, pure Rust hierarchy normalization, and pure Rust time-unit normalization. They will still make sense after this planning phase is forgotten, which is the polite word for “buried under later commits.”
  Date/Author: 2026-05-23 / Grin

- Decision: keep the existing `Waveform` facade and add `Backend::Fsdb(fsdb_backend::FsdbBackend)` behind `#[cfg(feature = "fsdb")]` rather than introducing a trait-object backend in this slice.
  Rationale: the previous backend refactor intentionally used an enum facade. Extending that enum is the smallest mechanical change and keeps Wellen fast paths explicit while FSDB implements only portable command surfaces for now.
  Date/Author: 2026-05-23 / Grin

- Decision: change `Waveform::scopes_depth_first` to return `Result<Vec<ScopeEntry>, WavepeekError>`.
  Rationale: Wellen scope traversal is currently infallible after open, but FSDB hierarchy traversal through native callbacks can fail on first use. Returning a `Result` avoids forcing `FsdbBackend::open` to eagerly read the full hierarchy just so `scope` can report errors.
  Date/Author: 2026-05-23 / Grin

- Decision: make `info` use FSDB metadata without building the hierarchy index, while `scope` and `signal` lazily build and cache an FSDB hierarchy index.
  Rationale: `info` needs only scale unit and min/max time tags. Building hierarchy for `info` would be decorative work in a hot path, and decorative work tends to become a tax nobody remembers voting for.
  Date/Author: 2026-05-23 / Grin

- Decision: expose only wavepeek-owned C ABI types from `native/fsdb/wavepeek_fsdb_shim.h`; C++ maps proprietary FSDB constants to project-owned raw records or enums before Rust sees them.
  Rationale: committed Rust must not depend on proprietary enum numeric values or generated proprietary bindings. The C++ shim may include local SDK headers at build time, but public committed declarations remain wavepeek-owned.
  Date/Author: 2026-05-23 / Grin

- Decision: keep FSDB integration tests structural and deterministic rather than checking in full golden outputs from Verdi-bundled examples.
  Rationale: the bundled examples are local licensed artifacts, not repository fixtures. Tests should prove contract behavior without storing design contents from those files in Git.
  Date/Author: 2026-05-23 / Grin

- Decision: for `value`, `change`, and `property` on an FSDB-enabled binary, return clear unsupported-operation errors until their dedicated implementation slices exist.
  Rationale: once `Waveform::open` can return `FsdbBackend`, later command paths may reach unimplemented methods. A deliberate error is better than an internal panic, a misleading Wellen parse error, or an accidental empty result.
  Date/Author: 2026-05-23 / Grin

- Decision: make FSDB metadata normalization lazy instead of a required `FsdbBackend::open` step.
  Rationale: `scope` and `signal` do not need timestamps. An FSDB file with unsupported time metadata should fail `info` and time-dependent later commands, not prevent hierarchy inspection if the Reader can still traverse scopes and variables.
  Date/Author: 2026-05-23 / Grin

- Decision: preserve the original Wellen result unless FSDB probing positively identifies FSDB, except that a probe error may become the final error for an FSDB-looking path only after Wellen also fails.
  Rationale: this preserves valid VCD/FST files with misleading suffixes and ordinary non-FSDB parse errors while still surfacing Reader failures for paths the user clearly presented as FSDB. Probe errors before Wellen success are advisory, not a license to break existing behavior.
  Date/Author: 2026-05-23 / Grin

- Decision: serialize native FSDB Reader calls that suppress process stdout/stderr.
  Rationale: the existing suppressor redirects process file descriptors. Parallel tests or concurrent backend calls can otherwise steal unrelated output. A mutex is not glamorous, but neither is debugging fd roulette.
  Date/Author: 2026-05-23 / Grin

- Decision: validate feature-enabled output kinds through the public schema enum or a test-local public-contract allowlist, not by importing crate-private constants.
  Rationale: integration tests should not widen production visibility just to share internal constants. The schema is the public machine contract that users actually consume.
  Date/Author: 2026-05-23 / Grin

- Decision: generate adjacent ignored FSDB counterparts for every checked-in VCD fixture under `tests/fixtures/hand/` through `make prepare-fsdb-fixtures`, and make `test-fsdb` depend on that target.
  Rationale: the hand-written VCD fixtures are already the compact command-contract corpus. Converting all of them to FSDB locally gives strong VCD-vs-FSDB parity coverage without a manifest, without committed binaries, and without asking future agents to remember which tiny fixture mattered this week. It is simple, which is sometimes allowed.
  Date/Author: 2026-05-24 / Grin

- Decision: keep generated FSDB parity tests and the bundled Verdi `cpu.fsdb` smoke tests as separate layers.
  Rationale: generated fixtures prove that FSDB follows the same public output contracts as known VCD inputs. The bundled Verdi example still proves the Reader can handle a real vendor-created FSDB that did not pass through our VCD converter path.
  Date/Author: 2026-05-24 / Grin

- Decision: make FSDB fixture generation fail-fast and atomic rather than merely short.
  Rationale: the target is still simple, but it must validate `vcd2fsdb`, avoid a subshell pipeline that can hide failures, and write to a temporary `.fsdb` before `mv` so failed conversion cannot create a stale “new enough” output. The optimistic version required the converter never to hiccup, which is not a plan; it is a wish with indentation.
  Date/Author: 2026-05-24 / Grin

- Decision: make `tests/fsdb_cli.rs` generate its own temporary FSDB files, while keeping `make prepare-fsdb-fixtures` as the user-visible fixture preparation target.
  Rationale: direct `cargo test --features fsdb --test fsdb_cli` should be useful during iteration without depending on persistent ignored files, and `make test-fsdb` still exercises the preparation script. The tests use temporary directories so generated `.fsdb` files and converter logs are cleaned automatically.
  Date/Author: 2026-05-24 / Grin

- Decision: box both `Backend::Wellen` and `Backend::Fsdb` in `src/waveform/mod.rs`.
  Rationale: the feature-enabled enum otherwise tripped `clippy::large-enum-variant`, and boxing both variants keeps the facade size predictable without changing command behavior.
  Date/Author: 2026-05-24 / Grin

## Outcomes & Retrospective

Implementation is complete for this FSDB hierarchy command slice and the plan remains in `active/` for user review. A feature-enabled build now opens real FSDB files through the Verdi Reader when probing says the file is FSDB, preserves Wellen behavior for VCD/FST including misleading `.fsdb` suffixes, normalizes FSDB metadata times, lists scopes, lists direct and recursive signals, resolves hierarchy-backed signal metadata, and fails `value`, `change`, and `property` with explicit unsupported FSDB messages rather than pretending sampled values exist. The native shim now exposes wavepeek-owned hierarchy records, serializes Reader calls around global output suppression, maps SDK scope/signal/datatype events into stable public aliases, and lets Rust build a lazy cached hierarchy index behind `FsdbBackend`. Default builds remain Verdi-free and schema-compatible.

Validation passed with `make check`, `make ci`, `make lint-fsdb`, `make test-fsdb`, focused helper unit tests, native metadata/hierarchy smokes, and `tests/fsdb_cli.rs`. The main remaining limitation is intentional: FSDB value sampling, change collection, and property evaluation are still future work. Generated `.fsdb` files remain ignored binary artifacts and were removed from the working tree after validation. Lessons learned: the FSDB Reader API is workable for hierarchy, but its process-global output behavior and `vcd2fsdb` side effects demand more containment than the usual cheerful CLI tool. Naturally.

## Context and Orientation

`wavepeek` is a Rust command-line tool for inspecting RTL waveform dumps. VCD and FST currently work through the Rust crate `wellen`. FSDB is the proprietary Synopsys Fast Signal Database format and must be read through a local licensed Verdi FSDB Reader SDK. The repository may store project-owned source that calls the Reader API, but it must not store Synopsys headers, libraries, documentation excerpts, generated bindings, or FSDB files.

The current command flow is simple. CLI modules under `src/cli/` parse arguments. Engine modules under `src/engine/` call the waveform facade in `src/waveform/mod.rs`. The facade opens the waveform once per command and delegates to a backend. `src/error.rs` maps `WavepeekError::File` to exit code `2`; `WavepeekError::Scope`, `WavepeekError::Signal`, and argument errors use existing categories and must not be replaced by a new FSDB-specific envelope.

The important source files today are:

- `Cargo.toml`, which already defines a non-default `fsdb` Cargo feature.
- `build.rs`, which already checks `$VERDI_HOME`, compiles `native/fsdb/wavepeek_fsdb_shim.cpp`, links `libnffr.so` and companion libraries, and is inert without the `fsdb` feature.
- `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp`, which currently expose `wp_fsdb_probe`, `wp_fsdb_open`, `wp_fsdb_close`, `wp_fsdb_read_metadata`, and free functions.
- `src/waveform/fsdb_native.rs`, which currently wraps the native metadata functions privately and has a feature-gated metadata smoke test.
- `src/waveform/mod.rs`, which currently has only `Backend::Wellen` and still tries Wellen first in `Waveform::open`.
- `src/waveform/types.rs`, which defines backend-neutral `SignalId`, `WaveformMetadata`, `ScopeEntry`, `SignalEntry`, `ResolvedSignal`, and stable scope/signal kind alias inventories.
- `src/waveform/wellen_backend.rs`, which is the existing VCD/FST backend and is the behavior reference for sorting, kind aliases, signal widths, and errors.
- `tests/info_cli.rs`, `tests/scope_cli.rs`, `tests/signal_cli.rs`, and `tests/fsdb_disabled_cli.rs`, which lock current public CLI behavior.

The completed preparation work matters. The build spike added the `fsdb` feature, native link smoke, environment checker, and `make check-fsdb-build`. The backend refactor moved Wellen behind an enum facade and introduced opaque `SignalId`. The default-build disabled UX added clear `.fsdb` errors when the feature is absent. This plan starts from those completed states and connects the feature-enabled backend for metadata and hierarchy commands only.

Canonical paths are dot-separated strings such as `top.u0.data`. FSDB may internally report `/` or another scope separator; that separator must not leak into public output. Scope entries are JSON objects with `path`, `depth`, and `kind`. Signal entries are JSON objects with `name`, `path`, `kind`, and optional `width`. The JSON schema already enumerates stable kind strings; FSDB code must map to those strings or to `unknown`, not invent new values.

The `.fst` files in this repository are binary and must never be read with the text `read` tool. Treat `.fsdb` files with the same caution: do not inspect their contents as text. Tests may run the Reader API against local Verdi examples and may use binary-safe filesystem checks such as existence, file size, or path construction.

## Open Questions

No product decision blocks this plan. The main implementation-time discovery is the exact datatype definition block enumeration API in the local FSDB Reader headers. The requirement is settled: do not treat block index `0` as complete unless the SDK explicitly says there is only one datatype block. Use a documented block count or iterator when available; otherwise use a bounded sequential read that stops only on the Reader’s documented normal end/no-data result and treats other return codes as errors. Record the exact API behavior in `Surprises & Discoveries` during implementation.

## Milestones

### Capture guardrails and confirm the starting point

Start from `/workspaces/wavepeek-fsdb` with a clean tree. Confirm the branch, current commit, default validation, and optional FSDB availability before editing. This milestone produces no source changes except updating this plan’s `Progress` section with the exact observations.

Run:

    cd /workspaces/wavepeek-fsdb
    git status --short --branch
    git rev-parse --short HEAD
    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env

If Verdi is available, also run:

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-build

Acceptance for this milestone is a recorded baseline: default checks pass, `check-fsdb-env` either prints the stable skip line or reports an SDK, and any FSDB build smoke result is recorded. Do not start the backend wiring from a dirty or already-failing tree unless you first document the unrelated failure and isolate it. Mystery failures age poorly.

### Add pure Rust FSDB time and hierarchy helpers

Create helper modules that can be unit-tested without a Verdi installation. Add `src/waveform/fsdb_time.rs` compiled with `#[cfg(any(test, feature = "fsdb"))]`. It should parse FSDB scale-unit strings and normalize raw integer tags into the same string format Wellen uses for metadata. Required accepted forms include integer units such as `1ns`, `10ps`, and `100ps`; short suffixes such as `1n` and `1p`; and exact fractional short forms such as `0.1n`, `0.01n`, and `0.001n`, which normalize to `100ps`, `10ps`, and `1ps`. Accepted suffixes are `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, and `s`, plus short single-letter `z`, `a`, `f`, `p`, `n`, `u`, and `m` where unambiguous. Reject zero scale factors, unsupported suffixes, inexact fractional conversions, malformed strings, negative values, and `raw * factor` overflow with `WavepeekError::File`.

Add `src/waveform/fsdb_hierarchy.rs`, also compiled with `#[cfg(any(test, feature = "fsdb"))]`. This module owns pure Rust records and normalization. It should define project-owned raw enums and structs such as `RawScopeKind`, `RawSignalKind`, `RawDatatypeKind`, `RawScopeRecord`, `RawSignalRecord`, `RawDatatypeRecord`, `FsdbHierarchyBuilder`, `FsdbHierarchyIndex`, and internal `FsdbSignalInfo`. These names are suggestions, not public API; keep them private or `pub(super)` unless tests require `pub(crate)`. The module should not refer to proprietary FSDB enum numeric values. It receives already-mapped raw records from `fsdb_native` and produces existing `ScopeEntry`, `SignalEntry`, `ResolvedSignal`, and later expression metadata where possible.

`fsdb_hierarchy` must implement canonical dot-separated path construction from scope push/pop events, hidden-scope subtree exclusion, duplicate scope deduplication by canonical path, duplicate signal deduplication by canonical signal path, deterministic scope DFS sorting, deterministic signal sorting by `(name, path)`, scope lookup, direct signal listing, recursive signal listing, signal name normalization, width extraction, and stable kind mapping. Signal name normalization strips only a trailing packed range matching the variable bit bounds. For example, `A[3:0]` with left `3` and right `0` becomes `A` width `4`; `a[0][1][7:0]` becomes `a[0][1]` width `8`; `B[3]` with scalar bit bounds stays `B[3]` width `1` because that is an array index, not the packed range.

Pure Rust tests must cover at least: all accepted and rejected scale-unit forms; overflow during metadata normalization; scope sorting and max-depth filtering through the index; hidden scope subtree exclusion; duplicate scope and signal deduplication; packed-range normalization examples; direct versus recursive signal listing; missing scope and missing signal errors; stable scope kind aliases are members of `STABLE_SCOPE_KIND_ALIASES`; stable signal kind aliases are members of `STABLE_SIGNAL_KIND_ALIASES`; datatype override for enum kind when a raw datatype record says a signal’s `dtidcode` is enum. These tests should run in the default build without Verdi.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo test -q fsdb_time
    cargo test -q fsdb_hierarchy

The tests should pass without setting `VERDI_HOME` and without enabling the `fsdb` Cargo feature.

### Extend the native FSDB shim for hierarchy traversal

Extend `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp` with wavepeek-owned C ABI types for hierarchy and datatype traversal. The header must not copy proprietary FSDB declarations. It may define project-owned enums for traversal event kind, scope kind, signal kind, and datatype kind, plus small project-owned records containing copied primitive fields and borrowed callback strings. Good names are `wp_fsdb_tree_event`, `wp_fsdb_scope_kind`, `wp_fsdb_signal_kind`, `wp_fsdb_datatype_kind`, `wp_fsdb_scope_record`, `wp_fsdb_signal_record`, `wp_fsdb_datatype_record`, and `wp_fsdb_tree_callback`.

The required native behavior is:

- Keep `wp_fsdb_probe`, `wp_fsdb_open`, `wp_fsdb_close`, and `wp_fsdb_read_metadata` working as they do today.
- Add a hierarchy traversal function, for example `wp_fsdb_read_scope_var_tree(wp_fsdb_reader *reader, wp_fsdb_tree_callback callback, void *user, char **error_message)`.
- Optionally add `wp_fsdb_read_scope_tree` if the separate scope tree path is straightforward, but command parity must not depend on this optional optimization. A full scope+var tree traversal is acceptable for `scope` because it does not load value changes.
- In C++, call `ffrHasScopeTree()` and `ffrReadScopeTree2` only if implementing a scope-only optimization. For the shared index, call `ffrReadScopeVarTree2` or the equivalent `ffrSetTreeCBFunc` + `ffrReadScopeVarTree` path.
- Handle tree events equivalent to begin-tree, scope, variable, upscope, end-tree, and end-all-tree. Ignore value-change, transaction, analog, power, coverage, and other non-hierarchy events.
- Before or during hierarchy traversal, collect datatype records needed for signal kind priority, especially enum datatypes. Read all datatype definition blocks, not just block index `0`. Prefer the SDK’s documented block count or iterator if available in the local headers. If the API is only block-index based, implement a bounded sequential loop starting at `0`, stop only on the documented normal end/no-data result, and treat any other return code as a Reader error. Record the exact local API behavior and stopping condition in this plan during implementation.
- Map proprietary FSDB scope and variable constants to project-owned raw enum variants in C++, not in Rust. Unknown, VHDL-specific, analog, transaction, power, property, coverage, and internal kinds should map to an `Unknown` raw variant unless they have an existing stable wavepeek alias.
- Copy strings on the Rust side during callbacks. Treat callback strings and buffers as borrowed and invalid after the callback returns.
- Do not let C++ exceptions or Rust panics cross the FFI boundary. C++ catches exceptions and returns status plus owned error string. Every Rust `extern "C"` callback body must wrap builder work in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`, store either the `WavepeekError` or a panic marker in the callback context, return non-zero to abort traversal, and avoid unwinding into C++. After the native traversal returns, `FsdbReader::read_hierarchy()` must prefer the stored Rust callback failure over a generic native callback-aborted error.
- Keep native stdout/stderr suppression around Reader calls, including the new traversal functions, unless `WAVEPEEK_FSDB_NATIVE_VERBOSE=1` is set. Because suppression redirects process file descriptors, serialize all suppressing FSDB Reader calls with a native `std::mutex` or a Rust-side global `Mutex`, including probe, open, metadata, hierarchy traversal, and close.

Extend `src/waveform/fsdb_native.rs` so `FsdbReader` and metadata are usable by `fsdb_backend`. The Rust wrapper should expose `pub(super)` methods like `probe(path)`, `FsdbReader::open(path)`, `FsdbReader::metadata()`, and `FsdbReader::read_hierarchy()`. The wrapper should translate native callback events into a `FsdbHierarchyIndex` by feeding `fsdb_hierarchy::FsdbHierarchyBuilder`. Keep unsafe blocks small and explain ownership at each FFI boundary. Native failures should become `WavepeekError::File` messages beginning with `FSDB Reader`.

Add or update feature-gated library tests in `src/waveform/fsdb_native.rs`. Keep the existing `fsdb_reader_metadata_smoke`; add a hierarchy smoke that opens `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`, traverses hierarchy, and asserts structural properties only: non-empty scopes, stable aliases, no empty path, dot-separated public paths, and deterministic second traversal. Do not assert a full list of vendor design paths. Do not try to prove native stdout/stderr cleanliness from this in-process library test; the CLI integration tests capture child stdout/stderr and are the right place for that contract.

Acceptance for this milestone on a Verdi-equipped machine:

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env
    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" cargo check --features fsdb
    VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture
    VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

On a no-Verdi machine, default `cargo test` must still compile and run without trying to include Verdi headers.

### Add `FsdbBackend` and wire feature-enabled opening

Create `src/waveform/fsdb_backend.rs` behind `#[cfg(feature = "fsdb")]`. `FsdbBackend` should own a `fsdb_native::FsdbReader` and a lazy hierarchy index. Do not require normalized metadata during `FsdbBackend::open`; either read and normalize metadata lazily in `metadata(&self)` or cache a recoverable raw/native metadata result that does not make `scope` and `signal` fail before they need timestamps. Use `RefCell<Option<FsdbHierarchyIndex>>` or an equivalent single-threaded interior mutability pattern so `metadata(&self)` does not build hierarchy but `scopes_depth_first(&self, ...)` and `signals_in_scope(&self, ...)` can initialize the index on demand. Avoid `static mut`, global caches, or process-wide mutable state except for the explicit native-call mutex described in the shim milestone.

Update `src/waveform/mod.rs` as follows:

- Add `#[cfg(feature = "fsdb")] mod fsdb_backend;`.
- Extend `enum Backend` with `#[cfg(feature = "fsdb")] Fsdb(fsdb_backend::FsdbBackend)`.
- Change `Waveform::open(path)` so feature-enabled builds can open FSDB files while preserving existing VCD/FST behavior. Use one explicit precedence rule: a positive FSDB probe, `Ok(true)`, opens `FsdbBackend` immediately; only `Ok(false)` or `Err(_)` falls through to Wellen. If the path has an FSDB-looking suffix, probe first. A valid VCD/FST file with a misleading `.fsdb` suffix still works because the probe should return `Ok(false)` and Wellen then parses it. If Wellen fails on an existing regular file, probe when no earlier positive/negative probe result exists. Only a positive `Ok(true)` probe may override the Wellen parse failure for non-FSDB-looking paths. For FSDB-looking paths where Wellen also fails, return `Ok(false)` as the original Wellen parse error and return a probe `Err` as an FSDB Reader file error. Preserve ordinary missing-file, permission, and directory errors by not probing paths that are not existing regular files.
- Keep the default no-feature branch using `fsdb_disabled::should_report_disabled_support` after Wellen parse failure.
- Change `Waveform::scopes_depth_first` to return `Result<Vec<ScopeEntry>, WavepeekError>` and update `src/engine/scope.rs` plus any unit tests to use `?` or `.expect(...)` as appropriate.
- Delegate `metadata`, `scopes_depth_first`, `signals_in_scope`, and `signals_in_scope_recursive` to `FsdbBackend` for the FSDB variant.
- For `resolve_signals`, `resolve_expr_signal`, `resolve_expr_signals`, `sample_resolved_optional`, `sample_expr_value`, `expr_event_occurred`, `collect_change_times_with_mode`, and `collect_expr_candidate_times_with_mode`, implement the minimum behavior needed for clear command outcomes. `resolve_signals` and expression resolution may use the hierarchy index to validate paths and return backend-neutral records, because later command paths resolve before sampling. Sampling and event methods must return explicit unsupported-operation errors for FSDB. Use command-specific text where practical, such as `FSDB value sampling is not implemented yet; info, scope, and signal are supported by this build`, `FSDB change collection is not implemented yet; info, scope, and signal are supported by this build`, and `FSDB property evaluation is not implemented yet; info, scope, and signal are supported by this build`. Optimized indexed capability methods should return `None` or `false` for FSDB, matching the optional capability boundary from the backend refactor, but they must not let `change` or `property` accidentally succeed with empty output.

`FsdbBackend::metadata` should convert native metadata with `fsdb_time`: preserve the normalized time unit string, render start and end as `<raw * factor><suffix>`, and reject unsupported floating/double xtag representations through a `WavepeekError::File` whose message includes `unsupported FSDB time tag representation`. Existing `wp_fsdb_read_metadata` already rejects non-integer xtags; keep that behavior or expose enough xtag information for Rust to reject explicitly. Do not silently round fractional time units.

`FsdbBackend::signals_in_scope` and `signals_in_scope_recursive` should use the hierarchy index, not native traversal each time. They must match Wellen behavior for missing scopes: `WavepeekError::Scope("scope '<scope>' not found in dump")`. They must return entries sorted by `(name, path)` within each visited scope and preserve duplicate requested output only where higher command layers already preserve duplicates; `signal` listing itself should not emit duplicate paths from repeated FSDB tree sections.

Acceptance for this milestone:

    cargo fmt -- --check
    cargo check
    cargo test -q waveform
    cargo test -q --test scope_cli --test signal_cli --test info_cli

With Verdi available, also run:

    verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
    VERDI_HOME="$verdi_home" cargo check --features fsdb
    VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture
    VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

The default tests prove VCD/FST did not regress; the feature-enabled checks prove the new code compiles and opens the bundled FSDB.

### Add feature-gated FSDB CLI integration tests and make targets

Add a `scripts/prepare_fsdb_fixtures.sh` helper and a `prepare-fsdb-fixtures` Makefile target before adding the feature-gated CLI tests. The target depends on `require-verdi`; the helper validates that `vcd2fsdb` is available, walks every `*.vcd` file under `tests/fixtures/hand/`, and generates ignored `.fsdb` files under `tests/fixtures/fsdb/`. It writes through temporary files and captures converter stdout/stderr unless conversion fails. Keep the target intentionally boring: no manifest, no checked-in generated files, and no conversion of FST fixtures in this slice. The existing `.gitignore` already ignores `*.fsdb`, `*.fsdb.*`, and `*Log/`, so generated FSDB files and Verdi converter logs stay local.

Add `tests/fsdb_cli.rs` with `#![cfg(feature = "fsdb")]`. Reuse `tests/common::wavepeek_cmd`. Feature-enabled Cargo commands still require a usable SDK because `build.rs` runs before test code. The test file should generate any VCD-derived FSDB inputs in temporary directories so direct `cargo test --features fsdb --test fsdb_cli` is useful during iteration and does not depend on persistent ignored files. The normal supported path remains `make test-fsdb`, which also exercises `make prepare-fsdb-fixtures` before running the Cargo test.

The integration tests must not store full golden output from FSDB files. They should use two complementary layers: exact checks for generated local fixtures that are small enough to have stable expected payloads, and structural smoke for the bundled Verdi example.

Generated-fixture tests should create temporary FSDB files from `scope_mixed_kinds.vcd` and `signal_recursive_depth.vcd` with `vcd2fsdb` running in the temp directory and stdout/stderr captured. Assert exact `info`, `scope`, and `signal` payloads for those fixtures, including depth bounding, task/function aliases, direct signal listing, recursive signal listing, missing scope error category, and the “VCD with `.fsdb` suffix still uses Wellen” fallback. If a generated FSDB differs from its source VCD, treat that as a real bug or an explicit converter limitation that must be recorded in `Surprises & Discoveries`; do not silently weaken parity into vague non-empty checks.

Bundled-example smoke tests should still use `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`, because it was produced outside this repository and can expose Reader behavior that generated VCD-derived fixtures miss:

- `fsdb_info_json_uses_existing_contract`: run `wavepeek info --waves <cpu.fsdb> --json`; assert exit success, empty stderr, command `info`, empty warnings, `$schema` equals `expected_schema_url()`, `time_unit`, `time_start`, and `time_end` are non-empty strings, and `time_precision` is absent. Run the command twice and assert identical stdout bytes.
- `fsdb_scope_json_is_sorted_and_bounded`: run `wavepeek scope --waves <cpu.fsdb> --max unlimited --json`; assert entries are non-empty, each entry has only existing fields, every kind is in the public `scopeKind` enum from `schema/wavepeek.json` or equals `unknown`, paths are dot-separated and non-empty, depths are non-negative, order is deterministic across two runs, `--max 1` truncates and emits the existing truncation warning, and `--max-depth 0` returns only depth-zero entries.
- `fsdb_signal_json_is_sorted_and_stable`: dynamically choose a scope whose recursive signal query is non-empty. Run `wavepeek signal --waves <cpu.fsdb> --scope <scope> --recursive --max unlimited --json`; assert command `signal`, output is non-empty, paths remain dot-separated, kinds are in the public `signalKind` enum from `schema/wavepeek.json`, width is either absent or a positive integer, no signal name has a trailing packed range that exactly duplicates its width metadata, and the `(name, path)` order is deterministic for signals within the same scope traversal. Reuse one discovered signal from this test for unsupported later-command checks.
- `fsdb_scope_missing_uses_scope_error`: run `wavepeek signal --waves <cpu.fsdb> --scope __wavepeek_missing_scope__`; assert exit code `1`, empty stdout, stderr starts with `error: scope: scope '__wavepeek_missing_scope__' not found in dump`.
- `fsdb_later_commands_fail_clearly`: using the dynamically discovered scope and signal, run a minimal `value` command and a minimal `change` command and assert non-success errors containing the command-specific unsupported FSDB text. Also run a minimal `property` command using a discovered signal whose name is a simple expression identifier; if no such signal exists in the bundled example, fail with an explanatory message and record the fixture limitation rather than silently making the test vacuous.

Update `Makefile` so `check-fsdb-build` remains the feature-enabled build/native-smoke gate, `test-fsdb` depends on both `check-fsdb-build` and `prepare-fsdb-fixtures`, and `test-fsdb` runs the new feature-gated CLI integration test. Also add a gated `lint-fsdb` target so feature-enabled clippy uses the same resolved Verdi environment as the build smoke. A reasonable shape is:

    check-fsdb-build: require-verdi
        @verdi_home="$$(.devcontainer/resolve_verdi_home.sh)"; \
        if [ -z "$$verdi_home" ]; then \
            printf '%s\n' "error: fsdb: environment checker succeeded but no usable VERDI_HOME could be resolved" >&2; \
            exit 1; \
        fi; \
        VERDI_HOME="$$verdi_home" cargo check --features fsdb && \
        VERDI_HOME="$$verdi_home" cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture && \
        VERDI_HOME="$$verdi_home" cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

    test-fsdb: check-fsdb-build prepare-fsdb-fixtures
        @verdi_home="$$(.devcontainer/resolve_verdi_home.sh)"; \
        VERDI_HOME="$$verdi_home" cargo test --features fsdb --test fsdb_cli

    lint-fsdb: require-verdi
        @verdi_home="$$(.devcontainer/resolve_verdi_home.sh)"; \
        VERDI_HOME="$$verdi_home" cargo clippy --all-targets --features fsdb -- -D warnings

Keep default `make check`, `make ci`, and pre-commit Verdi-free.

Update `docs/DEVELOPMENT.md` because Makefile target semantics change. A short note is enough: `make prepare-fsdb-fixtures` generates ignored FSDB files from checked-in VCD fixtures under `tests/fixtures/hand/`; `make check-fsdb-build` verifies feature-enabled build/link and native smokes; `make lint-fsdb` runs feature-enabled clippy; `make test-fsdb` prepares generated fixtures and verifies Reader-backed `info`, `scope`, and `signal` through generated parity tests and the bundled Verdi example. Do not update public docs to promise full FSDB command support until `value`, `change`, and `property` are also implemented or the public wording is explicitly scoped.

Acceptance for this milestone:

    WAVEPEEK_IN_CONTAINER=1 make test-aux
    WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb
    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb

### Final validation, review, and handoff

Run the full default validation and the Verdi-gated validation. Then update this plan with exact outcomes, review findings, and any deviations. The implementation should not move this plan to `completed/` until the code is implemented, reviewed, committed, and the user or maintainer approves closure.

Required final commands from `/workspaces/wavepeek-fsdb`:

    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci
    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env

If Verdi is available:

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-build
    WAVEPEEK_IN_CONTAINER=1 make test-fsdb
    WAVEPEEK_IN_CONTAINER=1 make lint-fsdb

Also verify schema stability:

    WAVEPEEK_IN_CONTAINER=1 make update-schema
    git diff -- schema/wavepeek.json

If `schema/wavepeek.json` changes, stop and explain why. The expected result for this plan is no schema diff because FSDB uses the existing command payloads. Revert accidental schema churn unless a separate schema decision is recorded.

Run focused read-only reviews before final commit. Suggested lanes are native/FFI safety, Rust backend correctness, CLI contract/testing, and licensing/proprietary-payload hygiene. Fix substantive findings, re-run impacted tests, and then run one fresh independent control review on the consolidated diff.

Final acceptance is:

- default `make ci` passes without Verdi;
- feature-enabled `make check-fsdb-build`, `make test-fsdb`, and `make lint-fsdb` pass on a Verdi-equipped Linux x86_64 machine;
- generated adjacent FSDB fixtures under `tests/fixtures/hand/` match their VCD sources for `info`, `scope`, and `signal` JSON contracts;
- `info`, `scope`, and `signal` on the bundled FSDB example produce valid deterministic output through existing schemas;
- VCD/FST `info`, `scope`, and `signal` tests still pass;
- `value` on FSDB fails clearly until its implementation slice;
- no `.fsdb`, Verdi header/library/doc, generated binding, or proprietary golden output is tracked by Git;
- no new repository entity uses a milestone-labelled name.

## Plan of Work

Implement this change in small commits if possible, keeping each commit buildable. Start with pure Rust helpers and tests because they are cheap, deterministic, and Verdi-free. Then extend the native shim and feature-gated wrapper. Then wire `FsdbBackend` into `Waveform::open`. Finally add CLI integration tests and Makefile target coverage. This order keeps the cabinet from being rewired while the power is still on; exciting, yes, but not a maintenance strategy.

When editing source files, keep visibility narrow. Most new Rust types should be private or `pub(super)`. The only engine-facing additions should be the existing `Waveform` methods and existing backend-neutral types. If a helper is public only for tests, prefer `#[cfg(test)]` or a test-only constructor over broad crate visibility.

Avoid broad rewrites of Wellen code. The only Wellen-facing change expected is adapting `scopes_depth_first` to return `Ok(...)` and updating tests/callers for the new `Result` shape. Any unrelated refactor belongs in a separate plan or should be justified in `Decision Log`.

## Concrete Steps

1. Confirm no unrelated changes:

       git status --short --branch

2. Add `fsdb_time` and `fsdb_hierarchy` modules with no-Verdi unit tests. Update `src/waveform/mod.rs` module declarations with `#[cfg(any(test, feature = "fsdb"))]` for pure helpers. Run:

       cargo fmt -- --check
       cargo test -q fsdb_time
       cargo test -q fsdb_hierarchy

3. Extend the native C ABI and Rust `fsdb_native` wrapper. Run feature-enabled compile and native smoke on a Verdi machine:

       verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
       VERDI_HOME="$verdi_home" cargo check --features fsdb
       VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture
       VERDI_HOME="$verdi_home" cargo test --features fsdb --lib fsdb_reader_hierarchy_smoke -- --nocapture

4. Add `fsdb_backend`, wire `Backend::Fsdb`, update `Waveform::open`, and change `scopes_depth_first` to return `Result`. Update `src/engine/scope.rs` and Wellen tests. Run:

       cargo check
       cargo test -q waveform
       cargo test -q --test info_cli --test scope_cli --test signal_cli
       verdi_home="$(.devcontainer/resolve_verdi_home.sh)"
       VERDI_HOME="$verdi_home" cargo check --features fsdb

5. Add `prepare-fsdb-fixtures`, `tests/fsdb_cli.rs`, and updated FSDB Makefile targets, then run:

       WAVEPEEK_IN_CONTAINER=1 make test-aux
       WAVEPEEK_IN_CONTAINER=1 make prepare-fsdb-fixtures
       WAVEPEEK_IN_CONTAINER=1 make test-fsdb
       WAVEPEEK_IN_CONTAINER=1 make lint-fsdb

6. Run final gates:

       WAVEPEEK_IN_CONTAINER=1 make check
       WAVEPEEK_IN_CONTAINER=1 make ci
       WAVEPEEK_IN_CONTAINER=1 make update-schema
       git diff -- schema/wavepeek.json
       git status --short

Expected schema result is no diff. Expected `git status --short` should show only intentional source, test, Makefile/docs, and plan changes. Ignored generated `.fsdb` files may remain beside VCD fixtures after FSDB testing, but no `.fsdb` file may be tracked or staged. If generated FSDB artifacts show up in normal non-ignored status, remove them before review.

## Validation and Acceptance

Default-build validation proves public portability. `make ci` must pass with no Verdi SDK and without the `fsdb` feature. `tests/fsdb_disabled_cli.rs` must still pass, proving default `.fsdb` diagnostics remain intact. `tests/info_cli.rs`, `tests/scope_cli.rs`, and `tests/signal_cli.rs` must still pass for VCD/FST.

Pure helper validation proves important FSDB normalization without proprietary inputs. The default unit tests for `fsdb_time` and `fsdb_hierarchy` must cover edge cases that the bundled Verdi example may not contain: fractional scale units, overflow, hidden scope exclusion, duplicates, packed ranges, missing scopes/signals, and datatype kind priority.

Feature-enabled validation proves both parity and the real Reader path. On a Verdi-equipped Linux x86_64 machine, `make prepare-fsdb-fixtures` must generate ignored `.fsdb` files from the selected VCD fixtures under `tests/fixtures/hand/` without committing them. `make test-fsdb` must compile the native shim, run metadata and hierarchy smoke tests through its `check-fsdb-build` dependency, exercise the fixture preparation script, run generated-fixture parity checks that create temporary FSDB files for the test process, and run structural checks against `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`. Generated-fixture tests should compare exact command payloads against the VCD source; bundled-example tests should compare deterministic structure, not proprietary full golden output.

Manual spot checks are useful during implementation. With `VERDI_HOME` set, run:

    cargo run --features fsdb -- info --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --json
    cargo run --features fsdb -- scope --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --max 5 --json
    first_scope="$(cargo run --features fsdb -- scope --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --max 1 --json | jq -r '.data[0].path')"
    cargo run --features fsdb -- signal --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --scope "$first_scope" --recursive --max 5 --json

Expected observations: all three supported commands exit `0`; stderr is empty except for intentional warnings from limit options; JSON envelopes use the existing `$schema`; paths are dot-separated; kinds are existing stable aliases; repeated identical commands produce identical stdout bytes. The `jq` command is only for local convenience; tests should parse JSON in Rust with `serde_json`.

Unsupported command validation prevents accidental lies. Run `value`, `change`, and `property` commands on dynamically discovered FSDB signals where the test can construct valid arguments, and expect clear non-success errors containing the command-specific unsupported FSDB text. Do not accept a panic, an internal unwrap, an empty success payload, or a Wellen parse error after `FsdbBackend` has opened the file.

Licensing validation is a Git check plus common sense, which is regrettably not automated enough:

    git status --short
    git ls-files | rg '\.(fsdb|so|a|dylib|dll)$|(^|/)ffrAPI\.h$|(^|/)ffrKit\.h$|(^|/)fsdbShr\.h$' && echo "unexpected proprietary artifact" && false || true

The expected result is no tracked FSDB files, libraries, Verdi headers, generated bindings, or native dumps.

## Idempotence and Recovery

All build and test commands are safe to rerun. The native build output stays under `target/`. `make prepare-fsdb-fixtures` is idempotent: it regenerates a sibling `tests/fixtures/hand/<name>.fsdb` only when that file is missing or older than its source VCD. These generated FSDB files are ignored by Git and may be left in place for faster local reruns. Remove them with `find tests/fixtures/hand -type f -name '*.fsdb*' -delete` if you need a clean regeneration. Verdi converter logs such as `vcd2fsdbLog/` are also ignored; remove them whenever they stop being useful. Temporary logs, local command captures, and scratch comparison output should go under repository-root `tmp/`, which is ignored by Git.

If a feature-enabled build fails because Verdi is unavailable, first run `WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env`. If it prints the skip message, do not mark FSDB validation complete; record the limitation and run default validation. If `make check-fsdb-env` succeeds but `cargo check --features fsdb` fails, the issue is in the native shim/build integration and must be fixed before handoff.

If native hierarchy traversal crashes the test process, disable only the newly added traversal call while preserving the existing metadata smoke, record the crash in `Surprises & Discoveries`, and reduce the native callback surface until the failure is isolated. Do not leave a flaky segfaulting test enabled with a shrug. Shrugs do not make good guardrails.

If `make update-schema` changes `schema/wavepeek.json`, inspect the diff. This plan should not change schema. Revert accidental schema output changes unless a deliberate schema decision has been added to this plan and reviewed.

If coverage drops below the repository threshold, add focused tests for the new pure Rust helpers or backend error paths. Do not lower the threshold. The threshold is not a suggestion jar.

## Artifacts and Notes

Expected useful local artifacts during implementation:

- `tmp/fsdb-hierarchy-commands/default-ci.log` for final `make ci` output if a log is useful.
- `tmp/fsdb-hierarchy-commands/check-fsdb-build.log` and `tmp/fsdb-hierarchy-commands/test-fsdb.log` for final Verdi-gated validation.
- Generated sibling files such as `tests/fixtures/hand/m2_core.fsdb`, produced by `make prepare-fsdb-fixtures`. They are local ignored artifacts and must remain untracked.
- `tmp/fsdb-hierarchy-commands/fsdb-info.json`, `fsdb-scope.json`, and `fsdb-signal.json` only as disposable local captures. Do not commit these files.

Example supported command transcript shape, with actual data intentionally elided:

    $ cargo run --features fsdb -- info --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --json
    {"$schema":"https://raw.githubusercontent.com/kleverhq/wavepeek/v.../schema/wavepeek.json","command":"info","data":{"time_unit":"...","time_start":"...","time_end":"..."},"warnings":[]}

Example unsupported command transcript shape:

    $ cargo run --features fsdb -- value --waves "$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb" --scope <scope> --signals <signal> --at 0ns
    error: signal: FSDB value sampling is not implemented yet; info, scope, and signal are supported by this build

The exact unsupported category may be `signal` or `file` depending on where the error is raised, but it must be deterministic and must not be `internal` for normal user input.

## Interfaces and Dependencies

At the end of this implementation, `src/waveform/mod.rs` should have this effective shape:

    #[cfg(feature = "fsdb")]
    mod fsdb_backend;
    #[cfg(any(test, feature = "fsdb"))]
    mod fsdb_hierarchy;
    #[cfg(feature = "fsdb")]
    mod fsdb_native;
    #[cfg(any(test, feature = "fsdb"))]
    mod fsdb_time;

    enum Backend {
        Wellen(Box<wellen_backend::WellenBackend>),
        #[cfg(feature = "fsdb")]
        Fsdb(Box<fsdb_backend::FsdbBackend>),
    }

    impl Waveform {
        pub fn open(path: &Path) -> Result<Self, WavepeekError>;
        pub fn metadata(&self) -> Result<WaveformMetadata, WavepeekError>;
        pub fn scopes_depth_first(&self, max_depth: Option<usize>) -> Result<Vec<ScopeEntry>, WavepeekError>;
        pub fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>;
    }

`src/waveform/fsdb_backend.rs` should define a private or `pub(super)` backend roughly like:

    pub(super) struct FsdbBackend {
        reader: fsdb_native::FsdbReader,
        hierarchy: RefCell<Option<FsdbHierarchyIndex>>,
        // Optional: metadata_cache may store a normalized success or a recoverable error string,
        // but open must not require metadata normalization.
    }

    impl FsdbBackend {
        pub(super) fn open(path: &Path) -> Result<Self, WavepeekError>;
        pub(super) fn metadata(&self) -> Result<WaveformMetadata, WavepeekError>;
        pub(super) fn scopes_depth_first(&self, max_depth: Option<usize>) -> Result<Vec<ScopeEntry>, WavepeekError>;
        pub(super) fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub(super) fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>;
    }

`src/waveform/fsdb_time.rs` should expose only crate-internal helpers needed by the backend, for example:

    pub(super) struct FsdbTimeUnit {
        pub(super) factor: u64,
        pub(super) suffix: &'static str,
    }

    pub(super) fn parse_scale_unit(raw: &str) -> Result<FsdbTimeUnit, WavepeekError>;
    pub(super) fn normalize_raw_time(raw: u64, unit: FsdbTimeUnit) -> Result<String, WavepeekError>;
    pub(super) fn normalize_time_unit(raw: &str) -> Result<String, WavepeekError>;

`src/waveform/fsdb_hierarchy.rs` should expose a builder and immutable index to the backend, not to engine modules. A workable shape is:

    pub(super) struct FsdbHierarchyBuilder { /* private */ }
    pub(super) struct FsdbHierarchyIndex { /* private */ }

    impl FsdbHierarchyBuilder {
        pub(super) fn new() -> Self;
        pub(super) fn begin_tree(&mut self);
        pub(super) fn scope(&mut self, record: RawScopeRecord) -> Result<(), WavepeekError>;
        pub(super) fn signal(&mut self, record: RawSignalRecord) -> Result<(), WavepeekError>;
        pub(super) fn datatype(&mut self, record: RawDatatypeRecord) -> Result<(), WavepeekError>;
        pub(super) fn upscope(&mut self) -> Result<(), WavepeekError>;
        pub(super) fn end_tree(&mut self);
        pub(super) fn finish(self) -> FsdbHierarchyIndex;
    }

    impl FsdbHierarchyIndex {
        pub(super) fn scopes_depth_first(&self, max_depth: Option<usize>) -> Vec<ScopeEntry>;
        pub(super) fn signals_in_scope(&self, scope_path: &str) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub(super) fn signals_in_scope_recursive(&self, scope_path: &str, max_depth: Option<usize>) -> Result<Vec<SignalEntry>, WavepeekError>;
        pub(super) fn resolve_signal(&self, canonical_path: &str) -> Result<ResolvedSignal, WavepeekError>;
    }

The native header should remain a C ABI, not a Rust ABI wearing a false mustache. It should define opaque reader ownership, project-owned records, callback signatures, and free functions. Rust owns conversion to existing `Waveform` types. C++ owns interaction with `ffrAPI.h` and suppresses native output. Neither side should assume callback string lifetimes survive callback return.

The `Makefile` should expose these FSDB-specific targets at the end of this plan: `check-fsdb-env` for optional environment detection, `check-fsdb-build` for feature-enabled build and native smoke tests, `prepare-fsdb-fixtures` for adjacent VCD-to-FSDB generation under `tests/fixtures/hand/`, `test-fsdb` for generated parity plus bundled-example CLI tests, and `lint-fsdb` for feature-enabled clippy. Only `check-fsdb-env` may be safe on no-Verdi machines; the other FSDB targets must depend directly or indirectly on `require-verdi`.

The implementation depends on the existing `cc` build dependency, local Verdi Reader SDK discovered by `build.rs`, and the Verdi `vcd2fsdb` converter available through the resolved `VERDI_HOME` for generated fixtures. Do not add `bindgen`, `anyhow`, a runtime helper process, network downloads, or new fixture mount variables for this milestone.
