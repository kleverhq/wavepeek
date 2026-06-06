#!/usr/bin/env bash
set -eu

if [ "$#" -ne 0 ]; then
    printf '%s\n' "usage: $0" >&2
    printf '%s\n' "reads the GitHub token from the terminal or stdin" >&2
    exit 2
fi

config_dir="$HOME/.config/wavepeek-dev"
empty_env="$config_dir/github.empty.env"
maintainer_env="$config_dir/github.maintainer.env"
active_env="$config_dir/github.env"

if [ -L "$config_dir" ]; then
    printf '%s\n' "error: $config_dir must be a real directory, not a symlink" >&2
    printf '%s\n' "edit the GitHub auth env files manually." >&2
    exit 1
fi

if [ -e "$config_dir" ] && [ ! -d "$config_dir" ]; then
    printf '%s\n' "error: $config_dir exists and is not a directory" >&2
    printf '%s\n' "edit the GitHub auth env files manually." >&2
    exit 1
fi

if [ -e "$maintainer_env" ] || [ -L "$maintainer_env" ]; then
    printf '%s\n' "error: $maintainer_env already exists" >&2
    printf '%s\n' "edit the GitHub auth env files manually." >&2
    exit 1
fi

if [ -e "$empty_env" ] || [ -L "$empty_env" ]; then
    if [ ! -f "$empty_env" ] || [ -s "$empty_env" ] || [ -L "$empty_env" ]; then
        printf '%s\n' "error: $empty_env is not the default empty env file" >&2
        printf '%s\n' "edit the GitHub auth env files manually." >&2
        exit 1
    fi
fi

if [ -e "$active_env" ] || [ -L "$active_env" ]; then
    if [ ! -L "$active_env" ] || [ "$(readlink "$active_env")" != "github.empty.env" ]; then
        printf '%s\n' "error: $active_env is not the default github.empty.env symlink" >&2
        printf '%s\n' "edit the GitHub auth env files manually." >&2
        exit 1
    fi
fi

if [ -t 0 ]; then
    printf '%s' "GitHub token: " >&2
    if ! IFS= read -r -s token; then
        printf '\n%s\n' "error: failed to read GitHub token" >&2
        exit 1
    fi
    printf '\n' >&2
else
    if ! IFS= read -r token; then
        printf '%s\n' "error: failed to read GitHub token" >&2
        exit 1
    fi
fi

if [ -z "$token" ]; then
    printf '%s\n' "error: GitHub token must not be empty" >&2
    exit 1
fi

umask 077
mkdir -p "$config_dir"
chmod 700 "$config_dir"

: > "$empty_env"
{
    printf 'GH_TOKEN=%s\n' "$token"
    printf 'GITHUB_TOKEN=%s\n' "$token"
    printf 'WAVEPEEK_GITHUB_ROLE=maintainer\n'
} > "$maintainer_env"
rm -f "$active_env"
ln -s "github.maintainer.env" "$active_env"

chmod 600 \
    "$empty_env" \
    "$maintainer_env"

printf '%s\n' "ok: wrote $maintainer_env"
printf '%s\n' "ok: activated $active_env"
