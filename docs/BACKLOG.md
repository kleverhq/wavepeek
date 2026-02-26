# Backlog

## Issues

### `$schema` raw URL compatibility

- JSON `$schema` URL currently points to GitHub HTML (`https://github.com/.../blob/...`) instead of raw content.
- Update it to `https://raw.githubusercontent.com/...` so the schema is directly downloadable and auto-detected by tooling.
- Current pattern: `https://github.com/kleverhq/wavepeek/blob/vX.Y.Z/schema/wavepeek.json`
- Target pattern: `https://raw.githubusercontent.com/kleverhq/wavepeek/vX.Y.Z/schema/wavepeek.json`

### CLI help should be self-descriptive

- Command help quality is inconsistent; users still need `docs/DESIGN.md` for critical behavior details.
- Improve `--help`/`--long-help` so each command is usable without external docs: include semantics, defaults, boundary rules, error categories, and output shape notes.
- Reuse and adapt normative wording from `docs/DESIGN.md` to keep CLI help and design contract aligned.
- Add this as a foundational CLI design principle in project docs: CLI help must provide sufficient standalone guidance; external docs are for depth, not required basics.
- Close when all shipped commands pass a help-quality contract check and the principle is documented in the CLI design principles section.

### Rename `at` command to `value` and `--time` to `--at`

- Current naming emphasizes the coordinate (`at`) more than the user intent (read signal values at a point); `value` is clearer and aligns command name with output semantics.
- Rename CLI surface from `wavepeek at` to `wavepeek value`, and rename `--time <time>` to `--at <time>`.
- Keep output/JSON behavior equivalent to current `at` semantics (point-in-time sampling, deterministic ordering, same literal formatting), except for command/flag naming.
- Define migration behavior explicitly (compat alias vs hard break) and document it in help/release notes.
- Close when CLI/help/docs/schema/tests/changelog are updated and migration behavior is covered by integration tests.

### Replace `when` with `property` and SVA-like event/eval wording

- `when` is generic; `property` communicates assertion-like intent and reads closer to natural language for waveform checks.
- Target command shape: `wavepeek property --when "<event_expr>" --eval "<logical_expr>"`.
- `--when` stays event-driven and defaults to `*` (any change among signals referenced by `--eval`) to avoid per-time-unit output spam.
- Scope/name resolution and time-window behavior should stay deterministic and reuse current event/expression infrastructure where possible.
- Close when `property` runs end-to-end with stable human/JSON contracts and `when` docs/runtime status are fully migrated.

### Add `property --capture` modes for match/transition reporting

- Add `--capture=match|switch|assert|deassert` to control report granularity.
- Semantics: `match` emits each event where `--eval` is true; `switch` emits state transitions (`assert` on `0->1`, `deassert` on `1->0`); `assert` emits only `0->1`; `deassert` emits only `1->0`.
- Default `--capture=switch` as the best signal/noise tradeoff for CI and terminal usage.
- Human output target is compact and action-oriented: `@123ns assert`, `@1234ns deassert`, or `@1223ps match`.
- Close when all capture modes have deterministic contracts, CLI tests, and JSON representation parity with human semantics.

### Add recursive signal listing (`signal --recursive`, `--max-depth`)

- Extend `wavepeek signal` with `--recursive` to traverse nested scopes under the selected `--scope`.
- Add `--max-depth <n>` to bound recursion depth in recursive mode, with deterministic traversal and ordering.
- Keep default behavior unchanged: without `--recursive`, `signal` remains non-recursive.
- In human mode, show paths relative to `--scope` when recursive; keep `--abs` for canonical absolute paths.
- Keep JSON output contract stable (`path` remains canonical absolute path).
- Preserve bounded output behavior with `--max` truncation warnings and integration coverage for recursive/non-recursive parity.

### Add explicit `unlimited` values for limit flags (`--max`, `--max-depth`)

- Introduce `unlimited` as a literal limit value to request unbounded output without relying on magic large numbers.
- Scope includes `--max` across applicable commands (`scope`, `signal`, `change`, `when`) and `--max-depth` for recursive traversal commands.
- Keep `--max 0` invalid (`error: args`) to avoid ambiguous semantics.
- Preserve existing `--max-depth 0` behavior (depth limited to current/root level only).
- Allow independent combinations such as `--max unlimited --max-depth 3` and `--max 100 --max-depth unlimited`.
- Require explicit warning parity in human stderr and JSON `warnings` whenever any limit flag is set to `unlimited`.
- Close when behavior, CLI help/docs/contracts, and integration tests are aligned across affected commands.

### Post-MVP: temporal property language extensions

- Track follow-up evolution toward richer assertion/cover-like checks (temporal operators, implication, multi-event relations).
- Keep MVP scope explicit: event trigger + boolean eval + capture modes only.
- Close when a separate design contract defines syntax/semantics and phased rollout milestones.

## Tech Debt

### `change --when`: deferred `iff logical_expr` execution

- Event terms with `iff` are parsed but intentionally rejected at runtime with `error: args: iff logical expressions are not implemented yet`.
- This was introduced in `feat(change): implement --when triggers end to end` as a staged delivery compromise.
- Close when `change --when "... iff ..."` is evaluated end-to-end (true/false branches), and the hard-fail path is removed.

### Event-expression parser uses temporary `iff` capture rules

- `iff` clause capture currently relies on raw text segmentation with union separators (`or`/`,`) at parenthesis depth `0`.
- The current splitter tolerates unmatched closing parentheses via `saturating_sub`, which is acceptable for staged parsing but weak for strict validation.
- Close when parser explicitly rejects currently-ambiguous malformed cases (at minimum: unmatched `(` / `)`, empty `iff` clause, and broken nested `or`/`,` segmentation) with deterministic `error: args:` and targeted tests.

### Expression evaluator and `property` runtime path remain unimplemented

- Reusable expression/event types were expanded for `change`, but `src/expr/eval.rs` and `src/engine/when.rs` still return `Unimplemented`.
- This keeps logical-expression semantics fragmented and blocks end-to-end delivery of the planned `property` command semantics.
- Close when the property runtime path (implemented in the canonical engine module, with `when` migration handled) runs end-to-end on the shared evaluator path with CLI/integration tests.

### Duplicated event-expression tests (`expr/mod.rs` and `expr/parser.rs`)

- Equivalent `iff`-binding tests currently exist in two modules (`src/expr/mod.rs` and `src/expr/parser.rs`).
- This duplication appeared during the `change --when` rollout and increases maintenance drift risk.
- Close when one source of truth remains for these parser tests (remove duplicates), and coverage is confirmed by `cargo test expr::parser` plus the standard CI gate (`make ci`).

### `expr/lexer.rs` scaffolding is currently unused

- `src/expr/lexer.rs` exports tokenization types/helpers, but current parser/runtime paths do not consume them.
- This leaves an unowned partial implementation in the expression layer and increases drift risk while `property`/evaluator work is still deferred.
- Close when either (a) parser/evaluator use lexer as the single tokenization path with focused tests, or (b) lexer scaffolding is removed and expression parsing remains covered by existing tests.

### Duplicated time parsing/alignment logic in `at` and `change`

- Time token parsing, dump-precision alignment checks, and related `error: args:` behavior are implemented in parallel in `src/engine/at.rs` and `src/engine/change.rs`.
- Duplicate validation logic increases contract drift risk for bounds handling and precision errors.
- Close when both commands use a shared time-validation utility (or one canonical path) with regression tests covering parity of accepted/rejected tokens and error fragments.

### `at` performance anomaly on high-activity signal sets

- CLI E2E benchmarks show an anomaly where `at_picorv32_signals_1000` is much slower than other `at ... signals_1000` cases, even on smaller FST input.
- Investigation indicates the slowdown is in `at` sampling path (`src/engine/at.rs` -> `src/waveform/mod.rs`), where one-point queries still trigger full signal-history loading/decoding for selected signals; runtime is dominated by signal activity profile, not just file size.
- Signal-name duplication used to reach large `--signals` counts is currently allowed by contract and is not the primary contributor to this slowdown.
- Close when a dedicated `at` performance plan is implemented, semantics remain unchanged, and bench evidence shows expected speedup on this case.
