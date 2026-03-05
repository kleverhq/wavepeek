---
name: ask-review
description: Request focused peer review using directed reviewer lanes; delegate lanes in parallel for multi-focus changes.
---

## Purpose
Route reviews to the `review` subagent with enough context to act, while maximizing signal with focus-first scoping.

## Review model
- Prefer focused review requests over omnibus "review everything" prompts.
- If the change has multiple independent focuses, delegate to multiple `review` subagents in parallel.
- Recommended focus lanes:
  - **Code lane**: correctness, regressions, edge cases, tests.
  - **Docs lane**: README/docs/help/changelog contract wording and examples.
  - **Architecture lane**: module boundaries, ownership, layering, coupling.
  - **Performance lane**: complexity, hot paths, allocations, benchmark risk.
- Single-agent review is still valid for tiny/trivial scopes.

Tiny/trivial default criteria (all should hold):
- `<=3` changed files and small diff.
- No public contract/API/schema/help surface change.
- No architectural shift.
- No performance-critical path touched.

## Workflow
1. Classify scope into one or more focus lanes.
2. Gather concise context packet: references, diff/commit range, focus files, test/bench status, known risks or decisions.
3. Choose mode:
   - Single-lane mode for tiny/trivial scopes.
   - Multi-lane mode for multi-focus scopes.
4. Request review from `review`; provide pointers, not full file contents.
5. In multi-lane mode, launch lane reviews in parallel and keep each lane prompt scoped.
6. Reviewer sessions are stateful; use the same lane session for clarifications and follow-ups.
7. Merge lane findings into one prioritized list; deduplicate overlaps and tag cross-lane impacts.
8. Iterate: clarify findings, apply fixes, commit changes, and re-request only impacted lanes when needed.
9. Control review policy:
   - Multi-lane default: run one fresh independent control pass on the consolidated diff (not one fresh pass per lane).
   - Single-lane non-trivial default: run one fresh independent control pass.
   - Single-lane tiny/trivial: one clean pass may be enough.
10. Stop when no new substantive findings appear or the reviewer starts nitpicking; cap loops (default: 2 control passes).

## Prompt requirements (every lane)
- Scope summary: 2-4 sentences.
- Lane declaration: what this lane must focus on and what it may ignore.
- Reference: plan file or other context.
- Diff/commits: range or list.
- Focus files: explicit paths.
- Tests/bench: what ran and results.
- Known risks/assumptions: bullets.

## Prompt skeleton
- Single-lane request:
  - Scope summary
  - Lane: `<code|docs|architecture|performance>`
  - Diff/commits
  - Focus files
  - Tests/bench status
  - Risks/assumptions
- Multi-lane request (repeat per lane, in parallel):
  - Scope summary
  - Lane + non-goals
  - Diff/commits
  - Lane file subset
  - Tests/bench status relevant to lane
  - Risks/assumptions
- Aggregation step after parallel lanes:
  - Merge + dedupe findings
  - Order by severity
  - Mark cross-lane dependencies/conflicts
  - Decide fix order and rerun impacted lanes
