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

### Public workflow docs for events and protocol handshakes

Affecting flows:
- `llm-agent` — Must: agents need durable recipes for distinguishing value changes from protocol events and for extracting repeated handshakes without collapsing identical payloads.
- `user-manual` — Should: users benefit from cookbook-style waveform inspection workflows for common bus protocols without reverse-engineering command semantics from reference docs.
- `scripting` — Should: scripts can reuse documented event-enumeration patterns and avoid fragile ad hoc parsing or repeated one-off command loops.

- Expand `docs/public/workflows/` beyond first-change search with guidance for event and handshake extraction.
- Add a protocol-neutral workflow that explains how to identify a clock, define an event predicate, enumerate matches with `property --capture match`, sample payload signals with `value`, and use `change` only when value transitions are the desired result.
- Cover diagnostics and invariants in the workflow: check truncation diagnostics, verify row counts, distinguish unique payload values from transaction counts, and state whether completion was verified separately.
- Consider protocol-specific workflow topics when a protocol family has enough recurring inspection patterns to justify its own page; do not treat any initial set as exhaustive.
- Keep protocol topics parameterized around signal roles rather than design-specific names; possible examples include AXI AW handshakes, AXI-Stream `tvalid && tready`, APB transfer completion, AHB non-idle transfers with `hready`, Wishbone `cyc && stb && ack`, or other bus/stream protocols with similar event predicates.
- Link these workflows from command docs for `property`, `value`, and `change`, and from the packaged agent skill once the recipes are stable.
- Close when the public docs include at least one protocol-neutral event workflow and either protocol-specific recipes or a documented reason to keep protocol guidance generic.

### Batch, interactive, or MCP session mode for repeated queries

Affecting flows:
- `llm-agent` — Must: agents often issue many related waveform queries and should not pay full VCD/FSDB open/setup cost for every question.
- `user-manual` — Should: interactive users exploring a dump benefit from reusing loaded waveform state across repeated scope, signal, value, change, and property queries.
- `scripting` — Should: scripts that perform multi-step inspection need a supported way to amortize waveform setup without reimplementing orchestration around the CLI.

- Research and design a session-oriented execution mode that can reuse an opened waveform across many requests.
- Candidate shapes include a batch command, an interactive/REPL mode, a long-lived local service, an MCP server, or a combination of these; do not commit to one transport until requirements and trade-offs are documented.
- Optimize for formats with expensive per-process setup: large VCD files can pay high parse cost tied to dump size, while FSDB can pay high native reader and hierarchy setup cost tied to hierarchy and backend behavior.
- Treat FST as outside the initial motivation because its current indexed backend already has low per-command setup cost for repeated inspection compared with VCD and FSDB.
- Preserve existing one-shot CLI commands and output contracts; session mode should be additive and should define how command results, diagnostics, cancellation, and errors are represented.
- Define resource and lifecycle behavior explicitly, including memory retention, waveform close/reopen, stale file detection, concurrency limits, and cleanup on client disconnect.
- Close when a design note selects the initial interface, defines request/response contracts, documents format-specific reuse expectations, and includes benchmark acceptance criteria for repeated VCD/FSDB queries.

### FSDB hierarchy and idcode cache for one-shot CLI use

Affecting flows:
- `llm-agent` — Should: agents may continue to issue independent CLI calls even if a future session mode exists, and should not repeatedly rebuild FSDB hierarchy state.
- `user-manual` — Should: users exploring FSDB dumps through shell history need faster repeated `scope`, `signal`, and `value` calls without changing their workflow.
- `scripting` — Should: scripts built around one-shot commands need a way to amortize FSDB hierarchy setup without adopting a persistent transport.

- Research and design an on-disk cache for FSDB metadata and hierarchy-derived indexes across independent CLI invocations.
- Cache metadata needed for lookup and output, not waveform values: timescale, time bounds, scopes, signal records, full path to idcode mapping, widths, value encodings, datatypes, and enum metadata.
- Use the cache to avoid repeated full FSDB scope/var tree traversal on warm `scope`, `signal`, and path-based `value` queries where possible.
- Define strict invalidation using file identity, file size, mtime with sufficient precision, wavepeek version, cache schema version, and FSDB backend/cache version; consider an optional content fingerprint where needed.
- Define operational controls such as `--no-cache`, explicit rebuild, cache clean, atomic writes, and user-private permissions because hierarchy and signal names may be sensitive.
- Close when a design and prototype demonstrate warm-cache speedups for representative FSDB one-shot queries while preserving stale-cache detection and existing output contracts.

### VCD to FST sidecar cache for repeated inspection

Affecting flows:
- `llm-agent` — Should: agents querying large VCD dumps repeatedly need a path that avoids reparsing large textual waveform bodies on every command.
- `user-manual` — Could: users can already convert VCD to FST explicitly, but an integrated cache would make the faster path easier to discover and reuse.
- `scripting` — Should: scripts should be able to opt into or rely on a managed sidecar rather than carrying ad hoc conversion logic.

- Research an automatic or opt-in VCD-to-FST sidecar cache for repeated inspection of large VCD inputs.
- Prefer reusing FST as the parsed/indexed representation rather than inventing a separate VCD value-change cache.
- Make first-use conversion cost explicit; this is mainly valuable when the converted sidecar is reused across multiple queries or sessions, not necessarily for one-off commands.
- Define cache placement, invalidation, disk usage limits, cleanup behavior, concurrency handling, and disable/rebuild controls.
- Preserve direct VCD support and existing command outputs; sidecar use should be an implementation detail unless the user requests cache diagnostics.
- Close when a design selects opt-in versus automatic behavior, documents conversion trade-offs, and demonstrates repeated-query speedups on large VCD fixtures.

### Opt-in backend performance diagnostics

Affecting flows:
- `llm-agent` — Could: agents can use structured timing clues to choose formats, recommend conversion, or report why a query is slow.
- `user-manual` — Could: users can attach actionable timing breakdowns to performance reports instead of only reporting total command duration.
- `scripting` — Could: scripts and CI jobs can log backend phase timings when diagnosing regressions, while keeping normal output stable.

- Add an opt-in timing diagnostics mode for waveform backend phases without changing default stdout/stderr behavior.
- Capture durable phase boundaries such as format detection, open, header/body parse, hierarchy load, signal load, expression evaluation, value sampling, rendering, and backend cleanup where applicable.
- Decide whether diagnostics are human-readable stderr, machine-readable JSON/JSONL, or both; keep normal `--json` output contracts backward-compatible.
- Include enough format context to distinguish VCD parse cost, FST selective loading, and FSDB hierarchy/session setup without exposing sensitive signal values.
- Keep diagnostics low overhead when disabled and avoid relying on ad hoc temporary instrumentation.
- Close when the interface is documented, representative backend phases are covered, and tests verify that diagnostics are opt-in and do not perturb normal command output.

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
- Example: `property --capture match --json | value --at - --signals addr,data` projects one or more `property.data[].time` values into multi-timestamp value sampling.
- If raw line-oriented stdin is also supported for times, keep it separate from typed JSON projection, for example `property --capture match --json | jq -r '.data[].time' | value --at-file - --signals addr,data`.
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
