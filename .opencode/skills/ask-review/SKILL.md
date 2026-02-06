---
name: ask-review
description: Request a focused peer review by spawning the reviewer subagent and providing concise context.
---

## Purpose
Route plan or implementation reviews to the `reviewer` subagent with the right skill and enough context to act.

## Workflow
1. Determine review type: plan (`review-plan`) or implementation (`review-task`). If unclear, ask the user.
2. Gather concise context: plan/task reference, diff/commit range, focus files, test status, known risks or decisions.
3. Request review from `reviewer` with the appropriate skill; provide pointers, not full file contents.
4. Reviewer sessions are stateful; use the same session for clarifications and follow-ups.
5. Iterate: clarify findings, apply fixes, commit changes, and re-request review as needed.
6. Control review: spawn a fresh reviewer session for a final pass.
7. Stop when no new substantive findings appear or the reviewer starts nitpicking; cap loops (default: 2 control passes).

## Prompt skeleton
- Review type: plan or implementation (and required skill).
- Scope summary: 2-4 sentences.
- Plan/task reference: `.memory-bank/plans/.../PLAN.md` + task id.
- Diff/commits: range or list.
- Focus files: explicit paths.
- Tests: what ran and results.
- Known risks/assumptions: bullets.

## Context checklist
- Review type and required skill.
- Plan/task reference.
- Diff/commit range.
- Focus files list.
- Test status.
- Decisions or assumptions that affect review.
