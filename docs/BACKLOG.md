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

### Legacy `change --on` parser adapter remains separate from typed parser

- `src/engine/change.rs` still uses the compatibility adapter (`expr::parse_event_expr`) while strict typed parsing lives behind `wavepeek::expr::parse_event_expr_ast`.
- This keeps current command behavior stable but duplicates parser ownership and can drift if follow-up integration is delayed.
- Close when command runtime parsing converges on the typed parser boundary with explicit compatibility policy and regression coverage.

### Expression command integration and `property` runtime remain unimplemented

- Standalone typed event/logical runtime is now available in `src/expr/`, including
  rich types and full standalone `iff` semantics, but
  command wiring is still deferred: `src/engine/change.rs` keeps the legacy
  runtime path and `src/engine/property.rs` still returns `Unimplemented`.
- This leaves end-to-end `property` execution and shared command/runtime
  convergence for `change` as explicit remaining debt.
- Close when `property` runs end to end on the shared evaluator path and
  `change` converges to the same typed runtime with CLI/integration coverage.
