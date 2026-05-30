# WIP Tracker Guidance

## Scope

`docs/tracker/wip/` is for branch-local tracked artifacts that must survive across agent sessions, such as active execution plans or reviewed investigation notes.

## Local Guidance

- Use repository-root `tmp/` for ignored scratch files, logs, and disposable outputs.
- Use `docs/tracker/wip/` only when the artifact should be reviewed, committed, and available after a fresh checkout of the branch.
- Remove branch-local WIP artifacts before merging to the default branch unless a maintainer explicitly wants to keep them for handoff.
- Do not delete another agent's WIP files unless the current branch cleanup clearly owns them.
