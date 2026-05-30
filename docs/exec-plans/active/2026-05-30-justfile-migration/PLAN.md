# Migrate root automation from Makefile to justfile

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, contributors run repository automation through a root `justfile` instead of the current root `Makefile`. The observable result is simple: `just --list` shows the supported development recipes, `just check` runs the local pre-handoff gate, `just ci` runs the test-inclusive CI-parity gate, and devcontainer, CI, release, pre-commit, Codex, and maintainer docs all name `just` consistently.

This matters because `just` is a command runner, not a build graph tool. The current `Makefile` is already being used as a command runner; moving it to `justfile` removes Make-specific escaping, keeps shell recipes explicit, and avoids preserving a second automation surface that can drift. The migration must preserve behavior of the existing gates before it removes the root `Makefile`.

## Non-Goals

This plan does not change `wavepeek` CLI behavior, JSON output, schema shape, benchmark semantics, fixture versions, Rust dependencies, or release packaging. It does not refresh benchmark baselines. It does not introduce Cargo aliases or a new task runner beyond `just`. It does not keep a long-term compatibility `Makefile`; a temporary copy may exist only during local migration and must be gone before completion.

## Progress

- [x] (2026-05-30T11:33Z) Reviewed the current root `Makefile`, devcontainer config, GitHub workflows, pre-commit hooks, Codex setup scripts, developer docs, breadcrumb files, schema checker hint, and architecture docs for Make-specific contracts.
- [x] (2026-05-30T11:33Z) Confirmed the branch already contains a staged `.devcontainer/Dockerfile` change that installs the `just` package next to `make`, and confirmed the rebuilt container exposes `just 1.21.0`.
- [x] (2026-05-30T11:33Z) Created this active execution plan for the migration.
- [x] (2026-05-30T11:48Z) Reviewed this plan through automation, docs, and architecture lanes; addressed findings about `just` formatter instability, pre-commit filename passing, recipe list completeness, output echoing, and clean-checkout `tmp/` creation.
- [x] (2026-05-30T11:56Z) Ran a final control review pass and clarified that stale-reference sweeps target workflow/docs references, not the `.devcontainer/Dockerfile` package name `make`.
- [x] (2026-05-30T12:03Z) Extended the plan so root `justfile` formatting becomes a permanent guard in `format-check`, `check`, `ci`, and pre-commit rather than a one-time migration validation.
- [x] (2026-05-30T12:07Z) Ran one targeted review of the permanent justfile formatting guard addition; reviewer returned no substantive findings.
- [ ] Implement the root `justfile` while the existing `Makefile` is still available for comparison.
- [ ] Rewire devcontainer, GitHub Actions, release automation, pre-commit hooks, Codex setup, scripts, docs, breadcrumbs, and changelog entries to the `just` workflow.
- [ ] Delete the root `Makefile` after `just` parity is proven.
- [ ] Run the validation commands, complete implementation review, fix findings, and commit the migration.

## Surprises & Discoveries

- Observation: GitHub Actions currently reads `COVERAGE_SRC_THRESHOLD` from `Makefile` on the host runner before entering the devcontainer.
  Evidence: `.github/workflows/ci.yml` has a `Load coverage threshold from Makefile` step that runs `make -s print-coverage-src-threshold` outside the container.

- Observation: Removing `Makefile` creates a Codex bootstrap problem if the documented first setup command becomes `just codex-setup` before `just` is installed.
  Evidence: `docs/DEVELOPMENT.md` currently tells Codex environments to run `make codex-setup`, and `scripts/codex_setup.sh` is the actual shell script that can run before a task runner exists.

- Observation: The current development container already has `just` available.
  Evidence: `just --version` returned `just 1.21.0` after the staged `.devcontainer/Dockerfile` change added `just` to the apt package list.

- Observation: `just 1.21.0` treats formatter commands as unstable.
  Evidence: `just --fmt --check --justfile tmp/comment-test.just` exited with `error: The --fmt command is currently unstable`; `just --unstable --fmt --check --justfile tmp/comment-test.just` invoked the formatter check.

- Observation: Pre-commit passes matched filenames to hooks unless `pass_filenames: false` is set.
  Evidence: existing Rust hooks use `entry: make format`, `entry: make lint`, and `entry: make check-build` without `pass_filenames: false`; `just format src/lib.rs` would treat `src/lib.rs` as another recipe instead of ignoring it like Make effectively does for existing file paths.

- Observation: `just` echoes recipe lines by default.
  Evidence: output-sensitive recipes such as threshold printing need `@` prefixes, otherwise exact-output checks may see the shell command before the intended payload.

- Observation: A broad stale-reference sweep over `.devcontainer/` can hit the `make` package name in `.devcontainer/Dockerfile` even after workflow migration succeeds.
  Evidence: the intended Dockerfile package line keeps `make` and adds `just`; that package dependency is not the root automation interface and should not be treated as a stale `make <target>` instruction.

- Observation: The first reviewed plan made `just --unstable --fmt --check` a migration acceptance command but did not explicitly keep that check in the future automation gates.
  Evidence: `format-check`, `check`, `ci`, and the pre-commit hook plan did not yet name a permanent justfile formatting recipe, so a later edit could let the root `justfile` drift after migration. Tiny gap, classic place for entropy to set up a folding chair.

## Decision Log

- Decision: Use a lower-case root file named `justfile`.
  Rationale: `just` discovers `justfile` by default, the user requested a justfile explicitly, and lower-case avoids implying a generated or platform-specific file.
  Date/Author: 2026-05-30 / Grin

- Decision: Preserve every existing callable Make target name as a callable `just` recipe: `print-coverage-src-threshold`, `require-container`, `check-rtl-artifacts`, `update-schema`, `check-schema`, `check-actions`, `dev-setup`, `codex-setup`, `codex-resume`, `format`, `format-check`, `lint`, `lint-fix`, `check-build`, `test`, `coverage-src-data`, `coverage-src`, `coverage-src-check`, `test-aux`, `build-release`, `bench-e2e-update-baseline`, `bench-e2e-run`, `bench-expr-update-baseline`, `bench-expr-run`, `bench-e2e-smoke-commit`, `pre-commit`, `check-commit`, `check`, `ci`, `fix`, `clean`, and `help`; add new `format-justfile` and `format-justfile-check` recipes for the new automation file.
  Rationale: The migration should change the command runner, not force contributors or automation to learn renamed gates at the same time. Pure helper recipes may be hidden from `just --list`, but their names should still work when invoked directly. The new `justfile` also needs first-class formatting commands because it replaces the file that previously defined formatting gates.
  Date/Author: 2026-05-30 / Grin

- Decision: Hide helper and plumbing recipes with `just`'s `[private]` attribute while keeping their current names, and keep `help` as `just --list` rather than hand-maintaining an `awk` formatter.
  Rationale: `just --list` is the native discovery surface. Hiding helper recipes keeps the listed surface close to the old `make help` output without keeping Make's comment-parsing machinery. Yes, the old `awk` incantation can finally go back to whatever swamp spawned it.
  Date/Author: 2026-05-30 / Grin

- Decision: Do not keep a compatibility `Makefile` in the final state.
  Rationale: Keeping both command surfaces would make every future automation change a two-file drift risk. The branch should fail fast on stale `make` references instead of silently preserving old habits.
  Date/Author: 2026-05-30 / Grin

- Decision: Codex first-time setup should be documented as `bash scripts/codex_setup.sh`, and that script should install or verify `just`; after setup, normal maintenance can use `just codex-resume`.
  Rationale: A task runner cannot bootstrap itself before it exists. Direct shell script entry is already present and is the smallest safe bootstrap path.
  Date/Author: 2026-05-30 / Grin

- Decision: Remove the host-side CI step that shells out to `make -s print-coverage-src-threshold`; let `just ci` use the same default threshold inside the devcontainer unless `COVERAGE_SRC_THRESHOLD` is explicitly set by the workflow environment.
  Rationale: Requiring `just` on the GitHub host runner only to read one default would be silly machinery. The actual coverage gate runs inside the container and can own the default there.
  Date/Author: 2026-05-30 / Grin

- Decision: Use `just --unstable --fmt` and `just --unstable --fmt --check` for formatting validation while the container ships `just 1.21.0`.
  Rationale: Formatter support exists but is gated as unstable in this installed version. The validation command should match the actual tool, not a nicer future we wish existed.
  Date/Author: 2026-05-30 / Grin

- Decision: Add `pass_filenames: false` to Rust pre-commit hooks when changing their entries to `just format`, `just lint`, and `just check-build`.
  Rationale: Pre-commit appends filenames by default. Make tolerates existing file path goals well enough for the current setup, but `just` treats extra command-line words as recipe names and would fail on filenames.
  Date/Author: 2026-05-30 / Grin

- Decision: Prefix output-sensitive `just` recipe lines with `@`, especially `print-coverage-src-threshold`, `help`, and guard/check helper commands.
  Rationale: Unlike the existing Makefile lines that already use `@` selectively, `just` echoes recipe lines by default. Exact-output plumbing should print the payload, not the command that produced it.
  Date/Author: 2026-05-30 / Grin

- Decision: Scope stale-reference checks to workflow, docs, scripts, and hook entrypoints, and allow the literal package name `make` in `.devcontainer/Dockerfile` unless it is attached to a root automation command.
  Rationale: The image may still provide GNU Make for third-party tooling or developer convenience. The migration removes the root `Makefile` interface; it does not require purging the package named `make` from the container.
  Date/Author: 2026-05-30 / Grin

- Decision: Make root `justfile` formatting a permanent gate by adding `format-justfile` and `format-justfile-check`, making `format-check` depend on the justfile check, and adding a dedicated pre-commit hook for `justfile`.
  Rationale: A command runner file is code. If it is not checked in the same automation paths as Rust formatting, it will drift exactly when nobody is looking, because apparently whitespace has ambitions.
  Date/Author: 2026-05-30 / Grin

## Outcomes & Retrospective

The migration is not implemented yet. At plan handoff, the only implementation-adjacent change observed is the staged devcontainer package addition for `just`. The intended completed state is a repository with one root automation entrypoint, `justfile`, no live automation or maintainer documentation that tells contributors to use root `make` targets, and permanent justfile formatting coverage in `format-check`, `check`, `ci`, and pre-commit.

## Context and Orientation

The repository currently uses a root `Makefile` as a command runner. A command runner is a file of named shell commands such as `check` and `ci`; unlike a real build graph, these commands usually run every time and are meant for humans and CI. The current `Makefile` exports `WAVEPEEK_RTL_ARTIFACTS_DIR`, guards most targets with `WAVEPEEK_IN_CONTAINER=1`, checks external RTL waveform fixtures, runs formatting, linting, schema validation, tests, coverage, benchmark helpers, pre-commit hooks, release builds, and cleanup.

The current root `Makefile` is referenced by several repository surfaces. `.devcontainer/devcontainer.json` runs `make dev-setup` after container start. `.github/workflows/ci.yml` runs `make -s print-coverage-src-threshold` on the host and `make ci` inside the devcontainer. `.github/workflows/release.yml` runs `make ci` inside the devcontainer. `.pre-commit-config.yaml` invokes `make` targets for hooks. `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, root `AGENTS.md`, `.devcontainer/AGENTS.md`, `scripts/AGENTS.md`, `schema/AGENTS.md`, `docs/ARCHITECTURE.md`, `scripts/check_schema_contract.py`, and the changelog currently name Make or Makefile in live workflow text.

`just` is the replacement command runner. A named command in `just` is called a recipe. A `justfile` recipe can depend on another recipe, can run shell commands, can use variables, and can be listed with `just --list`. The implementation should use `bash` with strict flags so shell failures are visible: `bash -euo pipefail -c` means exit on the first failing command, reject unset shell variables, and fail a pipeline when any command in it fails.

The repository is container-first. Recipes that build, test, run hooks, touch fixtures, or rely on container-installed tools must keep the current guard: if `WAVEPEEK_IN_CONTAINER` is not `1`, print `error: container: this target must run inside a wavepeek-managed container environment (set WAVEPEEK_IN_CONTAINER=1)` to stderr and exit non-zero. `print-coverage-src-threshold`, if kept for diagnostics, must remain callable outside the container because the old workflow used it before the devcontainer existed.

Treat all `.fst` files as binary files. Do not read them as text while validating this plan or the implementation.

## Open Questions

There are no unresolved design questions at handoff. If a reviewer or maintainer asks for a temporary compatibility `Makefile`, revise this plan first because that changes the final-state contract and drift risk.

## Plan of Work

Milestone 1 creates `justfile` while preserving the current `Makefile` for comparison. Add a root `justfile` and port the current variables and targets recipe-for-recipe. Use `set shell := ["bash", "-euo", "pipefail", "-c"]` near the top. Define a `default` recipe that depends on `help`, and define `help` as `just --list`. Keep the existing environment contract: resolve `RTL_ARTIFACTS_DIR` with `.devcontainer/resolve_rtl_artifacts_dir.sh`, export `WAVEPEEK_RTL_ARTIFACTS_DIR`, use `schema/wavepeek.json`, keep the benchmark run directories, keep `WAVEPEEK_RELEASE_BIN=./target/release/wavepeek`, keep `PYTHON=python3 -B`, and keep `COVERAGE_SRC_THRESHOLD` defaulting to `90` unless the environment overrides it.

In the first `justfile` draft, preserve every behavior from the current `Makefile`. The private container guard recipe named `require-container` should be a dependency of all recipes that currently depend on `require-container`. The private RTL-fixture check recipe named `check-rtl-artifacts` should source `.devcontainer/env_contract.sh` and verify every file named by `WAVEPEEK_RTL_ARTIFACT_FILES` exists under the resolved RTL artifacts directory. Keep `print-coverage-src-threshold`, `require-container`, `check-rtl-artifacts`, and `coverage-src-data` hidden from `just --list` with `[private]`, but still callable by name.

The recipe bodies should stay mechanically close to the current `Makefile` bodies, with one deliberate addition for the new automation file. Add `format-justfile`, which runs `just --unstable --fmt`, and `format-justfile-check`, which runs `just --unstable --fmt --check`. Make `format` run Rust formatting and `format-justfile`, and make `format-check` run Rust formatting checks and `format-justfile-check`. Because `check` and `ci` already depend on `format-check`, this makes root `justfile` formatting part of both local and CI quality gates. `update-schema` still writes runtime schema output to a temporary file and atomically moves it to `schema/wavepeek.json`. `check-schema` still runs `python3 -B scripts/check_schema_contract.py schema/wavepeek.json`. `check-actions` still runs `actionlint .github/workflows/*.yml`. `lint` and `lint-fix` still run the same clippy commands. `test` and benchmark recipes still depend on the RTL fixture check. `check` should still be the local non-test gate: format check, clippy, schema check, actions lint, cargo check, and commit-message check. `ci` should still be the test-inclusive gate: format check, clippy, schema check, actions lint, auxiliary Python tests, source coverage check, and cargo check.

A representative top of file should look like this; adjust only if `just --unstable --fmt --check` or `just --list` proves the syntax needs a smaller change for `just 1.21.0`:

    set shell := ["bash", "-euo", "pipefail", "-c"]

    default_rtl_artifacts_dir := `./.devcontainer/resolve_rtl_artifacts_dir.sh`
    rtl_artifacts_dir := env_var_or_default("RTL_ARTIFACTS_DIR", default_rtl_artifacts_dir)
    export WAVEPEEK_RTL_ARTIFACTS_DIR := rtl_artifacts_dir
    schema_path := "schema/wavepeek.json"
    bench_e2e_runs_dir := "bench/e2e/runs"
    bench_e2e_baseline_dir := bench_e2e_runs_dir / "baseline"
    bench_expr_runs_dir := "bench/expr/runs"
    bench_expr_baseline_dir := bench_expr_runs_dir / "baseline"
    wavepeek_release_bin := "./target/release/wavepeek"
    codex_setup_script := "scripts/codex_setup.sh"
    codex_resume_script := "scripts/codex_resume.sh"
    python := "python3 -B"
    coverage_src_threshold := env_var_or_default("COVERAGE_SRC_THRESHOLD", "90")

    default: help

    [private]
    print-coverage-src-threshold:
        @printf '%s\n' "{{coverage_src_threshold}}"

    [private]
    require-container:
        @if [ "${WAVEPEEK_IN_CONTAINER:-0}" != "1" ]; then \
            printf '%s\n' "error: container: this target must run inside a wavepeek-managed container environment (set WAVEPEEK_IN_CONTAINER=1)" >&2; \
            exit 1; \
        fi

    # Format root justfile in place
    format-justfile: require-container
        @just --unstable --fmt

    # Check root justfile formatting
    format-justfile-check: require-container
        @just --unstable --fmt --check

    # Show recipes
    help: require-container
        @just --list

Milestone 1 is accepted when these commands run from `/workspaces/wavepeek` and prove the new file is parseable and behaviorally comparable while the old file still exists:

    just --unstable --fmt --check
    just --list
    just print-coverage-src-threshold
    WAVEPEEK_IN_CONTAINER=0 just format-check
    WAVEPEEK_IN_CONTAINER=1 just format-justfile-check
    WAVEPEEK_IN_CONTAINER=1 just check-schema

Expected observations: `just --unstable --fmt --check` exits `0`; `just --list` shows the public development recipes including `format-justfile` and `format-justfile-check`; `just print-coverage-src-threshold` prints only `90` unless the environment overrides it; the `WAVEPEEK_IN_CONTAINER=0` command fails before invoking Cargo and prints the same container error shape as the old Make target; `WAVEPEEK_IN_CONTAINER=1 just format-justfile-check` passes; `WAVEPEEK_IN_CONTAINER=1 just check-schema` passes in the rebuilt container.

Milestone 2 rewires automation entrypoints from `make` to `just`. Update `.devcontainer/devcontainer.json` so `postStartCommand` runs `just dev-setup`. Leave `.devcontainer/devcontainer.ci.json` unchanged unless implementation discovers it needs an explicit command; CI passes the command from GitHub Actions. Keep the staged `.devcontainer/Dockerfile` package addition for `just`, and verify both `dev` and `ci` Docker targets inherit it. In `.github/workflows/ci.yml`, delete the host-side `Load coverage threshold from Makefile` step and change both devcontainer `runCmd` values to `just ci`. In `.github/workflows/release.yml`, change the release quality gate `runCmd` to `just ci`. In `.pre-commit-config.yaml`, change every hook entry from `make <target>` to `just <target>`, add `pass_filenames: false` to the Rust `format`, `lint`, and `check-build` hooks so pre-commit does not append file paths that `just` would parse as extra recipe names, and add a dedicated `justfile-format-check` hook with `entry: just format-justfile-check`, `files: ^justfile$`, `language: system`, `pass_filenames: false`, and `stages: [pre-commit]`.

Also update the Codex projection. Add a pinned `WAVEPEEK_JUST_VERSION` to `.devcontainer/env_contract.sh` unless the implementation chooses a better existing version source and records that decision here. Add an `ensure_just` function to `scripts/codex_env_common.sh` that installs `just` when absent or at the wrong version, preferably with `cargo install --locked just --version "$WAVEPEEK_JUST_VERSION"` after the Rust toolchain is selected. Call it from `ensure_codex_tooling`. Update `scripts/codex_setup.sh` and `scripts/codex_resume.sh` log text from “make targets” to “just recipes”. The first setup command in docs must be the direct shell script because `just` may not exist yet.

Milestone 2 is accepted when these commands and inspections work:

    just --version
    WAVEPEEK_IN_CONTAINER=1 just dev-setup
    WAVEPEEK_IN_CONTAINER=1 just codex-resume
    rg -n "entry: make|runCmd: .*make|postStartCommand.*make|Load coverage threshold from Makefile" .github .devcontainer .pre-commit-config.yaml scripts

Expected observations: `just --version` reports at least `just 1.21.0`; `just dev-setup` installs hooks and prints tool versions in the container; `just codex-resume` runs the existing resume script; the `rg` command prints no live hits.

Milestone 3 updates live documentation, hints, and breadcrumbs. Replace live workflow references to root `make` targets with `just` recipes in root `AGENTS.md`, `.devcontainer/AGENTS.md`, `docs/DEVELOPMENT.md`, `docs/RELEASE.md`, `scripts/AGENTS.md`, `schema/AGENTS.md`, `docs/ARCHITECTURE.md`, and `scripts/check_schema_contract.py`. Update `README.md` only if implementation finds a live development command reference there. Update `CHANGELOG.md` under `## [Unreleased]`, preferably in `### Changed`, to state that repository automation moved from root `Makefile` targets to root `justfile` recipes. Do not rewrite historical released changelog entries or completed execution plans just because they mention Make; history is allowed to look historical, a mercy rarely extended to software documentation.

Milestone 3 must preserve source-of-truth boundaries. `docs/DEVELOPMENT.md` remains the canonical developer workflow. Root and nested `AGENTS.md` files remain navigation maps, not manuals. `docs/ARCHITECTURE.md` should change the build automation row from `Cargo + Make` to `Cargo + just`, or equivalent wording, without implying that Cargo stopped owning Rust compilation.

Milestone 3 is accepted when a stale-reference sweep has only intentional historical or migration-plan hits:

    rg -n "\bmake\b|Makefile" AGENTS.md .devcontainer/AGENTS.md .devcontainer/devcontainer.json .devcontainer/devcontainer.ci.json .devcontainer/env_contract.sh .devcontainer/initialize.sh .devcontainer/resolve_rtl_artifacts_dir.sh docs scripts schema .github .pre-commit-config.yaml README.md CHANGELOG.md

Expected observations: live docs and automation should not instruct contributors to run `make`. Allowed hits are this active plan, completed plans under `docs/exec-plans/completed/`, immutable released changelog sections, ordinary English false positives such as “make a decision,” and explicitly recorded migration notes. If a broader sweep includes `.devcontainer/Dockerfile`, the package name `make` is allowed as long as no workflow or documentation tells users to run root `make` targets. If an allowed hit is not obvious, record it in `Surprises & Discoveries` before handoff.

Milestone 4 removes the root `Makefile` and validates the new single-entrypoint workflow. Delete `Makefile` only after the `justfile` has passed the targeted checks above. After deletion, run discovery, formatting, local gate, test-inclusive gate, and pre-commit through `just` from `/workspaces/wavepeek` inside the devcontainer or CI image:

    just --unstable --fmt --check
    just --list
    WAVEPEEK_IN_CONTAINER=0 just check-build
    WAVEPEEK_IN_CONTAINER=1 just format-justfile-check
    WAVEPEEK_IN_CONTAINER=1 just format-check
    WAVEPEEK_IN_CONTAINER=1 just check
    WAVEPEEK_IN_CONTAINER=1 just ci
    WAVEPEEK_IN_CONTAINER=1 just pre-commit

Expected observations: the direct formatter command, recipe-based justfile formatting check, aggregate format check, and list commands pass; the explicit non-container invocation fails with the documented container error; `just check`, `just ci`, and `just pre-commit` pass in the rebuilt container. If `just check-commit` fails because there is no meaningful commit message file yet, create or update `.git/COMMIT_EDITMSG` with the intended conventional commit subject and rerun `just check-commit`; do not weaken the hook.

Milestone 5 closes the branch through review and commit discipline. Request focused review after implementation using the `ask-review` skill. Use at least three lanes: automation correctness for recipe parity and CI/pre-commit/devcontainer wiring, documentation for stale Make references and maintainer wording, and architecture/bootstrap for the no-shim decision plus Codex first-run behavior. Fix findings in the main session, rerun impacted validation, then run one fresh control pass on the consolidated diff. Commit with a conventional message such as `chore(dev): migrate automation to just`, and do not bypass hooks unless the user explicitly asks.

## Concrete Steps

Run all commands from `/workspaces/wavepeek`. The intended environment is the rebuilt devcontainer or the CI image, with `WAVEPEEK_IN_CONTAINER=1` available for normal gates.

1. Capture current Make behavior before deleting anything:

       mkdir -p tmp
       WAVEPEEK_IN_CONTAINER=1 make help > tmp/make-help.before.txt
       WAVEPEEK_IN_CONTAINER=1 make print-coverage-src-threshold > tmp/coverage-threshold.before.txt

   Expected result: the first file contains the current public target list, and the second contains `90` unless the environment overrides `COVERAGE_SRC_THRESHOLD`.

2. Add and format the root `justfile`, then run the Milestone 1 checks:

       just --unstable --fmt
       just --unstable --fmt --check
       just --list
       just print-coverage-src-threshold
       WAVEPEEK_IN_CONTAINER=0 just format-check
       WAVEPEEK_IN_CONTAINER=1 just format-justfile-check
       WAVEPEEK_IN_CONTAINER=1 just format-check
       WAVEPEEK_IN_CONTAINER=1 just check-schema

3. Rewire automation and docs according to Milestones 2 and 3. Use targeted greps after each cluster of edits rather than waiting for the end:

       rg -n "entry: make|runCmd: .*make|postStartCommand.*make|Load coverage threshold from Makefile" .github .devcontainer .pre-commit-config.yaml scripts
       rg -n "\bmake\b|Makefile" AGENTS.md .devcontainer/AGENTS.md .devcontainer/devcontainer.json .devcontainer/devcontainer.ci.json .devcontainer/env_contract.sh .devcontainer/initialize.sh .devcontainer/resolve_rtl_artifacts_dir.sh docs scripts schema .github .pre-commit-config.yaml README.md CHANGELOG.md

4. Delete `Makefile`, then run the full validation set:

       rm Makefile
       just --unstable --fmt --check
       just --list
       WAVEPEEK_IN_CONTAINER=0 just check-build
       WAVEPEEK_IN_CONTAINER=1 just format-justfile-check
       WAVEPEEK_IN_CONTAINER=1 just format-check
       WAVEPEEK_IN_CONTAINER=1 just check
       WAVEPEEK_IN_CONTAINER=1 just ci
       WAVEPEEK_IN_CONTAINER=1 just pre-commit

5. Review, fix, rerun impacted gates, and commit:

       git status --short
       git diff --stat
       git add justfile .devcontainer .github .pre-commit-config.yaml scripts docs AGENTS.md schema CHANGELOG.md Makefile
       git commit -m "chore(dev): migrate automation to just"

   If `git add Makefile` warns because the file is deleted, use `git add -A Makefile` or `git add -A` after inspecting `git status`. The final commit should include the staged `.devcontainer/Dockerfile` `just` package addition unless it was already committed separately.

## Validation and Acceptance

The migration is complete when a human can verify all of these behaviors from a clean checkout of the branch:

- `just --list` is the discoverable root automation surface and lists the expected public development recipes.
- `Makefile` is absent from the repository root.
- `.devcontainer/devcontainer.json`, GitHub Actions workflows, release workflow, and pre-commit hooks invoke `just`, not `make`.
- `bash scripts/codex_setup.sh` can bootstrap a Codex-style environment before `just` is assumed, and subsequent `just codex-resume` works.
- The non-container guard still fails fast with the existing `error: container:` message.
- `just format-justfile-check` verifies root `justfile` formatting, `just format-check` includes that verification, and therefore `just check` and `just ci` both fail if the root `justfile` is not formatted.
- `just pre-commit` includes a dedicated `justfile-format-check` hook that runs `just format-justfile-check` when `justfile` changes.
- `just check`, `just ci`, and `just pre-commit` pass in the devcontainer or CI image.
- `scripts/check_schema_contract.py` points users to `just update-schema` when schema freshness fails.
- Live docs and breadcrumb maps no longer tell contributors to use root `make` targets; any remaining Make references are historical, this plan, or ordinary English false positives.

## Idempotence and Recovery

The migration is safe to apply incrementally. Keep the old `Makefile` until the new `justfile` has passed targeted checks, then delete it only once CI/devcontainer/pre-commit wiring points at `just`. If a recipe fails after the deletion, recover by reading the committed or pre-delete `Makefile` from Git history with `git show HEAD:Makefile` or from `tmp/make-help.before.txt`, port the missing command, and rerun only the affected recipe before the full gates.

Do not hand-edit generated coverage artifacts or benchmark run outputs while validating this change. The `tmp/` directory is disposable and ignored; store comparison logs there. If Codex setup installation of `just` fails, record the exact error in `Surprises & Discoveries`, leave `bash scripts/codex_setup.sh` as the documented bootstrap path, and do not reintroduce a root `Makefile` without revising the decision log.

## Artifacts and Notes

The expected staged Dockerfile change before this plan was written is:

    -    ca-certificates curl git make libatomic1 \
    +    ca-certificates curl git make just libatomic1 \

The expected non-container guard remains:

    error: container: this target must run inside a wavepeek-managed container environment (set WAVEPEEK_IN_CONTAINER=1)

The final stale-reference sweep may still mention `make` in historical changelog entries and completed execution plans. Do not churn those records unless they are otherwise being edited for a real reason.

## Interfaces and Dependencies

The final root automation interface is `justfile`, consumed by `just`. The minimum version known to work at plan time is `just 1.21.0`, because that is what the rebuilt container exposes. The implementation may use `just` features available in that version, including `[private]` recipe attributes, `env_var_or_default`, variable interpolation with `{{...}}`, recipe dependencies, and `just --unstable --fmt --check`.

The final callable recipe names should match the old Make target names, with new public `format-justfile` and `format-justfile-check` recipes added for the new root automation file. Private helper recipes should include `print-coverage-src-threshold`, `require-container`, `check-rtl-artifacts`, and `coverage-src-data` under their existing names with `[private]`, unless implementation records a deliberate compatibility exception in this plan. The external tools and scripts remain the same: Cargo owns Rust compilation and tests, `cargo llvm-cov` owns source coverage data, `actionlint` checks GitHub Actions workflows, `pre-commit` runs local hooks, `commitizen` validates commit messages, `bench/e2e/perf.py` and `bench/expr/perf.py` own benchmark harness workflows, and `scripts/check_schema_contract.py` validates the canonical schema artifact.

Revision Note: 2026-05-30 / Grin - Initial plan created from the current Makefile, devcontainer, CI, pre-commit, Codex, docs, schema, and architecture wiring. The plan intentionally selects a no-shim final state, direct Codex shell bootstrap, and native `just --list` discovery before implementation begins.

Revision Note: 2026-05-30 / Grin - Review findings from automation, docs, and architecture lanes were incorporated. The plan now uses `just --unstable --fmt` for the installed `just 1.21.0`, requires pre-commit filename suppression for Rust hooks, preserves all existing Make target names as callable recipes, adds `@` to output-sensitive just recipes, and creates `tmp/` before redirecting comparison artifacts.

Revision Note: 2026-05-30 / Grin - A final control pass found that the stale-reference sweep could confuse the Dockerfile package name `make` with a stale automation command. The plan now narrows the sweep to workflow/docs/hook/script entrypoints and explicitly allows the package name when it is not presented as a root command runner.

Revision Note: 2026-05-30 / Grin - The plan now makes root `justfile` formatting a permanent guard, not just a migration-time check. It adds `format-justfile` and `format-justfile-check`, requires `format-check`, `check`, `ci`, and pre-commit coverage for that check, and adds explicit acceptance criteria for the dedicated hook. A focused review of this addition returned no substantive findings.
