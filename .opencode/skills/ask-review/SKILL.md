---
name: ask-review
description: Request a focused peer review by spawning the reviewer subagent and providing concise context.
---

## Purpose
Route code reviews to the `review` subagent with the enough context to act.

## Workflow
1. Gather concise context: references, diff/commit range, focus files, test status, known risks or decisions.
2. Request review from `review`; provide pointers, not full file contents.
3. Reviewer sessions are stateful; use the same session for clarifications and follow-ups.
4. Iterate: clarify findings, apply fixes, commit changes, and re-request review as needed.
5. Control review: spawn a fresh reviewer session for a final pass.
6. Stop when no new substantive findings appear or the reviewer starts nitpicking; cap loops (default: 2 control passes).

## Prompt skeleton
- Scope summary: 2-4 sentences.
- Reference: plan file or other context.
- Diff/commits: range or list.
- Focus files: explicit paths.
- Tests: what ran and results.
- Known risks/assumptions: bullets.
