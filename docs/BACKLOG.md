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

## Tech Debt

### `change --when`: deferred `iff logical_expr` execution

- Event terms with `iff` are parsed but intentionally rejected at runtime with `error: args: iff logical expressions are not implemented yet`.
- This was introduced in `feat(change): implement --when triggers end to end` as a staged delivery compromise.
- Close when `change --when "... iff ..."` is evaluated end-to-end (true/false branches), and the hard-fail path is removed.

### Event-expression parser uses temporary `iff` capture rules

- `iff` clause capture currently relies on raw text segmentation with union separators (`or`/`,`) at parenthesis depth `0`.
- The current splitter tolerates unmatched closing parentheses via `saturating_sub`, which is acceptable for staged parsing but weak for strict validation.
- Close when parser explicitly rejects currently-ambiguous malformed cases (at minimum: unmatched `(` / `)`, empty `iff` clause, and broken nested `or`/`,` segmentation) with deterministic `error: args:` and targeted tests.

### Expression evaluator and `when` runtime remain unimplemented

- Reusable expression/event types were expanded for `change`, but `src/expr/eval.rs` and `src/engine/when.rs` still return `Unimplemented`.
- This keeps logical-expression semantics fragmented and delays reuse of the new event infrastructure in the standalone `when` flow.
- Close when `wavepeek when` runs end-to-end on the shared evaluator path with CLI/integration tests.

### Duplicated event-expression tests (`expr/mod.rs` and `expr/parser.rs`)

- Equivalent `iff`-binding tests currently exist in two modules (`src/expr/mod.rs` and `src/expr/parser.rs`).
- This duplication appeared during the `change --when` rollout and increases maintenance drift risk.
- Close when one source of truth remains for these parser tests (remove duplicates), and coverage is confirmed by `cargo test expr::parser` plus the standard CI gate (`make ci`).
