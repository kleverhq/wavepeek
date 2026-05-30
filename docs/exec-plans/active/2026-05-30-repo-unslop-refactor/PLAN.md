# Reorganize repository contributor docs, breadcrumbs, tracking, and helper tools

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill. In the current repository layout this plan starts at `docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md`. During implementation, after `docs/tracker/wip/` exists, move this exact plan to `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md` and continue updating it there. Do not delete the plan until the full refactor is complete, reviewed, validated, and either handed off or intentionally cleaned for merge.

## Purpose / Big Picture

After this change, contributors and coding agents should find one clean contributor-documentation structure instead of a tangle of stale maps, duplicated guidance, historical execution-plan archives, and helper scripts with unclear ownership. A new contributor should be able to open `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, and `docs/dev/` and quickly learn how to work on `wavepeek`, where roadmap/backlog tracking lives, what quality gates to run, and which local breadcrumb files matter for the area they are editing.

The observable result is a repository that still builds and tests exactly as before, but whose maintainer-facing files are rearranged and shortened: development docs live under `docs/dev/`, tracking docs live under `docs/tracker/`, branch-local work artifacts live under `docs/tracker/wip/`, helper automation lives under grouped `tools/` directories, and stale `docs/exec-plans/` plus `scripts/` paths are gone. The implementation must be autonomous, must use logical commits, and must include read-only review iterations because this is a broad refactor with many easy places for stale references to hide. Naturally, those places will hide. They always do.

## Non-Goals

This plan does not change the public `wavepeek` CLI behavior, command semantics, JSON schema shape, Rust runtime architecture, benchmark baselines, fixture versions, release version, or generated schema bytes. It does not rewrite historical release entries in `CHANGELOG.md` merely because old releases mention `make`; old changelog sections are factual history and should stay factual. It does not preserve `scripts/` or `docs/exec-plans/` as compatibility directories after the migration. It does not convert public embedded docs under `docs/public/` into a flag reference or duplicate generated help. It does not clean arbitrary files under repository-root `tmp/`; that directory may contain user or other-agent scratch state.

## Progress

- [x] (2026-05-30T19:22Z) Read `tmp/unslop.md` and converted its Russian refactor notes into this self-contained implementation plan.
- [x] (2026-05-30T19:22Z) Reviewed the current repository shape: root `AGENTS.md`, all tracked `AGENTS.md` breadcrumbs, `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, `docs/ARCHITECTURE.md`, `docs/BACKLOG.md`, `docs/ROADMAP.md`, `README.md`, `CHANGELOG.md`, `justfile`, `.github/workflows/`, `.pre-commit-config.yaml`, and the current `scripts/` helpers.
- [x] (2026-05-30T19:22Z) Noted that the user asked to leave the plan in the current plan location first, then move it into the new tracking layout during implementation.
- [x] (2026-05-30T19:41Z) Ran two read-only plan review lanes and incorporated findings about self-move commit commands, retained-versus-deleted breadcrumbs, public-doc relative paths, helper-test discovery, `repo_stats.py` root calculation, concise `.fst` safety wording, and `CHANGELOG.md` `Unreleased` stale-path sweeps.
- [x] (2026-05-30T19:59Z) Updated the helper-tools milestone so `scripts/opencode_loop.py` is deleted instead of moved to a new `tools/agent/` group.
- [ ] Implementation has not started: no repository files besides this plan have been intentionally refactored yet.
- [ ] Move this plan to `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md` once `docs/tracker/wip/` exists, and update this `Progress` section immediately after the move.
- [ ] Complete Milestone 1: establish `docs/tracker/`, move backlog and roadmap, remove stale backlog entry, and create the WIP artifact policy.
- [ ] Complete Milestone 2: split `docs/DEVELOPMENT.md`, move release and architecture docs into `docs/dev/`, update `README.md`, and add `CONTRIBUTING.md`.
- [ ] Complete Milestone 3: refactor all retained non-root breadcrumbs and then the root `AGENTS.md` so breadcrumbs are concise local guidance, not duplicated maps.
- [ ] Complete Milestone 4: replace `scripts/` with grouped `tools/` directories, update automation entrypoints, and keep helper tests running.
- [ ] Complete Milestone 5: remove `docs/exec-plans/`, sweep stale references, update changelog and repo statistics, and run the full quality gate.
- [ ] Complete required review iterations after the relevant milestones and fix every substantive finding before proceeding.
- [ ] Finalize outcomes, either leave this plan in `docs/tracker/wip/` for handoff or remove it as the last WIP cleanup before merge, and record the decision in this plan before deletion if deletion is chosen.

## Surprises & Discoveries

- Observation: `tmp/unslop.md` says development tasks are done through `make`, but the current repository has already migrated to a root `justfile` and has no root `Makefile`.
  Evidence: `justfile` exists at the repository root, `AGENTS.md` and `docs/DEVELOPMENT.md` name `just`, and `git status --short` was clean before this plan was created. The implementation must preserve `just` and must not recreate a Makefile. A smoking transformer may be warm; this one is already labeled `just`.

- Observation: `CONTRIBUTING.md` does not exist yet.
  Evidence: reading `CONTRIBUTING.md` returned `ENOENT` during plan research. The implementation must create it.

- Observation: `docs/exec-plans/` currently contains many completed historical plans plus one empty active directory from an older plan.
  Evidence: `find docs/exec-plans -maxdepth 3 -type f -name PLAN.md` listed completed plans such as `docs/exec-plans/completed/2026-05-30-justfile-migration/PLAN.md`; `docs/exec-plans/active/2026-04-12-docs-design-decomposition` currently contains no files. The requested new tracking model deletes this archive after the active plan is moved.

- Observation: `docs/BACKLOG.md` still tracks “JSON schema data-field detail hardening”, but the repository already has a completed execution plan for that work and the changelog says the schema was hardened.
  Evidence: `docs/exec-plans/completed/2026-05-16-json-schema-data-field-detail-hardening/PLAN.md` exists, and `CHANGELOG.md` `Unreleased` contains schema-hardening changes. Remove that stale backlog issue while moving backlog to `docs/tracker/backlog.md`.

- Observation: current automation and tests still point at `scripts/` helper paths.
  Evidence: `justfile` references `scripts/codex_setup.sh`, `scripts/codex_resume.sh`, `scripts/check_schema_contract.py`, `scripts/check_coverage.py`, `scripts/test_extract_release_notes.py`, and `scripts/test_check_coverage.py`; `.github/workflows/release.yml` calls `scripts/extract_release_notes.py`. These must move together or the quality gates will fail with path errors, the most boring and therefore most likely failure mode.

- Observation: `scripts/repo_stats.py` hardcodes both `scripts` and `docs/exec-plans` in its repository statistics categories.
  Evidence: it collects collateral files from `bench` and `scripts`, excludes markdown under `docs/exec-plans`, and prints an `Exec plans` line. Moving helpers to `tools/` and plans to `docs/tracker/wip/` requires updating this script or accepting stale metrics.

- Observation: `tests/fixtures/docs_embed/AGENTS.md` is intentionally a tiny fixture helper, not a real breadcrumb manual.
  Evidence: its entire content is `fixture helper file that collect_markdown_files should ignore`. Keep it tiny; do not expand it into a map while refactoring breadcrumbs.

- Observation: Review of this plan caught shell validation snippets that would have succeeded or failed for the wrong reason.
  Evidence: `git ls-files <path>` exits `0` even when it prints no files, and `test ! -d scripts || git ls-files scripts | grep -q . && ...` has unsafe `&&`/`||` grouping. The plan now uses explicit `if git ls-files ... | grep -q .; then ...; fi` checks. This is why we review plans instead of admiring them from across the room.

## Decision Log

- Decision: Keep `just` as the repository task runner even though `tmp/unslop.md` says `make`.
  Rationale: the current repository state and previous migration have made `justfile` authoritative. Reintroducing `make` would be a regression and would contradict the existing workflows, pre-commit hooks, and docs.
  Date/Author: 2026-05-30 / Grin

- Decision: Create this plan first under `docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md`, then move it during implementation to `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md`.
  Rationale: the user explicitly requested that the plan start where plans are currently left and then move to the new location as part of the refactor. This also lets the move prove that the new WIP artifact policy works on the plan itself.
  Date/Author: 2026-05-30 / Grin

- Decision: Delete `docs/exec-plans/` after the active plan has moved to `docs/tracker/wip/`, including the historical completed archive.
  Rationale: `tmp/unslop.md` explicitly says the current `docs/exec-plans` should be removed. Historical plans are not product documentation, and the new policy is that branch work artifacts live in `docs/tracker/wip/` only while a branch needs them.
  Date/Author: 2026-05-30 / Grin

- Decision: Use `docs/dev/automation.md` as the file name for the requested “automation” developer document.
  Rationale: every other requested developer document has a `.md` filename, and `automation.md` clearly matches the requested CI-and-scripts topic.
  Date/Author: 2026-05-30 / Grin

- Decision: Move `docs/BACKLOG.md` to `docs/tracker/backlog.md` and `docs/ROADMAP.md` to `docs/tracker/roadmap.md` using lowercase filenames.
  Rationale: the new directory groups tracking material; lowercase names match the requested new `docs/dev/*.md` style and reduce mixed-case path churn in references.
  Date/Author: 2026-05-30 / Grin

- Decision: Use `main` as the named default branch in new documentation, while explaining the WIP cleanup rule generically as applying before merge to the default branch.
  Rationale: the repository workflows target `main`. The user's note said `master`, but the current source of truth says `main`; documentation should match the repository people actually operate.
  Date/Author: 2026-05-30 / Grin

- Decision: Replace `scripts/` with grouped root `tools/` directories rather than a single flat `tools/` dump.
  Rationale: the user requested “tools” with each script or group in a named folder with `README` and tests when present. Grouped directories make ownership visible and keep helper tests near their helper code.
  Date/Author: 2026-05-30 / Grin

- Decision: Add `tools/AGENTS.md` as a concise local breadcrumb after moving helper scripts.
  Rationale: helper automation is a safety-sensitive area with CI entrypoints, deterministic-output expectations, and test placement rules. A short breadcrumb adds useful local guidance without recreating a directory map.
  Date/Author: 2026-05-30 / Grin

- Decision: Delete `scripts/opencode_loop.py` instead of moving it under `tools/agent/`.
  Rationale: the user clarified that the OpenCode loop helper is no longer needed. Keeping a dead agent-helper group would be tidy-looking clutter, which is still clutter wearing a little hat.
  Date/Author: 2026-05-30 / Grin

- Decision: Remove the bulky root `Critical Tool Safety Rule` section but preserve a concise binary-waveform safety bullet in the root core workflow.
  Rationale: the user requested removal of the stale section. A short safety sentence still protects future agents from treating `.fst` waveform dumps as text, while avoiding the old dedicated section that made the root breadcrumb too heavy.
  Date/Author: 2026-05-30 / Grin

- Decision: Require commit checkpoints and review iterations in the implementation plan.
  Rationale: this change touches repository guidance, docs, workflows, tests, and helper paths. Small commits and focused reviews make stale references and accidental behavior changes easier to catch and revert.
  Date/Author: 2026-05-30 / Grin

- Decision: Do not delete arbitrary files in repository-root `tmp/` during this refactor.
  Rationale: `tmp/` is git-ignored scratch space and may contain artifacts owned by other agents or the user. The new docs should say to use it, not to sweep it like a tiny digital landfill fire.
  Date/Author: 2026-05-30 / Grin

## Outcomes & Retrospective

At plan creation time, no implementation has been done. The plan resolves the conflict between the user note's `make` wording and the current repository's `just` migration, chooses concrete target paths, and provides a milestone sequence with validation, commits, and review loops. The main remaining risk is stale path fallout: old docs and automation references to `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, `docs/ARCHITECTURE.md`, `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/exec-plans/`, and `scripts/` must be swept carefully.

Add a new retrospective entry after each major milestone and again at completion. If the final branch removes this plan from `docs/tracker/wip/` before merge, write the final retrospective before deleting the plan and include the removal in the final commit message.

## Context and Orientation

`wavepeek` is a Rust command-line tool for deterministic inspection of `.vcd` and `.fst` waveform dumps. The product code lives under `src/`, public embedded user docs live under `docs/public/`, the packaged agent skill lives at `docs/skills/wavepeek.md`, JSON schema artifacts live under `schema/`, integration tests live under `tests/`, benchmarks live under `bench/`, and repository automation is driven by the root `justfile`.

An `AGENTS.md` file is a local context breadcrumb for coding agents. It should contain short, path-scoped guidance that helps an agent work safely in that area. It should not be a copied README, a parent/child directory index, a changelog, or a second backlog. This refactor intentionally removes map-like parent/child lists from non-root breadcrumbs and makes the root breadcrumb a short entry point plus map.

An ExecPlan is a living implementation plan. Historically, this repository stored active and completed ExecPlans under `docs/exec-plans/`. This refactor replaces that with a new tracker model: durable planning docs go under `docs/tracker/`, and branch-local artifacts such as active plans go under `docs/tracker/wip/` only while a working branch needs them. The `docs/tracker/wip/AGENTS.md` file must explain that WIP artifacts are the tracked counterpart to ignored `tmp/` files and should be cleared before merging to the default branch.

The current root developer guide is `docs/DEVELOPMENT.md`. It mixes environment setup, quality gates, testing commands, benchmark workflow, changelog policy, Rust style, CLI constraints, and public docs maintenance. The release runbook is `docs/RELEASE.md`, and the internal architecture document is `docs/ARCHITECTURE.md`. This plan splits those into the requested `docs/dev/` files so contributors can open the topic they need instead of spelunking one giant file with a flashlight and mild regret.

The current helper scripts live under `scripts/`. They are invoked by `justfile`, `.github/workflows/release.yml`, and Python unit tests. The new target is grouped `tools/` directories, each with a short `README.md` explaining ownership and commands. Helper tests should move with the helpers they test.

The current tracking docs are `docs/BACKLOG.md` and `docs/ROADMAP.md`. `docs/BACKLOG.md` contains open design questions plus issues; one issue, “JSON schema data-field detail hardening”, is stale and should be removed because that work is now present in completed history and `CHANGELOG.md`. `docs/ROADMAP.md` currently says the next post-v0.5.0 release is not planned yet and points back to `docs/BACKLOG.md`; after the move it should point to `docs/tracker/backlog.md` or use a relative `backlog.md` reference.

The current `README.md` has a short `Development` section that points only to `docs/DEVELOPMENT.md`. After this refactor it should contain a concise map of the new `docs/dev/` documents. `CONTRIBUTING.md` does not exist and must be created.

Quality gates are container-first. In this repository, `WAVEPEEK_IN_CONTAINER=1` is expected inside the devcontainer or CI image. The usual local pre-handoff gate is `just check`; the test-inclusive CI-parity gate is `just ci`. The implementation should also run targeted checks after moving helpers because path changes can fail quickly and cheaply before the full gate.

## Open Questions

No user-facing decision is intentionally left open. If the implementing agent discovers additional stale references or helper groups, resolve them consistently with this plan: prefer `docs/dev/` for developer process, `docs/tracker/` for planning/tracking, `docs/tracker/wip/` for branch artifacts, `tools/` for helper automation, and concise breadcrumbs for local agent guidance. Record any non-trivial deviation in the `Decision Log` before committing it.

## Plan of Work

Work in small commits. At the start of every milestone, run `git status --short` and read the current location of this plan. At the end of every milestone, update `Progress`, `Surprises & Discoveries`, `Decision Log` if needed, and `Outcomes & Retrospective` if the milestone produced a meaningful result. Then run the targeted validation listed for that milestone and commit with a conventional commit message. Do not ask the user for next steps; if a path conflict appears, use the policies in this plan and keep moving.

If this plan file is still untracked when implementation begins, make the first commit only for the plan:

    git status --short
    git add docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md
    git commit -m "docs(plan): add repository unslop refactor plan"

If hooks are installed, let them run. Do not use `--no-verify` unless the user explicitly asks. If the plan is already committed, skip this commit. After that, use the milestone commits below.

### Milestone 1: Establish tracker layout and move the active plan

Create the new tracking home before large docs edits so every later reference has a real destination. At the end of this milestone, `docs/tracker/backlog.md`, `docs/tracker/roadmap.md`, `docs/tracker/wip/AGENTS.md`, and the moved plan under `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md` should exist. The old `docs/BACKLOG.md` and `docs/ROADMAP.md` should be gone. `docs/exec-plans/` may still exist until Milestone 5, but this plan should no longer live there.

Move the tracking docs and remove the stale schema-hardening backlog issue:

    mkdir -p docs/tracker/wip/2026-05-30-repo-unslop-refactor
    git mv docs/BACKLOG.md docs/tracker/backlog.md
    git mv docs/ROADMAP.md docs/tracker/roadmap.md

Edit `docs/tracker/backlog.md` so it keeps the open design questions and still-open issues, but deletes the entire `### JSON schema data-field detail hardening` issue. Keep the title as `# Backlog` unless a better plain title is needed. Edit `docs/tracker/roadmap.md` so any reference to `docs/BACKLOG.md` becomes `docs/tracker/backlog.md` or simply `backlog.md` if the reference is local prose.

Create `docs/tracker/wip/AGENTS.md` with concise guidance, not a directory map. It should say that this directory is for branch-local tracked artifacts that must survive across agent sessions but should be removed before merge to the default branch. It must contrast `docs/tracker/wip/` with root `tmp/`: `tmp/` is ignored scratch, while `docs/tracker/wip/` is for reviewed, committed branch artifacts. It should also say not to delete other agents' WIP files unless the current branch cleanup explicitly owns them.

Move this plan. If the plan is tracked, use `git mv`:

    git mv docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md

If the plan is untracked because the initial plan commit was skipped, use ordinary `mv` and then `git add` the new file:

    mv docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md
    git add docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md

Immediately edit the moved plan's opening note and `Progress` entries so they name the new current path. From this point forward, every update should be made in `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md`, not in `docs/exec-plans/`.

Run targeted validation:

    test -f docs/tracker/backlog.md
    test -f docs/tracker/roadmap.md
    test -f docs/tracker/wip/AGENTS.md
    test -f docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md
    test ! -f docs/BACKLOG.md
    test ! -f docs/ROADMAP.md
    rg -n "JSON schema data-field detail hardening" docs/tracker/backlog.md && exit 1 || true
    git diff --check

Commit this milestone with:

    git add -A docs/tracker docs/BACKLOG.md docs/ROADMAP.md docs/exec-plans
    git commit -m "docs: establish tracker workspace"

The important thing is that the commit contains the tracker move, the WIP policy, the stale backlog deletion, and the plan move. Use `git status --short` to confirm both old paths are staged as removals and the new WIP plan path is staged as an addition or rename.

### Milestone 2: Split development docs and add contributor guidance

Create `docs/dev/` and split the current monolithic developer material into topic files. At the end of this milestone, `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, and `docs/ARCHITECTURE.md` should be gone; their maintained content should live under `docs/dev/`. `README.md` should have a useful `Development` map, and `CONTRIBUTING.md` should exist.

Create these files:

- `docs/dev/quality.md` for quality gates, coverage threshold, `just check`, `just ci`, `just pre-commit`, `just check-commit`, and how to interpret failures.
- `docs/dev/testing.md` for Rust tests, auxiliary Python tests, fixture strategy, snapshot policy at a high level, and when to use targeted versus full tests.
- `docs/dev/environment.md` for container-first workflow, devcontainer versus CI container, Codex setup/resume, fixture location resolution, debug mode, and repository-root `tmp/` guidance. It must say that `tmp/` is ignored scratch and must not be globally cleaned because other agents or the user may own files there.
- `docs/dev/style.md` for Rust formatting, imports, naming, ownership, error handling, deterministic output, CLI design constraints, and public docs metadata rules.
- `docs/dev/benchmarking.md` for CLI end-to-end benchmarks under `bench/e2e/` and expression microbenchmarks under `bench/expr/`.
- `docs/dev/changelog.md` for Keep a Changelog policy and release-note expectations.
- `docs/dev/release.md` by moving and updating `docs/RELEASE.md`.
- `docs/dev/architecture.md` by moving and updating `docs/ARCHITECTURE.md`.
- `docs/dev/git.md` for conventional commits, commit-message checks, when to commit, root `tmp/`, `docs/tracker/wip/`, and the rule that WIP artifacts should be removed before merging to the default branch.
- `docs/dev/automation.md` for root `justfile`, `.github/workflows/`, `.pre-commit-config.yaml`, CI/release automation, and the new `tools/` helper layout after Milestone 4.

Use `git mv docs/RELEASE.md docs/dev/release.md` and `git mv docs/ARCHITECTURE.md docs/dev/architecture.md` so history is preserved. For `docs/DEVELOPMENT.md`, it is acceptable to create new files by copying and reducing sections, then `git rm docs/DEVELOPMENT.md` once references are updated. Do not blindly duplicate the old file into every new file; split by topic and remove repeated wording. Keep each document source-of-truth oriented. If a detail is already canonical in `justfile`, a workflow, or a public reference doc, point to that file and add only the needed maintainer context.

Update paths inside the moved documents. In particular, `docs/dev/release.md` should reconcile roadmap via `docs/tracker/roadmap.md`, not `docs/ROADMAP.md`; release-note extraction should point to the eventual `tools/release/extract_release_notes.py` path after Milestone 4, or include a temporary note that Milestone 4 will update it before final validation. `docs/dev/architecture.md` should continue to describe internals and should not become a second CLI reference. Any references to `docs/DEVELOPMENT.md` should become the specific `docs/dev/*.md` topic that now owns the relevant content.

Create `CONTRIBUTING.md`. It should be short and direct. It must tell contributors to read `docs/dev/`, check `docs/tracker/backlog.md`, `docs/tracker/roadmap.md`, and GitHub issues before proposing work, and open an issue first for changes that need maintainer discussion. It must say that PRs opened without a relevant issue may be ignored or closed, that direct maintainer conversation is preferred when a discussion is needed, that contributions are welcome, and that low-effort AI-generated slop spam is categorically unwelcome. Keep it firm but not needlessly theatrical; this is a project boundary, not a manifesto nailed to a barn door.

Update the `README.md` `Development` section. Replace the single `docs/DEVELOPMENT.md` pointer with a concise map of the new docs, for example:

    ## Development

    Maintainer workflow lives under `docs/dev/`:

    - `docs/dev/environment.md` for devcontainer, CI image, Codex, fixtures, and `tmp/`.
    - `docs/dev/quality.md` for `just check`, `just ci`, coverage, and hooks.
    - `docs/dev/testing.md` for test strategy and fixtures.
    - `docs/dev/style.md` for Rust, CLI, output, and docs conventions.
    - `docs/dev/benchmarking.md` for E2E and expression benchmark workflows.
    - `docs/dev/automation.md` for CI, `justfile`, pre-commit, and helper tools.
    - `docs/dev/git.md`, `docs/dev/changelog.md`, and `docs/dev/release.md` for contribution hygiene and releases.
    - `docs/dev/architecture.md` for internal module boundaries.

Run targeted validation:

    for path in docs/dev/quality.md docs/dev/testing.md docs/dev/environment.md docs/dev/style.md docs/dev/benchmarking.md docs/dev/changelog.md docs/dev/release.md docs/dev/architecture.md docs/dev/git.md docs/dev/automation.md CONTRIBUTING.md; do test -f "$path" || exit 1; done
    test ! -f docs/DEVELOPMENT.md
    test ! -f docs/RELEASE.md
    test ! -f docs/ARCHITECTURE.md
    rg -n "docs/(DEVELOPMENT|RELEASE|ARCHITECTURE)\.md|DEVELOPMENT\.md|RELEASE\.md|ARCHITECTURE\.md" README.md CONTRIBUTING.md docs AGENTS.md bench schema src tests .devcontainer -g '!docs/tracker/wip/**' -g '!docs/exec-plans/**' && exit 1 || true
    git diff --check

The `rg` command should produce no live references except inside ignored WIP or soon-to-be-deleted historical plan files. If it finds stale references in live docs or breadcrumbs, fix them before committing. Do not rewrite historical completed plans if they still exist at this point; they will be removed in Milestone 5.

Run the first docs review iteration after the targeted validation and before the commit. Spawn one or more read-only reviewers. Ask them to focus on development-doc structure, missing source-of-truth links, stale paths, and whether `CONTRIBUTING.md` matches the requested policy. Fix substantive findings, update this plan, rerun targeted validation, then commit:

    git add README.md CONTRIBUTING.md docs/dev docs/DEVELOPMENT.md docs/RELEASE.md docs/ARCHITECTURE.md docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md
    git commit -m "docs: split developer guide"

### Milestone 3: Refactor breadcrumbs into concise local guidance

Refactor every retained tracked `AGENTS.md` except the root first, then rewrite the root `AGENTS.md`. Retained means breadcrumbs that will still exist after this refactor. Explicitly exempt `docs/exec-plans/AGENTS.md` and `scripts/AGENTS.md` from refactoring because those directories are deleted in later milestones. The goal is not to make breadcrumbs tiny at any cost; the goal is to remove duplicated maps, parent/child lists, stale references, and copied documentation while keeping the local gotchas that actually help an agent avoid damage.

For every breadcrumb, use only sections that carry value, such as `Scope`, `Source of Truth`, `Local Guidance`, `Workflow`, and `Safety`. Write paths relative to the breadcrumb file. Do not maintain parent-map or child-map lists. Do not add a local breadcrumb merely because a directory exists. Preserve `tests/fixtures/docs_embed/AGENTS.md` as a tiny fixture helper note because tests rely on it being ignorable fixture content.

Refactor these existing non-root breadcrumbs:

- `.devcontainer/AGENTS.md`: keep the non-obvious container decisions, fixture provisioning notes, and the coupling between `.devcontainer/env_contract.sh`, `.devcontainer/resolve_rtl_artifacts_dir.sh`, and the future `tools/codex/` scripts. Remove `Parent Map`.
- `bench/AGENTS.md`: keep performance guidance, point to `../docs/dev/benchmarking.md`, and remind agents to run benchmark work in the container. Remove the area map unless it adds non-obvious value.
- `bench/expr/AGENTS.md`: keep the expression-benchmark target list and run-artifact rules, update roadmap references to `../../docs/tracker/roadmap.md`, and point workflow to `../../docs/dev/benchmarking.md`.
- `docs/AGENTS.md`: make it a concise docs-local guidance file. Point to `public/intro.md`, `public/reference/`, `dev/`, `tracker/`, and `../CHANGELOG.md`, but do not list child breadcrumb files.
- `docs/public/AGENTS.md`, `docs/public/commands/AGENTS.md`, and `docs/public/reference/AGENTS.md`: keep topic metadata rules and source-of-truth pointers, update old development-doc references to the correct relative paths, and remove parent/child map sections. From `docs/public/AGENTS.md`, `docs/dev/style.md` is `../dev/style.md`; from `docs/public/commands/AGENTS.md` and `docs/public/reference/AGENTS.md`, it is `../../dev/style.md`.
- `docs/skills/AGENTS.md`: keep skill-source guidance and runtime source pointers, remove parent-map wording.
- `schema/AGENTS.md`: point schema generation and validation to `../justfile`, future `../tools/schema/check_schema_contract.py`, and `../docs/public/reference/machine-output.md`.
- `src/AGENTS.md`: point development conventions to `../docs/dev/style.md`, architecture to `../docs/dev/architecture.md`, and public contracts to `../docs/public/reference/`. Add the requested reminder that code changes should keep `../docs/dev/architecture.md` consistent when module boundaries or execution layers change. Remove the deleted completed-plan reference.
- `tests/AGENTS.md`: point testing workflow to `../docs/dev/testing.md`, quality gates to `../docs/dev/quality.md`, and product contracts to public references. Keep the existing expression fixture rules if they are still useful; those are real local guidance, not generic map filler.
- `tests/fixtures/docs_embed/AGENTS.md`: leave it as a one-line or similarly tiny fixture helper that says the docs embedding collector should ignore it.

After Milestone 4 moves `scripts/` to `tools/`, `scripts/AGENTS.md` will be deleted with the directory. In this milestone, if `tools/` does not exist yet, do not create a fake final breadcrumb there unless you are also moving tools now. The final `tools/AGENTS.md` is specified in Milestone 4.

Rewrite root `AGENTS.md` with `## Core Workflow` first. It should say:

- `wavepeek` is a Rust CLI tool for deterministic `.vcd` and `.fst` waveform inspection.
- Development is container-first; run repository gates in the devcontainer/CI image.
- Development tasks are run through root `justfile` recipes, not `make`.
- Standard quality gate is `just ci`; local pre-handoff gate is `just check`.
- Use repository-root `tmp/` for disposable scratch, logs, and ad hoc outputs, but never delete arbitrary existing files there because they may belong to the user or another agent.
- Treat binary waveform dumps such as `.fst` as binary data; inspect them through `wavepeek`, fixtures, or purpose-built tools rather than text-reading them directly.
- Read the nearest applicable `AGENTS.md` before editing files; local breadcrumbs may contain extra rules and gotchas.

Then add `## Map` with only a short bullet map. Include `src/` for sources, `tests/` for tests, `tools/` for helper automation after Milestone 4, `bench/` for benchmarks, `.github/workflows/` for CI, `.devcontainer/` for devcontainer setup, `docs/dev/` for maintainer docs, `docs/tracker/` for backlog/roadmap/WIP artifacts, `docs/public/` for embedded user docs, and `schema/` for schema artifacts. Remove `Start Here`, `Child Maps`, `Breadcrumb Policy`, the dedicated `Critical Tool Safety Rule` section, and `Devcontainer Notes` from the root file. The user explicitly asked for those section removals; the concise binary-data safety bullet above keeps the useful part without preserving the bulky section.

Run targeted validation:

    rg --files -g 'AGENTS.md' -g '!target/**' -g '!tmp/**' | sort
    rg -n "Parent Map|Parent Maps|Child Maps|Breadcrumb Policy|Devcontainer Notes|Critical Tool Safety Rule" -g 'AGENTS.md' -g '!target/**' -g '!tmp/**' -g '!docs/exec-plans/**' && exit 1 || true
    rg -n "docs/(DEVELOPMENT|RELEASE|ARCHITECTURE|BACKLOG|ROADMAP)\.md|docs/exec-plans|scripts/AGENTS\.md" -g 'AGENTS.md' -g '!target/**' -g '!tmp/**' -g '!docs/exec-plans/**' && exit 1 || true
    git diff --check

Run a breadcrumb review iteration. Use a read-only reviewer and provide this focus: check every changed `AGENTS.md` against the context-breadcrumb policy, verify relative paths, identify duplicated source-of-truth text, and flag any guidance that is stale after the planned `docs/dev`, `docs/tracker`, and `tools` moves. Fix substantive findings and rerun the validation. Commit:

    git add AGENTS.md .devcontainer/AGENTS.md bench/AGENTS.md bench/expr/AGENTS.md docs/AGENTS.md docs/public/AGENTS.md docs/public/commands/AGENTS.md docs/public/reference/AGENTS.md docs/skills/AGENTS.md schema/AGENTS.md src/AGENTS.md tests/AGENTS.md tests/fixtures/docs_embed/AGENTS.md docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md
    git commit -m "docs: simplify agent breadcrumbs"

### Milestone 4: Move helper scripts into grouped tools

Replace the flat `scripts/` directory with grouped `tools/` directories. At the end of this milestone, the repository should no longer need `scripts/` for tracked helper code, `justfile` should call the new paths, release workflow should extract notes through `tools/release/extract_release_notes.py`, and auxiliary tests should still run.

Use this target layout unless implementation discovers a clearly better grouping:

- `tools/coverage/README.md`, `tools/coverage/check_coverage.py`, and `tools/coverage/test_check_coverage.py`.
- `tools/schema/README.md` and `tools/schema/check_schema_contract.py`.
- `tools/release/README.md`, `tools/release/extract_release_notes.py`, and `tools/release/test_extract_release_notes.py`.
- `tools/codex/README.md`, `tools/codex/codex_env_common.sh`, `tools/codex/codex_setup.sh`, and `tools/codex/codex_resume.sh`.
- `tools/repo/README.md` and `tools/repo/repo_stats.py`.
- `tools/AGENTS.md` for concise helper-tool guidance.

Use `git mv` for tracked files that remain useful. Delete `scripts/opencode_loop.py` with `git rm` and do not create `tools/agent/`; that helper is no longer needed. Do not move `scripts/__pycache__/`; it is generated state and should remain untracked or ignored. If it is accidentally tracked in the local checkout, stop and inspect with `git ls-files scripts/__pycache__` before deciding. Generated Python cache files do not belong in the new `tools/` tree.

Update `tools/codex/*.sh` after moving them. The scripts currently compute `SCRIPT_DIR` and source `codex_env_common.sh` from the same directory, so same-directory moves should remain simple. In `tools/codex/codex_env_common.sh`, update `REPO_ROOT` calculation from `SCRIPT_DIR/..` to `SCRIPT_DIR/../..` because the script will be two levels below the repository root. Check any comments that mention `scripts/` and update them to `tools/codex/`.

Update `justfile`:

- `codex_setup_script` should become `tools/codex/codex_setup.sh`.
- `codex_resume_script` should become `tools/codex/codex_resume.sh`.
- `check-schema` should run `python3 -B tools/schema/check_schema_contract.py`.
- `coverage-src` and `coverage-src-check` should run `python3 -B tools/coverage/check_coverage.py`.
- `test-aux` should run the bench test suites and the moved helper tests. A simple explicit version is acceptable:

    python3 -B -m unittest discover -s bench/e2e -p "test_*.py"
    python3 -B -m unittest discover -s bench/expr -p "test_*.py"
    python3 -B -m unittest tools/release/test_extract_release_notes.py
    python3 -B -m unittest tools/coverage/test_check_coverage.py

  Do not replace these explicit helper-test commands with a single `python3 -B -m unittest discover -s tools -p "test_*.py"` unless you also make the grouped directories discoverable and prove the command runs a non-zero number of tests. Plain unittest discovery can silently miss tests in nested helper directories without package markers. Very helpful, in the way trapdoors are helpful.

Update `.github/workflows/release.yml` so release notes are extracted from `tools/release/extract_release_notes.py`. Update `.pre-commit-config.yaml` descriptions if they mention `scripts`. Update `docs/dev/automation.md`, `docs/dev/environment.md`, `docs/dev/release.md`, `schema/AGENTS.md`, `.devcontainer/AGENTS.md`, and any README snippets to point to `tools/` instead of `scripts/`.

Update `tools/repo/repo_stats.py` for the new layout. Because the file moves from `scripts/repo_stats.py` to `tools/repo/repo_stats.py`, change its repository-root calculation from `pathlib.Path(__file__).resolve().parent.parent` to `pathlib.Path(__file__).resolve().parents[2]`. It should collect collateral helper code from `tools` instead of `scripts`, exclude `docs/tracker/wip` from ordinary markdown counts, and either remove the separate `Exec plans` category or rename it to something like `WIP tracker artifacts` if counting `docs/tracker/wip` remains useful. The output should not mention `scripts` or `docs/exec-plans` after the migration.

Each new `tools/*/README.md` should be short. Include what the tool group is for, the normal entrypoint command, and how tests are run if tests exist. Do not duplicate entire scripts or workflow docs.

Run targeted validation:

    test -d tools
    test ! -e tools/agent
    if git ls-files scripts | grep -q .; then echo "tracked scripts remain" >&2; exit 1; fi
    python3 -B -m unittest tools/release/test_extract_release_notes.py
    python3 -B -m unittest tools/coverage/test_check_coverage.py
    WAVEPEEK_IN_CONTAINER=1 just test-aux
    WAVEPEEK_IN_CONTAINER=1 just check-schema
    WAVEPEEK_IN_CONTAINER=1 just check-actions
    python3 -B tools/repo/repo_stats.py
    python3 -c "import pathlib; root = pathlib.Path('tools/repo/repo_stats.py').resolve().parents[2]; assert (root / 'Cargo.toml').exists(), root; assert (root / 'src').is_dir(), root"
    rg -n "scripts/" justfile .github .pre-commit-config.yaml README.md CONTRIBUTING.md docs tools AGENTS.md bench schema src tests -g '!docs/tracker/wip/**' -g '!docs/exec-plans/**' && exit 1 || true
    git diff --check

Be careful with the tracked-file check: if untracked ignored cache files keep the physical `scripts/` directory present, do not fail the migration merely because ignored cache state exists. The real acceptance condition is that `git ls-files scripts` prints nothing and no live tracked reference points at `scripts/`.

Run an automation review iteration. Use a read-only reviewer focused on `justfile`, `.github/workflows/`, `.pre-commit-config.yaml`, `tools/`, helper tests, and path correctness. Ask for findings with file and line references. Fix substantive findings, rerun targeted validation, update this plan, then commit:

    git add -A scripts tools justfile .github/workflows/release.yml .pre-commit-config.yaml docs AGENTS.md .devcontainer/AGENTS.md schema/AGENTS.md docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md
    git commit -m "chore: organize helper tools"

### Milestone 5: Delete old execution-plan archive, sweep references, and finalize docs

After the active plan has moved and the new tracker model is documented, delete the old execution-plan archive. At the end of this milestone, `docs/exec-plans/` should not exist in tracked files. Any reference to old uppercase docs, old `scripts/` paths, or old `docs/exec-plans/` should either be gone or be explicitly historical in `CHANGELOG.md`.

Remove the old archive:

    git rm -r docs/exec-plans

If `git rm` reports that the current plan path is already gone, that is expected. If the directory contains untracked files, inspect them with `find docs/exec-plans -maxdepth 3 -type f -print` and decide whether they are disposable generated state or branch artifacts that should move under `docs/tracker/wip/`. Do not silently delete unknown user files.

Update references across live files:

- `docs/AGENTS.md` should no longer mention `exec-plans`.
- `src/AGENTS.md` should no longer point to a completed self-documenting CLI docs plan.
- `README.md`, `CONTRIBUTING.md`, and `AGENTS.md` should point to `docs/dev/` and `docs/tracker/`.
- `bench/expr/AGENTS.md` and `docs/dev/release.md` should point to `docs/tracker/roadmap.md` where roadmap context matters.
- `docs/dev/automation.md` should describe `tools/`, not `scripts/`.
- `tools/repo/repo_stats.py` should no longer mention `docs/exec-plans`.

Update `CHANGELOG.md` under `## [Unreleased]`. Add a concise contributor-facing note under `Changed` or `Added` that this branch reorganized maintainer docs into `docs/dev/`, tracking into `docs/tracker/`, WIP branch artifacts into `docs/tracker/wip/`, and helper automation into `tools/`. Also update any existing `Unreleased` item that points at a moved live path, such as `bash scripts/codex_setup.sh`, so it points at the new `tools/codex/codex_setup.sh` path. Do not rewrite older released sections that mention `make`; those are historical release notes.

Run stale-reference sweeps. Use these commands from the repository root:

    rg -n "docs/(DEVELOPMENT|RELEASE|ARCHITECTURE|BACKLOG|ROADMAP)\.md|DEVELOPMENT\.md|RELEASE\.md|ARCHITECTURE\.md|BACKLOG\.md|ROADMAP\.md" AGENTS.md README.md CONTRIBUTING.md docs tools justfile .github .pre-commit-config.yaml bench schema src tests -g '!docs/tracker/wip/**' && exit 1 || true
    rg -n "docs/exec-plans|exec-plans" AGENTS.md README.md CONTRIBUTING.md docs tools justfile .github .pre-commit-config.yaml bench schema src tests -g '!docs/tracker/wip/**' && exit 1 || true
    rg -n "scripts/" AGENTS.md README.md CONTRIBUTING.md docs tools justfile .github .pre-commit-config.yaml bench schema src tests -g '!docs/tracker/wip/**' && exit 1 || true
    awk '/^## \[Unreleased\]/{flag=1; next} /^## \[/{flag=0} flag {print}' CHANGELOG.md | rg -n "scripts/|docs/(DEVELOPMENT|RELEASE|ARCHITECTURE|BACKLOG|ROADMAP)\.md|docs/exec-plans" && exit 1 || true
    if git ls-files docs/exec-plans scripts | grep -q .; then echo "old tracked directories remain" >&2; exit 1; fi
    git diff --check

The sweeps intentionally exclude `CHANGELOG.md` because older changelog sections can mention old commands and paths. They also exclude the WIP plan because the plan necessarily discusses old paths while documenting the migration. If live files outside those exclusions still mention old paths, fix them.

Run full validation:

    WAVEPEEK_IN_CONTAINER=1 just format-check
    WAVEPEEK_IN_CONTAINER=1 just check
    WAVEPEEK_IN_CONTAINER=1 just ci

If `just ci` is too expensive for the current environment or fails due to external container/fixture constraints unrelated to this refactor, record the exact failure in `Surprises & Discoveries`, run the largest available subset (`just check`, `just test-aux`, `just check-schema`, `just check-actions`, and targeted helper tests), and state clearly in the final response what did not run. Do not pretend the fog is a landscape.

Run a final control review iteration after full validation. Use a fresh read-only reviewer not used for earlier focused reviews. Give it this scope: the entire diff from the branch start through the current working tree, this plan, and the target-state requirements from `tmp/unslop.md` as embedded in this plan. Ask it to find stale paths, broken commands, docs contradictions, breadcrumb-policy violations, and missing validation. Fix substantive findings, rerun relevant checks, and if the fix touches multiple areas, run one more targeted reviewer for that area. Stop after two control passes unless a critical issue remains.

Commit this milestone with:

    git add -A
    git commit -m "docs: finalize repository documentation refactor"

### Milestone 6: WIP plan cleanup or handoff

At this point, decide whether the branch is being handed off for more work or prepared for merge. If another agent or maintainer still needs this plan as restart context, leave `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md` in place and make sure `Outcomes & Retrospective` accurately describes current state, validation, review iterations, commits, and remaining work. This is the safer handoff posture.

If the branch is fully complete and is being prepared for merge to the default branch, follow the policy introduced by this refactor: remove branch-local WIP artifacts before merge. In that case, write a final `Outcomes & Retrospective` entry in this plan, then remove only this plan directory and leave `docs/tracker/wip/AGENTS.md` behind:

    git rm -r docs/tracker/wip/2026-05-30-repo-unslop-refactor
    test -f docs/tracker/wip/AGENTS.md
    git commit -m "docs: clear completed wip artifact"

Do not remove another branch artifact from `docs/tracker/wip/` unless it belongs to this same refactor and the plan explains why. Do not remove root `tmp/` files.

## Concrete Steps

A competent agent can implement the plan by following the milestones in order. The most important command sequence is:

    git status --short
    # Optional if this plan is uncommitted:
    git add docs/exec-plans/active/2026-05-30-repo-unslop-refactor/PLAN.md
    git commit -m "docs(plan): add repository unslop refactor plan"

    # Milestone 1: create docs/tracker, move backlog/roadmap, create wip AGENTS, move this plan.
    # Commit: docs: establish tracker workspace

    # Milestone 2: create docs/dev split, README Development map, CONTRIBUTING.md.
    # Review docs lane.
    # Commit: docs: split developer guide

    # Milestone 3: refactor AGENTS breadcrumbs.
    # Review breadcrumb lane.
    # Commit: docs: simplify agent breadcrumbs

    # Milestone 4: move scripts to tools and update automation.
    # Review automation lane.
    # Commit: chore: organize helper tools

    # Milestone 5: remove docs/exec-plans, sweep stale references, changelog, full validation.
    # Final control review.
    # Commit: docs: finalize repository documentation refactor

    # Milestone 6: leave or remove WIP plan depending on handoff versus merge readiness.

Use `git status --short` before every commit. Use `git diff --stat HEAD` and `git diff --name-only HEAD` before review prompts so reviewers can inspect scope. Keep this plan updated in the same commit as the work it describes.

## Validation and Acceptance

The final refactor is accepted when a human can verify these behaviors from a clean checkout of the branch:

- `README.md` `Development` points to `docs/dev/` and each listed file exists.
- `CONTRIBUTING.md` exists and sets the expected issue-first, maintainer-discussion, no-AI-slop policy.
- `docs/dev/quality.md`, `testing.md`, `environment.md`, `style.md`, `benchmarking.md`, `changelog.md`, `release.md`, `architecture.md`, `git.md`, and `automation.md` exist and replace the old monolithic developer docs.
- `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, and `docs/ARCHITECTURE.md` are not tracked.
- `docs/tracker/backlog.md`, `docs/tracker/roadmap.md`, and `docs/tracker/wip/AGENTS.md` exist.
- The stale backlog issue `JSON schema data-field detail hardening` is absent from `docs/tracker/backlog.md`.
- `docs/BACKLOG.md` and `docs/ROADMAP.md` are not tracked.
- `docs/exec-plans/` is not tracked after Milestone 5.
- The active plan has been moved to `docs/tracker/wip/2026-05-30-repo-unslop-refactor/PLAN.md` during implementation, unless the final merge-cleanup commit intentionally removed it after completion.
- `AGENTS.md` begins with `## Core Workflow`, says this is a Rust CLI for `.vcd`/`.fst` waveform inspection, says development tasks use `just`, tells agents not to delete arbitrary `tmp/` files, preserves a concise binary-waveform safety sentence, and contains a concise `## Map`.
- Non-root `AGENTS.md` files no longer contain parent/child breadcrumb maps and no longer point at old `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, `docs/ARCHITECTURE.md`, `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/exec-plans/`, or `scripts/AGENTS.md` paths.
- `tools/` exists with grouped helper directories and short READMEs; tracked `scripts/` files are gone.
- `justfile`, `.github/workflows/release.yml`, `.pre-commit-config.yaml`, docs, and breadcrumbs use `tools/` helper paths.
- `tools/release/test_extract_release_notes.py` and `tools/coverage/test_check_coverage.py` pass.
- `WAVEPEEK_IN_CONTAINER=1 just test-aux`, `WAVEPEEK_IN_CONTAINER=1 just check`, and `WAVEPEEK_IN_CONTAINER=1 just ci` pass, or any environment-caused inability to run the full gate is recorded with exact evidence.
- Read-only review iterations were run for docs, breadcrumbs, automation, and final control, and substantive findings were fixed or explicitly recorded as non-issues.

Expected successful targeted transcripts should look like this in shape, not exact timing:

    $ python3 -B -m unittest tools/release/test_extract_release_notes.py
    ....
    ----------------------------------------------------------------------
    Ran 4 tests in 0.0s

    OK

    $ python3 -B -m unittest tools/coverage/test_check_coverage.py
    ........
    ----------------------------------------------------------------------
    Ran 8 tests in 0.0s

    OK

    $ WAVEPEEK_IN_CONTAINER=1 just check
    # command output from rustfmt, clippy, schema check, actionlint, cargo check, and commit message check
    # exits 0

    $ WAVEPEEK_IN_CONTAINER=1 just ci
    # command output from format, lint, schema, actionlint, auxiliary tests, coverage gate, and cargo check
    # exits 0

## Idempotence and Recovery

Most steps are file moves and doc edits and can be retried safely with `git status --short` as the guide. If a `git mv` fails because a file was already moved, inspect `git status --short` and continue from the actual path. If a directory removal fails because untracked files exist, inspect them before deleting. Never use broad cleanup commands such as `rm -rf tmp/*`, `git clean -fdx`, or `rm -rf docs/tracker/wip/*` during this refactor unless the user explicitly asks and you have recorded why.

If a milestone commit fails because hooks detect a problem, fix the problem, rerun the targeted validation for that milestone, update this plan, and retry the commit. Do not bypass hooks. If `just check-commit` fails before the first commit because `.git/COMMIT_EDITMSG` is missing or stale, create the commit normally and let the commit-msg hook validate the actual message; record the behavior if it affects validation.

If a review finds a substantive issue after a milestone commit, make a follow-up fix commit with a narrow message such as `docs: fix tracker references after review` or amend the milestone commit only if the branch policy explicitly prefers amendments. Because the user requested commits and autonomous execution, default to additional clear commits rather than stopping to ask.

If full `just ci` cannot run because the environment lacks fixtures or container support, run the largest meaningful subset, record exact commands and errors in `Surprises & Discoveries`, and make sure the final response says the full gate was not completed. Do not claim success from partial validation.

## Artifacts and Notes

The source request in `tmp/unslop.md` is intentionally not required at implementation time because this plan embeds the requirements. The essential requirements are:

- simplify root `AGENTS.md`, put core workflow first, remove duplicate/breadcrumb-policy/devcontainer/FST safety sections, add a concise map, and mention this is a Rust CLI for `.vcd`/`.fst` waveform analysis;
- refactor all non-root `AGENTS.md` files by removing parent/child map lists and reducing duplicated docs;
- split `docs/DEVELOPMENT.md` into `docs/dev/quality.md`, `testing.md`, `environment.md`, `style.md`, `benchmarking.md`, `changelog.md`, `git.md`, and `automation.md`;
- move `docs/RELEASE.md` to `docs/dev/release.md` and `docs/ARCHITECTURE.md` to `docs/dev/architecture.md`;
- update the root README development section with a map of `docs/dev/`;
- create `docs/tracker/`, move backlog and roadmap there, create `docs/tracker/wip/AGENTS.md`, and delete current `docs/exec-plans/` after moving this plan;
- create `CONTRIBUTING.md` with the requested issue-first, maintainer-discussion, and no-neuroslop-spam policy;
- reorganize `scripts/` into grouped `tools/` directories with READMEs and tests where applicable;
- add the small fixes: `src/AGENTS.md` must mention consistency with architecture docs, root `tmp/` must not be globally deleted, and stale schema-hardening backlog entry must be removed.

Use `CHANGELOG.md` only for current Unreleased notes. Do not rewrite old release history to hide old `make` or schema references. History is allowed to be historical. Annoying, but that's the point of history.

## Interfaces and Dependencies

No Rust public API, CLI flag, JSON schema, or crate dependency should change. The interfaces changed by this refactor are repository paths and contributor commands.

The final path contracts are:

    docs/dev/quality.md
    docs/dev/testing.md
    docs/dev/environment.md
    docs/dev/style.md
    docs/dev/benchmarking.md
    docs/dev/changelog.md
    docs/dev/release.md
    docs/dev/architecture.md
    docs/dev/git.md
    docs/dev/automation.md
    docs/tracker/backlog.md
    docs/tracker/roadmap.md
    docs/tracker/wip/AGENTS.md
    tools/AGENTS.md
    tools/coverage/README.md
    tools/coverage/check_coverage.py
    tools/coverage/test_check_coverage.py
    tools/schema/README.md
    tools/schema/check_schema_contract.py
    tools/release/README.md
    tools/release/extract_release_notes.py
    tools/release/test_extract_release_notes.py
    tools/codex/README.md
    tools/codex/codex_env_common.sh
    tools/codex/codex_setup.sh
    tools/codex/codex_resume.sh
    tools/repo/README.md
    tools/repo/repo_stats.py

The root `justfile` remains the task interface. At completion these recipes must still work with the same names: `dev-setup`, `codex-setup`, `codex-resume`, `format`, `format-check`, `lint`, `lint-fix`, `check-build`, `test`, `coverage-src`, `coverage-src-check`, `test-aux`, `check-schema`, `check-actions`, `pre-commit`, `check-commit`, `check`, `ci`, `fix`, and `clean`. There is intentionally no final `tools/agent/` interface; `scripts/opencode_loop.py` is removed as obsolete helper code.

The release workflow interface remains `.github/workflows/release.yml`; it should call `python3 -B tools/release/extract_release_notes.py --version "$version"`. The schema checker interface remains `just check-schema`; internally it should call `tools/schema/check_schema_contract.py`. The source coverage checker interface remains `just coverage-src` and `just coverage-src-check`; internally they should call `tools/coverage/check_coverage.py`.

The review interface is read-only subagents when available. For each review prompt, provide the branch name, commit range or working tree diff, this plan path, tests run, and a focused lane. Require output in this form:

    - [severity: critical|high|medium|low] path:line — issue; impact; suggested fix

If a reviewer has no substantive findings, it should say `No substantive findings.` Fix findings in the main session, not by letting reviewers edit files, unless the user explicitly changes that policy.
