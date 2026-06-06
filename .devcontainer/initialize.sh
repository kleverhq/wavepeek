#!/usr/bin/env bash
set -euo pipefail

# Host-side Verdi SDK path. Inside the container VERDI_HOME is /opt/verdi.
HOST_VERDI_HOME="${VERDI_HOME-}"

WAVEPEEK_DEV_CONFIG_DIR="$HOME/.config/wavepeek-dev"
WAVEPEEK_VERDI_MOUNT_SOURCE="$WAVEPEEK_DEV_CONFIG_DIR/verdi"
WAVEPEEK_GITHUB_EMPTY_ENV="$WAVEPEEK_DEV_CONFIG_DIR/github.empty.env"
WAVEPEEK_GITHUB_ENV="$WAVEPEEK_DEV_CONFIG_DIR/github.env"

warn() {
    printf 'warning: %s\n' "$*" >&2
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

path_exists() {
    [ -e "$1" ] || [ -L "$1" ]
}

is_empty_dir() {
    [ -d "$1" ] && [ -z "$(find "$1" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]
}

is_expected_kind() {
    path="$1"
    kind="$2"

    case "$kind" in
        dir) [ -d "$path" ] ;;
        file) [ -f "$path" ] ;;
        *)
            printf '%s\n' "error: unknown mount source kind: $kind" >&2
            exit 2
            ;;
    esac
}

is_default_file() {
    path="$1"

    [ -f "$path" ] || return 1
    [ ! -s "$path" ] && return 0
    [ "$(tr -d '[:space:]' < "$path")" = '{}' ]
}

ensure_managed_placeholder() {
    managed_path="$1"
    kind="$2"

    if is_expected_kind "$managed_path" "$kind"; then
        return 0
    fi

    if [ -L "$managed_path" ]; then
        die "$managed_path is an invalid symlink for a $kind mount source; fix or remove it manually"
    fi

    if path_exists "$managed_path"; then
        die "$managed_path exists but is not a $kind; fix or remove it manually"
    fi

    case "$kind" in
        dir)
            mkdir -p "$managed_path"
            ;;
        file)
            printf '{}\n' > "$managed_path"
            chmod 600 "$managed_path" 2>/dev/null || true
            ;;
    esac
}

prepare_managed_for_legacy_link() {
    managed_path="$1"
    kind="$2"
    legacy_path="$3"

    if ! path_exists "$managed_path"; then
        return 0
    fi

    if [ -L "$managed_path" ]; then
        if is_expected_kind "$managed_path" "$kind"; then
            return 1
        fi
        die "$managed_path is an invalid symlink for a $kind mount source; fix or remove it manually"
    fi

    if [ "$kind" = "dir" ] && [ -d "$managed_path" ] && is_empty_dir "$managed_path"; then
        rmdir "$managed_path"
        return 0
    fi

    if [ "$kind" = "file" ] && [ -f "$managed_path" ] && is_default_file "$managed_path"; then
        rm "$managed_path"
        return 0
    fi

    warn "$managed_path already exists; not replacing it with a link to $legacy_path"
    return 1
}

prepare_agent_mount_source() {
    name="$1"
    legacy_path="$2"
    kind="$3"
    managed_path="$WAVEPEEK_DEV_CONFIG_DIR/$name"

    if path_exists "$legacy_path"; then
        if is_expected_kind "$legacy_path" "$kind"; then
            if prepare_managed_for_legacy_link "$managed_path" "$kind" "$legacy_path"; then
                ln -s "$legacy_path" "$managed_path"
            elif ! is_expected_kind "$managed_path" "$kind"; then
                ensure_managed_placeholder "$managed_path" "$kind"
            fi
        else
            warn "ignoring $legacy_path because it is not a $kind"
            ensure_managed_placeholder "$managed_path" "$kind"
        fi
    else
        ensure_managed_placeholder "$managed_path" "$kind"
    fi
}

resolve_dir() {
    (cd "$1" 2>/dev/null && pwd -P)
}

same_dir() {
    first="$(resolve_dir "$1")" || return 1
    second="$(resolve_dir "$2")" || return 1
    [ "$first" = "$second" ]
}

prepare_verdi_mount_source() {
    if [ -n "$HOST_VERDI_HOME" ] && [ -d "$HOST_VERDI_HOME" ]; then
        if path_exists "$WAVEPEEK_VERDI_MOUNT_SOURCE" && [ -d "$WAVEPEEK_VERDI_MOUNT_SOURCE" ] && same_dir "$WAVEPEEK_VERDI_MOUNT_SOURCE" "$HOST_VERDI_HOME"; then
            return 0
        fi

        if [ -L "$WAVEPEEK_VERDI_MOUNT_SOURCE" ]; then
            rm "$WAVEPEEK_VERDI_MOUNT_SOURCE"
        elif path_exists "$WAVEPEEK_VERDI_MOUNT_SOURCE"; then
            if [ -d "$WAVEPEEK_VERDI_MOUNT_SOURCE" ]; then
                if is_empty_dir "$WAVEPEEK_VERDI_MOUNT_SOURCE"; then
                    rmdir "$WAVEPEEK_VERDI_MOUNT_SOURCE"
                else
                    warn "$WAVEPEEK_VERDI_MOUNT_SOURCE is non-empty; not replacing it with $HOST_VERDI_HOME"
                    return 0
                fi
            else
                die "$WAVEPEEK_VERDI_MOUNT_SOURCE exists but is not a directory; fix or remove it manually"
            fi
        fi

        ln -s "$HOST_VERDI_HOME" "$WAVEPEEK_VERDI_MOUNT_SOURCE"
    else
        if [ -L "$WAVEPEEK_VERDI_MOUNT_SOURCE" ]; then
            rm "$WAVEPEEK_VERDI_MOUNT_SOURCE"
        fi

        if path_exists "$WAVEPEEK_VERDI_MOUNT_SOURCE"; then
            if [ -d "$WAVEPEEK_VERDI_MOUNT_SOURCE" ]; then
                return 0
            fi
            die "$WAVEPEEK_VERDI_MOUNT_SOURCE exists but is not a directory; fix or remove it manually"
        fi

        mkdir -p "$WAVEPEEK_VERDI_MOUNT_SOURCE"
    fi
}

# Create host-side mount sources up front so devcontainer bind mounts do not
# fail or create root-owned placeholders when the paths are missing.
if [ -L "$WAVEPEEK_DEV_CONFIG_DIR" ]; then
    die "$WAVEPEEK_DEV_CONFIG_DIR must be a real directory, not a symlink"
elif path_exists "$WAVEPEEK_DEV_CONFIG_DIR" && [ ! -d "$WAVEPEEK_DEV_CONFIG_DIR" ]; then
    die "$WAVEPEEK_DEV_CONFIG_DIR exists but is not a directory"
fi
mkdir -p "$WAVEPEEK_DEV_CONFIG_DIR"
chmod 700 "$WAVEPEEK_DEV_CONFIG_DIR" 2>/dev/null || true

# Optional GitHub auth env-file used by the dev container. This is
# intentionally outside the repository and empty by default.
if [ -L "$WAVEPEEK_GITHUB_EMPTY_ENV" ]; then
    die "$WAVEPEEK_GITHUB_EMPTY_ENV must be a regular empty file, not a symlink"
elif path_exists "$WAVEPEEK_GITHUB_EMPTY_ENV"; then
    if [ ! -f "$WAVEPEEK_GITHUB_EMPTY_ENV" ]; then
        die "$WAVEPEEK_GITHUB_EMPTY_ENV exists but is not a regular file"
    elif [ -s "$WAVEPEEK_GITHUB_EMPTY_ENV" ]; then
        die "$WAVEPEEK_GITHUB_EMPTY_ENV is not empty; fix GitHub env files manually"
    fi
else
    : > "$WAVEPEEK_GITHUB_EMPTY_ENV"
fi
chmod 600 "$WAVEPEEK_GITHUB_EMPTY_ENV" 2>/dev/null || true

if [ ! -e "$WAVEPEEK_GITHUB_ENV" ] && [ ! -L "$WAVEPEEK_GITHUB_ENV" ]; then
    ln -s "github.empty.env" "$WAVEPEEK_GITHUB_ENV"
elif [ ! -f "$WAVEPEEK_GITHUB_ENV" ]; then
    die "$WAVEPEEK_GITHUB_ENV must resolve to a regular env file"
fi

prepare_agent_mount_source "claude" "$HOME/.claude" dir
prepare_agent_mount_source "claude.json" "$HOME/.claude.json" file
prepare_agent_mount_source "codex" "$HOME/.codex" dir
prepare_agent_mount_source "pi" "$HOME/.pi" dir
prepare_verdi_mount_source
