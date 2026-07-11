---
name: wavepeek
description: Use this skill when you need to inspect or analyze `.vcd`, `.fst`, or `.fsdb` waveforms with the wavepeek CLI. Load it for dump metadata, hierarchy/signal discovery, point samples, value transitions, event/property checks, transfer/handshake row extraction, and JSON-backed automation.
---

Use `wavepeek` for waveform questions. Treat waveform files as CLI inputs, not as text files to inspect directly.

This skill is a compact router, not the full command reference. It should choose the right analysis primitive, then route to installed help/docs for exact syntax and edge-case semantics. Do not infer unsupported syntax from memory or from a nearby example.

## Safety and operating posture

- Do not read `.fst` or `.fsdb` files with generic text/binary tools. Avoid raw `.vcd` reads too; large dumps will waste context and can hide timing semantics.
- Confirm or infer these before expensive queries: waveform path, user goal, relevant scope/signals if known, clock/reset if relevant, and target time window if any.
- Keep output bounded by default. Prefer filters, focused signal lists, and explicit windows. Use unbounded output only when the expected result size is small or the user explicitly asks for it.
- Prefer `--json` for scripts, aggregation, and agent-side post-processing. Always inspect diagnostics before trusting counts.

## Progressive disclosure

Use the installed binary as the source of truth for exact syntax, defaults, and build-specific features:

    wavepeek -h
    wavepeek --help
    wavepeek help <command-path...>
    wavepeek docs topics
    wavepeek docs search <query>
    wavepeek docs show <topic-id>

When asked about wavepeek itself, its command semantics, JSON schema, documentation, or supported waveform formats, answer from installed help/docs rather than memory. Follow relevant `see_also` topics when the question depends on cross-cutting behavior.

Use these routes first:

- command choice: `wavepeek docs show commands/overview`
- exact flags/defaults: `wavepeek help <command>`
- `extract` event-row extraction: `wavepeek docs show commands/extract`
- `property` semantics and capture modes: `wavepeek docs show commands/property`
- `change` semantics, `--on`, and `--max`: `wavepeek docs show commands/change`
- scope/name rules, time windows, ordering, bounds: `wavepeek docs show reference/command-model`
- JSON envelopes, diagnostics, fatal errors, schema: `wavepeek docs show reference/machine-output`
- trigger and expression syntax for `change --on`, `property --on`, `property --eval`, `extract generic --on`, and `extract generic --when`: `wavepeek docs show reference/expression-language`
- empty results, scoped-vs-canonical mistakes, time-token errors: `wavepeek docs search <symptom>`

Before using any command in a nontrivial way, read `wavepeek help <command>` or the corresponding `commands/<command>` topic. Do this instead of trying to guess spellings.

## Command routing by task

- Dump bounds, time unit, or sanity check: `info`.
- Hierarchy discovery: `scope`.
- Signal discovery inside a known scope: `signal`.
- State at explicit timestamp(s): `value`.
- Moments when displayed signal values changed: `change`.
- Timestamps where a Boolean condition is true or changes state: `property`.
- Event/transaction rows, handshakes, beats, and counts with payload values: `extract`.
- Fallback timestamp-only event enumeration: `property --capture match`, then `value --at <sample_time>` for payload sampling.
- Machine parsing or aggregation: supported `--json`, plus `wavepeek schema` if the exact shape matters.

Start most investigations with:

    wavepeek info --waves <FILE> --json

When names are unknown, use built-in discovery filters before shell filtering large outputs:

    wavepeek scope  --waves <FILE> --filter '.*<block-or-instance>.*' --max 50 --json
    wavepeek signal --waves <FILE> --scope <SCOPE> --filter '.*<signal-fragment>.*' --max 50 --json

For large designs, search scopes broadly and signals locally. Use `signal --recursive` only when the signal may be below the selected scope; cap it with `--max` and, when useful, `--max-depth`.

## Naming discipline

Choose one naming mode per command:

- Without `--scope`, use canonical full paths everywhere.
- With `--scope <SCOPE>`, use names relative to that scope in `--signals`, `--on`, `--eval`, `--when`, and `--payload`.

Do not mix `--scope top.cpu` with references like `top.cpu.clk` in the same `value`, `change`, `property`, or `extract` query. If name lookup fails, run `scope`, then `signal --scope <SCOPE>`, then rewrite the query in one naming mode.

## RTL event model

Separate the sampling event from the condition being tested.

For synchronous RTL, the default mental model should be:

    --on "posedge <clock>" --eval "<condition over signals>"

Edge-triggered `change` and `property` queries default to pre-edge value sampling, and `extract` always uses pre-edge sampling: the row `time` is the clock edge, while `sample_time` is where values were evaluated, printed, or extracted. Use `sample_time` for follow-up payload sampling unless you intentionally want same-edge dump values.

Do not look for `posedge` of payload or control signals when the real question is about cycle-level RTL behavior. A ready/valid transfer, FSM state observation, enable, or sampled payload is normally checked on the owning clock edge. Use `posedge <signal>` only when the task explicitly asks for that signal edge or for asynchronous transition inspection.

Avoid `--on "*" --sample-mode native` for synchronous protocol counts unless you intentionally want change-driven sampling. It selects value-change timestamps, not clock cycles, and can include mid-cycle or duplicate evidence that is not a transaction boundary. Omitted `--on` is invalid.

`--on "posedge clk iff cond"` means “select clock edges gated by `cond`”. It is not a substitute for printing every event unless the chosen command emits every selected event. With `change`, the row can still be suppressed if the printed values did not change.

## Event and transaction enumeration

When the user asks for every occurrence, count, timestamp list, handshake, request, response, beat, or transaction, do not use `change` on payload signals as the primary counter.

`extract axi` supports AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 profiles. Use it when the user wants ready/valid channel transfer rows:

    wavepeek extract axi \
      --waves <FILE> \
      --scope <SCOPE> \
      --profile axi4 \
      --map aclk=<CLK> \
      --map aresetn=<RESET_N> \
      --include '<AXI_SIGNAL_REGEX>' \
      --json

Use `extract generic` on a clocked predicate when payload values are needed for non-AXI or custom handshakes:

    wavepeek extract generic \
      --waves <FILE> \
      --scope <SCOPE> \
      --from <START> --to <END> \
      --on "posedge <CLK>" \
      --when "<EVENT_PREDICATE>" \
      --payload <PAYLOAD_AND_CONTEXT_SIGNALS> \
      --json

`extract` emits every matching row, including repeated transfers with identical payload values. The row `time` is the event edge and `sample_time` is where the predicate and payload were sampled. `extract axi` reports channel transfers only; it does not reconstruct bursts, ordering rules, or outstanding request state.

Use `property --capture match` when you only need timestamp rows or when you need property capture modes rather than payload extraction. Use `value --at <sample_time>` as a fallback follow-up when a payload set is decided after the property query.

## Ready/valid protocol recipe

For ready/valid-style protocols, define the transfer on the owning channel clock:

    --on "posedge <clk>" --when "<valid> && <ready>" --payload <PAYLOAD_SIGNALS>

This is the generic pattern for repeated transfers, including repeated transfers with identical payload values.

For protocols with separate channels, count each channel on its own handshake. Do not require unrelated channel handshakes to occur in the same cycle unless the protocol or the user’s question explicitly says so.

## Using `change` correctly

Use `change` when the question is about value transitions or when you need raw snapshots around a focused time range:

    wavepeek change \
      --waves <FILE> \
      --scope <SCOPE> \
      --from <START> --to <END> \
      --on "posedge <CLK>" \
      --signals <SIGNALS> \
      --max <N> \
      --json

`change` emits a row only when at least one requested signal value changed compared with the previous selected sample. The trigger firing is not enough. Therefore `change` can miss repeated events or repeated transactions whose printed payload/control values are unchanged.

Examples of what `change` does and does not answer:

- `change --signals addr` answers “when did `addr` visibly change?”, not “how many transfers used this address?”.
- `change --on "posedge clk iff valid && ready" --signals addr,data` answers “when did printed payload values change on matching cycles?”, not “list every matching cycle”.
- `change --signals valid,ready,payload --on "posedge clk"` can be a useful inspection shortcut, but validate event counts or transfer rows with `extract`.

If you use `change` for broad scans, set a deliberate `--max` or `--max unlimited` only after judging the output size. A truncated `change` result is not reliable for counts.

## JSON and diagnostics rules

For every `--json` command, parse the top-level envelope and check `.diagnostics` before trusting the result.

Diagnostics do not necessarily change the exit code. In human mode, diagnostics may be on stderr; avoid `head`, `grep`, or redirection patterns that hide them. In JSON mode, diagnostics are in the envelope.

## Time windows and bounds

Use `info` to get `time_start`, `time_end`, and the dump time unit before time-range queries. Use explicit units such as `10ns`; do not use bare numbers.

For count-like questions, make the covered interval explicit. If no user window is given, cover the full dump or state that a narrower window was used. Before finalizing, ensure the final query reaches the intended `time_end` or user-specified `--to`.

## Self-checks before the final answer

Before writing a report or artifact:

- Verify that no truncation diagnostic was ignored.
- Verify that the final time window covers the requested range.
- For event/transaction counts, derive counts from `extract` rows or `property --capture match`, not from payload transitions.
- For repeated payloads, explicitly ensure the method does not collapse identical consecutive events.
- For protocol “completion” claims, state which channel/event proves request, data, and response completion; do not conflate them.
- Include scope/clock provenance when it matters.
- If independent logs or monitors are available, use them as validation, not as a replacement for waveform analysis when the user asked to inspect the dump.

## Recovery patterns

If a command fails with a name error, switch to discovery: `scope`, then `signal --scope`, then rerun in one naming mode.

If a command fails with an expression or trigger error, read `wavepeek help property`, `wavepeek help change`, and `reference/expression-language`; simplify to `posedge clk` plus a Boolean predicate.

If results are empty, check `diagnostics`, widen the time window, remove filters, simplify the trigger/predicate, and consult `troubleshooting/empty-results` or `troubleshooting/scoped-vs-canonical-names`.

If results are too large, narrow the scope, filter signal names, reduce the time window, set an extract row limit with `--max`, or use `property` to compute timestamp-only predicates before sampling payloads.
