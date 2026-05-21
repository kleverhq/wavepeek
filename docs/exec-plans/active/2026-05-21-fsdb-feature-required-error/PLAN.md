# FSDB feature-required error for default builds

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the repository `exec-plan` skill. It is intentionally self-contained: a contributor with only the current working tree and this file should be able to implement, validate, and review the change without prior conversation.

## Purpose / Big Picture

Default `wavepeek` binaries do not link against Synopsys Verdi FSDB Reader libraries. Today, if a user points a default binary at a `.fsdb` dump, the request can fall through to the VCD/FST parser and produce a generic parse failure. After this change, when the existing VCD/FST open path fails to parse a file whose name looks like an FSDB dump, the default binary will return a clear file-level error explaining that FSDB requires an explicitly feature-enabled build and a licensed Verdi installation. Valid VCD or FST content must still be accepted even if somebody, in a fit of archival whimsy, gave it an FSDB-looking filename.

The user-visible proof is simple: running `wavepeek info --waves path/to/dump.fsdb` with a default build must exit with code `2`, leave stdout empty, and print the exact file error described below. Existing VCD and FST behavior must not change. The implementation must also demonstrate that source coverage and command performance do not regress; this is not optional, because even tiny open-path checks have a habit of becoming “obviously harmless” right before they irritate every command invocation. Splendid little gremlins, those checks.

## Non-Goals

This plan does not implement FSDB parsing, hierarchy traversal, value sampling, change collection, property evaluation, or any C++ Reader API call in the user-facing command path. It does not add `.fsdb` fixtures to the repository, does not download FSDB files, and does not read any `.fsdb` file as text. It does not change the JSON schema payloads for successful commands. It does not publish or package FSDB-enabled binaries. It also does not introduce milestone-labelled names: no new file, module, function, test, benchmark run directory, documentation heading, or public symbol created by this plan may use a milestone prefix, suffix, or embedded label. Use behavior names such as `fsdb_disabled`, `fsdb_extension`, or `feature_required` instead.

## Progress

- [x] (2026-05-21 19:37Z) Researched the architecture milestone in `docs/fsdb/arch.md`, current waveform facade in `src/waveform/mod.rs`, Wellen open/error mapping in `src/waveform/wellen_backend.rs`, optional FSDB smoke code in `src/waveform/fsdb_native.rs`, public command/machine-output docs, development gates, coverage tooling, and benchmark workflow.
- [x] (2026-05-21 19:45Z) Drafted this implementation plan under `docs/exec-plans/active/2026-05-21-fsdb-feature-required-error/PLAN.md` with behavior-oriented names and explicit performance/coverage controls.
- [x] (2026-05-21 19:58Z) Ran independent plan-compliance, architecture/code-plan, and performance/coverage review lanes; folded their blocking feedback into this plan.
- [x] (2026-05-21 20:14Z) Captured default-build validation baselines before source edits: `make coverage-src` passed with source coverage regions `95.02%`, functions `95.74%`, lines `95.59%`; full `bench/e2e/tests.json` benchmark baseline completed under `tmp/fsdb-feature-required-error/perf-before` with `142` Hyperfine and `142` wavepeek JSON artifacts.
- [x] (2026-05-21 20:24Z) Added default-build FSDB-disabled routing in `src/waveform/fsdb_disabled.rs` and `Waveform::open`, preserving Wellen-first parsing and narrow translation of `cannot parse '<path>': ...` file errors only.
- [x] (2026-05-21 20:25Z) Added default-build unit and CLI integration tests in `src/waveform/fsdb_disabled.rs` and `tests/fsdb_disabled_cli.rs`; `cargo test -q --test fsdb_disabled_cli` passed with `6` tests and `cargo test -q fsdb_disabled` passed with `4` targeted unit tests.
- [x] (2026-05-21 20:35Z) Updated README, public introduction, command-model reference, top-level CLI help, per-command `--waves` help, help-contract assertions, and changelog language to describe default VCD/FST support and feature-gated FSDB behavior honestly.
- [x] (2026-05-21 20:36Z) Ran documentation/help checks: `cargo test -q --test cli_contract`, `cargo test -q --test docs_cli`, and `cargo test -q --test schema_cli` all passed; the stale-wording search no longer finds the old top-level or `--waves` help phrases.
- [x] (2026-05-21 20:55Z) Ran full post-change validation gates: `make format-check`, `make lint`, `make check-schema`, `make test-aux`, `make coverage-src`, `make coverage-src-check`, and `make check-build` all passed. Source coverage improved slightly from baseline to regions `95.04%`, functions `95.76%`, lines `95.61%`.
- [x] (2026-05-21 20:58Z) Ran optional FSDB environment validation in this container. `make check-fsdb-env` reported `ok: fsdb: Verdi FSDB Reader SDK found`, and `make check-fsdb-build` passed `cargo check --features fsdb` plus the `fsdb_reader_metadata_smoke` library test.
- [x] (2026-05-21 21:24Z) Ran the full `bench/e2e/tests.json` candidate benchmark and repeat/control performance investigations. The catalog-level zero-threshold compare proved too noisy even for same-binary control runs, so paired Hyperfine checks were run for the worst apparent regressions; paired old/new runs showed no repeatable slowdown and several were slightly faster within noise.
- [x] (2026-05-21 21:31Z) Ran final repository gates `make check` and `make ci`; both completed successfully.
- [x] (2026-05-21 21:43Z) Ran four read-only review lanes (code correctness, docs/help, architecture, performance). Code, docs, and performance lanes reported no substantive findings; architecture reported that the README could imply `--features fsdb` already gives full Reader-backed command support, so README and public docs were qualified and docs/help tests were rerun successfully.

## Surprises & Discoveries

- Observation: A default build cannot call `ffrObject::ffrIsFSDB`, so exact FSDB detection is impossible without the optional Reader SDK and feature-enabled C++ shim.
  Evidence: `docs/fsdb/arch.md` states that no-feature detection is best-effort through `.fsdb` / `.fsdb.gz` extension checks, while feature-enabled builds use the FSDB Reader probe.
- Observation: The current `Waveform::open` path is a thin facade over `wellen_backend::WellenBackend::open`, and every waveform command uses that facade through the engine layer.
  Evidence: `src/waveform/mod.rs` defines `Waveform::open(path)` and stores only `Backend::Wellen(wellen_backend::WellenBackend)` today.
- Observation: Source coverage is already a CI gate, but plan-level comparison against the pre-change baseline is still useful because this change adds code in `src/**`.
  Evidence: `docs/DEVELOPMENT.md` documents `make coverage-src-check` as a 90% source-coverage gate for lines, regions, and functions.
- Observation: Returning the feature-required error before trying the existing Wellen parser would break valid VCD/FST files that happen to have `.fsdb` or `.fsdb.gz` names.
  Evidence: `docs/fsdb/arch.md` says default-build detection is extension checks plus fallback after a failed `wellen` parse, not extension checks instead of parsing.
- Observation: Command-specific `--waves` help text is not only in `src/cli/mod.rs`; every waveform command argument module currently carries its own VCD/FST wording.
  Evidence: `rg "Path to VCD/FST waveform file" src/cli tests` reports matches in `src/cli/info.rs`, `scope.rs`, `signal.rs`, `value.rs`, `change.rs`, and `property.rs`, plus help-contract tests.
- Observation: `scripts/check_coverage.py` validates one coverage summary against thresholds; it does not compare two summaries for regression by itself.
  Evidence: The script accepts one `--summary-json` input and minimum thresholds, so this plan includes an explicit before/after comparison command.
- Observation: The current Wellen path reports fake invalid `.fsdb.gz` bytes as a parse failure instead of a gzip open/read failure, so the same narrow post-parse fallback covers `.fsdb` and `.fsdb.gz` temporary-file tests.
  Evidence: `cargo run -q -- info --waves tmp/invalid.fsdb.gz` before the routing change printed `error: file: cannot parse 'tmp/invalid.fsdb.gz': unknown file format, only GHW, FST and VCD are supported`.
- Observation: A tiny valid VCD file with a `.fsdb` suffix is accepted by the existing Wellen path, which validates that suffix checks must stay behind parser failure.
  Evidence: `cargo run -q -- info --waves tmp/test-valid.fsdb` before the routing change printed `time_unit: 1ns`, `time_start: 0ns`, and `time_end: 10ns`.
- Observation: The current `bench/e2e/perf.py compare` interface uses `--golden`, `--revised`, and `--max-negative-delta-pct`, not the older `--baseline-dir`, `--candidate-dir`, `--metric`, `--threshold`, and `--markdown-output` options shown in this plan's first draft.
  Evidence: `python3 bench/e2e/perf.py compare --help` prints `--revised REVISED --golden GOLDEN --max-negative-delta-pct MAX_NEGATIVE_DELTA_PCT`.
- Observation: A zero-threshold catalog compare is too sensitive to environment noise to be a sole blocking signal for this benchmark suite.
  Evidence: comparing two consecutive runs of the same post-change release binary with `--max-negative-delta-pct 0` still failed, including apparent same-binary regressions such as `info_scr1` median `0.042768s -> 0.056186s` (`-31.37%`) and `change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk` median `0.241656s -> 0.291143s` (`-20.48%`).
- Observation: Pairwise Hyperfine runs for the worst apparent catalog regressions did not reproduce a code slowdown when old and new binaries were measured in the same command.
  Evidence: paired 20-run checks showed `change_scr1_coremark_imem_axi_2sig_to_1000ps` new `34.6ms` versus old `34.9ms`, `change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any` new `166.0ms` versus old `167.9ms`, `change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk` new `264.1ms` versus old `267.2ms`, `info_scr1` new `15.9ms` versus old `16.3ms`, and `signal_scr1_top_recursive_depth2_json` new `7.6ms` versus old `7.7ms`; `change_scr1_coremark_imem_axi_1sig_to_1000ps` was the only selected pair with old slightly faster, `33.8ms` versus `34.3ms`, inside the observed standard deviation.

## Decision Log

- Decision: Preserve the public error category `file` and process exit code `2` for FSDB-disabled input.
  Rationale: `docs/fsdb/arch.md` explicitly recommends `error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME`, and `docs/public/reference/machine-output.md` assigns file-level open/parse/unsupported-format failures to exit code `2`. Adding `error: fsdb:` would create a taxonomy decision outside this slice.
  Date/Author: 2026-05-21 / Grin

- Decision: Use best-effort filename detection for default builds: case-insensitive `.fsdb` and `.fsdb.gz` suffixes on the final file name, applied only after the existing VCD/FST open path reports a parse or unsupported-format failure. Do not attempt magic-byte sniffing.
  Rationale: The architecture calls the no-feature path “extension checks plus fallback after a failed Wellen parse.” Magic sniffing would rely on undocumented proprietary details, likely require real FSDB samples for confidence, and risk false certainty. If a user renames an FSDB file to an unrelated suffix, the default binary may still report the old parse error; that limitation is acceptable until the Reader probe exists in the feature-enabled backend.
  Date/Author: 2026-05-21 / Grin

- Decision: Preserve ordinary open errors and valid VCD/FST parsing before returning the FSDB feature-required error.
  Rationale: A missing, unreadable, or directory path ending in `.fsdb` should still produce the ordinary `cannot open` file error. A valid VCD/FST file with a misleading suffix should still work. The feature-required message should mean “the current parser could not parse this FSDB-looking file, and the default binary cannot use the FSDB Reader,” not “any path ending in `.fsdb` is assumed to be FSDB.”
  Date/Author: 2026-05-21 / Grin

- Decision: Add tests using temporary files with `.fsdb` and `.fsdb.gz` suffixes instead of real FSDB fixtures, and also add a valid tiny VCD file with an FSDB-looking suffix.
  Rationale: Temporary byte files prove the routing and error text without storing proprietary dumps or reading binary FSDB content as text. The renamed-VCD case guards the required fallback-after-Wellen behavior.
  Date/Author: 2026-05-21 / Grin

- Decision: Gate the default-build integration tests with `#[cfg(not(feature = "fsdb"))]`.
  Rationale: The expected feature-required error is a default-build behavior. Future feature-enabled tests should verify Reader-backed behavior instead of inheriting default-build assertions.
  Date/Author: 2026-05-21 / Grin

- Decision: Update public docs, top-level help, and each waveform command’s `--waves` help wording to describe default VCD/FST support plus feature-gated FSDB recognition, without implying that full FSDB command support is complete.
  Rationale: The architecture says public docs should be updated after the optional FSDB feature is accepted. The honest shipped behavior at this slice is “default binaries give a clear FSDB feature-required error after VCD/FST parsing fails,” not “all waveform commands fully support FSDB.”
  Date/Author: 2026-05-21 / Grin

- Decision: Treat coverage and performance as blocking validation, not nice-to-have evidence.
  Rationale: The change touches the common waveform open path. Coverage after the change must be at least as good as the captured baseline and must satisfy the repository threshold. Benchmark comparison must first run with a zero allowed regression threshold; any repeatable mean or median slowdown after control reruns is blocking.
  Date/Author: 2026-05-21 / Grin

- Decision: Keep parse-failure eligibility as a narrow private text match in `fsdb_disabled::should_report_disabled_support` instead of changing `WavepeekError` or exposing a new Wellen error enum.
  Rationale: This slice must preserve the public error taxonomy and stderr text for existing VCD/FST behavior. The helper only translates messages matching the current `cannot parse '<path>': ...` shape for the same path, so ordinary open failures remain untouched while avoiding a wider internal refactor.
  Date/Author: 2026-05-21 / Grin

- Decision: Treat the zero-threshold benchmark compare as an initial smoke signal and rely on same-binary control plus paired old/new Hyperfine runs for repeatability when the catalog compare is noisy.
  Rationale: The plan already required investigating repeatability instead of quietly ignoring a red compare. The repository compare command flags any negative mean or median delta, and a same-binary control run failed by double-digit percentages, so the defensible check is to measure the worst apparent regressions with old and new binaries side by side. That paired measurement directly tests whether this open-path change has a real performance cost.
  Date/Author: 2026-05-21 / Grin

- Decision: Qualify public prose that mentions `--features fsdb` so it reads as prerequisites named by the default-build diagnostic, not as a promise that this slice ships full Reader-backed FSDB command support.
  Rationale: Independent architecture review caught that the README wording could lead users to reinstall with `--features fsdb` and expect complete FSDB command behavior, while this implementation only improves the default-build error. The exact diagnostic remains unchanged because it is part of the planned behavior, but surrounding docs now state that default-package docs do not otherwise claim full Reader-backed FSDB support.
  Date/Author: 2026-05-21 / Grin

## Outcomes & Retrospective

Implementation is complete and the plan intentionally remains in `docs/exec-plans/active/` for user review. Default builds now translate existing `.fsdb` and `.fsdb.gz` parse failures into the exact feature-required file error while preserving missing-file errors, unrelated parse failures, and valid VCD/FST content with misleading suffixes. Public help and docs now describe default VCD/FST support and feature-gated FSDB behavior without claiming full FSDB command support. Focused unit and integration tests cover the new routing; `make check`, `make ci`, and optional `make check-fsdb-build` passed in this container. Source coverage improved slightly from the baseline (`95.02/95.74/95.59`) to (`95.04/95.76/95.61`) for regions/functions/lines. Full-catalog performance comparison was noisy under a zero threshold, but same-binary controls exposed that noise and paired old/new Hyperfine checks of the worst apparent regressions did not show a repeatable slowdown. The remaining limitation is intentional: default builds only detect FSDB-looking final file names after VCD/FST parsing fails; exact FSDB probing belongs to the feature-enabled Reader backend.

## Context and Orientation

`wavepeek` is a stateless command-line tool for RTL waveform inspection. “Stateless” means every command opens one waveform dump, answers one query, writes output, and exits. The current public default path supports VCD, Value Change Dump, and FST, Fast Signal Trace, through the Rust crate `wellen`. `wellen` is wrapped by repository code in `src/waveform/wellen_backend.rs`.

FSDB is the Fast Signal Database format used by Synopsys Verdi. Reading real FSDB files requires the licensed Verdi FSDB Reader SDK, which is not redistributed by this repository. The optional Cargo feature is named `fsdb`; when enabled on a machine with `VERDI_HOME`, existing smoke code compiles a small C++ shim and verifies basic Reader access. That smoke code is not yet connected to user-facing waveform commands.

The command path relevant to this work is:

1. CLI modules under `src/cli/` parse commands such as `info`, `scope`, `signal`, `value`, `change`, and `property`.
2. Engine modules under `src/engine/` call the waveform facade.
3. `src/waveform/mod.rs` defines `Waveform::open(path: &Path) -> Result<Waveform, WavepeekError>` and the backend dispatch enum.
4. `src/waveform/wellen_backend.rs` currently opens the file with `wellen::simple::read(path)` and maps I/O errors to `WavepeekError::File("cannot open ...")` and parser errors to `WavepeekError::File("cannot parse ...")`.
5. `src/error.rs` maps `WavepeekError::File` to process exit code `2` and stderr text beginning with `error: file:`.

The existing optional FSDB code lives in `src/waveform/fsdb_native.rs` behind `#[cfg(feature = "fsdb")]`. It uses the Reader SDK only for build/link smoke and metadata probing tests. This plan must not depend on that module in the default build; the default build must remain usable without Verdi.

The repository policy treats `.fst` files as binary and says never to use the `Read` tool on them. Apply the same caution to `.fsdb` files: do not inspect `.fsdb` content with text tools, do not add proprietary fixtures, and do not paste vendor headers or documentation excerpts into this repository.

## Open Questions

There are no blocking open questions for this slice. Extension-only detection is an intentional limitation for default builds. Exact FSDB probing and feature-enabled command dispatch belong to the later backend work.

## Milestones

### Milestone 1: Capture current behavior and validation baselines

Start from a clean working tree in `/workspaces/wavepeek-fsdb`. Confirm the branch, commit, and current behavior before editing. This milestone does not change source files; it gives the implementer a baseline for correctness, coverage, and performance.

Run:

    cd /workspaces/wavepeek-fsdb
    git status --short --branch
    git rev-parse --short HEAD
    WAVEPEEK_IN_CONTAINER=1 make coverage-src
    mkdir -p tmp/fsdb-feature-required-error
    cp tmp/coverage/coverage-src-summary.json tmp/fsdb-feature-required-error/coverage-before.json
    WAVEPEEK_IN_CONTAINER=1 make check-rtl-artifacts
    command -v hyperfine
    WAVEPEEK_IN_CONTAINER=1 make build-release
    mkdir -p tmp/fsdb-feature-required-error/perf-before
    WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir tmp/fsdb-feature-required-error/perf-before

If the full end-to-end benchmark catalog is too slow for the current development loop, the implementer may first run `bench/e2e/tests_commit.json` to catch obvious problems, but the full `bench/e2e/tests.json` comparison remains blocking before handoff. Do not wave this away with “small change.” That phrase has personally assassinated more latency budgets than most algorithms.

Expected observations:

- `git status --short --branch` shows the expected branch and no unrelated changes.
- `make coverage-src` succeeds and prints source coverage for `src/**`.
- `make check-rtl-artifacts` succeeds, and `command -v hyperfine` prints the benchmark runner path.
- The benchmark run writes `tmp/fsdb-feature-required-error/perf-before/README.md` and per-test artifacts.

Acceptance for this milestone is the existence of `coverage-before.json` and a successful benchmark baseline directory. These artifacts stay under `tmp/` and must not be committed.

### Milestone 2: Add default-build FSDB-disabled detection in the waveform facade

Add the routing and error behavior in the waveform layer, not inside individual commands. Every waveform command should benefit because all of them open dumps through `Waveform::open`. The important ordering is: try the existing Wellen-backed VCD/FST path first; only if that path reports a parser or unsupported-format failure should an FSDB-looking filename become the feature-required error. Open errors, permission errors, directories, and valid VCD/FST content keep their current behavior.

Create a new source file `src/waveform/fsdb_disabled.rs`, compiled only when `feature = "fsdb"` is not enabled. Use behavior-oriented names only. The module should provide:

    pub(crate) const FSDB_DISABLED_MESSAGE: &str = "FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME";

    pub(crate) fn looks_like_fsdb_path(path: &std::path::Path) -> bool;

    pub(crate) fn disabled_support_error() -> crate::error::WavepeekError;

    pub(crate) fn should_report_disabled_support(path: &std::path::Path, error: &crate::error::WavepeekError) -> bool;

`looks_like_fsdb_path` should examine the final file name only, convert it to a lossy lower-case string, and return true for names ending in `.fsdb` or `.fsdb.gz`. It must return false for names such as `dump.fsdbx` or `dump.fsdb.gz.tmp`.

`should_report_disabled_support` should return true only when the path looks like FSDB and the Wellen open error is a parse or unsupported-format failure. It must return false for open/readability failures such as `cannot open '<path>': No such file or directory`, permission-denied paths, and directories. If the implementation has to inspect `WavepeekError::File` text because `WavepeekError` does not yet separate open and parse failures, keep the match narrow, document it in the helper, and only translate messages with the existing `cannot parse '<path>': ...` shape. If the implementer chooses to split Wellen open errors internally instead, keep that split private to `src/waveform/` and preserve public stderr text.

In `src/waveform/mod.rs`, add:

    #[cfg(not(feature = "fsdb"))]
    mod fsdb_disabled;

Then change `Waveform::open` so it tries `WellenBackend::open` first and translates only the eligible default-build parse failure. A clear implementation shape is:

    impl Waveform {
        pub fn open(path: &Path) -> Result<Self, WavepeekError> {
            match wellen_backend::WellenBackend::open(path) {
                Ok(backend) => Ok(Self {
                    backend: Backend::Wellen(backend),
                }),
                Err(error) => {
                    #[cfg(not(feature = "fsdb"))]
                    {
                        if fsdb_disabled::should_report_disabled_support(path, &error) {
                            return Err(fsdb_disabled::disabled_support_error());
                        }
                    }
                    Err(error)
                }
            }
        }
    }

Also remove any stale milestone-labelled wording from nearby comments if the edited block exposes it. For example, the top of `src/waveform/mod.rs` currently describes canonical path policy with a milestone label; when touching that area, rewrite it as behavior-oriented documentation such as “Canonical path policy” rather than a historical milestone note.

Acceptance for this milestone:

- A default build with an existing invalid `*.fsdb` or `*.fsdb.gz` file that Wellen cannot parse returns `WavepeekError::File(FSDB_DISABLED_MESSAGE)`.
- A missing, unreadable, or directory `*.fsdb` path still returns a `cannot open` file error, not the feature-required message.
- A valid VCD or FST file with an FSDB-looking suffix still opens successfully through `WellenBackend::open`.
- Existing VCD/FST code continues through `WellenBackend::open` unchanged.
- No newly created Rust module, function, constant, test, or comment uses a milestone label in its name or text.

### Milestone 3: Add integration and unit tests without real FSDB fixtures

Add tests that prove the behavior through the public CLI and small helper functions. Prefer a new integration test file `tests/fsdb_disabled_cli.rs`; the file name is behavior-oriented and does not create command-specific clutter in existing tests. Put `#![cfg(not(feature = "fsdb"))]` at the top of that integration test file, or gate every test in it with the same condition, because these assertions describe default-build behavior only.

Use `assert_cmd`, `predicates`, and `tempfile`, which are already dev-dependencies. Create fake files with binary bytes and suffixes; do not store `.fsdb` fixtures in `tests/fixtures/`. Also create one tiny valid VCD file in a temporary file with an FSDB-looking suffix to prove parsing happens before suffix-based fallback.

The integration tests should cover these cases:

1. `info --waves <temp>.fsdb` exits with code `2`, stdout is empty, and stderr is exactly or starts with:

       error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME

2. `info --waves <temp>.FSDB` behaves the same, proving case-insensitive suffix handling.
3. `info --waves <temp>.fsdb.gz` behaves the same.
4. `info --waves <missing>.fsdb` exits with code `2`, stdout is empty, stderr starts with `error: file: cannot open`, and stderr does not contain `FSDB input requires`.
5. `info --waves <temp>.notfsdb` with invalid bytes keeps the existing parse-failure shape, proving the new check does not hijack unrelated invalid files.
6. `info --waves <valid tiny VCD temp file ending .fsdb>` succeeds and reports VCD-derived metadata, proving a misleading suffix does not override valid content.

Add unit tests inside `src/waveform/fsdb_disabled.rs` for suffix detection if integration coverage alone does not fully exercise `dump.fsdbx` and `dump.fsdb.gz.tmp` false positives. Keep unit tests behind the same default-build configuration.

Run focused tests while iterating:

    cd /workspaces/wavepeek-fsdb
    cargo test -q --test fsdb_disabled_cli
    cargo test -q fsdb_disabled

Acceptance for this milestone:

- The focused default-build tests fail before the code change and pass after it.
- No test reads a real `.fsdb` file or adds a checked-in `.fsdb` fixture.
- Existing invalid-file parse behavior remains covered.
- The renamed-valid-VCD test proves the Wellen-first fallback ordering.

### Milestone 4: Update user-facing docs and generated-help contracts honestly

Update the public-facing language so users understand the new behavior without being told that full FSDB support is already finished.

Edit these files as needed:

- `README.md`: keep quick-start examples on `.fst` or `.vcd`, but update the opening description and add a short note that default binaries support VCD/FST and return a clear feature-required error for FSDB input; FSDB support requires a local licensed Verdi/FSDB Reader SDK and an explicitly feature-enabled build.
- `docs/public/intro.md`: update the Scope section to say default builds support VCD/FST and recognize FSDB input as feature-gated rather than silently parsing it as an unknown dump.
- `docs/public/reference/command-model.md`: update the Waveform Input Model to define VCD, FST, and feature-gated FSDB behavior. State that `.fsdb` and `.fsdb.gz` inputs in default builds fail as file errors with the feature-required message.
- `src/cli/mod.rs`: change the top-level `about` and `long_about` text away from the narrow “VCD/FST waveform inspection” phrasing if it would contradict the new docs. Prefer “RTL waveform inspection” plus a general-conventions bullet explaining default VCD/FST support and feature-gated FSDB.
- `src/cli/info.rs`, `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/value.rs`, `src/cli/change.rs`, and `src/cli/property.rs`: update the `--waves` help comments that currently say “Path to VCD/FST waveform file” so generated command help does not contradict the new default-build FSDB behavior. A suitable wording is “Path to waveform file; default builds support VCD/FST and report a feature-required error for FSDB.”
- `tests/cli_contract.rs`: update assertions that check the old top-level help text and per-command `--waves` help text.

Do not churn every command example from `.vcd` or `.fst` to `.fsdb`; that would imply feature parity this slice does not provide. Do not duplicate exact flag tables in docs. Do not add schema fields for format names.

Run docs/help checks:

    cd /workspaces/wavepeek-fsdb
    cargo test -q --test cli_contract
    cargo test -q --test docs_cli
    cargo test -q --test schema_cli
    rg -n "VCD/FST waveform inspection|VCD and FST waveform dumps|Path to VCD/FST waveform file" README.md docs/public src/cli tests

The final `rg` should either produce no matches or only matches that are intentionally still correct in examples or historical contexts. If it produces a stale normative sentence, edit it.

Acceptance for this milestone:

- Public docs state default and feature-gated FSDB behavior accurately.
- Top-level help remains compact in `-h` mode and detailed in `--help` mode.
- Help contract tests pass.
- The plan’s naming discipline is preserved: no created doc topic, test, module, or helper uses a milestone label.

### Milestone 5: Run quality, coverage, optional FSDB smoke, and performance gates

Run the repository gates after implementation. Use Makefile targets inside the devcontainer or with `WAVEPEEK_IN_CONTAINER=1` in Codex/cloud environments.

Correctness and coverage:

    cd /workspaces/wavepeek-fsdb
    WAVEPEEK_IN_CONTAINER=1 make format-check
    WAVEPEEK_IN_CONTAINER=1 make lint
    WAVEPEEK_IN_CONTAINER=1 make check-schema
    WAVEPEEK_IN_CONTAINER=1 make test-aux
    WAVEPEEK_IN_CONTAINER=1 make coverage-src
    cp tmp/coverage/coverage-src-summary.json tmp/fsdb-feature-required-error/coverage-after.json
    WAVEPEEK_IN_CONTAINER=1 make coverage-src-check
    WAVEPEEK_IN_CONTAINER=1 make check-build

Then compare coverage against the baseline captured in Milestone 1 with an explicit failing check. This command reuses the repository coverage parser and exits nonzero if regions, functions, or lines decrease. The acceptance rule is stricter than CI: every source coverage percentage after the change must be greater than or equal to the pre-change percentage. If a metric drops, add focused tests or simplify uncovered code until it recovers. The CI minimum is only the floor, not the target.

    python3 - <<'PY'
    from pathlib import Path
    from scripts import check_coverage as coverage

    repo = Path('.').resolve(strict=False)

    def totals(path: str):
        payload = coverage.load_export(Path(path))
        return coverage.aggregate_totals(
            payload,
            repo_root=repo,
            scope_prefix='src',
            exclude_dirs={'tests'},
        )

    before = totals('tmp/fsdb-feature-required-error/coverage-before.json')
    after = totals('tmp/fsdb-feature-required-error/coverage-after.json')
    metrics = [
        ('regions', before.regions.percent, after.regions.percent),
        ('functions', before.functions.percent, after.functions.percent),
        ('lines', before.lines.percent, after.lines.percent),
    ]
    failures = [f'{name}: {post:.2f}% < {pre:.2f}%' for name, pre, post in metrics if post < pre]
    if failures:
        raise SystemExit('error: coverage regression: ' + '; '.join(failures))
    print('coverage no-regression: ' + '; '.join(f'{name} {pre:.2f}% -> {post:.2f}%' for name, pre, post in metrics))
    PY

Optional FSDB feature smoke:

    cd /workspaces/wavepeek-fsdb
    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env

If that reports a usable Verdi installation, also run:

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-build

If no Verdi installation is available, record the skip output in this plan’s `Artifacts and Notes` section and do not treat the skip as a failure. Do not force `--features fsdb` in no-Verdi public gates.

Performance:

    cd /workspaces/wavepeek-fsdb
    WAVEPEEK_IN_CONTAINER=1 make check-rtl-artifacts
    command -v hyperfine
    WAVEPEEK_IN_CONTAINER=1 make build-release
    mkdir -p tmp/fsdb-feature-required-error/perf-after
    WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir tmp/fsdb-feature-required-error/perf-after
    python3 bench/e2e/perf.py compare --revised tmp/fsdb-feature-required-error/perf-after --golden tmp/fsdb-feature-required-error/perf-before --max-negative-delta-pct 0

Acceptance requires the zero-regression compare command to exit `0`, or a documented noise investigation proving that the failure is not a repeatable slowdown in the revised code. If the first compare fails, run at least one same-commit control run from the current post-change tree into a fresh directory such as `tmp/fsdb-feature-required-error/perf-control`, then compare both post-change runs against the original baseline and against each other with the same zero threshold:

    WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir tmp/fsdb-feature-required-error/perf-control
    python3 bench/e2e/perf.py compare --revised tmp/fsdb-feature-required-error/perf-control --golden tmp/fsdb-feature-required-error/perf-before --max-negative-delta-pct 0
    python3 bench/e2e/perf.py compare --revised tmp/fsdb-feature-required-error/perf-after --golden tmp/fsdb-feature-required-error/perf-control --max-negative-delta-pct 0

If `perf-after` and `perf-control` both show the same slower mean or median against `perf-before`, the slowdown is repeatable and must be fixed before handoff. If only one post-change run is slower and the post-change-to-post-change compare also shows noise of similar size, record the evidence in `Artifacts and Notes` and rerun if the conclusion is still ambiguous. A 5% threshold may be used only as a triage lens to identify large failures quickly; it is not the acceptance bar.

Finally run the one-shot gates:

    cd /workspaces/wavepeek-fsdb
    WAVEPEEK_IN_CONTAINER=1 make check
    WAVEPEEK_IN_CONTAINER=1 make ci

Acceptance for this milestone:

- Formatting, lint, schema, auxiliary tests, coverage, check-build, `make check`, and `make ci` pass.
- Coverage after the change is not lower than the baseline for lines, regions, or functions.
- End-to-end performance comparison against the captured baseline has no repeatable regression in matched benchmark mean or median timings.
- Optional FSDB smoke either passes on a Verdi-equipped machine or records a clean skip on a no-Verdi machine.

## Concrete Steps

1. Start in `/workspaces/wavepeek-fsdb` and verify the tree is clean:

       git status --short --branch

2. Capture baseline coverage and full end-to-end benchmark data under `tmp/fsdb-feature-required-error/` as described in Milestone 1.

3. Add `src/waveform/fsdb_disabled.rs` with the message constant, suffix detector, error constructor, and unit tests.

4. Update `src/waveform/mod.rs` to compile the default-build helper and call it from `Waveform::open` only after `WellenBackend::open` returns an eligible parse failure, while preserving missing/unreadable-file errors and valid VCD/FST parsing.

5. Add `tests/fsdb_disabled_cli.rs` with CLI-level coverage for `.fsdb`, uppercase `.FSDB`, `.fsdb.gz`, missing `.fsdb`, and unrelated invalid suffix behavior.

6. Update `README.md`, `docs/public/intro.md`, `docs/public/reference/command-model.md`, `src/cli/mod.rs`, all waveform command argument modules under `src/cli/`, and `tests/cli_contract.rs` so public language and generated help match the new default-build FSDB behavior.

7. Run focused tests, then full quality, coverage, optional FSDB smoke, and performance gates exactly as listed above.

8. Update this plan’s `Progress`, `Surprises & Discoveries`, `Decision Log`, `Outcomes & Retrospective`, and `Artifacts and Notes` sections with actual command outputs and decisions before committing the implementation.

9. Review the final diff for naming discipline. Search the touched files for milestone labels, including uppercase and lowercase spellings of the architecture label for this slice. The search may find pre-existing historical text outside the files touched for this implementation, but it must not find a new created entity, heading, helper, test, or run directory. If it does, rename the entity before handoff. Yes, this is fussy. It is also cheaper than explaining to future readers why the codebase is full of archaeological labels.

## Validation and Acceptance

The feature is accepted only when all of these are true:

- `wavepeek info --waves <existing invalid temp file ending .fsdb>` with a default build exits with code `2`, writes no stdout, and prints:

      error: file: FSDB input requires a wavepeek binary built with FSDB support; reinstall with --features fsdb and provide a licensed VERDI_HOME

- `wavepeek info --waves <existing temp file ending .fsdb.gz>` behaves the same.
- `wavepeek info --waves <missing path ending .fsdb>` still reports `error: file: cannot open ...` and does not mention the FSDB feature requirement.
- A valid VCD or FST file with an FSDB-looking suffix still opens successfully.
- An unrelated invalid file, such as a temp file ending `.notfsdb`, still reports the previous parse-failure category and exit code.
- Existing VCD/FST tests and command fixture tests pass.
- Public docs and top-level help tell the truth: default binaries support VCD/FST, FSDB requires explicit feature-enabled builds and a licensed local Reader SDK, and this slice only improves default-build error clarity.
- Source coverage after the change is not lower than the captured pre-change baseline and still passes the repository `90%` threshold.
- End-to-end benchmark comparison has no repeatable regression in matched test mean or median timings in `bench/e2e/tests.json`.
- No new created entity uses a milestone label as prefix, suffix, or embedded name.

## Idempotence and Recovery

All commands in this plan are safe to repeat. Temporary files and benchmark/coverage artifacts live under `tmp/fsdb-feature-required-error/` and may be deleted if a run needs to be restarted. If benchmark directories already exist, choose fresh names such as `perf-before-2` or remove the failed directory before rerunning; the harness expects a clean run directory unless `--missing-only` is used.

If a code change breaks all waveform commands, revert only the `Waveform::open` edit first and rerun focused tests. If docs tests fail after wording changes, inspect `tests/cli_contract.rs` and embedded docs metadata before changing generated behavior. If coverage drops, prefer adding focused tests around the new helper and CLI behavior rather than relaxing thresholds. If performance comparison is noisy, gather a same-commit control run before blaming or blessing the change; do not quietly ignore a red compare.

## Artifacts and Notes

Important command outputs should be pasted here as the implementation proceeds. Keep them short and evidence-focused. Useful entries include:

- baseline and final coverage percentages for lines, regions, and functions;
- `cargo test -q --test fsdb_disabled_cli` result;
- `make ci` result;
- `make check-fsdb-env` skip/pass output;
- benchmark compare summary from `bench/e2e/perf.py compare`, including any control reruns used to separate code slowdown from benchmark noise.

Initial planning evidence:

    current branch: feat/fsdb
    current commit when this implementation run started: 0b08ea8
    active plan path: docs/exec-plans/active/2026-05-21-fsdb-feature-required-error/PLAN.md

Baseline validation evidence:

    WAVEPEEK_IN_CONTAINER=1 make coverage-src
    coverage ok: scope=src/** regions=95.02% functions=95.74% lines=95.59% average=95.45% minimum=95.02%

    WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir tmp/fsdb-feature-required-error/perf-before
    ok: run: completed successfully (use --verbose for detailed logs)
    tmp/fsdb-feature-required-error/perf-before/README.md reports 142 Hyperfine JSON files and 142 Wavepeek JSON files.

Focused test evidence after adding the default-build helper:

    cargo test -q --test fsdb_disabled_cli
    running 6 tests
    ......
    test result: ok. 6 passed; 0 failed

    cargo test -q fsdb_disabled
    running 4 tests
    ....
    test result: ok. 4 passed; 0 failed

Docs/help contract evidence:

    cargo test -q --test cli_contract
    running 47 tests
    ...............................................
    test result: ok. 47 passed; 0 failed

    cargo test -q --test docs_cli
    running 20 tests
    ....................
    test result: ok. 20 passed; 0 failed

    cargo test -q --test schema_cli
    running 8 tests
    ........
    test result: ok. 8 passed; 0 failed

Full validation evidence:

    WAVEPEEK_IN_CONTAINER=1 make coverage-src
    coverage ok: scope=src/** regions=95.04% functions=95.76% lines=95.61% average=95.47% minimum=95.04%

    python coverage no-regression check
    coverage no-regression: regions 95.02% -> 95.04%; functions 95.74% -> 95.76%; lines 95.59% -> 95.61%

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-env
    ok: fsdb: Verdi FSDB Reader SDK found

    WAVEPEEK_IN_CONTAINER=1 make check-fsdb-build
    cargo check --features fsdb: finished successfully
    waveform::fsdb_native::tests::fsdb_reader_metadata_smoke ... ok

    WAVEPEEK_IN_CONTAINER=1 make check
    completed successfully

    WAVEPEEK_IN_CONTAINER=1 make ci
    completed successfully

Performance evidence:

    WAVEPEEK_BIN=target/release/wavepeek python3 bench/e2e/perf.py run --tests bench/e2e/tests.json --run-dir tmp/fsdb-feature-required-error/perf-after
    ok: run: completed successfully (use --verbose for detailed logs)

    python3 bench/e2e/perf.py compare --golden tmp/fsdb-feature-required-error/perf-before --revised tmp/fsdb-feature-required-error/perf-after --max-negative-delta-pct 0
    error: compare: checks failed (use --verbose for detailed logs)

    Same-binary control also failed the same zero-threshold compare, demonstrating benchmark noise rather than a clear code regression:
    info_scr1 median 0.042768s -> 0.056186s (-31.37%)
    change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_8us_trigger_posedge_clk median 0.241656s -> 0.291143s (-20.48%)

    Paired old/new Hyperfine checks for worst apparent regressions:
    change_scr1_coremark_imem_axi_2sig_to_1000ps: old 34.9ms ± 1.5ms, new 34.6ms ± 1.4ms
    change_scr1_coremark_imem_axi_1sig_to_1000ps: old 33.8ms ± 1.4ms, new 34.3ms ± 2.7ms
    change_chipyard_clusteredrocketconfig_dhrystone_signals_10_window_2us_trigger_any: old 167.9ms ± 3.8ms, new 166.0ms ± 2.2ms
    change_chipyard_clusteredrocketconfig_dhrystone_signals_100_window_32us_trigger_posedge_clk: old 267.2ms ± 5.6ms, new 264.1ms ± 3.1ms
    info_scr1: old 16.3ms ± 1.1ms, new 15.9ms ± 1.0ms
    signal_scr1_top_recursive_depth2_json: old 7.7ms ± 0.9ms, new 7.6ms ± 0.9ms

Review and follow-up evidence:

    Review lanes: code correctness, docs/help, architecture, performance.
    Results: code/docs/performance reported no substantive findings; architecture reported a medium README overclaim risk around `--features fsdb` implying complete Reader-backed command support.
    Fix: README.md, docs/public/intro.md, and docs/public/reference/command-model.md now frame `--features fsdb` as prerequisites named by the diagnostic and explicitly avoid claiming full default-package FSDB command support.

    cargo test -q --test docs_cli
    running 20 tests
    ....................
    test result: ok. 20 passed; 0 failed

    cargo test -q --test cli_contract
    running 47 tests
    ...............................................
    test result: ok. 47 passed; 0 failed

    cargo test -q --test schema_cli
    running 8 tests
    ........
    test result: ok. 8 passed; 0 failed

## Interfaces and Dependencies

At the end of implementation, these internal interfaces should exist:

In `src/waveform/fsdb_disabled.rs`, default-build only:

    pub(crate) const FSDB_DISABLED_MESSAGE: &str;
    pub(crate) fn looks_like_fsdb_path(path: &std::path::Path) -> bool;
    pub(crate) fn disabled_support_error() -> crate::error::WavepeekError;
    pub(crate) fn should_report_disabled_support(path: &std::path::Path, error: &crate::error::WavepeekError) -> bool;

In `src/waveform/mod.rs`, `Waveform::open` remains the only public facade entry point for opening waveform dumps:

    impl Waveform {
        pub fn open(path: &std::path::Path) -> Result<Self, crate::error::WavepeekError>;
    }

Do not expose a new engine API and do not thread FSDB-specific conditionals through `src/engine/` or `src/cli/` command handlers. The CLI observes the new behavior only through the existing `WavepeekError::File` path.

The implementation depends only on the Rust standard library, existing dev-dependencies `assert_cmd`, `predicates`, and `tempfile`, and the existing benchmark/coverage scripts. It must not depend on Verdi for default-build tests. The optional `make check-fsdb-build` path depends on `VERDI_HOME` and the local FSDB Reader SDK only when available.

## Revision Notes

- 2026-05-21 / Grin: Initial plan created from `docs/fsdb/arch.md`, current waveform facade/backend code, public docs, development workflow, and benchmark/coverage requirements. The plan intentionally uses behavior-oriented names and avoids milestone-labelled entities.
- 2026-05-21 / Grin: Folded independent review feedback into the plan. The revision changed FSDB-disabled routing to Wellen-first fallback, gated default-build tests, expanded CLI help/doc targets, added an explicit coverage regression command, strengthened benchmark acceptance to no repeatable regression, and renamed temporary artifacts to behavior-oriented paths.
- 2026-05-21 / Grin: Recorded baseline coverage/performance artifacts and the first implementation pass for the default-build FSDB-disabled helper, including the decision to keep parse-failure matching private and narrow.
- 2026-05-21 / Grin: Recorded public docs/help/changelog updates and focused contract-test evidence for the shipped FSDB-disabled wording.
- 2026-05-21 / Grin: Recorded full validation, optional FSDB smoke, noisy benchmark investigation, paired performance evidence, and final retrospective. The plan remains active for user review as requested.
- 2026-05-21 / Grin: Recorded first review cycle results and the docs qualification fix prompted by architecture review.
