# Consolidate Devcontainer Agent State Under `~/.config/wavepeek-dev`

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Note that this document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

A local `wavepeek` devcontainer currently prepares and mounts coding-agent state from several host home-directory locations, including `~/.claude`, `~/.codex`, `~/.pi`, `~/.cache/wavepeek`, and OpenCode-specific directories. After this change, a fresh host should only need one project-owned host directory, `~/.config/wavepeek-dev`, for wavepeek-managed devcontainer state. Existing user-level Claude, Codex, and Pi state can still be reused by placing symlinks inside that one directory, but the initialization script must stop creating new top-level dotfiles in the user's home directory.

The user-visible result is visible before a container starts: `.devcontainer/initialize.sh` creates `~/.config/wavepeek-dev/github.env`, `~/.config/wavepeek-dev/verdi`, `~/.config/wavepeek-dev/claude`, `~/.config/wavepeek-dev/claude.json`, `~/.config/wavepeek-dev/codex`, and `~/.config/wavepeek-dev/pi`. `.devcontainer/devcontainer.json` passes the GitHub env file from `~/.config/wavepeek-dev` through Docker `--env-file`, and bind-mounts only agent and Verdi state from that same directory. OpenCode is no longer installed, mounted, or recommended.

## Non-Goals

This plan does not remove or migrate a user's existing top-level `~/.claude`, `~/.claude.json`, `~/.codex`, or `~/.pi` paths. If those paths already exist, the new host state directory will point to them with symlinks so existing tools keep working. This plan does not mount host `~/.config/gh`; GitHub authentication remains env-file based. This plan does not change Rust code or waveform behavior.

## Progress

- [x] (2026-06-06 00:00Z) Loaded repository and subtree guidance for `.devcontainer/`, `docs/`, `docs/tracker/wip/`, and `tools/`.
- [x] (2026-06-06 00:03Z) Inspected current devcontainer config, host initialization script, Dockerfile agent installation block, GitHub-auth docs, FSDB docs, and helper scripts that reference old host paths.
- [x] (2026-06-06 00:08Z) Created this initial ExecPlan with the intended implementation and validation strategy.
- [x] (2026-06-06 00:18Z) Requested and incorporated read-only code/devcontainer-shell and docs/breadcrumbs reviews of this ExecPlan before implementation.
- [ ] Update devcontainer host-state setup and mounts.
- [ ] Remove OpenCode installation, mounts, extension, version constant, and stale references.
- [ ] Update docs and helper scripts to use `~/.config/wavepeek-dev`.
- [ ] Run targeted validation and repository quality gates.
- [ ] Request post-implementation review, apply any fixes, and run a final control review.
- [ ] Commit the completed change set.

## Surprises & Discoveries

- Observation: The current host initialization script creates top-level `~/.claude`, `~/.codex`, `~/.pi`, and `~/.claude.json` placeholders on every host before the devcontainer starts.
  Evidence: `.devcontainer/initialize.sh` has those paths in its `mkdir -p` list and writes `{}` to `$HOME/.claude.json` when missing.
- Observation: OpenCode is present in three independent surfaces: Docker image installation, host bind mounts, and VS Code extension recommendation.
  Evidence: `.devcontainer/Dockerfile` installs a pinned OpenCode release, `.devcontainer/devcontainer.json` mounts three `opencode` host paths, and the VS Code extensions list includes `sst-dev.opencode`.
- Observation: GitHub authentication is already env-file based and does not mount host `~/.config/gh`, but the path still uses `~/.config/wavepeek`.
  Evidence: `.devcontainer/devcontainer.json` passes `${localEnv:HOME}/.config/wavepeek/github.env` with `--env-file`, and `docs/dev/github-auth.md` documents that layout.
- Observation: Plan review found that a kind-agnostic legacy symlink helper would be unsafe if a user had a file where a directory was expected, or a directory where `claude.json` was expected.
  Evidence: Pre-implementation review reported that stray `~/.codex` or `~/.pi` files, or a `~/.claude.json` directory, could produce bad bind mounts or container startup failures.
- Observation: Plan review found that blindly recreating `~/.config/wavepeek-dev/verdi` could delete real contents if `VERDI_HOME` points to that path or if the path is a non-empty managed directory.
  Evidence: Pre-implementation review recommended unlinking symlinks only, guarding same-path cases, and warning rather than deleting non-empty directories.
- Observation: Plan review found that the existing maintainer GitHub setup helper's "directory must be empty" rule will not work after consolidation, because `initialize.sh` intentionally creates unrelated managed paths under `~/.config/wavepeek-dev`.
  Evidence: Pre-implementation docs review noted that `tools/repo/setup_github_env.sh` currently exits when the config directory exists and is non-empty.

## Decision Log

- Decision: Use `~/.config/wavepeek-dev` as the only wavepeek-managed host root for devcontainer bind sources and GitHub env files.
  Rationale: The user explicitly requested replacing both `~/.cache/wavepeek` and `~/.config/wavepeek` with a single dev-only config directory. Keeping all mount sources under one directory makes the invariant easy to audit.
  Date/Author: 2026-06-06 / Grin
- Decision: For existing top-level Claude, Codex, and Pi paths, create symlinks inside `~/.config/wavepeek-dev`; when they do not exist, create managed placeholders inside `~/.config/wavepeek-dev` instead of top-level home paths.
  Rationale: This preserves existing user state without newly dirtying `$HOME`. Static devcontainer bind mounts need a source path to exist before Docker starts; managed placeholders satisfy that need for fresh hosts.
  Date/Author: 2026-06-06 / Grin
- Decision: Treat `~/.config/wavepeek-dev/verdi` as a managed mount source without blind deletion: unlink an existing symlink, replace an empty directory when necessary, guard against `VERDI_HOME` resolving to the same path, and warn rather than delete non-empty real directories.
  Rationale: The old behavior intentionally made `/opt/verdi` either a usable host SDK or an empty directory meaning "Verdi unavailable", but moving the source under a durable config directory raises the cost of destructive cleanup. The machine spirits can have a broom, not a wood chipper.
  Date/Author: 2026-06-06 / Grin
- Decision: Remove OpenCode from the local dev image and VS Code recommendations, not merely stop mounting its state.
  Rationale: The user asked to fully remove OpenCode. Leaving a binary or extension recommendation would keep a half-removed agent surface, which is how configuration fossils breed.
  Date/Author: 2026-06-06 / Grin
- Decision: Make `tools/repo/setup_github_env.sh` validate only GitHub env-file conflicts inside `~/.config/wavepeek-dev`, not require the entire directory to be empty.
  Rationale: After consolidation, unrelated managed entries such as `claude`, `codex`, `pi`, and `verdi` are supposed to exist there. The helper should refuse unsafe GitHub file conflicts without treating a healthy shared state directory as contaminated.
  Date/Author: 2026-06-06 / Grin

## Outcomes & Retrospective

No implementation has been completed yet. The expected outcome is a devcontainer configuration where every host bind source for coding-agent and Verdi state begins with `${localEnv:HOME}/.config/wavepeek-dev/`, the GitHub env file is passed from `${localEnv:HOME}/.config/wavepeek-dev/github.env`, and no repository file still provisions OpenCode.

## Context and Orientation

The local development container is configured by `.devcontainer/devcontainer.json`. That JSON file describes environment variables, Docker run arguments, host-to-container bind mounts, and VS Code extensions for the interactive `wavepeek` development environment. A bind mount makes a host file or directory appear at a container path; Docker requires the host source to exist or it may create root-owned placeholders, so this repository runs `.devcontainer/initialize.sh` on the host before container creation.

The host initialization script `.devcontainer/initialize.sh` currently creates several host paths. It creates GitHub env files under `~/.config/wavepeek`, creates a Verdi mount source under `~/.cache/wavepeek/verdi`, and creates top-level agent paths such as `~/.claude`, `~/.codex`, and `~/.pi`. Verdi is the optional Synopsys waveform SDK used for FSDB development; inside the container it is always expected at `/opt/verdi` through the `VERDI_HOME=/opt/verdi` environment variable.

OpenCode is another coding agent. In this repository it is installed by `.devcontainer/Dockerfile`, has host state mounts in `.devcontainer/devcontainer.json`, and is recommended as a VS Code extension. The requested end state removes it entirely from the devcontainer.

GitHub auth is optional maintainer state. `.devcontainer/devcontainer.json` passes one host env file to Docker with `--env-file`; `.devcontainer/setup-github-auth.sh` configures repo-local Git credentials inside the container only when `GH_TOKEN` or `GITHUB_TOKEN` is present. `tools/repo/setup_github_env.sh` is a host helper that writes the optional token env files. Documentation for this flow lives in `docs/dev/github-auth.md`.

The maintainer docs that mention these paths live in `docs/dev/environment.md`, `docs/dev/fsdb.md`, and `docs/dev/github-auth.md`. The local breadcrumb `.devcontainer/AGENTS.md` also mentions which host paths and tools `.devcontainer/initialize.sh` prepares.

## Open Questions

There are no unresolved product decisions. Plan review resolved the main implementation risks: the helper must be type-aware, Verdi cleanup must not blindly delete non-empty directories, and the GitHub setup helper must tolerate unrelated entries under `~/.config/wavepeek-dev`.

## Plan of Work

First, review this ExecPlan using read-only subagents. One review lane should focus on devcontainer and shell correctness: whether static bind mounts will have valid host sources, whether symlink handling is safe, and whether OpenCode removal is complete. A second review lane should focus on documentation and breadcrumb consistency: whether all user-facing maintainer docs point to `~/.config/wavepeek-dev`, whether GitHub security guidance remains accurate, and whether stale `~/.cache/wavepeek` or `~/.config/wavepeek` references remain outside historical WIP context.

After plan review, update `.devcontainer/initialize.sh`. Replace `WAVEPEEK_STATE_DIR` and `WAVEPEEK_GITHUB_CONFIG_DIR` with one `WAVEPEEK_DEV_CONFIG_DIR="$HOME/.config/wavepeek-dev"`. Make `WAVEPEEK_VERDI_MOUNT_SOURCE="$WAVEPEEK_DEV_CONFIG_DIR/verdi"`, `WAVEPEEK_GITHUB_EMPTY_ENV="$WAVEPEEK_DEV_CONFIG_DIR/github.empty.env"`, and `WAVEPEEK_GITHUB_ENV="$WAVEPEEK_DEV_CONFIG_DIR/github.env"`. Stop creating `~/.config/opencode`, `~/.local/share/opencode`, `~/.cache/opencode`, `~/.claude`, `~/.codex`, `~/.pi`, and `~/.cache/wavepeek`. Create only `~/.config/wavepeek-dev` directly, with mode 700 where possible.

In the same script, add a small kind-aware helper for agent state bind sources. For directory state, the helper receives a managed path such as `~/.config/wavepeek-dev/claude` and a legacy path such as `~/.claude`. It must only link to the legacy path when the legacy path is actually a directory or a symlink that resolves as a directory. If the legacy directory exists and the managed path is absent, it creates a symlink from the managed path to the legacy path. If the legacy path does not exist, it creates the managed directory. If the legacy path exists but has the wrong type, it prints a warning and creates or preserves the managed placeholder instead of producing a bad mount. For the file state `claude.json`, apply the same rule with file checks: only link to a regular file or file symlink, and create a managed file containing `{}` when neither valid path exists. This prevents Docker from creating the file or directory itself and avoids mounting nonsense because someone, somewhere, made `~/.codex` a file. Of course they did.

Then update `.devcontainer/devcontainer.json`. Change the Docker `--env-file` from `${localEnv:HOME}/.config/wavepeek/github.env` to `${localEnv:HOME}/.config/wavepeek-dev/github.env`. Remove the three OpenCode mounts. Change the Claude, Codex, Pi, and Verdi mount sources to `${localEnv:HOME}/.config/wavepeek-dev/claude`, `${localEnv:HOME}/.config/wavepeek-dev/claude.json`, `${localEnv:HOME}/.config/wavepeek-dev/codex`, `${localEnv:HOME}/.config/wavepeek-dev/pi`, and `${localEnv:HOME}/.config/wavepeek-dev/verdi`. Remove `sst-dev.opencode` from VS Code extension recommendations.

Then update `.devcontainer/Dockerfile` and `.devcontainer/env_contract.sh`. In the Dockerfile, keep the npm install of `@openai/codex` and `@earendil-works/pi-coding-agent`, but remove the pinned OpenCode `curl`/`tar` install block. In the environment contract, remove `WAVEPEEK_OPENCODE_VERSION` because nothing should consume it after OpenCode is gone.

Then update documentation and helper scripts. Change `tools/repo/setup_github_env.sh` to write to `~/.config/wavepeek-dev`, but do not keep its old "config directory must be empty" rule. Instead, make it reject only pre-existing `github.empty.env`, `github.maintainer.env`, or `github.env` entries, because unrelated managed entries in that directory are expected after consolidation. Update `tools/repo/README.md`, `docs/dev/environment.md`, `docs/dev/github-auth.md`, and `docs/dev/fsdb.md` to describe the new single host state root. Update `.devcontainer/AGENTS.md` to remove OpenCode and old paths, and to say `initialize.sh` prepares Claude Code, Codex, Pi, Verdi, and optional GitHub env-file sources under `~/.config/wavepeek-dev`. Remove the stale OpenCode sentence from `tools/codex/codex_env_common.sh` because OpenCode is no longer an interactive dev-only exception.

Finally, search the repository for stale active references. Active config and docs should not contain `opencode`, `.cache/wavepeek`, or the exact old path `.config/wavepeek/` except where a document intentionally explains the old path as historical migration context. Because `docs/tracker/wip/proposal.md` is an old branch-local GitHub-auth proposal that currently documents old paths, either update it if keeping it useful is cheap or explicitly note that it is historical. Do not delete unrelated WIP files without evidence that this task owns them.

### Concrete Steps

Run these commands from the repository root `/workspaces/wavepeek` as the change proceeds:

    git status --short
    rg -n "OpenCode|opencode|\.cache/wavepeek|\.config/wavepeek(/|$)|\.claude|\.codex|\.pi" .devcontainer docs tools .github justfile -S

After editing shell and JSON files, run:

    bash -n .devcontainer/initialize.sh
    bash -n .devcontainer/setup-github-auth.sh
    bash -n tools/repo/setup_github_env.sh
    python3 -m json.tool .devcontainer/devcontainer.json >/dev/null

Exercise the host initializer in an isolated temporary home so it does not alter the real user home during validation:

    tmp_home="$(mktemp -d tmp/wavepeek-home.XXXXXX)"
    HOME="$tmp_home" VERDI_HOME= bash .devcontainer/initialize.sh
    find "$tmp_home" -maxdepth 4 -mindepth 1 | sort
    rm -rf "$tmp_home"

The expected listing should contain only `.config/wavepeek-dev` state below the temporary home, not top-level `.claude`, `.codex`, `.pi`, or `.cache/wavepeek` paths. A second initializer scenario should pre-create top-level legacy paths in a temporary home, then verify with explicit `test -L` and `readlink` checks that `~/.config/wavepeek-dev/claude`, `claude.json`, `codex`, and `pi` are symlinks to those existing paths. A third small scenario should create wrong-type legacy paths, such as a file at `~/.codex` or a directory at `~/.claude.json`, and confirm the initializer warns and still creates correctly typed managed placeholders.

Before handoff, run the repository gate:

    just check

If `just check` is unavailable because the session is not inside the devcontainer, record that fact and run the targeted syntax and search checks instead. In the current project workflow, `just check` is the local pre-handoff gate and should be attempted.

### Validation and Acceptance

The change is accepted when these observable checks pass:

A clean temporary home with no pre-existing agent state, after running `.devcontainer/initialize.sh`, contains `~/.config/wavepeek-dev` with managed subpaths for GitHub env files, Claude, Codex, Pi, and Verdi. It does not contain `~/.claude`, `~/.claude.json`, `~/.codex`, `~/.pi`, or `~/.cache/wavepeek`.

A temporary home with existing `~/.claude`, `~/.claude.json`, `~/.codex`, and `~/.pi`, after running `.devcontainer/initialize.sh`, contains symlinks inside `~/.config/wavepeek-dev` pointing back to those existing paths. This proves existing user agent state can still be mounted without the devcontainer creating new top-level state.

`.devcontainer/devcontainer.json` passes Docker the env file at `${localEnv:HOME}/.config/wavepeek-dev/github.env` and every host bind source for agent or Verdi state starts with `${localEnv:HOME}/.config/wavepeek-dev/`. It has no OpenCode mount and no `sst-dev.opencode` extension.

The Dockerfile no longer installs OpenCode, and `.devcontainer/env_contract.sh` no longer defines an OpenCode version. A repository search for `opencode` should return no active provisioning references.

The documented GitHub auth helper writes `~/.config/wavepeek-dev/github.empty.env`, `~/.config/wavepeek-dev/github.maintainer.env`, and `~/.config/wavepeek-dev/github.env`.

### Idempotence and Recovery

`.devcontainer/initialize.sh` must be safe to run repeatedly. Re-running it should keep `github.env` if it already exists, keep or recreate `github.empty.env`, and keep valid managed agent placeholders or symlinks. For Verdi, it may unlink an existing `verdi` symlink or replace an empty directory, but it must not delete a non-empty real directory and must not remove the target of a symlink. It should not remove top-level user agent state, should not overwrite token-bearing GitHub env files, and should not delete non-empty managed agent state under `~/.config/wavepeek-dev`.

If a managed path under `~/.config/wavepeek-dev` conflicts with an existing top-level legacy path, the safe fallback is to leave the managed path in place and print a warning. If a legacy path exists with the wrong type, the safe fallback is to warn and use the correctly typed managed placeholder. The user can manually move or delete the managed path if they want the symlink. This is less magical than deleting user data; less glamorous, substantially less likely to eat someone's config.

### Artifacts and Notes

Initial stale-reference search before implementation found active references in these files:

    .devcontainer/initialize.sh
    .devcontainer/devcontainer.json
    .devcontainer/AGENTS.md
    .devcontainer/Dockerfile
    .devcontainer/env_contract.sh
    docs/dev/github-auth.md
    docs/dev/fsdb.md
    docs/dev/environment.md
    tools/codex/codex_env_common.sh
    tools/repo/README.md
    tools/repo/setup_github_env.sh

`docs/tracker/wip/proposal.md` also contains old GitHub-auth design text. Because it is a tracked WIP artifact, update or annotate it only if needed to avoid misleading future agents on this branch.

Revision note, 2026-06-06: Incorporated pre-implementation review findings. The plan now requires kind-aware legacy handling, safer Verdi source management, GitHub-file-specific setup helper validation, clearer `--env-file` wording, and exact old-path stale-reference searches.

### Interfaces and Dependencies

No Rust interfaces change. The relevant shell contract after the change is:

    .devcontainer/initialize.sh
      Input environment:
        HOME: host home directory used by the devcontainer CLI.
        VERDI_HOME: optional host path to a Verdi installation.
      Output host paths:
        $HOME/.config/wavepeek-dev/github.empty.env
        $HOME/.config/wavepeek-dev/github.env
        $HOME/.config/wavepeek-dev/verdi
        $HOME/.config/wavepeek-dev/claude
        $HOME/.config/wavepeek-dev/claude.json
        $HOME/.config/wavepeek-dev/codex
        $HOME/.config/wavepeek-dev/pi

The relevant JSON contract after the change is:

    .devcontainer/devcontainer.json
      runArgs includes:
        --env-file
        ${localEnv:HOME}/.config/wavepeek-dev/github.env
      mounts include:
        source=${localEnv:HOME}/.config/wavepeek-dev/claude,target=/home/ubuntu/.claude,type=bind
        source=${localEnv:HOME}/.config/wavepeek-dev/claude.json,target=/home/ubuntu/.claude.json,type=bind
        source=${localEnv:HOME}/.config/wavepeek-dev/codex,target=/home/ubuntu/.codex,type=bind
        source=${localEnv:HOME}/.config/wavepeek-dev/pi,target=/home/ubuntu/.pi,type=bind
        source=${localEnv:HOME}/.config/wavepeek-dev/verdi,target=/opt/verdi,type=bind
