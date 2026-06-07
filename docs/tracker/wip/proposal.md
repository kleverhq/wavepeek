# Implement Optional GitHub Authentication for the `wavepeek` Dev Container

Historical note, 2026-06-06: this WIP proposal describes the earlier GitHub-auth change and the devcontainer shape that existed before coding-agent host state was consolidated. Do not use the path examples or OpenCode mount examples below as current implementation guidance. Current devcontainer agent, Verdi, and GitHub env-file state is managed under `~/.config/wavepeek-dev`; see `devcontainer-agent-state-consolidation.md` in this directory and the maintained docs under `docs/dev/`.

## Task summary

Implement an optional, repo-scoped GitHub authentication setup for the `kleverhq/wavepeek` development container.

The goal is to let maintainers and trusted local agents work with GitHub from inside the dev container without running `gh auth login` after every rebuild, while keeping the shared dev container safe and usable for external contributors who fork the repository and open pull requests.

The implementation must be **zero-secret by default**:

- A fresh clone must build and run without any GitHub token.
- No Personal Access Token, GitHub CLI credential store, or host-level GitHub config should be committed to the repository.
- Maintainers may opt in locally by creating a host-side env file outside the repository.
- External fork contributors must not need a token for the normal build/test/dev workflow.
- Maintainers must be able to disable GitHub write credentials when reviewing untrusted external PRs.

The change must also update maintainer documentation and local agent breadcrumbs so the repository has a single durable explanation of the new GitHub-auth behavior instead of leaving this WIP proposal as the only map. Documentation should be concise, security-oriented, and scoped to the devcontainer/maintainer workflow rather than public `wavepeek` CLI docs.

## Current repository state to account for

At the time this brief was written, `.devcontainer/devcontainer.json` has the following relevant behavior:

- `containerEnv` contains `WAVEPEEK_IN_CONTAINER=1` and `VERDI_HOME=/opt/verdi`.
- `runArgs` contains only `--network=host`.
- `initializeCommand` runs `.devcontainer/initialize.sh` on the host.
- `postCreateCommand` only marks the workspace as a Git safe directory.
- `mounts` includes a bind mount from `${localEnv:HOME}/.config/gh` to `/home/ubuntu/.config/gh`.

The current `.devcontainer/initialize.sh` also creates several host-side directories before container creation, including `~/.config/gh`.

The implementation below intentionally removes the host `~/.config/gh` mount and replaces it with an optional host-side env file.

## Design principles

### 1. Do not mount `~/.config/gh` into the container

Remove this mount from `.devcontainer/devcontainer.json`:

```json
"source=${localEnv:HOME}/.config/gh,target=/home/ubuntu/.config/gh,type=bind"
```

Rationale:

- GitHub CLI can authenticate directly from `GH_TOKEN` or `GITHUB_TOKEN`.
- Mounting host `~/.config/gh` may expose a broader host login to the container and to any agent running inside it.
- If `gh auth login` stores a token in a plain-text fallback config file, that file could end up persisted on the host through the bind mount.
- The desired setup is repo-scoped and explicit, not inherited from whatever the host `gh` login happens to contain.

### 2. Use an optional host-side env file

Use this file on the host:

```text
~/.config/wavepeek/github.env
```

This file must be outside the repository and must not be committed.

The dev container should receive it via Docker `--env-file`:

```json
"runArgs": [
  "--network=host",
  "--env-file",
  "${localEnv:HOME}/.config/wavepeek/github.env"
]
```

The host-side `initializeCommand` must create this file before container creation so that the container also starts for developers who do not have a token.

### 3. Do not set `GH_REPO` globally

Do **not** add `GH_REPO=kleverhq/wavepeek` to `containerEnv`.

`GH_REPO` changes the default repository used by many `gh` commands. That is undesirable for contributors working from forks, because their local `origin` may point to their fork while the upstream repository remains `kleverhq/wavepeek`.

Instead, add a project-specific variable:

```text
WAVEPEEK_UPSTREAM_REPO=kleverhq/wavepeek
```

When scripts need the upstream repository, they should pass it explicitly:

```bash
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO"
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO"
gh run list -R "$WAVEPEEK_UPSTREAM_REPO"
```

### 4. Keep maintainer and contributor flows separate

Maintainers working directly in `kleverhq/wavepeek` may use a fine-grained PAT scoped only to this repository.

External contributors working from forks should not need a `kleverhq/wavepeek` token. They should be able to:

- clone their fork,
- build the dev container,
- create a branch,
- push to their fork,
- open a PR against `kleverhq/wavepeek`.

### 5. Do not expose maintainer tokens to untrusted PR code

When a maintainer reviews an external PR from a fork, they must be able to run the dev container without a GitHub write token.

The required default is an empty host-side env file:

```text
~/.config/wavepeek/github.empty.env
~/.config/wavepeek/github.env -> github.empty.env
```

Maintainers may point `github.env` at a token-bearing env file for trusted work. However, symlink switching alone is not a complete external-PR defense if a token-bearing file remains at a readable host path such as `~/.config/wavepeek/github.maintainer.env`. Devcontainer `initializeCommand` runs on the host before Docker reads `--env-file`; malicious PR code could modify `.devcontainer/initialize.sh` and read any token file the host user can read. For untrusted PR review, the maintainer token file must be removed from the host user's readable filesystem, kept in a password manager or equivalent store, isolated in a separate host account/VM/codespace, or avoided by creating the container from a trusted base-branch configuration before checking out the PR.

Important: Docker reads `--env-file` at container creation time. After switching the active env file, recreate or rebuild the dev container. Merely attaching to an existing container keeps the old environment.

## Required repository changes

### Change 1: update `.devcontainer/devcontainer.json`

Modify the existing dev container config as follows.

Keep existing values such as `WAVEPEEK_IN_CONTAINER`, `VERDI_HOME`, `workspaceFolder`, `workspaceMount`, `remoteUser`, VS Code customizations, and the existing non-GitHub agent mounts unless there is a separate reason to change them.

Required changes:

1. Add `WAVEPEEK_UPSTREAM_REPO` and `GH_PROMPT_DISABLED` to `containerEnv`.
2. Add `--env-file ${localEnv:HOME}/.config/wavepeek/github.env` to `runArgs`.
3. Remove the bind mount for `${localEnv:HOME}/.config/gh`.
4. Extend `postCreateCommand` so it also runs `.devcontainer/setup-github-auth.sh`.

Target shape for the relevant fields:

```json
{
  "containerEnv": {
    "WAVEPEEK_IN_CONTAINER": "1",
    "VERDI_HOME": "/opt/verdi",
    "WAVEPEEK_UPSTREAM_REPO": "kleverhq/wavepeek",
    "GH_PROMPT_DISABLED": "1"
  },
  "runArgs": [
    "--network=host",
    "--env-file",
    "${localEnv:HOME}/.config/wavepeek/github.env"
  ],
  "mounts": [
    "source=${localEnv:HOME}/.config/opencode,target=/home/ubuntu/.config/opencode,type=bind",
    "source=${localEnv:HOME}/.local/share/opencode,target=/home/ubuntu/.local/share/opencode,type=bind",
    "source=${localEnv:HOME}/.cache/opencode,target=/home/ubuntu/.cache/opencode,type=bind",
    "source=${localEnv:HOME}/.claude,target=/home/ubuntu/.claude,type=bind",
    "source=${localEnv:HOME}/.claude.json,target=/home/ubuntu/.claude.json,type=bind",
    "source=${localEnv:HOME}/.codex,target=/home/ubuntu/.codex,type=bind",
    "source=${localEnv:HOME}/.pi,target=/home/ubuntu/.pi,type=bind",
    "source=${localEnv:HOME}/.cache/wavepeek/verdi,target=/opt/verdi,type=bind"
  ],
  "postCreateCommand": "git config --global --add safe.directory \"$PWD\" && bash .devcontainer/setup-github-auth.sh"
}
```

Do not blindly replace the whole file if the actual repository has drifted. Apply the same semantic changes to the current file.

### Change 2: update `.devcontainer/initialize.sh`

The host-side initialization script must create the optional GitHub env-file path before the container is created.

Add this logic near the existing host-side directory setup:

```bash
# Optional GitHub auth env-file used by the dev container.
# This is intentionally outside the repository and empty by default.
WAVEPEEK_GITHUB_CONFIG_DIR="$HOME/.config/wavepeek"
WAVEPEEK_GITHUB_EMPTY_ENV="$WAVEPEEK_GITHUB_CONFIG_DIR/github.empty.env"
WAVEPEEK_GITHUB_ENV="$WAVEPEEK_GITHUB_CONFIG_DIR/github.env"

mkdir -p "$WAVEPEEK_GITHUB_CONFIG_DIR"
chmod 700 "$WAVEPEEK_GITHUB_CONFIG_DIR" 2>/dev/null || true

if [ ! -e "$WAVEPEEK_GITHUB_EMPTY_ENV" ]; then
  : > "$WAVEPEEK_GITHUB_EMPTY_ENV"
fi
chmod 600 "$WAVEPEEK_GITHUB_EMPTY_ENV" 2>/dev/null || true

if [ ! -e "$WAVEPEEK_GITHUB_ENV" ]; then
  ln -s "github.empty.env" "$WAVEPEEK_GITHUB_ENV"
fi
```

Also remove `"$HOME/.config/gh"` from the host directory creation list if it is no longer used by any other mount.

Do not overwrite an existing `~/.config/wavepeek/github.env`. Maintainers may intentionally keep it as a symlink to their real maintainer token file.

### Change 3: add `.devcontainer/setup-github-auth.sh`

Create a new script:

```bash
#!/usr/bin/env bash
set -euo pipefail

upstream_repo="${WAVEPEEK_UPSTREAM_REPO:-kleverhq/wavepeek}"
token="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

if [ -n "$token" ]; then
  # Scope Git credential lookup to repository paths. This allows the helper to
  # decide whether the requested credential is for the wavepeek upstream repo.
  git config --local credential.useHttpPath true

  # Remove previous repo-local helpers installed by this script, if any. This
  # does not touch host-level or global credentials outside the container.
  git config --local --unset-all credential.https://github.com.helper 2>/dev/null || true

  # The helper stores no secret in git config. It reads GH_TOKEN/GITHUB_TOKEN
  # from the process environment at credential lookup time and only returns it
  # for the configured upstream repository.
  git config --local --add credential.https://github.com.helper \
    '!f() {
      test "$1" = get || exit 0

      protocol=""
      host=""
      path=""

      while IFS= read -r line; do
        case "$line" in
          protocol=*) protocol="${line#protocol=}" ;;
          host=*) host="${line#host=}" ;;
          path=*) path="${line#path=}" ;;
        esac
      done

      test "$protocol" = https || exit 0
      test "$host" = github.com || exit 0

      repo="${WAVEPEEK_UPSTREAM_REPO:-kleverhq/wavepeek}"
      case "$path" in
        "$repo"|"$repo.git")
          echo username=x-access-token
          echo password="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
          ;;
      esac
    }; f'

  if command -v gh >/dev/null 2>&1; then
    if gh repo view "$upstream_repo" >/dev/null 2>&1; then
      echo "GitHub auth configured; upstream repo is reachable: $upstream_repo"
    else
      echo "GitHub token is present, but $upstream_repo is not reachable with it."
      echo "This may be expected for fork contributors, unapproved organization fine-grained PATs, or insufficient token permissions."
    fi
  else
    echo "GitHub token is present, but GitHub CLI is not installed; skipped gh validation."
  fi
else
  echo "GH_TOKEN/GITHUB_TOKEN is not set; GitHub auth setup skipped."
fi

# For fork-based checkouts, keep origin unchanged and add/update an upstream
# remote pointing to kleverhq/wavepeek. Do not rewrite contributor forks.
origin_url="$(git remote get-url origin 2>/dev/null || true)"

if ! printf '%s\n' "$origin_url" | grep -Eq 'github.com[:/]kleverhq/wavepeek(\.git)?$'; then
  if git remote get-url upstream >/dev/null 2>&1; then
    git remote set-url upstream "https://github.com/${upstream_repo}.git"
  else
    git remote add upstream "https://github.com/${upstream_repo}.git"
  fi
fi
```

Then make it executable:

```bash
chmod +x .devcontainer/setup-github-auth.sh
```

Notes:

- The script must not fail if `GH_TOKEN`/`GITHUB_TOKEN` is absent.
- The script must not print token values.
- The script must not run `gh auth login`.
- The script must not write a PAT into `.git/config`, `.config/gh`, shell history, or any repository file.
- The repo-local Git credential helper is acceptable because it stores only shell logic, not the token.

### Change 4: add `tools/repo/setup_github_env.sh`

Add a small one-shot host helper for maintainers preparing a clean local GitHub auth env directory before rebuilding the devcontainer.

The helper contract is intentionally plain:

- path: `tools/repo/setup_github_env.sh`;
- invocation: `bash tools/repo/setup_github_env.sh`;
- the helper prompts for the token without echoing it, and may read one token line from stdin for non-interactive use;
- positional arguments are rejected so the documented path does not expose the token through shell history or process argv;
- if `~/.config/wavepeek` exists and is not empty, exit non-zero, print a short message, and tell the maintainer to edit the env files manually;
- if the directory does not exist or exists but is empty, create `github.empty.env`, `github.maintainer.env`, and `github.env -> github.maintainer.env`;
- do not print the token;
- keep the script simple. This is a bootstrap helper, not a tiny compliance department with a clipboard.

Update `tools/repo/README.md` and auxiliary tests for this helper.

### Change 5: add `docs/dev/github-auth.md`

Add a new maintainer runbook that is the source of truth for optional GitHub authentication in the devcontainer.

The document should cover:

- zero-secret default behavior;
- why the devcontainer does not mount host `~/.config/gh`;
- the host-side env-file layout:
  - `~/.config/wavepeek/github.empty.env`;
  - `~/.config/wavepeek/github.env` as the active file or symlink;
  - optional token-bearing maintainer env files for trusted work only;
- maintainer fine-grained PAT recommendations;
- the one-shot `tools/repo/setup_github_env.sh` clean-directory setup helper;
- how to activate maintainer credentials for trusted local work;
- why switching back to `github.empty.env` only changes newly created container environments and does not protect a readable host token file from malicious PR-controlled `initializeCommand`;
- safe external-PR review options such as removing the token-bearing file, using a password manager, using a separate host account/VM/codespace, or creating the container from trusted base-branch config before checking out the PR;
- the requirement to recreate or rebuild the devcontainer after changing `github.env`, because Docker reads `--env-file` at container creation time;
- why `GH_REPO` is intentionally not set globally;
- using `WAVEPEEK_UPSTREAM_REPO=kleverhq/wavepeek` and explicit `gh -R "$WAVEPEEK_UPSTREAM_REPO" ...` commands when scripts need the upstream repository;
- fork contributor remote layout and the browser-first PR flow;
- verification commands for no-token, maintainer-token, upstream remote, and secret-leakage cases.

Keep the runbook focused on the maintainer/devcontainer workflow. Do not copy every implementation detail from this proposal if it would become stale faster than the actual scripts.

Recommended structure:

```text
# Optional GitHub Authentication

## Security model
## Host env files
## Maintainer setup
## Disabling credentials for external PR review
## Fork contributor flow
## Verification
## Troubleshooting
```

### Change 6: update `docs/dev/environment.md`

Add a short devcontainer subsection, for example `## Optional GitHub Authentication`, that explains:

- the devcontainer starts without GitHub credentials by default;
- `.devcontainer/initialize.sh` creates `~/.config/wavepeek/github.env` as an empty default outside the repository;
- `.devcontainer/devcontainer.json` passes that file through Docker `--env-file`;
- `.devcontainer/setup-github-auth.sh` configures repo-local GitHub auth only when `GH_TOKEN` or `GITHUB_TOKEN` is present;
- the full setup and security runbook lives in `github-auth.md`.

Also adjust the existing host-state paragraph so it mentions both wavepeek-managed cache state under `~/.cache/wavepeek` and optional configuration state under `~/.config/wavepeek`. Do not imply that `~/.config/gh` is mounted or prepared.

### Change 7: update `docs/dev/git.md`

Add a short GitHub/fork hygiene section that explains:

- fork contributors should keep `origin` pointing at their fork;
- `upstream` should point at `https://github.com/kleverhq/wavepeek.git`;
- `.devcontainer/setup-github-auth.sh` may add or update `upstream` when `origin` is not the upstream repository, but must not rewrite `origin`;
- commands that intentionally target the upstream repository should use `-R "$WAVEPEEK_UPSTREAM_REPO"` or `-R kleverhq/wavepeek`;
- browser-based PR creation remains supported and must not require GitHub CLI authentication.

Keep this section brief and point readers to `github-auth.md` for token handling and security details.

### Change 8: update `docs/dev/automation.md`

Add a small devcontainer lifecycle-helper note that explains:

- `.devcontainer/initialize.sh` runs on the host before container creation and prepares bind-mount/env-file prerequisites;
- `.devcontainer/setup-github-auth.sh` runs inside the container from `postCreateCommand` and configures optional repo-local GitHub auth;
- changes to those helpers should keep `.devcontainer/devcontainer.json`, `environment.md`, and `github-auth.md` in sync.

Do not turn `automation.md` into a second GitHub-auth runbook. The source of truth should remain `github-auth.md`.

### Change 9: update breadcrumbs and root maintainer map

Update `.devcontainer/AGENTS.md` because its current guidance says `initialize.sh` prepares GitHub CLI state. After this change, that is stale and unsafe.

Expected breadcrumb guidance:

- `initialize.sh` prepares host mount sources for OpenCode, Claude Code, Codex, Pi, Verdi, and the optional wavepeek GitHub env-file;
- do not mount host `~/.config/gh` into the devcontainer;
- optional GitHub auth is documented in `../docs/dev/github-auth.md`;
- do not store PATs in repository files, `.git/config`, breadcrumb files, logs, or shell history.

Keep this breadcrumb short. It should point to the runbook, not duplicate it.

Update root `AGENTS.md` so the `docs/dev/` map includes:

```text
- `docs/dev/github-auth.md` for optional repo-scoped GitHub auth in the devcontainer.
```

No update is required for `docs/AGENTS.md` unless the final documentation layout changes in a way that invalidates its existing source-of-truth bullets.

### Change 10: update `CHANGELOG.md`

Add an `Unreleased` entry, preferably under `Changed`, such as:

```text
- Replaced the devcontainer host GitHub CLI config mount with optional zero-secret repo-scoped GitHub authentication via a host env-file.
```

If the implementation lands with materially different wording or scope, adjust the changelog line to match the actual user-visible maintainer workflow change.

### Documentation non-goals

Do not update `docs/public/**`: optional devcontainer GitHub authentication is not part of the public `wavepeek` CLI documentation surface.

Do not add this material to `docs/dev/quality.md` or `docs/dev/release.md` unless implementation details create a direct quality-gate or release-process requirement. Keep the verification commands in `docs/dev/github-auth.md`.

Before merging to the default branch, remove `docs/tracker/wip/proposal.md` unless a maintainer explicitly wants to keep this WIP handoff artifact. Once `docs/dev/github-auth.md` exists, this proposal must not become a second source of truth. Naturally. One map is already more than enough opportunity for humans to ignore it.

## Maintainer setup instructions

Each maintainer who wants an agent inside the dev container to work with GitHub should create their own fine-grained PAT.

Recommended token settings:

```text
Token type: Fine-grained personal access token
Resource owner: kleverhq
Repository access: Only selected repositories
Selected repository: kleverhq/wavepeek
Expiration: 30-90 days preferred; do not use no-expiration tokens unless there is a strong reason
```

Recommended baseline repository permissions:

```text
Metadata: Read
Contents: Read and write
Pull requests: Read and write
Issues: Read and write
Actions: Read
```

Add only if required:

```text
Actions: Read and write
```

Use this only if the agent must rerun, cancel, dispatch, or otherwise manage GitHub Actions runs.

Add only if required:

```text
Workflows: Read and write
```

Use this only if the agent must edit files under `.github/workflows/*`.

Avoid unless there is a specific, reviewed reason:

```text
Administration
Secrets
Webhooks
Security advisories
Dependabot secrets
Repository rules
```

For a clean host-side `~/.config/wavepeek` directory, create and activate the maintainer env files with the one-shot helper:

```bash
bash tools/repo/setup_github_env.sh
```

The helper prompts for the token without echoing it, writes `github.empty.env`, `github.maintainer.env`, and an active `github.env -> github.maintainer.env` symlink. If `~/.config/wavepeek` already exists and is not empty, it exits with an error and leaves manual edits to the maintainer.

The helper rejects positional arguments so the documented path does not put the token in shell history or expose it through process argv. For non-interactive use, pipe exactly one token line into the script and mind shell history there too.

Manual `github.maintainer.env` content has this shape:

```text
GH_TOKEN=<github-token>
GITHUB_TOKEN=<github-token>
WAVEPEEK_GITHUB_ROLE=maintainer
```

For trusted work, switching `github.env` controls what Docker injects into newly created containers. After switching the symlink, recreate or rebuild the dev container. Docker does not live-reload values from `--env-file` into an existing container.

Before reviewing untrusted external PR code, also make any token-bearing maintainer env file unavailable to the host user that will run the PR's devcontainer configuration. Switching `github.env` to the empty file is necessary but not sufficient when `initializeCommand` comes from the PR checkout.

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
# Also remove the token-bearing file, use a separate host account/VM, or create
# the container from trusted base-branch devcontainer config before checkout.
```

## External contributor flow

External contributors should not need a `kleverhq/wavepeek` token.

Recommended remote layout for fork contributors:

```text
origin   = git@github.com:<contributor-login>/wavepeek.git
upstream = https://github.com/kleverhq/wavepeek.git
```

The setup script should add or update the `upstream` remote when `origin` is not `kleverhq/wavepeek`, but it must not rewrite `origin`.

Typical contributor commands:

```bash
git checkout -b my-change
# edit, build, test
git push -u origin my-change
```

They can then open the PR through the GitHub web UI, or through GitHub CLI if they have their own authentication set up:

```bash
gh pr create \
  -R kleverhq/wavepeek \
  --head "<contributor-login>:my-change" \
  --base main \
  --fill
```

Do not require this CLI path. The browser PR flow must remain sufficient.

## Security guidance for maintainers

Treat code from fork PRs as untrusted. Do not run untrusted PR code in a container that has a write-capable `GH_TOKEN` or `GITHUB_TOKEN` for `kleverhq/wavepeek`.

Also do not leave a readable maintainer token file at a predictable host path when the PR checkout controls `.devcontainer/initialize.sh`. Before opening a dev container for external PR review, make the token unavailable to the host user or use a trusted base-branch devcontainer configuration, then point the active env file at the empty default:

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
```

Then recreate or rebuild the container and verify inside it:

```bash
printenv GH_TOKEN
printenv GITHUB_TOKEN
```

Both commands should print nothing.

Also verify that host GitHub CLI config is not mounted:

```bash
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

This command should succeed for the default zero-secret configuration.

## Acceptance criteria

### Zero-secret contributor case

Starting from a clean host with no maintainer token:

1. `~/.config/wavepeek/github.env` is created automatically by `.devcontainer/initialize.sh`.
2. The dev container builds and starts successfully.
3. Inside the container, `GH_TOKEN` and `GITHUB_TOKEN` are absent or empty.
4. `~/.config/gh` from the host is not mounted into the container.
5. Normal project setup still works, including the existing `postStartCommand` behavior.
6. If `origin` points to a fork, an `upstream` remote is present and points to `https://github.com/kleverhq/wavepeek.git`.
7. `origin` is not rewritten.

### Maintainer trusted-work case

With `~/.config/wavepeek/github.env` pointing to a valid maintainer env file:

1. The dev container starts without interactive GitHub login.
2. Inside the container, `gh repo view kleverhq/wavepeek` succeeds for a token that has sufficient access.
3. `gh pr list -R kleverhq/wavepeek` works.
4. `gh issue list -R kleverhq/wavepeek` works.
5. Git HTTPS operations against `https://github.com/kleverhq/wavepeek.git` can use the token without storing it in the remote URL.
6. No PAT appears in `.git/config`, `.devcontainer/*`, shell history, or logs.

### External PR review case

With `~/.config/wavepeek/github.env` pointing to `github.empty.env` and any maintainer token-bearing file removed, unavailable to the host user, or isolated away from PR-controlled host commands:

1. Rebuilt/recreated dev container has no `GH_TOKEN` or `GITHUB_TOKEN` in the environment.
2. The setup script does not fail.
3. The agent inside the container has no write-capable GitHub token for `kleverhq/wavepeek`.
4. Project build/test commands still run normally.
5. The documentation does not imply that symlink switching alone protects a readable host token file from malicious PR-controlled `initializeCommand`.

### Documentation and helper case

1. `tools/repo/setup_github_env.sh` exists, prompts for the token without echoing it, initializes a clean `~/.config/wavepeek`, rejects positional arguments, and refuses to modify a non-empty existing config directory.
2. `docs/dev/github-auth.md` exists and is the durable source of truth for optional devcontainer GitHub authentication.
3. `docs/dev/environment.md`, `docs/dev/git.md`, and `docs/dev/automation.md` point to `github-auth.md` instead of duplicating the full runbook.
4. `.devcontainer/AGENTS.md` no longer claims that host GitHub CLI state is mounted or prepared.
5. Root `AGENTS.md` mentions `docs/dev/github-auth.md` in the maintainer documentation map.
6. `CHANGELOG.md` records the devcontainer GitHub-auth workflow change under `Unreleased`.
7. `docs/public/**` remains untouched unless an unrelated user-facing CLI documentation change is made.

## Tests and checks to run

Run these after implementation.

Syntax checks:

```bash
bash -n .devcontainer/initialize.sh
bash -n .devcontainer/setup-github-auth.sh
bash -n tools/repo/setup_github_env.sh
python -m json.tool .devcontainer/devcontainer.json >/dev/null
python -m unittest discover -s tools/repo -p "test_*.py"
```

Container no-token check:

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
# Recreate or rebuild the dev container after this; do not merely attach to an existing one.
printenv GH_TOKEN
printenv GITHUB_TOKEN
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

For untrusted external PR review, also verify by process: the maintainer token-bearing file is removed, kept in a store unavailable to repo-controlled host commands, or isolated in a separate host account/VM/codespace before the PR checkout's `initializeCommand` can run.

Maintainer token check:

```bash
ln -sf github.maintainer.env ~/.config/wavepeek/github.env
# Recreate or rebuild the dev container after this; do not merely attach to an existing one.
gh repo view kleverhq/wavepeek
gh pr list -R kleverhq/wavepeek --limit 5
gh issue list -R kleverhq/wavepeek --limit 5
```

Secret leakage check:

```bash
# Should not print a token.
git remote -v

# Should not contain a token literal.
git config --local --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
git config --global --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
```

Documentation sanity checks:

```bash
rg -n '\.config/gh|GitHub CLI state|github-auth|github\.env|WAVEPEEK_UPSTREAM_REPO|GH_REPO|GH_TOKEN|GITHUB_TOKEN' \
  .devcontainer/AGENTS.md AGENTS.md docs/dev docs/tracker/wip/proposal.md CHANGELOG.md
```

Use the search output to verify there are no stale claims that host `~/.config/gh` is mounted into the devcontainer, and that the new GitHub-auth runbook is referenced from the short maintainer docs instead of copied everywhere like an ambitious mold colony.

## Documentation references

Use these primary references if behavior needs to be verified during implementation:

- GitHub CLI environment variables, including `GH_TOKEN`, `GITHUB_TOKEN`, `GH_PROMPT_DISABLED`, and `GH_REPO`: https://cli.github.com/manual/gh_help_environment
- GitHub CLI authentication behavior and fine-grained PAT recommendation: https://cli.github.com/manual/gh_auth_login
- VS Code Dev Containers environment variable and `--env-file` guidance: https://code.visualstudio.com/remote/advancedcontainers/environment-variables
- Dev Container metadata reference for `runArgs`, `containerEnv`, and lifecycle commands: https://devcontainers.github.io/implementors/json_reference/
- GitHub personal access token guidance and fine-grained PAT limitations: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens
- Fine-grained PAT permission requirements for REST endpoints: https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens
- Creating pull requests from forks: https://docs.github.com/articles/creating-a-pull-request-from-a-fork
- `gh pr create` options, including `--head`: https://cli.github.com/manual/gh_pr_create

## Non-goals

Do not implement a GitHub App in this task.

A GitHub App may be a better long-term design if the project later needs a dedicated organization-owned bot identity, short-lived installation tokens, or auditable machine-to-machine access independent of individual maintainers. For this task, use the simpler optional fine-grained PAT setup described above.

Do not add repository-local secret files such as:

```text
.devcontainer/github.env
.env
.env.local
```

Do not commit any sample file containing a plausible real token prefix except inert placeholders such as `github_pat_REPLACE_WITH_REAL_TOKEN`.
