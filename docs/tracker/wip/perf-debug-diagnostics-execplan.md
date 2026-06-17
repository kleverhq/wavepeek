# Add DEBUG=1 Performance Diagnostics

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek` users and maintainers need a useful answer to “why was this waveform query slow?” without changing normal command output. After this change, running a waveform command with `DEBUG=1` will add debug diagnostics describing coarse performance phases. In human output mode those diagnostics will appear on stderr; in `--json` mode they will appear in the existing `diagnostics` array. Running the same command without `DEBUG=1` will keep stdout and stderr unchanged.

The behavior can be observed with a small fixture command such as `DEBUG=1 cargo run -- value --waves tests/fixtures/cli/m2_core.vcd --at 0ns --signals top.clk --json`. The JSON envelope should still contain the normal command data, but its `diagnostics` array should include `kind: "debug"` records with `WPK-D1xxx` performance codes and structured `details` objects.

## Non-Goals

This change does not add a new public CLI flag. Debug performance diagnostics are enabled only by `DEBUG=1`, which already exists as the maintainer debug switch. This change does not alter fatal-error behavior: when a command fails before returning a successful `CommandResult`, stdout remains empty and stderr keeps the existing `fatal: <category>: <message>` shape. This change does not expose signal values, waveform file paths, or signal names in performance diagnostics. This change does not benchmark or optimize the backend; it only records low-overhead elapsed times and counts that help maintainers decide where later work belongs.

## Progress

- [x] (2026-06-17T20:45:25Z) Confirmed the repository starts clean on branch `feat/perf-diag` and issue #22 requests opt-in backend performance diagnostics.
- [x] (2026-06-17T20:45:25Z) Confirmed `DEBUG=1` currently only allows hidden `change --tune-*` controls and does not print debug output.
- [x] (2026-06-17T20:45:25Z) Recorded design decisions from the maintainer discussion: use `DEBUG=1`, only waveform commands, no fatal-output changes, `kind: debug`, `WPK-D####` codes, `D0001` generic debug message, `D1xxx` performance events, code as the discriminator for `details` shape.
- [x] (2026-06-17T20:55:00Z) Commit this ExecPlan before feature code changes.
- [x] (2026-06-17T22:05:00Z) Extend the diagnostic model, human renderer, machine-output docs, schema artifact, and schema checker to support debug diagnostics and optional structured details.
- [x] (2026-06-17T22:05:00Z) Add a low-overhead performance recorder that is enabled only when `DEBUG=1` and can produce `Diagnostic` records on successful waveform commands.
- [x] (2026-06-17T22:05:00Z) Instrument all waveform commands: `info`, `scope`, `signal`, `value`, `change`, and `property`.
- [x] (2026-06-17T22:05:00Z) Add tests proving human stderr diagnostics with `DEBUG=1`, JSON diagnostics with structured `details`, all waveform commands receiving debug diagnostics, and no debug diagnostics for the `schema` helper command.
- [x] (2026-06-17T22:10:00Z) Run focused tests and repository gate `just check`; all passed.
- [x] (2026-06-17T22:18:00Z) Commit the implementation as `feat(debug): add waveform performance diagnostics`.
- [x] (2026-06-17T22:35:00Z) Run focused review lanes for code/tests, docs/schema, and performance/architecture. Code/tests returned no substantive findings. Docs/schema found missing `phase_count` requiredness. Performance/architecture found two low-severity attribution issues.
- [x] (2026-06-17T22:45:00Z) Apply review fixes: require summary `phase_count` in schema/checker/tests, rename value's selection phase from `signal.resolve` to `signal.select`, and make edge-fast change diagnostics report dispatch/fallback risk with `selected_engine` and `may_fallback` metrics. `just check` passed after fixes.
- [x] (2026-06-17T22:55:00Z) Commit review fixes as `fix(debug): tighten performance diagnostic schema`.
- [x] (2026-06-17T23:05:00Z) Run a final control review over `main...HEAD`; it returned no substantive findings.
- [x] (2026-06-17T23:12:00Z) Push branch `feat/perf-diag` and open pull request #31.

## Surprises & Discoveries

- Observation: `DEBUG=1` exists but currently prints nothing by itself.
  Evidence: `rg` found the only runtime `DEBUG` check in `src/cli/mod.rs::is_debug_mode_enabled`, where it gates hidden `change --tune-*` flags. Explicit stderr output is limited to diagnostics in `src/output.rs` and fatal errors in `src/main.rs`.

- Observation: The current JSON schema explicitly rejects debug diagnostics and extra fields on diagnostics.
  Evidence: `schema/wavepeek_v1.json` had diagnostic `kind` enum `info`, `warning`, `error`; `code` pattern `^WPK-[WE][0-9]{4}$`; and `additionalProperties: false` for diagnostic objects.

- Observation: Pre-commit auxiliary tests failed during `git commit` because temporary-repository Git commands inherited hook-local `GIT_*` environment variables.
  Evidence: `git commit` initially failed in `tools/docs/test_publish_docs.py`, then `tools/release/test_publish_crate.py`, then `tools/repo/test_setup_github_auth.py`; direct test runs passed outside the hook. Clearing local Git environment variables fixed those tests and was committed as `test(tools): isolate git helper environment`.

- Observation: Tests that intentionally use `DEBUG=1` for hidden `change --tune-*` flags now receive debug diagnostics.
  Evidence: `tests/change_opt_equivalence.rs` and tune-mode cases in `tests/change_vcd_fst_parity.rs` compared entire diagnostics arrays and failed on nondeterministic `WPK-D1xxx` durations. Those assertions now compare non-debug diagnostics for equivalence.

- Observation: Review found that docs promised `phase_count` in summary details, but the schema did not require it.
  Evidence: Docs/schema review reported `schema/wavepeek_v1.json` accepted `WPK-D1003` summary details without `phase_count`. The schema, checker, and schema tests now require it.

- Observation: Review found two low-severity attribution issues in performance phase names/metrics.
  Evidence: Performance review noted that value `signal.resolve` did not perform backend resolution and that edge-fast may include fallback work. Value now uses `signal.select`; edge-fast uses `change.engine.edge-fast.dispatch` and emits `selected_engine` plus `may_fallback`.

## Decision Log

- Decision: Enable performance diagnostics with `DEBUG=1` instead of a new CLI flag.
  Rationale: The feature is primarily for maintainer diagnostics and user-provided performance reports. `DEBUG=1` already exists as the maintainer debug switch, so reusing it avoids adding another opt-in surface.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

- Decision: Emit debug diagnostics only for successful waveform commands and do not change fatal behavior.
  Rationale: The existing fatal contract keeps stdout empty and prints only `fatal: <category>: <message>` on stderr. Preserving that avoids surprising scripts and keeps this slice smaller.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

- Decision: Use `kind: debug` and require `WPK-D####` codes for debug diagnostics.
  Rationale: `kind` describes transport and severity behavior, while `code` is the stable discriminator for the shape of `details`. This avoids free-form debug blobs that scripts cannot safely consume.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

- Decision: Reserve `WPK-D0001` for generic debug messages and use `WPK-D1xxx` for performance events.
  Rationale: A generic code leaves room for future non-performance debug text, while `D1xxx` gives performance diagnostics a recognizable range.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

- Decision: Use a small set of performance record codes rather than a new code for every phase.
  Rationale: `WPK-D1001` can represent performance context, `WPK-D1002` can represent any timed phase with `details.phase`, and `WPK-D1003` can represent a summary. New phase names can be added without schema churn, while the code still discriminates the record shape.
  Date/Author: 2026-06-17 / Grin.

- Decision: Do not include signal values, signal names, or waveform file paths in performance details.
  Rationale: Issue #22 explicitly forbids signal value exposure, and names/paths may leak proprietary design context. Counts, backend names, formats, phase names, statuses, and durations are enough for performance diagnosis.
  Date/Author: 2026-06-17 / Grin with maintainer confirmation.

## Outcomes & Retrospective

Implementation, review, push, and PR creation are complete. The implementation adds `kind: debug`, `WPK-D####` code handling, optional structured `details`, a `PerfDiagnostics` recorder, waveform backend context helpers, instrumentation for all six waveform commands, public machine-output docs, schema contract updates, and focused tests. Review found one medium docs/schema issue and two low performance-attribution issues; all have been fixed. The final control review returned no substantive findings. Pull request #31 is open.

## Context and Orientation

The repository is a Rust CLI named `wavepeek`. Waveform commands inspect VCD, FST, and optionally FSDB waveform files. The CLI layer in `src/cli/` parses command arguments and dispatches to the engine layer in `src/engine/`. The engine layer opens waveform files through the backend-neutral facade in `src/waveform/mod.rs`, performs command-specific work, and returns `CommandResult`. The output layer in `src/output.rs` prints either human output or a JSON envelope.

A diagnostic is a non-fatal message attached to a successful command. In human mode diagnostics go to stderr. In `--json` mode diagnostics go into the JSON envelope field named `diagnostics`. Fatal errors are different: `src/main.rs` prints the formatted `WavepeekError` to stderr and exits non-zero. This plan does not change fatal errors.

The current diagnostic type lives in `src/diagnostic.rs`. It has `DiagnosticKind::{Info, Warning, Error}`, optional `code`, and `message`. Warning diagnostics use `WPK-W####` codes and error diagnostics use `WPK-E####` codes. Info diagnostics currently have no code. The current JSON schema lives in `schema/wavepeek_v1.json`, and the runtime embeds that exact artifact through `src/schema_contract.rs`. The helper `tools/schema/check_schema_contract.py` validates that `wavepeek schema` matches the artifact and that the diagnostic schema has the expected shape.

The existing debug mode is `DEBUG=1`, implemented in `src/cli/mod.rs::is_debug_mode_enabled`. Today it only allows hidden `change` tuning arguments. With this plan, `DEBUG=1` will also enable performance debug diagnostics for commands that require `--waves`: `info`, `scope`, `signal`, `value`, `change`, and `property`.

A performance phase is a coarse span of command execution measured with `std::time::Instant`. Coarse means one span around a meaningful chunk, such as opening the waveform or collecting candidate timestamps. The implementation must avoid per-sample hot-loop timing unless the loop is already command-level and the overhead is just one elapsed-time measurement around the whole loop.

## Open Questions

There are no product-design questions left from the maintainer discussion. Implementation details may still be adjusted if tests or Rust ownership expose a simpler shape, but the user-visible contract is fixed by the Decision Log.

## Plan of Work

First, extend the diagnostic contract. In `src/diagnostic.rs`, add `DiagnosticKind::Debug`, a `DebugDiagnosticCode` enum with `GenericMessage`, `PerformanceContext`, `PerformancePhase`, and `PerformanceSummary`, and a `details: Option<serde_json::Value>` field on `Diagnostic`. Add constructors for generic debug diagnostics and debug diagnostics with details. Existing info, warning, and error constructors should continue to produce the same JSON as before when `details` is absent. In `src/output.rs`, render debug diagnostics in human mode as `debug[WPK-D####]: <message>`. Update `docs/public/reference/machine-output.md` to describe `debug`, `WPK-D####`, and optional details. Update `schema/wavepeek_v1.json` and `tools/schema/check_schema_contract.py` so the schema accepts debug diagnostics, requires D-codes for debug diagnostics, still forbids codes for info diagnostics, and allows optional object-valued `details` on diagnostics.

Second, add a performance recorder module, preferably `src/perf_diag.rs`, and expose it from `src/lib.rs`. The recorder should be cheap when disabled: if `DEBUG` is not exactly `1`, it should avoid calling `Instant::now` for phases. A practical shape is `PerfDiagnostics::for_command(CommandName)`, which checks `DEBUG=1`, stores a command name and a start time only when enabled, and has methods to record context, timed phases, and summary diagnostics. Timed phases can be measured by calling a method that accepts a closure, records elapsed nanoseconds if enabled, and returns the closure result unchanged. Details should use JSON objects with `domain: "performance"`, `event: "context" | "phase" | "summary"`, and fields such as `phase`, `duration_ns`, `status`, `command`, `backend`, `format`, and `metrics`. Counts such as `signals`, `times`, `rows`, and `truncated` can live inside a nested `metrics` object. Do not include signal values, signal names, or file paths.

Third, provide backend context without leaking paths. In `src/waveform/mod.rs`, add methods such as `backend_name(&self) -> &'static str` and `format_name(&self) -> &'static str`. For the Wellen backend in `src/waveform/wellen_backend.rs`, convert `wellen::FileFormat` to stable strings such as `"vcd"`, `"fst"`, and `"unknown"` if needed. For FSDB builds in `src/waveform/fsdb_backend.rs`, return backend `"fsdb"` and format `"fsdb"`. Keep this small and behind the facade.

Fourth, instrument each waveform command. Each run function should create the recorder after validating cheap argument errors that should not produce a successful command anyway. Wrap `Waveform::open` or `open_shared_waveform` in `backend.open`. After a waveform is open, record performance context with command, backend, and format. Wrap metadata loading in `metadata.load`. For `scope`, wrap hierarchy traversal in `hierarchy.load` and regex filtering/limit application in command phases if useful. For `signal`, wrap direct or recursive signal loading as `signal.list`. For `value`, wrap signal resolution as `signal.resolve`, time parsing as `time.parse`, and the full sampling loop as `value.sample`. For `change`, wrap metadata, signal resolution, expression binding, time parsing, candidate collection, selected engine execution, and result finalization. For `property`, wrap metadata, time parsing, expression binding, candidate collection, schedule building, evaluation loop, and result finalization. At the end of a successful command, extend the existing diagnostics vector with recorder summary diagnostics. Existing warning diagnostics must remain present and deterministic; debug diagnostics should be appended after command warnings to avoid perturbing warning positions for clients that inspect the first warning.

Fifth, test the behavior. Add or update unit tests for diagnostic serialization and human rendering. Add integration tests using existing small CLI fixtures to prove: without `DEBUG=1` output remains unchanged; with `DEBUG=1` human mode sends debug diagnostics to stderr while stdout still contains the normal payload; with both `DEBUG=1` and `--json`, debug diagnostics appear in the JSON diagnostics array with `WPK-D1001`, `WPK-D1002`, and `WPK-D1003` records and object-valued details; helper commands such as `schema` do not emit debug diagnostics under `DEBUG=1`. Also validate schema changes with `just check-schema`.

## Concrete Steps

Work from the repository root `/workspaces/feat-perf-diag`.

1. Commit this plan before implementation:

    git add docs/tracker/wip/perf-debug-diagnostics-execplan.md
    git commit -m "docs(tracker): plan debug performance diagnostics"

2. Implement the diagnostic contract changes in `src/diagnostic.rs`, `src/output.rs`, `docs/public/reference/machine-output.md`, `schema/wavepeek_v1.json`, and `tools/schema/check_schema_contract.py`. Run focused tests:

    cargo test diagnostic
    cargo test output
    just check-schema

3. Implement `src/perf_diag.rs`, expose it from `src/lib.rs`, add backend context helpers in `src/waveform/mod.rs` and concrete backend modules, and instrument all six waveform command run functions.

4. Add integration/unit tests. Focused commands should include:

    cargo test --test value_cli debug
    cargo test --test cli_contract debug
    cargo test diagnostic
    just check-schema

   The exact test names may differ; use `cargo test debug` if the focused names are too broad.

5. Run formatting and broader checks:

    just format
    just check

   If `just check` cannot run because the environment is missing a declared dependency, capture the exact failure in this plan and run the narrowest meaningful alternatives.

6. Commit implementation changes with a conventional commit message, for example:

    git add src docs/public/reference/machine-output.md schema/wavepeek_v1.json tools/schema/check_schema_contract.py tests
    git commit -m "feat(debug): add waveform performance diagnostics"

7. Run read-only review lanes via subagents for code/schema/tests, docs/contract, and performance/architecture. Apply fixes in the main session only, rerun relevant tests, and commit any fixes.

8. Run a final independent control review. If clean, push the branch and open a PR:

    git push -u origin feat/perf-diag
    gh pr create --repo kleverhq/wavepeek --title "feat(debug): add waveform performance diagnostics" --body-file tmp/perf-debug-pr.md

## Validation and Acceptance

Acceptance is observable from CLI behavior. Without `DEBUG=1`, a waveform command must produce the same stdout, stderr, and JSON diagnostics it produced before. With `DEBUG=1`, a successful waveform command must still produce the same main payload, but debug diagnostics must be added. In human mode those diagnostics appear on stderr, for example:

    debug[WPK-D1001]: perf: context command=value backend=wellen format=vcd
    debug[WPK-D1002]: perf: backend.open 1.234ms
    debug[WPK-D1003]: perf: total 2.345ms

In JSON mode, the same command must emit one valid JSON envelope on stdout with records like:

    {
      "kind": "debug",
      "code": "WPK-D1002",
      "message": "perf: backend.open 1.234ms",
      "details": {
        "domain": "performance",
        "event": "phase",
        "phase": "backend.open",
        "duration_ns": 1234000,
        "status": "ok"
      }
    }

The exact durations are nondeterministic and must not be asserted exactly. Tests should assert structure, code presence, phase names, and that `duration_ns` is a number. Tests must also assert that no diagnostic exposes signal values.

The repository gate `just check` should pass before handoff. If it cannot run, the failure must be recorded with the exact command and output.

## Idempotence and Recovery

All edits are ordinary source, docs, schema, and test changes. The plan can be reread and steps can be rerun safely. If a schema check fails because `wavepeek schema` does not match `schema/wavepeek_v1.json`, either update the schema artifact deliberately or revert the schema-affecting code; do not leave runtime and artifact mismatched. If tests fail because durations vary, make assertions structural rather than exact. If a review finding requires changing the debug details shape, update `src/diagnostic.rs`, `schema/wavepeek_v1.json`, `tools/schema/check_schema_contract.py`, public docs, tests, and this ExecPlan together.

## Artifacts and Notes

Current baseline evidence:

    $ git status --short --branch
    ## feat/perf-diag

    $ rg -n "std::env::var\(\"DEBUG\"\)|eprintln!|println!|print!|dbg!|log::|tracing::" src -g '*.rs'
    src/cli/mod.rs:236:        println!(env!("CARGO_PKG_VERSION"));
    src/cli/mod.rs:241:        println!("wavepeek v{}", env!("CARGO_PKG_VERSION"));
    src/cli/mod.rs:253:    std::env::var("DEBUG")
    src/output.rs:49:    println!("{json}");
    src/output.rs:218:            DiagnosticKind::Info => eprintln!("info: {}", diagnostic.message()),
    src/output.rs:219:            DiagnosticKind::Warning => eprintln!(
    src/output.rs:226:            DiagnosticKind::Error => eprintln!(
    src/main.rs:7:            eprintln!("{error}");

## Interfaces and Dependencies

The implementation should use only the Rust standard library timing type `std::time::Instant` and the existing `serde` and `serde_json` dependencies. No logging framework should be introduced.

At the end of the feature, `src/diagnostic.rs` should expose debug constructors resembling:

    pub enum DebugDiagnosticCode {
        GenericMessage,
        PerformanceContext,
        PerformancePhase,
        PerformanceSummary,
    }

    impl Diagnostic {
        pub fn debug(code: DebugDiagnosticCode, message: impl Into<String>) -> Self;
        pub fn debug_with_details(
            code: DebugDiagnosticCode,
            message: impl Into<String>,
            details: serde_json::Value,
        ) -> Self;
    }

At the end of the feature, `src/perf_diag.rs` should expose a recorder resembling:

    pub struct PerfDiagnostics { ... }

    impl PerfDiagnostics {
        pub fn for_command(command: CommandName) -> Self;
        pub fn is_enabled(&self) -> bool;
        pub fn record_context(&mut self, backend: &'static str, format: &'static str);
        pub fn time_phase<T, E>(
            &mut self,
            phase: &'static str,
            f: impl FnOnce() -> Result<T, E>,
        ) -> Result<T, E>;
        pub fn push_phase_metrics(&mut self, phase: &'static str, metrics: serde_json::Value);
        pub fn finish(self) -> Vec<Diagnostic>;
    }

The exact method names may change to fit Rust ownership, but the observable diagnostics must match the contract above.

Plan revision note, 2026-06-17: Created the initial self-contained execution plan after maintainer confirmation of the debug-mode diagnostics design.

Plan revision note, 2026-06-17: Updated progress and discoveries after implementing debug performance diagnostics and passing `just check`.

Plan revision note, 2026-06-17: Recorded review findings and the follow-up fixes after the first review pass.

Plan revision note, 2026-06-17: Recorded the clean final control review before push and PR creation.

Plan revision note, 2026-06-17: Marked push and PR creation complete after opening pull request #31.
