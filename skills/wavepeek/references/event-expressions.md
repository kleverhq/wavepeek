# Event expression language contract

This document defines the `wavepeek` event expression language used by `property --on`, `change --on`, and `extract generic --on`. It follows the semantics of SystemVerilog clocking events, the forms ordinarily written inside `@(...)`, but omits the outer `@` and parentheses.

The contract is based on IEEE 1800-2023 SystemVerilog and aims to preserve
SystemVerilog-compatible syntax and semantics wherever practical for dump-based
waveform values. This document is a whitelist specification: only the syntax
and semantics explicitly described here are supported; anything not described
here is out of scope.

## 1.1 Surface Forms

Event expressions support these forms:

- wildcard event: `*`
- named event: `name`
- edge events: `posedge name`, `negedge name`, `edge name`
- unions: `event or event`, `event, event`
- gated events: `event iff logical_expr`

`*` denotes any change in the command-defined tracked set. `change` binds that
set to the resolved `--signals`; `property` binds it to the signals referenced
by `--eval`. `extract` does not support wildcard triggers. `change`,
`property`, and `extract` require an explicit `--on`; use `--on '*' --sample-mode native` for `change` or `property` when the intended trigger is the tracked-set wildcard.

## 1.2 Names and Resolution

A name may appear as a simple signal, a hierarchical path, or another canonical
dump-derived signal token accepted by the command surface. With `--scope`, names
may be relative or canonical paths inside that scope, and both forms may be
mixed. Names must resolve to signals; unresolved names are errors. Qualified
paths may contain JSON-quoted components. `$signal("path")` explicitly supplies
a path as a JSON string, including quoted root names and reserved words. See
[path resolution](paths.md).

## 1.3 Basic Event Semantics

Named event `name` means any value change of that signal. Wildcard event `*`
means any value change in the tracked set defined by the command. Edge events
use the previous sampled value strictly before the candidate timestamp and the
current sampled value at that timestamp.

Only the least-significant bit participates in edge classification. Nine-state
waveform values `h`, `u`, `w`, `l`, and `-` are normalized to `x` before edge
classification.

Edge classification follows SystemVerilog clocking-event semantics:

- `posedge` matches `0 -> 1/x/z` and `x/z -> 1`
- `negedge` matches `1 -> 0/x/z` and `x/z -> 0`
- `edge` matches either `posedge` or `negedge`

If no previous sampled value exists strictly before the timestamp, no edge is
detected at that timestamp.

## 1.4 Unions and `iff`

Union is logical OR over event terms. `or` and `,` are exact synonyms. If
multiple terms match at the same timestamp, they select the same candidate
timestamp rather than distinct duplicate events.

`iff` attaches only to the immediately preceding event term, not to the entire
union. For example, `negedge clk iff rstn or ready` means
`(negedge clk iff rstn) or ready`.

`logical_expr` uses the [Boolean expression language](boolean-expressions.md).
Parentheses are part of that `logical_expr` syntax; event expressions do not
define an independent parenthesized grouping form.

For pre-edge sampling, the default for `change` and `property` and the only mode for `extract`, the `--on` event expression still uses dump-native event detection at the trigger timestamp. This includes edge classification and any `iff` guard. Only the values printed by `change --signals`, evaluated by `property --eval`, or evaluated/sampled by `extract` predicates and payloads move to the pre-edge sample point recorded as `sample_time` in JSON and JSONL rows. The pre-edge mode is accepted only for explicit edge-only `--on` expressions: `posedge`, `negedge`, or `edge`, optionally with `iff`. For `change` and `property`, wildcard, plain-signal, and mixed triggers require `--sample-mode native`. For `extract`, wildcard, plain-signal, and mixed triggers are rejected because the command always samples pre-edge.

## 1.5 Precedence and Grouping

Event expressions have one composition operator: union. `iff` binds tighter than
union and applies to a single preceding event term. `or` and `,` have equal
precedence and associate left-to-right.

This gives the practical precedence order:

1. basic event forms: `*`, `name`, `posedge name`, `negedge name`, `edge name`
2. gated event term: `event iff logical_expr`
3. union: `event or event`, `event, event`

## 1.6 Grammar Sketch

```text
event_expr ::= event_term { ("or" | ",") event_term }

event_term ::= basic_event
             | basic_event "iff" logical_expr

basic_event ::= "*"
              | operand_reference
              | "posedge" operand_reference
              | "negedge" operand_reference
              | "edge" operand_reference
```

Notes:

- `operand_reference` follows the same name-resolution rules as elsewhere in
  this document, with command-specific scope handling.
- `iff` binds only to the immediately preceding `basic_event`.
- `or` and `,` are exact synonyms.
