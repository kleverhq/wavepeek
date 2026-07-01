# Proposal: Group `extract generic` Event Evaluation

## Context

The SCR1 AXI extraction baseline in `docs/tracker/wip/extract_generic_perf_baseline.md` shows that `extract generic` spends most runtime in the emit loop. The dominant measured component is event matching:

- 2-channel run: `2,497,724` event checks, `16.039s` in event matching, `23.078s` non-DEBUG wall time to `/dev/null`.
- 5-channel run: `6,244,310` event checks, `42.193s` in event matching, `56.607s` non-DEBUG wall time to `/dev/null`.

Both manifests use the same event expression for every source:

```text
posedge clk iff axi_rst_n
```

The 5-channel run performs `2.5x` more event checks than the 2-channel run, matching the source-count ratio. Candidate timestamp count stays constant at `1,248,862`.

## Problem

`extract generic` currently evaluates the event expression per source at every candidate timestamp:

```text
for timestamp in candidate_times:
  for source in sources:
    if event_expr_matches(source.on, timestamp):
      if source.when(sample_time):
        emit row
```

This makes event matching scale with `candidate_timestamps * source_count`, even when multiple sources share the same resolved event expression.

The problem is not specific to AXI. Any source file with repeated event expressions pays the same repeated event-matching cost. A file with two sources on one clock and three sources on another clock still evaluates each source independently instead of evaluating each distinct event expression once per relevant timestamp.

The current model also uses one global candidate timestamp set for all event handles. If sources use different clocks, sources can be checked at timestamps that are only relevant to other event expressions.

## Solution idea

Build an event-evaluation layer between bound sources and row emission.

Instead of treating every source as an independent event matcher, group sources by the resolved event expression used by `--on` or source JSON `on`:

```text
event_group:
  event expression
  candidate handles for that event expression
  sources using that event expression
```

Then evaluate each group once at candidate timestamps relevant to that group:

```text
for timestamp in merged_group_candidate_times:
  evaluate each active event group once
  for matching groups:
    evaluate each source predicate in declaration order
    emit matching rows
```

This preserves per-source `when` evaluation and payload sampling while avoiding duplicate event-expression evaluation.

## Group identity

A group key should identify the resolved event expression, not a protocol name or a specific AXI pattern. The key must include:

- event term kind: `posedge`, `negedge`, or `edge`;
- resolved event signal handle/path;
- term-level `iff` expression, including its resolved signal references;
- term order and union structure for comma/`or` event expressions.

If a safe semantic key is not available for a source, the source can be placed into a singleton group. That fallback preserves current behavior.

A conservative first key may group only expressions that are textually identical after binding context is known. That is safe but misses some equivalent expressions with different spelling. A semantic key groups more cases but requires careful structural representation of the bound expression.

## Multiple event expressions

The grouping model handles mixed source files naturally:

```text
2 sources: posedge clk_a iff rst_n
3 sources: posedge clk_b iff rst_n
```

This produces two event groups. Each group uses candidate timestamps from its own event handles. At timestamps where both groups are active, both groups are evaluated, and matching rows are ordered by source declaration index.

## Multi-term event expressions

Event expressions can contain a union of terms:

```text
posedge clk_a iff en_a or negedge clk_b iff en_b
```

For the grouping fix, the whole event expression should remain one group. Its candidate set is the union of candidate handles referenced by its terms. The existing `event_expr_matches` logic still determines whether the group matches at a timestamp.

This avoids changing expression semantics while still sharing the result across all sources that use the same multi-term event expression.

Partial grouping of shared subterms across different event expressions is outside the first grouping fix. For example, these should remain separate groups initially:

```text
posedge clk_a
posedge clk_a or posedge clk_b
```

## Edge kinds

The grouping fix does not require special cases for `posedge`, `negedge`, or `edge`.

The event expression remains responsible for edge classification. The grouping layer only avoids repeating the same classification work for sources that share the same resolved event expression.

An `edge clk` group may match both rising and falling transitions. A `posedge clk` group and a `negedge clk` group are distinct groups because their event semantics differ.

## Ordering and limits

The existing output contract must remain unchanged:

- rows are ordered by event `time`;
- rows at the same time are ordered by source declaration order;
- `--max` applies after that ordering;
- truncation behavior and diagnostics remain unchanged.

A grouped implementation must therefore not emit all rows from one group before another group. It must merge group candidate timestamps and apply declaration-order sorting at each timestamp.

## Expected behavior changes

The command output should not change for valid inputs. The only intended observable difference is runtime and DEBUG timing/counter values.

Expected internal changes for the SCR1 AXI 5-channel baseline:

- `candidate_times` should remain in the same range for the shared event expression.
- Event-expression checks should be closer to the number of distinct event groups times relevant candidate timestamps, not source count times candidate timestamps.
- Predicate evaluation remains per source and should become a larger share of the remaining runtime.

## Non-goals

- Do not add protocol-specific AXI handling.
- Do not special-case signal names such as `clk`, `valid`, or `ready`.
- Do not change `extract generic` output rows, JSON/JSONL schema, diagnostics, or ordering.
- Do not add edge-aware candidate filtering as part of the grouping change.
- Do not group source predicates or payload sampling in the first grouping change.

## Risks

- Incorrect group identity could merge expressions that are not semantically identical.
- Candidate timestamp merging could break row ordering or `--max` truncation if applied per group instead of globally.
- Multi-term event expressions with `iff` guards must continue to use existing event evaluation semantics.
- DEBUG counters should remain interpretable after grouping; new group-level counters may be needed to compare against the current baseline.
