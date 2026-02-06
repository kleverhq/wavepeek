# M1 Project Init Delivery Plan (v0.1.0)

## Summary
- This plan delivers PRD M1 as three implementation tasks: (1) Rust scaffold, (2) local workflow alignment + pre-merge CI, (3) release automation + release runbook.
- It incorporates your constraints: one pre-merge CI workflow for `master` updates, one semver-tag release workflow, manual human CI validation feedback, and a dry-run mode with zero release side effects.
- The result is a minimal but production-oriented baseline for M2+: compile-ready project skeleton, deterministic checks, and an auditable release process in memory bank docs.

## Goals
- Ship a compilable Rust CLI skeleton with PRD module boundaries and `clap` help-only command surface.
- Keep local checks (`make` + pre-commit) in parity with CI.
- Add exactly two GitHub workflows: pre-merge CI and release CI.
- Ensure release CI supports dry-run debugging via GitHub repo variable, while still executing the full pipeline without publishing side effects.
- Document the release procedure in `.memory-bank/RELEASE.md` with a concrete, step-by-step checklist.

## Non-Goals
- Implementing business logic for `info`, `tree`, `signals`, `at`, `changes`, `when`, or `schema`.
- Implementing waveform parsing/expression evaluation behavior beyond placeholders.
- Adding multi-OS CI matrix in M1 (Linux/devcontainer baseline only).
- Adding post-M1 features (benchmarks, MCP, advanced release hardening).

## Background / Context
- PRD M1 requires project initialization, CLI skeleton, Make/pre-commit hygiene, CI pipeline, and release pipeline.
- The repository already has `Makefile` quality targets and `.pre-commit-config.yaml` hooks.
- The repository still lacks Rust crate/module scaffold and GitHub workflows.
- User feedback for this plan adds operational constraints around manual CI validation and release dry-run controls.

## Problem Statement
- Without scaffold + CI/release automation, the project cannot reliably enforce quality gates or produce reproducible releases.
- Without a dry-run gate, release pipeline debugging risks accidental side effects (crate publish / GitHub Release creation).
- Without a written release checklist, release operations remain tribal and error-prone.

## Requirements
- **Functional:**
- `R1`: Initialize Rust binary crate and dependencies per PRD 5.4.
- `R2`: Create PRD-aligned module skeleton (`cli`, `engine`, `expr`, `waveform`, `output`, `error`) with compile-safe placeholders.
- `R3`: Implement `clap` help-only CLI skeleton for 7 subcommands (`schema`, `info`, `tree`, `signals`, `at`, `changes`, `when`) and PRD argument conventions (`--waves` rules, no positional args).
- `R4`: Keep local quality gates and pre-commit hooks executable and green with the new scaffold.
- `R5`: Add pre-merge CI workflow that runs all checks, triggers on pull requests to `master`, and triggers on direct pushes to `master`.
- `R6`: Run pre-merge CI on one OS in the development container environment, with explicit runtime provenance tied to `.devcontainer` (Dockerfile or pinned equivalent image).
- `R7`: Add release workflow triggered by semver tags matching `v*.*.*`.
- `R8`: Add release dry-run gate controlled by GitHub repo variable (for example `RELEASE_DRY_RUN=1`) so pipeline fully executes but skips side effects (crate publish + GitHub Release creation), and require explicit verification that no side effects occurred.
- `R9`: Add `.memory-bank/RELEASE.md` with an explicit release checklist (version bump, commit, tag, push, verification, rollback notes).
- `R10`: Capture manual human validation feedback for CI and release dry-run runs.
- **Non-functional / constraints:**
- `R11`: Keep automation simple and deterministic; avoid introducing unnecessary abstraction in M1 workflows.
- `R12`: Preserve local/CI command parity using existing Make targets wherever possible.

## Traceability Matrix
| Scope item | Requirement IDs | Tasks | DoD checks | Primary artifacts |
|---|---|---|---|---|
| Rust scaffold + module placeholders + CLI help surface | R1, R2, R3 | Task 1 | D1, D2, D3, D4 | `Cargo.toml`, `src/` tree, help outputs |
| Local workflow alignment + pre-merge CI on master | R4, R5, R6, R11, R12 | Task 2 | D5, D6, D7, D12 | `Make` logs, `.github/workflows/ci.yml`, CI run logs + runtime proof |
| Semver release workflow + dry-run gate + runbook + human feedback | R7, R8, R9, R10, R11 | Task 3 | D8, D9, D10, D11, D13 | `.github/workflows/release.yml`, `.memory-bank/RELEASE.md`, release run logs |

## Proposed Solution
- Task 1: bootstrap Rust crate and PRD module skeleton, then wire `clap` command/help contracts only.
- Task 2: align local checks and implement one pre-merge workflow that runs format/lint/test/build on `master` PRs and direct pushes.
- Task 3: implement semver-tag release workflow with dry-run guard from GitHub variable; dry-run runs all build/package logic but hard-skips publish/release side effects.
- Use one Linux baseline in CI via the dev container environment to minimize drift between local development and CI runtime.
- Add `.memory-bank/RELEASE.md` as operational runbook so release operations are reproducible and reviewable.

## Alternatives Considered
- **Keep eight granular tasks:** rejected to align with your preference for three top-level tasks.
- **Enable multi-OS CI now:** rejected for M1 to keep runtime and complexity low.
- **No dry-run mode in release workflow:** rejected because it makes release debugging risky.

## Risks and Mitigations
- **Risk:** dry-run gate misconfiguration causes accidental side effects. **Mitigation:** make publish/release steps explicitly conditional on `RELEASE_DRY_RUN != 1` and verify in dry-run logs.
- **Risk:** `master` naming mismatch with current default branch settings. **Mitigation:** target `master` as requested and document branch expectation in release/CI notes.
- **Risk:** devcontainer and CI environment drift. **Mitigation:** run CI job in devcontainer-compatible environment and keep toolchain setup minimal.
- **Risk:** manual validation feedback is delayed. **Mitigation:** make feedback capture a DoD criterion and block closure until recorded.

## Rollout / Migration Plan
- Phase 1: complete Task 1 (Rust scaffold + CLI skeleton).
- Phase 2: complete Task 2 (local alignment + pre-merge CI), then request manual CI validation.
- Phase 3: complete Task 3 (release workflow + dry-run + runbook), then request manual release dry-run validation.
- Rollback: if release workflow is unstable, keep workflow file but force `RELEASE_DRY_RUN=1` until fixes land.

## Observability
- Pre-merge CI: at least one green run on PR to `master` and one green run on direct push to `master`.
- Release CI: at least one green semver-tag dry-run with explicit log lines showing side effects were skipped.
- Runtime provenance: CI logs include the exact CI container source used for the green runs.
- Side-effect audit: dry-run logs include before/after release state checks and publish-step status.
- Human validation: feedback is reviewed after CI and dry-run release runs.

## Open Questions
- None blocking for M1.

## Assumptions
- Target integration branch for M1 CI is `master` per your requirement.
- Dry-run control is a GitHub repository-level variable named `RELEASE_DRY_RUN` (`1` = dry-run, `0`/unset = normal mode).
- Release side effects to guard are crate publish and GitHub Release creation/publication.
- CI scope for M1 is one Linux OS using the devcontainer baseline.

## Definition of Done
- [ ] `D1`: `Cargo.toml` exists with PRD-required dependencies (`clap`, `serde`, `serde_json`, `regex`, `thiserror`, `wellen`) and test deps (`assert_cmd`, `predicates`, `tempfile`).
- [ ] `D2`: `src/` contains `main.rs`, `cli/`, `engine/`, `expr/`, `waveform/`, `output.rs`, `error.rs`; `cargo check` exits `0`.
- [ ] `D3`: `cargo run -- --help` lists exactly 7 subcommands (`schema`, `info`, `tree`, `signals`, `at`, `changes`, `when`).
- [ ] `D4`: CLI contract checks pass: waveform commands require `--waves`, `schema` has no `--waves`, positional args are rejected.
- [ ] `D5`: `make format-check && make lint && make test && make check-build` exits `0`.
- [ ] `D6`: `make pre-commit` exits `0` on scaffold branch.
- [ ] `D7`: `.github/workflows/ci.yml` exists and triggers on `pull_request` to `master` and `push` to `master`; one run for each trigger is green.
- [ ] `D8`: `.github/workflows/release.yml` exists with tag trigger `v*.*.*`.
- [ ] `D9`: With `RELEASE_DRY_RUN=1`, release workflow run is green, logs show publish/release side effects were skipped, and no new GitHub Release is created for the dry-run tag.
- [ ] `D10`: `.memory-bank/RELEASE.md` exists with checklist covering version bump, commit, tag, push, dry-run/real mode, verification, and rollback notes.
- [ ] `D11`: Manual human validation feedback for CI and release dry-run is provided by reviewer.
- [ ] `D12`: CI logs include explicit runtime provenance showing the job environment is derived from `.devcontainer` baseline.
- [ ] `D13`: Dry-run logs confirm publish step was not executed (no registry upload action invoked).

## Implementation Plan (Task Breakdown)

### Task 1: Rust scaffold and CLI skeleton (~4h)
- Goal: Deliver compile-ready Rust project structure and help-only CLI contract.
- Inputs: `.memory-bank/PRD.md` (M1, 3.2, 5.3, 5.4), existing repository root.
- Known-unknowns: Exact crate patch versions resolved during implementation.
- Steps:
1. Initialize binary crate metadata and add PRD dependencies/dev-dependencies.
2. Create PRD-aligned module tree with compile-safe placeholder stubs.
3. Implement `clap` skeleton with 7 subcommands and PRD flag conventions.
4. Validate compile/help behavior (`cargo check`, `cargo run -- --help`, per-command help, positional rejection case).
- Outputs: Compiling Rust scaffold and validated CLI help contract.

### Task 2: Local workflow alignment and pre-merge CI (~4h)
- Goal: Ensure local and CI checks are aligned and enforceable on `master`.
- Inputs: `Makefile`, `.pre-commit-config.yaml`, devcontainer baseline, Task 1 outputs.
- Known-unknowns: Exact GitHub Actions syntax for devcontainer execution approach.
- Steps:
1. Validate and adjust local quality gates only where needed to preserve Make-first workflow.
2. Create `.github/workflows/ci.yml` with checks (format/lint/test/build).
3. Configure triggers for `pull_request` to `master` and direct `push` to `master`.
4. Run CI in one Linux/devcontainer baseline and capture runtime provenance logs for human review.
- Outputs: Green local checks, green pre-merge CI workflow, manual CI feedback request.

### Task 3: Release workflow, dry-run gate, and runbook (~4h)
- Goal: Make release process testable, debuggable, and documented without accidental side effects.
- Inputs: Task 2 CI baseline, GitHub Actions permissions model, memory bank docs.
- Known-unknowns: Availability of publish credentials in target repository settings.
- Steps:
1. Create `.github/workflows/release.yml` triggered by tags `v*.*.*`.
2. Implement dry-run gate via repo variable `RELEASE_DRY_RUN` so pipeline runs fully while skipping crate publish and GitHub Release creation when enabled.
3. Add `.memory-bank/RELEASE.md` with concrete release checklist (bump, commit, tag, push, verify, rollback).
4. Execute one manual human-verified dry-run cycle, including explicit no-side-effect checks (no release created, no publish invocation), then incorporate feedback.
- Outputs: Release automation with safe debug mode, release runbook, and captured human validation feedback.
