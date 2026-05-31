# Implement FSDB feature-gated build and metadata smoke

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill. All new repository entities named by this plan intentionally use descriptive FSDB/build names rather than milestone labels.

## Purpose / Big Picture

After this change, a normal `wavepeek` checkout still builds, lints, and tests without Synopsys Verdi installed, while a developer who explicitly asks for FSDB support gets an immediate, deterministic build-time check for a local FSDB Reader SDK. On a machine with a licensed Verdi installation, `cargo build --features fsdb` compiles and links a tiny project-owned C++ shim against the local FSDB Reader libraries; a focused smoke test opens the bundled `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` example and reads basic time metadata through the shim.

This matters because FSDB is a proprietary binary waveform format. The public repository must not carry Verdi headers, libraries, manuals, generated bindings, or FSDB fixtures, but it still needs a safe build seam before later work can add the full backend. A contributor can see the result working in two ways: public/no-Verdi environments keep passing `make ci`, and Verdi-equipped environments can run `make check-fsdb-build` to prove the feature-gated native link and metadata path are real rather than decorative wiring. Decorative wiring is where bugs go to become load-bearing, so we do not do that.

## Non-Goals

This plan does not make `wavepeek info --waves dump.fsdb` work through the normal CLI. The user-facing FSDB-disabled error and CLI format detection belong to a later FSDB-disabled UX slice.

This plan does not refactor `src/waveform/mod.rs` into backend-neutral Wellen and FSDB backends. That larger boundary move is a later backend-refactor slice.

This plan does not add hierarchy, signal listing, value sampling, change, or property support for FSDB. The native shim created here opens an FSDB file and reads metadata only.

This plan does not vendor, quote, or generate source from Synopsys proprietary material. Project-owned C++ may include local SDK headers at build time, but committed files must not contain copied header definitions, library binaries, PDF excerpts, Verdi examples, `.fsdb` fixtures, or generated bindings derived from proprietary headers.

This plan does not publish FSDB-enabled release binaries. Public builds remain default-feature VCD/FST binaries.

## Progress

- [x] (2026-05-21 08:10Z) Read `docs/fsdb/arch.md`, `docs/fsdb/verdi_home_map.md`, the command-specific FSDB API notes, `Cargo.toml`, `Makefile`, `.github/workflows/ci.yml`, `.pre-commit-config.yaml`, `docs/DEVELOPMENT.md`, and the current `src/waveform/mod.rs` facade.
- [x] (2026-05-21 08:20Z) Drafted this active ExecPlan under `docs/exec-plans/active/2026-05-21-fsdb-build-spike/PLAN.md` with descriptive entity names and no milestone-label prefixes or suffixes for new files, modules, functions, tests, or make targets.
- [x] (2026-05-21 08:45Z) Ran parallel review lanes for plan quality, build architecture, and licensing/source hygiene; revised the plan to clarify `VERDI_HOME` versus library-directory overrides, probe return shape, C++ compilation flags, and mandatory default validation.
- [x] (2026-05-21 08:55Z) Ran a fresh final independent control review on the revised plan; it returned no substantive findings.
- [x] (2026-05-21 09:05Z) Added the feature-gated Cargo/build-script foundation and default-feature automation changes: `fsdb` is a non-default feature, `build.rs` is inert without the feature, and default lint/coverage no longer use `--all-features`.
- [x] (2026-05-21 09:05Z) Added environment discovery helpers for local Verdi and bundled Verdi FSDB examples: `.devcontainer/resolve_verdi_home.sh`, `scripts/check_fsdb_env.py`, and deterministic no-Verdi Python tests.
- [x] (2026-05-21 09:05Z) Added the project-owned C++ metadata shim and Rust FFI smoke wrapper: `native/fsdb/wavepeek_fsdb_shim.{h,cpp}` and `src/waveform/fsdb_native.rs` are private to the `fsdb` feature and are not wired into the CLI.
- [x] (2026-05-21 09:05Z) Added focused tests and make targets that keep default automation Verdi-free while proving build/link/metadata behavior when Verdi is available. Local validation so far: `make format`, `python3 -m unittest scripts/test_check_fsdb_env.py`, `make check-fsdb-env`, `make check-build`, and `make check-fsdb-build` on a Verdi-equipped container.
- [x] (2026-05-21 09:25Z) Ran full default validation and targeted FSDB validation: `make ci`, `make check-fsdb-build`, simulated no-Verdi availability probing with an empty temporary `VERDI_HOME`, direct missing-`VERDI_HOME` `cargo check --features fsdb` failure, proprietary-payload search, and the existing VCD `wavepeek info --waves ... --json` smoke.
- [x] (2026-05-21 09:45Z) Ran first focused implementation review lanes and applied fixes: propagated native link metadata while keeping package-local `--no-as-needed`, made the shim metadata string allocation RAII-safe, changed FSDB environment success output to be path-free by default, documented the current `fsdb` feature as repository-local, and corrected/sanitized living-plan text.
- [x] (2026-05-21 10:00Z) Ran targeted re-review. Build/link re-review returned no substantive findings; docs/licensing re-review found one remaining low-severity path-leakage concern in explicit `scripts/check_fsdb_env.py` error output, which was fixed by making default missing-header/library errors component-only while preserving full paths behind `WAVEPEEK_FSDB_ENV_VERBOSE=1`.
- [x] (2026-05-21 10:15Z) Ran second targeted re-review on the environment-checker path-leakage fix; it returned no substantive findings.
- [x] (2026-05-21 10:20Z) Ran final independent control review on `4892aa2..HEAD`; it returned no substantive findings. At that point the plan remained in `active/` for user inspection as requested.
- [x] (2026-05-21 10:35Z) After user follow-up, searched `$VERDI_HOME` by filename only and found bundled FSDB example files. Ran `make check-fsdb-build` against the concrete `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` smoke fixture; the real-file metadata smoke passed, but the native Reader emitted `logDir = /tmp/`, so the shim now suppresses native stdout/stderr around FSDB Reader calls by default.
- [x] (2026-05-21 10:50Z) Re-ran validation after the real-file smoke/output-suppression fix: `make ci`, real-file `make check-fsdb-build`, feature-enabled `cargo clippy`, and simulated no-Verdi availability probing passed. A targeted review found that `wp_fsdb_close` also needed the scoped suppressor; the close path was fixed and the real-file smoke plus feature-enabled clippy still passed.
- [x] (2026-05-21 10:55Z) Ran final control review after the close-path suppression fix. It found one stale Decision Log sentence that omitted `wp_fsdb_close`; the sentence was updated, and no code findings remained.
- [x] (2026-05-21 11:20Z) Applied follow-up environment policy changes: removed hard-coded Verdi fallback paths from scripts, added a Makefile `require-verdi` gate for explicit FSDB build targets, kept `check-fsdb-env` as the non-failing availability probe, and updated tests/docs accordingly. Validation after the change: `python3 -m unittest scripts/test_check_fsdb_env.py`, `make check-fsdb-env`, `make require-verdi`, `make check-fsdb-build`, simulated no-Verdi `make check-fsdb-env`, simulated no-Verdi `make require-verdi` failure, and `make ci`. A targeted review of the Verdi gate follow-up returned no substantive findings.
- [x] (2026-05-21 11:40Z) Simplified FSDB smoke selection. The Rust smoke test now uses the concrete `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` path directly, the environment checker only validates the SDK, and targeted review returned no substantive findings. Validation: `cargo fmt -- --check`, `python3 -m unittest scripts/test_check_fsdb_env.py`, `make check-fsdb-env`, `make check-fsdb-build`, simulated no-Verdi `make check-fsdb-env`, simulated no-Verdi `make require-verdi`, `make test-aux`, `make ci`, and feature-enabled clippy.
- [x] (2026-05-21 12:00Z) Moved this plan from `docs/exec-plans/active/2026-05-21-fsdb-build-spike/PLAN.md` to `docs/exec-plans/completed/2026-05-21-fsdb-build-spike/PLAN.md` after the user approved closing the inspection loop, and added a reference to the completed plan in the M0 section of `docs/fsdb/arch.md`.

## Surprises & Discoveries

- Observation: before this implementation, the `make lint`, `make lint-fix`, and `make coverage-src` targets used `--all-features`.
  Evidence: the pre-change `Makefile` ran `cargo clippy --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features --fix ...`, and `cargo llvm-cov --workspace --all-features ...`. The implemented `Makefile` now uses default features for those public/default gates.

- Observation: public CI calls only `make ci`, so keeping `make ci` Verdi-free is the important public safety property.
  Evidence: `.github/workflows/ci.yml` runs `runCmd: make ci` inside `.devcontainer/devcontainer.ci.json`, which does not set `VERDI_HOME`.

- Observation: the devcontainer may mount `/opt/verdi` and set `VERDI_HOME`, but an empty mount must still count as unavailable.
  Evidence: `.devcontainer/devcontainer.json` sets `VERDI_HOME=/opt/verdi` and mounts `${HOME}/.cache/wavepeek/verdi` to `/opt/verdi`; `docs/fsdb/arch.md` says to treat Verdi as available only if the expected SDK files and libraries exist.

- Observation: the current Rust crate has no `build.rs`, no feature table, no native source directory, and no `cc` build dependency.
  Evidence: `Cargo.toml` has dependencies and dev-dependencies only, and `find . -maxdepth 2 -name build.rs` reports no build script.

- Observation: adding `cc` as a direct build dependency did not require refreshing all dependency versions because `cc` was already present transitively in `Cargo.lock`; only the root `wavepeek` dependency list needed the new `cc` entry.
  Evidence: after reverting an accidental broad `cargo generate-lockfile` refresh, the final `Cargo.lock` diff is one added `"cc"` line under the `wavepeek` package.

- Observation: linking `libnffr.so` and `libnsys.so` with ordinary `cargo:rustc-link-lib` caused `libnsys.so` to be dropped from the test binary on this toolchain, and the FSDB test executable then failed at dynamic-load time with an unresolved companion-library symbol from `libnffr.so`.
  Evidence: `ldd` on the feature-enabled test executable initially listed `libnffr.so` but not `libnsys.so`, and `cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture` exited 127 with a dynamic-loader `undefined symbol` error from `libnffr.so`.

- Observation: the local Verdi installation includes bundled example FSDB files that are useful for this build spike's real-file smoke test without adding repository fixtures.
  Evidence: filename-only search under `$VERDI_HOME` found example `.fsdb` files under `$VERDI_HOME/share/NPI`, `$VERDI_HOME/share/VIA`, and `$VERDI_HOME/share/verdi_perf`; no file contents were read.

- Observation: opening a bundled Verdi example FSDB through the Reader API initially printed `logDir = /tmp/` even though the shim called `ffrInfoSuppress(1)` and `ffrWarnSuppress(1)`.
  Evidence: `make check-fsdb-build` opened `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` and passed the Rust smoke test but initially included `logDir = /tmp/` in the output. After adding scoped native stdout/stderr suppression around FSDB Reader open, metadata, probe, and close calls, the same smoke command passed without that extra chatter.

- Observation: the existing waveform facade is still tightly coupled to Wellen handles.
  Evidence: `src/waveform/mod.rs` stores `simple::Waveform`, `wellen::FileFormat`, and `HashSet<SignalRef>`, and `ResolvedSignal` plus `ExprResolvedSignal` contain `wellen::SignalRef`. This plan must avoid pretending the backend boundary already exists.

## Decision Log

- Decision: name new entities after their purpose, such as `fsdb`, `check_fsdb_env.py`, `resolve_verdi_home.sh`, `check-fsdb-build`, `fsdb_native`, and `wavepeek_fsdb_shim`, rather than after milestone labels.
  Rationale: descriptive names survive beyond a planning phase and satisfy the explicit requirement that created entities not carry milestone-label prefixes or suffixes.
  Date/Author: 2026-05-21 / Grin

- Decision: add a Cargo feature named `fsdb` with no default membership, and make the build script do nothing unless that feature is enabled.
  Rationale: default `cargo build`, `cargo test`, `make check`, and `make ci` must remain portable and must not require a licensed Verdi installation.
  Date/Author: 2026-05-21 / Grin

- Decision: when `fsdb` is enabled, fail at build time if the target is not supported Linux or if `VERDI_HOME`/override variables do not resolve to a complete FSDB Reader SDK.
  Rationale: a user explicitly requesting FSDB support should get a clear installation error, not a binary that appears to support FSDB and falls over later. That smoking transformer is not a feature flag; it is a warning label.
  Date/Author: 2026-05-21 / Grin

- Decision: use a small project-owned C++ C-ABI shim and manual Rust FFI declarations, not `bindgen` and not direct Rust calls into C++ classes.
  Rationale: the FSDB Reader API is C++, direct class ABI from Rust is brittle, and generated bindings from proprietary headers must not be committed. A narrow C ABI also lets the shim own exception handling, string copying, and Reader warning suppression.
  Date/Author: 2026-05-21 / Grin

- Decision: keep the initial Rust wrapper private to the crate and test-focused; do not wire it into `Waveform::open` in this slice.
  Rationale: the current engine-facing `Waveform` type is still Wellen-specific. Connecting FSDB to CLI behavior before the backend-neutral refactor would spread temporary conditionals through production code and make the later refactor harder.
  Date/Author: 2026-05-21 / Grin

- Decision: replace default automation uses of `--all-features` with default-feature checks, and add explicit FSDB make targets for Verdi-equipped machines.
  Rationale: after the `fsdb` feature exists, `--all-features` means “requires Verdi.” Public CI and default hooks must not ask for proprietary SDKs by accident.
  Date/Author: 2026-05-21 / Grin

- Decision: make the feature-enabled linker invocation emit propagated `cargo:rustc-link-lib` metadata and also pass `libnffr.so` and `libnsys.so` as absolute linker arguments inside a temporary `--no-as-needed` block for this package's binaries and tests.
  Rationale: this Verdi installation's `libnffr.so` needs symbols from `libnsys.so`, but the default Rust/Cargo linker path dropped `libnsys.so` because the executable had no direct symbol reference to it. The propagated link metadata gives downstream builds the standard native-library information, while the package-local `--no-as-needed` block keeps both DSOs in the `NEEDED` set for the smoke test. This remains a development/build-smoke feature, not a supported downstream library contract.
  Date/Author: 2026-05-21 / Grin

- Decision: resolve Verdi only through `VERDI_HOME` in committed scripts, and make explicit FSDB build targets depend on a Makefile `require-verdi` gate.
  Rationale: the devcontainer may choose a conventional mount path, but helper scripts should not bake that path in as a second source of truth. `check-fsdb-env` remains a non-failing availability probe for ordinary no-Verdi machines, while `check-fsdb-build` and `test-fsdb` now fail clearly through `require-verdi` before Cargo tries a feature-enabled build. One tripwire is plenty; scattering hidden fallbacks is how maintenance gremlins reproduce.
  Date/Author: 2026-05-21 / Grin

- Decision: keep this plan in `docs/exec-plans/active/2026-05-21-fsdb-build-spike/PLAN.md` until the user inspected the result, then move it to `docs/exec-plans/completed/2026-05-21-fsdb-build-spike/PLAN.md` once they explicitly asked for closure.
  Rationale: the user initially asked to inspect the result before completion bookkeeping, then later requested the move. The plan therefore stayed active while review/follow-up work continued and moved only after the human finished rattling the cabinet door. Reasonable, if noisy.
  Date/Author: 2026-05-21 / Grin

- Decision: suppress native FSDB Reader stdout/stderr around `wp_fsdb_probe`, `wp_fsdb_open`, `wp_fsdb_read_metadata`, and `wp_fsdb_close` by default, with a local escape hatch `WAVEPEEK_FSDB_NATIVE_VERBOSE=1`.
  Rationale: the Reader can print process-global messages that are not disabled by its info/warning suppression knobs. The CLI and machine-output contracts require clean stdout/stderr; a scoped file-descriptor redirect is a blunt instrument, yes, but the alternative is letting a proprietary library scribble on our output like a raccoon with a marker.
  Date/Author: 2026-05-21 / Grin

## Outcomes & Retrospective

Current status: implementation, local validation, focused review, targeted re-review, and final control review are complete, including the follow-ups that made FSDB tests use the concrete Verdi-bundled `cpu.fsdb` fixture and added an explicit `require-verdi` Makefile gate. A real-file metadata smoke passes against that bundled example, and the shim suppresses native Reader chatter seen during that smoke across probe, open, metadata, and close calls. Default-feature `make ci` passes, `check-fsdb-env` is a non-failing availability probe, and explicit FSDB build targets now fail early without Verdi instead of carrying hidden site-path fallbacks.

The main risk remains ABI compatibility between the locally compiled shim and the Verdi-provided `libnffr.so`/companion libraries. The implementation already found one concrete runtime-link issue: `libnsys.so` must be kept in the executable's dynamic dependency set even though the Rust binary has no direct symbol reference to it. The current `build.rs` handles that with propagated native link metadata plus package-local absolute linker arguments and `--no-as-needed`, while still providing `WAVEPEEK_FSDB_READER_LIBDIR` and `WAVEPEEK_FSDB_ABI` overrides for other local Verdi layouts. First-round focused review found three low/medium issues, all fixed; targeted re-reviews and final control review returned no substantive findings.

## Context and Orientation

`wavepeek` is a Rust command-line tool. The main crate metadata lives in `Cargo.toml`, and the standard repository gates live in `Makefile`. The normal user-facing waveform reader today uses the Rust crate `wellen` for VCD and FST files. VCD is a text waveform format; FST is another waveform format supported by `wellen`; FSDB is a proprietary binary waveform format that must be read through a licensed local Synopsys Verdi FSDB Reader SDK.

The current waveform adapter is `src/waveform/mod.rs`. It defines `Waveform`, `WaveformMetadata`, `ScopeEntry`, `SignalEntry`, resolved-signal structs, sampling helpers, and expression-host hooks. It directly stores Wellen types such as `simple::Waveform`, `wellen::FileFormat`, and `SignalRef`. This plan intentionally does not change that boundary except to add a private `fsdb_native` module for build smoke tests, because the real backend-neutral refactor is a separate larger slice.

The temporary architecture proposal is `docs/fsdb/arch.md`. The relevant slice is the build spike section: add a feature-gated FSDB build, verify Linux-only build gating and default `linux64` Reader library selection, support `WAVEPEEK_FSDB_READER_LIBDIR` / `WAVEPEEK_FSDB_ABI` overrides, build a minimal shim that opens FSDB and reads metadata, verify clean failure without `VERDI_HOME` only when `--features fsdb` is enabled, and update public automation so no-Verdi gates do not build all features.

The local Verdi map is `docs/fsdb/verdi_home_map.md`. The relevant SDK root is `$VERDI_HOME/share/FsdbReader`. Expected headers are `ffrAPI.h`, `ffrKit.h`, and `fsdbShr.h`. Expected library directories include `linux64` and sometimes alternatives such as `linux64_gcc950`. The main Reader library is `libnffr.so`; the inspected installation also has `libnsys.so` as a companion runtime library. These files are local proprietary inputs and must never be committed.

Repository automation is container-first. `make` targets require `WAVEPEEK_IN_CONTAINER=1`. Public GitHub Actions uses `.devcontainer/devcontainer.ci.json`, which does not set `VERDI_HOME`, and runs `make ci`. The interactive devcontainer uses `.devcontainer/devcontainer.json`, sets `VERDI_HOME=/opt/verdi`, and may mount host Verdi at `/opt/verdi`. Because that mount can be empty, any Verdi detection must inspect files, not trust the variable.

The term “feature-gated” means Cargo compiles code only when a named feature is enabled. Here the feature is `fsdb`. With default features, the build script must not look for Verdi, must not compile native FSDB code, and must not link `libnffr.so`. With `--features fsdb`, the build script must verify the local SDK, compile the shim, and emit linker settings.

The term “rpath” means an ELF runtime search path embedded into a Linux binary or test executable so the dynamic loader can find `libnffr.so` at run time. Because this design links directly to local Verdi libraries, feature-enabled binaries depend on the selected Verdi library directory being available after build.

## Open Questions

No product questions block this build spike. The exact CLI error for opening `.fsdb` without FSDB support is intentionally left to the later FSDB-disabled UX slice.

There are two implementation-time checks to record if they behave differently on a real Verdi installation. First, if `libnsys.so` is not present in a supported Reader distribution but `libnffr.so` links and loads without it, update this plan and the environment checker to make `libnsys.so` optional. Second, if the default `linux64` directory fails with the devcontainer compiler while `linux64_gcc950` succeeds, record the exact compiler/linker error and keep the override path documented in the failure message.

## Plan of Work

Milestone 1 establishes the feature-gated build foundation without touching runtime waveform behavior. Add `[features] default = []` and `fsdb = []` to `Cargo.toml`. Add `[build-dependencies] cc = "~1"`. Create `build.rs`. When `CARGO_FEATURE_FSDB` is absent, `build.rs` should emit only relevant `cargo:rerun-if-env-changed` lines and return successfully. When the feature is present, it should fail unless `CARGO_CFG_TARGET_OS=linux`; for this first implementation, also fail unless `CARGO_CFG_TARGET_ARCH=x86_64`, because the planned default Verdi directories are `linux64` ABI directories. The failure must be explicit, for example: `FSDB support is Linux x86_64 only in this build spike`.

In the feature-enabled branch of `build.rs`, resolve the Reader SDK with one clear rule: `VERDI_HOME` is always required because the build needs headers from `$VERDI_HOME/share/FsdbReader`. `WAVEPEEK_FSDB_READER_LIBDIR` overrides only the library directory, not the header root. The selected library directory is resolved in this order: `WAVEPEEK_FSDB_READER_LIBDIR`, then `$VERDI_HOME/share/FsdbReader/$WAVEPEEK_FSDB_ABI`, then `$VERDI_HOME/share/FsdbReader/linux64`. Check for `ffrAPI.h`, `ffrKit.h`, `fsdbShr.h`, `libnffr.so`, and `libnsys.so`. If a check fails, panic from the build script with a message that names the missing path and suggests setting `VERDI_HOME`, `WAVEPEEK_FSDB_READER_LIBDIR`, or `WAVEPEEK_FSDB_ABI=linux64_gcc950` as appropriate.

Milestone 2 changes automation so default gates stay public and Verdi-free. In `Makefile`, change `lint` from `cargo clippy --all-targets --all-features -- -D warnings` to `cargo clippy --all-targets -- -D warnings`. Change `lint-fix` the same way by removing `--all-features`. Change `coverage-src` to remove `--all-features`. Keep `test` as default-feature `cargo test -q`. Add explicit targets named `check-fsdb-env`, `require-verdi`, `check-fsdb-build`, and `test-fsdb`. `check-fsdb-env` runs the environment checker as a non-failing availability probe: it prints the stable no-Verdi skip line and exits 0 when the SDK is unavailable. `require-verdi` is analogous to `require-container`; it depends on `require-container` and fails clearly when `VERDI_HOME` does not resolve to a usable SDK. `check-fsdb-build` depends on `require-verdi` and then runs `cargo check --features fsdb` plus `cargo test --features fsdb --lib fsdb_reader_metadata_smoke -- --nocapture`. `test-fsdb` should depend on `check-fsdb-build` initially; later slices can expand it into bundled-example integration tests. Update `.pre-commit-config.yaml` descriptions only if they currently promise all-feature linting; do not make pre-commit require Verdi. Update `docs/DEVELOPMENT.md` so the direct Cargo equivalents no longer recommend `--all-features` for default lint/coverage, and add a short FSDB development paragraph explaining `make check-fsdb-env`, `require-verdi`, and `make check-fsdb-build`.

Milestone 3 adds local Verdi discovery helpers. Create `.devcontainer/resolve_verdi_home.sh`. This script must not bake in a site-specific Verdi path; it should inspect `VERDI_HOME` only, print that path when it contains `share/FsdbReader/ffrAPI.h` and the default or selected Reader library directory contains `libnffr.so`, otherwise print nothing and exit 0. Create `scripts/check_fsdb_env.py` using only the Python standard library. It should inspect only SDK-related configuration: `VERDI_HOME`, `WAVEPEEK_FSDB_READER_LIBDIR`, and `WAVEPEEK_FSDB_ABI`. Tests that need an FSDB file should use the concrete `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` path directly. The checker exits 0 when the SDK is usable and exits 77 in optional mode when Verdi is unavailable, so `check-fsdb-env` can print a skip line without failing. With `--require`, the checker exits 1 for unavailable Verdi so `require-verdi` can stop explicit FSDB build targets before Cargo. It also exits 1 for contradictory explicit configuration, such as `WAVEPEEK_FSDB_READER_LIBDIR` without a usable `VERDI_HOME`, or an explicit library directory that exists but does not contain `libnffr.so`. Use stable stderr/stdout messages, for example `skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks`.

Milestone 4 adds the native shim. Create `native/fsdb/wavepeek_fsdb_shim.h` and `native/fsdb/wavepeek_fsdb_shim.cpp`. These are project-owned files. The header must define only wavepeek-owned C ABI types and function declarations. Use names such as `wp_fsdb_status`, `wp_fsdb_error`, `wp_fsdb_reader`, `wp_fsdb_metadata`, `wp_fsdb_probe`, `wp_fsdb_open`, `wp_fsdb_close`, `wp_fsdb_read_metadata`, `wp_fsdb_free_error`, and `wp_fsdb_free_string`. The implementation may include `ffrAPI.h` from the local SDK but must not copy declarations from it into committed source. Wrap all exported functions in `extern "C"`. Do not let C++ exceptions cross the C ABI boundary. Convert failures to status codes and heap-owned error strings that Rust can free through shim-provided free functions.

The shim behavior for this slice is intentionally small. `wp_fsdb_probe(path, is_fsdb, error)` calls `ffrObject::ffrIsFSDB(path)` and writes `1` or `0` through the `is_fsdb` out-parameter so a successful non-FSDB probe is distinct from an API failure. `wp_fsdb_open(path, out, error)` opens the file with `ffrObject::ffrOpen3(path)`, stores the returned reader object in an opaque `wp_fsdb_reader`, and suppresses FSDB Reader info/warning output before normal command output can be polluted. `wp_fsdb_read_metadata(reader, out, error)` reads `ffrGetScaleUnit()`, `ffrGetXTagType()`, `ffrGetMinFsdbTag64()`, and `ffrGetMaxFsdbTag64()`. Store the scale unit as an owned C string and the min/max tags as raw high/low-derived `uint64_t` values. Treat floating/double xtag types as unsupported in the Rust wrapper for now, but still expose the raw xtag type in metadata so later slices can improve diagnostics. `wp_fsdb_close(reader)` closes and deletes the reader handle safely.

Milestone 5 adds the private Rust FFI wrapper and smoke test. Add `#[cfg(feature = "fsdb")] mod fsdb_native;` near the top of `src/waveform/mod.rs`, but do not call it from `Waveform::open`. Create `src/waveform/fsdb_native.rs`. Declare the C ABI functions manually with an `unsafe extern "C"` block. Define a private Rust struct such as `FsdbReader` with `open(path: &Path) -> Result<Self, WavepeekError>` and `metadata(&self) -> Result<FsdbNativeMetadata, WavepeekError>`. Define `FsdbNativeMetadata` with `scale_unit: String`, `time_start_raw: u64`, `time_end_raw: u64`, and `xtag_type: u32`. Implement `Drop` for `FsdbReader` so `wp_fsdb_close` always runs. Convert shim failures into `WavepeekError::File` with messages that start with `FSDB Reader` for SDK/native failures.

In the same Rust module, add a feature-gated unit test named `fsdb_reader_metadata_smoke`. The test builds the concrete path `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`, opens that file through `FsdbReader`, reads metadata, asserts that `scale_unit` is non-empty, and asserts that `time_end_raw >= time_start_raw`. Do not perform separate existence checks: a Verdi-equipped test environment is expected to provide this bundled example. Do not use the `read` tool or text utilities on `.fst` or `.fsdb` files. The test opens the FSDB only through the Reader API.

Milestone 6 adds tests for no-Verdi behavior and environment-script determinism. Add Python unit tests if useful under `scripts/test_check_fsdb_env.py`, and wire them into `test-aux` only if they do not require Verdi. At minimum, test that an empty temporary `VERDI_HOME` causes the checker to emit the skip status, and that an explicit bad `WAVEPEEK_FSDB_READER_LIBDIR` produces a non-skip failure. Add Rust unit tests for any pure helper in `build.rs` only if you factor resolver logic into testable functions; otherwise rely on script tests and make-target validation. Do not add `.fsdb` fixtures to the repository.

Milestone 7 closes with validation, review, and documentation bookkeeping. Run default-feature validation first in the available container: `make format-check`, `make lint`, `make check-schema`, `make check-build`, and `make ci`. If the required RTL fixture payload is genuinely unavailable in the current environment, record that as a validation limitation in this plan and run the component gates that do not need that payload; do not silently downgrade the standard gate. Run `make check-fsdb-env` in a no-Verdi simulation and confirm it exits 0 after printing the skip line. Run `make check-fsdb-build` in the available Verdi environment; it must pass through `require-verdi`, compile/link the shim, and run the metadata smoke test against `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`. Update `CHANGELOG.md` only if the team treats the new `fsdb` Cargo feature and FSDB build smoke as user-visible; otherwise keep the change documented in this plan and temporary FSDB collateral. After implementation commits, run focused review lanes for build/link correctness, docs/automation consistency, and licensing/source hygiene, fix findings in follow-up commits, run one final control review, keep this plan in `active/` for user inspection if requested, and then move it to `docs/exec-plans/completed/2026-05-21-fsdb-build-spike/PLAN.md` when the user asks for completion bookkeeping.

### Concrete Steps

Run all commands from the repository root, `/workspaces/wavepeek-fsdb`.

1. Confirm the starting state.

    git status --short --branch
    rg --line-number "all-features|features fsdb|check-fsdb|fsdb_native|wavepeek_fsdb_shim" Cargo.toml Makefile docs scripts src native .github .pre-commit-config.yaml

Expected before implementation: the worktree is clean, `Cargo.toml` has no `fsdb` feature, there is no `build.rs`, there is no `native/fsdb/`, and default make targets still contain `--all-features` in lint/coverage paths.

2. Edit `Cargo.toml`.

Add:

    [features]
    default = []
    fsdb = []

Add:

    [build-dependencies]
    cc = "~1"

Run:

    cargo generate-lockfile

This may update `Cargo.lock` to include `cc` and its transitive build dependencies. Keep the lockfile change committed.

3. Create `build.rs`.

Implement the feature gate, Linux x86_64 check, SDK resolution, required-file checks, `cc` compilation of `native/fsdb/wavepeek_fsdb_shim.cpp`, link search path, `nffr`/`nsys` link libraries, C++ standard library link if needed by `cc`, and Linux rpath emission. Use `cc::Build::new().cpp(true)` and request a compatible explicit standard, for example with `flag_if_supported("-std=c++17")`, so the feature-enabled path does not rely on C compiler defaults. Include `cargo:rerun-if-env-changed=VERDI_HOME`, `cargo:rerun-if-env-changed=WAVEPEEK_FSDB_READER_LIBDIR`, and `cargo:rerun-if-env-changed=WAVEPEEK_FSDB_ABI`.

4. Create helper scripts.

Create:

    .devcontainer/resolve_verdi_home.sh
    scripts/check_fsdb_env.py

Make both executable. Keep script output deterministic and non-interactive.

5. Update make and development docs.

Edit:

    Makefile
    docs/DEVELOPMENT.md
    .pre-commit-config.yaml

Remove default `--all-features` usage from default lint/coverage paths and add the explicit FSDB targets. Keep public CI untouched unless a workflow text label needs to stop implying all-feature coverage.

6. Create the native shim and Rust wrapper.

Create:

    native/fsdb/wavepeek_fsdb_shim.h
    native/fsdb/wavepeek_fsdb_shim.cpp
    src/waveform/fsdb_native.rs

Edit `src/waveform/mod.rs` only enough to declare the gated private module. Do not connect the wrapper to `Waveform::open`.

7. Add tests for scripts or pure helper behavior, if not already covered by make-target smoke.

Prefer:

    scripts/test_check_fsdb_env.py

and add it to `make test-aux` only if it is deterministic without Verdi.

8. Validate no-Verdi behavior.

In an environment without a valid SDK, run:

    make format-check
    make lint
    make check-schema
    make check-build
    make check-fsdb-env

Expected FSDB availability-probe output includes one stable skip line and exit status 0:

    skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks

Also run the explicit Verdi gate and the intentionally failing feature build directly, and verify both messages are clear:

    VERDI_HOME=$(mktemp -d) make require-verdi
    env -u VERDI_HOME cargo check --features fsdb

Expected result: `make require-verdi` exits non-zero before Cargo and says to set `VERDI_HOME`; direct feature-enabled Cargo exits non-zero with a build error that names missing `VERDI_HOME` or the missing SDK path.

9. Validate Verdi behavior when available.

On a machine with Verdi available through `VERDI_HOME`, run:

    make check-fsdb-build

Expected result: `require-verdi` passes, `cargo check --features fsdb` passes, and the feature-gated unit test opens `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` through FSDB Reader, reads non-empty `scale_unit`, observes `time_end_raw >= time_start_raw`, and exits 0 without printing FSDB Reader chatter to stdout/stderr beyond the test harness line.

10. Run the full gate when practical.

    make ci

Expected result in public/no-Verdi environments: existing VCD/FST tests, schema checks, action lint, auxiliary tests, and cargo build checks pass without trying to link Verdi.

### Validation and Acceptance

The implementation is accepted only when default-feature behavior and feature-enabled behavior are both demonstrated.

In a public or no-Verdi environment, `make ci` must pass and must not mention `libnffr.so`, `ffrAPI.h`, or `VERDI_HOME` except in tests that intentionally validate FSDB skip behavior. `make check-fsdb-env` must exit 0 with a clear skip line, while `make require-verdi` and `make check-fsdb-build` must fail clearly before Cargo because they explicitly require Verdi. Direct `cargo check --features fsdb` without a valid SDK must fail fast with a build-script error that explains what path or variable is missing.

In a Verdi-equipped environment, `cargo check --features fsdb` must compile and link the shim when `VERDI_HOME` is set. `make check-fsdb-build` must run the gated Rust test, and that test must actually open `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` through FSDB Reader and read metadata; it is not enough to compile an unused function.

The implementation must leave the public CLI contract unchanged. Running `wavepeek info --waves tests/fixtures/hand/m2_core.vcd --json` after the change should produce the same VCD metadata as before. Running FSDB through the normal CLI is not accepted in this slice because the backend refactor and disabled UX are separate work.

The implementation must leave the repository clean of proprietary payload. Verification command:

    find . -path './target' -prune -o \( -name '*.fsdb' -o -name 'ffrAPI.h' -o -name 'ffrKit.h' -o -name 'fsdbShr.h' -o -name 'libnffr.so' -o -name 'libnsys.so' \) -print

Expected output: empty, unless the path is under an ignored local scratch directory such as `tmp/`, in which case remove it before committing.

### Idempotence and Recovery

The new make targets are safe to rerun. `check-fsdb-env` must skip before invoking Cargo when Verdi is unavailable. `check-fsdb-build` and `test-fsdb` must depend on `require-verdi`, so they fail early when Verdi is unavailable and run normal Cargo checks when Verdi is available. They must not create or download FSDB fixtures.

If `cargo check --features fsdb` fails with an ABI or missing-library error, rerun with an explicit ABI override:

    WAVEPEEK_FSDB_ABI=linux64_gcc950 cargo check --features fsdb

If the library directory is non-standard, use:

    WAVEPEEK_FSDB_READER_LIBDIR=$VERDI_HOME/share/FsdbReader/linux64_gcc950 cargo check --features fsdb

If a feature-enabled binary or test was built against a Verdi path that moved, clean and rebuild after restoring `VERDI_HOME` or selecting a new library directory:

    cargo clean
    cargo check --features fsdb

Do not bypass pre-commit hooks. If hooks fail because the environment lacks Verdi, that is a bug in the default hooks; fix the hook target so it uses default features rather than skipping hooks.

### Artifacts and Notes

Record concise validation transcripts here as implementation proceeds. The simulated no-Verdi availability-probe transcript observed locally was:

    $ VERDI_HOME=$(mktemp -d) make check-fsdb-env
    skip: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run FSDB build checks

The explicit Verdi gate now fails before Cargo when Verdi is unavailable:

    $ VERDI_HOME=$(mktemp -d) make require-verdi
    error: fsdb: Verdi FSDB Reader SDK not found; set VERDI_HOME to run this target

The feature-enabled metadata smoke transcript on the available Verdi-equipped container was:

    $ make check-fsdb-build
    running 1 test
    test waveform::fsdb_native::tests::fsdb_reader_metadata_smoke ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 108 filtered out

The direct feature-enabled build failure without `VERDI_HOME` was:

    $ env -u VERDI_HOME -u WAVEPEEK_FSDB_READER_LIBDIR -u WAVEPEEK_FSDB_ABI cargo check --features fsdb
    status=101
    FSDB support requires VERDI_HOME; set VERDI_HOME to a Synopsys Verdi installation containing share/FsdbReader

Default-feature and repository hygiene validation was:

    $ make ci
    [OK] Build successful

    $ find . -path './target' -prune -o -path './tmp' -prune -o \( -name '*.fsdb' -o -name 'ffrAPI.h' -o -name 'ffrKit.h' -o -name 'fsdbShr.h' -o -name 'libnffr.so' -o -name 'libnsys.so' \) -print
    <no output>

    $ cargo run --quiet -- info --waves tests/fixtures/hand/m2_core.vcd --json
    {"$schema":"https://raw.githubusercontent.com/kleverhq/wavepeek/v0.5.0/schema/wavepeek.json","command":"info","data":{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"},"warnings":[]}

The smoke test uses the bundled Verdi example at `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb`. No repository FSDB fixture was added.

### Interfaces and Dependencies

At the end of this implementation, `Cargo.toml` must contain the `fsdb` feature and a `cc` build dependency. The default feature set must stay empty.

`build.rs` must expose no public Rust API, but it must implement this effective interface through environment variables:

    VERDI_HOME
    WAVEPEEK_FSDB_READER_LIBDIR
    WAVEPEEK_FSDB_ABI

The C shim header `native/fsdb/wavepeek_fsdb_shim.h` must define project-owned C ABI declarations equivalent to:

    typedef enum wp_fsdb_status {
        WP_FSDB_STATUS_OK = 0,
        WP_FSDB_STATUS_ERROR = 1
    } wp_fsdb_status;

    typedef struct wp_fsdb_reader wp_fsdb_reader;

    typedef struct wp_fsdb_metadata {
        char *scale_unit;
        unsigned long long time_start_raw;
        unsigned long long time_end_raw;
        unsigned int xtag_type;
    } wp_fsdb_metadata;

    wp_fsdb_status wp_fsdb_probe(const char *path, int *is_fsdb, char **error_message);
    wp_fsdb_status wp_fsdb_open(const char *path, wp_fsdb_reader **out, char **error_message);
    void wp_fsdb_close(wp_fsdb_reader *reader);
    wp_fsdb_status wp_fsdb_read_metadata(wp_fsdb_reader *reader, wp_fsdb_metadata *out, char **error_message);
    void wp_fsdb_free_string(char *value);

The exact C names may differ slightly if implementation finds a safer ownership shape, but they must remain descriptive and must not include milestone labels.

The Rust module `src/waveform/fsdb_native.rs` must define private types equivalent to:

    struct FsdbReader { raw: NonNull<ffi::wp_fsdb_reader> }

    struct FsdbNativeMetadata {
        scale_unit: String,
        time_start_raw: u64,
        time_end_raw: u64,
        xtag_type: u32,
    }

    impl FsdbReader {
        fn open(path: &Path) -> Result<Self, WavepeekError>;
        fn metadata(&self) -> Result<FsdbNativeMetadata, WavepeekError>;
    }

`Makefile` must provide these explicit FSDB targets:

    check-fsdb-env
    check-fsdb-build
    test-fsdb

Their names are part of the developer workflow. They must be deterministic, non-interactive, and safe in no-Verdi public environments.

Revision Note: 2026-05-21 / Grin - Initial draft created from `docs/fsdb/arch.md` build-spike requirements and current repository automation. The draft intentionally avoids milestone-label names for new entities and keeps CLI FSDB behavior out of scope for this slice.

Revision Note: 2026-05-21 / Grin - Revised after parallel review to clarify the `VERDI_HOME` header requirement, make `wp_fsdb_probe` distinguish successful non-FSDB probes from failures, require explicit C++ compilation settings for `cc`, make `make ci` a required default validation gate unless the environment cannot provide standard RTL fixtures, and define explicit-library-dir-without-headers as a configuration error rather than an ordinary no-Verdi skip.

Revision Note: 2026-05-21 / Grin - Recorded the clean final independent control review and updated the retrospective to mark planning complete while leaving implementation progress unchecked.

Revision Note: 2026-05-21 / Grin - Updated progress, discoveries, decisions, and closure instructions after implementing the FSDB build spike foundation. The note records the `libnsys.so` dynamic-link issue, the decision to keep invalid devcontainer `VERDI_HOME` as an optional-target skip, and the user-requested choice to leave this plan active for inspection until explicit completion bookkeeping.

Revision Note: 2026-05-21 / Grin - Recorded validation evidence after `make ci`, Verdi-enabled `make check-fsdb-build`, simulated no-Verdi skip, direct missing-`VERDI_HOME` feature-build failure, proprietary-payload search, and VCD CLI smoke all behaved as expected.

Revision Note: 2026-05-21 / Grin - Applied first implementation-review fixes in the plan text: made the old `--all-features` discovery explicitly historical, sanitized local dynamic-loader details, and updated the FSDB checker transcript to the new path-free default output.

Revision Note: 2026-05-21 / Grin - Recorded targeted re-review results and the follow-up fix that keeps explicit FSDB checker error output path-free by default while allowing verbose local path diagnostics through `WAVEPEEK_FSDB_ENV_VERBOSE=1`.

Revision Note: 2026-05-21 / Grin - Recorded the clean second targeted re-review and final independent control review. The implementation was ready for user inspection, so this plan intentionally remained under `active/` at that point rather than being moved to `completed/` until the user asked for closure.

Revision Note: 2026-05-21 / Grin - After the user asked about `$VERDI_HOME` examples, recorded that bundled Verdi example FSDB files exist, added real-file smoke evidence using a bundled example, and documented the follow-up native-output suppression needed to keep FSDB Reader chatter out of command/test output.

Revision Note: 2026-05-21 / Grin - Recorded final follow-up review and fixed the stale Decision Log sentence so it names `wp_fsdb_close` alongside the other native calls covered by scoped output suppression.

Revision Note: 2026-05-21 / Grin - Updated the living plan after removing hard-coded Verdi fallback paths from scripts and adding the `require-verdi` Makefile gate. The plan now distinguishes the non-failing `check-fsdb-env` availability probe from explicit FSDB build targets that require Verdi, and records the clean targeted review of this follow-up.

Revision Note: 2026-05-21 / Grin - Simplified the plan and implementation to use `$VERDI_HOME/share/VIA/demo/waveform/cpu.fsdb` directly in the Rust smoke test. Recorded validation and the clean targeted review.

Revision Note: 2026-05-21 / Grin - Moved the plan to `docs/exec-plans/completed/2026-05-21-fsdb-build-spike/PLAN.md` after the user requested completion bookkeeping and updated `docs/fsdb/arch.md` M0 to point at the completed implementation record.
