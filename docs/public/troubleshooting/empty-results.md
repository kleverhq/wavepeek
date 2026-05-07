---
id: troubleshooting/empty-results
title: Empty results
summary: Diagnose empty output when a bounded query legitimately returns no rows.
section: troubleshooting
see_also:
  - commands/change
  - reference/command-model
  - workflows/find-first-change
---
# Empty results

An empty result is not always a failure. It often means the query was valid but too narrow for the selected time window, selector set, or trigger.

## When change queries return nothing

Recheck the time window, confirm the signal names, and verify that the trigger is not filtering out the rows you expected to see.

Use `wavepeek info` to confirm dump bounds and time unit before widening a time query. Use `wavepeek scope` and `wavepeek signal` if a scope or signal name may be wrong.

## Recover safely

Widen one dimension at a time: first the time range, then the trigger, then the signal list. This keeps the next result set explainable and avoids flooding your terminal or agent context.
