---
id: troubleshooting/empty-results
title: Empty results
summary: Diagnose empty output when a bounded query legitimately returns no rows.
section: troubleshooting
see_also:
  - commands/change
  - concepts/time
---
# Empty results

An empty result is not always a failure. It often means the query was valid but
too narrow for the selected time window or selector set.

## When change queries return nothing

Recheck the time window, confirm the signal names, and verify that the trigger
is not filtering out the rows you expected to see.

## Recover safely

Widen one dimension at a time: first the time range, then the trigger, then the
signal list. This keeps the next result set explainable.
