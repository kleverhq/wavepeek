---
id: troubleshooting/empty-results
title: Empty results
summary: Diagnose valid queries that return no rows, no matches, or only a warning.
section: troubleshooting
see_also:
  - commands/docs
  - commands/scope
  - commands/signal
  - commands/change
  - commands/property
  - workflows/find-first-change
---
# Empty results

An empty result is not always a failure.

In `wavepeek`, many queries are allowed to succeed even when nothing matched. The fix is usually to widen or correct the query, not to debug the CLI itself.

## Know the difference between empty success and real failure

A real failure prints `error: ...` on stderr and exits non-zero.

An empty-but-valid query stays successful and usually looks like one of these forms:

- `docs search`: no matches printed,
- `scope`: no matching scopes printed,
- `signal`: no matching signals printed,
- `property`: no captured events printed,
- `change`: no rows plus `warning: no signal changes found in selected time range`.

## `docs search` can legitimately find nothing

If `wavepeek docs search <query>` prints nothing, that means the search completed but no topic matched the query tokens.

Typical recovery steps:

- shorten the query,
- try fewer or broader words,
- search for the command family first, such as `change`, `property`, or `scope`.

## `scope` and `signal` can return empty matches

Hierarchy and signal discovery commands also allow empty success.

Common causes:

- the regex is too narrow,
- the expected block or signal is spelled differently in the dump,
- the search is happening at the wrong scope,
- `signal --filter` is matching the leaf signal name, not the displayed recursive prefix,
- `scope --filter` is matching the full canonical scope path.

When in doubt, remove the filter first and confirm the raw names that the dump actually contains.

## `change` can be valid but still show no rows

`change` is the easiest command to misread here.

A query can be fully valid, the trigger can fire, and the command can still show no rows if none of the requested `--signals` changed at those sampled timestamps.

Common causes:

- the selected time window is too narrow,
- the trigger never fires in that window,
- the trigger fires, but the printed signals did not change,
- the signal list is correct, but the values were already stable,
- the query uses the wrong naming mode for `--scope` versus canonical paths.

If `change` finds no qualifying rows, it warns instead of failing:

```text
warning: no signal changes found in selected time range
```

## `property` can succeed with no output

`property` returns rows only when the selected timestamps satisfy the chosen capture mode.

That means empty output is normal when:

- the trigger selected no timestamps,
- the predicate never became true,
- `--capture assert` or `--capture deassert` asked for a transition that never happened,
- the window starts after the transition you were hoping to see.

## Most empty results come from one of four mistakes

1. **Wrong names** — the scope path or signal spelling is wrong.
2. **Wrong naming mode** — canonical names were used under `--scope`, or short names were used without it.
3. **Wrong filter or trigger** — the regex or event expression is narrower than intended.
4. **Wrong time window** — the interesting activity is outside the selected bounds.

## Recover safely

Use this order so each retry stays explainable:

1. Run `wavepeek info` to confirm dump bounds and time unit.
2. Run `wavepeek scope` and `wavepeek signal` to confirm the exact names.
3. Remove filters or simplify the trigger.
4. Widen the time window.
5. Add constraints back one at a time until the empty result reappears.

That sequence usually reveals whether the real issue is naming, filtering, or timing.
