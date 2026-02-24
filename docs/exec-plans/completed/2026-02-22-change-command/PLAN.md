# Implement `change --when` Event Triggers (SV2023-Aligned)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

This work upgrades `wavepeek change` from a split clocked/unclocked model (`--clk` present or absent) to one unified trigger model based on `--when "<event_expr>"`. After completion, users can express transition sampling using one SystemVerilog-style event expression model: wildcard non-edge changes, signal-specific changes, edge triggers (`posedge`, `negedge`, `edge`), and event unions via `or` or comma.

The new contract follows the SV2023 event-control meaning of `@(event_expr)` for the supported subset and keeps backward user intent intact: old unclocked behavior maps to non-edge event `*` (and is the default when `--when` is omitted), while old clean two-state clocked behavior maps to edge event `posedge <clk>`.

This revision also keeps previously aligned `at`-style output guarantees for `change`: canonical path identity in JSON, `@<time>` in human output only, and `--abs` for canonical human display without changing JSON.

`change` remains a delta-oriented command: event timestamps are only candidates, and rows are emitted only when sampled `--signals` values actually changed.

## Non-Goals

This plan does not implement the standalone `when` command execution. It prepares reusable event-expression infrastructure so future `when` work (and its planned rename) can consume the same parser/types/evaluation hooks.

This plan does not change JSON envelope behavior outside `change` payload shape, and does not redesign already-shipped command contracts for `info`/`scope`/`signal`/`at`.

This plan intentionally defers full `logical_expr` execution inside `iff` clauses. The `event iff logical_expr` model and reusable interfaces are part of this plan; complete logical-expression evaluation semantics are tracked as follow-up work and must not be silently ignored.

## Progress

- [x] (2026-02-22 14:22Z) Drafted initial ExecPlan with milestones, acceptance criteria, and repository context.
- [x] (2026-02-22 14:41Z) Revised plan after review feedback: made command contract self-contained, added exact expected outputs/errors, and added milestone-level proof criteria.
- [x] (2026-02-22 14:52Z) Revised plan after independent control review: fixed canonical command invocation guidance and expanded JSON envelope acceptance checks.
- [x] (2026-02-22 16:02Z) Revised plan after final control findings: aligned milestone sequencing for help contract, added missing human/error acceptance cases, and added explicit exit-code verification steps.
- [x] (2026-02-23 20:16Z) Rebased plan contract on shipped `at` behavior: `--abs` for `change`, canonical JSON identity only, and human-only `@` time prefix.
- [x] (2026-02-23 23:55Z) Reframed contract from `--clk` mode split to unified `--when "<event_expr>"` model with SV2023-aligned edge semantics and explicit `iff` staging.
- [x] (2026-02-24 00:20Z) Tightened acceptance contract after review: added exact expected outputs for core `--when` cases, boundary rule at `--from`, explicit `or`/comma parity check, and unit-test matrix requirements for SV2023 + nine-state edge classification.
- [x] (2026-02-24 00:38Z) Closed second review gap set: added executable warning/parity checks, explicit `--abs` JSON-parity check, and overlap dedup proof requirement for union-trigger timestamps.
- [x] (2026-02-24 00:49Z) Fixed acceptance command robustness (shell quoting), pinned overlap timestamp contract at `30ns`, and added explicit human `--abs` assertion.
- [x] (2026-02-24 01:08Z) Added parser-binding contract for `iff` with union separators, CLI acceptance for `negedge`/`edge`, and executable acceptance checks for duplicate signal order plus default `--max=50` behavior.
- [x] (2026-02-24 01:34Z) Clarified delta semantics explicitly: candidate event timestamps without sampled signal deltas must not emit snapshots; updated normative rules and acceptance examples accordingly.
- [x] (2026-02-24 01:42Z) Added explicit JSON/human warning-parity acceptance for the zero-delta path (event fired but no sampled `--signals` change).
- [x] (2026-02-24 01:53Z) Removed two remaining ambiguities: defined per-signal baseline initialization rules for delta filtering (including mixed-prior cases) and specified temporary parenthesis-depth rule for deferred `iff` capture; added acceptance hooks for both.
- [x] (2026-02-24 02:07Z) Locked start-boundary semantics: `--from` is baseline-only (always sample/init at `--from`) and snapshot emission begins strictly after that timestamp.
- [x] (2026-02-24 02:19Z) Added omitted-`--from` and `--to == --from` acceptance coverage for baseline-only start semantics, and made collateral milestone explicitly require DESIGN doc boundary update.
- [x] (2026-02-24 02:26Z) Hardened omitted-`--from` acceptance by switching boundary proof to non-edge `*` trigger, avoiding false positives caused by edge previous-state requirements at dump start.
- [x] (2026-02-24 03:28Z) Added failing contract coverage for `change --when` in `tests/change_cli.rs`, updated `cli_contract` help assertions, and switched CLI surface/help text from `--clk` to `--when`/`--abs`.
- [x] (2026-02-24 03:49Z) Implemented reusable event-expression parsing/types (`EventExpr`/`EventTerm`/`EventKind`) plus waveform helpers for SV2023 edge classification, nine-state normalization, LSB-only edge detection, and delta-baseline update semantics.
- [x] (2026-02-24 04:07Z) Implemented end-to-end `change` runtime and rendering: default `--when "*"`, scoped resolution parity, union dedup, baseline-only `--from`, zero-delta suppression, warning parity, and `--abs` human-label behavior.
- [x] (2026-02-24 04:26Z) Synced collateral (`schema`, `DESIGN`, `README`, `CHANGELOG`), regenerated/validated schema checks, and passed full gates (`make test`, `make check`).
- [x] (2026-02-24 04:49Z) Completed review pass #1, fixed eager event-signal validation for empty in-range windows, and added integration coverage for scoped/unscoped variants.
- [x] (2026-02-24 05:03Z) Completed fresh independent review pass #2, fixed scoped canonical-token ambiguity hardening and `--max` short-circuit truncation path, and re-ran full quality gates.

## Surprises & Discoveries

- Observation: `change` CLI/dispatch exists but runtime is still a stub.
  Evidence: `src/engine/change.rs` returns `WavepeekError::Unimplemented("`change` command execution is not implemented yet")`.

- Observation: expression infrastructure exists only as scaffolding, not as executable grammar/evaluator.
  Evidence: `src/expr/lexer.rs`, `src/expr/parser.rs`, and `src/expr/eval.rs` currently provide shell-level structures and placeholders.

- Observation: there are no existing `iff` references in docs/code-level grammar.
  Evidence: repository search for `iff` returned no parser/token/evaluator integration points.

- Observation: current product docs still describe `change` with `--clk` and `when` with mandatory `--clk`.
  Evidence: `docs/DESIGN.md` sections 3.2.5 and 3.2.6 still model clocked/unclocked behavior with explicit clock arguments.

- Observation: `change` delta semantics require sampled-state advancement at every dump timestamp, not only at event-hit timestamps, to satisfy "strictly before t" comparisons.
  Evidence: initial implementation failed `change_negedge_wiring_is_end_to_end` (`10ns` row missing) until sampled baselines were updated on non-trigger timestamps (`5ns` baseline update enabled `1->0` delta at `10ns`).

- Observation: `cargo test expr::tests` only matches tests under `expr::tests::*` and does not match `expr::parser::tests::*` names.
  Evidence: milestone command initially ran zero tests until expression acceptance tests were mirrored in `src/expr/mod.rs` under `expr::tests::*`.

- Observation: `make update-schema` is artifact-sync only (`wavepeek schema` emits `include_str!`), so command/data contract expansion still requires editing `schema/wavepeek.json` in source.
  Evidence: first `make update-schema` produced no changes until `schema/wavepeek.json` was updated with `change` definitions.

## Decision Log

- Decision: Replace `change --clk` with `change --when "<event_expr>"` as the single trigger interface.
  Rationale: One trigger language scales beyond binary clocked/unclocked modes and mirrors user mental model from SV event controls.
  Date/Author: 2026-02-23 / OpenCode

- Decision: `--when` is optional and defaults to `*` (wildcard non-edge trigger on any tracked-signal change).
  Rationale: Preserves old unclocked behavior without requiring a new flag.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Implement edge detection per SV2023 table semantics for four-state values, then map nine-state values (`h/u/w/l/-`) to `x` before edge classification.
  Rationale: Matches expected hardware semantics and avoids ad-hoc transition handling.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Support event union by both `or` and comma as exact synonyms.
  Rationale: Matches expected SV event-expression ergonomics and requested examples.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Stage `iff` delivery: add grammar/AST/resolution hook now, defer complete logical-expression evaluation.
  Rationale: Unblocks shared event infrastructure for `change` and future `when` rename while controlling implementation risk.
  Date/Author: 2026-02-23 / OpenCode

- Decision: Keep `change` on the existing envelope pipeline (`CommandResult` -> `output::write`) and keep `@` in human output only.
  Rationale: Preserves deterministic output behavior already used by shipped commands.
  Date/Author: 2026-02-23 / OpenCode

- Decision: `change` snapshots are delta-only, even in edge-driven `--when` scenarios.
  Rationale: Command intent is change tracking; event expressions provide candidate sampling points, not unconditional sampling rows.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Delta baseline initialization is per sampled signal; emit when any comparable sampled signal changed, otherwise initialize missing baselines without forcing emission.
  Rationale: Preserves strict delta semantics while avoiding loss of real changes in mixed-prior scenarios.
  Date/Author: 2026-02-24 / OpenCode

- Decision: The exact `--from` timestamp is baseline-only and never emitted as a `change` row.
  Rationale: `change` reports deltas after the selected start checkpoint; this avoids boundary artifacts and matches operator expectation for range start behavior.
  Date/Author: 2026-02-24 / OpenCode

- Decision: For deferred `iff` support, `logical_expr_raw` capture uses separator detection at parenthesis depth `0`.
  Rationale: Provides deterministic parser behavior before full logical-expression parsing/validation is implemented.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Delta baseline state for sampled `--signals` is advanced on every in-window dump timestamp (after baseline), not only on event-hit timestamps.
  Rationale: Contract requires comparison against last known sampled state strictly before candidate timestamp; this must reflect non-trigger timestamps too.
  Date/Author: 2026-02-24 / OpenCode

- Decision: Keep scoped full-path rejection behavior by reusing existing scope-relative path composition (`<scope>.<token>`) for both `--signals` and event names.
  Rationale: Preserves established `at` semantics and keeps failure category as `error: signal:` without bespoke path-mode branches.
  Date/Author: 2026-02-24 / OpenCode

## Outcomes & Retrospective

`change` is now fully implemented and no longer a runtime stub. Users can query delta snapshots with one unified trigger surface: omitted `--when` defaults to `*`, explicit named non-edge triggers are supported, edge triggers (`posedge`/`negedge`/`edge`) follow SV2023 classification with nine-state normalization, and `or`/comma unions are equivalent with same-timestamp deduplication.

Reusable event-expression infrastructure was added under `src/expr/` (`EventKind`, `EventTerm`, `EventExpr`, parser segmentation/binding rules), so future `when` work can consume shared parsing/types. `iff` is intentionally staged: parser binding/capture is shipped now; runtime intentionally fails fast with `error: args: iff logical expressions are not implemented yet`.

`change` output and contract alignment are complete: JSON payload uses canonical paths only, human output uses `@<time> <display>=<value> ...`, `--abs` affects only human labels, warnings are parity-stable between JSON/human, and empty/truncation behavior is deterministic. Baseline semantics were locked and verified (`--from` checkpoint is baseline-only; omitted `--from` behaves the same at dump start).

Residual risk is concentrated in deferred `iff logical_expr` evaluation and any future expansion of expression syntax/precedence. Current parser intentionally stores raw `iff` text with temporary separator-depth rules; full logical parsing/evaluation must preserve current binding behavior and error-category guarantees.

## Context and Orientation

The repository is a single Rust crate with clear layers. `src/cli/` defines clap args and help text, `src/engine/` performs command logic, `src/waveform/mod.rs` is the parser adapter over `wellen`, and `src/output.rs` renders human/JSON outputs. Errors are normalized by `src/error.rs` and top-level parse behavior is centralized in `src/cli/mod.rs`.

`change` argument shape lives in `src/cli/change.rs`, dispatch in `src/engine/mod.rs`, and execution remains a stub in `src/engine/change.rs`. `at` is already implemented end-to-end and provides reusable patterns for time parsing/normalization and output shape. Expression modules in `src/expr/` are currently placeholders and must be made concrete enough to carry `event iff logical_expr` model data for future reuse.

Primary files for this plan are `src/cli/change.rs`, `src/cli/mod.rs`, `src/engine/change.rs`, `src/engine/mod.rs`, `src/waveform/mod.rs`, `src/output.rs`, `src/expr/lexer.rs`, `src/expr/parser.rs`, `src/expr/mod.rs`, `tests/change_cli.rs` (new), `tests/cli_contract.rs`, `tests/fixtures/hand/change_edge_cases.vcd`, `schema/wavepeek.json`, `docs/DESIGN.md`, `README.md`, and `CHANGELOG.md`.

## Normative `change` Contract (Authoritative in This Plan)

This section is the execution contract. Implementers must follow this section even if wording elsewhere differs.

Command surface is:

    wavepeek change --waves <file> [--from <time>] [--to <time>] [--scope <path>] --signals <names> [--when <event_expr>] [--max <n>] [--abs] [--json]

`--waves` and `--signals` are required. `--signals` is a comma-separated list and output order must match user order exactly (including duplicates). `--max` defaults to `50` and must be greater than `0`; `--max 0` is an `args` error.

If `--when` is omitted, it defaults to `*`.

Name resolution matches `at` and applies to both `--signals` and signal references inside `--when`:

- Without `--scope`, names are interpreted as canonical full paths.
- With `--scope <path>`, names are interpreted as short names relative to that scope.

Scoped mode does not accept canonical full-path tokens inside `--signals` or `--when`; those must fail as `error: signal:` lookup failures under scoped resolution, consistent with `at` behavior.

Time strings require explicit units (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`) and integer numeric parts. Bare numbers are rejected as `args` errors. Parsed times must convert exactly into dump precision; non-exact conversion is an `args` error with fragment `cannot be represented exactly in dump precision`. `--from` and `--to` define an inclusive candidate window. Missing `--from` means dump start, missing `--to` means dump end, both missing means full dump. If both are present and `from > to`, return `args` error.

Supported `event_expr` forms:

- Non-edge events:
  - `*` means any change of the resolved `--signals` set (default behavior).
  - `<name>` means any change of that signal.
- Edge events:
  - `posedge <name>`
  - `negedge <name>`
  - `edge <name>` (any posedge or negedge)
- Composition:
  - `event or event`
  - `event, event` (comma is an exact synonym for `or`)
- Optional clause:
  - `event iff logical_expr`

Binding and segmentation rules in this slice:

- `event_expr := event_term ((or | ,) event_term)*`
- `event_term := basic_event [iff logical_expr_raw]`
- `iff` attaches only to the immediately preceding `basic_event`.
- `or` and `,` split event terms. For deferred `iff` support, `logical_expr_raw` capture stops at the next separator token at parenthesis depth `0` (temporary parser rule for this slice).
- Example parse shape: `negedge clk iff rstn or bar` is two terms: `(negedge clk iff rstn)` OR `(bar)`.

For this delivery slice, full evaluation of `logical_expr` is deferred. The parser and internal model for `iff` must be implemented and reusable; runtime must fail fast with `error: args:` and message fragment `iff logical expressions are not implemented yet` when an `iff` clause is encountered. Parsing behavior is explicit in this slice: recognize `iff` and capture trailing clause text, but do not syntax-validate `logical_expr` yet. This behavior is temporary and explicitly tracked.

Event trigger semantics:

- Compute candidate timestamps in the inclusive `--from`/`--to` window.
- A timestamp is selected when any event term in the union fires.
- If multiple event terms fire at one timestamp, evaluate that timestamp once.
- Snapshot values are always sampled for `--signals`, not for all event-referenced names.
- `--from` is a baseline-initialization checkpoint: sample all `--signals` at `--from` (or dump start when `--from` is omitted), initialize delta tracking state, and never emit a row at that timestamp.
- Output emission eligibility is only for timestamps strictly greater than the baseline checkpoint.
- `change` emits a row only when at least one sampled `--signals` value changed relative to the last known sampled state strictly before that timestamp.
- If an event fires but sampled `--signals` values did not change at that timestamp, no row is emitted.
- Delta comparison is per sampled signal:
  - if a sampled signal has prior state, compare and include it in delta decision;
  - if a sampled signal lacks prior state at that timestamp, initialize its baseline there and exclude it from delta decision for that timestamp.
- Emit a row when at least one comparable sampled signal changed. If no sampled signal is comparable at that timestamp, emit no row.
- Edge classification at timestamp `t` must use the previous known value strictly before `t` (even when that value is before `--from`). If no previous value exists for the signal, that sample does not produce an edge event.

Edge classification follows SV2023 table semantics after per-sample normalization to four-state domain:

- Four-state domain values are `0`, `1`, `x`, `z`.
- Nine-state extension values `h`, `u`, `w`, `l`, `-` must be mapped to `x` before classification.
- For multi-bit signals, edge detection is based on the least significant bit only.
- `posedge`: `0 -> 1/x/z` and `x/z -> 1`.
- `negedge`: `1 -> 0/x/z` and `x/z -> 0`.
- `edge`: `posedge OR negedge`.

Values are formatted as Verilog literals `<width>'h<digits>` with lowercase hex digits and support for `x`/`z`.

JSON mode (`--json`) returns one envelope object with `command: "change"`, `data` as an array of snapshots, and `warnings` as an array of strings. Each snapshot object has:

- `time`: normalized plain time string (for example `5ns`, no `@`)
- `signals`: ordered array of objects `{ "path": <canonical path>, "value": <literal> }`

Human mode defaults to one line per snapshot:

    @<time> <display_1>=<value_1> <display_2>=<value_2> ...

`<display_i>` uses exact tokens passed in `--signals` by default. With `--abs`, `<display_i>` switches to canonical full paths. `--abs` does not alter JSON payload content.

Warnings must match byte-for-byte between JSON warning strings and human stderr warning text after `warning: ` prefixing.

If no rows are emitted in range (because there are no trigger timestamps or because all candidate timestamps preserve sampled `--signals` values), return success with empty `data` and one warning exactly `no signal changes found in selected time range`.

If snapshots exceed `--max`, truncate to first `--max` snapshots in time order and add warning exactly `truncated output to <n> entries (use --max to increase limit)`.

Runtime lookup failures for unknown scope/signal use existing categories (`error: scope:` or `error: signal:`) and non-zero exit. Parse/validation failures use `error: args:`. Success exits with code `0`; errors exit with code `1`.

## Open Questions

No blocking questions remain for this plan revision. The only planned deferral is full `iff logical_expr` evaluation; this is explicitly specified as temporary `args` failure behavior until the expression evaluator milestone lands.

## Plan of Work

Milestone 1 locks the updated contract with failing tests first (TDD). Add `tests/change_cli.rs` for `--when` default and explicit event-expression forms, and update `tests/cli_contract.rs` and CLI help strings to remove `--clk` and document `--when`. This milestone is complete when `cargo test --test change_cli` fails due to runtime stubs (not contract mismatch), while `cargo test --test cli_contract` is green with new help text.

Milestone 2 introduces reusable event-expression infrastructure. Add parser/types for event terms, union composition (`or` and comma), edge kinds, and `iff` attachment model. Resolve signal references under `scope` rules and add edge-classification helpers in waveform-side logic, including nine-state-to-`x` mapping and LSB extraction. This milestone is complete when parser/unit tests are green and waveform transition helpers are green.

Milestone 3 implements `change` runtime and output wiring using resolved event expressions. Remove `--clk` handling, apply default `--when "*"`, evaluate non-edge and edge triggers, deduplicate timestamps, suppress rows where sampled `--signals` did not change, and keep existing output/warning/parity guarantees.

Milestone 4 updates collateral: schema, design docs, README, changelog. The docs must explain the new unified trigger model, explicit temporary `iff` limitation, and `--from` baseline-only emission rule (including omitted-`--from` behavior at dump start).

Milestone 5 executes full quality gates and records deterministic evidence.

### Concrete Steps

Run all commands from `/workspaces/feat-cmd-change`.

1. Add/adjust contract tests first and update help text contracts.

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: `change_cli` fails because runtime implementation is still incomplete; `cli_contract` passes with `--when`-based help.

2. Implement event-expression parser/types and waveform edge helpers.

       cargo test expr::tests
       cargo test waveform::tests

   Expected now: parser and edge helper tests pass, including SV2023 edge table cases and nine-state mapping cases.

3. Implement `change` engine integration and renderer wiring.

       cargo test --test change_cli
       cargo test --test cli_contract

   Expected now: both suites pass with exact JSON/human output behavior and warning parity.

4. Sync schema/docs/changelog.

       make update-schema
       make check-schema

   Expected now: no schema drift and `change` contract reflects `--when`.

5. Run full quality gates.

       make test
       make check

   Expected now: tests, lint, schema checks, and build checks pass.

## Validation and Acceptance

Acceptance is behavioral and requires exact outputs or exact assertions for commands below.

Example A, default trigger equals wildcard non-edge (`--when` omitted):

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --json

Expected `data` exactly:

    [
      {
        "time": "5ns",
        "signals": [
          {"path": "top.clk", "value": "1'h1"},
          {"path": "top.data", "value": "8'h00"}
        ]
      },
      {
        "time": "10ns",
        "signals": [
          {"path": "top.clk", "value": "1'h1"},
          {"path": "top.data", "value": "8'h0f"}
        ]
      }
    ]

with `warnings` exactly `[]`.

Example B, explicit wildcard non-edge parity:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --json > /tmp/change_when_default.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --when "*" --json > /tmp/change_when_star.json
    cmp /tmp/change_when_default.json /tmp/change_when_star.json

Expected behavior: `cmp` exits `0`.

Example C, named non-edge trigger:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --signals data,clk --when "data" --json

Expected `data` exactly one row at `10ns`:

    [
      {
        "time": "10ns",
        "signals": [
          {"path": "top.data", "value": "8'h0f"},
          {"path": "top.clk", "value": "1'h1"}
        ]
      }
    ]

Example D, event fires but unchanged sampled signal must not emit a row:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --signals data --when "posedge clk" --json

Expected `data` exactly `[]` and `warnings` exactly `["no signal changes found in selected time range"]`.

Example D2, zero-delta path warning parity between JSON and human:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --signals data --when "posedge clk" --json > /tmp/change_zero_delta.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 0ns --to 10ns --scope top --signals data --when "posedge clk" > /tmp/change_zero_delta.out 2> /tmp/change_zero_delta.err
    python -c 'import json; d=json.load(open("/tmp/change_zero_delta.json")); assert d["data"] == []; assert d["warnings"] == ["no signal changes found in selected time range"]; assert open("/tmp/change_zero_delta.out").read() == ""; assert open("/tmp/change_zero_delta.err").read().strip() == "warning: no signal changes found in selected time range"'

Expected behavior: assertion passes and warning text matches byte-for-byte after removing `warning: ` prefix.

Example D3, missing-prior-baseline initialization is parser/waveform-locked:

    cargo test waveform::tests::delta_filter_initializes_without_prior_state

Expected assertion: when the first candidate timestamp has no comparable sampled baseline (all sampled signals uninitialized), engine initializes baseline and emits no row at that timestamp.

Example D4, mixed prior availability still emits on comparable deltas:

    cargo test waveform::tests::delta_filter_mixed_prior_state_emits_on_comparable_change

Expected assertion: if at least one sampled signal has prior baseline and changes at the candidate timestamp, a row is emitted even when another sampled signal initializes baseline at that same timestamp.

Example E0, omitted `--from` still applies dump-start baseline checkpoint:

Contract fixture requirement: add `tests/fixtures/hand/change_from_boundary.vcd` where sampled signal `top.sig` changes at dump start (`0ns`) and changes again at `5ns`.

    cargo run --quiet -- change --waves tests/fixtures/hand/change_from_boundary.vcd --to 5ns --scope top --signals sig --when "*" --json

Expected `data[*].time` exactly: `["5ns"]` and `warnings` exactly `[]` (no row at `0ns`).

Example E, baseline checkpoint suppresses emission at exact `--from` timestamp:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 5ns --to 10ns --scope top --signals clk --when "posedge clk" --json

Expected `data` exactly `[]` and `warnings` exactly `["no signal changes found in selected time range"]`.

Example E2, `--to == --from` never emits baseline timestamp row:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 5ns --to 5ns --scope top --signals clk --when "posedge clk" --json

Expected `data` exactly `[]` and `warnings` exactly `["no signal changes found in selected time range"]`.

Example F, event union with comma and `or` synonym parity:

    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --signals clk1 --when "posedge clk1, posedge clk2" --json > /tmp/change_when_union_comma.json
    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --signals clk1 --when "posedge clk1 or posedge clk2" --json > /tmp/change_when_union_or.json
    cmp /tmp/change_when_union_comma.json /tmp/change_when_union_or.json

Expected behavior: `cmp` exits `0`, proving comma and `or` are exact synonyms.

`tests/fixtures/hand/change_edge_cases.vcd` must include an overlap at `30ns` where both `posedge clk1` and `posedge clk2` occur (contract fixture requirement). For that timestamp, union-trigger output must still contain exactly one snapshot row.

    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --signals clk1 --when "posedge clk1, posedge clk2" --json > /tmp/change_when_union_overlap.json
    python -c 'import json; d=json.load(open("/tmp/change_when_union_overlap.json")); times=[r["time"] for r in d["data"]]; assert times.count("30ns") == 1, times'

Expected behavior: Python assertion passes, proving deduplication when multiple event terms fire at one timestamp.

Contract fixture requirement for CLI edge-form integration: `tests/fixtures/hand/change_edge_cases.vcd` must encode `top.clk` transitions so that event timestamps are deterministic and asserted as:

- `posedge top.clk` at `5ns`, `15ns`, `30ns`
- `negedge top.clk` at `10ns`, `20ns`, `25ns`
- `edge top.clk` at `5ns`, `10ns`, `15ns`, `20ns`, `25ns`, `30ns`

Example F2, CLI `negedge` wiring end-to-end:

    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --signals clk --when "negedge clk" --json

Expected `data[*].time` exactly: `["10ns", "20ns", "25ns"]` and `warnings` exactly `[]`.

Example F3, CLI `edge` wiring end-to-end:

    cargo run --quiet -- change --waves tests/fixtures/hand/change_edge_cases.vcd --from 0ns --to 30ns --scope top --signals clk --when "edge clk" --json

Expected `data[*].time` exactly: `["5ns", "10ns", "15ns", "20ns", "25ns", "30ns"]` and `warnings` exactly `[]`.

Example F4, `iff` + union segmentation is parser-locked:

    cargo test expr::tests::event_expr_iff_binding_with_union

Expected assertion: parsing `negedge clk iff rstn or bar` yields two terms exactly: `(negedge clk, iff_expr="rstn")` and `(any-change bar, iff_expr=None)`.

Example F5, deferred `iff` capture with parenthesized separators is parser-locked:

    cargo test expr::tests::event_expr_iff_capture_parenthesized_or

Expected assertion: parsing `posedge clk iff (a or b) or bar` yields two terms exactly: `(posedge clk, iff_expr="(a or b)")` and `(any-change bar, iff_expr=None)`.

Example G, SV2023 edge-semantics matrix is locked by unit tests:

    cargo test waveform::tests::edge_classification_sv2023_matrix

Expected assertions (exact):

- `posedge == true` for `0->1`, `0->x`, `0->z`, `x->1`, `z->1`.
- `negedge == true` for `1->0`, `1->x`, `1->z`, `x->0`, `z->0`.
- `edge == (posedge || negedge)` for all tested transitions.

Example H, nine-state (`h/u/w/l/-`) normalization to `x` is locked by unit tests:

    cargo test waveform::tests::edge_classification_ninestate_maps_to_x

Expected assertions (exact examples):

- `h->1` is treated as `x->1` and matches `posedge`.
- `1->l` is treated as `1->x` and matches `negedge`.
- `u->0` is treated as `x->0` and matches `negedge`.
- `0->w` is treated as `0->x` and matches `posedge`.
- `-->1` is treated as `x->1` and matches `posedge`.

Example I, edge detection on multi-bit signals uses only LSB:

    cargo test waveform::tests::edge_detection_uses_lsb_only

Expected assertions (exact): non-LSB bit flips alone do not trigger edge events; changing only LSB can trigger events according to SV edge rules.

Example J, `iff` staged behavior (temporary):

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals data --when "negedge clk iff rstn"

Expected behavior in this delivery slice: fail with stderr starting `error: args:` and include `iff logical expressions are not implemented yet`.

Example K, malformed `iff` logical text still returns deferred error (no syntax validation yet):

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals data --when "posedge clk iff ("

Expected behavior: fail with stderr starting `error: args:` and include `iff logical expressions are not implemented yet`.

Example L, empty-result warning parity between JSON and human:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data --json > /tmp/change_empty.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 6ns --to 9ns --signals top.clk,top.data > /tmp/change_empty.out 2> /tmp/change_empty.err
    python -c 'import json; d=json.load(open("/tmp/change_empty.json")); assert d["data"] == []; assert d["warnings"] == ["no signal changes found in selected time range"]; assert open("/tmp/change_empty.out").read() == ""; assert open("/tmp/change_empty.err").read().strip() == "warning: no signal changes found in selected time range"'

Expected behavior: assertion passes and warning text matches byte-for-byte after removing `warning: ` prefix.

Example M, truncation warning parity between JSON and human:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1 --json > /tmp/change_trunc.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.clk,top.data --max 1 > /tmp/change_trunc.out 2> /tmp/change_trunc.err
    python -c "import json; d=json.load(open('/tmp/change_trunc.json')); assert len(d['data']) == 1; assert d['warnings'] == ['truncated output to 1 entries (use --max to increase limit)']; assert open('/tmp/change_trunc.out').read().strip() == \"@5ns top.clk=1'h1 top.data=8'h00\"; assert open('/tmp/change_trunc.err').read().strip() == 'warning: truncated output to 1 entries (use --max to increase limit)'"

Expected behavior: assertion passes and warning text matches byte-for-byte after removing `warning: ` prefix.

Example N, `--abs` changes only human labels, not JSON payload:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --json > /tmp/change_abs_default.json
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --json --abs > /tmp/change_abs_flag.json
    cmp /tmp/change_abs_default.json /tmp/change_abs_flag.json

Expected behavior: `cmp` exits `0`.

Example O, human-mode `--abs` switches labels to canonical paths:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk > /tmp/change_abs_human_default.out
    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --scope top --signals data,clk --abs > /tmp/change_abs_human_abs.out
    python -c "d=open('/tmp/change_abs_human_default.out').read().splitlines(); a=open('/tmp/change_abs_human_abs.out').read().splitlines(); assert d[0] == \"@5ns data=8'h00 clk=1'h1\"; assert a[0] == \"@5ns top.data=8'h00 top.clk=1'h1\""

Expected behavior: assertion passes, proving default human mode preserves requested tokens while `--abs` switches to canonical paths.

Example P, duplicate `--signals` tokens are preserved in output order:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 1ns --to 10ns --signals top.data,top.clk,top.data --json > /tmp/change_dupe_signals.json
    python -c "import json; d=json.load(open('/tmp/change_dupe_signals.json')); row=d['data'][0]['signals']; assert [s['path'] for s in row] == ['top.data','top.clk','top.data'], row"

Expected behavior: assertion passes and duplicate ordering is unchanged.

Example Q, omitted `--max` defaults to `50` with truncation warning:

Contract fixture requirement: add `tests/fixtures/hand/change_many_events.vcd` containing at least 51 trigger timestamps for default `--when "*"` against selected signals.

    cargo run --quiet -- change --waves tests/fixtures/hand/change_many_events.vcd --signals top.sig --json > /tmp/change_default_max.json
    python -c "import json; d=json.load(open('/tmp/change_default_max.json')); assert len(d['data']) == 50; assert d['warnings'] == ['truncated output to 50 entries (use --max to increase limit)']"

Expected behavior: assertion passes, proving default `--max` behavior without explicitly passing `--max`.

Error acceptance checks:

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --signals top.clk --max 0

must fail with stderr starting `error: args: --max must be greater than 0.`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 10ns --to 1ns --signals top.clk

must fail with stderr starting `error: args:` and include `--from must be less than or equal to --to`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 10 --signals top.clk

must fail with stderr starting `error: args:` and include `requires units`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top --signals top.clk --when "posedge top.clk"

must fail with stderr starting `error: signal:` (scoped mode expects short names only for both `--signals` and `--when`).

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --when "posedge nope" --signals top.clk

must fail with stderr starting `error: signal:`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --scope top.nope --signals clk

must fail with stderr starting `error: scope:`.

    cargo run --quiet -- change --waves tests/fixtures/hand/m2_core.vcd --from 15ps --signals top.clk

must fail with stderr starting `error: args:` and include `cannot be represented exactly in dump precision`.

All success examples must exit `0`. All error examples must exit `1`.

## Idempotence and Recovery

All steps are safe to repeat. Tests, schema regeneration, and checks are idempotent from a clean checkout.

If schema checks fail after command implementation, run `make update-schema` and then `make check-schema`. If mismatch persists, compare runtime `wavepeek schema` output to `schema/wavepeek.json` to separate intentional contract changes from accidental serialization drift.

If implementation is partially complete and failing, recover in this order: keep compile green, keep `at` tests green, keep parser tests green, re-enable non-edge trigger paths, then edge trigger paths, then warning/error exactness, then docs/schema updates.

## Artifacts and Notes

Primary files to modify:

    src/cli/change.rs
    src/cli/mod.rs
    src/engine/change.rs
    src/engine/mod.rs
    src/waveform/mod.rs
    src/output.rs
    src/expr/mod.rs
    src/expr/lexer.rs
    src/expr/parser.rs
    tests/change_cli.rs
    tests/cli_contract.rs
    tests/fixtures/hand/change_edge_cases.vcd
    tests/fixtures/hand/change_from_boundary.vcd
    tests/fixtures/hand/change_many_events.vcd
    tests/fixtures/hand/change_scope_ambiguous.vcd
    schema/wavepeek.json
    docs/DESIGN.md
    README.md
    CHANGELOG.md

Boundary behavior that must be explicitly tested includes `--from == --to`, omitted bounds, default `--max = 50`, duplicate signal tokens preserving order, scoped short-name mode parity with `at`, deterministic deduplication when multiple event terms fire at one timestamp, suppression of event timestamps with zero sampled deltas, and nine-state-to-`x` normalization for edge classification.

## Interfaces and Dependencies

No new external dependencies are required.

In `src/cli/change.rs`, replace `--clk` argument with:

    #[arg(long)]
    pub when: Option<String>,

Keep:

    #[arg(long)]
    pub abs: bool,

In event-expression modules (`src/expr/*`), provide reusable types and parser support for:

    enum EventKind { AnyTracked, AnyChange(String), Posedge(String), Negedge(String), Edge(String) }
    struct EventTerm { event: EventKind, iff_expr: Option<String> }
    struct EventExpr { terms: Vec<EventTerm> }

`iff_expr` is stored as source text or parsed-expression handle (implementation choice), but it must remain reusable by future `when` work.

In `src/engine/change.rs`, keep payload types aligned with `at` identity model:

    pub struct ChangeSignalValue { display: String, path: String, value: String }
    pub struct ChangeSnapshot { time: String, signals: Vec<ChangeSignalValue> }

In `src/waveform/mod.rs`, expose helper responsibilities needed by `change`:

    resolve signal references (including names from event terms)
    enumerate timestamps for non-edge and edge event terms
    classify edges with SV2023 semantics (including nine-state mapping)
    sample resolved signal values at selected timestamps
    normalize timestamp strings for output

These responsibilities are part of the plan contract to reduce ambiguity; concrete function names may vary if behavior and tests remain exact.

Revision Note: 2026-02-23 / OpenCode - Rewrote the active `change` ExecPlan from `--clk` split semantics to unified `--when "<event_expr>"` semantics, aligned edge behavior with SV2023 table expectations (including nine-state mapping), and added explicit staged handling for `iff logical_expr` so reusable infrastructure lands now while full expression evaluation is deferred with a clear error contract.
Revision Note: 2026-02-24 / OpenCode - Incorporated reviewer findings by removing contract ambiguities (`*` trigger scope, edge boundary behavior, `iff` parse-vs-runtime rule) and replacing qualitative acceptance text with exact expected outputs/parity checks plus explicit unit-test matrices for SV edge semantics.
Revision Note: 2026-02-24 / OpenCode - Added missing executable acceptance coverage for warning parity, `--abs` JSON invariance, and same-timestamp deduplication when union event terms fire together.
Revision Note: 2026-02-24 / OpenCode - Hardened acceptance command reliability (shell-safe quoting), pinned overlap timestamp to `30ns` for deterministic dedup checks, and added explicit human-label verification for `--abs`.
Revision Note: 2026-02-24 / OpenCode - Added explicit `iff` binding/segmentation grammar for deferred logical clauses, plus CLI-level `negedge`/`edge` acceptance and executable checks for duplicate signal-order retention and implicit `--max=50` truncation behavior.
Revision Note: 2026-02-24 / OpenCode - Clarified that `change` is delta-oriented under `--when`: event timestamps are candidate points only, and rows are emitted strictly when sampled `--signals` values change; updated normative text and acceptance examples to enforce zero-delta suppression.
Revision Note: 2026-02-24 / OpenCode - Added a dedicated acceptance parity check for the zero-delta path to ensure JSON/human warning behavior is identical even when events fire but no sampled changes are emitted.
Revision Note: 2026-02-24 / OpenCode - Finalized two ambiguity fixes: explicit per-signal baseline-initialization and mixed-prior delta rules, plus deterministic deferred-`iff` capture with parenthesis-depth `0` separators and dedicated parser/engine acceptance hooks.
Revision Note: 2026-02-24 / OpenCode - Added explicit start-boundary behavior: `--from` is always baseline initialization and never produces a snapshot row; emission begins only after `--from`.
Revision Note: 2026-02-24 / OpenCode - Added missing acceptance coverage for omitted `--from` and `--to == --from` baseline behavior, and explicitly required matching DESIGN-doc boundary updates to avoid contract drift.
Revision Note: 2026-02-24 / OpenCode - Refined omitted-`--from` baseline proof to use non-edge `*` trigger so the check validates baseline suppression directly rather than relying on edge previous-state availability at dump start.
Revision Note: 2026-02-24 / OpenCode - Completed end-to-end implementation: added `change --when` runtime/parser/waveform infrastructure, contract fixtures/tests, output/schema/docs/changelog updates, and full quality-gate evidence (`make test`, `make check`).
Revision Note: 2026-02-24 / OpenCode - Closed both mandatory review passes by hardening scoped canonical-token rejection, making unknown `--when` names fail even on empty timestamp windows, and short-circuiting `--max` truncation processing.
