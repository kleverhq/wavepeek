#!/usr/bin/env bash
set -euo pipefail

HOST_VERDI_HOME="${HOST_VERDI_HOME-}"
if [ -z "$HOST_VERDI_HOME" ] && [ "$#" -gt 0 ]; then
    HOST_VERDI_HOME="$1"
fi
if [ -z "$HOST_VERDI_HOME" ]; then
    HOST_VERDI_HOME="${VERDI_HOME-}"
fi

WAVEPEEK_STATE_DIR="$HOME/.cache/wavepeek"
WAVEPEEK_VERDI_MOUNT_SOURCE="$WAVEPEEK_STATE_DIR/verdi"

same_directory() {
    local left="$1"
    local right="$2"
    local left_real
    local right_real

    [ -d "$left" ] && [ -d "$right" ] || return 1
    left_real="$(cd "$left" && pwd -P)"
    right_real="$(cd "$right" && pwd -P)"
    [ "$left_real" = "$right_real" ]
}

replace_with_symlink() {
    local link_path="$1"
    local target_path="$2"

    if same_directory "$link_path" "$target_path"; then
        return 0
    fi

    if [ -L "$link_path" ]; then
        rm "$link_path"
    elif [ -d "$link_path" ]; then
        if [ -n "$(find "$link_path" -mindepth 1 -maxdepth 1 -print -quit)" ]; then
            printf '%s\n' \
                "error: refusing to replace non-empty Verdi mount source: $link_path" \
                "Set VERDI_HOME=$link_path if this is the intended SDK, or move/remove it manually." >&2
            exit 1
        fi
        rmdir "$link_path"
    elif [ -e "$link_path" ]; then
        printf '%s\n' \
            "error: refusing to replace non-directory Verdi mount source: $link_path" \
            "Move/remove it manually before reopening the devcontainer." >&2
        exit 1
    fi

    ln -s "$target_path" "$link_path"
}

ensure_directory_mount_source() {
    local mount_path="$1"

    if [ -L "$mount_path" ]; then
        rm "$mount_path"
    elif [ -e "$mount_path" ] && [ ! -d "$mount_path" ]; then
        printf '%s\n' \
            "error: Verdi mount source exists but is not a directory: $mount_path" \
            "Move/remove it manually before reopening the devcontainer." >&2
        exit 1
    fi

    mkdir -p "$mount_path"
}

# Create host-side mount sources up front so devcontainer bind mounts do not
# fail or create root-owned placeholders when the paths are missing.
mkdir -p \
    "$HOME/.config/opencode" \
    "$HOME/.config/gh" \
    "$HOME/.local/share/opencode" \
    "$HOME/.cache/opencode" \
    "$HOME/.claude" \
    "$HOME/.codex" \
    "$HOME/.pi" \
    "$WAVEPEEK_STATE_DIR"

# Claude Code may keep top-level state/config here; bind mounts need the file to exist.
if [ ! -e "$HOME/.claude.json" ]; then
    printf '{}\n' > "$HOME/.claude.json"
fi

if [ -n "$HOST_VERDI_HOME" ] && [ -d "$HOST_VERDI_HOME" ]; then
    replace_with_symlink "$WAVEPEEK_VERDI_MOUNT_SOURCE" "$HOST_VERDI_HOME"
else
    ensure_directory_mount_source "$WAVEPEEK_VERDI_MOUNT_SOURCE"
fi
