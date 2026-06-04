# Backlog

## Open Design Questions

These unresolved design questions stay here so they remain visible without
polluting the stable design contracts.

Stable user-facing contracts live under `../public/reference/`, starting from
`../public/intro.md` for the public documentation map.

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

### Temporal property language extensions over waveforms

Affecting flows:
- `llm-agent` — Could: richer temporal operators would let agents ask higher-level verification questions, but current inspection flows remain usable without them.
- `user-manual` — Could: this is a real feature expansion for verification users, not a baseline usability gap in the shipped command set.
- `scripting` — Could: it can collapse multi-step post-processing into one query, but existing scripts can still compose current commands externally.

- Explore an "SVA over waves" direction: evaluate temporal/property-style checks directly on recorded waveform data instead of requiring a live simulator assertion flow.
- Use the existing SV-like expression surface as the starting point, then extend it with temporal operators and assertion-style composition where that produces a coherent user model.
- Reference material for the idea and possible UX shape:
  - DVCon paper: `https://dvcon-proceedings.org/wp-content/uploads/72781.pdf`
  - Blog summary: `https://ahmedalsawi.github.io/posts/2022/12/sawd-the-fun-version/`
- Keep the current shipped `property` scope explicit while this stays exploratory: event trigger + boolean eval + capture modes only.
- Close when a separate design contract defines syntax/semantics, compatibility boundaries with the existing expression language, and a phased rollout plan.

### Streaming JSON output mode for large result sets

Affecting flows:
- `llm-agent` — Could: mainly helps partial-result recovery on timeout or cancellation, but many agent harnesses still deliver command output only after completion.
- `user-manual` — Could: interactive humans usually prefer bounded human output, so NDJSON is not the primary operator path.
- `scripting` — Must: large pipelines benefit from incremental consumption, simpler failure recovery, and not buffering one giant JSON document in memory.

- Large waveform queries (especially recursive signal collection on big `.fst`) are expensive to consume as one buffered JSON envelope.
- Add an opt-in streaming mode via `--jsonl` (NDJSON) for high-volume/long-running commands, while keeping current `--json` contract unchanged.
- Define a dedicated stream schema (for example, `schema/wavepeek-stream-v1.json`) with deterministic record ordering and explicit terminal summary.
- Suggested stream record kinds: `begin`, `item` (command-specific payload), `warning`, `end` (with counters and truncation flags).
- Close when `--json` remains backward-compatible, `--jsonl` is documented in CLI help plus `docs/public/commands/overview.md` and `docs/public/reference/machine-output.md`, and integration tests cover ordering, truncation/warnings, and end-of-stream summary semantics.

### Typed stdin projection from wavepeek JSON

Affecting flows:
- `llm-agent` — Could: agents can already parse prior JSON results and pass explicit arguments, so this is mostly ergonomic sugar for them.
- `user-manual` — Should: common ad hoc pipelines become shorter and less dependent on `jq`, `python -c`, or fragile text scraping.
- `scripting` — Should: typed producer→consumer chaining reduces glue code and avoids hand-written JSON field extraction for common command pairs.

- Consider allowing selected consumer arguments to use `-` as a typed stdin source from another `wavepeek --json` command instead of adding a separate chaining output mode.
- Example: `scope --json | signal --scope -` projects exactly one `scope.data[].path`; `signal --json | value --signals -` projects one or more `signal.data[].path` values.
- Keep compatibility explicit per argument/producer pair, preserve upstream warnings, reject ambiguous multi-stdin usage, and fail fast on wrong producer command or invalid cardinality.

## Tech Debt

### FSDB real/string value sampling

Affecting flows:
- `llm-agent` — Should: agents may need to inspect analog-like or message-carrying signals from FSDB dumps, but today must route around them.
- `user-manual` — Should: users can discover real/string signals in FSDB hierarchy, then hit unsupported-value errors when asking value-oriented questions.
- `scripting` — Could: scripts can often reformulate checks through related bit-vector signals, but direct typed sampling would avoid format-specific gaps.

- FSDB hierarchy recovery can identify `real`, `short_real`, and `string`-like signals, but the native FSDB sample ABI currently returns only bit-vector payloads.
- `value`/`change` therefore reject these signals as unsupported non-bit-vector encodings, and FSDB expression sampling rejects `ExprTypeKind::Real` and `ExprTypeKind::String` even though the expression engine has typed real/string payloads for other backends.
- Close when the FSDB native shim exposes typed real/string samples, Rust backend plumbing returns `SampledValue::Real`/`SampledValue::String`, public output/schema behavior is intentionally defined, and FSDB fixtures cover value, change/property, and unsupported-edge cases.
