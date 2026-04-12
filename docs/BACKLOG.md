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
- Close when `--json` remains backward-compatible, `--jsonl` is documented in CLI help plus `docs/design/reference/cli.md` and `docs/design/contracts/machine_output.md`, and integration tests cover ordering, truncation/warnings, and end-of-stream summary semantics.

## Tech Debt

No open command-integration debt is tracked here right now.
