# Backlog

## Issues

### Post-MVP: temporal property language extensions

- Track follow-up evolution toward richer assertion/cover-like checks (temporal operators, implication, multi-event relations).
- Keep MVP scope explicit: event trigger + boolean eval + capture modes only.
- Close when a separate design contract defines syntax/semantics and phased rollout milestones.

### Streaming JSON output mode for large result sets

- Large waveform queries (especially recursive signal collection on big `.fst`) are expensive to consume as one buffered JSON envelope.
- Add an opt-in streaming mode via `--jsonl` (NDJSON) for high-volume/long-running commands, while keeping current `--json` contract unchanged.
- Define a dedicated stream schema (for example, `schema/wavepeek-stream-v1.json`) with deterministic record ordering and explicit terminal summary.
- Suggested stream record kinds: `begin`, `item` (command-specific payload), `warning`, `end` (with counters and truncation flags).
- Close when `--json` remains backward-compatible, `--jsonl` is documented in CLI help + `docs/DESIGN.md`, and integration tests cover ordering, truncation/warnings, and end-of-stream summary semantics.

## Tech Debt

### `change --on`: deferred `iff logical_expr` execution

- Event terms with `iff` are parsed but intentionally rejected at runtime with `error: args: iff logical expressions are not implemented yet`.
- This was introduced in the staged trigger rollout as a delivery compromise.
- Close when `change --on "... iff ..."` is evaluated end-to-end (true/false branches), and the hard-fail path is removed.

### Expression evaluator and `property` runtime path remain unimplemented

- Reusable expression/event types were expanded for `change`, but `src/expr/eval.rs` and the command-runtime path (`src/engine/property.rs`) still return `Unimplemented`.
- This keeps logical-expression semantics fragmented and blocks end-to-end delivery of the planned `property` command semantics.
- Close when the property runtime path (implemented in the canonical engine module) runs end-to-end on the shared evaluator path with CLI/integration tests.
