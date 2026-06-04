# Optional GitHub Authentication

The local devcontainer supports optional, repo-scoped GitHub authentication for maintainers and trusted local agents. The default is deliberately boring: a fresh clone builds and runs without a token, and no host GitHub CLI credential store is mounted into the container. Boring is good here. Exciting credential plumbing is how secrets escape and start a career in incident response.

## Security model

The devcontainer reads an optional host-side env file at container creation time:

```text
~/.config/wavepeek/github.env
```

`.devcontainer/initialize.sh` creates that path outside the repository and points it at an empty file by default. `.devcontainer/devcontainer.json` passes it to Docker with `--env-file`, and `.devcontainer/setup-github-auth.sh` configures repo-local Git credential behavior only when `GH_TOKEN` or `GITHUB_TOKEN` is present.

The devcontainer does **not** mount host `~/.config/gh`. GitHub CLI can authenticate from `GH_TOKEN` or `GITHUB_TOKEN`, and mounting the host GitHub CLI config would expose whatever broader host login happens to exist. It could also persist a `gh auth login` fallback token back onto the host. Do not add that mount back unless you want the security model to become interpretive dance.

The container sets `WAVEPEEK_UPSTREAM_REPO=kleverhq/wavepeek` and `GH_PROMPT_DISABLED=1`. It intentionally does **not** set `GH_REPO`; that variable changes the default repository used by many `gh` commands and can surprise fork contributors whose `origin` points at their fork.

When scripts need the upstream repository, pass it explicitly:

```bash
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO"
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO"
gh run list -R "$WAVEPEEK_UPSTREAM_REPO"
```

## Host env files

Use this host-side layout:

```text
~/.config/wavepeek/github.empty.env
~/.config/wavepeek/github.maintainer.env
~/.config/wavepeek/github.env -> github.empty.env or github.maintainer.env
```

`github.empty.env` is created automatically and should stay empty. `github.env` is the active file Docker reads when the container is created. Maintainers may keep it as a symlink to the empty or maintainer file.

Docker reads `--env-file` only when creating the container. After switching `github.env`, recreate, rebuild, or reopen the devcontainer so the environment actually changes. Existing containers do not live-reload this file. Naturally, because that would be convenient.

## Maintainer setup

For trusted local work in `kleverhq/wavepeek`, create a fine-grained Personal Access Token with repository access limited to `kleverhq/wavepeek`.

Recommended settings:

```text
Token type: Fine-grained personal access token
Resource owner: kleverhq
Repository access: Only selected repositories
Selected repository: kleverhq/wavepeek
Expiration: 30-90 days preferred
```

Recommended baseline repository permissions:

```text
Metadata: Read
Contents: Read and write
Pull requests: Read and write
Issues: Read and write
Actions: Read
```

Add only when the task actually needs it:

```text
Actions: Read and write
Workflows: Read and write
```

Avoid permissions such as `Administration`, `Secrets`, `Webhooks`, `Security advisories`, `Dependabot secrets`, and `Repository rules` unless there is a specific reviewed reason.

Create the maintainer env file on the host:

```bash
mkdir -p ~/.config/wavepeek
chmod 700 ~/.config/wavepeek

cat > ~/.config/wavepeek/github.maintainer.env <<'EOF'
GH_TOKEN=github_pat_REPLACE_WITH_REAL_TOKEN
GITHUB_TOKEN=github_pat_REPLACE_WITH_REAL_TOKEN
WAVEPEEK_GITHUB_ROLE=maintainer
EOF

chmod 600 ~/.config/wavepeek/github.maintainer.env
```

Activate it for trusted local work:

```bash
ln -sf github.maintainer.env ~/.config/wavepeek/github.env
```

Then recreate, rebuild, or reopen the devcontainer.

Inside the recreated container, validation should work for a token with sufficient access:

```bash
gh repo view "$WAVEPEEK_UPSTREAM_REPO"
gh pr list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
gh issue list -R "$WAVEPEEK_UPSTREAM_REPO" --limit 5
```

Git HTTPS operations against `https://github.com/kleverhq/wavepeek.git` can use the token through a repo-local credential helper. The helper stores shell logic in `.git/config`, not the token value, and only returns credentials for the configured upstream repository path.

## Disabling credentials for external PR review

Treat code from fork PRs as untrusted. Before reviewing or running an external PR in the devcontainer, switch back to the empty env file on the host:

```bash
ln -sf github.empty.env ~/.config/wavepeek/github.env
```

Then recreate, rebuild, or reopen the devcontainer.

Verify inside the new container:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
```

Those checks confirm that no maintainer token is present in the environment and that host GitHub CLI config is not mounted.

## Fork contributor flow

External contributors do not need a `kleverhq/wavepeek` token for normal development.

Recommended remotes for fork contributors:

```text
origin   = git@github.com:<contributor-login>/wavepeek.git
upstream = https://github.com/kleverhq/wavepeek.git
```

`.devcontainer/setup-github-auth.sh` adds or updates `upstream` when `origin` is not the upstream repository. It must not rewrite `origin`; contributors should keep pushing their branches to their own forks.

Typical flow:

```bash
git checkout -b my-change
# edit, build, test
git push -u origin my-change
```

The GitHub web UI is sufficient for opening a pull request. Contributors who already have their own GitHub CLI auth may use:

```bash
gh pr create \
  -R kleverhq/wavepeek \
  --head "<contributor-login>:my-change" \
  --base main \
  --fill
```

Do not require this CLI path.

## Verification

Syntax checks after changing the devcontainer auth setup:

```bash
bash -n .devcontainer/initialize.sh
bash -n .devcontainer/setup-github-auth.sh
python3 -m json.tool .devcontainer/devcontainer.json >/dev/null
```

No-token container checks:

```bash
test -z "${GH_TOKEN-}"
test -z "${GITHUB_TOKEN-}"
test ! -e /home/ubuntu/.config/gh/hosts.yml
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
# Remotes must not contain tokens.
git remote -v

# Git config must not contain token literals.
git config --local --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
git config --global --list --show-origin | grep -Ei 'github_pat|ghp_' && exit 1 || true
```

Documentation sanity check when changing this workflow:

```bash
rg -n '\.config/gh|GitHub CLI state|github-auth|github\.env|WAVEPEEK_UPSTREAM_REPO|GH_REPO|GH_TOKEN|GITHUB_TOKEN' \
  .devcontainer/AGENTS.md AGENTS.md docs/dev CHANGELOG.md
```

Use the search output to catch stale claims that host `~/.config/gh` is mounted or that `GH_REPO` is globally set.

## Troubleshooting

If `gh repo view "$WAVEPEEK_UPSTREAM_REPO"` fails with a token present, check that the fine-grained PAT is approved for the organization, has access to `kleverhq/wavepeek`, and has the permissions needed for the command. Some failures are expected for fork contributors using their own token without upstream write access.

If switching between maintainer and empty credentials appears to have no effect, recreate the devcontainer. Docker does not reread `--env-file` for an existing container.

If `upstream` points somewhere unexpected, inspect `origin` first:

```bash
git remote -v
```

The setup script only adds or updates `upstream` when `origin` is not `kleverhq/wavepeek`; it never rewrites `origin`.
