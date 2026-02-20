# <Short, action-oriented description>

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with `exec-plan` skill.

## Purpose / Big Picture

Explain in a few sentences what someone gains after this change and how they can see it working. State the user-visible behavior you will enable.

## Non-Goals

Explicitly list what is out of scope.

## Progress

Use a list with checkboxes to summarize granular steps. Every stopping point must be documented here, even if it requires splitting a partially completed task into two (“done” vs. “remaining”). This section must always reflect the actual current state of the work.

- [x] (2025-10-01 13:00Z) Example completed step.
- [ ] Example incomplete step.
- [ ] Example partially completed step (completed: X; remaining: Y).

Use timestamps to measure rates of progress.

## Surprises & Discoveries

Document unexpected behaviors, bugs, optimizations, or insights discovered during implementation. Provide concise evidence.

- Observation: …
  Evidence: …

## Decision Log

Record every decision made while working on the plan in the format:

- Decision: …
  Rationale: …
  Date/Author: …

## Outcomes & Retrospective

Summarize outcomes, gaps, and lessons learned at major milestones or at completion. Compare the result against the original purpose.

## Context and Orientation

Describe the current state relevant to this task as if the reader knows nothing. Name the key files and modules by full path. Define any non-obvious term you will use. Do not refer to prior plans.

## Open Questions

List remaining unknowns or decisions needed.

## Plan of Work

Describe, in prose, the sequence of edits and additions. For each edit, name the file and location (function, module) and what to insert or change. Keep it concrete and minimal.

### Concrete Steps

State the exact commands to run and where to run them (working directory). When a command generates output, show a short expected transcript so the reader can compare. This section must be updated as work proceeds.

### Validation and Acceptance

Describe how to start or exercise the system and what to observe. Phrase acceptance as behavior, with specific inputs and outputs. If tests are involved, say "run <project’s test command> and expect <N> passed; the new test <name> fails before the change and passes after>".

### Idempotence and Recovery

If steps can be repeated safely, say so. If a step is risky, provide a safe retry or rollback path. Keep the environment clean after completion.

### Artifacts and Notes

Include the most important transcripts, diffs, or snippets as indented examples. Keep them concise and focused on what proves success.

### Interfaces and Dependencies

Be prescriptive. Name the libraries, modules, and services to use and why. Specify the types, traits/interfaces, and function signatures that must exist at the end of the milestone. Prefer stable names and paths such as `crate::module::function` or `package.submodule.Interface`. E.g.:

In crates/foo/planner.rs, define:

    pub trait Planner {
        fn plan(&self, observed: &Observed) -> Vec<Action>;
    }
