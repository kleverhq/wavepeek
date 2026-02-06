---
name: write-plan
description: Create or refine PLAN.md from context and orchestrate agentic review plus user review gates.
---

## Purpose
Turn ideas and context into an execution-ready PLAN.md with clear tasks and validation.

## Inputs
- User context, goals, constraints, and success criteria.
- Repository conventions and existing docs.

## Workflow
1. Gather context; use `explore` subagents for codebase searches when needed.
2. Ask the minimum number of clarifying questions if blocked; otherwise document safe assumptions.
3. Draft PLAN.md using `references/plan-template.md` at `.memory-bank/plans/YYYY-MM-DD-<slug>/PLAN.md`.
4. Keep tasks atomic (~2-4h) and include Goal, Inputs, Known-unknowns, Steps, Outputs for each.
5. Run agentic review:
   - Load `ask-review` and request `reviewer` with the `review-plan` skill.
   - Address findings, document decisions, and commit revisions (no user gate needed).
   - After approval, spawn a fresh reviewer for a control review.
   - Stop when the fresh reviewer finds no new substantive issues or starts nitpicking; cap loops.
6. User review gate:
   - Present a plan summary and ask for feedback.
   - If sent back for changes, repeat the agentic review loop.
   - On approval, ensure the latest plan changes are committed.

## Output rules
- Plan location: `.memory-bank/plans/YYYY-MM-DD-<slug>/PLAN.md`.
- Use the template in `references/plan-template.md`.
- Default language is English unless the user requests otherwise.
- Use imperative verbs in task steps and keep assumptions explicit.
- Use concise conventional commit messages (e.g., `docs(memory): add <slug> plan`).
- Report final plan path and commit SHAs.
