# Optional GitHub Authentication

The local devcontainer supports optional GitHub authentication for maintainer work. The default host setup uses an empty env file, and normal build/test flows do not require a token. If the host already has `~/.config/wavepeek/github.env` pointing at a maintainer token, newly created containers will receive that token.

## Authentication Model

The devcontainer reads one host-side env file when the container is created:

```text
~/.config/wavepeek/github.env
```

`.devcontainer/initialize.sh` creates an empty default outside the repository. `.devcontainer/devcontainer.json` passes that file to Docker with `--env-file`. `.devcontainer/setup-github-auth.sh` runs inside the container and configures repo-local GitHub access only when `GH_TOKEN` or `GITHUB_TOKEN` is present.

The devcontainer does not mount host `~/.config/gh`. GitHub CLI uses `GH_TOKEN` or `GITHUB_TOKEN` from the container environment instead.

The container sets:

```text
WAVEPEEK_UPSTREAM_REPO=kleverhq/wavepeek
GH_PROMPT_DISABLED=1
```

It intentionally does not set `GH_REPO`. Scripts that target the upstream repository should pass it explicitly:

```bash
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO"
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO"
gh run list -R "$WAVEPEEK_UPSTREAM_REPO"
```

## Host Env Files

Expected host-side layout:

```text
~/.config/wavepeek/github.empty.env
~/.config/wavepeek/github.maintainer.env
~/.config/wavepeek/github.env -> github.empty.env or github.maintainer.env
```

`github.empty.env` is created automatically and should stay empty. `github.env` is the active file Docker reads at container creation time. Maintainers can point it at `github.maintainer.env` for trusted work.

Docker does not reread `--env-file` for an existing container. After changing `github.env`, recreate or rebuild the devcontainer. Attaching to an existing container keeps the old environment.

## Maintainer Setup

Create a GitHub token limited to `kleverhq/wavepeek` with only the permissions needed for the task.

For a clean host-side `~/.config/wavepeek` directory, run this from the repository root on the host:

```bash
bash tools/repo/setup_github_env.sh
```

The helper prompts for the token without echoing it, then writes:

```text
~/.config/wavepeek/github.empty.env
~/.config/wavepeek/github.maintainer.env
~/.config/wavepeek/github.env -> github.maintainer.env
```

The helper rejects positional arguments so the token is not exposed through shell history or process argv by the documented command. For non-interactive use, pipe exactly one token line into the script:

```bash
printf '%s\n' "$TOKEN" | bash tools/repo/setup_github_env.sh
```

If `~/.config/wavepeek` already exists and is not empty, the helper exits without changing it. Edit the files manually in that case.

Manual `github.maintainer.env` content:

```text
GH_TOKEN=<github-token>
GITHUB_TOKEN=<github-token>
WAVEPEEK_GITHUB_ROLE=maintainer
```

After setup, recreate or rebuild the devcontainer. Then verify inside the container:

```bash
gh repo view "$WAVEPEEK_UPSTREAM_REPO"
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
```

Git HTTPS operations against `https://github.com/kleverhq/wavepeek.git` use a repo-local credential helper. The helper stores shell logic in `.git/config`, not the token value, and only returns credentials for the configured upstream repository path.

## External PR Review

Treat code from fork PRs as untrusted. Switching `github.env` to the empty file removes the token from newly created containers, but it does not protect a readable token file on the host from PR-controlled `initializeCommand` code.

Before opening a devcontainer from an untrusted PR checkout, make the maintainer token unavailable to the host process that will run the devcontainer configuration. Use one of these approaches:

- remove the token-bearing env file and recreate it later;
- use a separate host account, VM, or codespace without the token file;
- keep the token in a password manager and generate `github.env` only for trusted work;
- create the container from trusted base-branch devcontainer config, then check out the PR inside that container.

Also switch the active env file to the empty default:

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
```

Then recreate or rebuild the devcontainer. Do not attach to an existing container that may still have the old token environment.

Verify inside the new container:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

## Fork Contributor Flow

External contributors do not need a `kleverhq/wavepeek` token for normal development.

Recommended remotes for fork contributors:

```text
origin   = git@github.com:<contributor-login>/wavepeek.git
upstream = https://github.com/kleverhq/wavepeek.git
```

`.devcontainer/setup-github-auth.sh` adds or updates `upstream` when `origin` is not the upstream repository. It does not rewrite `origin`.

Typical flow:

```bash
git checkout -b my-change
# edit, build, test
git push -u origin my-change
```

A pull request can be opened through the GitHub web UI. Contributors who already have GitHub CLI authentication may use:

```bash
gh pr create \
  -R kleverhq/wavepeek \
  --head "<contributor-login>:my-change" \
  --base main \
  --fill
```

## Verification

Syntax checks after changing this workflow:

```bash
bash -n .devcontainer/initialize.sh
bash -n .devcontainer/setup-github-auth.sh
bash -n tools/repo/setup_github_env.sh
python3 -m json.tool .devcontainer/devcontainer.json >/dev/null
```

No-token container checks:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

For fork checkouts where `origin` is not `kleverhq/wavepeek`, also verify the upstream remote:

```bash
git remote get-url upstream
```

Maintainer-token checks:

```bash
gh repo view "$WAVEPEEK_UPSTREAM_REPO"
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
```

Secret leakage checks:

```bash
git remote -v
git config --local --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
git config --global --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
```

## Troubleshooting

If `gh repo view "$WAVEPEEK_UPSTREAM_REPO"` fails with a token present, check that the token has access to `kleverhq/wavepeek` and the permissions needed for the command.

If switching between maintainer and empty credentials appears to have no effect, recreate or rebuild the devcontainer. Docker does not reread `--env-file` for an existing container.

If `upstream` points somewhere unexpected, inspect `origin` first:

```bash
git remote -v
```

The setup script only adds or updates `upstream` when `origin` is not `kleverhq/wavepeek`; it never rewrites `origin`.
