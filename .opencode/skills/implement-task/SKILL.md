---
name: implement-task
description: Execute a PLAN.md task end-to-end with audit, user gates, commits, and review cycles.
---

## Inputs
- `PLAN.md` and task number, or a standalone task brief with Goal, Inputs, Known-unknowns, Steps, Outputs.
- Repository instructions (AGENTS, README, tooling) and environment constraints.

## Workflow
1. Prepare
   - Read the target task and restate scope, Definition of Done, and risks.
   - Identify dependencies, tests, rollout/metrics needs, and required tools.
2. Audit & clarify
   - Ask concise questions only if missing or contradictory info blocks execution.
   - Maintain an agreed summary (scope, DoD, risks, test plan).
3. User Gate 1 - Plan Lock
   - Present the summary and ask for approval to implement.
   - If plan/doc edits were made, commit them with a concise conventional message.
4. Implementation
   - Execute the Steps in order; note and document deviations if needed.
   - Run required checks/tests and record results.
   - Commit implementation when DoD is met (`feat|fix|chore: ...`), without waiting for a user gate.
5. Agentic review
   - Load `ask-review` and request `reviewer` with the `review-task` skill.
   - Address findings, document decisions, and commit revisions (no user gate needed).
   - After approval, spawn a fresh reviewer for a control review.
   - Stop when the fresh reviewer finds no new substantive issues or starts nitpicking; cap loops.
6. User Gate 2 - Finalize
   - Present final diff summary and test results; ask for approval.
   - If sent back, repeat the agentic review loop, then finalize.

## Output rules
- Keep changes within the agreed scope; call out any deviations.
- Use concise conventional commits and avoid bundling unrelated changes.
- Report final commit SHAs and any skipped tests.

## Reference
- Task format aligns with `.opencode/skills/write-plan/references/plan-template.md`.
