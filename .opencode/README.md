# OpenCode Plan-Implement Workflow

This repository defines a minimal, consistent agent/skill system for a semi-autonomous Plan-Implement workflow.

## Roles
- `planner` (primary): planning lead, produces PLAN.md, orchestrates agentic review.
- `reviewer` (primary/subagent): focused review for plans or implementations.

## Skills
- `write-plan`: drafts PLAN.md and runs agentic review + control review + user gate.
- `review-plan`: audits PLAN.md for readiness and consistency.
- `implement-task`: executes a task from PLAN.md with audit, user gates, commits, and review cycles.
- `review-task`: reviews task implementation against the plan.
- `ask-review`: protocol for requesting reviewer feedback.

## Planning flow
1. Start a session with `@planner` and provide context.
2. Planner drafts `.memory-bank/plans/YYYY-MM-DD-<slug>/PLAN.md` via `write-plan` (may use `explore` subagents).
3. Agentic review loop: `reviewer` reviews the plan, fixes are committed, then a fresh reviewer runs a control pass.
4. User review gate: you review the plan; if changes are needed, the agentic review loop repeats.
5. Planner reports the plan path and commit SHAs.

## Implementation flow
1. Use the Build agent with `implement-task` and a specific task id from PLAN.md.
2. Audit & clarify the task; User Gate 1 to proceed.
3. Implement steps, run tests, and commit.
4. Agentic review + control review via `ask-review` and `reviewer`.
5. User Gate 2; if changes are requested, repeat the review loop.
6. Agent reports final commit SHAs.

## Review guidance
- Provide diff/commit ranges, focus files, and test results.
- Reviewer loads `review-plan` or `review-task` depending on scope.

## Conventions
- Plan path: `.memory-bank/plans/YYYY-MM-DD-<slug>/PLAN.md`.
- Tasks are atomic (~2-4h) and include Goal, Inputs, Known-unknowns, Steps, Outputs.
- Commits are required at each review iteration; pre-commit hooks are an implicit gate.
- Default language is English unless you request otherwise.
