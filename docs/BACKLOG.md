# Backlog

## Open Design Questions

These unresolved design questions stay here so they remain visible without
polluting the stable design contracts.

Stable design contracts and the design corpus entrypoint live under
[`docs/design/`](design/), starting from [`docs/design/index.md`](design/index.md).

1. **Scope and path canonicalization.** What is the canonical path syntax and
   escaping policy for VCD escaped identifiers and other unusual names across
   formats?
2. **Warnings as codes versus free text.** Should warnings remain free-form
   strings, or should wavepeek eventually introduce stable warning codes for
   promote/suppress flows?
3. **Value radix options.** Should a future release add `--radix` (for example
   `hex`, `bin`, `dec`, `auto`), and if so what default policy should replace
   or complement Verilog-literal output?
4. **Schema evolution policy.** Should the project keep one canonical schema
   forever, or eventually split machine contracts into per-command schemas?
5. **Signal metadata schema.** Which JSON fields beyond `kind` and `width`
   should be part of the stable `signal` machine contract across dump formats?
6. **GHW support scope.** If GHW support is added after MVP, what acceptance
   criteria and priority should gate that work?

## Issues

### JSON schema data-field detail hardening

- Tighten schema coverage for `data` payload fields that currently validate only as generic strings.
- Add explicit enum definitions for enumeration-like fields such as `scope.data[].kind` and `signal.data[].kind`, sourced from the stable aliases emitted by the waveform adapter.
- Add concise `description` text for command payload fields so `wavepeek schema` is more self-documenting for machine clients and agent workflows.
- Include drift protection so Rust-side emitted aliases and schema enum values stay in sync.
- Close when `schema/wavepeek.json`, `wavepeek schema`, schema checks, and relevant contract tests cover the richer field metadata without changing existing JSON output bytes except for the schema document itself.

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

### Typed stdin projection from wavepeek JSON

- Consider allowing selected consumer arguments to use `-` as a typed stdin source from another `wavepeek --json` command instead of adding a separate chaining output mode.
- Example: `scope --json | signal --scope -` projects exactly one `scope.data[].path`; `signal --json | value --signals -` projects one or more `signal.data[].path` values.
- Keep compatibility explicit per argument/producer pair, preserve upstream warnings, reject ambiguous multi-stdin usage, and fail fast on wrong producer command or invalid cardinality.

## Tech Debt

No open command-integration debt is tracked here right now.
