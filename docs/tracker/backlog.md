# Backlog

## Proposals

### Public performance guide for waveform formats

Affecting flows:
- `llm-agent` — Should: agents need format-aware expectations for command latency, conversion trade-offs, and memory pressure before choosing a dump strategy.
- `user-manual` — Should: users need practical guidance for choosing between FSDB, VCD, and FST without reverse-engineering benchmark artifacts.
- `scripting` — Should: scripts and CI jobs need predictable guidance on when repeated CLI invocations are expensive and when conversion or batching is appropriate.

- Add a public performance guide under `docs/public/` that explains format-level behavior and expected resource classes for supported waveform inputs.
- Cover VCD as large textual input that can require full body parsing for value-oriented workflows, with high CPU and memory pressure on large dumps.
- Cover FST as the preferred indexed/compact format for repeated inspection, with selective signal loading and generally lower memory use.
- Cover FSDB as an optional native backend whose value lookup can be fast after setup, but whose short-lived CLI use may pay repeated hierarchy/session setup costs.
- Explain why repeated independent CLI commands can be much slower than a future batched/persistent workflow because per-process waveform setup is not reused.
- Describe conversion trade-offs, especially FSDB/VCD to FST for repeated analysis, without promising universal speedups for one-off queries.
- Include order-of-magnitude memory expectations and caveats rather than benchmark-specific tables; keep detailed benchmark artifacts outside the public guide as source material for drafting.
- Close when the guide is linked from public docs navigation/help, documents VCD/FST/FSDB behavior and conversion guidance, and includes validation notes for memory and timing claims.

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
- Suggested stream record kinds: `begin`, `item` (command-specific payload), `diagnostic`, `end` (with counters and truncation flags).
- Close when `--json` remains backward-compatible, `--jsonl` is documented in CLI help plus `docs/public/commands/overview.md` and `docs/public/reference/machine-output.md`, and integration tests cover ordering, truncation diagnostics, and end-of-stream summary semantics.

### Typed stdin projection from wavepeek JSON

Affecting flows:
- `llm-agent` — Could: agents can already parse prior JSON results and pass explicit arguments, so this is mostly ergonomic sugar for them.
- `user-manual` — Should: common ad hoc pipelines become shorter and less dependent on `jq`, `python -c`, or fragile text scraping.
- `scripting` — Should: typed producer→consumer chaining reduces glue code and avoids hand-written JSON field extraction for common command pairs.

- Consider allowing selected consumer arguments to use `-` as a typed stdin source from another `wavepeek --json` command instead of adding a separate chaining output mode.
- Example: `scope --json | signal --scope -` projects exactly one `scope.data[].path`; `signal --json | value --signals -` projects one or more `signal.data[].path` values.
- Keep compatibility explicit per argument/producer pair, preserve upstream diagnostics, reject ambiguous multi-stdin usage, and fail fast on wrong producer command or invalid cardinality.

### GHW waveform input support

Affecting flows:
- `llm-agent` — Could: agents could inspect VHDL-oriented waveform dumps without requiring an external conversion step, but VCD/FST/FSDB already cover the current stable flows.
- `user-manual` — Could: users with GHDL or VHDL-heavy workflows could point wavepeek directly at `.ghw` files instead of exporting another format.
- `scripting` — Could: scripts could avoid format-conversion glue when upstream tools emit GHW natively.

- Explore adding `.ghw` input support with the same command surface and JSON contracts used for VCD, FST, and FSDB.
- Define stable hierarchy, signal-kind, value-sampling, time-unit, and unsupported-encoding behavior before implementation.
- Keep this proposal unscheduled until a concrete backend choice, fixture source, and parity test strategy are documented.
- Close when a design note identifies the reader implementation, fixture policy, stable kind/value mappings, and minimum acceptance tests for `info`, `scope`, `signal`, `value`, `change`, and `property`.

## Tech Debt

### FSDB real/string value sampling

Affecting flows:
- `llm-agent` — Should: agents may need to inspect analog-like or message-carrying signals from FSDB dumps, but today must route around them.
- `user-manual` — Should: users can discover real/string signals in FSDB hierarchy, then hit unsupported-value errors when asking value-oriented questions.
- `scripting` — Could: scripts can often reformulate checks through related bit-vector signals, but direct typed sampling would avoid format-specific gaps.

- FSDB hierarchy recovery can identify `real`, `short_real`, and `string`-like signals, but the native FSDB sample ABI currently returns only bit-vector payloads.
- `value`/`change` therefore reject these signals as unsupported non-bit-vector encodings, and FSDB expression sampling rejects `ExprTypeKind::Real` and `ExprTypeKind::String` even though the expression engine has typed real/string payloads for other backends.
- Close when the FSDB native shim exposes typed real/string samples, Rust backend plumbing returns `SampledValue::Real`/`SampledValue::String`, public output/schema behavior is intentionally defined, and FSDB fixtures cover value, change/property, and unsupported-edge cases.
