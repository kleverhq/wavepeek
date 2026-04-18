---
id: commands/change
title: Change command
summary: Inspect value transitions across a bounded time range.
section: commands
see_also:
  - concepts/time
  - workflows/find-first-change
  - troubleshooting/empty-results
---
# Change command

Use `change` when you want transition snapshots instead of one point sample.

## Good fit

This command is useful when you already know the signals that matter and want a
bounded list of transition timestamps.

## Keep the query bounded

Start with a narrow time window and a short signal list. If the result set is
too large, tighten the trigger, shrink the range, or lower `--max`.

## Follow up with the workflow topic

If you are hunting for the earliest interesting transition, use the workflow in
`workflows/find-first-change` after confirming the signal set.
