#!/usr/bin/env bash
set -euo pipefail

upstream_repo="${WAVEPEEK_UPSTREAM_REPO:-kleverhq/wavepeek}"
token="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

# Remove previous repo-local helpers installed by this script, if any. The
# value pattern avoids touching unrelated local credential helpers.
git config --local --unset-all \
    credential.https://github.com.helper \
    '^!wavepeek_github_credential_helper' 2>/dev/null || true

if [ -n "$token" ]; then
    # Scope Git credential lookup to repository paths. This allows the helper
    # to decide whether the requested credential is for the wavepeek upstream
    # repo.
    git config --local credential.useHttpPath true

    # The helper stores no secret in git config. It reads GH_TOKEN/GITHUB_TOKEN
    # from the process environment at credential lookup time and only returns
    # it for the configured upstream repository.
    git config --local --add credential.https://github.com.helper \
        '!wavepeek_github_credential_helper() {
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
            "$repo"|"$repo/"|"$repo.git"|"$repo.git/")
              echo username=x-access-token
              echo password="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
              ;;
          esac
        }; wavepeek_github_credential_helper'

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
