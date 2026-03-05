---
name: repo-gc
description: Run repository garbage collection to remove drift and AI-slop by fixing broken references, stale docs, duplication, and unnecessary complexity while preserving behavior.
---

## Purpose
Keep the repository coherent as agent throughput increases. This skill defines a repeatable cleanup protocol: detect drift, fix high-signal issues, simplify code and docs, and leave evidence.

## Use when
- Running periodic maintenance (daily or weekly).
- A change set spans many files and patterns look inconsistent.
- Docs, tests, and implementation appear out of sync.
- You observe duplicate helpers, dead paths, or cargo-cult abstractions.

## Non-goals
- Do not add feature scope that is unrelated to cleanup.
- Do not change semantics unless the user explicitly asks.
- Do not run destructive cleanup (mass deletions, history rewrite) without strong proof and clear user intent.
- Do not touch archived (e.g. `docs/exec-plans/completed`) or generated (e.g. `bench/e2e/runs`) files

## Golden principles
1. Preserve behavior first.
2. Prefer deletion to indirection.
3. Keep one source of truth for each concept.
4. Fix root patterns, not one-off symptoms.
5. Make every cleanup verifiable with tests, checks, or runnable evidence.

## Garbage-collection protocol (mandatory order)

### 1) Frame scope and baseline
- Define scope explicitly: changed files first, then repository-wide hotspots.
- Capture baseline with `git status` and relevant quality-gate command(s).
- Record assumptions and constraints before editing.

### 2) Build a drift ledger (evidence before edits)
Create a short ledger grouped by category. Each item must include `path`, `evidence`, and `proposed fix`.

Required scan categories:
- **Reference integrity**: broken markdown links, missing documentation targets, stale command snippets, nonexistent file paths.
- **Spec drift**: docs/README/CLI help disagree with real behavior, flags, defaults, or outputs.
- **Duplication**: near-identical helpers, repeated constants or schemas, copy-pasted branches, redundant wrappers.
- **Dead or stale artifacts**: unused files, obsolete sections, abandoned TODO/FIXME notes, dead config toggles.
- **Complexity inflation**: unnecessary abstraction layers, pass-through functions, deeply nested flow where simpler control flow works.
- **Naming or structure drift**: one concept named differently across modules, inconsistent placement of similar responsibilities.

Delegation protocol (mandatory when step 2 is delegated):
- Delegate with directed subagents in parallel; do not use one omnibus scan by default.
- Minimum lanes:
  - **Code lane**: implementation, tests, configs, scripts.
  - **Docs lane**: README, docs, CLI help/man text, examples.
  - **Architecture lane**: module boundaries, ownership, layering, naming/system structure.
- A single-agent scan is allowed only for explicitly tiny scope; record the reason in baseline notes.
- Each lane must return ledger-ready items with `path`, `evidence`, and `proposed fix`.

Prompt completeness rule (mandatory):
- Do not truncate delegated prompts.
- Include full scope, exclusions, required scan categories, and output format.
- Include the full **AI-slop smell checklist** from this skill verbatim (not summarized, not referenced by title only).
- Require cross-lane tagging when one finding affects multiple lanes.

Merge rule:
- Consolidate all lane outputs into one drift ledger.
- Deduplicate overlaps, keep strongest evidence, and preserve cross-lane links.

### 3) Prioritize and batch
- Execute fixes in risk order:
  - P0: broken links, contracts, tests.
  - P1: stale docs and misleading references.
  - P2: duplication and simplification.
  - P3: cosmetic consistency.
- Keep batches atomic and reviewable; one cleanup theme per batch.

### 4) Repair with simplification rules
For each batch:
- Preserve external behavior and interfaces unless asked otherwise.
- Remove dead code only with concrete proof (no references, no runtime path, or superseded by tests/docs).
- Replace copy-paste with shared utilities only when complexity actually drops.
- Collapse unnecessary indirection and keep control flow explicit.
- Update nearby docs/tests when behavior descriptions depend on touched code.
- Avoid clever rewrites; optimize for legibility to future agents.

### 5) Validate every batch
- Run the smallest relevant checks first, then the repository standard gate when impact is broad.
- Confirm links, paths, and commands edited during cleanup actually resolve and run.
- If validation cannot run, state exactly what was not verified and why.

### 6) Report and handoff
Return a concise cleanup report with:
- Scope and baseline commands.
- Fixed items (`before -> after`).
- Deferred items with reasons and suggested follow-up.
- Validation results and residual risks.

### 7) Commit traceability (recommended)
To make cleanup cadence visible in git history, use Conventional Commit scope `gc`.

Recommended commit title format:
- `chore(gc): <batch-theme> [gc:<YYYY-MM-DD>]`

Examples:
- `chore(gc): fix stale docs links [gc:2026-02-21]`
- `chore(gc): dedupe parser helpers [gc:2026-02-21]`

Recommended commit body (short):
- `Run: gc:<YYYY-MM-DD>`
- `Ledger: <N> fixed, <M> deferred`
- `Validation: <commands and result>`

If one cleanup run has multiple commits, keep the same `gc:<YYYY-MM-DD>` tag in each commit title or body so history can be filtered by run.

## AI-slop smell checklist
Treat these as high-signal drift indicators:
- Same logic implemented in three or more places with minor renaming.
- Utility functions that only forward arguments.
- Divergent constants for the same domain concept.
- Docs describing options or flags that are not present in code.
- Generated-looking comments that restate obvious code.
- Temporary TODOs with no owner or expiry that survive multiple revisions.

## Tooling hints
- For step 2, run parallel directed `explore` subagents (code/docs/architecture) and merge outputs into one ledger.
- Avoid single-agent full-repo scans unless scope is explicitly tiny and justified.
- Use `explore` for broad repository scans and pattern inventory.
- Use `review` for a focused cleanup pass before final handoff.
- Follow repository conventions from `AGENTS.md` and `docs/DEVELOPMENT.md`.

## Execution posture
Operate proactively: scan, fix what is safe, and document what is risky.
Escalate only when uncertainty could change behavior, security posture, or data safety.
